//! A library scanned before the credit splitter existed still has to lose its
//! phantom "A$AP Rocky feat. Drake" artist, without a rescan.

use std::path::PathBuf;

use db::Source;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Executor};

fn unique_db() -> PathBuf {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("kopuz-ac-{pid}-{nanos}-{seq}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("kopuz.db")
}

/// Write rows the way the pre-split ingest did (the whole credit as one
/// artist), then clear the marker so the next open backfills them.
async fn seed_pre_split(db_path: &std::path::Path, rows: &[(&str, &str)]) {
    let mut conn = SqliteConnectOptions::new()
        .filename(db_path)
        .connect()
        .await
        .unwrap();
    conn.execute("BEGIN").await.unwrap();
    for (i, (key, artist)) in rows.iter().enumerate() {
        let artists_json = serde_json::to_string(&[artist]).unwrap();
        sqlx::query(
            "INSERT INTO tracks (source, track_key, title, artist, album, artists_json) \
             VALUES ('local', ?1, ?2, ?3, 'Album', ?4)",
        )
        .bind(key)
        .bind(format!("Track {i}"))
        .bind(artist)
        .bind(&artists_json)
        .execute(&mut conn)
        .await
        .unwrap();
    }
    conn.execute("DELETE FROM metadata_cache WHERE cache_key = 'artist_credits'")
        .await
        .unwrap();
    conn.execute("COMMIT").await.unwrap();
}

async fn stored_credits(db_path: &std::path::Path, track_key: &str) -> Vec<String> {
    let mut conn = SqliteConnectOptions::new()
        .filename(db_path)
        .connect()
        .await
        .unwrap();
    let json: String = sqlx::query_scalar("SELECT artists_json FROM tracks WHERE track_key = ?1")
        .bind(track_key)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    serde_json::from_str(&json).unwrap()
}

#[tokio::test]
async fn backfill_splits_joined_credits_without_a_rescan() {
    let db_path = unique_db();
    let db = db::init(&db_path).await.unwrap();
    drop(db);

    seed_pre_split(
        &db_path,
        &[
            ("/music/1.flac", "A$AP Rocky"),
            ("/music/2.flac", "A$AP Rocky feat. Drake"),
            ("/music/3.flac", "A$AP Rocky ft. Tyler, The Creator"),
            ("/music/4.flac", "Earth, Wind & Fire"),
        ],
    )
    .await;

    let db = db::init(&db_path).await.unwrap();

    assert_eq!(
        stored_credits(&db_path, "/music/1.flac").await,
        ["A$AP Rocky"]
    );
    assert_eq!(
        stored_credits(&db_path, "/music/2.flac").await,
        ["A$AP Rocky", "Drake"]
    );
    assert_eq!(
        stored_credits(&db_path, "/music/3.flac").await,
        ["A$AP Rocky", "Tyler, The Creator"]
    );
    // A real name that only looks like a join is left exactly as it was.
    assert_eq!(
        stored_credits(&db_path, "/music/4.flac").await,
        ["Earth, Wind & Fire"]
    );

    let artists = db.artists(&Source::Local).await.unwrap();
    let names: Vec<&str> = artists.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "A$AP Rocky",
            "Drake",
            "Earth, Wind & Fire",
            "Tyler, The Creator"
        ]
    );

    // The primary is counted on every track that credits them, not only the
    // ones where the joined string happened to match exactly.
    let count = |name: &str| {
        artists
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    assert_eq!(count("A$AP Rocky"), 3);
    assert_eq!(count("Drake"), 1);
    assert_eq!(count("Tyler, The Creator"), 1);
    assert_eq!(count("Earth, Wind & Fire"), 1);
}

#[tokio::test]
async fn backfill_runs_once_per_database() {
    let db_path = unique_db();
    drop(db::init(&db_path).await.unwrap());

    seed_pre_split(&db_path, &[("/music/1.flac", "A feat. B")]).await;
    drop(db::init(&db_path).await.unwrap());
    assert_eq!(stored_credits(&db_path, "/music/1.flac").await, ["A", "B"]);

    // A later hand edit is not undone by a second open.
    let mut conn = SqliteConnectOptions::new()
        .filename(&db_path)
        .connect()
        .await
        .unwrap();
    conn.execute("UPDATE tracks SET artists_json = '[\"Kept\"]'")
        .await
        .unwrap();
    drop(conn);

    drop(db::init(&db_path).await.unwrap());
    assert_eq!(stored_credits(&db_path, "/music/1.flac").await, ["Kept"]);
}

