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

    pub fn from_json(json: &serde_json::Value) ->
        Result<InsertableBlock, Box<std::error::Error>>{
            let newblock = InsertableBlock {
                hash: json["hash"].to_string(),
                height: json["height"].as_i64().unwrap(),
                miner: json["miner"].to_string(),
                nonce: bigdecimal::BigDecimal::from(1),
                prev_hash: json["prev_hash"].to_string(),
                state_hash: json["state_hash"].to_string(),
                txs_hash: json["txs_hash"].to_string(),
                target: json["target"].as_i64().unwrap(),
                time: json["time"].as_i64().unwrap(),
                version: json["version"].as_i64().unwrap() as i32,
            };
            Ok(newblock)
        }    
    
    pub fn extract_and_save(conn: &PgConnection, block: serde_json::Value) ->
        Result<i64, Box<std::error::Error>>{
            let newblock = InsertableBlock::from_json(&block)?;
            newblock.save(conn)
        }

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
            Ok(InsertableBlock {
                hash: jb.hash.clone(),
                height: jb.height,
                miner: jb.miner.clone(),
                nonce: bigdecimal::BigDecimal::from(1),
                prev_hash: jb.prev_hash.clone(),
                state_hash: jb.state_hash.clone(),
                target: jb.target.clone(),
                time: jb.time,
                txs_hash: jb.txs_hash.clone(),
                version: jb.version,
            })
        }
            

}

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
}

use super::schema::transactions;

#[derive(Queryable)]
pub struct Transaction {
    pub id: i32,
    pub block_id: i32,
    pub original_json: String,
    pub recipient_pubkey: String,
    pub amount: i64,
    pub fee: i64,
    pub ttl: i64,
    pub sender: String,
    pub payload: String,
}

#[derive(Insertable,Debug)]
#[table_name="transactions"]
pub struct InsertableTransaction {
    pub block_id: i32,
    pub original_json: String,
    pub recipient_pubkey: String,
    pub amount: i64,
    pub fee: i64,
    pub ttl: i64,
    pub sender: String,
    pub payload: String,
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

}
