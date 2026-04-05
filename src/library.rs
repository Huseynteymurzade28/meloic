use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn scan_music(folder: &str) -> Vec<PathBuf> {
    scan_music_path(Path::new(folder))
}

pub fn default_scan_roots() -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut roots = Vec::new();

    let mut candidates = vec![
        PathBuf::from("/Music"),
        PathBuf::from("Music"),
        PathBuf::from("music"),
        PathBuf::from("."),
    ];

    if let Ok(home) = std::env::var("HOME") {
        candidates.insert(0, PathBuf::from(&home).join("Music"));
        candidates.insert(1, PathBuf::from(home).join("music"));
    }

    for candidate in candidates {
        if !candidate.exists() || !candidate.is_dir() {
            continue;
        }

        let key = candidate
            .canonicalize()
            .unwrap_or_else(|_| candidate.clone())
            .to_string_lossy()
            .to_string();

        if seen.insert(key) {
            roots.push(candidate);
        }
    }

    roots
}

pub fn scan_music_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut tracks = Vec::new();

    for root in roots {
        let root_str = root.to_string_lossy();
        let mut scanned = scan_music(&root_str);
        tracks.append(&mut scanned);
    }

    tracks.sort();
    tracks.dedup();
    tracks
}

fn scan_music_path(folder: &Path) -> Vec<PathBuf> {
    let mut tracks = Vec::new();

    if !folder.exists() || !folder.is_dir() {
        return tracks;
    }

    for entry in WalkDir::new(folder).into_iter().flatten() {
        if entry.file_type().is_file() && is_supported_audio(entry.path()) {
            tracks.push(entry.path().to_path_buf());
        }
    }

    tracks
}

fn is_supported_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac"))
        .unwrap_or(false)
}
