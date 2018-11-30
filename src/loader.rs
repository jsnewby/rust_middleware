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
            self.store_or_update_transaction(&conn, &trans.transactions[i], None);
            hashes_in_mempool.push(format!("'{}'", trans.transactions[i].hash));
        }
        let sql = format!(
            "UPDATE transactions SET VALID=FALSE WHERE \
             micro_block_id IS NULL AND \
             hash NOT IN ({})",
            hashes_in_mempool.join(", "));
        println!("__sql: {}", sql);
    }        

    pub fn scan(epoch: &Epoch, _tx: &std::sync::mpsc::Sender<i64>) {
        let connection = epoch::establish_connection();
        let top_block_chain = key_block_from_json(epoch.latest_key_block().unwrap()).unwrap();
        let top_block_db = KeyBlock::top_height(&connection.get().unwrap()).unwrap();
        if top_block_chain.height == top_block_db {
            println!("Up-to-date");
            return;
        }
        if top_block_chain.height < top_block_db {
            println!("Fork detected");
            //in_fork();
            return;
        }
        println!(
            "Reading blocks {} to {}",
            top_block_db+1, top_block_chain.height
        );
        let mut height = top_block_chain.height;
        loop {
            if height <= top_block_db {
                break;
            }
            if !KeyBlock::height_exists(&connection.get().unwrap(), height) {
                println!("Fetching block {}", height);
                match _tx.send(height) {
                    Ok(x) => println!("Success: {:?}", x),
                    Err(e) => println!("Error: {:?}", e),
                };
            } else {
                println!("Block already in DB at height {}", height);
            }
            height -= 1;
        }
    }

    fn load_blocks(&self, height: i64) -> Result<String, Box<std::error::Error>> {
        let connection = self.connection.get()?;
        let mut kb = self.epoch.get_key_block_by_height(height)?; 
        let newblock = key_block_from_json(kb)?;
        let ib: InsertableKeyBlock = InsertableKeyBlock::from_json_key_block(&newblock)?;
        let key_block_id = ib.save(&connection)? as i32;
        let mut prev = newblock.prev_hash;
        // we're currently (naively) getting transactions working back from the latest mined keyblock
        // so this is necessary in order to get the keyblock to which these microblocks relate
        let last_key_block = match KeyBlock::load_at_height(&connection, height - 1) {
            Some(x) => x,
            None => return Ok(String::from("Block not found")),
        };
        while str::eq(&prev[0..1], "m") {
            // loop until we run out of microblocks
            let mut mb = micro_block_from_json(self.epoch.get_micro_block_by_hash(&prev)?)?;
            mb.key_block_id = last_key_block.id;
            println!("Inserting micro w. key_block_id={}", key_block_id);
            let _micro_block_id = mb.save(&connection).unwrap() as i32;
            let trans: JsonTransactionList =
                serde_json::from_value(self.epoch.get_transaction_list_by_micro_block(&prev)?)?;
            for i in 0..trans.transactions.len() {
                self.store_or_update_transaction(&connection, &trans.transactions[i],
                                                 Some(_micro_block_id)).unwrap();
            }
            prev = mb.prev_hash;
        }
        Ok(String::from("foo"))
    }

    pub fn store_or_update_transaction(&self, conn: &PgConnection,
                                       trans: &JsonTransaction,
                                       _micro_block_id: Option<i32>
    ) ->
        Result<i32, Box<std::error::Error>>
    {
        let mut results: Vec<Transaction> = sql_query(
            "select * from transactions where hash = '?' limit 1").
            bind::<diesel::sql_types::Text, _>(trans.hash.clone()).
            get_results(conn)?;
        match results.pop() {
            Some(x) => {
                println!("Found {}", &trans.hash);
                diesel::update(&x).set(micro_block_id.eq(_micro_block_id));
                Ok(x.id)
            },
            None => {
                println!("Not found {}", &trans.hash);
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
