use diesel::ExpressionMethods;
use diesel::pg::PgConnection;
use diesel::query_builder::SqlQuery;
use diesel::query_dsl::QueryDsl;
use diesel::RunQueryDsl;
use diesel::sql_query;
use std::slice::SliceConcatExt;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;

use epoch;
use epoch::*;
use models::*;
//use super::schema::transactions;
use super::schema::key_blocks;
use super::schema::key_blocks::dsl::*;
use super::schema::transactions::dsl::*;

use serde_json;

use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
pub struct BlockLoader {
    epoch: Epoch,
    pub connection: Arc<Pool<ConnectionManager<PgConnection>>>, // DB connection
    rx: std::sync::mpsc::Receiver<i64>,
    pub tx: std::sync::mpsc::Sender<i64>,
}

impl BlockLoader {
    pub fn new(
        connection: Arc<Pool<ConnectionManager<PgConnection>>>,
        epoch_url: String,
    ) -> BlockLoader {
        let (_tx, rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
        let epoch = Epoch::new(epoch_url.clone());
        BlockLoader {
            epoch, connection, rx, tx: _tx,
        }
    }

    pub fn in_fork(_height: i64) {
        let connection = epoch::establish_sql_connection();
        connection.execute("DELETE FROM key_blocks where height >= $1", &[&_height]);
    }

    /*
     * We walk backward through the chain loading blocks from the DB,
     * and requesting them from the chain. We pause 1 second between
     * each check, and only check 500 blocks (~1 day) back. For each
     * pair of blocks we check that their hashes and their
     * beneficiaries match, if now we delete the block from the DB
     * (which cascades to delete the microblocks and transactions),
     * and put the height onto the load queue.
     *
     * TODO: disassociate the TXs from the micro-blocks and keep them
     * for reporting purposes.
     */

    pub fn detect_forks(&self, epoch: &Epoch, _tx: &std::sync::mpsc::Sender<i64>) {
        let conn = epoch::establish_connection().get().unwrap();
        let mut _height = KeyBlock::top_height(&conn).unwrap();
        let stop_height = _height - 500; // a day, more or less
        loop {
            if _height <= stop_height {
                break;
            }
            let mut block_db = KeyBlock::load_at_height(&conn, _height).unwrap();
            let mut block_chain = match epoch.get_key_block_by_height(_height)
            {
                Ok(x) => x,
                Err(y) => {
                    info!("Couldn't load block from chain at height {}", _height);
                    self.invalidate_block_at_height(_height, &conn, &_tx);
                    _height -= 1;
                    continue;
                }
            }; // Block was found on chain.            
            if block_db.hash != block_chain["hash"] &&
                block_db.beneficiary != block_chain["beneficiary"] {
                self.invalidate_block_at_height(_height, &conn, &_tx);
            }
//            debug!("Block checks out at height {}", _height);
            _height -= 1;
            thread::sleep(std::time::Duration::new(2,0));
        }
    }

    fn invalidate_block_at_height(&self, _height: i64, conn: &PgConnection,
                                  _tx: &std::sync::mpsc::Sender<i64>) {
        debug!("Invalidating block at height {}", _height);
        _tx.send(_height).unwrap();
        diesel::delete(key_blocks.filter(height.eq(&_height))).execute(conn).unwrap();
    }

    pub fn load_mempool(&self, epoch: &Epoch) {
        let conn = epoch::establish_connection().get().unwrap();
        let trans: JsonTransactionList =
            serde_json::from_value(self.epoch.get_pending_transaction_list().unwrap()).unwrap();
        let mut hashes_in_mempool = vec!();
        for i in 0..trans.transactions.len() {
            self.store_or_update_transaction(&conn, &trans.transactions[i], None).unwrap();
            hashes_in_mempool.push(format!("'{}'", trans.transactions[i].hash));
        }
        let sql = format!(
            "UPDATE transactions SET VALID=FALSE WHERE \
             micro_block_id IS NULL AND \
             hash NOT IN ({})",
            hashes_in_mempool.join(", "));
        epoch::establish_sql_connection().execute(&sql, &[]);
    }        

    pub fn scan(epoch: &Epoch, _tx: &std::sync::mpsc::Sender<i64>) {
        let connection = epoch::establish_connection();
        let top_block_chain = key_block_from_json(epoch.latest_key_block().unwrap()).unwrap();
        let top_block_db = KeyBlock::top_height(&connection.get().unwrap()).unwrap();
        if top_block_chain.height == top_block_db {
            trace!("Up-to-date");
            return;
        }
        if top_block_chain.height < top_block_db {
            info!("Fork detected");
            //in_fork();
            return;
        }
        debug!(
            "Reading blocks {} to {}",
            top_block_db+1, top_block_chain.height
        );
        let mut _height = top_block_chain.height;
        loop {
            if _height <= top_block_db {
                break;
            }
            if !KeyBlock::height_exists(&connection.get().unwrap(), _height) {
                debug!("Fetching block {}", _height);
                match _tx.send(_height) {
                    Ok(x) => debug!("Success: {:?}", x),
                    Err(e) => error!("Error fetching block at height {}: {:?}", _height, e),
                };
            } else {
                info!("Block already in DB at height {}", _height);
            }
            _height -= 1;
        }
    }

    /*
     * At this height, load the key block, using the generations call
     * to grab the block and all of its microblocks.
     */    
    fn load_blocks(&self, _height: i64) -> Result<i32, Box<std::error::Error>> {
        let mut count = 0;
        let connection = self.connection.get()?;
        let mut generation: JsonGeneration = serde_json::from_value(
            self.epoch.get_generation_at_height(_height as i32)?)?;
        let ib: InsertableKeyBlock = InsertableKeyBlock::from_json_key_block(&generation.key_block)?;
        let key_block_id = ib.save(&connection)? as i32;
        for mb_hash in &generation.micro_blocks {
            let mut mb: InsertableMicroBlock = serde_json::from_value(
                self.epoch.get_micro_block_by_hash(&mb_hash)?)?;
            mb.key_block_id = Some(key_block_id);
            let _micro_block_id = mb.save(&connection).unwrap() as i32;
            let trans: JsonTransactionList =
                serde_json::from_value(self.epoch.get_transaction_list_by_micro_block(&mb_hash)?)?;
            for i in 0..trans.transactions.len() {
                self.store_or_update_transaction(&connection, &trans.transactions[i],
                                                 Some(_micro_block_id)).unwrap();
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
    pub fn store_or_update_transaction(&self, conn: &PgConnection,
                                       trans: &JsonTransaction,
                                       _micro_block_id: Option<i32>
    ) ->
        Result<i32, Box<std::error::Error>>
    {
        let sql = format!("select * from transactions where hash = '{}' limit 1", &trans.hash);
        debug!("{}", sql);
        let mut results: Vec<Transaction> = sql_query(sql).
            // bind::<diesel::sql_types::Text, _>(trans.hash.clone()).
            get_results(conn)?;
        match results.pop() {
            Some(x) => {
                debug!("Found {}", &trans.hash);
                diesel::update(&x).set(micro_block_id.eq(_micro_block_id));
                Ok(x.id)
            },
            None => {
                debug!("Not found {}", &trans.hash);
                let _tx_type: String = from_json(&serde_json::to_string(&trans.tx["type"])?);
                let _tx: InsertableTransaction =
                    InsertableTransaction::from_json_transaction(&trans, _tx_type, _micro_block_id)?;
                _tx.save(conn)
            },
        }
    }

    pub fn start(&self) {
        for b in &self.rx {
            self.load_blocks(b);
        }
    }
}
