use crate::config::DBConfig;
use crate::routes::{
    check::check_names, expiry::check_expiry, health_check::hello, register::register, watch::watch,
};
use crate::types::api::AppState;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use alloy::providers::Provider;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

pub fn run(listener: TcpListener, app_state: Arc<AppState>) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(app_state);
    let server = HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .route("/", web::get().to(hello))
                    .route("/check", web::get().to(check_names))
                    .route("/expiry", web::get().to(check_expiry))
                    .route("/register", web::post().to(register))
                    .route("/watch", web::post().to(watch)),
            )
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[tracing::instrument(name = "pool", skip_all)]
pub async fn create_pool(config: &DBConfig) -> Result<PgPool, sqlx::Error> {
    tracing::info!("Creating database pool");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(200))
        .idle_timeout(Duration::from_secs(300))
        .connect(&config.connection_string())
        .await?;

    tracing::info!("Database connection pool created");
    Ok(pool)
}
