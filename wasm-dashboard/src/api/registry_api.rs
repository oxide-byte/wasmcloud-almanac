use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmComponent {
    pub name: String,
    pub tag: String,
    pub digest: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RegistryResponse {
    pub repositories: Vec<String>,
}

/// List all WASM components in the registry
#[server]
pub async fn list_wasm_components() -> Result<Vec<WasmComponent>, ServerFnError> {
    use reqwest::Client;

    let registry_url = "http://localhost:5001";
    let client = Client::new();

    // Fetch repositories
    let repo_response = client
        .get(&format!("{}/v2/_catalog", registry_url))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to registry: {}", e)))?;

    let repo_data: RegistryResponse = repo_response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse repositories: {}", e)))?;

    let mut components = Vec::new();

    // For each repository, fetch its tags
    for repo_name in repo_data.repositories {
        let tags_response = client
            .get(&format!("{}/v2/{}/tags/list", registry_url, repo_name))
            .send()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch tags for {}: {}", repo_name, e)))?;

        if let Ok(tags_data) = tags_response.json::<serde_json::Value>().await {
            if let Some(tags) = tags_data.get("tags").and_then(|t| t.as_array()) {
                for tag in tags {
                    if let Some(tag_str) = tag.as_str() {
                        // Fetch manifest to get digest and size
                        let manifest_response = client
                            .get(&format!("{}/v2/{}/manifests/{}", registry_url, repo_name, tag_str))
                            .header("Accept", "application/vnd.docker.distribution.manifest.v2+json,application/vnd.oci.image.manifest.v1+json")
                            .send()
                            .await;

                        if let Ok(manifest) = manifest_response {
                            let digest = manifest
                                .headers()
                                .get("docker-content-digest")
                                .and_then(|h| h.to_str().ok())
                                .unwrap_or("unknown")
                                .to_string();

                            let size = manifest
                                .content_length()
                                .unwrap_or(0);

                            components.push(WasmComponent {
                                name: repo_name.clone(),
                                tag: tag_str.to_string(),
                                digest,
                                size,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(components)
}

/// Delete a WASM component from the registry
#[server]
pub async fn delete_wasm_component(name: String, tag: String) -> Result<String, ServerFnError> {
    use reqwest::Client;
    use sha2::{Digest, Sha256};

    let registry_url = "http://localhost:5001";
    let client = Client::new();

    // Fetch the manifest to get the digest
    let manifest_url = format!("{}/v2/{}/manifests/{}", registry_url, name, tag);

    let manifest_response = client
        .get(&manifest_url)
        .header("Accept", "application/vnd.docker.distribution.manifest.v2+json,application/vnd.oci.image.manifest.v1+json")
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch manifest: {}", e)))?;

    if !manifest_response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to fetch manifest: HTTP {}",
            manifest_response.status()
        )));
    }

    // Try to get the digest from the response header
    let digest = match manifest_response
        .headers()
        .get("docker-content-digest")
        .and_then(|h| h.to_str().ok())
    {
        Some(d) => d.to_string(),
        None => {
            // Fallback: calculate digest from response body
            let body = manifest_response
                .bytes()
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to read manifest body: {}", e)))?;

            let mut hasher = Sha256::new();
            hasher.update(&body);
            let digest_hash = hasher.finalize();
            format!("sha256:{:x}", digest_hash)
        }
    };

    // Delete using the digest
    let delete_url = format!("{}/v2/{}/manifests/{}", registry_url, name, &digest);

    let delete_response = client
        .delete(&delete_url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete component: {}", e)))?;

    match delete_response.status() {
        status if status.is_success() || status.as_u16() == 202 => {
            Ok(format!("Successfully deleted {}:{}", name, tag))
        }
        status => {
            let body = delete_response
                .text()
                .await
                .unwrap_or_default();
            Err(ServerFnError::new(format!(
                "Failed to delete component: HTTP {} - {}",
                status, body
            )))
        }
    }
}

/// Upload a WASM component file to the registry using OCI Registry V2 protocol
#[server]
pub async fn upload_wasm_file(
    registry_name: String,
    tag: String,
    file_data: Vec<u8>,
) -> Result<String, ServerFnError> {
    use reqwest::Client;
    use sha2::{Digest, Sha256};

    let registry_url = "http://localhost:5001";
    let client = Client::new();

    // Calculate the SHA256 digest of the blob
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let digest = hasher.finalize();
    let digest_str = format!("sha256:{:x}", digest);

    // Step 1: Initiate blob upload
    let init_url = format!("{}/v2/{}/blobs/uploads/", registry_url, registry_name);
    let init_response = client
        .post(&init_url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to initiate upload: {}", e)))?;

    let upload_location = init_response
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ServerFnError::new("No upload location in response".to_string()))?
        .to_string();

    // Step 2: PATCH to upload blob data
    let patch_response = client
        .patch(&upload_location)
        .header("Content-Type", "application/octet-stream")
        .body(file_data.clone())
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to upload blob data: {}", e)))?;

    if patch_response.status() != reqwest::StatusCode::ACCEPTED {
        return Err(ServerFnError::new(format!(
            "Blob upload PATCH failed: HTTP {}",
            patch_response.status()
        )));
    }

    let upload_location2 = patch_response
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ServerFnError::new("No location after PATCH".to_string()))?
        .to_string();

    // Step 3: PUT to complete blob upload with digest
    let complete_url = format!("{}&digest={}", upload_location2, digest_str);
    let complete_response = client
        .put(&complete_url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to complete blob upload: {}", e)))?;

    if !complete_response.status().is_success() {
        let body = complete_response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Failed to complete blob upload: {}",
            body
        )));
    }

    // Step 4: Create and upload the manifest
    let manifest = serde_json::json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
        "config": {
            "mediaType": "application/vnd.wasmcloud.component.v1+wasm",
            "size": file_data.len(),
            "digest": digest_str
        }
    });

    let manifest_json = serde_json::to_vec(&manifest)
        .map_err(|e| ServerFnError::new(format!("Failed to serialize manifest: {}", e)))?;

    let manifest_url = format!("{}/v2/{}/manifests/{}", registry_url, registry_name, tag);
    let manifest_response = client
        .put(&manifest_url)
        .header("Content-Type", "application/vnd.docker.distribution.manifest.v2+json")
        .body(manifest_json)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to upload manifest: {}", e)))?;

    if !manifest_response.status().is_success() {
        let status = manifest_response.status();
        let body = manifest_response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Failed to upload manifest: HTTP {} - {}",
            status, body
        )));
    }

    Ok(format!(
        "Successfully uploaded {}:{} to registry",
        registry_name, tag
    ))
}
