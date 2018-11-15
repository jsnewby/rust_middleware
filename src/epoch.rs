use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use std;
use std::env;

use curl::easy::Easy;

use models::InsertableKeyBlock;
use models::InsertableTransaction;
use models::InsertableMicroBlock;
use models::JsonKeyBlock;
use models::JsonTransaction;
use models::JsonTransactionList;

use serde_json;
use serde_json::Value;

use r2d2::{Pool, };
use r2d2_diesel::ConnectionManager;

use regex::Regex;

use std::sync::{Arc, };

pub fn establish_connection() -> Arc<Pool<ConnectionManager<PgConnection>>> {
    dotenv().ok(); // Grabbing ENV vars

    // Pull DATABASE_URL env var
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");


    // Create a connection pool manager for a Postgres connection at the `database_url`
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    // Create the pool with the default config and the r2d2_diesel connection manager
    Arc::new(Pool::new(manager).expect("Failed to create pool."))
} 
sql_function!(fn currval(x: VarChar) -> BigInt);

pub fn get_insert_id(conn: &PgConnection, table_name: String) ->
    Result<i64, Box<std::error::Error + 'static >> {
        use diesel::dsl::{select};
        let id = select(currval(format!("{}_id_seq", table_name))).get_result::<i64>(conn)?;
        Ok(id)
    }


pub struct Epoch {
    base_uri: String,
}

impl Epoch {
    pub fn new(base_url: String) -> Epoch {
        Epoch { base_uri: base_url } }

    pub fn current_generation(&self) ->
        Result<serde_json::Value, Box<std::error::Error>> {
            self.get(&String::from("/generations/current"))
    }

    pub fn get(&self, operation: &String) -> Result<serde_json::Value, Box<std::error::Error>> {
        self.get_naked(&String::from("/v2/"), operation)
    }
    
    // Get a URL, and parse the JSON returned.
    pub fn get_naked(&self, prefix: &String, operation: &String) -> Result<serde_json::Value, Box<std::error::Error>> {
        let uri = self.base_uri.clone() + prefix  + operation;
        println!("{}", uri);
            let mut data = Vec::new();
            let mut handle = Easy::new();
            handle.url(&uri)?;
            {
                let mut transfer = handle.transfer();
                transfer.write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                })?;
                transfer.perform()?;
            }
		
        let value: Value = serde_json::from_str(std::str::from_utf8(&data)?)?;
        Ok(value)
    }

    fn get_key_block_by_hash(&self, hash: &String) ->
        Result<serde_json::Value, Box<std::error::Error>> {
            let result =
                self.get(&format!("{}{}", String::from("/key-blocks/hash/"),&hash))?;
            Ok(result)
        }

    fn get_micro_block_by_hash(&self, hash: &String) ->
        Result<serde_json::Value, Box<std::error::Error>> {
            let result =
                self.get(&format!("{}{}{}",
                                  String::from("/micro-blocks/hash/"),
                                  &hash,
                                  String::from("/header")))?;
            Ok(result)
        }

    fn get_transaction_list_by_micro_block(&self, hash: &String) ->
        Result<serde_json::Value, Box<std::error::Error>> {
            let result =
                self.get(&format!("{}{}{}",
                                  String::from("/micro-blocks/hash/"),
                                  &hash,
                                  String::from("/transactions")))?;
            Ok(result)
                
        }

}

pub fn from_json(val: &String) -> String {
    let re = Regex::new("^\"(.*)\"$").unwrap();
    match re.captures(val) {
        Some(matches) => {
            println!("Match: {:?}", String::from(&matches[1]));
            String::from(&matches[1])
        }
        None => val.clone()
    }
}

fn key_block_from_json(json: Value) -> Result<JsonKeyBlock, Box<std::error::Error>> {
    let block: JsonKeyBlock = serde_json::from_value(json)?;
    Ok(block)
}

fn micro_block_from_json(json: Value) -> Result<InsertableMicroBlock, Box<std::error::Error>> {
    let block: InsertableMicroBlock = serde_json::from_value(json)?;
    Ok(block)
}

fn transaction_from_json(json: Value) -> Result<JsonTransaction, Box<std::error::Error>> {
    let transaction: JsonTransaction = serde_json::from_value(json)?;
    Ok(transaction)
}

/*
Walk backward through the chain, grabbing the transactions and stash them in the DB.

The strings that this function returns are meaningless.
*/
pub fn populate_db(connection: &PgConnection, epoch: &Epoch, top_hash: String) -> Result<String,
                                                                  Box<std::error::Error>> {
    let mut next_hash = top_hash;
    loop  {
        if next_hash == "null" {
            break;
        }
        let result = epoch.get_key_block_by_hash(&next_hash);
        match result {
            Ok(block) => {
                let newblock = key_block_from_json(block).unwrap(); // TODO: handle error better
                next_hash = newblock.prev_key_hash.clone();
                let ib: InsertableKeyBlock = InsertableKeyBlock::from_json_key_block(&newblock)?;
                ib.save(connection)?;
                let key_block_id = get_insert_id(connection, String::from("key_blocks"))? as i32;
                let mut prev = newblock.prev_hash;
                while str::eq(&prev[0..1], "m") { // loop until we run out of microblocks
                    let mut mb = micro_block_from_json(epoch.get_micro_block_by_hash(&prev)?)?;
                    mb.key_block_id = key_block_id;
                    mb.save(connection).unwrap();
                    let micro_block_id = get_insert_id(connection, String::from("micro_blocks"))? as i32;
                    let mut trans: JsonTransactionList =
                        serde_json::from_value(epoch.get_transaction_list_by_micro_block(&prev)?)?;
                    for i in 0 .. trans.transactions.len() {
                        let jtx: &JsonTransaction = &trans.transactions[i];
                        let tx_type: String = from_json(&serde_json::to_string(&jtx.tx["type"])?);
                        let tx: InsertableTransaction =
                            InsertableTransaction::from_json_transaction(jtx, tx_type, micro_block_id)?;
                        tx.save(connection)?;
                    }
                    prev = mb.prev_hash;
                }
            }
            Err(_) => {
                break;
            }
        }
    }
    Ok(String::from("AOK"))
}               

