use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

pub fn read_album_art(path: &Path) -> Option<Vec<u8>> {
    let tagged = Probe::open(path).ok()?.read().ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag())?;
    tag.pictures().first().map(|p| p.data().to_vec())
}

#[derive(Debug, Clone, Default)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub track_number: Option<u32>,
    pub duration: Option<Duration>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
}

impl TrackMetadata {
    pub fn display_title<'a>(&'a self, fallback: &'a str) -> &'a str {
        self.title.as_deref().unwrap_or(fallback)
    }

    pub fn display_artist(&self) -> &str {
        self.artist.as_deref().unwrap_or("Unknown artist")
    }

    pub fn display_album(&self) -> &str {
        self.album.as_deref().unwrap_or("Unknown album")
    }
}

pub fn read_metadata(path: &Path) -> Option<TrackMetadata> {
    let tagged = Probe::open(path).ok()?.read().ok()?;
    let properties = tagged.properties();

    let mut meta = TrackMetadata {
        duration: Some(properties.duration()),
        sample_rate: properties.sample_rate(),
        bit_depth: properties.bit_depth(),
        channels: properties.channels(),
        ..Default::default()
    };

    if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
        meta.title = tag.title().map(|s| s.to_string());
        meta.artist = tag.artist().map(|s| s.to_string());
        meta.album = tag.album().map(|s| s.to_string());
        meta.year = tag.year();
        meta.genre = tag.genre().map(|s| s.to_string());
        meta.track_number = tag.track();
    }

    Some(meta)
}

pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

/// Reads tags for whole batches of tracks on a background thread so grouping
/// and search can use every track's metadata without blocking the UI.
pub struct MetadataLoader {
    tx: Sender<Vec<PathBuf>>,
    rx: Receiver<(PathBuf, Option<TrackMetadata>)>,
    pending: usize,
}

impl MetadataLoader {
    pub fn new() -> Self {
        let (req_tx, req_rx) = channel::<Vec<PathBuf>>();
        let (res_tx, res_rx) = channel();

        thread::spawn(move || {
            while let Ok(batch) = req_rx.recv() {
                for path in batch {
                    let meta = read_metadata(&path);
                    if res_tx.send((path, meta)).is_err() {
                        return;
                    }
                }
            }
        });

        Self {
            tx: req_tx,
            rx: res_rx,
            pending: 0,
        }
    }

    pub fn request(&mut self, paths: Vec<PathBuf>) {
        self.pending += paths.len();
        let _ = self.tx.send(paths);
    }

    /// Everything read since the last call.
    pub fn poll(&mut self) -> Vec<(PathBuf, Option<TrackMetadata>)> {
        let results: Vec<_> = self.rx.try_iter().collect();
        self.pending = self.pending.saturating_sub(results.len());
        results
    }

    /// Tracks still waiting to be read.
    pub fn pending(&self) -> usize {
        self.pending
    }
}
