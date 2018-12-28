#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::key_blocks;
use super::schema::key_blocks::dsl::*;
use super::schema::micro_blocks;
use super::schema::transactions;

use diesel::dsl::exists;
use diesel::dsl::select;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;

extern crate serde_json;
use serde_json::Number;

use bigdecimal;
use bigdecimal::ToPrimitive;
use std;
use std::str::FromStr;

use epoch;

#[derive(Queryable, QueryableByName, Hash, PartialEq, Eq)]
#[table_name = "key_blocks"]
pub struct KeyBlock {
    pub id: i32,
    pub hash: String,
    pub height: i64,
    pub miner: String,
    pub beneficiary: String,
    pub pow: String,
    pub nonce: bigdecimal::BigDecimal,
    pub prev_hash: String,
    pub prev_key_hash: String,
    pub state_hash: String,
    pub target: i64,
    pub time: i64,
    pub version: i32,
}

impl KeyBlock {
    pub fn from_json_key_block(jb: &JsonKeyBlock) -> Result<KeyBlock, Box<std::error::Error>> {
        let n: u64 = match jb.nonce.as_u64() {
            Some(val) => val,
            None => 0,
        };
        Ok(KeyBlock {
            id: -1, // TODO
            hash: jb.hash.clone(),
            height: jb.height,
            miner: jb.miner.clone(),
            nonce: bigdecimal::BigDecimal::from(n),
            beneficiary: jb.beneficiary.clone(),
            pow: format!("{:?}", jb.pow),
            prev_hash: jb.prev_hash.clone(),
            prev_key_hash: jb.prev_key_hash.clone(),
            state_hash: jb.state_hash.clone(),
            target: jb.target.clone(),
            time: jb.time,
            version: jb.version,
        })
    }

    pub fn top_height(conn: &PgConnection) -> Result<i64, Box<std::error::Error>> {
        let b = key_blocks::table
            .order(key_blocks::height.desc())
            .load::<KeyBlock>(conn)?;
        let h = match b.first() {
            Some(x) => x,
            None => return Ok(0),
        };
        Ok(h.height)
    }

    pub fn load_at_height(conn: &PgConnection, _height: i64) -> Option<KeyBlock> {
        let mut blocks = match key_blocks::table
            .filter(height.eq(_height))
            .limit(1)
            .load::<KeyBlock>(conn)
        {
            Ok(x) => x,
            Err(y) => {
                error!("Error loading key block: {:?}", y);
                return None;
            }
        };
        Some(blocks.pop()?)
    }

    pub fn load_at_hash(conn: &PgConnection, _hash: &String) -> Option<KeyBlock> {
        let mut blocks = match key_blocks::table
            .filter(hash.eq(_hash))
            .load::<KeyBlock>(conn)
        {
            Ok(x) => x,
            Err(y) => {
                error!("Error loading key block: {:?}", y);
                return None;
            }
        };
        Some(blocks.pop()?)
    }

    pub fn height_exists(conn: &PgConnection, h: i64) -> bool {
        match select(exists(key_blocks.filter(height.eq(h)))).get_result(conn) {
            Ok(result) => result,
            _ => false,
        }
    }
}

#[derive(Insertable)]
#[table_name = "key_blocks"]
pub struct InsertableKeyBlock {
    pub hash: String,
    pub height: i64,
    pub miner: String,
    pub nonce: bigdecimal::BigDecimal,
    pub beneficiary: String,
    pub pow: String,
    pub prev_hash: String,
    pub prev_key_hash: String,
    pub state_hash: String,
    pub target: i64,
    pub time: i64,
    pub version: i32,
}

impl InsertableKeyBlock {
    pub fn save(&self, conn: &PgConnection) -> Result<i32, Box<std::error::Error>> {
        use diesel::dsl::insert_into;
        use diesel::RunQueryDsl;
        use schema::key_blocks::dsl::*;
        let generated_ids: Vec<i32> = insert_into(key_blocks)
            .values(self)
            .returning(id)
            .get_results(&*conn)?;
        Ok(generated_ids[0])
    }

