extern crate ws;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use chashmap::*;
use super::models::{WsMessage, WsOp, WsPayload};
use super::serde_json;
use self::ws::{listen, CloseCode, Sender, Message, Handshake, Handler, Result};

#[derive(Clone)]
struct Client {
    out: Sender,
    rules: CHashMap<WsPayload, bool>,
}

lazy_static! {
    static ref CLIENT_LIST: Arc<Mutex<Option<Vec<Client>>>> = Arc::new(Mutex::new(Some(vec![])));
}

impl Handler for Client {

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("WebSocket closing for ({:?}) {}", code, reason);
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        match CLIENT_LIST.lock().unwrap().as_mut() {
            Some(list) => {
                for client in list.into_iter() {
                    if self.out == client.out {
                        let value: WsMessage = unpack_message(msg.clone());
                        if value.payload.is_some() {
                            match value.op {
                                Some(WsOp::subscribe) => {
                                    match value.payload {
                                        Some(WsPayload::key_blocks) => { client.rules.insert(WsPayload::key_blocks, true); },
                                        Some(WsPayload::micro_blocks) => { client.rules.insert(WsPayload::micro_blocks, true); },
                                        Some(WsPayload::transactions) => { client.rules.insert(WsPayload::transactions, true); },
                                        Some(WsPayload::tx_update) => { client.rules.insert(WsPayload::tx_update, true); },
                                        _ => {}
                                    }
                                },
                                Some(WsOp::unsubscribe) => { client.rules.remove(&value.payload.unwrap()); break; },
                                _ => {},
                            }
                        }
                    }
                }
            },
            _ => {},
        }
        Ok(())
    }

    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        match CLIENT_LIST.lock().unwrap().as_mut() {
            Some(list) => list.push(self.clone()),
            _ => {},
        }
        self.out.send("connected")
    }
}

pub fn unpack_message(msg: Message) -> WsMessage {
    let value = msg.into_text().unwrap_or("{}".to_string());
    serde_json::from_str(&value).expect("Unable to deserialize")
}

pub fn start_ws() {
	let server = thread::spawn(move || {
        listen("0.0.0.0:3020", |out| {
            let rules: CHashMap<WsPayload, bool> = CHashMap::<WsPayload, bool>::new();
            Client { out: out, rules: rules }
        }).expect("Unable to start the websocket server");
    });
    sleep(Duration::from_millis(10)); // waiting for server to initialize fully
    let _ = server.join();
}


pub fn broadcast_ws(rule: WsPayload, data: String) {
    match CLIENT_LIST.lock().unwrap().as_ref() {
            Some(list) => {
                for client in list.into_iter() {
                     match client.rules.get(&rule) {
                        Some(_value) => { client.out.send(data.clone()); },
                        _ => {},
                    }
                }
            },
            _ => {},
    };
}