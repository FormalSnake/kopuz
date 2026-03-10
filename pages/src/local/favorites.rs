use components::track_row::TrackRow;
use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;
use reader::{FavoritesStore, Library};
use std::path::PathBuf;

#[component]
pub fn LocalFavorites(
    favorites_store: Signal<FavoritesStore>,
    library: Signal<Library>,
    config: Signal<AppConfig>,
    mut queue: Signal<Vec<reader::models::Track>>,
) -> Element {
    let mut ctrl = use_context::<PlayerController>();
    let mut active_menu_track = use_signal(|| None::<PathBuf>);

    let displayed_tracks: Vec<(reader::models::Track, Option<String>)> = {
        let store = favorites_store.read();
        let lib = library.read();
        lib.tracks
            .iter()
            .filter(|t| store.is_local_favorite(&t.path))
            .map(|t| {
                let cover_url = lib
                    .albums
                    .iter()
                    .find(|a| a.id == t.album_id)
                    .and_then(|a| a.cover_path.as_ref())
                    .and_then(|cp| utils::format_artwork_url(Some(cp)));
                (t.clone(), cover_url)
            })
            .collect()
    };

    let queue_tracks: Vec<reader::models::Track> =
        displayed_tracks.iter().map(|(t, _)| t.clone()).collect();

    let is_empty = displayed_tracks.is_empty();

    let tracks_nodes = displayed_tracks
        .into_iter()
        .enumerate()
        .map(|(idx, (track, cover_url))| {
            let track_menu = track.clone();
            let queue_source = queue_tracks.clone();
            let track_key = format!("{}-{}", track.path.display(), idx);
            let is_menu_open = active_menu_track.read().as_ref() == Some(&track.path);

            rsx! {
                TrackRow {
                    key: "{track_key}",
                    track: track.clone(),
                    cover_url: cover_url.clone(),
                    is_menu_open,
                    on_click_menu: move |_| {
                        if active_menu_track.read().as_ref() == Some(&track_menu.path) {
                            active_menu_track.set(None);
                        } else {
                            active_menu_track.set(Some(track_menu.path.clone()));
                        }
                    },
                    on_add_to_playlist: move |_| active_menu_track.set(None),
                    on_close_menu: move |_| active_menu_track.set(None),
                    on_delete: move |_| active_menu_track.set(None),
                    on_play: move |_| {
                        queue.set(queue_source.clone());
                        ctrl.play_track(idx);
                    },
                }
            }
        });

    rsx! {
        div {
            if is_empty {
                div {
                    class: "flex flex-col items-center justify-center h-64 text-slate-500",
                    i { class: "fa-regular fa-heart text-4xl mb-4 opacity-30" }
                    p { class: "text-base", "No favorites yet." }
                    p { class: "text-sm mt-1 opacity-70",
                        "Heart a track while it's playing to add it here."
                    }
                }
            } else {
                div {
                    class: "space-y-1",
                    {tracks_nodes}
                }
            }
        }
    }
}
