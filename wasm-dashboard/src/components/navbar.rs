use dioxus::prelude::*;
use crate::pages::{Home, Models};

#[component]
fn Navbar() -> Element {
    rsx! {
        nav {
            id: "navbar",
            class: "flex gap-4 p-4 bg-gray-800 text-white",
            Link {
                to: Route::Home {},
                class: "hover:text-gray-300 font-medium",
                "Home"
            }
            Link {
                to: Route::Models {},
                class: "hover:text-gray-300 font-medium",
                "Models"
            }
        }
        Outlet::<Route> {}
    }
}

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/models")]
    Models {},
}
