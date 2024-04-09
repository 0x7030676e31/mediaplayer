use actix_web::Scope;

mod stream;
mod client;
mod media;
mod assets;
mod group;

pub fn routes() -> Scope {
  Scope::new("/api")
    .service(media::routes())
    .service(group::routes())
    .service(client::routes())
    .service(stream::stream)
    .service(stream::dashboard_stream)
    .service(assets::asset)
}
