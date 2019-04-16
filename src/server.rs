use diesel::sql_query;

use models::*;
use node::Node;

use chrono::prelude::*;
use diesel::RunQueryDsl;
use rocket;
use rocket::http::Method;
use rocket::State;
use rocket_contrib::databases::diesel;
use rocket_contrib::json::*;
use rocket_cors;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use serde_json;
use std::path::PathBuf;

use SQLCONNECTION;

use PGCONNECTION;
pub struct MiddlewareServer {
    pub node: Node,
    pub dest_url: String, // address to forward to
    pub port: u16,        // port to listen on
}

// SQL santitizing method to prevent injection attacks.
fn sanitize(s: &String) -> String {
    s.replace("'", "\\'")
}

/*
 * GET handler for Node
 */
#[get("/<path..>", rank = 6)]
fn node_get_handler(state: State<MiddlewareServer>, path: PathBuf) -> Json<serde_json::Value> {
    debug!("Fetching from node: {}", path.to_str().unwrap());
    Json(
        state
            .node
            .get_naked(&String::from("/v2/"), &String::from(path.to_str().unwrap()))
            .unwrap(),
    )
}

/*
 * POST handler for Node
 */
#[post("/<path..>", format = "application/json", data = "<body>")]
fn node_post_handler(
    state: State<MiddlewareServer>,
    path: PathBuf,
    body: Json<String>,
) -> Json<serde_json::Value> {
    debug!("{:?}", body);
    let response = state
        .node
        .post_naked(
            &String::from("/v2/"),
            &String::from(path.to_str().unwrap()),
            body.to_string(),
        )
        .unwrap();
    debug!("Response: {}", response);
    Json(serde_json::from_str(response.as_str()).unwrap())
}

/*
 * Node's only endpoint which lives outside of /v2/...
 */
#[get("/")]
fn node_api_handler(state: State<MiddlewareServer>) -> Json<serde_json::Value> {
    Json(
        state
            .node
            .get_naked(&String::from("/api"), &String::from(""))
            .unwrap(),
    )
}

#[get("/generations/current", rank = 1)]
fn current_generation(state: State<MiddlewareServer>) -> Json<serde_json::Value> {
    let _height = KeyBlock::top_height(&PGCONNECTION.get().unwrap()).unwrap();
    generation_at_height(state, _height)
}

#[get("/generations/height/<height>", rank = 1)]
fn generation_at_height(state: State<MiddlewareServer>, height: i64) -> Json<serde_json::Value> {
    match JsonGeneration::get_generation_at_height(
        &SQLCONNECTION.get().unwrap(),
        &PGCONNECTION.get().unwrap(),
        height,
    ) {
        Some(x) => Json(serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap()),
        None => {
            info!("Generation not found at height {}", height);
            let mut path = std::path::PathBuf::new();
            path.push(format!("generations/height/{}", height));
            return node_get_handler(state, path);
        }
    }
}

#[get("/key-blocks/current/height", rank = 1)]
fn current_key_block(_state: State<MiddlewareServer>) -> Json<JsonValue> {
    let _height = KeyBlock::top_height(&PGCONNECTION.get().unwrap()).unwrap();
    Json(json!({
        "height" : _height,
    }))
}

#[get("/key-blocks/height/<height>", rank = 1)]
fn key_block_at_height(state: State<MiddlewareServer>, height: i64) -> Json<String> {
    let key_block = match KeyBlock::load_at_height(&PGCONNECTION.get().unwrap(), height) {
        Some(x) => x,
        None => {
            info!("Generation not found at height {}", height);
            return Json(
                serde_json::to_string(&state.node.get_generation_at_height(height).unwrap())
                    .unwrap(),
            );
        }
    };
    info!("Serving key block {} from DB", height);
    Json(serde_json::to_string(&JsonKeyBlock::from_key_block(&key_block)).unwrap())
}

