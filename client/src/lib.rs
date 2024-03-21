use std::sync::Arc;

use stream::create_stream;
use state::Data;

use tokio::time::{sleep, Duration};
use futures_util::StreamExt;
use tokio::sync::RwLock;

mod state;
mod stream;

pub const ADDR: &str = if cfg!(debug_assertions) {
  "http://localhost:7777"
} else {
  "http://70.34.254.149:7777"
};

#[no_mangle]
#[tokio::main]
pub async extern "C" fn load() {
  let data = Data::init().await;
  let client_id = data.id;

  let _data = Arc::new(RwLock::new(data));

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
        stream::Payload::Ready => println!("Ready"),
        stream::Payload::Ping => println!("Ping"),
        stream::Payload::DownloadMedia(_id) => todo!(),
        stream::Payload::PlayMedia(_id) => todo!(),
        stream::Payload::StopMedia => todo!(),
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
