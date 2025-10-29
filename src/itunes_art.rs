use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ITunesArt {
    pub url: String,
}

pub async fn fetch_itunes_art(artist: &str, album: &str) -> Option<String> {
    let query = format!("{} {}", artist, album);
    let url = format!("https://itunes.apple.com/search?term={}&entity=album&limit=1", urlencoding::encode(&query));
    let resp = reqwest::get(&url).await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let results = json["results"].as_array()?;
    if let Some(album) = results.first() {
        if let Some(art_url) = album["artworkUrl100"].as_str() {
            // Use higher-res version if available
            return Some(art_url.replace("100x100bb", "600x600bb"));
        }
    }
    None
}
