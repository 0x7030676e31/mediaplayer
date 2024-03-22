use crate::model::stream::{DashboardPayload, HintedSender, Payload};
use crate::reader::FileStreamer;
use crate::AppState;

use std::io::Write;
use std::ops::Not;

use futures::{future, StreamExt};
use actix_web::{error, web, HttpRequest, Responder, HttpResponse};


#[actix_web::post("/media/{name}")]
async fn upload_media(state: web::Data<AppState>, payload: web::Payload, name: web::Path<String>) -> impl Responder {
  let mut state_ = state.write().await;
  let (id, mut writer) = state_.get_audio_writer(name.into_inner());

  drop(state_);
  let mut payload = payload;
  while let Some(chunk) = payload.next().await {
    let chunk = chunk.unwrap();
    writer.write_all(&chunk).unwrap();
  }

  log::info!("Created media with id {}", id);

  let state = state.read().await;
  let payload = Payload::DownloadMedia(id);
  state.broadcast(payload).await;

  id.to_string()
}

#[actix_web::get("/media/{id}")]
async fn get_media(req: HttpRequest, state: web::Data<AppState>, id: web::Path<u16>) -> Result<impl Responder, actix_web::Error> {
  let mut state = state.write().await;

  if !state.library.iter().any(|media| media.id == *id) {
    return Err(error::ErrorNotFound("Media not found"));
  }

  let reader = match state.get_audio_reader(*id) {
    Some(reader) => reader,
    None => return Err(error::ErrorInternalServerError("Failed to open media")),
  };

  log::debug!("Streaming media with id {}", id);
  let streamer = FileStreamer(reader);
  let client_id = match req.headers().get("X-Client-Id") {
    Some(id) => id,
    None => return Ok(HttpResponse::Ok().streaming(streamer)),
  };

  let client_id = match client_id.to_str() {
    Ok(id) => id,
    Err(_) => return Err(error::ErrorBadRequest("Invalid client id")),
  };

  let client_id = match client_id.parse::<u16>() {
    Ok(id) => id,
    Err(_) => return Err(error::ErrorBadRequest("Invalid client id")),
  };

  log::debug!("Client {} is downloading media {}", client_id, id);
  let media = state.library.iter_mut().find(|media| media.id == *id).unwrap();
  if media.downloaded.insert(client_id) {
    let payload = DashboardPayload::MediaDownloaded {
      media: *id,
      client: client_id,
    };

    state.broadcast_to_dashboard(payload).await;
  }

  Ok(HttpResponse::Ok().streaming(streamer))
}

#[actix_web::post("/media/request_download/{id}")]
async fn request_download(state: web::Data<AppState>, id: web::Path<u16>, clients: web::Json<Vec<u16>>) -> impl Responder {
  let state = state.read().await;
  let media = match state.library.iter().find(|media| media.id == *id) {
    Some(media) => media,
    None => return HttpResponse::NotFound().finish(),
  };
  
  
  let id = id.into_inner();
  let payload = Payload::DownloadMedia(id).into_bytes();
  
  let futs = state.streams.iter().filter_map(|(tx, client_id)| {
    if clients.contains(client_id) && !media.downloaded.contains(client_id) {
      Some(tx.send_hinted(payload.clone()))
    } else {
      None
    }
  });

  future::join_all(futs).await;
  log::info!("Requested download of media {} for {} clients", id, clients.len());

  HttpResponse::Ok().finish()
}
