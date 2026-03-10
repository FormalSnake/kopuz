use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use reader::Library;

use crate::jellyfin::logs::JellyfinLogs;
use crate::local::logs::LocalLogs;

#[component]
pub fn Logs(library: Signal<Library>, config: Signal<AppConfig>) -> Element {
    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;

    rsx! {
        if is_jellyfin {
            JellyfinLogs { library, config }
        } else {
            LocalLogs { library, config }
        }
    }
}
