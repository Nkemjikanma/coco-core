use super::errors::ConfigError;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub eth_rpc: String,
    pub application_port: String,
    pub base_rpc: String,
    pub subgraph_url: String,
    pub database: DBConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DBConfig {
    pub username: String,
    pub password: String,
    pub port: String,
    pub host: String,
    pub database_name: String,
}

impl Config {
    pub fn load_env() -> Result<Self, ConfigError> {
        Ok(Self {
            eth_rpc: std::env::var("ETH_RPC")
                .map_err(|_| ConfigError::MissingEnv("ETH_RPC".to_string()))?,
            base_rpc: std::env::var("BASE_RPC")
                .map_err(|_| ConfigError::MissingEnv("BASE_RPC".to_string()))?,
            subgraph_url: std::env::var("SUBGRAPH_URL")
                .map_err(|_| ConfigError::MissingEnv("SUBGRAPH_URL".to_string()))?,
            application_port: std::env::var("APP_PORT")
                .map_err(|_| ConfigError::MissingEnv("APP_PORT".to_string()))?,
            database: DBConfig {
                username: std::env::var("DB_USERNAME").unwrap_or_else(|_| {
                    tracing::warn!("Using default username for db");
                    "postgres".to_string()
                }),
                password: std::env::var("DB_PASSWORD").unwrap_or_else(|_| {
                    tracing::warn!("Using default password for db");
                    "password".to_string()
                }),
                port: std::env::var("DB_PORT").unwrap_or_else(|_| {
                    tracing::warn!("Using default port for db");
                    "5432".to_string()
                }),

                host: std::env::var("DB_HOST").unwrap_or_else(|_| {
                    tracing::warn!("Using default host address for db");
                    "127.0.0.1".to_string()
                }),

                database_name: std::env::var("DB_NAME").unwrap_or_else(|_| {
                    tracing::warn!("Using default name for db");
                    "coco_core".to_string()
                }),
            },
        })
    }
}

impl DBConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
