#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

use curl::easy::Easy;

use diesel::sql_query;

use diesel::pg::PgConnection;
use diesel::RunQueryDsl;    
use rocket;
use rocket::State;
use rocket::request::Form;
use rocket::response::content;
use rocket_contrib::Json;
use std::sync::{Arc, };

use epoch::from_json;

use epoch::Epoch;
use models::{Transaction, JsonTransaction, JsonTransactionList, };

use serde_json;

use r2d2::{Pool, };
use r2d2_diesel::ConnectionManager;

use std::path::PathBuf;

pub struct MiddlewareServer {
    pub epoch: Epoch,
    pub dest_url: String, // address to forward to
    pub port: u16, // port to listen on
    pub connection: Arc<Pool<ConnectionManager<PgConnection>>>, // DB connection
}

// SQL santitizing method to prevent injection attacks.
fn sanitize(s: String) -> String {
    s.replace("'", "\\'")
}

/*
 * GET handler for Epoch
 */
#[get("/<path..>")]
fn epoch_get_handler(state: State<MiddlewareServer>, path: PathBuf) -> Json<serde_json::Value> {
    Json(state.epoch.get_naked(&String::from("/v2/"),
                               &String::from(path.to_str().unwrap())).unwrap())
}

/*
 * POST handler for Epoch
 */
#[post("/<path..>", format="application/json", data="<body>")]
fn epoch_post_handler(state: State<MiddlewareServer>, path: PathBuf, body: String) -> Json {
    println!("{}", body);
    let response = state.epoch.post_naked(&String::from("/v2/"),
                                          &String::from(path.to_str().unwrap()),
                                          body).unwrap();
    println!("Response: {}", response);
    Json(serde_json::from_str(response.as_str()).unwrap())
}


/*
 * Epoch's only endpoint which lives outside of /v2/...
 */
#[get("/")]
fn epoch_api_handler(state: State<MiddlewareServer>) -> Json<serde_json::Value> {
    Json(state.epoch.get_naked(&String::from("/api"), &String::from("")).unwrap())
}

/*
 * Gets all transactions for an account
 */
#[get("/transactions/account/<account>")]
fn transactions_for_account(state: State<MiddlewareServer>, account: String) -> Json<JsonTransactionList> {
    let sql = format!("select * from transactions where (tx->>'type' = 'SpendTx' AND tx->>'sender_id'='{}')", sanitize(account));
    let transactions: Vec<Transaction> = sql_query(sql).load(&*state.connection.get().unwrap()).unwrap();
    let mut trans: Vec<JsonTransaction> = vec!();
    for i in 0 .. transactions.len() {
        trans.push(JsonTransaction::from_transaction(&transactions[i]));
    }
    let list = JsonTransactionList {
        transactions: trans,
    };
    Json(list)
}

impl MiddlewareServer {
    pub fn start(self) {
        rocket::ignite()
            .mount("/middleware", routes![transactions_for_account])
            .mount("/v2", routes![epoch_get_handler])
            .mount("/v2", routes![epoch_post_handler])
            .mount("/api", routes![epoch_api_handler])
            .manage(self)
            .launch();
    }

}
