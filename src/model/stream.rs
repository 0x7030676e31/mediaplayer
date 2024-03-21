use crate::model::state::{Media, Client};

use std::collections::HashSet;
use std::task::{Context, Poll};

use actix_web_lab::sse::{Data, Event};
use pin_project_lite::pin_project;
use futures::ready;
use futures::stream::Stream;
use actix_web::{Responder, HttpResponse, web};
use actix_web::body::{BoxBody, MessageBody, BodySize};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc;
use tokio_stream::wrappers;
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum Payload {
  Ready,
  DownloadMedia(u16),
  PlayMedia(u16),
  StopMedia,
  SelfDestruct,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum DashboardPayload<'a> {
  Ready {
    library: &'a [Media],
    clients: &'a [Client],
  },
  ClientCreated(&'a Client),
  ClientConnected(u16),
  ClientDisconnected(&'a HashSet<u16>),
  MediaDownloaded {
    media: u16,
    client: u16,
  },
}

impl Payload {
  pub fn into_bytes(self) -> web::Bytes {
    web::Bytes::from(serde_json::to_string(&self).unwrap())
  }
}

impl<'a> DashboardPayload<'a> {
  pub fn into_event(self) -> Event {
    Event::Data(Data::new_json(self).unwrap())
  }
}

pin_project! {
  pub struct InfallibleStream<S> {
    #[pin]
    stream: S,
  }
}

impl<S> InfallibleStream<S> {
  pub fn new(stream: S) -> Self {
    Self { stream }
  }
}

impl<S: Stream> Stream for InfallibleStream<S> {
  type Item = Result<S::Item, std::convert::Infallible>;

  fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    Poll::Ready(ready!(self.project().stream.poll_next(cx)).map(Ok))
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.stream.size_hint()
  }
}

impl Responder for InfallibleStream<wrappers::ReceiverStream<web::Bytes>> {
  type Body = BoxBody;

  fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
    HttpResponse::Ok()
      .content_type("text/plain")
      .body(self)
  }
}

impl MessageBody for InfallibleStream<wrappers::ReceiverStream<web::Bytes>> {
  type Error =  Box<dyn std::error::Error>;

  fn size(&self) -> BodySize {
    BodySize::Stream
  }

  fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Result<web::Bytes, Self::Error>>> {
    let this = self.project();

    if let Poll::Ready(item) = this.stream.poll_next(cx) {
      return match item {
        Some(item) => Poll::Ready(Some(Ok(item))),
        None => Poll::Ready(None),
      }
    }

    Poll::Pending
  }
}

pub trait HintedSender<T> {
  async fn send_hinted(&self, value: T) -> Result<(), SendError<T>>;
}

impl HintedSender<web::Bytes> for mpsc::Sender<web::Bytes> {
  async fn send_hinted(&self, value: web::Bytes) -> Result<(), SendError<web::Bytes>> {
    let mut combined_value = value.to_vec();
    combined_value.extend_from_slice(b"\x00");
    self.send(web::Bytes::from(combined_value)).await
  }
}
