use crate::dots_menu::{DotsMenu, MenuAction};
use dioxus::prelude::*;
use reader::models::Track;

#[component]
pub fn TrackRow(
    track: Track,
    cover_url: Option<String>,
    on_click_menu: EventHandler<()>,
    is_menu_open: bool,
    on_add_to_playlist: EventHandler<()>,
    on_close_menu: EventHandler<()>,
    on_play: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    let actions = vec![
        MenuAction::new("Add to Playlist", "fa-solid fa-plus"),
        MenuAction::new("Delete Song", "fa-solid fa-trash").destructive(),
    ];

    rsx! {
        div {
            class: "flex items-center p-2 rounded-lg hover:bg-white/5 group transition-colors relative",
            onclick: move |_| on_play.call(()),

            div { class: "w-10 h-10 bg-white/5 rounded overflow-hidden flex items-center justify-center mr-4 shrink-0",
                if let Some(url) = cover_url {
                    img {
                        src: "{url}",
                        class: "w-full h-full object-cover",
                        loading: "lazy",
                        decoding: "async",
                    }
                } else {
                    i { class: "fa-solid fa-music text-white/20" }
                }
            }

            div { class: "flex-1 min-w-0 pr-4",
                p { class: "text-sm font-medium text-white/90 truncate", "{track.title}" }
                p { class: "text-xs text-slate-500 truncate", "{track.artist}" }
            }

            DotsMenu {
                actions,
                is_open: is_menu_open,
                on_open: move |_| on_click_menu.call(()),
                on_close: move |_| on_close_menu.call(()),
                button_class: "opacity-0 group-hover:opacity-100 focus:opacity-100".to_string(),
                anchor: "right".to_string(),
                on_action: move |idx: usize| match idx {
                    0 => on_add_to_playlist.call(()),
                    1 => on_delete.call(()),
                    _ => {}
                },
            }
        }
    }
}
