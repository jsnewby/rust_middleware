#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

use diesel::pg::PgConnection;
use std::sync::{Arc, };
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use epoch;
use epoch::*;
use models::*;

use serde_json;

use r2d2::{Pool, };
use r2d2_diesel::ConnectionManager;
pub struct BlockLoader {
    epoch: Epoch,
    pub connection: Arc<Pool<ConnectionManager<PgConnection>>>, // DB connection
    rx: std::sync::mpsc::Receiver<i64>,
    pub tx: std::sync::mpsc::Sender<i64>,
}

impl BlockLoader {
    pub fn new(connection: Arc<Pool<ConnectionManager<PgConnection>>>,
           epoch_url: String)
                           -> BlockLoader {
        let(tx, rx) : (Sender<i64>, Receiver<i64>) = mpsc::channel();
        let epoch = Epoch::new(epoch_url.clone());
        BlockLoader {
            epoch: epoch,
            connection: connection,
            rx: rx,
            tx: tx,
        }
    }

    pub fn scan(epoch: &Epoch, tx: &std::sync::mpsc::Sender<i64> ) {
        let connection = epoch::establish_connection();
        let top_block_chain =
            key_block_from_json(epoch.latest_key_block().
                                unwrap()).unwrap();
        let top_block_db = KeyBlock::top_height(&connection.
                                                get().unwrap()).unwrap();
        println!("Reading blocks {} to {}",
                 top_block_chain.height, top_block_db);
        let mut height = top_block_chain.height;
        loop {
            if height == top_block_db {
                break;
            }
            if ! KeyBlock::height_exists(&connection.get().unwrap(),
                                         height) {
                println!("Fetching block {}", height);
                match tx.send(height) {
                    Ok(x) => println!("Success: {:?}", x),
                    Err(e) => println!("Error: {:?}", e),
                    };
            } else {
                println!("Block already in DB at height {}", height);
            }
            height = height - 1;
        }
    }

    fn load_blocks(&self, height: i64) -> Result<String, Box<std::error::Error>> {
        let connection = self.connection.get()?;
        let kb = self.epoch.get_key_block_by_height(height)?;
        let newblock = key_block_from_json(kb)?;
        let ib: InsertableKeyBlock = InsertableKeyBlock::from_json_key_block(&newblock)?;
        ib.save(&connection)?;
        let key_block_id = get_insert_id(&connection, String::from("key_blocks"))? as i32;
        let mut prev = newblock.prev_hash;
        while str::eq(&prev[0..1], "m") { // loop until we run out of microblocks
            let mut mb = micro_block_from_json(self.epoch.get_micro_block_by_hash(&prev)?)?;
            mb.key_block_id = key_block_id;
            mb.save(&connection).unwrap();
            let micro_block_id = get_insert_id(&connection, String::from("micro_blocks"))? as i32;
            let trans: JsonTransactionList =
                serde_json::from_value(self.epoch.get_transaction_list_by_micro_block(&prev)?)?;
            for i in 0 .. trans.transactions.len() {
                let jtx: &JsonTransaction = &trans.transactions[i];
                let tx_type: String = from_json(&serde_json::to_string(&jtx.tx["type"])?);
                let tx: InsertableTransaction =
                    InsertableTransaction::from_json_transaction(jtx, tx_type, micro_block_id)?;
                tx.save(&connection)?;
            }
            prev = mb.prev_hash;
        }
        Ok(String::from("foo"))
    }

    pub fn start(&self) {
        for b in &self.rx {
            self.load_blocks(b);
        }
    }
}
