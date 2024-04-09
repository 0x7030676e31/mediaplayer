use crate::model::stream::{DashboardPayload, HintedSender, Payload};
use crate::reader::FileStreamer;
use crate::AppState;

use std::io::Write;

use futures::{future, StreamExt};
use actix_web::{error, web, HttpRequest, HttpResponse, Responder, Scope};
use tokio::time::{sleep, Duration};

#[actix_web::post("/upload/{nonce}/{name}")]
async fn upload_media(state: web::Data<AppState>, payload: web::Payload, path: web::Path<(u64, String)>) -> impl Responder {
  let mut state_ = state.write().await;
  let (nonce, name) = path.into_inner();

  let (id, mut writer) = state_.get_audio_writer(name.clone());

  drop(state_);
  let mut payload = payload;
  while let Some(chunk) = payload.next().await {
    let chunk = chunk.unwrap();
    writer.write_all(&chunk).unwrap();
  }

  log::info!("Created media with id {} (nonce: {})", id, nonce);

  let mut state = state.write().await;
  let length = state.set_audio_length(id);

  let payload = Payload::DownloadMedia(id);
  state.broadcast(payload).await;

  let payload = DashboardPayload::MediaCreated {
    id,
    name: &name,
    length,
  };

  state.broadcast_to_dashboard_with_nonce(payload, nonce).await;
  id.to_string()
}

#[actix_web::get("/{id}")]
async fn download_media(req: HttpRequest, state: web::Data<AppState>, id: web::Path<u16>) -> Result<impl Responder, actix_web::Error> {
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

#[actix_web::post("/{id}/request_download")]
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

#[actix_web::delete("/{id}")]
async fn delete_media(state: web::Data<AppState>, id: web::Path<u16>) -> impl Responder {
  let mut state = state.write().await;
  let id = id.into_inner();
  state.library.retain(|media| media.id != id);
  state.write();

  let payload = Payload::DeleteMedia(id).into_bytes();
  let futs = state.streams.iter().map(|(tx, _)| tx.send_hinted(payload.clone()));

  future::join_all(futs).await;
  let payload = DashboardPayload::MediaDeleted(id);
  state.broadcast_to_dashboard(payload).await;

  log::info!("Deleted media with id {}", id);
  HttpResponse::Ok().finish()
}

#[actix_web::post("/{id}/play")]
async fn play_media(state: web::Data<AppState>, id: web::Path<u16>, clients: web::Json<Vec<u16>>) -> impl Responder {
  let state = state.read().await;
  if !state.library.iter().any(|media| media.id == *id) {
    return HttpResponse::NotFound().finish();
  }

  let id = id.into_inner();
  let payload = Payload::PlayMedia(id).into_bytes();
  
  let futs = state.streams.iter().filter_map(|(tx, client_id)| {
    state.clients.iter()
      .find(|client| client.id == *client_id)
      .filter(|client| clients.contains(&client.id))
      .map(|client| client.playing.is_none().then(|| tx.send_hinted(payload.clone())))
      .flatten()
  });

  future::join_all(futs).await;
  log::info!("Requested play of media {} for {} clients", id, clients.len());

  HttpResponse::Ok().finish()
}

#[actix_web::post("/stop")]
async fn stop_media(state: web::Data<AppState>, clients: web::Json<Vec<u16>>) -> impl Responder {
  let state = state.read().await;
  let payload = Payload::StopMedia.into_bytes();

  let futs = state.streams.iter().filter_map(|(tx, client_id)| {
    clients.contains(client_id).then(|| tx.send_hinted(payload.clone()))
  });

  future::join_all(futs).await;
  log::info!("Requested stop for {} clients", clients.len());

  HttpResponse::Ok().finish()
}

#[actix_web::post("/{id}/playing")]
async fn playing_media(req: HttpRequest, state: web::Data<AppState>, id: web::Path<u16>) -> impl Responder {
  let client_id = match req.headers().get("X-Client-Id") {
    Some(id) => id,
    None => return HttpResponse::BadRequest().finish(),
  };

  let client_id = match client_id.to_str() {
    Ok(id) => id,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  let client_id = match client_id.parse::<u16>() {
    Ok(id) => id,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  let mut state_ = state.write().await;
  let duration = match state_.library.iter().find(|media| media.id == *id) {
    Some(media) => media.length,
    None => return HttpResponse::NotFound().finish(),
  };

  let client = match state_.clients.iter_mut().find(|client| client.id == client_id) {
    Some(client) => client,
    None => return HttpResponse::NotFound().finish(),
  };

  let id = *id;
  let state2 = state.clone();
  let handle = tokio::spawn(async move {
    sleep(Duration::from_millis(duration)).await;
    let mut state = state2.write().await;
    
    let client = match state.clients.iter_mut().find(|client| client.id == client_id) {
      Some(client) => client,
      None => return,
    };

    if let Some(_) = client.playing.take() {
      let payload = DashboardPayload::MediaStopped(client_id);
      state.broadcast_to_dashboard(payload).await;
      
      log::info!("Client {} stopped playing media {}", client_id, id);
    }
  });

  client.playing = Some((id, handle));
  let payload = DashboardPayload::MediaStarted {
    media: id,
    client: client_id,
  };

  state_.broadcast_to_dashboard(payload).await;
  log::info!("Client {} started playing media {}", client_id, id);

  HttpResponse::Ok().finish()
}

#[actix_web::post("/{id}/stopped")]
async fn stopped_media(req: HttpRequest, state: web::Data<AppState>, id: web::Path<u16>) -> impl Responder {
  let client_id = match req.headers().get("X-Client-Id") {
    Some(id) => id,
    None => return HttpResponse::BadRequest().finish(),
  };

  let client_id = match client_id.to_str() {
    Ok(id) => id,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  let client_id = match client_id.parse::<u16>() {
    Ok(id) => id,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  let mut state = state.write().await;
  let client = match state.clients.iter_mut().find(|client| client.id == client_id) {
    Some(client) => client,
    None => return HttpResponse::NotFound().finish(),
  };

  if let Some((_, handle)) = client.playing.take() {
    handle.abort();
    
    let payload = DashboardPayload::MediaStopped(client_id);
    state.broadcast_to_dashboard(payload).await;
    
    log::info!("Client {} stopped playing media {}", client_id, id);
  }
  
  HttpResponse::Ok().finish()
}

pub fn routes() -> Scope {
  Scope::new("/media")
    .service(upload_media)
    .service(download_media)
    .service(request_download)
    .service(delete_media)
    .service(play_media)
    .service(stop_media)
    .service(playing_media)
    .service(stopped_media)
}