use super::models::{WsMessage, WsOp, WsPayload};
use super::serde_json;
use regex::Regex;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::thread;
use ws::{listen, CloseCode, Handler, Handshake, Message, Result, Sender};

use crate::middleware_result::MiddlewareResult;

pub struct Candidate {
    pub payload: WsPayload,
    pub data: serde_json::Value,
}

type Object = String;
type VanillaSub = HashSet<Client>;
type ObjectSubFwd = HashMap<Client, HashSet<Object>>;
type ObjectSubRvs = HashMap<Object, VanillaSub>;

pub struct Subscriptions {
    pub kb_sub: VanillaSub,
    pub mb_sub: VanillaSub,
    pub tx_sub: VanillaSub,
    pub tu_sub: VanillaSub,
    pub object_subs_fwd: ObjectSubFwd,
    pub object_subs_rvs: ObjectSubRvs,
}

impl Subscriptions {
    pub fn new() -> Self {
        Self {
            kb_sub: VanillaSub::new(),
            mb_sub: VanillaSub::new(),
            tx_sub: VanillaSub::new(),
            tu_sub: VanillaSub::new(),
            object_subs_fwd: ObjectSubFwd::new(),
            object_subs_rvs: ObjectSubRvs::new(),
        }
    }

    pub fn get_subscription(&self, kind: WsPayload) -> Option<VanillaSub> {
        match kind {
            WsPayload::key_blocks => Some(self.kb_sub.clone()),
            WsPayload::micro_blocks => Some(self.mb_sub.clone()),
            WsPayload::transactions => Some(self.tx_sub.clone()),
            WsPayload::tx_update => Some(self.tu_sub.clone()),
            _ => None,
        }
    }

    pub fn vanilla_subscribe(&self, kind: WsPayload, client: Client) {
        if let Some(mut sub) = self.get_subscription(kind) {
            sub.insert(client.clone());
        }
    }

    pub fn vanilla_unsubscribe(&self, kind: WsPayload, client: Client) {
        if let Some(mut sub) = self.get_subscription(kind) {
            sub.remove(&client);
        }
    }

    pub fn object_subscribe(&mut self, client: Client, object: Object) {
        let mut v: VanillaSub = match self.object_subs_rvs.get(&object) {
            Some(x) => (*x).to_owned(),
            None => VanillaSub::new().to_owned(),
        };
        v.insert(client.clone());
        self.object_subs_rvs.insert(object.clone(), v);
        let mut objs: HashSet<Object> = match self.object_subs_fwd.get(&client) {
            Some(x) => (*x).to_owned(),
            None => HashSet::new(),
        };
        objs.insert(object);
        self.object_subs_fwd.insert(client, objs);
    }

    pub fn object_unsubscribe(&mut self, client: Client, object: Object) {
        let mut v: VanillaSub = match self.object_subs_rvs.get(&object) {
            Some(x) => (*x).to_owned(),
            None => VanillaSub::new().to_owned(),
        };
        v.remove(&client);
        self.object_subs_rvs.insert(object.clone(), v);
        let mut objs: HashSet<Object> = match self.object_subs_fwd.get(&client) {
            Some(x) => (*x).to_owned(),
            None => HashSet::new(),
        };
        objs.remove(&object);
        self.object_subs_fwd.insert(client, objs);
    }

    pub fn client_unsubscribe(&mut self, client: Client) {
        let mut objs: HashSet<Object> = match self.object_subs_fwd.get(&client) {
            Some(x) => (*x).to_owned(),
            None => HashSet::new(),
        };
        for object in objs.iter() {
            self.object_unsubscribe(client.clone(), object.to_string());
        }
    }
}

#[test]
fn test_subs() {
    let subs = Subscriptions::new();
    let sub = subs.get_subscription(WsPayload::key_blocks).unwrap();
    assert_eq!(sub.len(), 0);
}

lazy_static! {
    static ref SUBSCRIPTIONS: Arc<Mutex<RefCell<Subscriptions>>> =
        Arc::new(Mutex::new(RefCell::new(Subscriptions::new())));
}

pub fn vanilla_subscribe(kind: WsPayload, client: Client) {
    if let Ok(x) = (*SUBSCRIPTIONS).lock() {
        (*x).borrow_mut().vanilla_subscribe(kind, client);
    }
}

pub fn vanilla_unsubscribe(kind: WsPayload, client: Client) {
    if let Ok(x) = (*SUBSCRIPTIONS).lock() {
        (*x).borrow_mut().vanilla_unsubscribe(kind, client);
    }
}

pub fn object_subscribe(client: Client, object: Object) {
    if let Ok(x) = (*SUBSCRIPTIONS).lock() {
        (*x).borrow_mut().object_subscribe(client, object);
    }
}

pub fn object_unsubscribe(client: Client, object: Object) {
    if let Ok(x) = (*SUBSCRIPTIONS).lock() {
        (*x).borrow_mut().object_unsubscribe(client, object);
    }
}

pub fn client_unsubscribe(client: Client) {
    if let Ok(x) = (*SUBSCRIPTIONS).lock() {
        (*x).borrow_mut().client_unsubscribe(client);
    }
}

fn match_objects(objects: Vec<String>, tx: String) -> bool {
    let tx_objects = get_objects(tx);
    for object in objects {
        if tx_objects.contains(&object) {
            return true;
        }
    }
    false
}

pub fn get_objects(tx: String) -> HashSet<String> {
    lazy_static! {
        static ref OBJECT_REGEX: Regex = Regex::new(
            "[a-z][a-z]_[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]{38,60}"
        )
        .unwrap();
    }

    OBJECT_REGEX
        .find_iter(&tx)
        .map(|mat| mat.as_str().to_string())
        .collect()
}

