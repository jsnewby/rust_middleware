extern crate ws;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::Mutex;
use self::ws::{listen, CloseCode, Sender, Handshake, Handler, Result};

struct Server {
        out: Sender,
}

//TODO: create a client struct and use that instead
lazy_static! {
    static ref client_list: Mutex<Vec<Sender>> = Mutex::new(vec![]);
}

impl Handler for Server {

        fn on_close(&mut self, code: CloseCode, reason: &str) {
            println!("WebSocket closing for ({:?}) {}", code, reason);
            self.out.shutdown().unwrap();
        }

        fn on_open(&mut self, shake: Handshake) -> Result<()> {
            let sender: Sender = self.out.clone();
            client_list.lock().unwrap().push(sender);
            self.out.send("connected")
        }
    }

pub fn start_ws() {
	let server = thread::spawn(move || {
        listen("0.0.0.0:3020", |out| {

            Server { out: out }

        }).unwrap()
    });
    sleep(Duration::from_millis(10)); // waiting for server to initialize fully
    let _ = server.join();
}

//TODO: make it more generic
pub fn broadcast_ws(data: String) {
	let clients = client_list.lock().unwrap().clone();
	for client in clients {
		client.send(data.clone());
	}
}