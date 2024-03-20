use crate::AppState;

use actix_web::{error, web, HttpRequest, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct Client {
  hostname: String,
}

#[actix_web::post("/client")]
pub async fn client(req: HttpRequest, state: web::Data<AppState>, body: web::Json<Client>) -> Result<impl Responder, actix_web::Error> {
  let connection_info = req.connection_info();
  let ip = match connection_info.peer_addr() {
    Some(ip) => ip,
    None => return Err(error::ErrorInternalServerError("Failed to get client IP")),
  };
  
  let mut state = state.write().await;
  let id = state.new_client(body.0.hostname, ip.to_owned());
  
  log::info!("Created a new client from {} with id {}", ip, id);
  Ok(id.to_string())
}
