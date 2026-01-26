use crate::components::playlist_modal::PlaylistModal;
use crate::components::track_row::TrackRow;
use crate::hooks::use_search_data::use_search_data;
use crate::player::player;
use crate::reader::Library;
use dioxus::prelude::*;

#[component]
pub fn Search(
    library: Signal<Library>,
    playlist_store: Signal<crate::reader::PlaylistStore>,
    search_query: Signal<String>,
    player: Signal<player::Player>,
    mut is_playing: Signal<bool>,
    mut current_playing: Signal<u64>,
    mut current_song_cover_url: Signal<String>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    mut queue: Signal<Vec<crate::reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
) -> Element {
    let mut data = use_search_data(library, search_query);
    let mut selected_genre = use_signal(|| None::<String>);

    let mut active_menu_track = use_signal(|| None::<std::path::PathBuf>);
    let mut show_playlist_modal = use_signal(|| false);
    let mut selected_track_for_playlist = use_signal(|| None::<std::path::PathBuf>);

    let genre_tracks = use_memo(move || {
        let genre = selected_genre.read();
        if let Some(g) = &*genre {
            let lib = library.read();
            let valid_album_ids: std::collections::HashSet<&String> = lib
                .albums
                .iter()
                .filter(|a| a.genre.eq_ignore_ascii_case(g))
                .map(|a| &a.id)
                .collect();

            let album_map: std::collections::HashMap<&String, &crate::reader::models::Album> =
                lib.albums.iter().map(|a| (&a.id, a)).collect();

            let mut matching_tracks = Vec::new();
            for track in &lib.tracks {
                if valid_album_ids.contains(&track.album_id) {
                    let cover = album_map
                        .get(&track.album_id)
                        .and_then(|a| a.cover_path.as_ref())
                        .and_then(|c| crate::utils::format_artwork_url(Some(c)));
                    matching_tracks.push((track.clone(), cover));
                }
            }
            matching_tracks
        } else {
            Vec::new()
        }
    });

    rsx! {
        div {
            class: "p-8",

            if *show_playlist_modal.read() {
                PlaylistModal {
                    playlist_store: playlist_store,
                    on_close: move |_| show_playlist_modal.set(false),
                    on_add_to_playlist: move |playlist_id: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let mut store = playlist_store.write();
                            if let Some(playlist) = store.playlists.iter_mut().find(|p| p.id == playlist_id) {
                                if !playlist.tracks.contains(&path) {
                                    playlist.tracks.push(path);
                                }
                            }
                        }
                        show_playlist_modal.set(false);
                        active_menu_track.set(None);
                    },
                    on_create_playlist: move |name: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let mut store = playlist_store.write();
                            store.playlists.push(crate::reader::models::Playlist {
                                id: uuid::Uuid::new_v4().to_string(),
                                name,
                                tracks: vec![path],
                            });
                        }
                        show_playlist_modal.set(false);
                        active_menu_track.set(None);
                    }
                }
            }

            if let Some(genre) = selected_genre.read().as_ref() {
                div {
                    class: "space-y-6",
                    button {
                        class: "mb-4 flex items-center gap-2 text-slate-400 hover:text-white transition-colors",
                         onclick: move |_| selected_genre.set(None),
                         i { class: "fa-solid fa-arrow-left" }
                         "Back to Browse"
                    }

                    div { class: "flex items-end gap-6 mb-8",
                         if let Some((_, Some(url))) = (data.genres)().iter().find(|(g, _)| g == genre) {
                             img { src: "{url}", class: "w-48 h-48 rounded-lg shadow-2xl object-cover" }
                         } else {
                             div { class: "w-48 h-48 rounded-lg bg-gradient-to-br from-indigo-600 to-purple-700 flex items-center justify-center shadow-2xl",
                                 i { class: "fa-solid fa-music text-6xl text-white/20" }
                             }
                         }

                         div {
                             h2 { class: "text-sm font-bold text-white/60 uppercase tracking-widest mb-2", "Genre" }
                             h1 { class: "text-5xl font-bold text-white mb-4", "{genre}" }
                             p { class: "text-slate-400", "{genre_tracks.read().len()} tracks" }
                         }
                    }

                    div { class: "space-y-1 pb-20",
                         for (idx, (track, cover_url)) in genre_tracks.read().iter().enumerate() {
                             {
                                 let track = track.clone();
                                 let track_key = track.path.display().to_string();
                                 let track_menu = track.clone();
                                 let track_add = track.clone();
                                 let track_play = track.clone();
                                 let track_delete = track.clone();
                                 let cover_play = cover_url.clone();
                                 let is_menu_open = active_menu_track.read().as_ref() == Some(&track.path);
                                 let genre_tracks_list: Vec<crate::reader::models::Track> = genre_tracks.read().iter().map(|(t, _)| t.clone()).collect();

                                 rsx! {
                                     TrackRow {
                                         key: "{track_key}",
                                         track: track.clone(),
                                         cover_url: cover_url.clone(),
                                         is_menu_open: is_menu_open,
                                         on_click_menu: move |_| {
                                             if active_menu_track.read().as_ref() == Some(&track_menu.path) {
                                                 active_menu_track.set(None);
                                             } else {
                                                 active_menu_track.set(Some(track_menu.path.clone()));
                                             }
                                         },
                                         on_add_to_playlist: move |_| {
                                             selected_track_for_playlist.set(Some(track_add.path.clone()));
                                             show_playlist_modal.set(true);
                                             active_menu_track.set(None);
                                         },
                                         on_close_menu: move |_| active_menu_track.set(None),
                                         on_delete: move |_| {
                                             active_menu_track.set(None);
                                             if std::fs::remove_file(&track_delete.path).is_ok() {
                                                 library.write().remove_track(&track_delete.path);
                                                 let cache_dir = std::path::Path::new("./cache").to_path_buf();
                                                 let lib_path = cache_dir.join("library.json");
                                                 let _ = library.read().save(&lib_path);
                                             }
                                         },
                                         on_play: move |_| {
                                             queue.set(genre_tracks_list.clone());
                                             current_queue_index.set(idx);

                                             if let Ok(file) = std::fs::File::open(&track_play.path) {
                                                  if let Ok(source) = rodio::Decoder::new(std::io::BufReader::new(file)) {
                                                     player.write().play(source);
                                                     current_song_title.set(track_play.title.clone());
                                                     current_song_artist.set(track_play.artist.clone());
                                                     current_song_duration.set(track_play.duration);
                                                     current_song_progress.set(0);
                                                     is_playing.set(true);
                                                     if let Some(cover) = &cover_play {
                                                         current_song_cover_url.set(cover.clone());
                                                     }
                                                  }
                                             }
                                         }
                                     }
                                 }
                             }
                        }
                    }
                }
            } else {
                 div {
                    class: "relative max-w-2xl mb-8",
                    i { class: "fa-solid fa-magnifying-glass absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" }
                    input {
                        r#type: "text",
                        placeholder: "Search for artists, albums or tracks...",
                        class: "w-full bg-white/5 border border-white/10 rounded-full py-3 pl-12 pr-4 text-white focus:outline-none focus:border-white/20 transition-colors",
                        value: "{data.search_query}",
                        oninput: move |evt| data.search_query.set(evt.value())
                    }
                }

                if let Some((tracks, albums)) = (data.search_results)() {
                    div { class: "mt-8 space-y-8",
                        if !tracks.is_empty() {
                            div {
                                h2 { class: "text-xl font-semibold text-white/80 mb-4", "Tracks" }
                                div { class: "space-y-2",
                                    for (idx, (track, cover_url)) in tracks.iter().enumerate() {
                                        {
                                            let track = track.clone();
                                            let track_key = track.path.display().to_string();
                                            let track_menu = track.clone();
                                            let track_add = track.clone();
                                            let track_play = track.clone();
                                            let track_delete = track.clone();
                                            let cover_play = cover_url.clone();
                                            let is_menu_open = active_menu_track.read().as_ref() == Some(&track.path);
                                            let search_queue: Vec<crate::reader::models::Track> = tracks.iter().map(|(t, _)| t.clone()).collect();

                                            rsx! {
                                                TrackRow {
                                                    key: "{track_key}",
                                                    track: track.clone(),
                                                    cover_url: cover_url.clone(),
                                                    is_menu_open: is_menu_open,
                                                    on_click_menu: move |_| {
                                                        if active_menu_track.read().as_ref() == Some(&track_menu.path) {
                                                            active_menu_track.set(None);
                                                        } else {
                                                            active_menu_track.set(Some(track_menu.path.clone()));
                                                        }
                                                    },
                                                    on_add_to_playlist: move |_| {
                                                        selected_track_for_playlist.set(Some(track_add.path.clone()));
                                                        show_playlist_modal.set(true);
                                                        active_menu_track.set(None);
                                                    },
                                                    on_close_menu: move |_| active_menu_track.set(None),
                                                    on_delete: move |_| {
                                                        active_menu_track.set(None);
                                                        if std::fs::remove_file(&track_delete.path).is_ok() {
                                                            library.write().remove_track(&track_delete.path);
                                                            let cache_dir = std::path::Path::new("./cache").to_path_buf();
                                                            let lib_path = cache_dir.join("library.json");
                                                            let _ = library.read().save(&lib_path);
                                                        }
                                                    },
                                                    on_play: move |_| {
                                                        queue.set(search_queue.clone());
                                                        current_queue_index.set(idx);

                                                        if let Ok(file) = std::fs::File::open(&track_play.path) {
                                                            if let Ok(source) = rodio::Decoder::new(std::io::BufReader::new(file)) {
                                                                player.write().play(source);
                                                                current_song_title.set(track_play.title.clone());
                                                                current_song_artist.set(track_play.artist.clone());
                                                                current_song_duration.set(track_play.duration);
                                                                current_song_progress.set(0);
                                                                is_playing.set(true);
                                                                if let Some(cover) = &cover_play {
                                                                    current_song_cover_url.set(cover.clone());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if !albums.is_empty() {
                            div {
                                h2 { class: "text-xl font-semibold text-white/80 mb-4", "Albums" }
                                div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4",
                                    for (album, cover_url) in &albums {
                                        div {
                                            key: "{album.id}",
                                            class: "p-4 bg-white/5 rounded-xl hover:bg-white/10 transition-colors cursor-pointer group",
                                            div {
                                                class: "aspect-square rounded-lg bg-black/40 mb-3 overflow-hidden shadow-lg relative",
                                                if let Some(url) = cover_url {
                                                    img {
                                                        src: "{url}",
                                                        class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300",
                                                        loading: "lazy",
                                                        decoding: "async",
                                                    }
                                                } else {
                                                    div { class: "w-full h-full flex items-center justify-center",
                                                        i { class: "fa-solid fa-compact-disc text-4xl text-white/20" }
                                                    }
                                                }
                                            }
                                            h3 { class: "text-white font-medium truncate", "{album.title}" }
                                            p { class: "text-sm text-slate-400 truncate", "{album.artist}" }
                                        }
                                    }
                                }
                            }
                        }

                        if tracks.is_empty() && albums.is_empty() {
                            div { class: "text-center py-12 text-slate-500",
                                p { "No results found for \"{data.search_query}\"" }
                            }
                        }
                    }
                } else {
                    div { class: "mt-12",
                        h2 { class: "text-xl font-semibold text-white/80 mb-4", "Browse Genres" }
                        if (data.genres)().is_empty() {
                            p { class: "text-slate-500 italic", "No genres found in your library." }
                        } else {
                            div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                                for (genre, cover_url) in (data.genres)() {
                                    div {
                                        key: "{genre}",
                                        class: "aspect-video bg-gradient-to-br from-indigo-600 to-purple-700 rounded-xl p-4 cursor-pointer hover:scale-[1.02] transition-transform flex items-end relative overflow-hidden group content-visibility-auto",
                                        onclick: {
                                            let genre = genre.clone();
                                            move |_| selected_genre.set(Some(genre.clone()))
                                        },
                                        if let Some(url) = cover_url {
                                            img {
                                                src: "{url}",
                                                class: "absolute inset-0 w-full h-full object-cover opacity-60 group-hover:scale-110 transition-transform duration-500 will-change-transform",
                                                loading: "lazy",
                                                decoding: "async",
                                            }
                                            div { class: "absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent" }
                                        }
                                        span { class: "text-lg font-bold text-white relative z-10", "{genre}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
