#![allow(redundant_semicolon)]
#![feature(plugin)]
#![feature(slice_concat_ext)]
#![feature(custom_attribute)]
#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
extern crate backtrace;
extern crate base58;
extern crate base58check;
extern crate base64;
extern crate bigdecimal;
extern crate blake2;
extern crate byteorder;
extern crate chashmap;
extern crate chrono;
extern crate clap;
extern crate crypto;
extern crate curl;
extern crate daemonize;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;
extern crate env_logger;
extern crate flexi_logger;
extern crate futures;
extern crate hex;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate log4rs_email;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate r2d2_postgres;
extern crate rand;
extern crate regex;
extern crate reqwest;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;
extern crate rust_base58;
extern crate rust_decimal;
#[macro_use]
extern crate serde_derive;
extern crate postgres;
extern crate serde_json;
extern crate ws;

extern crate aepp_middleware;

use std::thread;
use std::thread::JoinHandle;

use clap::clap_app;
use log::LevelFilter;
use log4rs::config::{Appender, Config, Root};
use std::env;

pub mod coinbase;
pub mod compiler;
pub mod hashing;
pub mod loader;
pub mod middleware_result;
pub mod node;
pub mod schema;
pub mod server;
pub mod websocket;

pub use bigdecimal::BigDecimal;
use loader::BlockLoader;
use middleware_result::MiddlewareResult;
use server::MiddlewareServer;
pub mod models;

use daemonize::Daemonize;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

embed_migrations!("migrations/");

use loader::PGCONNECTION;

/*
 * This function does two things--initially it asks the DB for the
* heights not present between 0 and the height returned by
* /generations/current.  After it has queued all of them it spawns the
* detect_forks thread, then it starts the blockloader, which does not
* return.
*/

fn fill_missing_heights(url: String, _tx: std::sync::mpsc::Sender<i64>) -> MiddlewareResult<bool> {
    debug!("In fill_missing_heights()");
    let node = node::Node::new(url.clone());
    let top_block = node::key_block_from_json(node.latest_key_block()?)?;
    let missing_heights = node.get_missing_heights(top_block.height)?;
    for height in missing_heights {
        debug!("Adding {} to load queue", &height);
        match loader::queue(height as i64, &_tx) {
            Ok(_) => (),
            Err(x) => {
                error!("Error queuing block to send: {}", x);
                BlockLoader::recover_from_db_error();
            }
        };
    }
    _tx.send(loader::BACKLOG_CLEARED)?;
    Ok(true)
}

fn init_logging() {
    match env::var("LOG_CONF") {
        Ok(x) => {
            let mut deserializers = log4rs::file::Deserializers::default();
            log4rs_email::log4rs_email::register(&mut deserializers);
            let _result = log4rs::init_file(x, deserializers).unwrap();
        }
        Err(_) => {
            let stdout = log4rs::append::console::ConsoleAppender::builder().build();
            let config = Config::builder()
                .appender(Appender::builder().build("console", Box::new(stdout)))
                .build(Root::builder().appender("console").build(LevelFilter::Warn))
                .unwrap();
            let _handle = log4rs::init_config(config).unwrap();
        }
    }
}

fn setup_protocols(url: String) {
    let connection = crate::loader::SQLCONNECTION.get().unwrap();
    let node = crate::node::Node::new(url);
    let status = node.get(&"status".to_string()).unwrap();
    let protocols = status["protocols"].as_array().unwrap();
    let trans = connection.transaction().unwrap();
    connection.execute("DELETE FROM protocols", &[]).unwrap();
    for protocol in protocols {
        let effective_at_height = protocol["effective_at_height"].as_i64().unwrap();
        let version = protocol["version"].as_i64().unwrap();
        connection.execute(
            "INSERT INTO protocols(effective_at_height, version) VALUES ($1, $2)",
            &[&effective_at_height, &version]).unwrap();
    }
    trans.commit().unwrap();
}

