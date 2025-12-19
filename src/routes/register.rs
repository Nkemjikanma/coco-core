use crate::types::api::RegisterBody;
use actix_web::{HttpRequest, HttpResponse, web};

// register, renew, transfer, set-records
// #[post("/register")]
pub async fn register(json: web::Json<Vec<RegisterBody>>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
