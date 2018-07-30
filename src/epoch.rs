use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use std;
use std::env;

use models::InsertableBlock;
use models::InsertableTransaction;
use models::JsonBlock;
use models::JsonTransaction;

use serde_json;
use serde_json::Value;

use curl::easy::Easy;

use regex::Regex;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

sql_function!(fn currval(x: VarChar) -> BigInt);

pub fn get_last_block_id(conn: &PgConnection) ->
    Result<i64, Box<std::error::Error + 'static >> {
        use diesel::dsl::{select};
        let id = select(currval("blocks_id_seq")).get_result::<i64>(conn)?;
        Ok(id)
    }

pub struct Epoch {
    base_uri: String,
}

impl Epoch {
    pub fn new(base_url: String) -> Epoch {
        Epoch { base_uri: base_url } }

    pub fn top(&self) -> Result<serde_json::Value, Box<std::error::Error>> {
            self.get(&String::from("/top"))
    }

    // Get a URL, and parse the JSON returned.
    fn get(&self, operation: &String) -> Result<serde_json::Value, Box<std::error::Error>> {
        let uri = self.base_uri.clone() + "/v2" + operation;
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

    fn get_block_by_hash(&self, hash: &String) ->
        Result<serde_json::Value, Box<std::error::Error>> {
            self.get(&format!("{}{}", String::from("/block/hash/"),&hash))
        }

}

fn from_json(val: &String) -> String {
    let foo = "^\"(.*)\"$";
    // I think the below unwrap() is OK b/c if we can't compile a regexp which we know to be
    // good then something is badly wrong. But I am not sure.
    let re = Regex::new(foo).unwrap();
    match re.captures(val) {
        Some(matches) => {
            println!("Match: {:?}", String::from(&matches[1]));
            String::from(&matches[1])
        }
        None => val.clone()
    }
}

fn block_from_json(json: Value) -> Result<JsonBlock, Box<std::error::Error>> {
    let block: JsonBlock = serde_json::from_value(json)?;
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
fn populate_db(connection: &PgConnection, epoch: Epoch, top_hash: String) -> Result<String,
                                                                  Box<std::error::Error>> {
    let mut _hash = top_hash;
    loop  {
        if _hash == "null" {
            break;
        }
        let result = epoch.get_block_by_hash(&_hash);
        match result {
            Ok(block) => {
                let newblock = match block_from_json(block) {
                    Ok(newblock) => newblock,
                    Err(_) => return Ok(String::from("Rootytoot"))
                };
                _hash = newblock.prev_hash.clone();
                let ib: InsertableBlock = InsertableBlock::from_json_block(&newblock)?;
                ib.save(connection)?;
                let block_id = get_last_block_id(connection)? as i32;
                for i in 0 .. newblock.transactions.len() {
                    let jtx: &JsonTransaction = &newblock.transactions[i];
                    let tx_type: String = from_json(&serde_json::to_string(&jtx.tx["type"])?);
                    let tx: InsertableTransaction =
                        InsertableTransaction::from_json_transaction(jtx, tx_type, block_id)?;
                    tx.save(connection)?;
                }

            }
            Err(_) => {
                break;
            }
        }
    }
    Ok(String::from("AOK"))
}               

