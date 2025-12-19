use crate::{
    errors::CocoError,
    types::api::{AddressQuery, AppState},
};
use actix_web::{HttpResponse, web};
use alloy::primitives::{Address, address};
use std::sync::Arc;

pub async fn check_portfolio(
    query_address: web::Query<AddressQuery>,
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let query = &query_address.address;

    HttpResponse::Ok().finish()
}
