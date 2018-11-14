extern crate futures;

extern crate hyper;

//use diesel::sql_query;

use diesel::pg::PgConnection;

use rocket;
use rocket::State;
use std::sync::{Arc, };

extern crate http;
// use http::request::{Parts, };

use epoch::Epoch;
//use models::{Transaction, };


use r2d2::{Pool, };
use r2d2_diesel::ConnectionManager;

pub struct MiddlewareServer {
    pub epoch: Epoch,
    pub dest_url: String, // address to forward to
    pub port: u16, // port to listen on
    pub connection: Arc<Pool<ConnectionManager<PgConnection>>>, // DB connection
}

#[get("/")]
fn transactions_for_account(state: State<MiddlewareServer>) -> String {
    String::from("foo")
}

impl MiddlewareServer {
    pub fn start(self) {
        rocket::ignite()
            .mount("/", routes![transactions_for_account])
            .manage(self)
            .launch();
    }

}
