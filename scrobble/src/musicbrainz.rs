use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
pub struct TrackMetadata<'a> {
    artist_name: &'a str,
    track_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    release_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_info: Option<HashMap<&'a str, &'a str>>,
}

#[derive(Serialize)]
pub struct Listen<'a> {
    listened_at: i64,
    track_metadata: TrackMetadata<'a>,
}

#[derive(Serialize)]
pub struct SubmitListens<'a> {
    listen_type: &'a str,
    payload: Vec<Listen<'a>>,
}

pub async fn submit_listens(
    token: &str,
    listens: Vec<Listen<'_>>,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = Client::new();
    let url = "https://api.listenbrainz.org/1/submit-listens";
    let body = SubmitListens {
        listen_type: "single",
        payload: listens,
    };

    let resp = client
        .post(url)
        .header("Authorization", token)
        .json(&body)
        .send()
        .await?;

    resp.error_for_status_ref()?;

    Ok(resp)
}

pub fn make_listen<'a>(artist: &'a str, track: &'a str, release: Option<&'a str>) -> Listen<'a> {
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    Listen {
        listened_at: now_unix,
        track_metadata: TrackMetadata {
            artist_name: artist,
            track_name: track,
            release_name: release.filter(|s| !s.is_empty()),
            additional_info: None,
        },
    }
}
