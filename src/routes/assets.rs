use crate::reader::FileStreamer;
use crate::state::path;

use std::fs;

use actix_web::{Responder, HttpResponse, web};

#[actix_web::get("/assets/{name}")]
pub async fn asset(name: web::Path<String>) -> impl Responder {
  let name = name.into_inner();
  let path = format!("{}/assets/{}", path(), name);
  match fs::File::open(&path) {
    Ok(file) => {
      let streamer = FileStreamer(file);
      log::debug!("Streaming asset {}", name);

      HttpResponse::Ok().streaming(streamer)
    },
    Err(_) => return HttpResponse::NotFound().finish(),
  }
}
