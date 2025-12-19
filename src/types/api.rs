use crate::{
    config,
    types::{alloy_providers::AppProvider, ens::EnsContractAddresses},
};
use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
// --------------- App State --------------
pub struct AppState {
    pub app_config: config::Config,
    pub provider: AppProvider,
    pub connection: PgPool,
    pub ens_contract_addresses: EnsContractAddresses,
}

// ---- Api error -----
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}
// ----- Check -------
#[derive(Deserialize)]
pub struct CheckQuery {
    pub names: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceResponse {
    pub base: U256,
    pub premium: U256,
}
#[derive(Debug, Serialize)]
pub struct CheckNameResponse {
    pub name: String,
    pub available: bool,
    pub price: Option<PriceResponse>,
    pub owner: Option<Address>,
    pub expires: Option<U256>,
}

// -------------- Expirty ----------------
#[derive(Debug, Serialize)]
pub struct CheckExpiryResponse {
    pub name: String,
    pub available: bool,
    pub expiry_date: Option<U256>, // date like
}
// -------------- Register -----------------
#[derive(Deserialize)]
pub struct RegisterBody {
    pub name: String,
    pub duration: u8,
}

// -------------- Watch -----------------
#[derive(Deserialize)]
pub struct WatchBody {
    pub name: String,
    pub user_id: String,
    pub channel_id: String,
    pub thread_id: String,
}