#[tokio::test]
async fn artists_falls_back_to_the_joined_column_when_no_credits_are_stored() {
    let db_path = unique_db();
    let db = db::init(&db_path).await.unwrap();

    let mut conn = SqliteConnectOptions::new()
        .filename(&db_path)
        .connect()
        .await
        .unwrap();
    conn.execute(
        "INSERT INTO tracks (source, track_key, title, artist, album, artists_json) \
         VALUES ('local', '/music/1.flac', 'T', 'Solo Artist', 'Album', '[]')",
    )
    .await
    .unwrap();
    drop(conn);

    let artists = db.artists(&Source::Local).await.unwrap();
    assert_eq!(artists, [("Solo Artist".to_string(), 1)]);
}

/// The case a user on the previous build is actually in: the first pass already
/// ran, stored its partial split, and burned the marker. The new rules have to
/// reach them anyway, and have to work from the untouched `artist` column,
/// because the first pass flattened the head/tail shape the contributor-list
/// rule reads.
#[tokio::test]
async fn a_library_backfilled_by_the_previous_revision_is_re_split() {
    const CREDIT: &str = "A$AP Rocky feat. Joe Fox x Future x M.I.A. \u{2022} A$AP Rocky \u{2022} \
                          Joe Fox \u{2022} Future \u{2022} M.I.A. \u{2022} Rakim Mayers \u{2022} \
                          Rameses Magnus-George \u{2022} Axel Morgan \u{2022} Ricci Rierra \u{2022} \
                          Nayvadius Wilburn";

    let db_path = unique_db();
    drop(db::init(&db_path).await.unwrap());

    let mut conn = SqliteConnectOptions::new()
        .filename(&db_path)
        .connect()
        .await
        .unwrap();
    // Exactly what the previous revision left behind: feat/x resolved, the
    // bullet tail still welded into one entry.
    let previous = serde_json::to_string(&[
        "A$AP Rocky",
        "Joe Fox",
        "Future",
        "M.I.A. \u{2022} A$AP Rocky \u{2022} Joe Fox \u{2022} Future \u{2022} M.I.A. \u{2022} \
         Rakim Mayers \u{2022} Rameses Magnus-George \u{2022} Axel Morgan \u{2022} Ricci Rierra \
         \u{2022} Nayvadius Wilburn",
    ])
    .unwrap();
    sqlx::query(
        "INSERT INTO tracks (source, track_key, title, artist, album, artists_json) \
         VALUES ('local', '/music/1.flac', 'T', ?1, 'Album', ?2)",
    )
    .bind(CREDIT)
    .bind(&previous)
    .execute(&mut conn)
    .await
    .unwrap();
    // Leave exactly the marker the previous revision wrote, which must not
    // block the new one.
    sqlx::query("DELETE FROM metadata_cache WHERE cache_key = 'artist_credits'")
        .execute(&mut conn)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO metadata_cache (cache_key, kind, payload) \
         VALUES ('artist_credits', 'split', '1')",
    )
    .execute(&mut conn)
    .await
    .unwrap();
    drop(conn);

    drop(db::init(&db_path).await.unwrap());

    assert_eq!(
        stored_credits(&db_path, "/music/1.flac").await,
        ["A$AP Rocky", "Joe Fox", "Future", "M.I.A."]
    );

    // One marker row, at the new revision: the superseded one is cleared out.
    let mut conn = SqliteConnectOptions::new()
        .filename(&db_path)
        .connect()
        .await
        .unwrap();
    let kinds: Vec<String> =
        sqlx::query_scalar("SELECT kind FROM metadata_cache WHERE cache_key = 'artist_credits'")
            .fetch_all(&mut conn)
            .await
            .unwrap();
    assert_eq!(kinds, ["v2-bullets"]);
}

/// A per-artist list richer than the credit string (Jellyfin's `Artists` array
/// against a joined display name) is not thrown away by re-deriving.
#[tokio::test]
async fn a_source_supplied_list_survives_the_backfill() {
    let db_path = unique_db();
    drop(db::init(&db_path).await.unwrap());

    let mut conn = SqliteConnectOptions::new()
        .filename(&db_path)
        .connect()
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO tracks (source, track_key, title, artist, album, artists_json) \
         VALUES ('local', '/music/1.flac', 'T', 'Gorillaz', 'Album', ?1)",
    )
    .bind(serde_json::to_string(&["Gorillaz", "Del The Funky Homosapien"]).unwrap())
    .execute(&mut conn)
    .await
    .unwrap();
    sqlx::query("DELETE FROM metadata_cache WHERE cache_key = 'artist_credits'")
        .execute(&mut conn)
        .await
        .unwrap();
    drop(conn);

    drop(db::init(&db_path).await.unwrap());

    assert_eq!(
        stored_credits(&db_path, "/music/1.flac").await,
        ["Gorillaz", "Del The Funky Homosapien"]
    );
}
