use crate::reader::Library;
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Copy)]
enum SortOrder {
    Title,
    Artist,
    Album,
}

#[component]
pub fn LibraryPage(library: Signal<Library>, on_rescan: EventHandler) -> Element {
    let lib = library.read();
    let mut sort_order = use_signal(|| SortOrder::Title);

    let artist_count = {
        let mut artists = std::collections::HashSet::new();
        for album in &lib.albums {
            artists.insert(&album.artist);
        }
        for track in &lib.tracks {
            artists.insert(&track.artist);
        }
        artists.len()
    };
    let mut all_tracks: Vec<_> = lib
        .tracks
        .iter()
        .map(|track| {
            let album = lib.albums.iter().find(|a| a.id == track.album_id);
            let cover_url = album.and_then(|a| a.cover_path.as_ref()).map(|p| {
                let p_str = p.to_string_lossy();
                if p_str.starts_with("./") {
                    let abs_path = std::env::current_dir()
                        .unwrap_or_default()
                        .join(&p_str[2..]);
                    format!("artwork://local{}", abs_path.to_string_lossy())
                } else {
                    p_str.into_owned()
                }
            });
            (track.clone(), cover_url)
        })
        .collect();

    match *sort_order.read() {
        SortOrder::Title => {
            all_tracks.sort_by(|(a, _), (b, _)| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        }
        SortOrder::Artist => all_tracks
            .sort_by(|(a, _), (b, _)| a.artist.to_lowercase().cmp(&b.artist.to_lowercase())),
        SortOrder::Album => {
            all_tracks.sort_by(|(a, _), (b, _)| a.album.to_lowercase().cmp(&b.album.to_lowercase()))
        }
    }

    rsx! {
        div {
            class: "p-8",
            div {
                class: "flex items-center justify-between mb-6",
                h1 { class: "text-3xl font-bold text-white", "Your Library" }
                button {
                    onclick: move |_| on_rescan.call(()),
                    class: "text-white/60 hover:text-white transition-colors p-2 rounded-full hover:bg-white/10",
                    title: "Rescan Library",
                    i { class: "fa-solid fa-rotate" }
                }
            }

            div {
                class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-12",
                StatCard { label: "Tracks", value: "{lib.tracks.len()}", icon: "fa-music" }
                StatCard { label: "Albums", value: "{lib.albums.len()}", icon: "fa-compact-disc" }
                StatCard { label: "Artists", value: "{artist_count}", icon: "fa-user" }
                StatCard { label: "Playlists", value: "0", icon: "fa-list" }
            }

            div {
                class: "flex items-center justify-between mb-4",
                h2 { class: "text-xl font-semibold text-white/80", "All Tracks" }
                div {
                    class: "flex space-x-1 bg-[#0A0A0A] border border-white/5 p-1 rounded-lg",
                    SortButton { active: *sort_order.read() == SortOrder::Title, label: "Title", onclick: move |_| sort_order.set(SortOrder::Title) }
                    SortButton { active: *sort_order.read() == SortOrder::Artist, label: "Artist", onclick: move |_| sort_order.set(SortOrder::Artist) }
                    SortButton { active: *sort_order.read() == SortOrder::Album, label: "Album", onclick: move |_| sort_order.set(SortOrder::Album) }
                }
            }
            div {
                class: "space-y-1",
                if lib.tracks.is_empty() {
                    p { class: "text-slate-500 italic", "Scanning your music collection..." }
                } else {
                    for (track , cover_url) in all_tracks {
                        div {
                            class: "flex items-center p-2 rounded-lg hover:bg-white/5 group transition-colors",
                            div { class: "w-10 h-10 bg-white/5 rounded overflow-hidden flex items-center justify-center mr-4 shrink-0",
                                if let Some(url) = cover_url {
                                    img {
                                        src: "{url}",
                                        class: "w-full h-full object-cover"
                                    }
                                } else {
                                    i { class: "fa-solid fa-music text-white/20" }
                                }
                            }
                            div { class: "flex-1 min-w-0",
                                p { class: "text-sm font-medium text-white/90 truncate",
                                    "{track.title}"
                                }
                                p { class: "text-xs text-slate-500 truncate",
                                    "{track.artist}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatCard(label: &'static str, value: String, icon: &'static str) -> Element {
    rsx! {
        div {
            class: "bg-[#0A0A0A] border border-white/5 p-5 rounded-xl flex items-center space-x-4",
            div {
                class: "w-12 h-12 rounded-lg bg-white/5 flex items-center justify-center shrink-0",
                i { class: "fa-solid {icon} text-lg text-white/60" }
            }
            div {
                p { class: "text-xs font-medium text-slate-500 uppercase tracking-wider", "{label}" }
                p { class: "text-2xl font-bold text-white", "{value}" }
            }
        }
    }
}

#[component]
fn SortButton(active: bool, label: &'static str, onclick: EventHandler) -> Element {
    rsx! {
        button {
            onclick: move |_| onclick.call(()),
            class: if active { "px-3 py-1 text-xs rounded-md bg-white/10 text-white font-medium transition-all" } else { "px-3 py-1 text-xs rounded-md text-white/40 hover:text-white/80 transition-all" },
            "{label}"
        }
    }
}
