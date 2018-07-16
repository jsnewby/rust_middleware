use super::schema::blocks;

use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::pg::PgConnection;

extern crate serde_json;

use std;

#[derive(Queryable)]
pub struct Block {
    pub id: i32,
    pub hash: Option<String>,
    pub height: Option<i64>,
    pub miner: Option<String>,
    pub nonce: Option<i64>,
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
#[derive(Serialize, Deserialize)]
#[table_name="blocks"]
pub struct InsertableBlock {
    pub hash: String,
    pub height: i64,
    pub miner: String,
    pub nonce: i64,
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
                nonce: json["nonce"].as_i64().unwrap(),
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

/*
    pub fn from_json(block_id: i32, json: &serde_json::Value) ->
        Result<InsertableTransaction, Box<std::error::Error>>{
            let tran = InsertableTransaction {
                block_id: block_id,
                pub original_json: String::from(""),
                pub recipient_pubkey: ,
                pub amount: i64,
                pub fee: i64,
                pub ttl: i64,
                pub sender: String,
                pub payload: String,
            };
            Ok(tran)
        }    
    
    pub fn extract_and_save(conn: &PgConnection, tx: serde_json::Value) ->
        Result<i64, Box<std::error::Error>>{
            let tran = InsertableTransaction::from_json(&tx)?;
            tran.save(conn)
        }

*/    pub fn save(&self, conn: &PgConnection) ->
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