    pub fn from_json_key_block(
        jb: &JsonKeyBlock,
    ) -> Result<InsertableKeyBlock, Box<std::error::Error>> {
        //TODO: fix this.
        let n: u64 = match jb.nonce.as_u64() {
            Some(val) => val,
            None => 0,
        };
        Ok(InsertableKeyBlock {
            hash: jb.hash.clone(),
            height: jb.height,
            miner: jb.miner.clone(),
            nonce: bigdecimal::BigDecimal::from(n),
            beneficiary: jb.beneficiary.clone(),
            pow: format!("{:?}", jb.pow),
            prev_hash: jb.prev_hash.clone(),
            prev_key_hash: jb.prev_key_hash.clone(),
            state_hash: jb.state_hash.clone(),
            target: jb.target.clone(),
            time: jb.time,
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
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonKeyBlock {
    pub hash: String,
    pub height: i64,
    pub miner: String,
    pub beneficiary: String,
    #[serde(default = "zero")]
    pub nonce: Number,
    #[serde(default = "zero_vec_i32")]
    pub pow: Vec<i32>,
    pub prev_hash: String,
    pub prev_key_hash: String,
    pub state_hash: String,
    pub target: i64,
    pub time: i64,
    pub version: i32,
}

impl JsonKeyBlock {
    pub fn eq(&self, other: &JsonKeyBlock) -> bool {
        (self.hash.eq(&other.hash) &&
                self.height == other.height &&
                self.miner.eq(&other.miner) &&
                self.beneficiary.eq(&other.beneficiary) &&
//                self.nonce == other.nonce && // These don't compare well. TODO -- fix
                self.pow.len() == other.pow.len() && // TODO <-- fix
                self.prev_hash.eq(&other.prev_hash) &&
                self.prev_key_hash.eq(&other.prev_key_hash) &&
                self.state_hash.eq(&other.state_hash) &&
                self.target == other.target &&
                self.time == other.time &&
                self.version == other.version)
    }

    pub fn from_key_block(kb: &KeyBlock) -> JsonKeyBlock {
        let pows: Vec<serde_json::Value> = serde_json::from_str(&kb.pow).unwrap();
        let mut pows2 = Vec::<i32>::new();
        for val in pows {
            pows2.push(i32::from_str(&serde_json::to_string(&val).unwrap()).unwrap());
        }
        JsonKeyBlock {
            hash: kb.hash.clone(),
            height: kb.height,
            miner: kb.miner.clone(),
            beneficiary: kb.beneficiary.clone(),
            nonce: serde_json::Number::from_f64(kb.nonce.to_f64().unwrap()).unwrap(),
            pow: pows2,
            prev_hash: kb.prev_hash.clone(),
            prev_key_hash: kb.prev_key_hash.clone(),
            state_hash: kb.state_hash.clone(),
            target: kb.target,
            time: kb.time,
            version: kb.version,
        }
    }
}

fn zero() -> Number {
    serde_json::Number::from_f64(0.0).unwrap()
}

fn zero_vec_i32() -> Vec<i32> {
    vec![0]
}

#[derive(Identifiable, Associations, Queryable, QueryableByName)]
#[belongs_to(KeyBlock)]
#[table_name = "micro_blocks"]
pub struct MicroBlock {
    pub id: i32,
    #[sql_type = "diesel::sql_types::Int4"]
    #[column_name = "key_block_id"]
    pub key_block: Option<KeyBlock>,
    pub hash: String,
    pub pof_hash: String,
    pub prev_hash: String,
    pub prev_key_hash: String,
    pub signature: String,
    pub state_hash: String,
    pub txs_hash: String,
    pub version: i32,
}

fn option_i32() -> Option<i32> {
    None
}

impl MicroBlock {
    pub fn get_microblock_hashes_for_key_block_hash(kb_hash: &String) -> Option<Vec<String>> {
        let sql = format!(
            "SELECT mb.hash FROM micro_blocks mb, key_blocks kb WHERE mb.key_block_id=kb.id and kb.hash='{}' ORDER BY mb.hash",
            kb_hash
        );
        let mut micro_block_hashes = Vec::new();
        for row in &epoch::establish_sql_connection().query(&sql, &[]).unwrap() {
            micro_block_hashes.push(row.get(0));
        }
        Some(micro_block_hashes)
    }

}

#[derive(Identifiable, Associations, Queryable, QueryableByName)]
#[belongs_to(KeyBlock)]
#[table_name = "micro_blocks"]
pub struct DumbMicroBlock {
    pub id: i32,
    pub key_block_id: Option<i32>,
    pub hash: String,
    pub pof_hash: String,
    pub prev_hash: String,
    pub prev_key_hash: String,
    pub signature: String,
    pub state_hash: String,
    pub txs_hash: String,
    pub version: i32,
}

impl DumbMicroBlock {
        pub fn load_at_hash(conn: &PgConnection, _hash: &String) -> Option<DumbMicroBlock> {
        let sql = format!("select * from micro_blocks where hash='{}'", _hash);
        let mut _blocks: Vec<DumbMicroBlock> = sql_query(sql).load(conn).unwrap();
        Some(_blocks.pop()?)
    }
}


#[derive(Insertable)]
#[table_name = "micro_blocks"]
#[derive(Serialize, Deserialize)]
pub struct InsertableMicroBlock {
    pub key_block_id: Option<i32>,
    pub hash: String,
    pub pof_hash: String,
    pub prev_hash: String,
    pub prev_key_hash: String,
    pub signature: String,
    pub state_hash: String,
    pub txs_hash: String,
    pub version: i32,
}

impl InsertableMicroBlock {
    pub fn save(&self, conn: &PgConnection) -> Result<i32, Box<std::error::Error>> {
        use diesel::dsl::insert_into;
        use diesel::RunQueryDsl;
        use schema::micro_blocks::dsl::*;
        let generated_ids: Vec<i32> = insert_into(micro_blocks)
            .values(self)
            .returning(id)
            .get_results(&*conn)?;
        Ok(generated_ids[0])
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonGeneration {
    pub key_block: JsonKeyBlock,
    pub micro_blocks: Vec<String>,
}

impl JsonGeneration {
    pub fn get_generation_at_height(conn: &PgConnection, _height: i64) -> Option<JsonGeneration> {
        let key_block = match KeyBlock::load_at_height(conn, _height) {
            Some(x) => x,
            None => {
                debug!("Didn't find key block at height {} in DB", _height);
                return None;
            }
        };
        info!("Serving generation {} from DB", _height);
        let sql = format!(
            "SELECT hash FROM micro_blocks WHERE key_block_id={}",
            key_block.id
        );
        let mut micro_block_hashes = Vec::new();
        for row in &epoch::establish_sql_connection().query(&sql, &[]).unwrap() {
            micro_block_hashes.push(row.get(0));
        }
        Some(JsonGeneration {
            key_block: JsonKeyBlock::from_key_block(&key_block),
            micro_blocks: micro_block_hashes,
        })
    }

    pub fn eq(&self, other: &JsonGeneration) -> bool {
        debug!("\nComparing {:?} to \n{:?}\n", &self, &other);
        if self.micro_blocks.len() != other.micro_blocks.len() {
            debug!(
                "Different lengths of microblocks array: {} vs {}",
                self.micro_blocks.len(),
                other.micro_blocks.len()
            );
            return false;
        }
        for i in 0..self.micro_blocks.len() {
            if self.micro_blocks[i] != other.micro_blocks[i] {
                return false;
            }
        }
        self.key_block.eq(&other.key_block)
    }
}

#[derive(Queryable, QueryableByName, Identifiable, Serialize, Deserialize)]
#[table_name = "transactions"]
pub struct Transaction {
    pub id: i32,
    pub micro_block_id: Option<i32>,
    pub block_height: i32,
    pub block_hash: String,
    pub hash: String,
    pub signatures: String,
    pub fee: i64,
    pub size: i32,
    pub tx: serde_json::Value,
}

impl Transaction {
    pub fn load_at_hash(conn: &PgConnection, _hash: &String) -> Option<Transaction> {
        let sql = format!("select * from transactions where hash='{}'", _hash);
        let mut _transactions: Vec<Transaction> = sql_query(sql).load(conn).unwrap();
        /*
        let mut _transactions = match transactions::table
            .filter(hash.eq(_hash))
            .limit(1)
            .load::<Transaction>(conn) {
                Ok(x) => x,
                Err(y) => {
                    error!("Error loading key block: {:?}", y);
                    return None;
                },
            };
         */
        Some(_transactions.pop()?)
    }

    pub fn load_for_micro_block(conn: &PgConnection, mb_hash: &String) -> Option<Vec<Transaction>> {
        let sql = format!("select t.* from transactions t, micro_blocks mb where t.micro_block_id = mb.id and mb.hash='{}'", mb_hash);
        let mut _transactions: Vec<Transaction> = sql_query(sql).load(conn).unwrap();
        Some(_transactions)
    }
        
            
}

#[derive(Serialize, Deserialize)]
pub struct JsonTransaction {
    pub block_height: i32,
    pub block_hash: String,
    pub hash: String,
    pub signatures: Vec<String>,
    pub tx: serde_json::Value,
}

impl JsonTransaction {
    pub fn from_transaction(t: &Transaction) -> JsonTransaction {
        let mut signatures: Vec<String> = vec![];
        let _s = t.signatures.split(", ");
        for s in _s {
            signatures.push(String::from(s));
        }
        JsonTransaction {
            block_height: t.block_height,
            block_hash: t.block_hash.clone(),
            hash: t.hash.clone(),
            signatures,
            tx: t.tx.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonTransactionList {
    pub transactions: Vec<JsonTransaction>,
}

#[derive(Insertable)]
#[table_name = "transactions"]
pub struct InsertableTransaction {
    pub micro_block_id: Option<i32>,
    pub block_height: i32,
    pub block_hash: String,
    pub hash: String,
    pub signatures: String,
    pub tx_type: String,
    pub fee: i64,
    pub size: i32,
    pub tx: serde_json::Value,
}

impl InsertableTransaction {
    pub fn save(&self, conn: &PgConnection) -> Result<i32, Box<std::error::Error>> {
        use diesel::dsl::insert_into;
        use diesel::RunQueryDsl;
        use schema::transactions::dsl::*;
        let generated_ids: Vec<i32> = insert_into(transactions)
            .values(self)
            .returning(id)
            .get_results(&*conn)?;
        Ok(generated_ids[0])
    }

    pub fn from_json_transaction(
        jt: &JsonTransaction,
        tx_type: String,
        micro_block_id: Option<i32>,
    ) -> Result<InsertableTransaction, Box<std::error::Error>> {
        let mut signatures = String::new();
        for i in 0..jt.signatures.len() {
            if i > 0 {
                signatures.push_str(" ");
            }
            signatures.push_str(&jt.signatures[i].clone());
        }
        Ok(InsertableTransaction {
            micro_block_id,
            block_height: jt.block_height,
            block_hash: jt.block_hash.clone(),
            hash: jt.hash.clone(),
            signatures,
            tx_type,
            fee: jt.tx["fee"].as_i64().unwrap(),
            size: jt.tx.to_string().len() as i32,
            tx: serde_json::from_str(&jt.tx.to_string()).unwrap(),
        })
    }
}
