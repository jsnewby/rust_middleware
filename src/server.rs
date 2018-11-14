extern crate futures;

extern crate hyper;
use futures::future;

use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, Uri};

use diesel::sql_query;

use diesel::pg::PgConnection;

use rocket;
use rocket::State;
use std::sync::{Mutex};

extern crate http;
// use http::request::{Parts, };

use epoch::Epoch;
use models::{Transaction, };
type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

pub struct MiddlewareServer {
    pub epoch: Epoch,
    pub dest_url: String, // address to forward to
    pub port: u16, // port to listen on
    pub connection: PgConnection, // DB connection
}

pub type LockedMS = Mutex<MiddlewareServer>;

#[get("/")]
fn transactions_for_account(state: State<LockedMS>) -> String {
    String::from("foo")
}

impl MiddlewareServer {
    pub fn start(&self) {
        rocket::ignite()
            .mount("/", routes![server::transactions_for_account])
            .manage(Mutex::new(self))
            .launch();
    }

}
