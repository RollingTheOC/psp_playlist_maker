use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub path: String,
    pub artist: String,
    pub album: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicLibrary {
    pub tracks: Vec<Track>,
}

impl Default for MusicLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicLibrary {
    pub fn new() -> Self {
        MusicLibrary { tracks: Vec::new() }
    }
    /// Recursively scan a music directory and index tracks
    pub fn scan_dir(music_dir: &str) -> Self {
        use walkdir::WalkDir;
        
        use rayon::prelude::*;
        let tracks: Vec<Track> = WalkDir::new(music_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                let path = e.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    matches!(ext.to_lowercase().as_str(), "mp3" | "flac" | "wav" | "m4a")
                } else {
                    false
                }
            })
            .collect::<Vec<_>>()
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                let path_str = path.display().to_string();
                // Try to extract embedded metadata first
                let (title, artist, album) = match crate::embedded_art::extract_metadata(&path_str) {
                    Some((t, a, al)) => {
                        let title = if t.is_empty() { None } else { Some(t) };
                        let artist = if a.is_empty() { None } else { Some(a) };
                        let album = if al.is_empty() { None } else { Some(al) };
                        (title, artist, album)
                    },
                    _ => (None, None, None)
                };
                // Fallback to folder names if metadata missing
                let rel_path = match path.strip_prefix(music_dir) {
                    Ok(p) => p,
                    Err(_) => path,
                };
                let mut components = rel_path.components();
                let fallback_artist = components.next().map(|c| c.as_os_str().to_string_lossy().to_string()).unwrap_or_default();
                let (fallback_album, fallback_title) = match (components.next(), components.next()) {
                    (Some(album), Some(title)) => (album.as_os_str().to_string_lossy().to_string(), title.as_os_str().to_string_lossy().to_string()),
                    (Some(title), _) => (String::new(), title.as_os_str().to_string_lossy().to_string()),
                    (_, _) => (String::new(), path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()),
                };
                Some(Track {
                    path: path_str,
                    artist: artist.unwrap_or(fallback_artist),
                    album: album.unwrap_or(fallback_album),
                    title: title.unwrap_or(fallback_title),
                })
            }).collect();
        MusicLibrary { tracks }
    }
}
