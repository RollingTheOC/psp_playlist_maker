use lofty::prelude::*;
use lofty::probe::Probe;
use std::collections::HashMap;
use std::sync::Mutex;
use std::fs;
use std::time::SystemTime;

lazy_static::lazy_static! {
    static ref METADATA_CACHE: Mutex<HashMap<(String, u64), Option<(String, String, String)>>> = Mutex::new(HashMap::new());
}

pub fn extract_embedded_art(path: &str) -> Option<Vec<u8>> {
    let tagged_file = Probe::open(path).ok()?.read().ok()?;
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag())?;
    let pictures = tag.pictures();
    if !pictures.is_empty() {
        Some(pictures[0].data().to_vec())
    } else {
        None
    }
}

pub fn extract_metadata(path: &str) -> Option<(String, String, String)> {
    let mtime = fs::metadata(path).and_then(|m| m.modified()).ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs()).unwrap_or(0);
    let key = (path.to_string(), mtime);
    {
        let cache = METADATA_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }
    let tagged_file = Probe::open(path).ok()?.read().ok()?;
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag())?;
    let title = tag.get_string(&ItemKey::TrackTitle).unwrap_or_default().to_string();
    let artist = tag.get_string(&ItemKey::TrackArtist).unwrap_or_default().to_string();
    let album = tag.get_string(&ItemKey::AlbumTitle).unwrap_or_default().to_string();
    let result = Some((title, artist, album));
    let mut cache = METADATA_CACHE.lock().unwrap();
    cache.insert(key, result.clone());
    result
}
