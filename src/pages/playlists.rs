use dioxus::prelude::*;

#[component]
pub fn PlaylistsPage() -> Element {
    rsx! {
        div {
            class: "p-6 text-white",
            "Playlists"
        }
    }
}
