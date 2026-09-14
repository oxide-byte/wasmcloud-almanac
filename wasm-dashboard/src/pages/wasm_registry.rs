use crate::api::{delete_wasm_component, list_wasm_components, upload_wasm_file};
use dioxus::prelude::*;

#[component]
pub fn WasmRegistry() -> Element {
    let mut components = use_resource(|| async { list_wasm_components().await });

    rsx! {
        div {
            class: "p-8",
            h1 {
                class: "text-3xl font-bold mb-3",
                "WASM Registry"
            }

            div {
                class: "bg-white rounded-lg shadow p-6 mb-6",
                UploadForm {
                    on_upload_complete: move || {
                        components.restart();
                    }
                }
            }

            div {
                class: "bg-white rounded-lg shadow p-6",
                h2 {
                    class: "text-xl font-semibold mb-4",
                    "Components in Registry"
                }

                ComponentsList { components_resource: components }
            }
        }
    }
}

#[component]
fn UploadForm(on_upload_complete: EventHandler<()>) -> Element {
    let mut registry_name = use_signal(|| String::new());
    let mut tag = use_signal(|| String::from("0.1.0"));
    let mut selected_file_name = use_signal(|| Option::<String>::None);
    let mut file_bytes = use_signal(|| Vec::<u8>::new());
    let mut upload_status = use_signal(|| String::new());
    let mut is_uploading = use_signal(|| false);

    let handle_file_change = move |evt: FormEvent| {
        let file_list = evt.files();
        if let Some(file) = file_list.first() {
            selected_file_name.set(Some(file.name()));

            let file_clone = file.clone();
            spawn(async move {
                if let Ok(bytes) = file_clone.read_bytes().await {
                    file_bytes.set(bytes.to_vec());
                }
            });
        }
    };

    let handle_upload = move |_| {
        if registry_name().is_empty() {
            upload_status.set("Please enter a registry name".to_string());
            return;
        }

        if file_bytes().is_empty() {
            upload_status.set("Please select a WASM file".to_string());
            return;
        }

        is_uploading.set(true);
        upload_status.set("Uploading...".to_string());

        let name = registry_name();
        let tag_val = tag();
        let bytes = file_bytes();

        spawn({
            async move {
                match upload_wasm_file(name, tag_val, bytes).await {
                    Ok(msg) => {
                        upload_status.set(format!("Success: {}", msg));
                        registry_name.set(String::new());
                        tag.set(String::from("0.1.0"));
                        selected_file_name.set(None);
                        file_bytes.set(Vec::new());
                        on_upload_complete.call(());
                    }
                    Err(e) => {
                        upload_status.set(format!("Error: {}", e));
                    }
                }
                is_uploading.set(false);
            }
        });
    };

    rsx! {
        h2 {
            class: "text-xl font-semibold mb-4",
            "Upload WASM Component"
        }
        p {
            class: "text-gray-600 mb-6",
            "Select a compiled WASM file and specify the registry name and tag."
        }

        div {
            class: "space-y-4",

            div {
                class: "grid grid-cols-2 gap-4",
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 mb-2",
                        "Component Name"
                    }
                    input {
                        r#type: "text",
                        placeholder: "e.g., hello-world",
                        value: "{registry_name}",
                        oninput: move |evt| registry_name.set(evt.value()),
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                    }
                }
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 mb-2",
                        "Tag"
                    }
                    input {
                        r#type: "text",
                        placeholder: "e.g., 0.1.0",
                        value: "{tag}",
                        oninput: move |evt| tag.set(evt.value()),
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                    }
                }
            }

            div {
                label {
                    class: "block text-sm font-medium text-gray-700 mb-2",
                    "WASM File"
                }
                input {
                    r#type: "file",
                    accept: ".wasm",
                    onchange: handle_file_change,
                    disabled: is_uploading(),
                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                }
            }

            if let Some(file_name) = selected_file_name() {
                div {
                    class: "p-2 bg-blue-100 border border-blue-300 rounded text-blue-800 text-sm",
                    "Selected file: {file_name}"
                }
            }

            UploadStatusMessage { message: upload_status }

            button {
                r#type: "button",
                onclick: handle_upload,
                disabled: is_uploading() || registry_name().is_empty() || file_bytes().is_empty(),
                class: "w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:bg-gray-400",
                if is_uploading() { "Uploading..." } else { "Upload to Registry" }
            }
        }
    }
}

