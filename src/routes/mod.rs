use actix_web::Scope;

mod stream;
mod client;
mod media;
mod assets;

pub fn routes() -> Scope {
  Scope::new("/api")
    .service(stream::stream)
    .service(stream::dashboard_stream)
    .service(client::client)
    .service(media::upload_media)
    .service(media::get_media)
    .service(assets::asset)
}
