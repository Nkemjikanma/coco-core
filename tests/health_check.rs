//! tests/health_check.rs
use alloy::providers::{Provider, ProviderBuilder};
use coco::types::api::AppState;
use serde::Serialize;
use serde_json::json;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

#[tokio::test]
pub async fn home_works() {
    let (_, address) = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/check/ens.eth", address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}

#[derive(Serialize)]
struct RegisterBody {
    name: String,
    duration: u8,
}

#[tokio::test]
async fn test_register_endpoint_returns_200() {
    let (_, address) = spawn_app().await;

    let client = reqwest::Client::new();

    let body: Vec<RegisterBody> = vec![RegisterBody {
        name: "ens.eth".to_string(),
        duration: 4,
    }];

    let response = client
        .post(&format!("{}/api/register", address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

// Parameterised test
#[tokio::test]
async fn test_register_endpoint_returns_400() {
    let (_, address) = spawn_app().await;

    let client = reqwest::Client::new();

    let test_case = vec![
        (json!([{"name": "", "duration": 4}]), "missing name"),
        (
            json!([{"name": "ens.eth", "duration": 0}]),
            "invalid duration",
        ),
    ];

    for (invalid_json_data, msg) in test_case {
        let response = client
            .post(format!("{}/api/register", address))
            .json(&invalid_json_data)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(400, response.status().as_u16(),)
    }
}

#[tokio::test]
async fn test_watch() {
    let (mut connection, address) = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/api/watch", address))
        .header("Content-Type", "application/json")
        .query("ens")
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let save = sqlx::query!("SELECT name, status::text FROM watch_list",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved name in watch list");
}

pub async fn spawn_app() -> (PgConnection, String) {
    unsafe {
        std::env::set_var("ETH_RPC", "http://127.0.0.1:8545");
        std::env::set_var("BASE_RPC", "http://127.0.0.1:8545");
        std::env::set_var("APP_PORT", "8000");
        std::env::set_var("SUBGRAPH_URL", "http://127.0.0.1:8000/subgraphs/name/test");
    }

    let config = coco::config::Config::load_env().expect("Missing required env vars for Config");

    let connection_string = config.database.connection_string();
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    let provider = ProviderBuilder::new()
        .connect(&config.eth_rpc)
        .await
        .expect("Failed to connect provider");

    let app_state = AppState {
        app_config: config,
        provider,
    };

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();

    let server = coco::startup::run(listener, app_state).expect("Failed to bind server");

    let _ = tokio::spawn(server);

    let address = format!("http://127.0.0.1:{}", port);
    (connection, address)
}
