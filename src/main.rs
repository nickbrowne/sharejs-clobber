extern crate curl;
extern crate rand;
extern crate termion;
extern crate time;

use curl::easy::Easy;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::io::{Write, stdout};
use std::sync::mpsc::{channel};
use std::thread;
use termion::raw::IntoRawMode;

const CONCURRENCY: usize = 2;

fn create_document(easy: &mut Easy, uid: &String) {
  use std::io::Read;

  let url = format!("http://localhost:9000/doc/{}", uid);
  let mut data = "{\"type\":\"text\"}".as_bytes();

  easy.url(&url).expect(&format!("Possible bad url: {:?}", url));
  easy.put(true).unwrap();

  let mut transfer = easy.transfer();
  transfer.read_function(|buffer| {
    Ok(data.read(buffer).unwrap_or(0))
  }).expect("Failed to read buffer");

  transfer.perform().expect("Failed to transfer data");
}

fn insert(easy: &mut Easy, uid: &String, v: i32, text: &str) {
  use std::io::Read;

  let url = format!("http://localhost:9000/doc/{}?v={}", uid, v);
  let data = format!("[{{\"i\":{:?},\"p\":0}}]", text);

  let mut data = data.as_bytes();

  easy.url(&url).expect(&format!("Possible bad url: {:?}", url));
  easy.post(true).unwrap();
  easy.post_field_size(data.len() as u64).unwrap();

  let mut transfer = easy.transfer();
  transfer.read_function(|buffer| {
    Ok(data.read(buffer).unwrap_or(0))
  }).expect("Failed to read buffer");

  transfer.perform().expect("Failed to transfer data");
}

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
      let mut easy = Easy::new();

      create_document(&mut easy, &uid);
      easy.reset();

      for v in 0..1000 {
        insert(&mut easy, &uid, v, "hello from rust\n");
        easy.reset();

        thread_tx.send(1).expect("Receiver died");
      }
    });
  }

  // because we cloned it? otherwise the iterator waits indefinitely
  // for the channel to be closed
  drop(tx);

  let start_time = time::precise_time_s();

  println!("Clobbering sharejs!");
  println!("------------------");

  let mut stdout = stdout().into_raw_mode().unwrap();

  let mut rx_iter = rx.iter();

  while let Some(result) = rx_iter.next() {
    let curr_time = time::precise_time_s();
    let elapsed_time = curr_time - start_time;

    requests = requests + result;

    let rps = requests as f64 / elapsed_time;
    write!(stdout, "\rRPS: {:.2} RPM: {:.2}", rps, rps * 60.0).unwrap();
  }

  // get us out of raw mode
  drop(stdout);

  let end_time = time::precise_time_s();
  let time_taken = end_time - start_time;

  println!("\n------------------");
  println!("Total reqs: {:?}", requests);
  println!("Time taken: {:?}s", time_taken);
}