#[test]
fn test_get_objects() {
    let tx: serde_json::Value = serde_json::from_str(
        r#"
{
    "block_hash": "mh_7iCkawgwm9akyXaBaEgfoL2Uhgz9k5b8vbSqx97spp9Ae1mLa",
    "block_height": 85113,
    "hash": "th_2vjbhonApccV6r7PjbR6qojfa2gyZ84xzTzX37g6vXgzZ9UKUn",
    "time": 1558700970133,
    "tx": {
      "amount": 13370000000000000,
      "fee": 16900000000000,
      "nonce": 32,
      "payload": "ba_Xfbg4g==",
      "recipient_id": "ak_gxMtcfvnd7aN9XdpmdNgRRETnLL4TNQ4uJgyLzcbBFa3vx6Da",
      "sender_id": "ak_2eid5UDLCVxNvqL95p9UtHmHQKbiFQahRfoo839DeQuBo8A3Qc",
      "type": "SpendTx",
      "version": 1
    }
  }
"#,
    )
    .unwrap();
    println!("{}", tx.to_string());
    let objects = get_objects(tx.to_string());
    println!("{:?}", objects);
    assert!(objects.contains("mh_7iCkawgwm9akyXaBaEgfoL2Uhgz9k5b8vbSqx97spp9Ae1mLa"));
}

#[derive(Clone, Debug)]
pub struct Client {
    out: Sender,
}

impl PartialEq for Client {
    fn eq(&self, other: &Client) -> bool {
        self.out.token() == other.out.token()
    }
}

impl Eq for Client {}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.out.token().hash(state)
    }
}

impl Handler for Client {
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        debug!("WebSocket closing with code {:?} because {}", code, reason);
        client_unsubscribe(self.clone());
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let value: WsMessage = match unpack_message(msg.to_owned()) {
            Ok(x) => x,
            Err(err) => {
                error!("Error unpacking message: {:?}", err);
                return Err(ws::Error {
                    kind: ws::ErrorKind::Custom(Box::new(err)),
                    details: Cow::from("error unpacking message"),
                });
            }
        };
        debug!("Value is {:?}", value);
        match value.op {
            Some(WsOp::subscribe) => {
                debug!("Subscription with payload {:?}", value.payload);
                match value.payload {
                    Some(WsPayload::key_blocks)
                    | Some(WsPayload::micro_blocks)
                    | Some(WsPayload::transactions)
                    | Some(WsPayload::tx_update) => {
                        vanilla_subscribe(value.payload.unwrap(), self.clone())
                    }
                    Some(WsPayload::object) => {
                        if let Some(target) = value.target {
                            object_subscribe(self.clone(), target)
                        }
                    }
                    _ => (),
                }
            }
            Some(WsOp::unsubscribe) => {
                debug!("Unsubscription with payload {:?}", value.payload);
                match value.payload {
                    Some(WsPayload::key_blocks)
                    | Some(WsPayload::micro_blocks)
                    | Some(WsPayload::transactions)
                    | Some(WsPayload::tx_update) => {
                        vanilla_unsubscribe(value.payload.unwrap(), self.clone())
                    }
                    Some(WsPayload::object) => {
                        if let Some(target) = value.target {
                            object_unsubscribe(self.clone(), target)
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        self.out.send(format!("No rules to send back yet"));
        Ok(())
    }

    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        debug!("Client connected pre");
        self.out.send("connected")?;
        debug!("Returning");
        Ok(())
    }
}

pub fn unpack_message(msg: Message) -> MiddlewareResult<WsMessage> {
    debug!("Received message {:?}", msg);
    let value = msg.into_text()?;
    Ok(serde_json::from_str(&value)?)
}

pub fn start_ws() {
    let _server = thread::spawn(move || {
        let ws_address = env::var("WEBSOCKET_ADDRESS").unwrap_or("0.0.0.0:3020".to_string());
        listen(ws_address, |out| Client { out }).expect("Unable to start the websocket server");
    });
}

/*
 * The function which actually sends the data to clients
 *
 * everything is wrapped in a JSON object with details of the
 * subscription to which it relates.
 */
pub fn broadcast_ws(candidate: &Candidate) -> MiddlewareResult<()> {
    /*
        for client in get_clients() {
            let rules = get_client_rules(&client);
        }
    */
    Ok(())
}

#[test]
fn test_unpack_message() {
    let msg: Message =
        Message::from(r#"{"op":"subscribe", "payload": "micro_blocks"}"#.to_string());
    let ws_msg = unpack_message(msg).unwrap();
    assert_eq!(ws_msg.op.unwrap(), WsOp::subscribe);
    assert_eq!(ws_msg.payload.unwrap(), WsPayload::micro_blocks);
    assert_eq!(ws_msg.target, None);

    let msg = Message::from(r#"{"op":"subscribe", "payload": "object", "target": "ak_2eid5UDLCVxNvqL95p9UtHmHQKbiFQahRfoo839DeQuBo8A3Qc"}"#.to_string());
    let ws_msg = unpack_message(msg).unwrap();
    assert_eq!(ws_msg.op.unwrap(), WsOp::subscribe);
    assert_eq!(ws_msg.payload.unwrap(), WsPayload::object);
    assert_eq!(
        ws_msg.target.unwrap(),
        String::from("ak_2eid5UDLCVxNvqL95p9UtHmHQKbiFQahRfoo839DeQuBo8A3Qc")
    );
}
