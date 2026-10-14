//! User preferences persisted between sessions in `~/.config/meloic/settings`.
//!
//! The file is a plain `key = value` list so it can be edited by hand.
//! Unknown keys and invalid values are ignored and fall back to defaults,
//! so a damaged file never prevents meloic from starting.

use crate::playlist::config_dir;
use crate::ui::app::{RepeatMode, SortMode, ThemeMode};
use std::path::{Path, PathBuf};

pub const DEFAULT_VOLUME: f32 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    pub theme: ThemeMode,
    pub volume: f32,
    pub muted: bool,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub sort: SortMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Catppuccin,
            volume: DEFAULT_VOLUME,
            muted: false,
            shuffle: false,
            repeat: RepeatMode::Off,
            sort: SortMode::NameAsc,
        }
    }
}

fn settings_path() -> PathBuf {
    config_dir().join("settings")
}

/// Theme file written by meloic 0.4 and earlier, read once for migration.
fn legacy_theme_path() -> PathBuf {
    config_dir().join("theme")
}

impl Settings {
    pub fn load() -> Self {
        Self::load_from(&settings_path(), &legacy_theme_path())
    }

    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, self.to_text());
    }

    fn load_from(path: &Path, legacy_theme: &Path) -> Self {
        if let Ok(text) = std::fs::read_to_string(path) {
            return Self::parse(&text);
        }
        let theme = std::fs::read_to_string(legacy_theme)
            .ok()
            .and_then(|s| ThemeMode::from_label(s.trim()));
        Self {
            theme: theme.unwrap_or(ThemeMode::Catppuccin),
            ..Self::default()
        }
    }

    fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "theme" => {
                    if let Some(theme) = ThemeMode::from_label(value) {
                        settings.theme = theme;
                    }
                }
                "volume" => {
                    if let Ok(percent) = value.parse::<u32>() {
                        settings.volume = percent.min(100) as f32 / 100.0;
                    }
                }
                "muted" => settings.muted = parse_bool(value).unwrap_or(settings.muted),
                "shuffle" => settings.shuffle = parse_bool(value).unwrap_or(settings.shuffle),
                "repeat" => {
                    if let Some(repeat) = RepeatMode::from_label(value) {
                        settings.repeat = repeat;
                    }
                }
                "sort" => {
                    if let Some(sort) = SortMode::from_key(value) {
                        settings.sort = sort;
                    }
                }
                _ => {}
            }
        }
        settings
    }

    fn to_text(self) -> String {
        format!(
            "# meloic settings, rewritten whenever a preference changes\n\
             theme = {}\n\
             volume = {}\n\
             muted = {}\n\
             shuffle = {}\n\
             repeat = {}\n\
             sort = {}\n",
            self.theme.label(),
            (self.volume * 100.0).round() as u32,
            self.muted,
            self.shuffle,
            self.repeat.label(),
            self.sort.key(),
        )
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "on" | "yes" | "1" => Some(true),
        "false" | "off" | "no" | "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_field() {
        let settings = Settings {
            theme: ThemeMode::Gruvbox,
            volume: 0.35,
            muted: true,
            shuffle: true,
            repeat: RepeatMode::One,
            sort: SortMode::Folder,
        };
        assert_eq!(Settings::parse(&settings.to_text()), settings);
    }

    #[test]
    fn ignores_unknown_keys_and_invalid_values() {
        let parsed = Settings::parse(
            "theme = not-a-theme\nvolume = loud\nshuffle = maybe\nfoo = bar\nrepeat = all\n",
        );
        assert_eq!(
            parsed,
            Settings {
                repeat: RepeatMode::All,
                ..Settings::default()
            }
        );
    }

    #[test]
    fn clamps_volume_to_100_percent() {
        assert_eq!(Settings::parse("volume = 250").volume, 1.0);
    }

    #[test]
    fn migrates_legacy_theme_file() {
        let dir = std::env::temp_dir().join(format!("meloic-settings-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let legacy = dir.join("theme");
        std::fs::write(&legacy, "nord\n").unwrap();

        let loaded = Settings::load_from(&dir.join("settings"), &legacy);
        assert_eq!(loaded.theme, ThemeMode::Nord);
        assert_eq!(loaded.volume, DEFAULT_VOLUME);

        std::fs::remove_dir_all(dir).unwrap();
    }
}
