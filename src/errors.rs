use actix_web::http::StatusCode;
use alloy::providers::MulticallError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::api::ApiError;

#[derive(Debug, Error, Clone)]
pub enum ConfigError {
    #[error("Missing environment variables: {0}")]
    MissingEnv(String),
}

#[derive(Debug, Error)]
pub enum CocoError {
    #[error("Invalid query input")]
    InvalidQueryInput,

    #[error("Couldn't normalise the name: {0}")]
    InvalidName(String),

    #[error("Invalid Ethereum address")]
    InvalidAddress,

    #[error("Something went wrong during the ENS multicall process")]
    Ens(#[source] MulticallError),
}

impl From<MulticallError> for CocoError {
    fn from(e: MulticallError) -> Self {
        CocoError::Ens(e)
    }
}

impl CocoError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            CocoError::InvalidQueryInput => StatusCode::BAD_REQUEST,
            CocoError::InvalidName(_) => StatusCode::BAD_REQUEST,
            CocoError::InvalidAddress => StatusCode::BAD_REQUEST,
            CocoError::Ens(_) => StatusCode::BAD_GATEWAY, // RPC and chain errors
        }
    }

    pub fn to_api_error(&self) -> ApiError {
        match self {
            CocoError::InvalidQueryInput => Api {
                code: "invalid_query_input",
                message: "Input is malformed".to_string(),
            }
            CocoError::InvalidName(msg) => ApiError {
                code: "invalid_name",
                message: msg.clone(),
            },
            CocoError::InvalidAddress => ApiError {
                code: "invalid_address",
                message: "Wallet address provided is not valid".to_string(),
            },

            CocoError::Ens(_) => ApiError {
                code: "ens_eeor",
                message: "ENS lookup failed".to_string(),
            },
        }
    }
}
