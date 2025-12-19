//! src/lib.rs

pub mod config;
pub mod errors;
pub mod routes;
pub mod services;
pub mod startup;
pub mod types;

// use routes::{check::check_names, register::register, watch::watch};
// use types::api::RegisterBody;
//
// use actix_web::dev::Server;
// use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
// use serde::Deserialize;
//
// use std::net::TcpListener;