#[get("/transactions/<hash>")]
fn transaction_at_hash(state: State<MiddlewareServer>, hash: String) -> Json<serde_json::Value> {
    let tx: Transaction = match Transaction::load_at_hash(&PGCONNECTION.get().unwrap(), &hash) {
        Some(x) => x,
        None => {
            info!("Transaction not found at hash {}", &hash);
            let mut path = std::path::PathBuf::new();
            path.push(format!("/transactions/hash/{}", hash));
            return node_get_handler(state, path);
        }
    };
    Json(
        serde_json::from_str(
            &serde_json::to_string(&JsonTransaction::from_transaction(&tx)).unwrap(),
        )
        .unwrap(),
    )
}

#[get("/key-blocks/hash/<hash>", rank = 1)]
fn key_block_at_hash(state: State<MiddlewareServer>, hash: String) -> Json<serde_json::Value> {
    let key_block = match KeyBlock::load_at_hash(&PGCONNECTION.get().unwrap(), &hash) {
        Some(x) => x,
        None => {
            info!("Key block not found at hash {}", &hash);
            let mut path = std::path::PathBuf::new();
            path.push(format!("/key-blocks/hash/{}", hash));
            return node_get_handler(state, path);
        }
    };
    debug!("Serving key block {} from DB", hash);
    Json(
        serde_json::from_str(
            &serde_json::to_string(&JsonKeyBlock::from_key_block(&key_block)).unwrap(),
        )
        .unwrap(),
    )
}

#[get("/micro-blocks/hash/<hash>/transactions", rank = 1)]
fn transactions_in_micro_block_at_hash(
    _state: State<MiddlewareServer>,
    hash: String,
) -> Json<JsonTransactionList> {
    let sql = format!("select t.* from transactions t, micro_blocks m where t.micro_block_id = m.id and m.hash = '{}'", sanitize(&hash));
    let transactions: Vec<Transaction> =
        sql_query(sql).load(&*PGCONNECTION.get().unwrap()).unwrap();
    let mut trans: Vec<JsonTransaction> = vec![];
    for i in 0..transactions.len() {
        trans.push(JsonTransaction::from_transaction(&transactions[i]));
    }
    let list = JsonTransactionList {
        transactions: trans,
    };
    Json(list)
}

#[get("/micro-blocks/hash/<hash>/header", rank = 1)]
fn micro_block_header_at_hash(_state: State<MiddlewareServer>, hash: String) -> Json<JsonValue> {
    let sql = "select m.hash, k.height, m.pof_hash, m.prev_hash, m.prev_key_hash, m.signature, m.state_hash, m.time_, m.txs_hash, m.version from micro_blocks m, key_blocks k where m.key_block_id=k.id and m.hash = $1";
    let rows = SQLCONNECTION.get().unwrap().query(sql, &[&hash]).unwrap();
    #[derive(Serialize)]
    struct JsonMicroBlock {
        hash: String,
        height: i64,
        pof_hash: String,
        prev_hash: String,
        prev_key_hash: String,
        signature: String,
        state_hash: String,
        time: i64,
        txs_hash: String,
        version: i32,
    };
    if rows.len() > 0 {
        let r = rows.get(0);
        let val = json!(JsonMicroBlock {
            hash: r.get(0),
            height: r.get(1),
            pof_hash: r.get(2),
            prev_hash: r.get(3),
            prev_key_hash: r.get(4),
            signature: r.get(5),
            state_hash: r.get(6),
            time: r.get(7),
            txs_hash: r.get(8),
            version: r.get(9),
        });
        return Json(val);
    } else {
        return Json(json!(""));
    }
}

/*
 * Gets the amount spent, and number of transactions between specific dates,
 * broken up by day.
 *
 */
#[get("/transactions/rate/<from>/<to>")]
fn transaction_rate(_state: State<MiddlewareServer>, from: String, to: String) -> Json<JsonValue> {
    let from = NaiveDate::parse_from_str(&from, "%Y%m%d").unwrap();
    let to = NaiveDate::parse_from_str(&to, "%Y%m%d").unwrap();
    debug!("{:?} - {:?}", from, to);
    Json(json!(Transaction::rate(
        &SQLCONNECTION.get().unwrap(),
        from,
        to
    )
    .unwrap()))
}

