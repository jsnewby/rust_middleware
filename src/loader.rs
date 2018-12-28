use diesel::pg::PgConnection;
use diesel::query_dsl::QueryDsl;
use diesel::sql_query;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use std::slice::SliceConcatExt;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;

use super::schema::key_blocks::dsl::*;
use super::schema::transactions::dsl::*;
use epoch;
use epoch::*;
use models::*;

use serde_json;

use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
pub struct BlockLoader {
    epoch: Epoch,
    pub connection: Arc<Pool<ConnectionManager<PgConnection>>>, // DB connection
    rx: std::sync::mpsc::Receiver<i64>,
    pub tx: std::sync::mpsc::Sender<i64>,
}

/*
* You may notice the use of '_tx' as a variable name in this file,
* rather than the more natural 'tx'. This is because the macros which
* diesel generates from the code in models.rs cause every occurrence
* of their field names to be replaced, which obviously causes
* problems.
*/

impl BlockLoader {
    /*
     * Makes a new BlockLoader object, initializes its DB pool.
     */
    pub fn new(
        connection: Arc<Pool<ConnectionManager<PgConnection>>>,
        epoch_url: String,
    ) -> BlockLoader {
        let (_tx, rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
        let epoch = Epoch::new(epoch_url.clone(), 1);
        BlockLoader {
            epoch,
            connection,
            rx,
            tx: _tx,
        }
    }

    /*
     * We walk backward through the chain loading generations from the
     * DB, and requesting them from the chain. We pause 1 second
     * between each check, and only check 500 blocks (~1 day)
     * back. For each pair of blocks we compare them using their eq()
     * mehods. If false we delete the block from the DB (which
     * cascades to delete the microblocks and transactions), and put
     * the height onto the load queue.
     *
     * TODO: disassociate the TXs from the micro-blocks and keep them
     * for reporting purposes.
     */
    pub fn detect_forks(epoch: &Epoch, from: i64, to: i64, _tx: &std::sync::mpsc::Sender<i64>) {
        let conn = epoch.get_connection().unwrap();
        let mut _height = KeyBlock::top_height(&conn).unwrap() - from;
        let stop_height = _height - to;
        loop {
            if _height <= stop_height {
                break;
            }
            let jg: JsonGeneration = match JsonGeneration::get_generation_at_height(&conn, _height)
            {
                Some(x) => x,
                None => {
                    info!("Couldn't load generation {} from DB", _height);
                    _height -= 1;
                    continue;
                }
            };
            let gen_from_server: JsonGeneration =
                serde_json::from_value(epoch.get_generation_at_height(_height).unwrap()).unwrap();
            if !jg.eq(&gen_from_server) {
                info!("Invalidating block at height {}", _height);
                BlockLoader::invalidate_block_at_height(_height, &conn, &_tx);
                _height -= 1;
                continue;
            }
            debug!("Block checks out at height {}", _height);
            _height -= 1; // only sleep if no fork found.
            thread::sleep(std::time::Duration::new(2, 0));
        }
    }

    /*
     * Delete a key block at height (which causes the deletion of the
     * associated micro blocks and transaactions, and then adds the
     * height to the queue of heights to be loaded.
     */

    pub fn invalidate_block_at_height(
        _height: i64,
        conn: &PgConnection,
        _tx: &std::sync::mpsc::Sender<i64>,
    ) {
        debug!("Invalidating block at height {}", _height);
        match _tx.send(_height) {
            Ok(()) => (),
            Err(e) => {
                error!("Error queuing block at height {}: {:?}", _height, e);
                BlockLoader::recover_from_db_error();
            }
        };
        diesel::delete(key_blocks.filter(height.eq(&_height)))
            .execute(conn)
            .unwrap();
    }

    /*
     * Load the mempool from the node--this will only be possible if
     * the debug endpoint is also available on the main URL. Otherwise
     * it harmlessly explodes.
     */
    pub fn load_mempool(&self, epoch: &Epoch) {
        let conn = epoch.get_connection().unwrap();
        let trans: JsonTransactionList =
            serde_json::from_value(self.epoch.get_pending_transaction_list().unwrap()).unwrap();
        let mut hashes_in_mempool = vec![];
        for i in 0..trans.transactions.len() {
            match self.store_or_update_transaction(&conn, &trans.transactions[i], None) {
                Ok(_) => (),
                Err(x) => error!(
                    "Failed to insert transaction {} with error {}",
                    trans.transactions[i].hash, x
                ),
            }
            hashes_in_mempool.push(format!("'{}'", trans.transactions[i].hash));
        }
        let sql = format!(
            "UPDATE transactions SET VALID=FALSE WHERE \
             micro_block_id IS NULL AND \
             hash NOT IN ({})",
            hashes_in_mempool.join(", ")
        );
        epoch::establish_sql_connection().execute(&sql, &[]);
    }

    /*
     * this method scans the blocks from the heighest reported by the
     * node to the highest in the DB, filling in the gaps.
     */
    pub fn scan(epoch: &Epoch, _tx: &std::sync::mpsc::Sender<i64>) {
        let connection = epoch.get_connection().unwrap();
        let top_block_chain = key_block_from_json(epoch.latest_key_block().unwrap()).unwrap();
        let top_block_db = KeyBlock::top_height(&connection).unwrap();
        if top_block_chain.height == top_block_db {
            trace!("Up-to-date");
            return;
        }
        debug!(
            "Reading blocks {} to {}",
            top_block_db + 1,
            top_block_chain.height
        );
        let mut _height = top_block_chain.height;
        loop {
            if _height <= top_block_db {
                break;
            }
            if !KeyBlock::height_exists(&connection, _height) {
                debug!("Fetching block {}", _height);
                match _tx.send(_height) {
                    Ok(x) => debug!("Success: {:?}", x),
                    Err(e) => {
                        error!("Error queuing block at height {}: {:?}", _height, e);
                        BlockLoader::recover_from_db_error();
                    }
                };
            } else {
                info!("Block already in DB at height {}", _height);
            }
            _height -= 1;
        }
    }

    /*
     * In fact there is no recovery currently--we just exit, and someone else can restart
     * the process.
     */
    pub fn recover_from_db_error() {
        error!("Quitting...");
        ::std::process::exit(-1);
    }

    /*
     * At this height, load the key block, using the generations call
     * to grab the block and all of its microblocks.
     */

    fn load_blocks(&self, _height: i64) -> Result<i32, Box<std::error::Error>> {
        let mut count = 0;
        let connection = self.connection.get()?;
        let generation: JsonGeneration =
            serde_json::from_value(self.epoch.get_generation_at_height(_height)?)?;
        let ib: InsertableKeyBlock =
            InsertableKeyBlock::from_json_key_block(&generation.key_block)?;
        let key_block_id = ib.save(&connection)? as i32;
        for mb_hash in &generation.micro_blocks {
            let mut mb: InsertableMicroBlock =
                serde_json::from_value(self.epoch.get_micro_block_by_hash(&mb_hash)?)?;
            mb.key_block_id = Some(key_block_id);
            let _micro_block_id = mb.save(&connection)? as i32;
            let trans: JsonTransactionList =
                serde_json::from_value(self.epoch.get_transaction_list_by_micro_block(&mb_hash)?)?;
            for i in 0..trans.transactions.len() {
                self.store_or_update_transaction(
                    &connection,
                    &trans.transactions[i],
                    Some(_micro_block_id),
                )?;
            }
            count += 1;
        }
        Ok(count)
    }

    /*
     * transactions in the mempool won't have a micro_block_id, so as we scan the chain we may
     * need to insert them, or update them with the id of the micro block with which they're
     * now associated. We may also need to move them to a different micro block, in the event
     * of a fork.
     */
    pub fn store_or_update_transaction(
        &self,
        conn: &PgConnection,
        trans: &JsonTransaction,
        _micro_block_id: Option<i32>,
    ) -> Result<i32, Box<std::error::Error>> {
        let sql = format!(
            "select * from transactions where hash = '{}' limit 1",
            &trans.hash
        );
        debug!("{}", sql);
        let mut results: Vec<Transaction> = sql_query(sql).
            // bind::<diesel::sql_types::Text, _>(trans.hash.clone()).
            get_results(conn)?;
        match results.pop() {
            Some(x) => {
                debug!("Updating transaction with hash {}", &trans.hash);
                diesel::update(&x).set(micro_block_id.eq(_micro_block_id));
                Ok(x.id)
            }
            None => {
                debug!("Inserting transaction with hash {}", &trans.hash);
                let _tx_type: String = from_json(&serde_json::to_string(&trans.tx["type"])?);
                let _tx: InsertableTransaction = InsertableTransaction::from_json_transaction(
                    &trans,
                    _tx_type,
                    _micro_block_id,
                )?;
                _tx.save(conn)
            }
        }
    }

    /*
     * The very simple function which pulls heights from the queue and
     * loads them into the DB
     */
    pub fn start(&self) {
        for b in &self.rx {
            debug!("Pulling height {} from queue for storage", b);
            match self.load_blocks(b) {
                Ok(x) => info!("Loaded {} in total", x),
                Err(x) => error!("Error loading blocks {}", x),
            };
        }
        // if we fall through here something has gone wrong. Let's quit!
        error!("Failed to read from the queue, quitting.");
        BlockLoader::recover_from_db_error();
    }

    /*
     * Verify the DB, printing results to stdout.
     *
     * methodology: 
     * pick the highest of DB height and height reported by node
     * load blocks from DB and from node
     * compare the microblocks and transactions in each
     * for each, log with - if missing from DB, + if present in DB but not on chain
     * - key block
     * - micro blocks
     * - transactions
     */
    pub fn verify(&self) {
        let top_chain = self.epoch.latest_key_block().unwrap()["height"].as_i64().unwrap();
        let conn = self.epoch.get_connection().unwrap();
        let top_db = KeyBlock::top_height(&conn).unwrap();
        let top_max = std::cmp::max(top_chain,top_db);
        let mut i = top_max;
        loop {
            self.compare_chain_and_db(i, &conn);
            i -= 1;
        }
    }

    pub fn compare_chain_and_db(&self, _height: i64, conn: &PgConnection) {
        let block_chain = self.epoch.get_key_block_by_height(_height);
        let block_db = KeyBlock::load_at_height(conn, _height);
        match block_chain {
            Ok(x) => {
                match block_db {
                    Some(y) => {
                        self.compare_key_blocks(conn, x, y);
                    },
                    None => {
                        println!("{} missing from DB", _height);
                    },
                };
            },
            Err(_) => {
                match block_db {
                    Some(_) => {
                        println!("{} present in DB not in chain", _height);
                    },
                    None => {
                        println!("Not found at either, something weird happened");
                    },
                };
            },
        };
    }

    pub fn compare_key_blocks(&self, conn: &PgConnection, block_chain: serde_json::Value,
                              block_db: KeyBlock) {
        let chain_hash = block_chain["hash"].as_str().unwrap();
        if ! block_db.hash.eq(&chain_hash) {
            println!("{} Hashes differ: {} chain vs {} block",
                     block_db.height, chain_hash, block_db.hash);
            return;
        }
        let mut db_mb_hashes = MicroBlock::get_microblock_hashes_for_key_block_hash(
            &String::from(chain_hash)).unwrap();
        db_mb_hashes.sort_by(|a,b| a.cmp(b));
        let chain_gen = self.epoch.get_generation_at_height(block_db.height).unwrap();
        let mut chain_mb_hashes = chain_gen["micro_blocks"].as_array().unwrap().clone();
        chain_mb_hashes.sort_by(|a,b| a.as_str().unwrap().cmp(b.as_str().unwrap()));
        if db_mb_hashes.len() != chain_mb_hashes.len() {
            println!("{} Microblock array size differs: {} chain vs {} db",
                     block_db.height, chain_mb_hashes.len(), block_db.hash.len());
            return;
        }
        let mut all_good = true;
        for i in 0 .. db_mb_hashes.len() {
            let chain_mb_hash = String::from(chain_mb_hashes[i].as_str().unwrap());
            let db_mb_hash = db_mb_hashes[i].clone();
            all_good = all_good && self.compare_micro_blocks(&conn, block_db.height, db_mb_hash, chain_mb_hash);
        }
        if all_good {
            println!("{} OK", block_db.height);
        }
    }

    pub fn compare_micro_blocks(&self, conn: &PgConnection, _height: i64, db_mb_hash: String, chain_mb_hash: String) -> bool {
        if db_mb_hash != chain_mb_hash {
            println!("{} micro block hashes don't match: {} chain vs {} db",
                     _height, chain_mb_hash, db_mb_hash);
            return false;
        }
        let mut db_transactions = Transaction::load_for_micro_block(conn, &db_mb_hash).unwrap();
        db_transactions.sort_by(|a,b| a.hash.cmp(&b.hash));
        let ct = self.epoch.get_transaction_list_by_micro_block(&chain_mb_hash).unwrap();
        let mut chain_transactions = ct["transactions"].as_array().unwrap().clone();
        chain_transactions.sort_by(|a,b| a["hash"].as_str().unwrap().cmp(b["hash"].as_str().unwrap()));
        if db_transactions.len() != chain_transactions.len() {
            println!("{} micro block {} transaction count differs: {} chain vs {} DB",
                     _height, db_mb_hash, chain_transactions.len(), db_transactions.len());
            return false;
        }
        let mut diffs: Vec<(usize, String,String)> = vec!();
        for i in 0 .. db_transactions.len() {
            let chain_tx_hash = chain_transactions[i]["hash"].as_str().unwrap();
            if db_transactions[i].hash != chain_tx_hash {
                diffs.push((i, db_transactions[i].hash.clone(), chain_tx_hash.to_string()));
            }
        }
        if diffs.len() != 0 {
            loop {
                match diffs.iter().next() {
                    Some(x) => println!("{} micro block {} transaction {} of {} hashes differ: {} chain vs {} DB",
                                        _height, db_mb_hash, x.0, diffs.len(), x.2, x.1),
                    None => break,
                }
            }
            return false;
        }
        true
    }
}
