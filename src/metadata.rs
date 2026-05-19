use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::Path;
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
