use components::playlist_detail::PlaylistDetail;
use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use player::player;
use reader::{Library, PlaylistStore};

use crate::jellyfin::playlists::JellyfinPlaylists;
use crate::local::playlists::LocalPlaylists;

#[component]
pub fn PlaylistsPage(
    playlist_store: Signal<PlaylistStore>,
    library: Signal<Library>,
    config: Signal<AppConfig>,
    player: Signal<player::Player>,
    mut is_playing: Signal<bool>,
    mut current_playing: Signal<u64>,
    mut current_song_cover_url: Signal<String>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    mut queue: Signal<Vec<reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
    mut selected_playlist_id: Signal<Option<String>>,
) -> Element {
    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;

    let mut last_source = use_signal(|| config.read().active_source.clone());
    if *last_source.read() != config.read().active_source {
        selected_playlist_id.set(None);
        last_source.set(config.read().active_source.clone());
    }

    rsx! {
        div {
            class: "p-8",

            if let Some(pid) = selected_playlist_id.read().clone() {
                PlaylistDetail {
                    playlist_id: pid,
                    playlist_store,
                    library,
                    config,
                    player,
                    is_playing,
                    current_playing,
                    current_song_cover_url,
                    current_song_title,
                    current_song_artist,
                    current_song_duration,
                    current_song_progress,
                    queue,
                    current_queue_index,
                    on_close: move |_| selected_playlist_id.set(None),
                }
            } else {
                div { class: "flex items-center justify-between mb-8",
                    h1 { class: "text-3xl font-bold text-white", "Playlists" }
                }

                if is_jellyfin {
                    JellyfinPlaylists {
                        playlist_store,
                        library,
                        config,
                        selected_playlist_id,
                    }
                } else {
                    LocalPlaylists {
                        playlist_store,
                        library,
                        config,
                        selected_playlist_id,
                    }
                }
            }
        }
    }
}
