# Quick Start Guide

## First Time Setup

1. **Mount your PSP** (via USB)
   ```bash
   # Your PSP should appear as /media/psp or /mnt/psp
   # Verify the MUSIC folder exists
   ls /mnt/psp/MUSIC
   ```

2. **Index your music library**
   ```bash
   ./psp_playlist_maker scan /mnt/psp/MUSIC
   ```
   This creates a `music_index.db` file with all your tracks.

3. **Launch the GUI**
   ```bash
   ./psp_playlist_maker gui
   ```

## Creating Your First Playlist

1. Click **"📝 Manage Playlists"** in the top toolbar
2. Type a playlist name (e.g., "Road Trip Mix")
3. Click **"➕ Create"**
4. Click on your new playlist to select it (it will highlight)
5. Browse through Artists → Albums → Songs
6. Click **"➕"** next to any song to add it
7. When finished, click **"💾 Export"** next to your playlist
8. Save the `.m3u8` file to your PSP's MUSIC folder

## Tips

- **Album Art**: Requires internet connection. Fetched automatically when viewing track details.
- **Multiple Playlists**: Create different playlists for different moods!
- **Fast Browsing**: Click albums directly - only songs load when album is selected for better performance
- **Re-scan**: Run the scan command again if you add new music to your PSP

## Troubleshooting

**Can't see any music?**
- Make sure you ran the `scan` command first
- Check that music files are in `/MUSIC/<artist>/<album>/` structure

**Playlist doesn't work on PSP?**
- Ensure the `.m3u8` file is in the MUSIC folder (not in a subfolder)
- Check that song paths in the playlist match your actual file structure

**GUI won't start?**
- Make sure you built with `--release` for best performance
- Check you have display/graphics drivers installed

## Advanced

### Debug Mode
```bash
./psp_playlist_maker gui --debug
```
Shows additional information and raw database contents.

### Re-indexing
Just run the scan command again - it will rebuild the database:
```bash
./psp_playlist_maker scan /mnt/psp/MUSIC
```

### Last.fm Integration
For better album art coverage:
```bash
export LASTFM_API_KEY=your_key_here
./psp_playlist_maker gui
```

Get a free API key at: https://www.last.fm/api/account/create
