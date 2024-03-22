use crate::state::Data;
use crate::ADDR;

use std::io::Write;
use std::sync::Arc;
use std::fs::File;

use futures_util::StreamExt;
use tokio::sync::RwLock;
use reqwest::Client;

async fn download(id: u16, client_id: u16, state: Arc<RwLock<Data>>) -> Result<(), ()> {
  let addr = format!("{}/api/media/{}", ADDR, id);
  let client = Client::new();

  let req = client.get(&addr)
    .header("X-Client-Id", client_id)
    .send()
    .await;

  let res = req.map_err(|e| eprintln!("Req error: {}", e))?;

  let mut stream = res.bytes_stream();
  let mut writer = File::create(id.to_string()).unwrap();

  while let Some(chunk) = stream.next().await {
    let chunk = chunk.map_err(|e| eprintln!("Chunk error: {}", e))?;
    writer.write_all(&chunk).unwrap();
  }

  let mut state = state.write().await;
  state.library.insert(id);

  println!("Downloaded media: {}", id);
  Ok(())
}

pub trait DataApi {
  fn download(&self, id: u16, client_id: u16);
}


impl DataApi for Arc<RwLock<Data>> {
  fn download(&self, id: u16, client_id: u16) {
    let state = self.clone();
    tokio::spawn(download(id, client_id, state));
  }
}
