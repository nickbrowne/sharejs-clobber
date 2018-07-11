extern crate curl;

use self::curl::easy::Easy;
use std::io::Read;

pub struct Doc {
  uid: String,
}

impl Doc {
  pub fn new(uid: String) -> Doc {
    Doc { uid }
  }

  pub fn create(&self) {
    let mut easy = Easy::new();
    let url = format!("http://localhost:9000/doc/{}", self.uid);
    let mut data = "{\"type\":\"text\"}".as_bytes();

    easy.url(&url).expect(&format!("Possible bad url: {:?}", url));
    easy.put(true).unwrap();

    let mut transfer = easy.transfer();
    transfer.read_function(|buffer| {
      Ok(data.read(buffer).unwrap_or(0))
    }).expect("Failed to read buffer");

    transfer.perform().expect("Failed to transfer data");
  }

  pub fn insert(&self, v: i32, text: &str) {
    let mut easy = Easy::new();

    let url = format!("http://localhost:9000/doc/{}?v={}", self.uid, v);
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
}
