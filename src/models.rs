use super::schema::blocks;

use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::pg::PgConnection;

extern crate serde_json;
use serde_json::Value;
use std;

#[derive(Queryable)]
pub struct Block {
    pub id: i32,
    pub hash: String,
    pub height: i64,
    pub miner: String,
    pub nonce: i64,
    pub prev_hash: String,
    pub state_hash: String,
    pub target: i64,
    pub time_: i64,
    pub txs_hash: String,
    pub version: i32,
}

sql_function!(fn currval(x: VarChar) -> BigInt);

impl Block {

    pub fn from_json(json: &serde_json::Value) ->
        Result<Block, Box<std::error::Error>>{
            let newblock = Block {
                hash: json["hash"].to_string(),
                height: json["height"].as_i64().unwrap(),
                miner: json["miner"].to_string(),
                nonce: json["nonce"].as_i64().unwrap(),
                prev_hash: json["prev_hash"].to_string(),
                state_hash: json["state_hash"].to_string(),
                txs_hash: json["txs_hash"].to_string(),
                target: json["target"].as_i64().unwrap(),
                time_: json["time"].as_i64().unwrap(),
                version: json["version"].as_i64().unwrap() as i32,
            };
            Ok(newblock)
        }    
    
    pub fn extract_and_save(conn: &PgConnection, block: serde_json::Value) ->
        Result<i64, Box<std::error::Error>>{
            let newblock = Block::from_json(&block)?;
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

    pub fn max_id(conn: &PgConnection) -> Result<i32, Box<std::error::Error>> {
        let block: Block = blocks::table.find(1).first(conn).unwrap();
        Ok(1)
    }
                                                           
}

use super::schema::transactions;

#[derive(Queryable)]
pub struct Transaction {
    pub id: i32,
    pub block_id: i64,
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

