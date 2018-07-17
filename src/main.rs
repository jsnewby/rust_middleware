#![allow(missing_docs, unused_variables, trivial_casts)]

#[allow(unused_extern_crates)]
extern crate swagger;
#[allow(unused_extern_crates)]
extern crate futures;
extern crate swagger_client;
#[allow(unused_extern_crates)]
extern crate uuid;
extern crate clap;
extern crate tokio_core;
extern crate crypto;
extern crate hex;
extern crate blake2b;
extern crate rand;

extern crate regex;
use regex::Regex;

extern crate curl;
use curl::easy::Easy;

extern crate rust_base58;
//use rust_base58::{ToBase58, FromBase58};

extern crate rust_sodium;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use serde_json::Value;
use serde_json::Value::Array;

#[allow(unused_imports)]
use futures::{Future, future, Stream, stream};
use tokio_core::reactor;

pub mod transaction;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate bigdecimal;
pub use bigdecimal::BigDecimal;

extern crate itertools;

extern crate rocket;

pub mod schema;


use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use std::env;

pub mod models;
use models::InsertableBlock;
use models::InsertableTransaction;
use models::JsonBlock;
use models::JsonTransaction;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn get_last_block_id(conn: &PgConnection) ->
    Result<i64, Box<std::error::Error + 'static >> {
        use diesel::dsl::{select};
        let id = select(currval("blocks_id_seq")).get_result::<i64>(conn)?;
        Ok(id)
    }

sql_function!(fn currval(x: VarChar) -> BigInt);

pub struct Epoch {
    client: swagger_client::client::Client,
    base_uri: String,
}

impl Epoch {
    fn new(base_url: String) -> Epoch {
        let core = reactor::Core::new().unwrap();
        let client;
        if base_url.starts_with("https://") {
            client = swagger_client::client::Client::try_new_https(&base_url, "test/ca.pem").expect("Failed to connect");
        } else {
            client = swagger_client::client::Client::try_new_http(&base_url).expect("Failed to connect");
        }
        let context = swagger_client::Context::new_with_span_id(self::uuid::Uuid::new_v4().to_string());

        Epoch { client: client, base_uri: base_url } }

    fn top(&self) -> Result<serde_json::Value, Box<std::error::Error>> {
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
    
fn populate_db(connection: &PgConnection, epoch: Epoch, top_hash: String) -> Result<String,
                                                                  Box<std::error::Error>> {
    let mut _hash = top_hash;
    loop  {
        println!("{}", _hash);
        if _hash == "null" {
            break;
        }
        let result = epoch.get_block_by_hash(&_hash);
        match result {
            Ok(block) => {
                let newblock = block_from_json(block)?;
                _hash = newblock.prev_hash.clone();
                let ib: InsertableBlock = InsertableBlock::from_json_block(&newblock)?;
                ib.save(connection);
                let block_id = get_last_block_id(connection)? as i32;
                for i in 0 .. newblock.transactions.len() {
                    let jtx: &JsonTransaction = &newblock.transactions[i];
                    let tx_type: String = serde_json::to_string(&jtx.tx["type"])?.clone();
                    let tx: InsertableTransaction =
                        InsertableTransaction::from_json_transaction(jtx, tx_type, block_id)?;
                    tx.save(connection);
                }

            }
            Err(_) => {
                break;
            }
        }
    }
    Ok(String::from("AOK"))
}               

fn main() {
    let connection = establish_connection();
    
    let epoch = Epoch::new(String::from("http://localhost:3013"));
    println!("Top: {:?}", epoch.top());
    let top_response = epoch.top().unwrap();
    let mut top_hash = from_json(&top_response["hash"].to_string());
    populate_db(&connection, epoch, top_hash);

    
    
}

#[cfg(test)]
mod tests {
    use transaction::KeyPair;
    use get_last_block_id;
    #[test]
    fn test_read_sign_verify() {
        // Read a key pair from a file (these were generated by the JS
        // SDK so this also tests ineroperability. Sign and check
        // verification works
        let key_pair = KeyPair::read_from_files(&String::from("test/keys/testkey.pub"),
                                                &String::from("test/keys/testkey"),
                                                &String::from(""));
        let msg = b"this is a test thing";
        let mut bytes = key_pair.sign(msg).unwrap();
        println!("Sig: {:?}", KeyPair::bytes_to_hex(bytes));
        key_pair.verify(&bytes, msg).unwrap();
    }
    #[test]
    #[should_panic(expected = "Verification failed")]
    fn test_generate_sign_verify() {
        // generate 2 key pairs. Generate with one, verify with the
        // other. Should blow up!
        let key_pair = KeyPair::generate().unwrap();
        let new_key_pair = KeyPair::generate().unwrap();
        let msg  =b"this is a test thing";
        let bytes = new_key_pair.sign(msg).unwrap();
        key_pair.verify(&bytes, msg).unwrap();
    }

    #[test]
    fn test_write_sign_verify() {
        // generate a key pair, write it to a file. Read from the file
        // into a new variable, sign with one and check that
        // verification with the other works
        let new_key_pair = KeyPair::generate().unwrap();
        new_key_pair.write_to_files(&String::from("test/keys/new.pub"),
                                    &String::from("test/keys/new")).unwrap();
        let msg  =b"this is a test thing";
        let bytes = new_key_pair.sign(msg).unwrap();
        let loaded_key_pair = KeyPair::read_from_files(&String::from("test/keys/new.pub"),
                                                       &String::from("test/keys/new"),
                                                       &String::from(""));
        loaded_key_pair.verify(&bytes, msg).unwrap();
    }

    use diesel::prelude::*;
    use diesel::pg::PgConnection;
    use dotenv::dotenv;
    use std::env;
    
    pub fn establish_connection() -> PgConnection {
        dotenv().ok();
        
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
    }

        
    #[test]
    fn test_save_block() {
        use Epoch;

        let epoch = Epoch::new(String::from("http://localhost:3013"));
        
        use models::InsertableBlock;
        let block = InsertableBlock {
            hash: String::from("bh$abcdef0123456789abcdef0123456789abcdef0123456789"),
            height: 123456,
            miner: String::from("ak$abcdef0123456789abcdef0123456789abcdef0123456789"),
            nonce: bigdecimal::BigDecimal::from(567876876876),
            prev_hash: String::from("bh$abcdef0123456789abcdef0123456789abcdef0123456789"),
            state_hash: String::from("sh$abcdef0123456789abcdef0123456789abcdef0123456789"),
            txs_hash: String::from("th$abcdef0123456789abcdef0123456789abcdef0123456789"),
            target: 12345676,
            time: 78798797987,
            version: 1,
        };
        let conn = establish_connection();
        use models::Block;
        let last_id = Block::max_id(&conn).unwrap();
        block.save(&conn);
        let id = get_last_block_id(&conn).unwrap();
        assert_eq!((last_id+1i32) as i64, id);
        
    }

    
}

