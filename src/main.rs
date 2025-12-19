use alloy::providers::{Provider, ProviderBuilder};
use coco::{
    config,
    startup::{create_pool, run},
    types::{api::AppState, ens::EnsContractAddresses},
};
use std::net::TcpListener;
use std::sync::Arc;
use tracing::info;

type AppError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    // config setup
    let config = config::Config::load_env()?;

    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;

    info!("Listening here: {:?}", listener);

    // Alloy
    let rpc = &config.eth_rpc;
    let provider = ProviderBuilder::new().connect(rpc).await?;
    // FillProvider<JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>, RootProvider>

    // SQLx - Postgres Pool
    let connection = create_pool(&config.database).await?;

    // contract addresses
    let contract_addresses = EnsContractAddresses::mainnet();

    let app_state = Arc::new(AppState {
        app_config: config,
        provider,
        connection,
        ens_contract_addresses: contract_addresses,
    });

    run(listener, app_state)
        .map_err(|e| -> AppError { Box::new(e) })?
        .await
        .map_err(Into::into)
}

pub fn get_listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").expect("Failed to bind IP and Port in main")
}
// curl "http://127.0.0.1:8000/api/check?names=alice&names=bob&names=vitalik"