/*
 * Gets this size of the chain at some height
 */
#[get("/size/height/<height>")]
fn size(_state: State<MiddlewareServer>, height: i32) -> Json<JsonValue> {
    let size = size_at_height(&SQLCONNECTION.get().unwrap(), height).unwrap();
    Json(json!({
        "size": size,
    }))
}

/*
 * return the current size of the DB
 */
#[get("/size/current")]
fn current_size(_state: State<MiddlewareServer>) -> Json<JsonValue> {
    let _height = KeyBlock::top_height(&PGCONNECTION.get().unwrap()).unwrap();
    size(_state, _height as i32)
}

/*
 * Gets count of transactions for an account
 */
#[get("/transactions/account/<account>/count")]
fn transaction_count_for_account(
    _state: State<MiddlewareServer>,
    account: String,
) -> Json<JsonValue> {
    let s_acc = sanitize(&account);
    let sql = format!(
        "select count(1) from transactions where \
         tx->>'sender_id'='{}' or \
         tx->>'account_id' = '{}' or \
         tx->>'recipient_id'='{}' or \
         tx->>'owner_id' = '{}' ",
        s_acc, s_acc, s_acc, s_acc
    );
    debug!("{}", sql);
    let rows = SQLCONNECTION.get().unwrap().query(&sql, &[]).unwrap();
    let count: i64 = rows.get(0).get(0);
    Json(json!({
        "count": count,
    }))
}

fn offset_limit(limit: Option<i32>, page: Option<i32>) -> (String, String) {
    let offset_sql;
    let limit_sql = match limit {
        None => {
            offset_sql = String::from(" 0 ");
            String::from(" all ")
        }
        Some(x) => {
            offset_sql = match page {
                None => String::from(" 0 "),
                Some(y) => format!(" {} ", (y - 1) * x),
            };
            format!(" {} ", x)
        }
    };
    (offset_sql, limit_sql)
}

/*
 * Gets all transactions for an account
 */
#[get("/transactions/account/<account>?<limit>&<page>")]
fn transactions_for_account(
    _state: State<MiddlewareServer>,
    account: String,
    limit: Option<i32>,
    page: Option<i32>,
) -> Json<JsonTransactionList> {
    let s_acc = sanitize(&account);
    let (offset_sql, limit_sql) = offset_limit(limit, page);
    let sql = format!(
        "select * from transactions where \
         tx->>'sender_id'='{}' or \
         tx->>'account_id' = '{}' or \
         tx->>'recipient_id'='{}' or \
         tx->>'owner_id' = '{}' \
         order by id desc \
         limit {} offset {} ",
        s_acc, s_acc, s_acc, s_acc, limit_sql, offset_sql
    );
    info!("{}", sql);
    let transactions: Vec<Transaction> =
        sql_query(sql).load(&*PGCONNECTION.get().unwrap()).unwrap();
    let mut trans: Vec<JsonTransaction> = vec![];
    for i in 0..transactions.len() {
        trans.push(JsonTransaction::from_transaction(&transactions[i]));
    }
    let list = JsonTransactionList {
        transactions: trans,
    };
    Json(list)
}

/*
 * Gets transactions between blocks
 */
#[get("/transactions/interval/<from>/<to>?<limit>&<page>")]
fn transactions_for_interval(
    _state: State<MiddlewareServer>,
    from: i64,
    to: i64,
    limit: Option<i32>,
    page: Option<i32>,
) -> Json<JsonTransactionList> {
    let (offset_sql, limit_sql) = offset_limit(limit, page);
    let sql = format!(
        "select t.* from transactions t, micro_blocks m, key_blocks k where \
         t.micro_block_id=m.id and \
         m.key_block_id=k.id and \
         k.height >={} and k.height <= {} \
         order by k.height desc, t.id desc \
         limit {} offset {} ",
        from, to, limit_sql, offset_sql
    );
    let transactions: Vec<Transaction> =
        sql_query(sql).load(&*PGCONNECTION.get().unwrap()).unwrap();
    let mut trans: Vec<JsonTransaction> = vec![];
    for i in 0..transactions.len() {
        trans.push(JsonTransaction::from_transaction(&transactions[i]));
    }
    let list = JsonTransactionList {
        transactions: trans,
    };
    Json(list)
}

