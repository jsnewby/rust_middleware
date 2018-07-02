use super::schema::blocks;

#[derive(Insertable)]
#[derive(Queryable)]
pub struct Block {
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

#[derive(Insertable)]
#[table_name="transactions"]
pub struct InsertableTransaction {
    // pub block_id: i64,
    // TODO: the above prevents compilation. Foreign key!
    pub original_json: String,
    pub recipient_pubkey: String,
    pub amount: i64,
    pub fee: i64,
    pub ttl: i64,
    pub sender: String,
    pub payload: String,
}

