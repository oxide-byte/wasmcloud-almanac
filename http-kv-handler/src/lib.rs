mod bindings {
    wit_bindgen::generate!({
        generate_all,
    });
}

use bindings::wasi::keyvalue::store::open;
use wstd::http::{Body, Request, Method, Response, StatusCode};
use serde::Deserialize;

/// The keyvalue backend to use.
///
/// This constant is used as the bucket identifier passed to `open()`. The host
/// runtime selects the actual backend based on `.wash/config.yaml`:
///
/// | `BACKEND`      | Required config                              |
/// |----------------|----------------------------------------------|
/// | `"in_memory"`  | none (default)                               |
/// | `"filesystem"` | `wasi_keyvalue_path: /tmp/keyvalue-store`    |
/// | `"nats"`       | `wasi_keyvalue_nats_url: nats://...`         |
/// | `"redis"`      | `wasi_keyvalue_redis_url: redis://...`       |
///
/// Change this constant and uncomment the matching section in
/// `.wash/config.yaml` to switch backends.
const BACKEND: &str = "redis";

#[derive(Deserialize, Debug)]
struct QueryParams {
    key: String
}

#[derive(Deserialize, Debug)]
struct Payload {
    key: String,
    value: String
}

#[wstd::http_server]
async fn main(request: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    let result: Result<Response<Body>, wstd::http::Error>  = match request.method() {
        &Method::POST => handle_post(request).await,
        &Method::GET => handle_get(request).await,
        _ => response_factory("Method Not Allowed\n", StatusCode::METHOD_NOT_ALLOWED),
    };

    match result {
        Ok(response) => Ok(response),
        Err(err) => response_factory(format!("Error: {}",err).as_str(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn handle_get(request: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    if let Some(query_str) = request.uri().query() {
        if let Ok(key_value) = serde_qs::from_str::<QueryParams>(query_str) {

            let bucket = match open(BACKEND) {
                Ok(b) => b,
                Err(e) =>
                    return response_factory(format!("Error: {}",e).as_str(), StatusCode::BAD_REQUEST)
            };

            match bucket.get(&key_value.key) {
                Ok(Some(bytes)) => {
                    let message = format!("[{BACKEND}] {}\n", String::from_utf8_lossy(&bytes));
                    response_factory(message.as_str(), StatusCode::OK)
                },
                Ok(None) => response_factory("Key not found", StatusCode::NOT_FOUND),
                Err(e) => response_factory(format!("Error: {}",e).as_str(), StatusCode::INTERNAL_SERVER_ERROR),
            }
        } else {
            response_factory("Missing required query parameter: key\n", StatusCode::BAD_REQUEST)
        }
    } else {
        response_factory("Missing required query parameter: key\n", StatusCode::BAD_REQUEST)
    }
}

async fn handle_post(request: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    let body_bytes = request.into_body().bytes_contents().await;
    if body_bytes.is_ok() {
        match serde_json::from_slice::<Payload>(&body_bytes.unwrap()) {
            Ok(payload) => {
                
                let bucket = match open(BACKEND) {
                    Ok(b) => b,
                    Err(e) => return response_factory(format!("[{BACKEND}] Bucket cannot be accessed: {:?}",e).as_str(), StatusCode::INTERNAL_SERVER_ERROR),
                };

                match bucket.set(&payload.key, payload.value.as_bytes()) {
                    Ok(_) => response_factory("Key Stored", StatusCode::OK),
                    Err(e) => response_factory(format!("[{BACKEND}] Cannot store key/value {:?}",e).as_str(), StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
            Err(_e) => {
                response_factory("Invalid JSON (expected {{\"key\":\"...\",\"value\":\"...\"}})", StatusCode::BAD_REQUEST)
            }
        }
    } else {
        response_factory("Invalid JSON (expected {{\"key\":\"...\",\"value\":\"...\"}})", StatusCode::BAD_REQUEST)
    }
}

fn response_factory(message: &str, code: StatusCode) -> Result<Response<Body>, wstd::http::Error> {
    Response::builder()
        .status(code)
        .body(message.into())
        .map_err(Into::into)
}