#[get("/micro-blocks/hash/<hash>/transactions/count")]
/*
 * Gets count of transactions in a microblock
 */
fn transaction_count_in_micro_block(
    _state: State<MiddlewareServer>,
    hash: String,
) -> Json<JsonValue> {
    Json(json!({
        "count": MicroBlock::get_transaction_count(&SQLCONNECTION.get().unwrap(), &hash, ),
    }))
}

#[get("/contracts/transactions/address/<address>")]
fn transactions_for_contract_address(
    _state: State<MiddlewareServer>,
    address: String,
) -> Json<JsonTransactionList> {
    let sql = format!(
        "select t.* from transactions t where \
         t.tx_type='ContractCallTx' and \
         t.tx->>'contract_id' = '{}' or \
         t.id in (select transaction_id from contract_identifiers where \
         contract_identifier='{}')",
        sanitize(&address),
        sanitize(&address)
    );
    let transactions: Vec<Transaction> =
        sql_query(sql).load(&*PGCONNECTION.get().unwrap()).unwrap();
    let mut trans: Vec<JsonTransaction> = vec![];
    for i in 0..transactions.len() {
        trans.push(JsonTransaction::from_transaction(&transactions[i]));
    }
    let list = JsonTransactionList {
        transactions: trans,
    };
    Json(list)
}

