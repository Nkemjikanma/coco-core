//! check.rs
use crate::services::ens::check_name_availability;
use crate::types::api::{AppState, CheckNameResponse, CheckQuery};
use actix_web::{HttpRequest, HttpResponse, http::StatusCode, web};
use alloy::primitives::{Address, U256};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ---- Response to match bot -----
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NameCheckData {
    pub values: Vec<NameCheckResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NameCheckResponse {
    pub name: String,
    pub is_available: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,

    // Use an ISO string for DateLike (easy for Node to parse)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub registeration_price: Option<Cost>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cost {
    pub base: String, // not sure how to conver yet
    pub premium: String,
}

pub async fn check_names(
    req: HttpRequest,
    query_names: web::Query<CheckQuery>,
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let names: Vec<String> = query_names
        .names
        .split(",")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    dbg!("{}", &names);
    match check_name_availability(state.get_ref(), &names).await {
        Ok(rows) => {
            let response = prepare_response_data(rows);
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::build(e.status_code()).json(e.to_api_error()),
    }
    // HttpResponse::Ok().finish()
}

// utils
fn prepare_response_data(rows: Vec<CheckNameResponse>) -> NameCheckData {
    let values = rows
        .into_iter()
        .map(|r| {
            let days_left = r.expires.and_then(u256_to_days_left);

            let registeration_price = r.price.map(|p| Cost {
                base: p.base.to_string(),
                premium: p.premium.to_string(),
            });

            NameCheckResponse {
                name: r.name,
                is_available: r.available,
                owner: r.owner,
                expiration: days_left,
                registeration_price,
            }
        })
        .collect();

    NameCheckData { values }
}

pub fn u256_to_days_left(expires: U256) -> Option<u64> {
    // fit unit seconds from ens to u64 for chrono
    let expiry_secs: u64 = expires.try_into().ok()?;

    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    if expiry_secs < now_secs {
        return Some(0);
    }

    let secs_left = expiry_secs - now_secs;

    Some(secs_left / 86_400)
}
