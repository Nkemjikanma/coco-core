//! tests/health_check.rs
use alloy::providers::{Provider, ProviderBuilder};
use coco::AppState;
use serde::Serialize;
use serde_json::json;
use std::net::TcpListener;

#[tokio::test]
pub async fn home_works() {
    let address = spawn_app().await;

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
    let address = spawn_app().await;

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
    let address = spawn_app().await;

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

pub async fn spawn_app() -> String {
    unsafe {
        std::env::set_var("ETH_RPC", "http://127.0.0.1:8545");
        std::env::set_var("BASE_RPC", "http://127.0.0.1:8545");
        std::env::set_var("SUBGRAPH_URL", "http://127.0.0.1:8000/subgraphs/name/test");
    }

    let config = coco::config::Config::load_env().expect("Missing required env vars for Config");

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

    let server = coco::run(listener, app_state).expect("Failed to bind server");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
