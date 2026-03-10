use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use reader::{FavoritesStore, Library};

use crate::jellyfin::favorites::JellyfinFavorites;
use crate::local::favorites::LocalFavorites;

#[component]
pub fn FavoritesPage(
    favorites_store: Signal<FavoritesStore>,
    library: Signal<Library>,
    config: Signal<AppConfig>,
    player: Signal<player::player::Player>,
    mut is_playing: Signal<bool>,
    mut current_playing: Signal<u64>,
    mut current_song_cover_url: Signal<String>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    mut queue: Signal<Vec<reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
) -> Element {
    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;

    rsx! {
        div {
            class: "p-8 min-h-full",

            div {
                class: "flex items-center gap-3 mb-8",
                i { class: "fa-solid fa-heart text-red-400 text-2xl" }
                h1 { class: "text-3xl font-bold text-white", "Favorites" }
            }

            if is_jellyfin {
                JellyfinFavorites {
                    favorites_store,
                    library,
                    config,
                    queue,
                }
            } else {
                LocalFavorites {
                    favorites_store,
                    library,
                    config,
                    queue,
                }
            }
        }
    }
}
