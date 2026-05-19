use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub fn parse_m3u(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tracks = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let raw = PathBuf::from(trimmed);
        let resolved = if raw.is_absolute() {
            raw
        } else {
            base.join(raw)
        };
        if resolved.exists() {
            tracks.push(resolved);
        }
    }

    Ok(tracks)
}

pub fn write_m3u(path: &Path, tracks: &[PathBuf]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(file, "#EXTM3U")?;
    for track in tracks {
        writeln!(file, "{}", track.display())?;
    }
    Ok(())
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("meloic")
}

pub fn default_queue_path() -> PathBuf {
    config_dir().join("queue.m3u")
}

pub fn is_playlist_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()),
        Some(ref ext) if ext == "m3u" || ext == "m3u8"
    )
}