#[component]
fn ComponentsList(components_resource: Resource<Result<Vec<crate::api::WasmComponent>, dioxus::prelude::ServerFnError>>) -> Element {
    rsx! {
        match &*components_resource.read() {
            None => rsx! {
                div {
                    class: "text-center py-8",
                    p {
                        class: "text-gray-500",
                        "Loading components..."
                    }
                }
            },
            Some(Err(e)) => rsx! {
                div {
                    class: "text-center py-8 bg-red-50 border border-red-200 rounded p-4",
                    p {
                        class: "text-red-800 font-semibold",
                        "Error loading components:"
                    }
                    p {
                        class: "text-red-600 mt-2 text-sm",
                        "{e}"
                    }
                }
            },
            Some(Ok(comps)) => {
                if comps.is_empty() {
                    rsx! {
                        div {
                            class: "text-center py-8 bg-gray-50 border border-gray-200 rounded p-4",
                            p {
                                class: "text-gray-600",
                                "No components found in registry"
                            }
                        }
                    }
                } else {
                    rsx! {
                        div {
                            class: "overflow-x-auto",
                            table {
                                class: "w-full",
                                thead {
                                    tr {
                                        class: "bg-gray-100 border-b",
                                        th {
                                            class: "px-6 py-3 text-left font-semibold text-gray-700",
                                            "Name"
                                        }
                                        th {
                                            class: "px-6 py-3 text-left font-semibold text-gray-700",
                                            "Tag"
                                        }
                                        th {
                                            class: "px-6 py-3 text-left font-semibold text-gray-700",
                                            "Size"
                                        }
                                        th {
                                            class: "px-6 py-3 text-left font-semibold text-gray-700",
                                            "Digest (short)"
                                        }
                                        th {
                                            class: "px-6 py-3 text-left font-semibold text-gray-700",
                                            "Actions"
                                        }
                                    }
                                }
                                tbody {
                                    {comps.iter().map(|comp| {
                                        let comp_id = format!("{}:{}", comp.name, comp.tag);
                                        let short_digest = if comp.digest.len() > 12 {
                                            format!("{}...", &comp.digest[..12])
                                        } else {
                                            comp.digest.clone()
                                        };

                                        let name = comp.name.clone();
                                        let tag = comp.tag.clone();

                                        rsx! {
                                            ComponentRow {
                                                key: "{comp_id}",
                                                name: name.clone(),
                                                tag: tag.clone(),
                                                size: comp.size,
                                                digest: short_digest,
                                                on_delete: move |_| {
                                                    let name = name.clone();
                                                    let tag = tag.clone();
                                                    spawn({
                                                        async move {
                                                            let _ = delete_wasm_component(name, tag).await;
                                                            components_resource.restart();
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }
            }
        }

        button {
            class: "mt-6 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition",
            onclick: move |_| {
                components_resource.restart();
            },
            "Refresh"
        }
    }
}

#[component]
fn ComponentRow(
    name: String,
    tag: String,
    size: u64,
    digest: String,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        tr {
            class: "border-b hover:bg-gray-50",
            td {
                class: "px-6 py-4 text-gray-800 font-mono text-sm",
                "{name}"
            }
            td {
                class: "px-6 py-4 text-gray-800 font-mono text-sm",
                "{tag}"
            }
            td {
                class: "px-6 py-4 text-gray-600 text-sm",
                "{format_bytes(size)}"
            }
            td {
                class: "px-6 py-4 text-gray-600 text-xs font-mono",
                "{digest}"
            }
            td {
                class: "px-6 py-4",
                div {
                    class: "flex gap-2",
                    button {
                        class: "px-3 py-1 bg-red-600 text-white rounded text-sm hover:bg-red-700 transition",
                        onclick: on_delete,
                        "Delete"
                    }
                }
            }
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

#[component]
fn UploadStatusMessage(message: Signal<String>) -> Element {
    let status_text = message();
    let is_error = status_text.starts_with("Error");

    rsx! {
        if !status_text.is_empty() {
            div {
                class: if is_error {
                    "p-3 rounded bg-red-50 border border-red-200"
                } else {
                    "p-3 rounded bg-green-50 border border-green-200"
                },
                p {
                    class: if is_error {
                        "text-sm text-red-800"
                    } else {
                        "text-sm text-green-800"
                    },
                    "{status_text}"
                }
            }
        }
    }
}
