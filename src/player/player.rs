use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use std::time::{Duration, Instant};

pub struct Player {
    #[allow(dead_code)]
    stream: OutputStream,
    sink: Sink,
    start_time: Option<Instant>,
    elapsed: Duration,
}

impl Player {
    pub fn new() -> Self {
        let stream = OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = Sink::connect_new(&stream.mixer());

        Self {
            stream,
            sink,
            start_time: None,
            elapsed: Duration::from_secs(0),
        }
    }

    pub fn play(&mut self, source: impl Source<Item = f32> + Send + 'static) {
        let new_sink = Sink::connect_new(&self.stream.mixer());
        new_sink.append(source);
        new_sink.play();

        self.sink = new_sink;

        self.start_time = Some(Instant::now());
        self.elapsed = Duration::from_secs(0);
    }

    pub fn pause(&mut self) {
        if !self.sink.is_paused() {
            self.sink.pause();
            if let Some(start) = self.start_time {
                self.elapsed += start.elapsed();
                self.start_time = None;
            }
        }
    }

    pub fn play_resume(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
            self.start_time = Some(Instant::now());
        }
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn get_position(&self) -> Duration {
        if let Some(start) = self.start_time {
            self.elapsed + start.elapsed()
        } else {
            self.elapsed
        }
    }

    pub fn seek(&mut self, time: Duration) {
        if self.sink.try_seek(time).is_ok() {
            self.elapsed = time;
            if !self.sink.is_paused() {
                self.start_time = Some(Instant::now());
            }
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
