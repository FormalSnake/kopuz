use components::track_row::TrackRow;
use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;
use reader::{FavoritesStore, Library};
use server::jellyfin::JellyfinRemote;
use std::path::PathBuf;

#[component]
pub fn JellyfinFavorites(
    favorites_store: Signal<FavoritesStore>,
    library: Signal<Library>,
    config: Signal<AppConfig>,
    mut queue: Signal<Vec<reader::models::Track>>,
) -> Element {
    let mut ctrl = use_context::<PlayerController>();
    let mut active_menu_track = use_signal(|| None::<PathBuf>);
    let mut has_synced = use_signal(|| false);
    let mut is_syncing = use_signal(|| false);

    use_effect(move || {
        if !*has_synced.read() {
            has_synced.set(true);
            is_syncing.set(true);
            spawn(async move {
                let (server_config, device_id) = {
                    let conf = config.peek();
                    if let Some(server) = &conf.server {
                        if let (Some(token), Some(user_id)) =
                            (&server.access_token, &server.user_id)
                        {
                            (
                                Some((server.url.clone(), token.clone(), user_id.clone())),
                                conf.device_id.clone(),
                            )
                        } else {
                            (None, conf.device_id.clone())
                        }
                    } else {
                        (None, conf.device_id.clone())
                    }
                };

                if let Some((url, token, user_id)) = server_config {
                    let remote =
                        JellyfinRemote::new(&url, Some(&token), &device_id, Some(&user_id));
                    if let Ok(items) = remote.get_favorite_items().await {
                        let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
                        let mut store = favorites_store.write();
                        store.jellyfin_favorites = ids;
                    }
                }
                is_syncing.set(false);
            });
        }
    });

    let displayed_tracks: Vec<(reader::models::Track, Option<String>)> = {
        let store = favorites_store.read();
        let lib = library.read();
        let server = config.read();
        let server_ref = server.server.as_ref().cloned();

        lib.jellyfin_tracks
            .iter()
            .filter(|t| {
                let path_str = t.path.to_string_lossy();
                let parts: Vec<&str> = path_str.split(':').collect();
                if parts.len() >= 2 {
                    store.is_jellyfin_favorite(parts[1])
                } else {
                    false
                }
            })
            .map(|t| {
                let cover_url = if let Some(ref srv) = server_ref {
                    let path_str = t.path.to_string_lossy();
                    let parts: Vec<&str> = path_str.split(':').collect();
                    if parts.len() >= 2 {
                        let id = parts[1];
                        let mut url = format!("{}/Items/{}/Images/Primary", srv.url, id);
                        let mut params = Vec::new();
                        if parts.len() >= 3 {
                            params.push(format!("tag={}", parts[2]));
                        }
                        if let Some(token) = &srv.access_token {
                            params.push(format!("api_key={}", token));
                        }
                        if !params.is_empty() {
                            url.push('?');
                            url.push_str(&params.join("&"));
                        }
                        Some(url)
                    } else {
                        None
                    }
                } else {
                    None
                };
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
            if *is_syncing.read() {
                div {
                    class: "flex items-center gap-2 text-slate-400 text-sm mb-4",
                    i { class: "fa-solid fa-circle-notch fa-spin" }
                    span { "Syncing with Jellyfin..." }
                }
            }

            if is_empty && !*is_syncing.read() {
                div {
                    class: "flex flex-col items-center justify-center h-64 text-slate-500",
                    i { class: "fa-regular fa-heart text-4xl mb-4 opacity-30" }
                    p { class: "text-base", "No favorites yet." }
                    p { class: "text-sm mt-1 opacity-70",
                        "Heart a track while it's playing to add it here, and it'll sync to Jellyfin."
                    }
                }
            } else if !is_empty {
                div {
                    class: "space-y-1",
                    {tracks_nodes}
                }
            }
        }
    }
}
