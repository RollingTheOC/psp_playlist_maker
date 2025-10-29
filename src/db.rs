use rusqlite::{Connection, Result};
use crate::music::Track;

pub fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL,
            artist TEXT,
            album TEXT,
            title TEXT
        )",
        [],
    )?;
    Ok(conn)
}

pub fn insert_track(conn: &Connection, track: &Track) -> Result<()> {
    conn.execute(
        "INSERT INTO tracks (path, artist, album, title) VALUES (?1, ?2, ?3, ?4)",
        [&track.path, &track.artist, &track.album, &track.title],
    )?;
    Ok(())
}

use crate::music::MusicLibrary;

pub fn save_library(conn: &mut Connection, library: &MusicLibrary) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO tracks (path, artist, album, title) VALUES (?1, ?2, ?3, ?4)"
        )?;
        for track in &library.tracks {
            let mut track = track.clone();
            track.path = track.path.replace(" ", "%20");
            stmt.execute([&track.path, &track.artist, &track.album, &track.title])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn load_tracks(conn: &Connection) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare("SELECT path, artist, album, title FROM tracks")?;
    let tracks = stmt.query_map([], |row| {
        Ok(Track {
            path: row.get::<_, String>(0)?.replace("%20", " "),
            artist: row.get(1)?,
            album: row.get(2)?,
            title: row.get(3)?,
        })
    })?;
    tracks.collect()
}
