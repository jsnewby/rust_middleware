use curl::easy::{Easy, List};
use regex::Regex;
use serde_json;
use serde_json::Value;
use std;
use std::io::Read;

use models::InsertableMicroBlock;
use models::JsonKeyBlock;
use models::JsonTransaction;
use middleware_result::MiddlewareResult;

use SQLCONNECTION;

pub struct Epoch {
    base_uri: String,
}

impl Epoch {
    pub fn new(base_url: String) -> Epoch {
        Epoch {
            base_uri: base_url,
        }
    }

    pub fn clone(&self) -> Epoch {
        Epoch::new(self.base_uri.clone())
    }

    pub fn get_missing_heights(&self, height: i64) ->
        MiddlewareResult<Vec<i32>> {
        let sql = format!("SELECT * FROM generate_series(0,{}) s(i) WHERE NOT EXISTS (SELECT height FROM key_blocks WHERE height = s.i)", height);
        debug!("{}", &sql);
        let mut missing_heights = Vec::new();
        for row in SQLCONNECTION.get()?.query(&sql, &[])?.iter() {
            missing_heights.push(row.get(0));
        }
        Ok(missing_heights)
    }

    pub fn current_generation(&self) -> MiddlewareResult<serde_json::Value> {
        self.get(&String::from("generations/current"))
    }

    pub fn get_generation_at_height(
        &self,
        height: i64,
    ) -> MiddlewareResult<serde_json::Value> {
        let path = format!("generations/height/{}", height);
        self.get(&String::from(path))
    }

    pub fn latest_key_block(&self) -> MiddlewareResult<serde_json::Value> {
        self.get(&String::from("key-blocks/current"))
    }

    pub fn get(&self, operation: &String) -> MiddlewareResult<serde_json::Value> {
        self.get_naked(&String::from("/v2/"), operation)
    }

    // Get a URL, and parse the JSON returned.
    pub fn get_naked(
        &self,
        prefix: &String,
        operation: &String,
    ) -> MiddlewareResult<serde_json::Value> {
        let uri = self.base_uri.clone() + prefix + operation;
        debug!("get_naked() fetching {}", uri);
        let mut data = Vec::new();
        let mut handle = Easy::new();
        handle.timeout(std::time::Duration::from_secs(20))?;
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
        debug!("get_naked() received {}", serde_json::to_string(&value)?);

        Ok(value)
    }

    pub fn post_naked(
        &self,
        prefix: &String,
        operation: &String,
        body: String,
    ) -> MiddlewareResult<String> {
        let uri = self.base_uri.clone() + prefix + operation;
        debug!("post_naked posting to URL: {}, body: {}", uri, body);
        let mut data = body.as_bytes();
        let mut handle = Easy::new();
        handle.url(&uri)?;
        let mut list = List::new();
        list.append("content-type: application/json")?;
        handle.http_headers(list)?;
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
        let resp = String::from(std::str::from_utf8(&response)?);
        debug!("get_naked() returning {}", resp);
        Ok(resp)
    }

    pub fn get_key_block_by_hash(
        &self,
        hash: &String,
    ) -> MiddlewareResult<serde_json::Value> {
        let result = self.get(&format!("{}{}", String::from("key-blocks/hash/"), &hash))?;
        Ok(result)
    }

    pub fn get_key_block_by_height(
        &self,
        height: i64,
    ) -> MiddlewareResult<serde_json::Value> {
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
    ) -> MiddlewareResult<serde_json::Value> {
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
    ) -> MiddlewareResult<serde_json::Value> {
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
    ) -> MiddlewareResult<serde_json::Value> {
        let result = self.get(&String::from("debug/transactions/pending"))?;
        Ok(result)
    }
}

pub fn from_json(val: &String) -> String {
    let re = Regex::new("^\"(.*)\"$").unwrap();
    match re.captures(val) {
        Some(matches) => String::from(&matches[1]),
        None => val.clone(),
    }
}

pub fn key_block_from_json(json: Value) -> MiddlewareResult<JsonKeyBlock> {
    let block: JsonKeyBlock = serde_json::from_value(json)?;
    Ok(block)
}

pub fn micro_block_from_json(json: Value) -> MiddlewareResult<InsertableMicroBlock> {
    let block: InsertableMicroBlock = serde_json::from_value(json)?;
    Ok(block)
}

pub fn transaction_from_json(json: Value) -> MiddlewareResult<JsonTransaction> {
    let transaction: JsonTransaction = serde_json::from_value(json)?;
    Ok(transaction)
}
