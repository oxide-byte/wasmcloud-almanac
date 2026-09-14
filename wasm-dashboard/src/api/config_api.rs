use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub nats_server: String,
    pub wadm_endpoint: String,
}

/// Get NATS server URL from environment or use default
pub fn get_nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| "nats://wasmcloud-almanac-nats-1:4222".to_string())
}

/// Fetch configuration from the server
#[server]
pub async fn fetch_config() -> Result<Config, ServerFnError> {
    let nats_server = get_nats_url();

    Ok(Config {
        nats_server,
        wadm_endpoint: "wadm.api.default.model.list".to_string(),
    })
}
