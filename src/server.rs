extern crate futures;

extern crate hyper;

use diesel::sql_query;

use diesel::pg::PgConnection;
use diesel::RunQueryDsl;    
use rocket;
use rocket::State;
use std::sync::{Arc, };

use epoch::from_json;

use epoch::Epoch;
use models::{Transaction, JsonTransaction, JsonTransactionList, };

use serde_json;

use r2d2::{Pool, };
use r2d2_diesel::ConnectionManager;

pub struct MiddlewareServer {
    pub epoch: Epoch,
    pub dest_url: String, // address to forward to
    pub port: u16, // port to listen on
    pub connection: Arc<Pool<ConnectionManager<PgConnection>>>, // DB connection
}

fn sanitize(s: String) -> String {
    s
}

#[get("/transactions/account/<account>")]
fn transactions_for_account(state: State<MiddlewareServer>, account: String) -> String {
    let sql = format!("select * from transactions where (tx->>'type' = 'SpendTx' AND tx->>'sender_id'='{}')", sanitize(account));
    let transactions: Vec<Transaction> = sql_query(sql).load(&*state.connection.get().unwrap()).unwrap();
    let mut trans: Vec<JsonTransaction> = vec!();
    for i in 0 .. transactions.len() {
        trans.push(JsonTransaction::from_transaction(&transactions[i]));
    }
    let list = JsonTransactionList {
        transactions: trans,
    };
    serde_json::to_string(&list).unwrap()
}

impl MiddlewareServer {
    pub fn start(self) {
        rocket::ignite()
            .mount("/", routes![transactions_for_account])
            .manage(self)
            .launch();
    }

}
