use alloy::providers::{Provider, ProviderBuilder};
use coco::{AppState, config, errors, run};
use std::net::TcpListener;
use tracing::{error, info};

type AppError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let config = config::Config::load_env()?;

    let listener = get_listener();
    info!("Listening here: {:?}", listener);

    let rpc = &config.eth_rpc;
    let provider = ProviderBuilder::new().connect(rpc).await?;

    let app_state = AppState {
        app_config: config,
        provider,
    };

    run(listener, app_state)
        .map_err(|e| -> AppError { Box::new(e) })?
        .await
        .map_err(Into::into)
}

pub fn get_listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").expect("Failed to bind IP and Port in main")
}
