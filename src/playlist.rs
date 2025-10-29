use crate::music::Track;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Convert a full file path to PSP-relative format
/// Example: /mnt/psp/MUSIC/Album/song.mp3 -> /MUSIC/Album/song.mp3
fn to_psp_path(full_path: &str) -> String {
    let path = Path::new(full_path);
    
    // Find the MUSIC directory in the path
    let components: Vec<_> = path.components().collect();
    if let Some(music_idx) = components.iter().position(|c| {
        c.as_os_str().to_string_lossy().to_uppercase() == "MUSIC"
    }) {
        // Build path from MUSIC onwards
        let psp_components: Vec<_> = components[music_idx..].iter()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        
        // PSP expects normal paths with spaces and special characters as-is
        format!("/{}", psp_components.join("/"))
    } else {
        // Fallback: just use the filename with /MUSIC/ prefix
        let filename = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        format!("/MUSIC/{}", filename)
    }
}

pub fn write_m3u8(playlist_path: &str, tracks: &[&Track]) -> io::Result<()> {
    let mut file = File::create(playlist_path)?;
    writeln!(file, "#EXTM3U")?;
    
    for track in tracks {
        let psp_path = to_psp_path(&track.path);
        writeln!(file, "{}", psp_path)?;
    }
    
    Ok(())
}
