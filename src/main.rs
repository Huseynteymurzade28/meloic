mod library;
mod ui;

fn main() {
    // Önce "music" klasörünü tara, bulamazsa mevcut dizini tara
    let mut tracks = library::scan_music("music");

    if tracks.is_empty() {
        // "music" klasöründe dosya yoksa mevcut dizini tara
        tracks = library::scan_music(".");
    }

    if tracks.is_empty() {
        println!("🎵 No music found!");
        println!("💡 Try:");
        println!("   - Put music files in a 'music' folder");
        println!("   - Or run meloic from a folder containing music files");
        std::thread::sleep(std::time::Duration::from_secs(3));
        return;
    }

    // UI'ı çalıştır
    match ui::run_ui(&tracks) {
        Ok(_) => {
            println!("👋 Exiting meloic. Goodbye!");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
