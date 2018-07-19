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

    #[get("/middle/reverse-name")]
    fn reverse_name() -> &'static str {
        
    }
    
    pub fn start() {
        let mut git_routes = vec![];
        for method in &[Get, Put, Post, Delete, Options, Head, Trace, Connect, Patch] {
            epoch_routes.push(Route::new(*method, "/", git_http));
        }
        rocket.ignite().mount("/v2", epoch_routes).launch()
}