fn main() {
    dotenv::dotenv().ok();

    init_logging();

    let matches = clap_app!(mdw =>
        (name: env!("CARGO_PKG_NAME"))
        (version: VERSION)
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
            (@arg server: -s --server "Start the middleware server")
            (@arg populate: -p --populate "Populate the DB")
            (@arg websocket: -w --websocket requires[populate] "Start middleware WebSocket Server")
            (@arg daemonize: -d --daemonize "Daemonize the middleware process")
            (@arg protocols: -P --protocols "Populate protocols table and exit")
            (@arg heights: -H --("load-heights") +takes_value "Load specific heights, values separated by comma, ranges with from-to accepted")
            (@subcommand verify =>
                (name: "verify")
                (about: "Verify middleware DB integrity against the chain")
                (@arg blocks: -b --blocks +takes_value "Key-blocks to verify. Values separated by comma, ranges with from-to and single values are accepted")
                (@arg all: -a --all "Verify all the key blocks in the database. Overrides the -b option.")
            )
    ).get_matches();
    let url = env::var("NODE_URL")
        .expect("NODE_URL must be set")
        .to_string();

    let populate = matches.is_present("populate");
    let serve = matches.is_present("server");
    let heights = matches.is_present("heights");
    let daemonize = matches.is_present("daemonize");
    let websocket = matches.is_present("websocket");
    let protocols = matches.is_present("protocols");

    if protocols {
        setup_protocols(url.clone());
        return;
    }

    if daemonize {
        let daemonize = Daemonize::new();
        if let Ok(x) = env::var("PID_FILE") {
            daemonize.pid_file(x).start().unwrap();
        } else {
            daemonize.start().unwrap();
        }
    }

    if let Some(v_matches) = matches.subcommand_matches("verify") {
        debug!("Verifying");
        let loader = BlockLoader::new(url.clone());
        if v_matches.is_present("all") {
            match loader.verify_all() {
                Ok(_) => (),
                Err(x) => error!("Blockloader::verify() returned an error: {}", x),
            };
            return;
        } else if v_matches.is_present("blocks") {
            let blocks = String::from(v_matches.value_of("blocks").unwrap());
            for height in range(&blocks) {
                match loader.verify_height(height) {
                    Ok(_) => (),
                    Err(e) => error!("Error in hashing::min_b(): {:?}", e),
                }
            }
        }
    }

    // Run migrations if populate or heights set
    if populate || heights {
        let connection = PGCONNECTION.get().unwrap();
        let mut migration_output = Vec::new();
        let migration_result =
            embedded_migrations::run_with_output(&*connection, &mut migration_output);
        for line in migration_output.iter() {
            info!("migration out: {}", line);
        }
        migration_result.unwrap();
        setup_protocols(url.clone());
    }

    /*
     * The `heights` argument is of this form: 1,10-15,1000 which
     * would cause blocks 1, 10,11,12,13,14,15 and 1000 to be loaded.
     */
    if heights {
        let to_load = matches.value_of("heights").unwrap();
        let loader = BlockLoader::new(url.clone());
        for h in range(&String::from(to_load)) {
            loader.load_blocks(h).unwrap();
        }
    }

    let mut populate_thread: Option<JoinHandle<()>> = None;

    /*
     * We start 3 populate processes--one queries for missing heights
     * and works through that list, then exits. Another polls for
     * new blocks to load, then sleeps and does it again, and yet
     * another reads the mempool (if available).
     */
    if populate {
        let url = url.clone();
        let loader = BlockLoader::new(url.clone());
        match fill_missing_heights(url.clone(), loader.tx.clone()) {
            Ok(_) => (),
            Err(x) => error!("fill_missing_heights() returned an error: {}", x),
        };
        populate_thread = Some(thread::spawn(move || {
            loader.start();
        }));
        if websocket {
            websocket::start_ws();
        }
    }

    if serve {
        let ms: MiddlewareServer = MiddlewareServer {
            node: node::Node::new(url.clone()),
            dest_url: url.to_string(),
            port: 3013,
        };
        ms.start();
        loop {
            // just to stop main() thread exiting.
            thread::sleep(std::time::Duration::new(40, 0));
        }
    }
    if !populate && !serve && !heights {
        warn!("Nothing to do!");
    }

    /*
     * If we have a populate thread running, wait for it to exit.
     */
    match populate_thread {
        Some(x) => {
            x.join().unwrap();
            ()
        }
        None => (),
    }
}

// takes args of the form X,Y-Z,A and returns a vector of the individual numbers
// ranges in the form X-Y are INCLUSIVE
fn range(arg: &String) -> Vec<i64> {
    let mut result = vec![];
    for h in arg.split(',') {
        let s = String::from(h);
        match s.find("-") {
            Some(_) => {
                let fromto: Vec<String> = s.split('-').map(|x| String::from(x)).collect();
                for i in fromto[0].parse::<i64>().unwrap()..fromto[1].parse::<i64>().unwrap() + 1 {
                    result.push(i);
                }
            }
            None => {
                result.push(s.parse::<i64>().unwrap());
            }
        }
    }
    result
}

#[test]
fn test_range() {
    assert_eq!(range(&String::from("1")), vec!(1));
    assert_eq!(range(&String::from("2-5")), vec!(2, 3, 4, 5));
    assert_eq!(range(&String::from("1,2-5,10")), vec!(1, 2, 3, 4, 5, 10));
}
