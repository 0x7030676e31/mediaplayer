use std::io::Read;
use std::task::{Context, Poll};
use std::fs::File;
use std::pin::Pin;

use actix_web::web;
use futures::stream::Stream;

pub struct FileStreamer(pub File);

impl Stream for FileStreamer {
  type Item = Result<web::Bytes, std::io::Error>;

  fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let mut buffer = [0; 1024];
    match self.0.read(&mut buffer) {
      Ok(0) => Poll::Ready(None),
      Ok(n) => Poll::Ready(Some(Ok(web::Bytes::copy_from_slice(&buffer[..n])))),
      Err(e) => Poll::Ready(Some(Err(e))),
    }
  }
}
