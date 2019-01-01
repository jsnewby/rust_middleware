#![feature(slice_concat_ext)]
#![feature(custom_attribute)]
#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)] 
extern crate rand;

extern crate bigdecimal;
extern crate blake2b;
extern crate crypto;
extern crate curl;
#[macro_use] extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate hex;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate r2d2_postgres;
extern crate regex;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate rocket_cors;
extern crate rust_base58;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use std::thread;
extern crate itertools;

extern crate futures;
extern crate postgres;

extern crate clap;
use clap::{App, Arg};

use std::env;

pub mod epoch;
pub mod loader;
pub mod schema;
pub mod server;

use loader::BlockLoader;
use server::MiddlewareServer;
pub use bigdecimal::BigDecimal;

pub mod models;

use dotenv::dotenv;
use diesel::PgConnection;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use r2d2_postgres::{PostgresConnectionManager};
use std::sync::Arc;

lazy_static! {
    static ref PGCONNECTION: Arc<Pool<ConnectionManager<PgConnection>>> = {
        dotenv().ok(); // Grabbing ENV vars
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .max_size(20) // only used for emergencies...
            .build(manager)
            .expect("Failed to create pool.");
        Arc::new(pool)
    };
}        

fn start_blockloader(url: &String, _tx: std::sync::mpsc::Sender<i64>) {
    debug!("In start_blockloader()");
    //    let u = url.clone();
    let u2 = url.clone();
    thread::spawn(move || {
        thread::spawn(move || {
            let epoch = epoch::Epoch::new(u2.clone(), 1);
            loop {
                debug!("Scanning for new blocks");
                loader::BlockLoader::scan(&epoch, &_tx);
                debug!("Sleeping.");
                thread::sleep(std::time::Duration::new(40, 0));
            }
        });
    });
}

fn load_mempool(url: &String) {
    debug!("In load_mempool()");
    let u = url.clone();
    let u2 = u.clone();
    let loader = BlockLoader::new(String::from(u));
    thread::spawn(move || {
        let epoch = epoch::Epoch::new(u2.clone(), 1);
        loop {
            loader.load_mempool(&epoch);
            thread::sleep(std::time::Duration::new(5, 0));
        }
    });
}

/*
 * This function does two things--initially it asks the DB for the
* heights not present between 0 and the height returned by
* /generations/current.  After it has queued all of them it spawns the
* detect_forks thread, then it starts the blockloader, which does not
* return.
*/

fn fill_missing_heights(url: String, _tx: std::sync::mpsc::Sender<i64>) ->
    Result<bool, Box<std::error::Error>>
{
    debug!("In fill_missing_heights()");
    let u = url.clone();
    let u2 = u.clone();

    //TODO refactor this into a function for better error handling
    thread::spawn(move || {
        let loader = BlockLoader::new(String::from(u));
        let epoch = epoch::Epoch::new(u2.clone(), 1);
        let top_block = epoch::key_block_from_json(epoch.latest_key_block().unwrap()).unwrap();
        let missing_heights = match epoch.get_missing_heights(top_block.height) {
            Ok(x) => x,
            Err(x) => {
                error!("Error finding missing blocks {}", x);
                return;
            },
        };
        for height in missing_heights {
            debug!("Adding {} to load queue", &height);
            match _tx.send(height as i64) {
                Ok(_) => (),
                Err(x) => {
                    error!("Error queuing block to send: {}", x);
                    BlockLoader::recover_from_db_error();
                }
            };
        }
        detect_forks(&url.clone(), 1, 10, _tx.clone());
        detect_forks(&url.clone(), 11, 50, _tx.clone());
        detect_forks(&url.clone(), 51, 500, _tx.clone());
        loader.start();
    });
    Ok(true)
}

/*
 * Detect forks iterates through the blocks in the DB asking for them and checking
 * that they match what we have in the DB.
 */
fn detect_forks(url: &String, from: i64, to: i64, _tx: std::sync::mpsc::Sender<i64>) {
    debug!("In detect_forks()");
    let u = url.clone();
    let u2 = u.clone();
    thread::spawn(move || {
        let epoch = epoch::Epoch::new(u2.clone(), 1);
        loop {
            debug!("Going into fork detection");
            loader::BlockLoader::detect_forks(&epoch, from, to, &_tx);
            debug!("Sleeping.");
            thread::sleep(std::time::Duration::new(2, 0));
        }
    });
}

fn main() {
    env_logger::init();
    let matches = App::new("æternity middleware")
        .version("0.1")
        .author("John Newby <john@newby.org>")
        .about("----")
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
        .arg(
            Arg::with_name("verify")
                .short("v")
                .long("verify")
                .help("Verify DB integrity against chain")
                .takes_value(false),
        )
        .get_matches();

    let url = env::var("EPOCH_URL")
        .expect("EPOCH_URL must be set")
        .to_string();
    let populate = matches.is_present("populate");
    let serve = matches.is_present("server");
    let verify = matches.is_present("verify");

    if verify {
        println!("Verifying");
        let loader = BlockLoader::new(url.clone());
        loader.verify();
        return;
    }

    /*
     * We start 3 populate processes--one queries for missing heights
     * and works through that list, then exits. Another polls for
     * new blocks to load, then sleeps and does it again, and yet
     * another reads the mempool (if available).
     */
    if populate {
        let url = url.clone();
        let loader = BlockLoader::new(url.clone());
        load_mempool(&url);
        fill_missing_heights(url.clone(), loader.tx.clone());
        start_blockloader(&url, loader.tx.clone());
        thread::spawn(move || {
            loader.start();
        });
    }

    if serve {
        let ms: MiddlewareServer = MiddlewareServer {
            epoch: epoch::Epoch::new(url.clone(), 20),
            dest_url: url.to_string(),
            port: 3013,
        };
        ms.start();
    }
    if !populate && !serve {
        warn!("Nothing to do!");
    }
    loop {
        thread::sleep(std::time::Duration::new(40, 0));
    }
}
