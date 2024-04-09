use crate::model::stream::{DashboardPayload, InfallibleStream, Payload, HintedSender};
use crate::model::state::Activity;
use crate::AppState;

use actix_web::{error, web, HttpRequest, Responder};
use tokio_stream::wrappers::ReceiverStream;
use actix_web_lab::sse::Sse;
use tokio::sync::mpsc;

#[actix_web::get("/stream")]
pub async fn stream(req: HttpRequest, state: web::Data<AppState>) -> Result<impl Responder, actix_web::Error> {
  let id = match req.headers().get("X-Client-Id") {
    Some(id) => id,
    None => return Err(error::ErrorBadRequest("Missing X-Client-Id header")),
  };

  let id = match id.to_str() {
    Ok(id) => id,
    Err(_) => return Err(error::ErrorBadRequest("Invalid X-Client-Id header")),
  };

  let id = match id.parse::<u16>() {
    Ok(id) => id,
    Err(_) => return Err(error::ErrorBadRequest("Invalid X-Client-Id header")),
  };
  
  let (tx, rx) = mpsc::channel(32);
  let rx = ReceiverStream::new(rx);

  let mut state = state.write().await;
  
  let client = state.clients.iter_mut().find(|client| client.id == id).unwrap();
  client.activity = Activity::Online;
  
  state.streams.push((tx.clone(), id));
  let set_to_deletion = state.to_delete.iter().any(|to_delete| *to_delete == id);
  let payload = Payload::Ready(set_to_deletion).into_bytes();
  
  tokio::spawn(async move {
    if let Err(err) = tx.send_hinted(payload).await {
      log::error!("Failed to send ready payload: {}", err);
    }
  });

  state.broadcast_to_dashboard(DashboardPayload::ClientConnected(id)).await;

  log::info!("Client {} connected", id);
  Ok(InfallibleStream::new(rx))
}

#[actix_web::get("/dashboard/stream")]
pub async fn dashboard_stream(state: web::Data<AppState>) -> impl Responder {
  let (tx, rx) = mpsc::channel(32);

  tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

  let mut state = state.write().await;
  let ack = state.next_ack();
  state.dashboard_streams.push(tx.clone());

  let playing = state.clients.iter().filter_map(|client| match client.playing {
    Some(_) => Some(client.id),
    None => None,
  }).collect();
  let payload = DashboardPayload::Ready {
    library: &state.library,
    clients: &state.clients,
    groups: &state.groups,
    playing,
  };

  let payload = payload.into_event(ack, None);
  tokio::spawn(async move {
    if let Err(err) = tx.send(payload).await {
      log::error!("Failed to send ready payload: {}", err);
    }
  });

  log::info!("Dashboard connected");
  Sse::from_infallible_receiver(rx)
}
