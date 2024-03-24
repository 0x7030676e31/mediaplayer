use std::sync::Arc;

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

// pub fn reset() {
//   let path = state::path();
//   std::fs::remove_dir_all(path).unwrap();
//   std::process::exit(0);
// }

#[no_mangle]
#[tokio::main]
#[allow(unreachable_code, unused_variables)]
pub async extern "C" fn load() {
  // reset();

  let data = Data::init().await;
  println!("Initiated data with id: {}", data.id);
  println!("Library: {:?}", data.library);

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
          eprintln!("Error: {}", e);
          continue;
        },
      };

      match payload {
        stream::Payload::Ready => println!("Client ready"),
        stream::Payload::Ping => {},
        stream::Payload::DownloadMedia(id) => data.download(id, client_id),
        stream::Payload::PlayMedia(id) => data.play(id),
        stream::Payload::StopMedia => data.stop(),
        stream::Payload::SelfDestruct => todo!(),
      }
    }

    sleep(Duration::from_secs(5)).await;
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_init() {
    let id = stream::create_client().await.unwrap();
    println!("Client id: {}", id);

    let mut stream = create_stream(id).await.unwrap();
    let payload = stream.next().await;

    println!("Payload: {:?}", payload);
    sleep(Duration::from_secs(20)).await;

    println!("Done");
  }
}
