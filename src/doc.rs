extern crate curl;
extern crate ws;

use self::curl::easy::Easy;
use std::io::Read;
use std::sync::mpsc;
use ws::CloseCode;

#[derive(Debug)]
enum ConnectionState {
  Unauthenticated,
  Authenticated,
  Opened,
  Finished,
  Errored,
}

impl ConnectionState {
  fn handle(&self, msg: ws::Message) -> ConnectionState {
    match *self {
      ConnectionState::Unauthenticated => ConnectionState::Authenticated,
      ConnectionState::Authenticated => ConnectionState::Opened,
      ConnectionState::Opened => ConnectionState::Opened,
      ConnectionState::Finished => ConnectionState::Finished,
      _ => ConnectionState::Errored,
    }
  }
}

pub struct Doc {
  state: ConnectionState,
  uid: String,
  v: u32,
  sender: ws::Sender,
  thread_tx: mpsc::Sender<u32>,
}

impl Doc {
  pub fn new(uid: &String, sender: ws::Sender, thread_tx: mpsc::Sender<u32>) -> Doc {
    Doc {
      state: ConnectionState::Unauthenticated,
      uid: (*uid).clone(),
      v: 0,
      sender,
      thread_tx
    }
  }

  fn authenticate(&self) -> ws::Result<()> {
    let msg = json!({"auth": null}).to_string();
    self.sender.send(msg)
  }

  fn open_document(&self) -> ws::Result<()> {
    let msg = json!({"doc": self.uid, "create": true, "open": true, "type": "text"}).to_string();
    self.sender.send(msg)
  }

  pub fn insert(&mut self, text: &str) -> ws::Result<()> {
    let msg = json!({"op": [{"i": text, "p": 0}], "v": self.v}).to_string();
    self.v = self.v + 1;
    self.thread_tx.send(1);
    self.sender.send(msg)
  }

  pub fn close(&self) -> ws::Result<()> {
    self.sender.close(CloseCode::Normal)
  }
}

impl ws::Handler for Doc {
  fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
    // println!("connection open");
    self.authenticate()
  }

  fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
    // println!("got msg '{:?}'", msg);
    self.state = self.state.handle(msg);

    if self.v > 1000 {
      self.state = ConnectionState::Finished;
    }

    // println!("state: {:?}", self.state);

    match self.state {
      ConnectionState::Authenticated => self.open_document(),
      ConnectionState::Opened => self.insert("what r next"),
      _ => self.close()
    }
  }
}
