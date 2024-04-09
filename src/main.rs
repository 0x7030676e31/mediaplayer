use std::sync::Arc;
use std::fs::File;
use std::{env, io};

use model::state::{CleanupLoop, path};
use model::state;
use reader::FileStreamer;

use tokio::sync::RwLock;
use actix_web::{web, HttpResponse, error, Responder};

mod model;
mod routes;
mod reader;

const INNER_PORT: u16 = 7777;
pub type AppState = Arc<RwLock<state::State>>;


#[tokio::main]
async fn main() -> io::Result<()> {
  dotenv::dotenv().ok();

  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "info");
  }

  pretty_env_logger::init();
  log::info!("Starting server on port {}", INNER_PORT);

  let state = state::State::new();
  let state = Arc::new(RwLock::new(state));
  state.start_cleanup_loop();

  let server = actix_web::HttpServer::new(move || {
    actix_web::App::new()
      .wrap(actix_cors::Cors::permissive())
      .app_data(web::Data::new(state.clone()))
      .service(routes::routes())
      .service(script)
      .service(index)
      .service(r#static)
      .default_service(web::get().to(fallback))
  });

  let ip = env::var("IP").unwrap_or(String::from("0.0.0.0"));
  server.bind((ip, INNER_PORT))?.run().await
}

#[actix_web::get("/script")]
async fn script() -> impl Responder {
  let file = File::open(format!("{}/assets/script.ps1", path())).unwrap();
  HttpResponse::Ok().content_type("text/plain").streaming(FileStreamer(file))
}

#[actix_web::get("/")]
async fn index() -> impl Responder {
  let file = File::open(format!("{}/static/index.html", path())).unwrap();
  HttpResponse::Ok().content_type("text/html").streaming(FileStreamer(file))
}

async fn fallback() -> impl Responder {
  let file = File::open(format!("{}/static/index.html", path())).unwrap();
  HttpResponse::Ok().content_type("text/html").streaming(FileStreamer(file))
}

#[actix_web::get("/assets/{asset}")]
async fn r#static(asset: web::Path<String>) -> Result<impl Responder, actix_web::Error> {
  let asset = asset.into_inner();
  let path = format!("{}/static/{}", path(), asset);
  let file = match File::open(path) {
    Ok(file) => file,
    Err(_) => return Err(error::ErrorNotFound("File not found")),
  };

  let content_type = match asset.split('.').last() {
    Some("css") => "text/css",
    Some("js") => "text/javascript",
    Some("png") => "image/png",
    Some("ico") => "image/x-icon",
    _ => "text/plain",
  };

  Ok(HttpResponse::Ok().content_type(content_type).streaming(FileStreamer(file)))
}
