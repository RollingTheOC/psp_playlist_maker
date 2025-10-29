# PSP Playlist Maker

A Rust application for managing music playlists for PlayStation Portable (PSP). Features a GUI for browsing your music library, creating playlists, and exporting them in PSP-compatible M3U8 format.

## Features

### 🎵 Music Library Management
- **Automatic Indexing**: Scans your PSP's MUSIC folder and indexes all audio files
- **Metadata Extraction**: Reads ID3 tags from MP3 files (title, artist, album)
- **SQLite Database**: Stores metadata locally for fast access
- **Metadata Caching**: Smart caching system for optimal performance

### 🖼️ Album Art Fetching
- **Multi-Source Search**: Automatically fetches album art from multiple online sources:
  - Deezer API
  - iTunes API
  - MusicBrainz + Cover Art Archive
  - Last.fm (optional, requires API key)
- **Background Fetching**: Non-blocking album art downloads
- **Smart Caching**: Downloaded artwork is cached for instant display

### 📝 Playlist Creation
- **Visual Interface**: Browse by Artists → Albums → Songs
- **Multiple Playlists**: Create and manage multiple playlists
- **Drag-and-Add**: Easy one-click song addition to playlists
- **PSP-Compatible Export**: Generates M3U8 files with proper PSP paths

### 🎨 User Interface
- **Three-Column Browser**: Miller Columns-style navigation (Artists | Albums | Songs)
- **Track Details Window**: View metadata and album art for selected tracks
- **Responsive Design**: Full-height columns that adapt to window size
- **Performance Optimized**: Smooth scrolling even with large libraries

## Installation

### Prerequisites
- Rust 1.70 or later
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/psp_playlist_maker.git
cd psp_playlist_maker

# Build release version
cargo build --release

# Binary will be in target/release/psp_playlist_maker
```

## Usage

### Command Line Interface

#### Scan Your Music Library
```bash
# Scan PSP's MUSIC folder
./psp_playlist_maker scan /mnt/psp/MUSIC

# Or if mounted at different location
./psp_playlist_maker scan /media/psp/MUSIC
```

#### Launch GUI
```bash
# Start the GUI application
./psp_playlist_maker gui

# Start with debug mode
./psp_playlist_maker gui --debug
```

### Creating Playlists

1. **Index Your Music**: Run the scan command to index your PSP's music files
2. **Launch GUI**: Start the application with `./psp_playlist_maker gui`
3. **Open Playlist Manager**: Click "📝 Manage Playlists" button
4. **Create a Playlist**: Enter a name and click "➕ Create"
5. **Select Playlist**: Click on your playlist to select it
6. **Add Songs**: Browse artists/albums, click "➕" next to songs to add them
7. **Export**: Click "💾 Export" to save as M3U8 file
8. **Copy to PSP**: Place the M3U8 file in your PSP's MUSIC folder

### Playlist File Format

The exported M3U8 files use PSP-compatible paths:

```
#EXTM3U
/MUSIC/Artist Name/Album Name/01 - Song Title.mp3
/MUSIC/Another Artist/Album/Track.mp3
```

## Configuration

### Last.fm API (Optional)
For enhanced album art coverage, set your Last.fm API key:

```bash
export LASTFM_API_KEY=your_api_key_here
```

Get a free API key at: https://www.last.fm/api/account/create

## Project Structure

```
src/
  ├── main.rs          # CLI entry point
  ├── lib.rs           # Library exports
  ├── gui.rs           # GUI application (egui/eframe)
  ├── music.rs         # Music library scanning
  ├── db.rs            # SQLite database operations
  ├── playlist.rs      # M3U8 playlist generation
  ├── metadata.rs      # Metadata extraction
  ├── embedded_art.rs  # ID3 tag reading
  ├── album_art.rs     # Multi-source album art fetching
  └── itunes_art.rs    # iTunes API integration
```

## Dependencies

- **eframe/egui**: GUI framework (0.27+)
- **rusqlite**: SQLite database
- **id3**: ID3 tag reading for MP3 files
- **reqwest**: HTTP client for album art
- **tokio**: Async runtime
- **image**: Image processing
- **serde**: Serialization for API responses
- **rfd**: Native file dialogs

## Performance Features

- Metadata extracted once at startup, cached in memory
- Album art fetched in background threads
- Efficient ScrollArea rendering with unique IDs
- Smart deduplication of API requests
- Progressive loading indicators

## Troubleshooting

### No album art found
- Make sure you have internet connection
- Try different artists/albums (coverage varies)
- Set up Last.fm API key for better results

### Songs not playing on PSP
- Ensure paths start with `/MUSIC/`
- Check that files exist in the correct location
- Verify M3U8 file is in MUSIC folder

### GUI performance issues
- Use release build: `cargo build --release`
- Reduce library size or use selective scanning

## License

MIT License - see LICENSE file for details

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Acknowledgments

- Album art from Deezer, iTunes, MusicBrainz, and Last.fm
- Built with Rust and egui
- Designed for PlayStation Portable (PSP)
