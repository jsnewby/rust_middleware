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

use epoch;
use epoch::*;
use models::*;
//use super::schema::transactions;
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

    pub fn in_fork(height: i64) {
        let connection = epoch::establish_sql_connection();
        connection.execute("DELETE FROM key_blocks where height >= $1", &[&height]);
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
        let mut height = top_block_chain.height;
        loop {
            if height <= top_block_db {
                break;
            }
            if !KeyBlock::height_exists(&connection.get().unwrap(), height) {
                debug!("Fetching block {}", height);
                match _tx.send(height) {
                    Ok(x) => debug!("Success: {:?}", x),
                    Err(e) => error!("Error fetching block at height {}: {:?}", height, e),
                };
            } else {
                info!("Block already in DB at height {}", height);
            }
            height -= 1;
        }
    }

    /*
     * At this height, load the key block, using the generations call
     * to grab the block and all of its microblocks.
     */    
    fn load_blocks(&self, height: i64) -> Result<i32, Box<std::error::Error>> {
        let mut count = 0;
        let connection = self.connection.get()?;
        let mut generation: JsonGeneration = serde_json::from_value(
            self.epoch.get_generation_at_height(height as i32)?)?;
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
