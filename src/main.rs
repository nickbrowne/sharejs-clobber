extern crate rand;
extern crate time;
extern crate ws;

#[macro_use]
extern crate serde_json;

mod doc;

use doc::Doc;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::sync::mpsc::{channel};
use std::thread;
use ws::{connect, CloseCode};

const CONCURRENCY: usize = 4;

fn random_alpha() -> String {
  thread_rng().sample_iter(&Alphanumeric).take(16).collect()
}

fn main() {
  let mut requests = 0;

  let document_uids: Vec<String> = (0..CONCURRENCY).into_iter().map(|_| random_alpha()).collect();

  let (tx, rx) = channel();

  for uid in document_uids {
    let thread_tx = tx.clone();

    thread::spawn(move || {
      connect("ws://localhost/websocket", |out| {
        Doc::new(&uid, out, thread_tx.clone())
      }).expect("Failed to connect to server");
    });
  }

  // because we cloned it? otherwise the iterator waits indefinitely
  // for the channel to be closed
  drop(tx);

  let start_time = time::precise_time_s();

  println!("Clobbering sharejs!");
  println!("------------------");

  let mut rx_iter = rx.iter();

  while let Some(result) = rx_iter.next() {
    let curr_time = time::precise_time_s();
    let elapsed_time = curr_time - start_time;

    requests = requests + result;

    let rps = requests as f64 / elapsed_time;
    print!("\rRPS: {:.2} RPM: {:.2}", rps, rps * 60.0);
  }

  let end_time = time::precise_time_s();
  let time_taken = end_time - start_time;

  println!("\n------------------");
  println!("Total reqs: {:?}", requests);
  println!("Time taken: {:.2}s", time_taken);
}
