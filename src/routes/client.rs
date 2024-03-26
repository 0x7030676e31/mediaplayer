use crate::model::stream::DashboardPayload;
use crate::{model::stream::Payload, state::ClientInfo};
use crate::AppState;

use actix_web::{error, web, HttpRequest, HttpResponse, Responder};

#[actix_web::post("/client")]
pub async fn client(req: HttpRequest, state: web::Data<AppState>, body: web::Json<ClientInfo>) -> Result<impl Responder, actix_web::Error> {
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

#[actix_web::delete("/client/{id}")]
pub async fn delete_client(state: web::Data<AppState>, id: web::Path<u16>) -> impl Responder {
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

#[actix_web::post("/client/seppuku")]
pub async fn seppuku(req: HttpRequest, state: web::Data<AppState>) -> Result<impl Responder, actix_web::Error> {
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
