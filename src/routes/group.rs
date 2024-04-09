use crate::model::stream::DashboardPayload;
use crate::AppState;

use actix_web::{web, HttpResponse, Scope};

#[actix_web::post("")]
async fn create_group(state: web::Data<AppState>) -> HttpResponse {
  let mut state = state.write().await;
  let id = state.new_group();

  let payload = DashboardPayload::GroupCreated(id);
  state.broadcast_to_dashboard(payload).await;
  
  HttpResponse::Ok().finish()
}

pub fn routes() -> Scope {
  web::scope("/group")
    .service(create_group)
}