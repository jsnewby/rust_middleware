extern crate aepp_middleware;
extern crate diesel;
#[macro_use]
extern crate assert_json_diff;
#[macro_use]
extern crate serde_json;

use aepp_middleware::loader::BlockLoader;
use aepp_middleware::loader::PGCONNECTION;
use aepp_middleware::models::*;
use diesel::sql_query;
use diesel::RunQueryDsl;
use std::env;

fn get_blockloader() -> BlockLoader {
    let url = env::var("NODE_URL")
        .expect("NODE_URL must be set")
        .to_string();
    BlockLoader::new(url)
}

fn load_blocks(list: Vec<i64>) {
    let loader = get_blockloader();
    for block in list {
        loader.load_blocks(block).unwrap();
    }
}

#[test]
pub fn test_contract_call() {
    load_blocks(vec![7132, 7139]);
    let calls: Vec<ContractCall> = sql_query("SELECT CC.* FROM contract_calls cc, transactions t WHERE cc.transaction_id=t.id AND t.block_height=7139".to_string()).load(&*PGCONNECTION.get().unwrap()).unwrap();
    assert_eq!(calls.len(), 1);
    let args = json!({"function": "createProof",
                      "arguments": [{"type": "string", "value": "testtest"}]});
    let result = json!({"data": {"type": "tuple", "value": []}});
    let cc = &calls[0];
    assert_json_eq!(cc.arguments.clone(), args);
    assert_json_eq!(cc.result.clone().unwrap(), result);
}
