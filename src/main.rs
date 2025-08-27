mod library;
mod player;
mod ui;

fn main() {
    let tracks = library::scan_music("music");
    if tracks.is_empty() {
        println!("🎵 No music found!");
        return;
    }

    match ui::run_ui(&tracks) {
        Ok(Some(selected_track)) => {
            println!("Selected track: {}", selected_track.display());
            player::play_file(&selected_track);
        }
        Ok(None) => {
            println!("No track selected.");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
