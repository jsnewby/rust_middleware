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
extern crate env_logger;
pub use bigdecimal::BigDecimal;
extern crate hex;
#[macro_use]
extern crate lazy_static;
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

use dotenv::dotenv;
use std::env;

pub mod epoch;
pub mod loader;
pub mod schema;
pub mod server;

use loader::BlockLoader;
use server::MiddlewareServer;

pub mod models;

fn start_blockloader(url: &String,  _tx: std::sync::mpsc::Sender<i64>) {
    debug!("In start_blockloader()");
    let u = url.clone();
    let u2 = url.clone();
    thread::spawn(move || {
        thread::spawn(move || {
            let epoch = epoch::Epoch::new(u2.clone(), 1);
            loop {
                debug!("Scanning for new blocks");
                loader::BlockLoader::scan(&epoch, &_tx);
                debug!("Sleeping.");
                thread::sleep_ms(40000);
            }
        });
    });
}
    
fn load_mempool(url: &String) {
    debug!("In load_mempool()");
    let u = url.clone();
    let u2 = u.clone();
    let loader = BlockLoader::new(epoch::establish_connection(1), String::from(u));
    thread::spawn(move || {
        let epoch = epoch::Epoch::new(u2.clone(), 1);
        loop {
            loader.load_mempool(&epoch);
            thread::sleep(std::time::Duration::new(5,0));
        }
    });
}

/*
 * This function does two things--initially it asks the DB for the
* heights not present between 0 and the height returned by
* /generations/current.  After it has quued all of them it spawns the
* detect_forks thread, then it starts the blockloader, which does not
* return.
*/  
fn fill_missing_heights(url: String, _tx: std::sync::mpsc::Sender<i64>) {
    debug!("In fill_missing_heights()");
    let u = url.clone();
    let u2 = u.clone();
    let handle = thread::spawn(move || {
        let loader = BlockLoader::new(epoch::establish_connection(1), String::from(u));
        let epoch = epoch::Epoch::new(u2.clone(), 1);
        let top_block = epoch::key_block_from_json(epoch.latest_key_block().unwrap()).unwrap();
        let missing_heights = epoch::get_missing_heights(top_block.height);
        for height in missing_heights {
            debug!("Adding {} to load queue", &height);
            _tx.send(height as i64);
        }
        detect_forks(&url.clone(), _tx.clone());
        loader.start();
    });
}

/*
 * Detect forks iterates through the blocks in the DB asking for them and checking
 * that they match what we have in the DB. 
 */
fn detect_forks(url: &String, _tx: std::sync::mpsc::Sender<i64>) {
    debug!("In detect_forks()");
    let u = url.clone();
    let u2 = u.clone();
    thread::spawn(move || {
        thread::spawn(move || {
            let epoch = epoch::Epoch::new(u2.clone(), 1);
            loop {
                debug!("Going into fork detection");
                loader::BlockLoader::detect_forks(&epoch, &_tx);
                debug!("Sleeping.");
                thread::sleep_ms(40000);
            }
        });
    });
}

fn main() {
    env_logger::init();
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

    
    let url = env::var("EPOCH_URL").expect("EPOCH_URL must be set").
        to_string();;
    let connection = epoch::establish_connection(1);

    let populate = matches.is_present("populate");
    let serve = matches.is_present("server");

    /*
     * we start 3 populate processes--one queries for missing heights
     * and works through that list, then exits. Another polls for
     * new blocks to load, then sleeps and does it again, and yet 
     * another reads the mempool (if available).
     */
    if populate {
        let url = url.clone();
        let loader = BlockLoader::new(epoch::establish_connection(1), url.clone());
        load_mempool(&url);
        fill_missing_heights(url.clone(), loader.tx.clone());
        start_blockloader(&url, loader.tx.clone());
        loader.start();
    }

    if serve {
        let ms: MiddlewareServer = MiddlewareServer {
            epoch: epoch::Epoch::new(url.clone(), 20),
            dest_url: url.to_string(),
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
