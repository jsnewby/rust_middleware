use super::schema::blocks;

use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::pg::PgConnection;

extern crate serde_json;
use serde_json::Number;

use bigdecimal;

use std;

#[derive(Queryable)]
pub struct Block {
    pub id: i32,
    pub hash: Option<String>,
    pub height: Option<i64>,
    pub miner: Option<String>,
    pub nonce: Option<bigdecimal::BigDecimal>,
    pub prev_hash: Option<String>,
    pub state_hash: Option<String>,
    pub target: Option<i64>,
    pub time: Option<i64>,
    pub txs_hash: Option<String>,
    pub version: Option<i32>,
}

sql_function!(fn currval(x: VarChar) -> BigInt);

impl Block {

    pub fn max_id(conn: &PgConnection) -> Result<i32, Box<std::error::Error>> {
        let b = blocks::table.order(blocks::id.desc()).load::<Block>(conn)?;
        Ok(b.first().unwrap().id)
    }
                                                           
}

#[derive(Insertable)]
#[table_name="blocks"]
pub struct InsertableBlock {
    pub hash: String,
    pub height: i64,
    pub miner: String,
    pub nonce: bigdecimal::BigDecimal,
    pub prev_hash: String,
    pub state_hash: String,
    pub target: i64,
    pub time: i64,
    pub txs_hash: String,
    pub version: i32,
}


impl InsertableBlock {
    
    pub fn save(&self, conn: &PgConnection) ->
        Result<i64, Box<std::error::Error>> {
            use diesel::dsl::{select, insert_into};
            use diesel::RunQueryDsl;
            use schema::blocks::dsl::*;
            insert_into(blocks)
                .values(self).execute(&*conn)?;
            let generated_id = select(currval("blocks_id_seq")).get_result::<i64>(&*conn)?;
            Ok(generated_id)
        }

    pub fn from_json_block(jb: &JsonBlock) ->
        Result<InsertableBlock, Box<std::error::Error>> {
            //TODO: fix this.
            let nonce: u64 = match jb.nonce.as_u64() {
                Some(val) => val,
                None => 0,
            };
            Ok(InsertableBlock {
                hash: jb.hash.clone(),
                height: jb.height,
                miner: jb.miner.clone(),
                nonce: bigdecimal::BigDecimal::from(nonce),
                prev_hash: jb.prev_hash.clone(),
                state_hash: jb.state_hash.clone(),
                target: jb.target.clone(),
                time: jb.time,
                txs_hash: jb.txs_hash.clone(),
                version: jb.version,
            })
        }
            

}

/*
In a better world, the serialization object would be the same as we
use for persistence, but in this one right now that doesn't work,
because serde_json needs serde_json::Number, and diesel needs a
bigdecimal::BigDecimal. So this struct exists to be pulled from the
JSON.  If @newby gets smart enough he will write the implementations
for these missing methods.
*/
#[derive(Serialize, Deserialize)]
pub struct JsonBlock {
    pub hash: String,
    pub height: i64,
    pub miner: String,
    pub nonce: Number,
    pub prev_hash: String,
    pub state_hash: String,
    pub target: i64,
    pub time: i64,
    pub txs_hash: String,
    pub version: i32,
    pub transactions: Vec<JsonTransaction>,
}

#[derive(Serialize, Deserialize)]
pub struct JsonTransaction {
    pub block_height: i32,
    pub block_hash: String,
    pub hash: String,
    pub signatures: Vec<String>,
    pub tx: serde_json::Value,
} 

use super::schema::transactions;

#[derive(Queryable)]
pub struct Transaction {
    pub id: i32,
    pub block_id: i32,
    pub block_height: i32,
    pub block_hash: String,
    pub hash: String,
    pub signatures: String,
    pub tx: serde_json::Value,
}

#[derive(Insertable)]
#[table_name="transactions"]
pub struct InsertableTransaction {
    pub block_id: i32,
    pub block_height: i32,
    pub block_hash: String,
    pub hash: String,
    pub signatures: String,
    pub tx_type: String,
    pub tx: serde_json::Value,
}

impl InsertableTransaction {

    pub fn save(&self, conn: &PgConnection) ->
        Result<i64, Box<std::error::Error>> {
            use diesel::dsl::{select, insert_into};
            use diesel::RunQueryDsl;
            use schema::transactions::dsl::*;
            insert_into(transactions)
                .values(self).execute(&*conn)?;
            let generated_id = select(currval("transactions_id_seq")).get_result::<i64>(&*conn)?;
            Ok(generated_id)
        }

    pub fn from_json_transaction(jt: &JsonTransaction, tx_type: String, block_id: i32)
                                 -> Result<InsertableTransaction,
                                           Box<std::error::Error>>
    {
        
        let mut signatures = String::new();
        for i in 0 .. jt.signatures.len() {
            if i > 0 {
                signatures.push_str(" ");
            }
            signatures.push_str(&jt.signatures[i].clone());
        }
        Ok(InsertableTransaction {
            block_id: block_id,
            block_height: jt.block_height,
            block_hash: jt.block_hash.clone(),
            hash: jt.hash.clone(),
            signatures: signatures,
            tx_type: tx_type,
            tx: serde_json::from_str(&jt.tx.to_string()).unwrap(),
        })
    }
                
}
