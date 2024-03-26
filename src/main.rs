use std::sync::Arc;
use std::collections::HashMap;
use std::fs::File;
use std::{env, io};

use model::state::{CleanupLoop, path};
use model::state;
use reader::FileStreamer;

use tokio::sync::RwLock;
use actix_web::{web, HttpResponse, error, Responder};
use rustls::{ServerConfig, Certificate, PrivateKey};
use rustls::server::{ResolvesServerCert, ClientHello};
use rustls::sign::{CertifiedKey, any_supported_type};

mod model;
mod routes;
mod reader;

const INNER_PORT: u16 = 7777;
pub type AppState = Arc<RwLock<state::State>>;

struct MultiDomainResolver (HashMap<String, Arc<CertifiedKey>>);

impl ResolvesServerCert for MultiDomainResolver {
  fn resolve(&self, dns_name: ClientHello) -> Option<Arc<CertifiedKey>> {
    dns_name.server_name().and_then(|name| {
      let name = if let Some(name) = name.strip_prefix("www.") { name } else { name };
      self.0.get(name).cloned()
    })
  }
}

const DOMAIN: &str = "entitia";
impl MultiDomainResolver {
  pub fn add_cert(&mut self, path: &String, top_domain: &str) -> io::Result<()> {
    let cert = format!("{}{}/certificate.pem", path, top_domain);
    let key = format!("{}{}/private.pem", path, top_domain);

    let cert_file = File::open(cert)?;
    let mut cert_reader = io::BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut cert_reader)
      .map(|cert|cert.map(|cert| Certificate(cert.iter().map(|byte| byte.to_owned()).collect())))
      .collect::<Result<Vec<_>, _>>()?;

    let key_file = File::open(key)?;
    let mut key_reader = io::BufReader::new(key_file);
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader).collect::<Result<Vec<_>, _>>()?;

    let key = match keys.len() {
      1 => PrivateKey(keys.remove(0).secret_pkcs8_der().to_vec()),
      0 => return Err(io::Error::new(io::ErrorKind::InvalidInput, "No keys found")),
      _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Multiple keys found")),
    };

    let key = match any_supported_type(&key) {
      Ok(key) => key,
      Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid key type")),
    };

    let cert = CertifiedKey::new(certs, key);
    self.0.insert(format!("{}.{}", DOMAIN, top_domain), Arc::new(cert));

    Ok(())
  }
}

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
  let is_production = env::var("PRODUCTION").map_or(false, |prod| prod == "true");

  if !is_production {
    return server
      .bind((ip, INNER_PORT))?
      .run()
      .await
  }

  let mut resolver = MultiDomainResolver(HashMap::new());
  let path = if is_production { "/root/".into() } else { env::var("FS").unwrap_or("/root/".into()) };

  resolver.add_cert(&path, "com")?;
  let cfg = ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth()
    .with_cert_resolver(Arc::new(resolver));

  server
    .bind_rustls_021(format!("{}:{}", ip, INNER_PORT), cfg)?
    .run()
    .await
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
