pub mod config;
pub mod errors;
pub mod types;

use config::Config;
use types::api::RegisterBody;

use actix_web::dev::Server;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use alloy::providers::{Provider, ProviderBuilder};
use serde::Deserialize;

use std::net::TcpListener;

pub struct AppState<P: Provider> {
    pub app_config: config::Config,
    pub provider: P,
}

// #[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

// web::Path
// #[get("/check_names/")]
async fn check_names(req: HttpRequest, names: web::Query<Vec<String>>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

// register, renew, transfer, set-records
// #[post("/register")]
async fn register(json: web::Json<Vec<RegisterBody>>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

// watch
async fn watch(names: web::Query<Vec<String>>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run<P: Provider>(
    listener: TcpListener,
    app_state: AppState<P>,
) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new().service(
            web::scope("/api")
                .route("/", web::get().to(hello))
                .route("/check/{name}", web::get().to(check_names))
                .route("/register", web::post().to(register))
                .route("/watch", web::post().to(watch)),
        )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
