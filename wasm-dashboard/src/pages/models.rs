use crate::api::{delete_model, deploy_model, fetch_config, fetch_models, undeploy_model};
use dioxus::prelude::*;

#[component]
pub fn Models() -> Element {
    // Use resource to fetch models and config from the server
    let mut models = use_resource(|| async { fetch_models().await });
    let config = use_resource(|| async { fetch_config().await });

    rsx! {
        div {
            class: "p-8",
            h1 {
                class: "text-3xl font-bold mb-6",
                "Models"
            }

            div {
                class: "bg-white rounded-lg shadow p-6",
                p {
                    class: "text-gray-600 mb-6",
                    "List of deployed wasmCloud models from the application deployment manager (wadm)"
                }

                // Deploy new model section
                DeployForm {
                    on_deploy_complete: move || {
                        models.restart();
                    }
                }

                // Display loading state, error state, or models
                match &*models.read() {
                    None => rsx! {
                        div {
                            class: "text-center py-8",
                            p {
                                class: "text-gray-500",
                                "Loading models..."
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div {
                            class: "text-center py-8 bg-red-50 border border-red-200 rounded p-4",
                            p {
                                class: "text-red-800 font-semibold",
                                "Failed to load models"
                            }
                            p {
                                class: "text-red-600 text-sm mt-2 font-mono",
                                "{e}"
                            }
                        }
                    },
                    Some(Ok(model_list)) => {
                        if model_list.is_empty() {
                            rsx! {
                                div {
                                    class: "text-center py-8 bg-yellow-50 border border-yellow-200 rounded p-4",
                                    p {
                                        class: "text-yellow-800",
                                        "No models deployed in the lattice"
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    class: "space-y-4",
                                    {model_list.iter().map(|model| {
                                        rsx! {
                                            ModelCard {
                                                key: "{model.name}",
                                                name: model.name.clone(),
                                                status: model.status.clone(),
                                                deployed: model.deployed.clone(),
                                                on_action_complete: move || {
                                                    models.restart();
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }

                div {
                    class: "mt-6 p-4 bg-blue-50 border border-blue-200 rounded",
                    match &*config.read() {
                        None => rsx! {
                            p {
                                class: "text-sm text-blue-800 font-mono",
                                "Loading configuration..."
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p {
                                class: "text-sm text-blue-600 font-mono",
                                "Data fetched from: {e}"
                            }
                        },
                        Some(Ok(cfg)) => rsx! {
                            p {
                                class: "text-sm text-blue-800 font-mono",
                                "Data fetched from: {cfg.wadm_endpoint}"
                            }
                            p {
                                class: "text-sm text-blue-800 font-mono mt-2",
                                "NATS Server: {cfg.nats_server}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ModelCard(
    name: String,
    status: String,
    deployed: String,
    on_action_complete: EventHandler<()>,
) -> Element {
    let mut undeploy_error = use_signal(|| String::new());
    let mut delete_error = use_signal(|| String::new());
    let mut deploy_error = use_signal(|| String::new());
    let mut undeploy_loading = use_signal(|| false);
    let mut delete_loading = use_signal(|| false);
    let mut deploy_loading = use_signal(|| false);

    let is_deployed = status.to_lowercase() == "deployed";

    let handle_undeploy = {
        let name = name.clone();
        move |_| {
            let name = name.clone();
            spawn({
                async move {
                    undeploy_loading.set(true);
                    undeploy_error.set(String::new());
                    match undeploy_model(name).await {
                        Ok(_) => {
                            undeploy_loading.set(false);
                            on_action_complete.call(());
                        }
                        Err(e) => {
                            undeploy_error.set(format!("Undeploy failed: {}", e));
                            undeploy_loading.set(false);
                        }
                    }
                }
            });
        }
    };

    let handle_delete = {
        let name = name.clone();
        move |_| {
            let name = name.clone();
            spawn({
                async move {
                    delete_loading.set(true);
                    delete_error.set(String::new());
                    match delete_model(name).await {
                        Ok(_) => {
                            delete_loading.set(false);
                            on_action_complete.call(());
                        }
                        Err(e) => {
                            delete_error.set(format!("Delete failed: {}", e));
                            delete_loading.set(false);
                        }
                    }
                }
            });
        }
    };

    let handle_deploy = {
        let name = name.clone();
        move |_| {
            let name = name.clone();
            spawn({
                async move {
                    deploy_loading.set(true);
                    deploy_error.set(String::new());
                    match deploy_model(name).await {
                        Ok(_) => {
                            deploy_loading.set(false);
                            on_action_complete.call(());
                        }
                        Err(e) => {
                            deploy_error.set(format!("Deploy failed: {}", e));
                            deploy_loading.set(false);
                        }
                    }
                }
            });
        }
    };

    rsx! {
        div {
            class: "border border-gray-200 rounded p-4 hover:shadow-md transition-shadow",
            div {
                class: "flex justify-between items-start mb-2",
                h3 {
                    class: "text-lg font-semibold text-gray-800",
                    "{name}"
                }
                span {
                    class: if is_deployed {
                        "px-3 py-1 bg-green-100 text-green-800 rounded-full text-sm font-medium"
                    } else {
                        "px-3 py-1 bg-yellow-100 text-yellow-800 rounded-full text-sm font-medium"
                    },
                    "{status}"
                }
            }
            p {
                class: "text-sm text-gray-500 mb-4",
                "Deployed: {deployed}"
            }

            // Error messages
            if !undeploy_error.read().is_empty() {
                div {
                    class: "mb-3 p-2 bg-red-50 border border-red-200 rounded text-red-700 text-sm",
                    "{undeploy_error}"
                }
            }
            if !delete_error.read().is_empty() {
                div {
                    class: "mb-3 p-2 bg-red-50 border border-red-200 rounded text-red-700 text-sm",
                    "{delete_error}"
                }
            }
            if !deploy_error.read().is_empty() {
                div {
                    class: "mb-3 p-2 bg-red-50 border border-red-200 rounded text-red-700 text-sm",
                    "{deploy_error}"
                }
            }

            // Action buttons - show Deploy/Delete if undeployed, Undeploy/Delete if deployed
            match is_deployed {
                true => rsx! {
                    div {
                        class: "flex gap-2",
                        button {
                            onclick: handle_undeploy,
                            disabled: undeploy_loading() || delete_loading(),
                            class: "px-3 py-2 bg-yellow-500 text-white rounded text-sm font-medium hover:bg-yellow-600 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors",
                            if undeploy_loading() {
                                "Undeploying..."
                            } else {
                                "Undeploy"
                            }
                        }
                        button {
                            onclick: handle_delete,
                            disabled: delete_loading() || undeploy_loading(),
                            class: "px-3 py-2 bg-red-500 text-white rounded text-sm font-medium hover:bg-red-600 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors",
                            if delete_loading() {
                                "Deleting..."
                            } else {
                                "Delete"
                            }
                        }
                    }
                },
                false => rsx! {
                    div {
                        class: "flex gap-2",
                        button {
                            onclick: handle_deploy,
                            disabled: deploy_loading(),
                            class: "px-3 py-2 bg-green-500 text-white rounded text-sm font-medium hover:bg-green-600 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors",
                            if deploy_loading() {
                                "Deploying..."
                            } else {
                                "Deploy"
                            }
                        }
                        button {
                            onclick: handle_delete,
                            disabled: delete_loading() || undeploy_loading(),
                            class: "px-3 py-2 bg-red-500 text-white rounded text-sm font-medium hover:bg-red-600 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors",
                            if delete_loading() {
                                "Deleting..."
                            } else {
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DeployForm(on_deploy_complete: EventHandler<()>) -> Element {
    let mut model_name = use_signal(|| String::new());
    let mut deploy_error = use_signal(|| String::new());
    let mut deploy_success = use_signal(|| String::new());
    let mut deploy_loading = use_signal(|| false);

    let handle_deploy = move |_| {
        let name = model_name.read().clone();
        if name.trim().is_empty() {
            deploy_error.set("Model name cannot be empty".to_string());
            return;
        }

        spawn({
            async move {
                deploy_loading.set(true);
                deploy_error.set(String::new());
                deploy_success.set(String::new());

                match deploy_model(name.clone()).await {
                    Ok(_response) => {
                        deploy_success.set(format!("Successfully deployed: {}", name));
                        model_name.set(String::new());
                        deploy_loading.set(false);
                        on_deploy_complete.call(());
                    }
                    Err(e) => {
                        deploy_error.set(format!("Deploy failed: {}", e));
                        deploy_loading.set(false);
                    }
                }
            }
        });
    };

    rsx! {
        div {
            class: "mb-6 p-4 bg-blue-50 border border-blue-200 rounded",
            h3 {
                class: "text-lg font-semibold text-blue-900 mb-3",
                "Deploy New Model"
            }

            div {
                class: "flex gap-2 mb-3",
                input {
                    class: "flex-1 px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500",
                    placeholder: "Enter model name (e.g., hello-world-app)",
                    value: "{model_name}",
                    oninput: move |evt| {
                        model_name.set(evt.value());
                        deploy_error.set(String::new());
                        deploy_success.set(String::new());
                    },
                    disabled: deploy_loading(),
                }
                button {
                    onclick: handle_deploy,
                    disabled: deploy_loading() || model_name.read().trim().is_empty(),
                    class: "px-4 py-2 bg-blue-500 text-white rounded font-medium hover:bg-blue-600 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors",
                    if deploy_loading() {
                        "Deploying..."
                    } else {
                        "Deploy"
                    }
                }
            }

            // Error message
            if !deploy_error.read().is_empty() {
                div {
                    class: "p-2 bg-red-50 border border-red-200 rounded text-red-700 text-sm",
                    "{deploy_error}"
                }
            }

            // Success message
            if !deploy_success.read().is_empty() {
                div {
                    class: "p-2 bg-green-50 border border-green-200 rounded text-green-700 text-sm",
                    "{deploy_success}"
                }
            }
        }
    }
}
