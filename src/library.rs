use std::path::PathBuf;
use walkdir::WalkDir;

pub fn scan_music(folder: &str) -> Vec<PathBuf> {
    let mut tracks = Vec::new();

    // Klasör mevcut değilse boş liste döndür
    if !std::path::Path::new(folder).exists() {
        return tracks;
    }

    for entry in WalkDir::new(folder) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        match ext_str.to_lowercase().as_str() {
                            "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" => {
                                tracks.push(path.to_path_buf());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    tracks
}
