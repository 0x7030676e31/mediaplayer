use crate::model::stream::DashboardPayload;
use crate::{model::stream::Payload, state::ClientInfo};
use crate::AppState;

use actix_web::{error, web, HttpRequest, HttpResponse, Responder, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
struct RenameInfo {
  nonce: u64,
  alias: String,
}

#[actix_web::post("/{id}/rename")]
pub async fn rename(state: web::Data<AppState>, new_name: web::Json<RenameInfo>, id: web::Path<u16>) -> Result<impl Responder, actix_web::Error> {
  let mut state = state.write().await;
  let id = id.into_inner();
  let RenameInfo { nonce, alias } = new_name.into_inner();

  let target = match state.clients.iter_mut().find(|c| c.id == id) {
    Some(target) => target,
    None => return Err(error::ErrorNotFound("Client not found")),
  };
  
  target.alias = if alias.is_empty() { None } else { Some(alias) };
  let payload = DashboardPayload::ClientRenamed(id, target.alias.clone());
  state.broadcast_to_dashboard_with_nonce(payload, nonce).await;

  state.write();
  Ok(HttpResponse::Ok().finish())
}

#[actix_web::post("")]
async fn client(req: HttpRequest, state: web::Data<AppState>, body: web::Json<ClientInfo>) -> Result<impl Responder, actix_web::Error> {
  let connection_info = req.connection_info();
  let ip = match connection_info.peer_addr() {
    Some(ip) => ip,
    None => return Err(error::ErrorInternalServerError("Failed to get client IP")),
  };
  
  let mut state = state.write().await;
  let id = state.new_client(body.0, ip.to_owned()).await;
  
  log::info!("Created a new client from {} with id {}", ip, id);
  Ok(id.to_string())
}

#[actix_web::delete("/{id}")]
async fn delete_client(state: web::Data<AppState>, id: web::Path<u16>) -> impl Responder {
  let mut state = state.write().await;
  let id = id.into_inner();

  if state.clients.iter().find(|c| c.id == id).is_none() {
    return HttpResponse::NotFound().finish();
  }

  state.to_delete.insert(id);
  state.write();

  log::info!("Client {} scheduled for deletion", id);

  let payload = Payload::SelfDestruct;
  state.broadcast_to(id, payload).await;

  HttpResponse::Ok().finish()
}

#[actix_web::post("/{id}/shutdown")]
async fn shutdown_client(state: web::Data<AppState>, id: web::Path<u16>) -> impl Responder {
  let state = state.read().await;
  let id = id.into_inner();

  if state.clients.iter().find(|c| c.id == id).is_none() {
    return HttpResponse::NotFound().finish();
  }

  let payload = Payload::Shutdown;
  state.broadcast_to(id, payload).await;
  log::info!("Client {} has been instructed to shutdown", id);

  HttpResponse::Ok().finish()
}

#[actix_web::post("/seppuku")]
async fn seppuku(req: HttpRequest, state: web::Data<AppState>) -> Result<impl Responder, actix_web::Error> {
  let client_id = match req.headers().get("X-Client-Id") {
    Some(id) => id,
    None => return Err(error::ErrorBadRequest("Missing X-Client-Id header")),
  };

  let client_id = match client_id.to_str() {
    Ok(id) => id,
    Err(_) => return Err(error::ErrorBadRequest("Invalid X-Client-Id header")),
  };

  let client_id = match client_id.parse::<u16>() {
    Ok(id) => id,
    Err(_) => return Err(error::ErrorBadRequest("Invalid X-Client-Id header")),
  };

  let mut state = state.write().await;
  state.to_delete.remove(&client_id);
  state.clients.retain(|c| c.id != client_id);
  state.library.iter_mut().for_each(|media| {
    media.downloaded.retain(|c| *c != client_id);
  });
  
  state.write();

  log::info!("Client {} has committed seppuku", client_id);

  let payload = DashboardPayload::ClientDeleted(client_id);
  state.broadcast_to_dashboard(payload).await;

  Ok(client_id.to_string())
}

pub fn routes() -> Scope {
  Scope::new("/client")
    .service(rename)
    .service(client)
    .service(delete_client)
    .service(shutdown_client)
    .service(seppuku)
}