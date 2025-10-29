use psp_playlist_maker::db;
use psp_playlist_maker::music;
use psp_playlist_maker::gui;

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("PSP Playlist Maker CLI");
        println!("Usage: psp_playlist_maker <command> [options]\nCommands:\n  scan <music_dir>   Index music files\n  gui               View indexed tracks in GUI\n  help              Show this message");
        return;
    }
    match args[1].as_str() {
        "scan" => {
            let music_dir = if args.len() > 2 { &args[2] } else { "/MUSIC" };
            println!("Scanning music directory: {}", music_dir);
            let library = music::MusicLibrary::scan_dir(music_dir);
            println!("Indexed {} tracks.", library.tracks.len());
            let db_path = "music_index.db";
            match db::init_db(db_path) {
                Ok(mut conn) => {
                    match db::save_library(&mut conn, &library) {
                        Ok(_) => println!("Library saved to {}.", db_path),
                        Err(e) => eprintln!("Failed to save library: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to initialize database: {}", e),
            }
        }
        "gui" => {
            let options = eframe::NativeOptions::default();
            let debug = args.iter().any(|a| a == "--debug");
            let _ = eframe::run_native(
                "PSP Playlist Track Viewer",
                options,
                Box::new(move |_cc| Box::new(gui::TrackViewerApp::with_debug(debug))),
            );
        }
        "help" => {
            println!("Usage: psp_playlist_maker <command> [options]\nCommands:\n  scan <music_dir>   Index music files\n  gui               View indexed tracks in GUI\n  help              Show this message");
        }
        _ => {
            println!("Unknown command. Use 'help' for usage.");
        }
    }
}
