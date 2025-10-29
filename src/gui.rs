use eframe::egui::{self, Context, TextureHandle};
use crate::db;
use crate::music::Track;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

// Cached metadata to avoid repeated disk reads
#[derive(Clone)]
struct CachedMetadata {
    title: String,
    artist: String,
    album: String,
}

// Playlist structure
#[derive(Clone, Debug)]
struct Playlist {
    name: String,
    track_indices: Vec<usize>, // Indices into the main tracks vector
}

pub struct TrackViewerApp {
    tracks: Vec<Track>,
    metadata_cache: Vec<CachedMetadata>, // Same index as tracks
    selected_track: Option<usize>,
    selected_artist_idx: Option<usize>,
    selected_album_idx: Option<usize>,
    hovered_index: Option<usize>,
    music_dir: String,
    is_scanning: bool,
    debug: bool,
    image_cache: HashMap<String, TextureHandle>,
    // Cache album art URLs to avoid repeated API calls
    album_art_cache: HashMap<(String, String), Option<String>>, // (artist, album) -> url
    // Track which song's details window is currently open
    last_details_track: Option<usize>,
    // Track if we're currently fetching album art
    fetching_art_for: HashMap<(String, String), bool>, // (artist, album) -> is_fetching
    // Channel for receiving album art URLs from background threads
    art_receiver: Receiver<((String, String), Option<String>)>,
    art_sender: Sender<((String, String), Option<String>)>,
    // Playlist management
    playlists: Vec<Playlist>,
    show_playlist_manager: bool,
    new_playlist_name: String,
    selected_playlist_idx: Option<usize>,
}

