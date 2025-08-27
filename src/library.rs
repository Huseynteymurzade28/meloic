use walkdir::WalkDir;
use std::path::{ PathBuf };

pub fn scan_music(folder: &str) -> Vec<PathBuf> {
    let mut tracks = Vec::new();

    for entry in WalkDir::new(folder) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "mp3" || ext == "flac" || ext == "wav" {
                    tracks.push(path.to_path_buf());
                }
            }
        }
    }

    tracks
}