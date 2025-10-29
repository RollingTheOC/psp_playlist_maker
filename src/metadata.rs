use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct MBTrack {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover_url: Option<String>,
}

pub async fn fetch_metadata(artist: &str, album: &str, title: &str) -> Option<MBTrack> {
    let query = format!("recording:{} AND artist:{} AND release:{}", title, artist, album);
    let url = format!("https://musicbrainz.org/ws/2/recording/?query={}&fmt=json", urlencoding::encode(&query));
    let resp = reqwest::get(&url).await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let recording = json["recordings"].get(0)?;
    let title = recording["title"].as_str()?.to_string();
    let artist = recording["artist-credit"][0]["name"].as_str()?.to_string();
    let album = recording["releases"][0]["title"].as_str()?.to_string();
    let cover_url = None; // For now, MusicBrainz does not provide cover art directly
    Some(MBTrack { title, artist, album, cover_url })
}
