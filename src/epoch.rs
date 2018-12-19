use curl::easy::{Easy, List};

use diesel::pg::PgConnection;
use dotenv::dotenv;
use postgres::{Connection, TlsMode};
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use regex::Regex;
use serde_json;
use serde_json::Value;
use std;
use std::env;
use std::io::Read;
use std::sync::Arc;

use models::InsertableMicroBlock;
use models::JsonKeyBlock;
use models::JsonTransaction;

pub fn establish_sql_connection() -> postgres::Connection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Connection::connect(database_url, TlsMode::None).unwrap()
}

pub fn get_missing_heights(height: i64) -> Vec<i32> {
    let sql = format!("SELECT * FROM generate_series(0,{}) s(i) WHERE NOT EXISTS (SELECT height FROM key_blocks WHERE height = s.i)", height);
    debug!("{}", &sql);
    let mut missing_heights = Vec::new();
    for row in &establish_sql_connection().query(&sql, &[]).unwrap() {
        missing_heights.push(row.get(0));
    }
    missing_heights
}

pub fn establish_connection(size: u32) -> Arc<Pool<ConnectionManager<PgConnection>>> {
    dotenv().ok(); // Grabbing ENV vars
    debug!("Making new pool size {}", size);

    // Pull DATABASE_URL env var
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create a connection pool manager for a Postgres connection at the `database_url`
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = r2d2::Pool::builder().max_size(size).build(manager).expect("Failed to create pool.");
        

    // Create the pool with the default config and the r2d2_diesel connection manager
    Arc::new(pool)
}

pub struct Epoch {
    base_uri: String,
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl Epoch {
    pub fn new(base_url: String, pool_size: u32) -> Epoch {
        let connection = establish_connection(pool_size);
        Epoch { base_uri: base_url, pool: connection }
    }

    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
        return self.pool.get();
    }

    pub fn current_generation(&self) -> Result<serde_json::Value, Box<std::error::Error>> {
        self.get(&String::from("generations/current"))
    }

    pub fn get_generation_at_height(&self, height: i64) ->
        Result<serde_json::Value, Box<std::error::Error>> {
            let path = format!("generations/height/{}", height);
            self.get(&String::from(path))
    }

    pub fn latest_key_block(&self) -> Result<serde_json::Value, Box<std::error::Error>> {
        self.get(&String::from("key-blocks/current"))
    }

    pub fn get(&self, operation: &String) -> Result<serde_json::Value, Box<std::error::Error>> {
        self.get_naked(&String::from("/v2/"), operation)
    }

    // Get a URL, and parse the JSON returned.
    pub fn get_naked(
        &self,
        prefix: &String,
        operation: &String,
    ) -> Result<serde_json::Value, Box<std::error::Error>> {
        let uri = self.base_uri.clone() + prefix + operation;
        debug!("get_naked() fetching {}", uri);
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
        debug!("get_naked() received {}", serde_json::to_string(&value).unwrap());

        Ok(value)
    }

    pub fn post_naked(
        &self,
        prefix: &String,
        operation: &String,
        body: String,
    ) -> Result<String, Box<std::error::Error>> {
        let uri = self.base_uri.clone() + prefix + operation;
        debug!("post_naked posting to URL: {}, body: {}", uri, body);
        let mut data = body.as_bytes();
        let mut handle = Easy::new();
        handle.url(&uri)?;
        let mut list = List::new();
        list.append("content-type: application/json").unwrap();
        handle.http_headers(list).unwrap();
        handle.post(true)?;
        handle.post_field_size(data.len() as u64)?;
        let mut response = Vec::new();
        {
            let mut transfer = handle.transfer();
            transfer.read_function(|buf| Ok(data.read(buf).unwrap_or(0)))?;
            transfer.write_function(|new_data| {
                response.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
            transfer.perform()?;
        }
        let resp = String::from(std::str::from_utf8(&response).unwrap());
        debug!("get_naked() returning {}", resp);
        Ok(resp)
    }

    pub fn get_key_block_by_hash(
        &self,
        hash: &String,
    ) -> Result<serde_json::Value, Box<std::error::Error>> {
        let result = self.get(&format!("{}{}", String::from("key-blocks/hash/"), &hash))?;
        Ok(result)
    }

    pub fn get_key_block_by_height(
        &self,
        height: i64,
    ) -> Result<serde_json::Value, Box<std::error::Error>> {
        let result = self.get(&format!(
            "{}{}",
            String::from("key-blocks/height/"),
            &height
        ))?;
        Ok(result)
    }

    pub fn get_micro_block_by_hash(
        &self,
        hash: &String,
    ) -> Result<serde_json::Value, Box<std::error::Error>> {
        let result = self.get(&format!(
            "{}{}{}",
            String::from("micro-blocks/hash/"),
            &hash,
            String::from("/header")
        ))?;
        Ok(result)
    }

    pub fn get_transaction_list_by_micro_block(
        &self,
        hash: &String,
    ) -> Result<serde_json::Value, Box<std::error::Error>> {
        let result = self.get(&format!(
            "{}{}{}",
            String::from("micro-blocks/hash/"),
            &hash,
            String::from("/transactions")
        ))?;
        Ok(result)
    }

    pub fn get_pending_transaction_list(
        &self,
    ) -> Result<serde_json::Value, Box<std::error::Error>> {
        let result = self.get(&String::from("debug/transactions/pending"))?;
        Ok(result)
    }
}

pub fn from_json(val: &String) -> String {
    let re = Regex::new("^\"(.*)\"$").unwrap();
    match re.captures(val) {
        Some(matches) => {
            String::from(&matches[1])
        }
        None => val.clone(),
    }
}

pub fn key_block_from_json(json: Value) -> Result<JsonKeyBlock, Box<std::error::Error>> {
    let block: JsonKeyBlock = serde_json::from_value(json)?;
    Ok(block)
}

pub fn micro_block_from_json(json: Value) -> Result<InsertableMicroBlock, Box<std::error::Error>> {
    let block: InsertableMicroBlock = serde_json::from_value(json)?;
    Ok(block)
}

pub fn transaction_from_json(json: Value) -> Result<JsonTransaction, Box<std::error::Error>> {
    let transaction: JsonTransaction = serde_json::from_value(json)?;
    Ok(transaction)
}
