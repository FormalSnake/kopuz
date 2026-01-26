use dioxus::prelude::*;
mod components;
pub mod config;
pub mod pages;
pub mod reader;
use components::{bottombar::Bottombar, sidebar::Sidebar};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Route {
    Home,
    Search,
    Library,
    Playlists,
    Settings,
}

const FAVICON: Asset = asset!("assets/favicon.ico");
const MAIN_CSS: Asset = asset!("assets/main.css");
const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

fn main() {
    let config =
        dioxus::desktop::Config::new().with_custom_protocol("artwork", |_headers, request| {
            let path = request.uri().path();
            let path = percent_encoding::percent_decode_str(path).decode_utf8_lossy();

            let mime = if path.ends_with(".png") {
                "image/png"
            } else {
                "image/jpeg"
            };

            let content = std::fs::read(path.as_ref())
                .map(|bytes| std::borrow::Cow::from(bytes))
                .unwrap_or_else(|_| std::borrow::Cow::from(Vec::new()));

            http::Response::builder()
                .header("Content-Type", mime)
                .body(content)
                .unwrap()
        });

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut library = use_signal(reader::Library::default);
    let mut current_route = use_signal(|| Route::Home);
    let cache_dir = use_memo(|| std::path::Path::new("./cache").to_path_buf());
    let lib_path = use_memo(move || cache_dir().join("library.json"));
    let config_path = use_memo(move || cache_dir().join("config.json"));
    let config = use_signal(|| config::AppConfig::load(&config_path()));
    let cover_cache = use_memo(move || std::path::Path::new("./cache/covers").to_path_buf());
    let mut trigger_rescan = use_signal(|| 0);

    use_effect(move || {
        let _ = config.read().save(&config_path());
    });

    use_hook(move || {
        spawn(async move {
            if let Ok(loaded) = reader::Library::load(&lib_path()) {
                library.set(loaded);
            }
        });
    });

    use_effect(move || {
        let music_dir = config.read().music_directory.clone();
        let _ = trigger_rescan.read();

        spawn(async move {
            if music_dir.exists() {
                let mut current_lib = library.peek().clone();

                if current_lib.root_path != music_dir {
                    current_lib = reader::Library::new(music_dir.clone());
                    library.set(current_lib.clone());
                }

                if let Ok(_) =
                    reader::scan_directory(music_dir, cover_cache(), &mut current_lib).await
                {
                    library.set(current_lib.clone());
                    let _ = current_lib.save(&lib_path());
                }
            }
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: "https://fonts.bunny.net/css?family=jetbrains-mono:400,500,700,800&display=swap" }
        document::Link { rel: "stylesheet", href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" }
        div {
            class: "flex flex-col h-screen",
            div {
                class: "flex flex-1 overflow-hidden",
                Sidebar {
                    current_route,
                    on_navigate: move |route| current_route.set(route)
                }
                div {
                    class: "flex-1 overflow-y-auto bg-black",
                    match *current_route.read() {
                        Route::Home => rsx! { pages::home::Home {} },
                        Route::Search => rsx! { pages::search::Search {} },
                        Route::Library => rsx! {
                            pages::library::LibraryPage {
                                library: library,
                                on_rescan: move |_| *trigger_rescan.write() += 1
                            }
                        },
                        Route::Playlists => rsx! { pages::playlists::PlaylistsPage {} },
                        Route::Settings => rsx! { pages::settings::Settings { config } },
                    }
                }
            }
            Bottombar {}
        }
    }
}
