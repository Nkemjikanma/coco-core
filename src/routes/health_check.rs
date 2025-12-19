use actix_web::{HttpRequest, HttpResponse, web};

// #[get("/")]
pub async fn hello() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}
