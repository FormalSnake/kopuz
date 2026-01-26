use dioxus::prelude::*;

#[component]
pub fn Search() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-3xl font-bold text-white mb-6", "Search" }

            div {
                class: "relative max-w-2xl",
                i { class: "fa-solid fa-magnifying-glass absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" }
                input {
                    r#type: "text",
                    placeholder: "Search for artists, albums or tracks...",
                    class: "w-full bg-white/5 border border-white/10 rounded-full py-3 pl-12 pr-4 text-white focus:outline-none focus:border-white/20 transition-colors"
                }
            }

            div { class: "mt-12",
                h2 { class: "text-xl font-semibold text-white/80 mb-4", "Browse Categories" }
                div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                    for genre in ["Rock", "Pop", "Jazz", "Electronic"] {
                        div {
                            class: "aspect-video bg-gradient-to-br from-indigo-600 to-purple-700 rounded-xl p-4 cursor-pointer hover:scale-[1.02] transition-transform",
                            span { class: "text-lg font-bold text-white", "{genre}" }
                        }
                    }
                }
            }
        }
    }
}