impl TrackViewerApp {
    pub fn with_debug(debug: bool) -> Self {
        let db_path = "music_index.db";
        let tracks = match db::init_db(db_path) {
            Ok(conn) => db::load_tracks(&conn).unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        
        // Build metadata cache once at startup
        let metadata_cache: Vec<CachedMetadata> = tracks.iter()
            .map(|track| {
                let (title, artist, album) = crate::embedded_art::extract_metadata(&track.path)
                    .unwrap_or((track.title.clone(), track.artist.clone(), track.album.clone()));
                CachedMetadata {
                    title: if title.is_empty() { track.title.clone() } else { title },
                    artist: if artist.is_empty() { track.artist.clone() } else { artist },
                    album: if album.is_empty() { track.album.clone() } else { album },
                }
            })
            .collect();
        
        // Create channel for background album art fetching
        let (art_sender, art_receiver) = std::sync::mpsc::channel();
        
        Self {
            tracks,
            metadata_cache,
            selected_track: None,
            selected_artist_idx: None,
            selected_album_idx: None,
            hovered_index: None,
            music_dir: "/mnt/psp/MUSIC".to_string(),
            is_scanning: false,
            debug,
            image_cache: HashMap::new(),
            album_art_cache: HashMap::new(),
            last_details_track: None,
            fetching_art_for: HashMap::new(),
            art_receiver,
            art_sender,
            playlists: Vec::new(),
            show_playlist_manager: false,
            new_playlist_name: String::new(),
            selected_playlist_idx: None,
        }
    }

    fn get_or_load_image(&mut self, ctx: &Context, url: &str) -> Option<TextureHandle> {
        if let Some(tex) = self.image_cache.get(url) {
            return Some(tex.clone());
        }
        if let Ok(response) = reqwest::blocking::get(url) {
            if let Ok(bytes) = response.bytes() {
                if let Ok(image) = image::load_from_memory(&bytes) {
                    let image_buffer = image.to_rgba8();
                    let size = [image_buffer.width() as usize, image_buffer.height() as usize];
                    let pixels = image_buffer.as_flat_samples();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                    let texture = ctx.load_texture(url, color_image, Default::default());
                    self.image_cache.insert(url.to_string(), texture.clone());
                    return Some(texture);
                }
            }
        }
        None
    }

    fn clear_image_cache(&mut self, _ctx: &Context) {
        self.image_cache.clear();
    }

    fn get_artists(&self) -> Vec<String> {
        let mut artists: Vec<String> = self.metadata_cache.iter()
            .map(|meta| meta.artist.clone())
            .collect();
        artists.sort();
        artists.dedup();
        artists
    }

    fn get_albums_for_artist(&self, artist: Option<&String>) -> Vec<String> {
        let mut albums: Vec<String> = self.metadata_cache.iter()
            .filter(|meta| {
                artist.is_none_or(|a| &meta.artist == a)
            })
            .map(|meta| {
                if meta.album.is_empty() { 
                    "(Unknown Album)".to_string() 
                } else { 
                    meta.album.clone() 
                }
            })
            .collect();
        albums.sort();
        albums.dedup();
        albums
    }

    fn count_tracks_for_artist(&self, artist: &str) -> usize {
        self.metadata_cache.iter()
            .filter(|meta| meta.artist == artist)
            .count()
    }

    fn count_tracks_for_album(&self, artist: Option<&String>, album: &str) -> usize {
        self.metadata_cache.iter()
            .filter(|meta| {
                let track_album = if meta.album.is_empty() { 
                    "(Unknown Album)".to_string() 
                } else { 
                    meta.album.clone() 
                };
                let artist_matches = artist.is_none_or(|a| &meta.artist == a);
                artist_matches && track_album == album
            })
            .count()
    }
    
    fn export_playlist(&self, playlist_idx: usize) {
        if let Some(playlist) = self.playlists.get(playlist_idx) {
            // Collect the actual tracks
            let playlist_tracks: Vec<&Track> = playlist.track_indices.iter()
                .filter_map(|idx| self.tracks.get(*idx))
                .collect();
            
            if playlist_tracks.is_empty() {
                eprintln!("[Playlist] Cannot export empty playlist");
                return;
            }
            
            // Use file dialog to choose save location
            let default_name = format!("{}.m3u8", playlist.name);
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("M3U8 Playlist", &["m3u8"])
                .save_file()
            {
                match crate::playlist::write_m3u8(path.to_str().unwrap(), &playlist_tracks) {
                    Ok(_) => {
                        eprintln!("[Playlist] Exported {} tracks to: {:?}", playlist_tracks.len(), path);
                    }
                    Err(e) => {
                        eprintln!("[Playlist] Export failed: {}", e);
                    }
                }
            }
        }
    }
}

impl eframe::App for TrackViewerApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Check for album art results from background threads
            while let Ok((key, url)) = self.art_receiver.try_recv() {
                self.album_art_cache.insert(key.clone(), url.clone());
                self.fetching_art_for.remove(&key);
                if let Some(ref url_str) = url {
                    eprintln!("[GUI] Received album art: {}", url_str);
                }
                ctx.request_repaint(); // Trigger UI update
            }
            
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Indexed Tracks");
                ui.label("Select your PSP's MUSIC folder below before scanning. Example: /mnt/psp/MUSIC");
                ui.horizontal(|ui| {
                    if ui.button("Choose MUSIC folder...").clicked() {
                        if let Some(dir) = rfd::FileDialog::new().set_directory(&self.music_dir).pick_folder() {
                            self.music_dir = dir.display().to_string();
                        }
                    }
                    ui.label(format!("Current folder: {}", self.music_dir));
                    
                    ui.separator();
                    
                    // Playlist Manager button
                    if ui.button("📝 Manage Playlists").clicked() {
                        self.show_playlist_manager = !self.show_playlist_manager;
                    }
                    ui.label(format!("({} playlists)", self.playlists.len()));
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, "To scan or rescan your PSP music library, please use the CLI: psp_playlist_maker scan <music_dir>");
                });
                if self.is_scanning {
                    ui.spinner();
                    ui.label("Scanning, please wait...");
                } else {
                    // Three-column browser view with equal widths and full height
                    let available_height = ui.available_height();
                    let panel_width = ui.available_width() / 3.0 - 8.0; // Subtract for separators
                    
                    ui.horizontal(|ui| {
                        // Left: Artists
                        ui.allocate_ui(egui::vec2(panel_width, available_height), |ui| {
                            ui.vertical(|ui| {
                                let artists = self.get_artists();
                                ui.heading(format!("🎤 Artists ({})", artists.len()));
                                ui.add_space(4.0);
                                
                                let scroll_height = ui.available_height();
                                egui::ScrollArea::vertical()
                                    .id_source("artists_scroll")
                                    .auto_shrink([false, false])
                                    .max_height(scroll_height)
                                    .show(ui, |ui| {
                                        if artists.is_empty() {
                                            ui.colored_label(egui::Color32::GRAY, "No artists found");
                                        }
                                        for (i, artist) in artists.iter().enumerate() {
                                            let track_count = self.count_tracks_for_artist(artist);
                                            let label = format!("{} ({})", artist, track_count);
                                            if ui.selectable_label(self.selected_artist_idx == Some(i), label).clicked() {
                                                self.selected_artist_idx = Some(i);
                                                self.selected_album_idx = None;
                                                self.selected_track = None;
                                            }
                                        }
                                    });
                            });
                        });
                        
                        ui.separator();
                        
                        // Middle: Albums for selected artist
                        ui.allocate_ui(egui::vec2(panel_width, available_height), |ui| {
                            ui.vertical(|ui| {
                                let artists = self.get_artists();
                                let selected_artist = self.selected_artist_idx.and_then(|i| artists.get(i));
                                let albums = self.get_albums_for_artist(selected_artist);
                                
                                let title = if let Some(artist) = selected_artist {
                                    format!("💿 Albums by {} ({})", artist, albums.len())
                                } else {
                                    format!("💿 Albums ({})", albums.len())
                                };
                                ui.heading(title);
                                ui.add_space(4.0);
                                
                                let scroll_height = ui.available_height();
                                egui::ScrollArea::vertical()
                                    .id_source("albums_scroll")
                                    .auto_shrink([false, false])
                                    .max_height(scroll_height)
                                    .show(ui, |ui| {
                                        if albums.is_empty() {
                                            ui.colored_label(egui::Color32::GRAY, "← Select an artist");
                                        }
                                        for (i, album) in albums.iter().enumerate() {
                                            let track_count = self.count_tracks_for_album(selected_artist, album);
                                            let label = format!("{} ({})", album, track_count);
                                            if ui.selectable_label(self.selected_album_idx == Some(i), label).clicked() {
                                                self.selected_album_idx = Some(i);
                                                self.selected_track = None;
                                            }
                                        }
                                    });
                            });
                        });
                        
                        ui.separator();
                        
                        // Right: Songs for selected album (only show if album is selected)
                        ui.allocate_ui(egui::vec2(panel_width, available_height), |ui| {
                            ui.vertical(|ui| {
                                let artists = self.get_artists();
                                let selected_artist = self.selected_artist_idx.and_then(|i| artists.get(i));
                                let albums = self.get_albums_for_artist(selected_artist);
                                let selected_album = self.selected_album_idx.and_then(|i| albums.get(i));
                                
                                // Only compute filtered tracks if an album is selected
                                if let Some(album) = selected_album {
                                    let filtered_tracks: Vec<(usize, &Track, &CachedMetadata)> = self.tracks.iter()
                                        .zip(self.metadata_cache.iter())
                                        .enumerate()
                                        .filter(|(_, (_, meta))| {
                                            let track_album = if meta.album.is_empty() { 
                                                "(Unknown Album)".to_string() 
                                            } else { 
                                                meta.album.clone() 
                                            };
                                            let artist_matches = selected_artist.is_none_or(|a| &meta.artist == a);
                                            let album_matches = &track_album == album;
                                            artist_matches && album_matches
                                        })
                                        .map(|(idx, (track, meta))| (idx, track, meta))
                                        .collect();
                                    
                                    let title = if let Some(artist) = selected_artist {
                                        format!("🎵 {} - {} ({})", artist, album, filtered_tracks.len())
                                    } else {
                                        format!("🎵 {} ({})", album, filtered_tracks.len())
                                    };
                                    ui.heading(title);
                                    ui.add_space(4.0);
                                    
                                    let scroll_height = ui.available_height();
                                    egui::ScrollArea::vertical()
                                        .id_source("songs_scroll")
                                        .auto_shrink([false, false])
                                        .max_height(scroll_height)
                                        .show(ui, |ui| {
                                            for (orig_idx, _track, meta) in filtered_tracks {
                                                ui.horizontal(|ui| {
                                                    let display = if !meta.title.is_empty() {
                                                        meta.title.clone()
                                                    } else {
                                                        "(Untitled)".to_string()
                                                    };
                                                    
                                                    if ui.selectable_label(self.selected_track == Some(orig_idx), display).clicked() {
                                                        self.selected_track = Some(orig_idx);
                                                    }
                                                    
                                                    // Add to playlist button
                                                    if !self.playlists.is_empty() && ui.small_button("➕").clicked() {
                                                        // Add to selected playlist or show menu
                                                        if let Some(pl_idx) = self.selected_playlist_idx {
                                                            if let Some(playlist) = self.playlists.get_mut(pl_idx) {
                                                                if !playlist.track_indices.contains(&orig_idx) {
                                                                    playlist.track_indices.push(orig_idx);
                                                                    eprintln!("[Playlist] Added track to '{}'", playlist.name);
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                        });
                                } else {
                                    // Show hint when no album is selected
                                    ui.heading("🎵 Songs");
                                    ui.add_space(4.0);
                                    ui.colored_label(egui::Color32::GRAY, "← Select an album to view songs");
                                }
                            });
                        });
                    }); // end three-column horizontal
                    // Details pane for selected track as a pop-out window, with embedded metadata and album art
                    if let Some(idx) = self.selected_track {
                        // Prepare data before the Window closure to avoid borrowing issues
                        let meta = self.metadata_cache.get(idx).cloned();
                        
                        // Only fetch album art if we switched to a different track
                        if self.last_details_track != Some(idx) {
                            self.last_details_track = Some(idx);
                            
                            if let Some(meta_ref) = &meta {
                                if !meta_ref.artist.is_empty() && !meta_ref.album.is_empty() {
                                    let key = (meta_ref.artist.clone(), meta_ref.album.clone());
                                    
                                    // Check if we already have it cached or are fetching it
                                    if !self.album_art_cache.contains_key(&key) && !self.fetching_art_for.contains_key(&key) {
                                        // Mark as fetching
                                        self.fetching_art_for.insert(key.clone(), true);
                                        
                                        eprintln!("[GUI] Starting background fetch for: {} - {}", meta_ref.artist, meta_ref.album);
                                        
                                        // Spawn background thread to fetch album art
                                        let sender = self.art_sender.clone();
                                        let artist = meta_ref.artist.clone();
                                        let album = meta_ref.album.clone();
                                        
                                        std::thread::spawn(move || {
                                            let rt = tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build()
                                                .unwrap();
                                            
                                            let result = rt.block_on(crate::album_art::fetch_album_art(&artist, &album));
                                            let _ = sender.send((key, result));
                                        });
                                        
                                        // Request repaint so UI shows loading state
                                        ctx.request_repaint();
                                    }
                                }
                            }
                        }
                        
                        // Check if we're currently fetching for this album
                        let is_fetching = meta.as_ref().map(|m| {
                            let key = (m.artist.clone(), m.album.clone());
                            self.fetching_art_for.contains_key(&key)
                        }).unwrap_or(false);
                        
                        // Get the album art URL from cache
                        let album_art_url = meta.as_ref().and_then(|m| {
                            let key = (m.artist.clone(), m.album.clone());
                            self.album_art_cache.get(&key).and_then(|opt| opt.clone())
                        });
                        
                        egui::Window::new("Track Details").open(&mut true).show(ctx, |ui| {
                            if let Some(meta) = meta {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.heading("Track Details");
                                        ui.label(format!("Title: {}", meta.title));
                                        ui.label(format!("Artist: {}", meta.artist));
                                        ui.label(format!("Album: {}", meta.album));
                                    });
                                    
                                    // Display album art if available
                                    if is_fetching {
                                        ui.vertical(|ui| {
                                            ui.spinner();
                                            ui.label("Searching for album art...");
                                        });
                                    } else if let Some(url) = album_art_url {
                                        if let Some(tex) = self.get_or_load_image(ctx, &url) {
                                            let available = ui.available_size();
                                            let max_dim = 256.0_f32.min(available.x.min(available.y));
                                            let size = tex.size_vec2();
                                            let aspect = size.x / size.y;
                                            let (w, h) = if aspect > 1.0 {
                                                (max_dim, max_dim / aspect)
                                            } else {
                                                (max_dim * aspect, max_dim)
                                            };
                                            ui.image((tex.id(), egui::vec2(w, h)));
                                        } else {
                                            ui.label("Loading image...");
                                        }
                                    } else {
                                        ui.label("No album art found.");
                                    }
                                });
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Clear Image Cache").clicked() {
                            self.clear_image_cache(ctx);
                        }
                    });
                    // Debug window: show raw music_index contents and highlight hovered track
                    if self.debug {
                        egui::Window::new("music_index.db contents").show(ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, track) in self.tracks.iter().enumerate() {
                                    let album_name = if track.album.is_empty() { "(no album)" } else { &track.album };
                                    let text = format!(
                                        "{} - {} [{}] | Track #: N/A | path: {}",
                                        track.artist, track.title, album_name, track.path
                                    );
                                    if Some(i) == self.hovered_index {
                                        ui.colored_label(egui::Color32::YELLOW, text);
                                    } else {
                                        ui.label(text);
                                    }
                                }
                            });
                        });
                    }
                    
                    // Playlist Manager Window
                    if self.show_playlist_manager {
                        egui::Window::new("📝 Playlist Manager")
                            .default_width(500.0)
                            .show(ctx, |ui| {
                                ui.heading("Your Playlists");
                                ui.separator();
                                
                                // Create new playlist section
                                ui.horizontal(|ui| {
                                    ui.label("New playlist:");
                                    ui.text_edit_singleline(&mut self.new_playlist_name);
                                    if ui.button("➕ Create").clicked() && !self.new_playlist_name.trim().is_empty() {
                                        self.playlists.push(Playlist {
                                            name: self.new_playlist_name.trim().to_string(),
                                            track_indices: Vec::new(),
                                        });
                                        self.new_playlist_name.clear();
                                    }
                                });
                                
                                ui.separator();
                                ui.add_space(5.0);
                                
                                // List existing playlists
                                if self.playlists.is_empty() {
                                    ui.colored_label(egui::Color32::GRAY, "No playlists yet. Create one above!");
                                } else {
                                    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                                        let mut to_remove = None;
                                        let mut to_export = None;
                                        
                                        for (idx, playlist) in self.playlists.iter().enumerate() {
                                            let is_selected = self.selected_playlist_idx == Some(idx);
                                            
                                            ui.horizontal(|ui| {
                                                if ui.selectable_label(is_selected, format!("📋 {}", playlist.name)).clicked() {
                                                    self.selected_playlist_idx = Some(idx);
                                                }
                                                
                                                ui.label(format!("({} tracks)", playlist.track_indices.len()));
                                                
                                                if ui.button("💾 Export").clicked() {
                                                    to_export = Some(idx);
                                                }
                                                
                                                if ui.button("🗑️").clicked() {
                                                    to_remove = Some(idx);
                                                }
                                            });
                                            
                                            // Show tracks in selected playlist
                                            if is_selected {
                                                ui.indent(idx, |ui| {
                                                    if playlist.track_indices.is_empty() {
                                                        ui.colored_label(egui::Color32::GRAY, "  (empty playlist)");
                                                    } else {
                                                        for track_idx in &playlist.track_indices {
                                                            if let Some(_track) = self.tracks.get(*track_idx) {
                                                                let meta = &self.metadata_cache[*track_idx];
                                                                ui.label(format!("  ♪ {} - {}", meta.artist, meta.title));
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        
                                        // Handle removals
                                        if let Some(idx) = to_remove {
                                            self.playlists.remove(idx);
                                            if self.selected_playlist_idx == Some(idx) {
                                                self.selected_playlist_idx = None;
                                            }
                                        }
                                        
                                        // Handle exports
                                        if let Some(idx) = to_export {
                                            self.export_playlist(idx);
                                        }
                                    });
                                }
                            });
                    }
                } // end else (is_scanning)
            }); // end CentralPanel
        }
    }