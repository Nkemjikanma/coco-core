use crate::types::api::WatchBody;
use actix_web::{HttpRequest, HttpResponse, web};

// watch
pub async fn watch(names: web::Json<WatchBody>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
