use std::sync::Arc;
use std::env;

use model::state::CleanupLoop;
use model::state;

use tokio::sync::RwLock;
use actix_web::web::Data;

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
      // .wrap(actix_web::middleware::Logger::default())
      .app_data(Data::new(state.clone()))
      .service(routes::routes())
  });

  let ip = env::var("IP").unwrap_or(String::from("0.0.0.0"));
  server.bind((ip, INNER_PORT))?.run().await
}
