use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DeezerSearchResponse {
    data: Vec<DeezerAlbum>,
}

#[derive(Debug, Deserialize)]
struct DeezerAlbum {
    cover_big: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LastFmResponse {
    results: Option<LastFmResults>,
}

#[derive(Debug, Deserialize)]
struct LastFmResults {
    albummatches: Option<LastFmAlbumMatches>,
}

#[derive(Debug, Deserialize)]
struct LastFmAlbumMatches {
    album: Vec<LastFmAlbum>,
}

#[derive(Debug, Deserialize)]
struct LastFmAlbum {
    image: Vec<LastFmImage>,
}

#[derive(Debug, Deserialize)]
struct LastFmImage {
    #[serde(rename = "#text")]
    text: String,
    size: String,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzResponse {
    releases: Option<Vec<MusicBrainzRelease>>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzRelease {
    id: String,
}

/// Fetch album art from multiple sources with fallback chain
pub async fn fetch_album_art(artist: &str, album: &str) -> Option<String> {
    // Try sources in order of reliability and coverage
    
    // 1. Try Deezer (no API key required, good coverage, fast)
    if let Some(url) = fetch_deezer_art(artist, album).await {
        eprintln!("[Album Art] Found via Deezer: {}", url);
        return Some(url);
    }
    
    // 2. Try iTunes (fast, good for mainstream)
    if let Some(url) = fetch_itunes_art(artist, album).await {
        eprintln!("[Album Art] Found via iTunes: {}", url);
        return Some(url);
    }
    
    // 3. Try MusicBrainz + Cover Art Archive (comprehensive but slower)
    if let Some(url) = fetch_musicbrainz_art(artist, album).await {
        eprintln!("[Album Art] Found via MusicBrainz: {}", url);
        return Some(url);
    }
    
    // 4. Try Last.fm (requires API key, but good coverage)
    if let Some(url) = fetch_lastfm_art(artist, album).await {
        eprintln!("[Album Art] Found via Last.fm: {}", url);
        return Some(url);
    }
    
    eprintln!("[Album Art] No cover found for: {} - {}", artist, album);
    None
}

/// Fetch from Deezer API (no authentication required)
async fn fetch_deezer_art(artist: &str, album: &str) -> Option<String> {
    let query = format!("{} {}", artist, album);
    let url = format!(
        "https://api.deezer.com/search/album?q={}",
        urlencoding::encode(&query)
    );
    
    let resp = reqwest::get(&url).await.ok()?;
    let json: DeezerSearchResponse = resp.json().await.ok()?;
    
    json.data.first()?.cover_big.clone()
}

/// Fetch from MusicBrainz + Cover Art Archive
async fn fetch_musicbrainz_art(artist: &str, album: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .user_agent("PSPPlaylistMaker/0.1.0 (github.com/user/psp_playlist_maker)")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    
    // Try different query formats - MusicBrainz is picky about syntax
    let queries = vec![
        format!("\"{}\" AND \"{}\"", artist, album), // Exact phrase match
        format!("{} {}", artist, album),              // Simple search
        format!("artist:{} release:{}", artist, album), // Field search
    ];
    
    for query in queries {
        let url = format!(
            "https://musicbrainz.org/ws/2/release/?query={}&fmt=json&limit=5",
            urlencoding::encode(&query)
        );
        
        eprintln!("[MusicBrainz] Trying query: {}", query);
        
        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[MusicBrainz] Request failed: {}", e);
                continue;
            }
        };
        
        let json: MusicBrainzResponse = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[MusicBrainz] JSON parse failed: {}", e);
                continue;
            }
        };
        
        if let Some(releases) = json.releases {
            // Try each release until we find one with cover art
            for release in releases {
                let cover_url = format!("https://coverartarchive.org/release/{}/front-500", release.id);
                
                // Check if the cover exists (HEAD request is fast)
                match client.head(&cover_url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        eprintln!("[MusicBrainz] Found cover for release: {}", release.id);
                        return Some(cover_url);
                    }
                    Ok(resp) => {
                        eprintln!("[MusicBrainz] No cover for release {}: status {}", release.id, resp.status());
                    }
                    Err(e) => {
                        eprintln!("[MusicBrainz] Cover check failed for {}: {}", release.id, e);
                    }
                }
            }
        }
        
        // MusicBrainz rate limiting: wait 1 second between queries
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
    
    None
}

/// Fetch from Last.fm API
async fn fetch_lastfm_art(_artist: &str, album: &str) -> Option<String> {
    // Note: You need to get a free API key from https://www.last.fm/api/account/create
    // For now, we'll make it optional - if no key is set, this just returns None
    let api_key = std::env::var("LASTFM_API_KEY").ok()?;
    
    let url = format!(
        "https://ws.audioscrobbler.com/2.0/?method=album.search&album={}&api_key={}&format=json",
        urlencoding::encode(album),
        api_key
    );
    
    let resp = reqwest::get(&url).await.ok()?;
    let json: LastFmResponse = resp.json().await.ok()?;
    
    let albums = json.results?.albummatches?.album;
    if let Some(first_album) = albums.first() {
        // Get the largest image
        for img in &first_album.image {
            if img.size == "extralarge" && !img.text.is_empty() {
                return Some(img.text.clone());
            }
        }
    }
    
    None
}

/// Fetch from iTunes API (original implementation)
async fn fetch_itunes_art(artist: &str, album: &str) -> Option<String> {
    let query = format!("{} {}", artist, album);
    let url = format!(
        "https://itunes.apple.com/search?term={}&entity=album&limit=1",
        urlencoding::encode(&query)
    );
    
    let resp = reqwest::get(&url).await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let results = json["results"].as_array()?;
    
    if let Some(album) = results.first() {
        if let Some(art_url) = album["artworkUrl100"].as_str() {
            // Use higher-res version
            return Some(art_url.replace("100x100bb", "600x600bb"));
        }
    }
    
    None
}
