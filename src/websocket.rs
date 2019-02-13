use std::collections::HashMap;
use std::thread;
use std::borrow::Cow;
use std::thread::sleep;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use std::sync::{Arc, Mutex,};
use std::cell::RefCell;
use chashmap::*;
use super::models::{WsMessage, WsOp, WsPayload};
use super::serde_json;
use ws::{listen, CloseCode, Sender, Message, Handshake, Handler, Result,};
use ws::util::Token;

use crate::middleware_result::MiddlewareResult;

type ClientRules = CHashMap<WsPayload, bool>;
type ClientMap = HashMap<Client, ClientRules>;

#[derive(Clone, Debug)]
pub struct Client {
    out: Sender,
}

impl PartialEq for Client {
    fn eq(&self, other: &Client) -> bool
    {
        self.out.token() == other.out.token()
    }
}

impl Eq for Client { }

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.out.token().hash(state)
    }
}

lazy_static! {
    static ref CLIENT_LIST: Arc<Mutex<RefCell<ClientMap>>> = Arc::new(Mutex::new(RefCell::new(ClientMap::new())));
}

pub fn add_client(client: &Client) {
    if let Ok(x) = (*CLIENT_LIST).lock() {
        x.borrow_mut().insert(client.clone(), ClientRules::new());
    };
}

pub fn remove_client(client: &Client) {
    if let Ok(x) = (*CLIENT_LIST).lock() {
        x.borrow_mut().remove(&client);
    };
}

pub fn update_client(client: &Client, rules: &ClientRules)
{
    if let Ok(x) = (*CLIENT_LIST).lock() {
        x.borrow_mut().insert(client.clone(), rules.clone());
    };
}

pub fn get_client_rules(client: &Client) -> MiddlewareResult<ClientRules>
{
    if let Ok(x) = (*CLIENT_LIST).lock() {
        match x.borrow_mut().get(&client) {
            Some(y) => return Ok(y.clone()),
            _ => Ok(ClientRules::new()),
        }
    } else {
        Ok(ClientRules::new()) // TODO check this is correct
    }
}

pub fn get_clients() -> Vec<Client> {
    if let Ok(x) = (*CLIENT_LIST).lock() {
        x.borrow().keys().map(|c| c.clone()).collect()
    } else {
        error!("Couldn't get client list");
        vec!()
    }
}

impl Handler for Client {

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        debug!("WebSocket closing with code {:?} because {}", code, reason);
        remove_client(&self);
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let rules = get_client_rules(&self).unwrap_or(ClientRules::new());
        let value: WsMessage = match unpack_message(msg.clone()) {
            Ok(x) => x,
            Err(err) => {
                error!("Error unpacking message: {:?}", err);
                return Err(ws::Error { kind: ws::ErrorKind::Custom(Box::new(err)),
                                       details: Cow::from("Foo") });
            }
        };
        debug!("Value is {:?}", value);
        match value.op {
            Some(WsOp::subscribe) => {
                debug!("Subscription with payload {:?}", value.payload);
                match value.payload {
                    Some(WsPayload::key_blocks) => { rules.insert(WsPayload::key_blocks, true); },
                    Some(WsPayload::micro_blocks) => { rules.insert(WsPayload::micro_blocks, true); },
                    Some(WsPayload::transactions) => { rules.insert(WsPayload::transactions, true); },
                    Some(WsPayload::tx_update) => { rules.insert(WsPayload::tx_update, true); },
                    _ => {}
                }
            },
            Some(WsOp::unsubscribe) => {
                if let Some(x) = value.payload {
                    rules.remove(&x);
                }
            },
            _ => (),
        }
        update_client(&self, &rules);
        debug!("Client with token {:?} has rules {:?}", self.out.token(), rules);
        Ok(())
    }

    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        debug!("Client connected pre");
        add_client(&self.clone());
        self.out.send("connected")?;
        debug!("Returning");
        Ok(())
    }
}

pub fn unpack_message(msg: Message)
                      -> crate::middleware_result::MiddlewareResult<WsMessage>
{
    debug!("Received message {:?}", msg);
    let value = msg.into_text()?;
    Ok(serde_json::from_str(&value)?)
}

pub fn start_ws() {
	let server = thread::spawn(move || {
        listen("0.0.0.0:3020", |out| {
            Client { out: out }
        }).expect("Unable to start the websocket server");
    });
    sleep(Duration::from_millis(10)); // waiting for server to initialize fully
    let _ = server.join();
}


pub fn broadcast_ws(rule: WsPayload, data: String) -> MiddlewareResult<()>
{
    for client in get_clients() {
        if let Ok(rules) = get_client_rules(&client) {
            match rules.get(&rule) {
                Some(_value) => { client.out.send(data.clone())?; },
                _ => {},
            }
        }
    }
    Ok(())
}
