//! The album counterpart of [`crate::track_actions::TrackActionsMenu`].
//!
//! Home renders albums as bare cards rather than through a row component, so
//! until now they carried no actions at all. The set is deliberately the subset
//! that needs nothing but an album id: deleting or editing an album belongs to
//! the album page, which has the state for it.

use crate::NavigationController;
use crate::dots_menu::{DotsMenu, MenuAction};
use dioxus::prelude::*;
use hooks::PlayerController;
use hooks::db_reactivity::Table;
use reader::Track;
use server::source::ActiveSource;

#[derive(Clone, Copy, PartialEq)]
enum Action {
    PlayNext,
    AddToQueue,
    AddToPlaylist,
    GoToArtist,
}

/// Album order as the album page shows it. A source is free to hand back its
/// own order, so queueing without this can interleave discs or open at track 7.
async fn album_tracks_in_order(source: &ActiveSource, album_id: &str) -> Vec<Track> {
    let mut tracks = source.album_tracks(album_id).await.unwrap_or_default();
    tracks.sort_by(|a, b| {
        a.disc_number
            .cmp(&b.disc_number)
            .then_with(|| a.track_number.cmp(&b.track_number))
            .then_with(|| a.title.cmp(&b.title))
    });
    tracks
}

/// The refs an album contributes to a playlist, in album order.
async fn album_track_refs(source: &ActiveSource, album_id: &str) -> Vec<String> {
    album_tracks_in_order(source, album_id)
        .await
        .iter()
        .map(|track| track.id.key().into_owned())
        .collect()
}

#[derive(Props, Clone, PartialEq)]
pub struct AlbumActionsMenuProps {
    pub album_id: String,
    pub album_title: String,
    #[props(default)]
    pub artist: String,

    /// Open state owned by the parent, for rows that keep at most one card menu
    /// open at a time. Leave unset and the menu owns its own state.
    #[props(default)]
    pub is_open: Option<bool>,
    #[props(default)]
    pub on_open: Option<EventHandler<()>>,
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,

    #[props(default)]
    pub button_class: String,
    #[props(default = "right".to_string())]
    pub anchor: String,
    #[props(default = "bottom".to_string())]
    pub placement: String,
}

#[component]
pub fn AlbumActionsMenu(props: AlbumActionsMenuProps) -> Element {
    let mut ctrl = use_context::<PlayerController>();
    let nav_ctrl = use_context::<NavigationController>();
    let active_source = use_context::<Signal<ActiveSource>>();
    let gens = hooks::db_reactivity::use_generations();
    let mut local_open = use_signal(|| false);
    let mut show_playlist_modal = use_signal(|| false);

    let capabilities = active_source.read().capabilities();
    let is_open = props.is_open.unwrap_or_else(|| *local_open.read());

    let on_open = props.on_open;
    let on_close = props.on_close;
    let mut close = move || match on_close {
        Some(handler) => handler.call(()),
        None => local_open.set(false),
    };

    let mut entries: Vec<(Action, MenuAction)> = vec![
        (
            Action::PlayNext,
            MenuAction::new(i18n::t("play_next"), "fa-solid fa-forward-step"),
        ),
        (
            Action::AddToQueue,
            MenuAction::new(i18n::t("add_to_queue"), "fa-solid fa-list-ul"),
        ),
    ];

    if capabilities.playlists != ::server::source::PlaylistOps::None {
        entries.push((
            Action::AddToPlaylist,
            MenuAction::new(i18n::t("add_to_playlist"), "fa-solid fa-plus"),
        ));
    }

    if !props.artist.trim().is_empty() {
        entries.push((
            Action::GoToArtist,
            MenuAction::new(i18n::t("go_to_artist"), "fa-solid fa-user"),
        ));
    }

    let dispatch: Vec<Action> = entries.iter().map(|(action, _)| *action).collect();
    let actions: Vec<MenuAction> = entries.into_iter().map(|(_, item)| item).collect();

    let dispatch_album = props.album_id.clone();
    let dispatch_artist = props.artist.clone();
    let add_album = props.album_id.clone();
    let create_album = props.album_id.clone();

    rsx! {
        DotsMenu {
            actions,
            is_open,
            aria_label: i18n::t_with("more_actions_for", &[("name", props.album_title.clone())]),
            button_class: props.button_class.clone(),
            anchor: props.anchor.clone(),
            placement: props.placement.clone(),
            on_open: move |_| {
                match on_open {
                    Some(handler) => handler.call(()),
                    None => local_open.set(true),
                }
            },
            on_close: move |_| close(),
            on_action: move |idx: usize| {
                let Some(action) = dispatch.get(idx).copied() else {
                    return;
                };
                match action {
                    Action::PlayNext | Action::AddToQueue => {
                        let source = active_source.peek().clone();
                        let album_id = dispatch_album.clone();
                        spawn(async move {
                            let tracks = album_tracks_in_order(&source, &album_id).await;
                            if tracks.is_empty() {
                                return;
                            }
                            if action == Action::PlayNext {
                                ctrl.queue_play_next(tracks);
                            } else {
                                ctrl.add_to_queue(tracks);
                            }
                        });
                    }
                    Action::AddToPlaylist => show_playlist_modal.set(true),
                    Action::GoToArtist => nav_ctrl.navigate_to_artist(dispatch_artist.clone()),
                }
                close();
            },
        }

        if *show_playlist_modal.read() {
            crate::playlist_modal::PlaylistModal {
                on_close: move |_| show_playlist_modal.set(false),
                on_add_to_playlist: move |playlist_id: String| {
                    let source = active_source.peek().clone();
                    let album_id = add_album.clone();
                    spawn(async move {
                        let refs = album_track_refs(&source, &album_id).await;
                        if refs.is_empty() {
                            return;
                        }
                        match source.add_to_playlist(&playlist_id, &refs).await {
                            Ok(_) => gens.bump(Table::Playlists),
                            Err(error) => tracing::warn!(%error, "album: add to playlist failed"),
                        }
                    });
                    show_playlist_modal.set(false);
                },
                on_create_playlist: move |name: String| {
                    let source = active_source.peek().clone();
                    let album_id = create_album.clone();
                    spawn(async move {
                        let refs = album_track_refs(&source, &album_id).await;
                        if refs.is_empty() {
                            return;
                        }
                        match source.create_playlist(&name, &refs).await {
                            Ok(_) => gens.bump(Table::Playlists),
                            Err(error) => tracing::warn!(%error, "album: create playlist failed"),
                        }
                    });
                    show_playlist_modal.set(false);
                },
            }
        }
    }
}