// TODO: Lot of refactoring in the below method
#[get("/generations/<from>/<to>?<limit>&<page>")]
fn generations_by_range(_state: State<MiddlewareServer>, from: i64, to: i64,
    limit: Option<i32>,
    page: Option<i32>,) -> Json<JsonValue> {
    let (offset, limit) = offset_limit(limit, page);
    let sql = format!("select k.height, k.beneficiary, k.hash, k.miner, k.nonce::text, k.pow, \
        k.prev_hash, k.prev_key_hash, k.state_hash, k.target, k.time_, k.\"version\", \
        m.hash, m.pof_hash, m.prev_hash, m.prev_key_hash, m.signature, \
        m.state_hash, m.time_, m.txs_hash, m.\"version\", \
        t.block_hash, t.block_height, t.hash, t.signatures, t.tx \
        from key_blocks k left join micro_blocks m on k.id = m.key_block_id \
        left join transactions t on m.id = t.micro_block_id \
        where k.height >={} and k.height <={} \
        order by k.height desc, m.time_ desc limit {} offset {}", from, to, limit, offset);
    let mut list = json!({});
    let mut mb_count = 0;
    let mut tx_count = 0;
    for row in &SQLCONNECTION.get().unwrap().query(&sql, &[]).unwrap() {
        let mut transaction = json!({"block_hash": ""});
        let mut micro_block = json!({"prev_key_hash":""});
        let mut key_block = json!({"height": ""});
        // check if tx is avaiable for a given row
        match row.get(21) {
            Some(val) =>  {
                let block_hash: String = val;
                let block_height:i32 =  row.get(22);
                let hash:String =  row.get(23);
                let signatures:String =  row.get(24);
                let tx_:serde_json::Value =  row.get(25);
                transaction = json!({
                    "block_hash": block_hash,
                    "block_height": block_height,
                    "hash": hash,
                    "signatures": signatures,
                    "tx": tx_
                });
            tx_count += 1;
            },
            _ => {},
        };
        //check if micro_block is avaialble for a given row
        match row.get(15) {
            Some(val) => {
                let prev_key_hash: String = val;
                let hash: String = row.get(12);
                let pof_hash:String = row.get(13);
                let prev_hash:String = row.get(14);
                let signature:String = row.get(16);
                let state_hash:String = row.get(17);
                let time:i64 = row.get(18);
                let txs_hash:String = row.get(19);
                let version:i32 = row.get(20);
                micro_block = json!({
                    "hash": hash, "pof_hash": pof_hash, "prev_hash": prev_hash,
                    "prev_key_hash": prev_key_hash, "signature": signature,
                    "state_hash": state_hash, "time": time, "txs_hash": txs_hash,
                    "version": version
            });
            },
            _ => {},
        };
        // get current key block
        match row.get(0) {
            Some(val) => {
                let height:i64 = val;
                let beneficiary:String = row.get(1);
                let hash:String = row.get(2);
                let miner:String = row.get(3);
                let nonce:String = row.get(4);
                let pow:String = row.get(5);
                let prev_hash:String = row.get(6);
                let prev_key_hash:String = row.get(7);
                let state_hash:String = row.get(8);
                let target:i64 = row.get(9);
                let time:i64 = row.get(10);
                let version:i32 = row.get(11);
                key_block = json!({
                    "height": height, "beneficiary": beneficiary, "hash": hash,
                    "miner": miner, "nonce": nonce, "pow": pow, "prev_hash": prev_hash,
                    "prev_key_hash": prev_key_hash, "state_hash": state_hash,
                    "target": target, "time": time, "version": version,
                    "micro_blocks": {}
            });
            },
            _ => {}
        }
        let block_height: i64 = serde_json::from_value(key_block["height"].clone()).unwrap();
        let key_height: String = block_height.to_string();
        if list[&key_height] != serde_json::json!(null) {
            if micro_block["prev_key_hash"] != "" {
                let mb_hash:String = serde_json::from_value(micro_block["hash"].clone()).unwrap();
                if list[&key_height]["micro_blocks"][&mb_hash] == serde_json::json!(null) {
                    list[&key_height]["micro_blocks"][&mb_hash] = serde_json::to_value(micro_block).unwrap();
                    list[&key_height]["micro_blocks"][&mb_hash]["transactions"] = serde_json::json!({});
                    mb_count += 1;
                }
                if transaction["block_hash"] != "" {
                    let hash:String = serde_json::from_value(transaction["hash"].clone()).unwrap();
                    list[&key_height]["micro_blocks"][mb_hash]["transactions"][hash] = serde_json::to_value(transaction).unwrap();;
                }
            }
        }
        else {
            list[&key_height] = serde_json::to_value(key_block).unwrap();
            if micro_block["prev_key_hash"] != "" {
                let mb_hash:String = serde_json::from_value(micro_block["hash"].clone()).unwrap();
                list[&key_height]["micro_blocks"][&mb_hash] = serde_json::json!({});
                list[&key_height]["micro_blocks"][&mb_hash] = serde_json::to_value(micro_block).unwrap();
                list[&key_height]["micro_blocks"][&mb_hash]["transactions"] = serde_json::json!({});
                mb_count += 1;
                if transaction["block_hash"] != "" {
                    let hash:String = serde_json::from_value(transaction["hash"].clone()).unwrap();
                    list[&key_height]["micro_blocks"][mb_hash]["transactions"][hash] = serde_json::to_value(transaction).unwrap();;
                }
            }
        }
    }

    Json(json!({
        "total_transactions": tx_count,
        "total_micro_blocks": mb_count,
        "data": list
    }))
}

#[get("/channels/transactions/address/<address>")]
fn transactions_for_channel_address(
    _state: State<MiddlewareServer>,
    address: String,
) -> Json<JsonTransactionList> {
    let sql = format!(
        "select t.* from transactions t where \
         t.tx->>'channel_id' = '{}' or \
         t.id in (select transaction_id from channel_identifiers where \
         channel_identifier='{}')",
        sanitize(&address),
        sanitize(&address)
    );
    debug!("{}", sql);
    let transactions: Vec<Transaction> =
        sql_query(sql).load(&*PGCONNECTION.get().unwrap()).unwrap();
    let mut trans: Vec<JsonTransaction> = vec![];
    for tx in transactions {
        trans.push(JsonTransaction::from_transaction(&tx));
    }
    Json(JsonTransactionList {
        transactions: trans,
    })
}

