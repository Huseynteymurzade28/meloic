mod library;
mod metadata;
mod playlist;
mod ui;
mod waveform;

use std::path::PathBuf;

fn main() {
    let arg = std::env::args().nth(1).map(PathBuf::from);

    // `meloic <folder>` scans only that folder instead of the default roots.
    let scan_roots = match &arg {
        Some(dir) if dir.is_dir() => vec![dir.clone()],
        _ => library::default_scan_roots(),
    };
    let tracks = library::scan_music_roots(&scan_roots);

    if tracks.is_empty() {
        println!("No music found.");
        println!("Checked these folders:");
        for root in &scan_roots {
            println!("  - {}", root.display());
        }
        println!("Try adding tracks to ~/Music, /Music, ./Music, ./music, or the current folder,");
        println!("or pass a folder to scan: meloic ~/path/to/music");
        std::thread::sleep(std::time::Duration::from_secs(3));
        return;
    }

    let initial_queue = arg
        .filter(|p| playlist::is_playlist_file(p))
        .and_then(|p| playlist::parse_m3u(&p).ok())
        .unwrap_or_default();

    match ui::run_ui(&tracks, scan_roots, initial_queue) {
        Ok(_) => {
            println!("👋 Exiting meloic. Goodbye!");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
