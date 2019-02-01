extern crate ws;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use chashmap::*;
use super::models::WsMessage;
use super::serde_json;
use self::ws::{listen, CloseCode, Sender, Message, Handshake, Handler, Result, Error};

#[derive(Clone)]
struct Server {
    out: Sender,
    rules: CHashMap<String, bool>,
}

lazy_static! {
    static ref client_list: Arc<Mutex<Option<Vec<Server>>>> = Arc::new(Mutex::new(Some(vec![])));
}

impl Handler for Server {

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        debug!("WebSocket closing for ({:?}) {}", code, reason);
        self.out.shutdown().unwrap();
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        match client_list.lock().unwrap().as_mut() {
            Some(list) => {
                for client in list.into_iter() {
                    let value: WsMessage = unpack_message(msg.clone());
                    client.rules.insert(value.value, true);
                }
            },
            None => {},
        }
        Ok(())
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        match client_list.lock().unwrap().as_mut() {
            Some(list) => list.push(self.clone()),
            None => {},
        }
        self.out.send("connected")
    }
}

pub fn unpack_message(msg: Message) -> WsMessage {
    let value = msg.into_text().unwrap();
    serde_json::from_str(&value).unwrap()
}

pub fn start_ws() {
	let server = thread::spawn(move || {
        listen("0.0.0.0:3020", |out| {
            let rules: CHashMap<String, bool> = CHashMap::<String, bool>::new();
            Server { out: out, rules: rules }
        }).unwrap();
    });
    sleep(Duration::from_millis(10)); // waiting for server to initialize fully
    let _ = server.join();
}


pub fn broadcast_ws(rule: String, data: String) {
    match client_list.lock().unwrap().as_ref() {
            Some(list) => {
                for client in list.into_iter() {
                     match client.rules.get(&rule) {
                        Some(value) => { client.out.send(data.clone()); },
                        None => {},
                    }
                }
            },
            None => {},
    };
}