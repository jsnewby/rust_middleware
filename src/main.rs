#![feature(slice_concat_ext)]
#![feature(proc_macro_hygiene, decl_macro)]
#![feature(custom_attribute)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

extern crate rand;

extern crate bigdecimal;
extern crate blake2b;
extern crate crypto;
extern crate curl;
#[macro_use]
extern crate diesel;
extern crate dotenv;
pub use bigdecimal::BigDecimal;
extern crate hex;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate regex;
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cors;
extern crate rust_base58;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::thread;
extern crate itertools;

extern crate futures;
extern crate postgres;

extern crate clap;
use clap::{App, Arg};

pub mod epoch;
pub mod loader;
pub mod schema;
pub mod server;

use loader::BlockLoader;
use server::MiddlewareServer;

pub mod models;

fn main() {
    let matches = App::new("æternity middleware")
        .version("0.1")
        .author("John Newby <john@newby.org>")
        .about("----")
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .value_name("URL")
                .help("URL of æternity node.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("start_hash")
                .short("h")
                .long("start_hash")
                .value_name("START_HASH")
                .help("Hash to start from.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .help("Start server")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("populate")
                .short("p")
                .long("populate")
                .help("Populate DB")
                .takes_value(false),
        )
        .get_matches();

    let url = match matches.value_of("url") {
        Some(u) => u,
        None => "https://sdk-testnet.aepps.com",
    };
    let connection = epoch::establish_connection();
    let epoch = epoch::Epoch::new(String::from(url));

    let populate = matches.is_present("populate");
    let serve = matches.is_present("server");

    /*
     * we start 3 populate processes--one queries for missing heights
     * and works through that list, then exits. Another polls for
     * new blocks to load, then sleeps and does it again, and yet 
     * another reads the mempool (if available).
     */
    if populate {
        let u = String::from(url);
        let u2 = u.clone();
        thread::spawn(move || {
            let loader = BlockLoader::new(epoch::establish_connection(), String::from(u));
            let epoch = epoch::Epoch::new(u2.clone());
            loop {
                loader.load_mempool(&epoch);
                thread::sleep(std::time::Duration::new(5,0));
            }
        });

        let u = String::from(url);
        let u2 = u.clone();
        thread::spawn(move || {
            let loader = BlockLoader::new(epoch::establish_connection(), String::from(u));
            let epoch = epoch::Epoch::new(u2.clone());
            let top_block = epoch::key_block_from_json(epoch.latest_key_block().unwrap()).unwrap();
            let missing_heights = epoch::get_missing_heights(top_block.height);
            for height in missing_heights {
                debug!("Adding {} to load queue", &height);
                loader.tx.send(height as i64);
            }
            loader.start();
        });
        let u = String::from(url);
        let u2 = u.clone();
        thread::spawn(move || {
            let loader = BlockLoader::new(epoch::establish_connection(), String::from(u));
            let tx = loader.tx.clone();
            thread::spawn(move || {
                let epoch = epoch::Epoch::new(u2.clone());
                loop {
                    debug!("Scanning for new blocks");
                    loader::BlockLoader::scan(&epoch, &tx);
                    debug!("Sleeping.");
                    thread::sleep_ms(40000);
                }
            });
            loader.start();
        });
    }

    if serve {
        let ms: MiddlewareServer = MiddlewareServer {
            epoch,
            dest_url: String::from("https://sdk-testnet.aepps.com"),
            port: 3013,
            connection,
        };
        ms.start();
    }
    if !populate && !serve {
        warn!("Nothing to do!");
    }
    loop {
        thread::sleep_ms(40000);
    }
}
