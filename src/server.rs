use diesel::sql_query;

use epoch::Epoch;
use models::*;

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

// DB
#[database("middleware")]
struct MiddlewareDbConn(diesel::PgConnection);

pub struct MiddlewareServer {
    pub epoch: Epoch,
    pub dest_url: String, // address to forward to
    pub port: u16,        // port to listen on
}

// SQL santitizing method to prevent injection attacks.
fn sanitize(s: String) -> String {
    s.replace("'", "\\'")
}

/*
 * GET handler for Epoch
 */
#[get("/<path..>", rank = 6)]
fn epoch_get_handler(state: State<MiddlewareServer>, path: PathBuf) -> Json<serde_json::Value> {
    debug!("Fetching from node: {}", path.to_str().unwrap());
    Json(
        state
            .epoch
            .get_naked(&String::from("/v2/"), &String::from(path.to_str().unwrap()))
            .unwrap(),
    )
}

#[get("/v2/<path..>")]
fn epoch_test_handler(state: State<MiddlewareServer>, path: PathBuf) -> Json<serde_json::Value> {
    Json(
        state
            .epoch
            .get_naked(&String::from("/v2/"), &String::from(path.to_str().unwrap()))
            .unwrap(),
    )
}

/*
 * POST handler for Epoch
 */
