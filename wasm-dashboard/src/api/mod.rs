pub mod config_api;
pub mod nats_api;
pub mod registry_api;

pub use config_api::fetch_config;
pub use config_api::get_nats_url;

pub use nats_api::delete_model;
pub use nats_api::deploy_model;
pub use nats_api::fetch_models;
pub use nats_api::undeploy_model;
pub use nats_api::upload_model;

pub use registry_api::{
    delete_wasm_component, list_wasm_components, upload_wasm_file, WasmComponent,
};
