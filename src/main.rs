mod library;
mod ui;

fn main() {
    let scan_roots = library::default_scan_roots();
    let tracks = library::scan_music_roots(&scan_roots);

    if tracks.is_empty() {
        println!("No music found.");
        println!("Checked these folders:");
        for root in &scan_roots {
            println!("  - {}", root.display());
        }
        println!("Try adding tracks to ~/Music, /Music, ./Music, ./music, or the current folder.");
        std::thread::sleep(std::time::Duration::from_secs(3));
        return;
    }

    match ui::run_ui(&tracks, scan_roots) {
        Ok(_) => {
            println!("👋 Exiting meloic. Goodbye!");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
