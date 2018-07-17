#![feature(plugin)]
#![plugin(rocket_codegen)]

use diesel::prelude::*;
use diesel::sql_types::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use std::env;

pub mod models;
use models::InsertableBlock;
use models::InsertableTransaction;
use models::JsonBlock;
use models::JsonTransaction;


use rocket::*;

struct server {
    PgConnection 

pub mod server {

    #[get("/")]
    fn index() -> &'static str {
        "Hello, world!"
    }
    
    pub fn start() {
        rocket::ignite().mount("/", routes![index]).launch();
    }
}

