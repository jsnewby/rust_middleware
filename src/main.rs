#![feature(proc_macro_hygiene, decl_macro)]
#![feature(custom_attribute)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)] 

extern crate crypto;
extern crate hex;
extern crate blake2b;
extern crate rand;

extern crate regex;
extern crate http;

extern crate curl;
extern crate rust_base58;
extern crate rust_sodium;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate diesel;

// Connection Pool lib + plugin for Diesel
extern crate r2d2;
extern crate r2d2_diesel;

extern crate dotenv;
extern crate bigdecimal;
pub use bigdecimal::BigDecimal;

extern crate itertools;

extern crate futures;
extern crate hyper;

extern crate clap;
use clap::{App, Arg, };

pub mod epoch;
pub mod schema;
pub mod server;
use server::MiddlewareServer;

//use std::sync::{Mutex, Arc, };

pub mod models;

fn main() {
    let matches = App::new("æternity middleware")
        .version("0.1")
        .author("John Newby <john@newby.org>")
        .about("----")
        .arg(Arg::with_name("url")
             .short("u").long("url").value_name("URL")
             .help("URL of æternity node.").takes_value(true))
        .arg(Arg::with_name("start_hash")
             .short("h").long("start").value_name("START_HASH")
             .help("Hash to start from.").takes_value(true))
        .arg(Arg::with_name("server")
             .short("s").long("server")
             .help("Start server").takes_value(false))
         .arg(Arg::with_name("populate")
             .short("p").long("populate")
             .help("Populate DB").takes_value(false))
        .get_matches();

    let url = match matches.value_of("url") {
        Some(u) => u,
        None => "https://sdk-testnet.aepps.com",
    };
    let connection = epoch::establish_connection();
    let epoch = epoch::Epoch::new(String::from(url));

    let populate = matches.is_present("populate");
    if populate {
        println!("Connecting to æternity node with URL {}", url);
        let current_generation = epoch.current_generation().unwrap();
        let top_hash = match matches.value_of("start") {
            Some(h) => String::from(h),
            None => String::from(epoch::from_json(&current_generation["key_block"]["hash"].to_string())),
        };
        println!("Starting from hash: {}", top_hash);
        epoch::populate_db(&connection.get().unwrap(), &epoch, top_hash).unwrap();
    }      

    let serve = matches.is_present("server");
    if serve {
        let ms: MiddlewareServer = MiddlewareServer {
            epoch: epoch,
            dest_url: String::from("https://sdk-testnet.aepps.com"),
            port: 3013,
            connection: connection,
        };
        ms.start();
    }
        if !populate && !serve {
        println!("Nothing to do!");
    }    
}
