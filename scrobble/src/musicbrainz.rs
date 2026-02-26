use reqwest::Client;

#[derive(Debug)]
pub struct MusicBrainz {
    client: Client,
}

impl MusicBrainz {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}
