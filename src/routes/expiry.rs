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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grace_period_end: Option<String>,

    pub is_expired: bool,
    pub is_in_grace_period: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
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
            let expiry_secs_opt: Option<i64> = r.expiry_date.and_then(u256_to_i64);

            let (expiry_date, grace_period_end, days_until_expiry) =
                if let Some(expiry_secs) = expiry_secs_opt {
                    let grace_end_secs = expiry_secs + GRACE_PERIOD_SECS;
                    (
                        unix_to_iso(expiry_secs),
                        unix_to_iso(grace_end_secs),
                        Some(((expiry_secs - now_secs) / 86_400).max(0)),
                    )
                } else {
                    (None, None, None)
                };

            let is_expired = match expiry_secs_opt {
                Some(expiry_secs) => now_secs >= expiry_secs,
                None => false,
            };

            let is_in_grace_period = match expiry_secs_opt {
                Some(expiry_secs) => {
                    let grace_end = expiry_secs + GRACE_PERIOD_SECS;
                    now_secs >= expiry_secs && now_secs < grace_end
                }
                None => false,
            };

            ExpiryResponse {
                name: r.name,
                expiry_date,
                grace_period_end,
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
