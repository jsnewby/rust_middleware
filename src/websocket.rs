use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::{Arc, Mutex,};
use std::cell::RefCell;
use chashmap::*;
use super::models::{WsMessage, WsOp, WsPayload};
use super::serde_json;
use ws::{listen, CloseCode, Sender, Message, Handshake, Handler, Result};

use crate::middleware_result::MiddlewareResult;

type ClientRules = CHashMap<WsPayload, bool>;

#[derive(Clone, Debug)]
pub struct Client {
    out: Sender,
    rules: ClientRules,
}

lazy_static! {
    static ref CLIENT_LIST: Arc<Mutex<RefCell<Vec<Client>>>> = Arc::new(Mutex::new(RefCell::new(vec!())));
}

pub fn add_client(client: &Client) {
    match (*CLIENT_LIST).lock() {
        Ok(x) => x.borrow_mut().push(client.clone()),
        Err(x) => error!("Error adding client to list {:?}", x),
    };
}

pub fn remove_client(client: &Client) {
    match (*CLIENT_LIST).lock() {
        Ok(x) => x.borrow_mut().retain(|ele| ele != client),
        Err(x) => error!("Error removing client from list {:?}", x),
    };
}

pub fn update_client(client: &Client) -> MiddlewareResult<()>
{
    remove_client(&client);
    add_client(&client);
    Ok(())
}

pub fn get_client(sender: &Sender) -> MiddlewareResult<Option<Client>>
{
    let list = (*CLIENT_LIST).lock()?;
    for ele in list.borrow_mut().iter() {
        if ele.out == *sender {
            return Ok(Some(ele.clone()));
        }
    }
    Ok(None)
}

pub fn client_rules(client: &Client) -> MiddlewareResult<Option<ClientRules>> {
    match get_client(&client.out)? {
        Some(x) => Ok(Some(x.rules.clone())),
        None => Ok(None),
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Client) -> bool
    {
        self.out == other.out
    }
}


impl Handler for Client {

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        debug!("WebSocket closing with code {:?} because {}", code, reason);
        remove_client(&self);
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let mut client = get_client(&self.out).unwrap().unwrap();
        let rules = client_rules(&client).unwrap().unwrap();
        let value: WsMessage = unpack_message(msg.clone()).unwrap();
        match value.op {
            Some(WsOp::subscribe) => {
                match value.payload {
                    Some(WsPayload::key_blocks) => { rules.insert(WsPayload::key_blocks, true); },
                    Some(WsPayload::micro_blocks) => { rules.insert(WsPayload::micro_blocks, true); },
                    Some(WsPayload::transactions) => { rules.insert(WsPayload::transactions, true); },
                    Some(WsPayload::tx_update) => { rules.insert(WsPayload::tx_update, true); },
                    _ => {}
                }
            },
            Some(WsOp::unsubscribe) => {
                rules.remove(&value.payload.unwrap());
            },
            _ => (),
        }
        client.rules = rules;
        update_client(&client);
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
    let value = msg.into_text()?;
    Ok(serde_json::from_str(&value)?)
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


pub fn broadcast_ws(rule: WsPayload, data: String) -> MiddlewareResult<()>
{
    match CLIENT_LIST.lock() {
        Ok(x) => {
            let list = &*x;
            for client in list.borrow_mut().iter() {
                match client.rules.get(&rule) {
                    Some(_value) => { client.out.send(data.clone())?; },
                    _ => {},
                }
            }
        },
        Err(x) => error!("Couldn't find client! {:?}", x),
    }
    Ok(())
}
