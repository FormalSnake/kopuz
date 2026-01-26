use dioxus::prelude::*;

#[component]
pub fn Bottombar() -> Element {
    rsx! {
        div {
            class: "h-24 bg-[#050505] border-t border-white/5 px-4 flex items-center justify-between select-none shrink-0",

            div {
                class: "flex items-center gap-4 w-1/4",
                div {
                    class: "w-14 h-14 bg-white/5 rounded-md flex-shrink-0 overflow-hidden shadow-lg",
                    img {
                        src: "https://api.dicebear.com/7.x/identicon/svg?seed=music",
                        class: "w-full h-full object-cover"
                    }
                }
                div {
                    class: "flex flex-col min-w-0",
                    span { class: "text-sm font-bold text-white/90 truncate hover:underline cursor-pointer", "Song Title" }
                    span { class: "text-xs text-slate-400 truncate hover:text-white/70 cursor-pointer", "Artist Name" }
                }
                button {
                    class: "ml-2 text-slate-400 hover:text-red-400 transition-colors",
                    i { class: "fa-regular fa-heart" }
                }
            }

            div {
                class: "flex flex-col items-center max-w-[40%] w-full gap-2",
                div {
                    class: "flex items-center gap-6",
                    button { class: "text-slate-400 hover:text-white transition-all active:scale-95", i { class: "fa-solid fa-shuffle text-sm" } }
                    button { class: "text-slate-400 hover:text-white transition-all active:scale-90", i { class: "fa-solid fa-backward-step text-xl" } }
                    button {
                        class: "w-10 h-10 bg-white rounded-full flex items-center justify-center text-black hover:scale-105 active:scale-95 transition-all",
                        i { class: "fa-solid fa-play text-lg ml-0.5" }
                    }
                    button { class: "text-slate-400 hover:text-white transition-all active:scale-90", i { class: "fa-solid fa-forward-step text-xl" } }
                    button { class: "text-slate-400 hover:text-white transition-all active:scale-95", i { class: "fa-solid fa-repeat text-sm" } }
                }

                div {
                    class: "flex items-center gap-2 w-full",
                    span { class: "text-[10px] text-slate-500 w-8 text-right font-mono", "0:00" }
                    div {
                        class: "flex-1 h-1 bg-white/10 rounded-full group cursor-pointer relative",
                        div {
                            class: "absolute top-0 left-0 h-full w-1/3 bg-white group-hover:bg-green-500 rounded-full transition-colors",
                            div { class: "absolute -right-1.5 -top-1 w-3 h-3 bg-white rounded-full shadow-lg opacity-0 group-hover:opacity-100 transition-opacity" }
                        }
                    }
                    span { class: "text-[10px] text-slate-500 w-8 font-mono", "3:45" }
                }
            }

            div {
                class: "flex items-center justify-end gap-4 w-1/4",
                button { class: "text-slate-400 hover:text-white", i { class: "fa-solid fa-list-ul text-xs" } }
                button { class: "text-slate-400 hover:text-white", i { class: "fa-solid fa-desktop text-xs" } }
                div {
                    class: "flex items-center gap-2 group",
                    i { class: "fa-solid fa-volume-high text-xs text-slate-400 group-hover:text-white" }
                    div {
                        class: "w-24 h-1 bg-white/10 rounded-full group/vol cursor-pointer relative",
                        div {
                            class: "absolute top-0 left-0 h-full w-2/3 bg-white group-hover/vol:bg-green-500 rounded-full transition-colors",
                            div { class: "absolute -right-1.5 -top-1 w-3 h-3 bg-white rounded-full shadow-lg opacity-0 group-hover/vol:opacity-100 transition-opacity" }
                        }
                    }
                }
                button { class: "text-slate-400 hover:text-white", i { class: "fa-solid fa-up-right-and-down-left-from-center text-xs" } }
            }
        }
    }
}