#[post("/<path..>", format = "application/json", data = "<body>")]
fn epoch_post_handler(
    state: State<MiddlewareServer>,
    path: PathBuf,
    body: Json<String>,
) -> Json<serde_json::Value> {
    debug!("{:?}", body);
    let response = state
        .epoch
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
 * Epoch's only endpoint which lives outside of /v2/...
 */
#[get("/")]
fn epoch_api_handler(state: State<MiddlewareServer>) -> Json<serde_json::Value> {
    Json(
        state
            .epoch
            .get_naked(&String::from("/api"), &String::from(""))
            .unwrap(),
    )
}

#[get("/generations/current", rank = 1)]
fn current_generation(
    conn: MiddlewareDbConn,
    state: State<MiddlewareServer>,
) -> Json<serde_json::Value> {
    let _height = KeyBlock::top_height(&conn).unwrap();
    generation_at_height(conn, state, _height)
}

#[get("/generations/height/<height>", rank = 1)]
fn generation_at_height(
    conn: MiddlewareDbConn,
    state: State<MiddlewareServer>,
    height: i64,
) -> Json<serde_json::Value> {
    match JsonGeneration::get_generation_at_height(
        &SQLCONNECTION.get().unwrap(),
        &conn,
        height,
    ) {
        Some(x) => Json(serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap()),
        None => {
            info!("Generation not found at height {}", height);
            let mut path = std::path::PathBuf::new();
            path.push(format!("generations/height/{}", height));
            return epoch_get_handler(state, path);
        }
    }
}

#[get("/key-blocks/current/height", rank = 1)]
fn current_key_block(
    conn: MiddlewareDbConn,
    state: State<MiddlewareServer>,
) -> Json<JsonValue> {
    let _height = KeyBlock::top_height(&conn).unwrap();
    Json(json!({
        "height" : _height,
    }))
}



#[get("/key-blocks/height/<height>", rank = 1)]
fn key_block_at_height(
    conn: MiddlewareDbConn,
    state: State<MiddlewareServer>,
    height: i64,
    ) -> Json<String> {
    let key_block = match KeyBlock::load_at_height(&conn, height) {
        Some(x) => x,
        None => {
            info!("Generation not found at height {}", height);
            return Json(serde_json::to_string(&state.epoch.get_generation_at_height(height).unwrap()).unwrap())
        }
    };
    info!("Serving key block {} from DB", height);
    Json(serde_json::to_string(&JsonKeyBlock::from_key_block(&key_block)).unwrap())
}

#[get("/transactions/<hash>")]
fn transaction_at_hash(
    conn: MiddlewareDbConn,
    state: State<MiddlewareServer>,
    hash: String,
) -> Json<serde_json::Value> {
    let tx: Transaction = match Transaction::load_at_hash(&conn, &hash) {
        Some(x) => x,
        None => {
            info!("Transaction not found at hash {}", &hash);
            let mut path = std::path::PathBuf::new();
            path.push(format!("/transactions/hash/{}", hash));
            return epoch_get_handler(state, path);
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
fn key_block_at_hash(
    conn: MiddlewareDbConn,
    state: State<MiddlewareServer>,
    hash: String,
) -> Json<serde_json::Value> {
    let key_block = match KeyBlock::load_at_hash(&conn, &hash) {
        Some(x) => x,
        None => {
            info!("Key block not found at hash {}", &hash);
            let mut path = std::path::PathBuf::new();
            path.push(format!("/key-blocks/hash/{}", hash));
            return epoch_get_handler(state, path);
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
    conn: MiddlewareDbConn,
    _state: State<MiddlewareServer>,
    hash: String,
) -> Json<JsonTransactionList> {
    let sql = format!("select t.* from transactions t, micro_blocks m where t.micro_block_id = m.id and m.hash = '{}'", sanitize(hash));
    let transactions: Vec<Transaction> = sql_query(sql).load(&*conn).unwrap();
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
fn micro_block_header_at_hash(
    conn: MiddlewareDbConn,
    _state: State<MiddlewareServer>,
    hash: String,
) -> Json<JsonValue> {
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
        version: i32 };
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
 * Gets all transactions for an account
 */
#[get("/transactions/account/<account>")]
fn transactions_for_account(
    conn: MiddlewareDbConn,
    _state: State<MiddlewareServer>,
    account: String,
) -> Json<JsonTransactionList> {
    let s_acc = sanitize(account);
    let sql = format!("select * from transactions where tx->>'sender_id'='{}' or tx->>'account_id' = '{}' or tx->>'recipient_id'='{}' or tx->>'owner_id' = '{}' order by id asc",
                          s_acc, s_acc, s_acc, s_acc);
    info!("{}", sql);
    let transactions: Vec<Transaction> = sql_query(sql).load(&*conn).unwrap();
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
#[get("/transactions/interval/<from>/<to>")]
fn transactions_for_interval(
    conn: MiddlewareDbConn,
    _state: State<MiddlewareServer>,
    from: i64,
    to: i64,
) -> Json<JsonTransactionList> {
    let sql = format!("select t.* from transactions t, micro_blocks m, key_blocks k where t.micro_block_id=m.id and m.key_block_id=k.id and k.height >={} and k.height <= {} order by k.height asc", from, to);
    let transactions: Vec<Transaction> = sql_query(sql).load(&*conn).unwrap();
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
    conn: MiddlewareDbConn,
    _state: State<MiddlewareServer>,
    hash: String
) -> Json<JsonValue> {
    Json(json!({
        "count": MicroBlock::get_transaction_count(&SQLCONNECTION.get().unwrap(), &hash, ),
    }))
}



impl MiddlewareServer {
    pub fn start(self) {
        let allowed_origins = AllowedOrigins::all();
        let options = rocket_cors::Cors {
            allowed_origins,
            allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            ..Default::default()
        };

        rocket::ignite()
            .mount("/middleware", routes![transactions_for_account])
            .mount("/middleware", routes![transactions_for_interval])
//            .mount("/middleware", routes![key_block_gas_price])
            .mount("/v2", routes![current_generation])
            .mount("/v2", routes![current_key_block])
            .mount("/v2", routes![epoch_get_handler])
            .mount("/v2", routes![epoch_post_handler])
            .mount("/api", routes![epoch_api_handler])
            .mount("/v2", routes![generation_at_height])
            .mount("/v2", routes![key_block_at_height])
            .mount("/v2", routes![key_block_at_hash])
            .mount("/v2", routes![micro_block_header_at_hash])
            .mount("/v2", routes![transaction_at_hash])
            .mount("/v2", routes![transaction_count_in_micro_block])
            .mount("/v2", routes![transactions_in_micro_block_at_hash])
            .attach(options)
            .attach(MiddlewareDbConn::fairing())
            .manage(self)
            .launch();
    }
}
