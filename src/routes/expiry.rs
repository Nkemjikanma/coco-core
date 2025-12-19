use super::check::u256_to_days_left;
use crate::services::ens::check_name_expiry;
use crate::types::api::{AppState, CheckExpiryResponse, CheckQuery};
use actix_web::{HttpResponse, web};
use alloy::primitives::U256;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const GRACE_PERIOD_SECS: i64 = 90 * 24 * 60 * 60;
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiryData {
    pub values: Vec<ExpiryResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiryResponse {
    pub name: String,

    pub expiry_date: Option<String>,
    pub grace_period_end: Option<String>,
    pub is_expired: bool,
    pub is_in_grace_period: bool,
    pub days_until_expiry: Option<i64>,
}

pub async fn check_expiry(
    query_names: web::Query<CheckQuery>,
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let names: Vec<String> = query_names
        .names
        .split(",")
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .collect();

    match check_name_expiry(state.get_ref(), &names).await {
        Ok(rows) => {
            dbg!("{}", &rows);
            let response = prepare_response_data(rows);
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::build(e.status_code()).json(e.to_api_error()),
    }
}

fn prepare_response_data(rows: Vec<CheckExpiryResponse>) -> ExpiryData {
    let now_secs: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let values = rows
        .into_iter()
        .map(|r| {
            // If the name is available, it’s not registered: no expiry info.
            if r.available {
                return ExpiryResponse {
                    name: r.name,
                    expiry_date: None,
                    grace_period_end: None,
                    is_expired: false,
                    is_in_grace_period: false,
                    days_until_expiry: None,
                };
            }

            // Not available => should have an expiry.
            let expiry_secs_opt: Option<i64> = r.expiry_date.and_then(u256_to_i64);

            let grace_end_secs_opt = expiry_secs_opt.map(|s| s + GRACE_PERIOD_SECS);

            let is_expired = expiry_secs_opt
                .map(|expiry| now_secs >= expiry)
                .unwrap_or(false);

            let is_in_grace_period = match (expiry_secs_opt, grace_end_secs_opt) {
                (Some(expiry), Some(grace_end)) => now_secs >= expiry && now_secs < grace_end,
                _ => false,
            };

            let days_until_expiry =
                expiry_secs_opt.map(|expiry| ((expiry - now_secs) / 86_400).max(0));

            ExpiryResponse {
                name: r.name,
                expiry_date: expiry_secs_opt.and_then(unix_to_iso),
                grace_period_end: grace_end_secs_opt.and_then(unix_to_iso),
                is_expired,
                is_in_grace_period,
                days_until_expiry,
            }
        })
        .collect();

    ExpiryData { values }
}

fn u256_to_i64(u: U256) -> Option<i64> {
    u.try_into().ok()
}

fn unix_to_iso(secs: i64) -> Option<String> {
    let dt = DateTime::<Utc>::from_timestamp(secs, 0)?;
    Some(dt.to_rfc3339())
}
