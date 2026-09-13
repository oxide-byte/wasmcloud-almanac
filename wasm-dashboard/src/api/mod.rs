use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub status: String,
    pub deployed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub nats_server: String,
    pub wadm_endpoint: String,
}

/// Get NATS server URL from environment or use default
#[allow(dead_code)]
fn get_nats_url() -> String {
    std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://wasmcloud-almanac-nats-1:4222".to_string())
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

#[server]
pub async fn fetch_models() -> Result<Vec<Model>, ServerFnError> {
    use async_nats;

    // Connect to NATS server
    let nats_url = get_nats_url();
    let client = async_nats::connect(&nats_url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to NATS: {}", e)))?;

    // Request model list from wadm
    let response = client
        .request("wadm.api.default.model.list", "".into())
        .await
        .map_err(|e| ServerFnError::new(format!("NATS request failed: {}", e)))?;

    // Parse the response as JSON
    let response_str = String::from_utf8(response.payload.to_vec())
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {}", e)))?;

    // Parse as raw JSON
    let json: serde_json::Value = serde_json::from_str(&response_str)
        .map_err(|e| ServerFnError::new(format!("Failed to parse JSON: {}", e)))?;

    // Extract models from the response
    // The response is a JSON array directly, not wrapped in an object
    let models = if let Some(models_array) = json.as_array() {
        models_array
            .iter()
            .filter_map(|m| {
                let name = m.get("name")?.as_str()?.to_string();
                let status = m.get("status")?.as_str().unwrap_or("Unknown").to_string();
                // wadm returns the deployment version in the response
                let deployed = m
                    .get("deployed_version")
                    .and_then(|d| d.as_str())
                    .unwrap_or("N/A")
                    .to_string();
                Some(Model {
                    name,
                    status,
                    deployed,
                })
            })
            .collect()
    } else {
        // If response is not an array, return empty list
        vec![]
    };

    Ok(models)
}

/// Undeploy a model from the lattice
#[server]
pub async fn undeploy_model(model_name: String) -> Result<String, ServerFnError> {
    use async_nats;

    let nats_url = get_nats_url();
    let client = async_nats::connect(&nats_url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to NATS: {}", e)))?;

    let endpoint = format!("wadm.api.default.model.undeploy.{}", model_name);
    let response = client
        .request(endpoint, "".into())
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to undeploy model: {}", e)))?;

    let response_str = String::from_utf8(response.payload.to_vec())
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {}", e)))?;

    Ok(response_str)
}

/// Delete a model from the lattice
#[server]
pub async fn delete_model(model_name: String) -> Result<String, ServerFnError> {
    use async_nats;

    let nats_url = get_nats_url();
    let client = async_nats::connect(&nats_url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to NATS: {}", e)))?;

    let endpoint = format!("wadm.api.default.model.del.{}", model_name);
    let response = client
        .request(endpoint, "".into())
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete model: {}", e)))?;

    let response_str = String::from_utf8(response.payload.to_vec())
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {}", e)))?;

    Ok(response_str)
}

/// Upload a new model to the lattice via wadm.yaml
#[server]
pub async fn upload_model(yaml_content: String) -> Result<String, ServerFnError> {
    use async_nats;

    let nats_url = get_nats_url();
    let client = async_nats::connect(&nats_url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to NATS: {}", e)))?;

    // Send the yaml content to the wadm.api.default.model.put endpoint
    let response = client
        .request("wadm.api.default.model.put", yaml_content.into())
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to upload model: {}", e)))?;

    let response_str = String::from_utf8(response.payload.to_vec())
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {}", e)))?;

    Ok(response_str)
}

/// Deploy a model to the lattice
#[server]
pub async fn deploy_model(model_name: String) -> Result<String, ServerFnError> {
    use async_nats;

    let nats_url = get_nats_url();
    let client = async_nats::connect(&nats_url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to NATS: {}", e)))?;

    let endpoint = format!("wadm.api.default.model.deploy.{}", model_name);
    let response = client
        .request(endpoint, "".into())
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to deploy model: {}", e)))?;

    let response_str = String::from_utf8(response.payload.to_vec())
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {}", e)))?;

    Ok(response_str)
}
