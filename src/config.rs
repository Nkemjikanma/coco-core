use super::errors::ConfigError;

#[derive(Debug, Clone)]
pub struct Config {
    pub eth_rpc: String,
    // PORT: u16,
    pub base_rpc: String,
    pub subgraph_url: String,
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
        })
    }
}
