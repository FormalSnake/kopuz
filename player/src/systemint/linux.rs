use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

#[derive(Debug)]
pub enum SystemEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Prev,
}

static EVENT_SENDER: OnceLock<Sender<SystemEvent>> = OnceLock::new();
static EVENT_RECEIVER: OnceLock<Mutex<Receiver<SystemEvent>>> = OnceLock::new();

fn get_tx() -> Sender<SystemEvent> {
    EVENT_SENDER
        .get_or_init(|| {
            let (tx, rx) = mpsc::channel();
            let _ = EVENT_RECEIVER.set(Mutex::new(rx));
            tx
        })
        .clone()
}

pub fn poll_event() -> Option<SystemEvent> {
    let _ = get_tx(); // Initialize channels on first call
    EVENT_RECEIVER.get()?.lock().ok()?.try_recv().ok()
}

pub fn update_now_playing(
    _title: &str,
    _artist: &str,
    _album: &str,
    _duration: f64,
    _position: f64,
    _playing: bool,
    _artwork_path: Option<&str>,
) {
    // TODO: Implement MPRIS D-Bus server for Linux
    // This would require running an async D-Bus service to expose
    // now playing information to desktop environments like GNOME/KDE
}
