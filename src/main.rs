mod library;
mod ui;

fn main() {
    let tracks = library::scan_music("music");
    if tracks.is_empty() {
        println!("🎵 No music found!");
        std::thread::sleep(std::time::Duration::from_secs(2));
        return;
    }

    // UI'ı çalıştır - artık tek seferlik
    match ui::run_ui(&tracks) {
        Ok(_) => {
            println!("👋 Exiting meloic. Goodbye!");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
