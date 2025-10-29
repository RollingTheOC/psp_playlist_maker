# Release Notes - v0.1.0

**Release Date**: October 29, 2025

## 🎉 Initial Release

PSP Playlist Maker v0.1.0 is a full-featured GUI application for managing music playlists on PlayStation Portable (PSP). This is the first public release with all core features implemented and tested.

## ✨ Features

### Music Library Management
- **Smart Indexing**: Fast SQLite-based music library indexing
- **Hierarchical Browser**: Three-column interface (Artists → Albums → Songs)
- **Metadata Extraction**: Automatic extraction of ID3 tags using lofty
- **Track Details**: View song information with album art

### Album Art Fetching
- **Multi-Source**: Fetches from 4 different APIs for maximum coverage
  - Deezer API
  - iTunes Store
  - MusicBrainz + Cover Art Archive
  - Last.fm (optional, with API key)
- **Async Loading**: Non-blocking background fetching
- **Smart Caching**: Downloaded art cached for performance

### Playlist Management
- **Create Playlists**: Intuitive playlist creation interface
- **Easy Song Addition**: Click to add songs from the browser
- **PSP-Compatible Export**: M3U8 format with correct relative paths
- **Multiple Playlists**: Manage as many playlists as you need
- **Edit & Delete**: Full playlist management capabilities

### Performance
- **Metadata Caching**: Instant UI with no repeated disk I/O
- **Responsive**: 60fps rendering even with large libraries
- **Memory Efficient**: Fixed memory leak issues from early development
- **Fast Scanning**: Efficient library indexing

## 📦 Downloads

### Linux x86_64
- **File**: `psp_playlist_maker-v0.1.0-linux-x64.tar.gz`
- **Size**: 8.5 MB (compressed), 22 MB (binary)
- **Requirements**: GTK3, OpenSSL

### Windows x86_64
- **Build Instructions**: See `WINDOWS_BUILD.md`
- **GitHub Actions**: Automatic builds on release
- **Requirements**: Windows 10 or later

## 🚀 Quick Start

1. **Download** the release for your platform
2. **Extract** the archive
3. **Index your music**:
   ```bash
   ./psp_playlist_maker scan /path/to/psp/MUSIC
   ```
4. **Launch the GUI**:
   ```bash
   ./psp_playlist_maker gui
   ```
5. **Create playlists** and export to your PSP!

See `QUICKSTART.md` for detailed instructions.

## 🔧 Technical Details

- **Language**: Rust 2021 Edition
- **GUI Framework**: eframe/egui 0.27
- **Database**: SQLite (rusqlite 0.29)
- **Metadata**: lofty 0.22.4
- **HTTP Client**: reqwest 0.11 + tokio 1.36
- **License**: MIT

## 📝 Known Limitations

- Windows builds require manual compilation (or use GitHub Actions)
- Album art fetching requires internet connection
- Last.fm integration requires API key (optional)

## 🐛 Bug Reports

Found a bug? Please open an issue on GitHub with:
- Your OS and version
- Steps to reproduce
- Expected vs actual behavior
- Console output (if applicable)

## 🙏 Acknowledgments

- Album art APIs: Deezer, iTunes, MusicBrainz, Last.fm
- GUI framework: egui by Emil Ernerfeldt
- Metadata extraction: lofty by Serial-ATA

## 📅 What's Next?

Planned for future releases:
- Drag-and-drop song reordering in playlists
- Playlist import functionality
- Batch operations (add album/artist to playlist)
- Search and filter capabilities
- Auto-playlist generation based on criteria
- macOS builds and support

---

**Full Changelog**: Initial release - all features are new!

## Installation Notes

### Linux
```bash
tar -xzf psp_playlist_maker-v0.1.0-linux-x64.tar.gz
cd psp_playlist_maker-linux-x64
./psp_playlist_maker --help
```

### Windows
See `WINDOWS_BUILD.md` for compilation instructions, or wait for automated builds from GitHub Actions.

---

**Thank you for using PSP Playlist Maker!** ❤️ If you find it useful, please star the repo and share it with other PSP enthusiasts!
