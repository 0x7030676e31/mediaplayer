use crate::ADDR;

use std::collections::VecDeque;
use std::task::{Context, Poll};
use std::pin::Pin;

use bytes::{Bytes, BytesMut};
use futures_util::{StreamExt, Stream};
use serde::de::Error;
use serde::Deserialize;
use reqwest::{Client, Result};
use whoami::fallible;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum Payload {
  Ready,
  Ping,
  DownloadMedia(u16),
  PlayMedia(u16),
  StopMedia,
  SelfDestruct,
}

pub struct PayloadStream<R: StreamExt<Item = reqwest::Result<Bytes>> + Unpin> {
  reader: R,
  buffer: BytesMut,
  queue: VecDeque<serde_json::Result<Payload>>,
}

impl<R: StreamExt<Item = reqwest::Result<Bytes>> + Unpin> PayloadStream<R> {
  fn new(reader: R) -> Self {
    PayloadStream {
      reader,
      buffer: BytesMut::new(),
      queue: VecDeque::new(),
    }
  }
}

impl<R: StreamExt<Item = reqwest::Result<Bytes>> + Unpin> Stream for PayloadStream<R> {
  type Item = serde_json::Result<Payload>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
    let this = self.get_mut();
    if let Some(payload) = this.queue.pop_front() {
      return Poll::Ready(Some(payload));
    }

    let reader = Pin::new(&mut this.reader);
    match reader.poll_next(cx) {
      Poll::Ready(Some(Ok(chunk))) => {
        let chunks = chunk.split_inclusive(|b| *b == b'\x00');
        for chunk in chunks.into_iter() {
          if chunk.last() != Some(&b'\x00') {
            this.buffer.extend_from_slice(&chunk);
            continue;
          }

          this.buffer.extend_from_slice(&chunk[..chunk.len() - 1]);
          let payload = serde_json::from_slice::<Payload>(&this.buffer);
          this.buffer.clear();

          this.queue.push_back(payload);
        }

        if let Some(payload) = this.queue.pop_front() {
          return Poll::Ready(Some(payload));
        }

        Poll::Pending
      },
      Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(serde_json::Error::custom(e.to_string())))),
      Poll::Ready(None) => Poll::Ready(None),
      Poll::Pending => Poll::Pending,
    }
  }
}

pub async fn create_stream(client_id: u16) -> Result<PayloadStream<impl Stream<Item = reqwest::Result<Bytes>>>> {
  let response = Client::new()
    .get(format!("{}/api/stream", ADDR))
    .header("X-Client-Id", client_id)
    .send()
    .await?;

  Ok(PayloadStream::new(response.bytes_stream()))
}

pub async fn create_client() -> Result<u16> {
  let response = Client::new()
  .post(format!("{}/api/client", ADDR))
  .header("Content-Type", "application/json")
  .body(format!("{{\"hostname\": \"{}\"}}", fallible::hostname().unwrap()))
  .send()
  .await?;

  let id = response.text().await?;
  Ok(id.parse::<u16>().unwrap())
}
