use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-3xl font-bold text-white mb-6", "Welcome to Rusic" }
            p { class: "text-slate-400", "Start listening to your local music collection." }

            div {
                class: "mt-12 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                // Placeholder for recent activity
                for _i in 1..=3 {
                    div {
                        class: "bg-white/5 border border-white/5 rounded-xl p-6 hover:bg-white/10 transition-colors cursor-pointer group",
                        div { class: "w-full aspect-square bg-white/5 rounded-lg mb-4 flex items-center justify-center",
                            i { class: "fa-solid fa-music text-4xl text-white/20 group-hover:scale-110 transition-transform" }
                        }
                        div { class: "h-4 bg-white/10 rounded w-3/4 mb-2" }
                        div { class: "h-3 bg-white/5 rounded w-1/2" }
                    }
                }
            }
        }
    }
}