#[get("/channels/active")]
fn active_channels(_state: State<MiddlewareServer>) -> Json<Vec<String>> {
    let sql = "select channel_identifier from channel_identifiers where \
               channel_identifier not in \
               (select tx->>'channel_id' from transactions where \
               tx_type in \
               ('ChannelCloseTx', 'ChannelCloseMutualTx', 'ChannelCloseSoloTx', 'ChannelSlashTx')) \
               order by id asc"
        .to_string();
    Json(
        SQLCONNECTION
            .get()
            .unwrap()
            .query(&sql, &[])
            .unwrap()
            .iter()
            .map(|x| x.get(0))
            .collect(),
    )
}

#[get("/oracles/all?<limit>&<page>")]
fn oracle_requests_responses(
    _state: State<MiddlewareServer>,
    limit: Option<i32>,
    page: Option<i32>,
) -> JsonValue {
    let (offset_sql, limit_sql) = offset_limit(limit, page);
    let sql = format!(
        "select oq.query_id, t1.tx, t2.tx from \
         oracle_queries oq \
         join transactions t1 on oq.transaction_id=t1.id \
         left outer join transactions t2 on t2.tx->>'query_id' = oq.query_id \
         limit {} offset {} ",
        limit_sql, offset_sql
    );
    let mut res: Vec<JsonValue> = vec![];
    for row in &SQLCONNECTION.get().unwrap().query(&sql, &[]).unwrap() {
        let query_id: String = row.get(0);
        let request: serde_json::Value = row.get(1);
        let response: Option<serde_json::Value> = row.get(2);
        res.push(json!({
            "query_id": query_id,
            "request": json!(request),
            "response": json!(response),
        }));
    }
    json!(res)
}

impl MiddlewareServer {
    pub fn start(self) {
        let allowed_origins = AllowedOrigins::all();
        let options = rocket_cors::Cors {
            allowed_origins,
            allowed_methods: vec![Method::Get ].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            ..Default::default()
        };

        use rocket::fairing::AdHoc;
        use rocket::http::Header;

        rocket::ignite()
            .mount("/middleware", routes![active_channels])
            .mount("/middleware", routes![current_size])
            .mount("/middleware", routes![generations_by_range])
            .mount("/middleware", routes![oracle_requests_responses])
            .mount("/middleware", routes![size])
            .mount("/middleware", routes![transaction_rate])
            .mount("/middleware", routes![transactions_for_account])
            .mount("/middleware", routes![transactions_for_interval])
            .mount("/middleware", routes![transaction_count_for_account])
            .mount("/middleware", routes![transactions_for_channel_address])
            .mount("/middleware", routes![transactions_for_contract_address])
            .mount("/v2", routes![current_generation])
            .mount("/v2", routes![current_key_block])
            .mount("/v2", routes![generation_at_height])
            .mount("/v2", routes![key_block_at_height])
            .mount("/v2", routes![key_block_at_hash])
            .mount("/v2", routes![micro_block_header_at_hash])
            .mount("/v2", routes![node_get_handler])
            .mount("/v2", routes![node_post_handler])
            .mount("/api", routes![node_api_handler])
            .mount("/v2", routes![transaction_at_hash])
            .mount("/v2", routes![transaction_count_in_micro_block])
            .mount("/v2", routes![transactions_in_micro_block_at_hash])
            .attach(AdHoc::on_request("Handle null origin", |request, _| {
                let mut headers = request.headers().to_owned();
                for mut header in headers.get("Origin") {
                    match header {
                        "null" => request.replace_header(Header::new("Origin", "http://null")),
                        _ => (),
                    }
                }
            }))
            .attach(options)
            .manage(self)
            .launch();
    }
}
