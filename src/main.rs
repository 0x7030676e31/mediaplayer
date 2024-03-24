use std::sync::Arc;
use std::fs::File;
use std::env;

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
async fn main() -> std::io::Result<()> {
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
      .service(index)
      .service(r#static)
  });

  let ip = env::var("IP").unwrap_or(String::from("0.0.0.0"));
  server.bind((ip, INNER_PORT))?.run().await
}

#[actix_web::get("/")]
async fn index() -> impl Responder {
  let file = File::open(format!("{}/static/index.html", path())).unwrap();
  HttpResponse::Ok().streaming(FileStreamer(file))
}

#[actix_web::get("/assets/{asset}")]
async fn r#static(asset: web::Path<String>) -> Result<impl Responder, actix_web::Error> {
  let path = format!("{}/static/{}", path(), asset.into_inner());
  let file = match File::open(path) {
    Ok(file) => file,
    Err(_) => return Err(error::ErrorNotFound("File not found")),
  };

  Ok(HttpResponse::Ok().streaming(FileStreamer(file)))
}
