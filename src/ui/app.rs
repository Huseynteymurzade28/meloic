use rand::seq::SliceRandom;
use ratatui::widgets::ListState;
use ratatui::style::Color;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::sync::Arc;

use crate::library;

/// Müzik çalma durumu
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::All,
            Self::All => Self::One,
            Self::One => Self::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::All => "all",
            Self::One => "one",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    NameAsc,
    NameDesc,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Neon,
    Sunset,
    Forest,
    Mono,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::Neon => Self::Sunset,
            Self::Sunset => Self::Forest,
            Self::Forest => Self::Mono,
            Self::Mono => Self::Neon,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Neon => "neon",
            Self::Sunset => "sunset",
            Self::Forest => "forest",
            Self::Mono => "mono",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub accent: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub text: Color,
    pub muted: Color,
    pub selection_bg: Color,
}

impl ThemeMode {
    pub fn palette(self) -> ThemePalette {
        match self {
            Self::Neon => ThemePalette {
                accent: Color::Rgb(255, 120, 198),
                secondary: Color::Rgb(120, 220, 255),
                success: Color::Rgb(140, 255, 180),
                warning: Color::Rgb(255, 210, 120),
                text: Color::Rgb(230, 235, 245),
                muted: Color::Rgb(120, 130, 150),
                selection_bg: Color::Rgb(62, 76, 104),
            },
            Self::Sunset => ThemePalette {
                accent: Color::Rgb(255, 140, 90),
                secondary: Color::Rgb(255, 208, 122),
                success: Color::Rgb(255, 178, 120),
                warning: Color::Rgb(255, 230, 160),
                text: Color::Rgb(250, 238, 225),
                muted: Color::Rgb(165, 134, 120),
                selection_bg: Color::Rgb(96, 61, 54),
            },
            Self::Forest => ThemePalette {
                accent: Color::Rgb(98, 196, 146),
                secondary: Color::Rgb(151, 223, 185),
                success: Color::Rgb(190, 255, 180),
                warning: Color::Rgb(224, 220, 142),
                text: Color::Rgb(226, 239, 228),
                muted: Color::Rgb(108, 136, 118),
                selection_bg: Color::Rgb(43, 69, 57),
            },
            Self::Mono => ThemePalette {
                accent: Color::Rgb(220, 220, 220),
                secondary: Color::Rgb(180, 180, 180),
                success: Color::Rgb(245, 245, 245),
                warning: Color::Rgb(210, 210, 210),
                text: Color::Rgb(235, 235, 235),
                muted: Color::Rgb(140, 140, 140),
                selection_bg: Color::Rgb(70, 70, 70),
            },
        }
    }
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            Self::NameAsc => Self::NameDesc,
            Self::NameDesc => Self::Folder,
            Self::Folder => Self::NameAsc,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NameAsc => "name a-z",
            Self::NameDesc => "name z-a",
            Self::Folder => "folder",
        }
    }
}

/// Uygulama durumunu tutan ana struct
pub struct App {
    pub items: Vec<PathBuf>,
    pub state: ListState,
    pub show_help: bool,
    pub filtered_items: Vec<(usize, PathBuf)>, // (original_index, path)
    pub total_tracks: usize,
    pub current_view: ViewMode,
    pub playback_state: PlaybackState,
    pub current_track: Option<PathBuf>,
    pub sink: Option<Arc<Sink>>,
    pub _stream: Option<OutputStream>, // _ ile başlıyor çünkü kullanmıyoruz ama yaşatmamız gerekiyor
    pub stream_handle: Option<OutputStreamHandle>,
    pub scan_roots: Vec<PathBuf>,
    pub search_query: String,
    pub is_searching: bool,
    pub repeat_mode: RepeatMode,
    pub sort_mode: SortMode,
    pub theme_mode: ThemeMode,
    pub status_message: String,
    pub last_error: Option<String>,
    pub waveform_hidden_tracks: HashSet<PathBuf>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Normal,
    Help,
}

impl App {
    /// Yeni bir App instance'ı oluştur
    pub fn new(items: Vec<PathBuf>, scan_roots: Vec<PathBuf>) -> App {
        let total = items.len();
        let mut app = App {
            items: items.clone(),
            state: ListState::default(),
            show_help: false,
            filtered_items: items.into_iter().enumerate().collect(),
            total_tracks: total,
            current_view: ViewMode::Normal,
            playback_state: PlaybackState::Stopped,
            current_track: None,
            sink: None,
            _stream: None,
            stream_handle: None,
            scan_roots,
            search_query: String::new(),
            is_searching: false,
            repeat_mode: RepeatMode::Off,
            sort_mode: SortMode::NameAsc,
            theme_mode: ThemeMode::Neon,
            status_message: format!("Loaded {total} tracks"),
            last_error: None,
            waveform_hidden_tracks: HashSet::new(),
        };
        app.update_filtered_items();
        app
    }

    /// İlk öğeyi seç
    pub fn select_first(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Sonraki öğeye geç (liste sonunda ise başa sar)
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Önceki öğeye geç (liste başında ise sona sar)
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Sayfa sonraki (10 öğe)
    pub fn next_page(&mut self) {
        if let Some(selected) = self.state.selected() {
            let new_index =
                std::cmp::min(selected + 10, self.filtered_items.len().saturating_sub(1));
            self.state.select(Some(new_index));
        }
    }

    /// Sayfa önceki (10 öğe)
    pub fn previous_page(&mut self) {
        if let Some(selected) = self.state.selected() {
            let new_index = selected.saturating_sub(10);
            self.state.select(Some(new_index));
        }
    }

    /// Liste başına git
    pub fn go_to_top(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Liste sonuna git
    pub fn go_to_bottom(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(self.filtered_items.len() - 1));
        }
    }

    /// Rastgele şarkı seç
    pub fn select_random(&mut self) {
        if !self.filtered_items.is_empty() {
            let mut rng = rand::thread_rng();
            let random_index = (0..self.filtered_items.len())
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap_or(0);
            self.state.select(Some(random_index));
            self.status_message = "Selected random track".to_string();
        }
    }

    /// Yardım penceresini aç/kapat
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.current_view = if self.show_help {
            ViewMode::Help
        } else {
            ViewMode::Normal
        };
    }

    /// Filtrelenmiş öğeleri güncelle (şu an sadece tüm öğeleri gösteriyor)
    fn update_filtered_items(&mut self) {
        let previous_selection = self.get_selected();
        let query = self.search_query.to_lowercase();

        self.filtered_items = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, path)| {
                if query.is_empty() {
                    return true;
                }

                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_lowercase().contains(&query))
                    .unwrap_or(false)
                    || path
                        .parent()
                        .and_then(|parent| parent.to_str())
                        .map(|parent| parent.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .map(|(i, path)| (i, path.clone()))
            .collect();

        self.filtered_items.sort_by(|a, b| {
            let a_name = display_name(&a.1).to_lowercase();
            let b_name = display_name(&b.1).to_lowercase();
            let a_folder = folder_name(&a.1).to_lowercase();
            let b_folder = folder_name(&b.1).to_lowercase();

            match self.sort_mode {
                SortMode::NameAsc => a_name.cmp(&b_name),
                SortMode::NameDesc => b_name.cmp(&a_name),
                SortMode::Folder => a_folder.cmp(&b_folder).then(a_name.cmp(&b_name)),
            }
        });

        if self.filtered_items.is_empty() {
            self.state.select(None);
            return;
        }

        if let Some(previous_selection) = previous_selection {
            if let Some(index) = self
                .filtered_items
                .iter()
                .position(|(_, path)| path == &previous_selection)
            {
                self.state.select(Some(index));
                return;
            }
        }

        self.state.select(Some(0));
    }

    /// Kütüphaneyi yenile
    pub fn refresh(&mut self) {
        self.items = library::scan_music_roots(&self.scan_roots);
        self.total_tracks = self.items.len();
        self.update_filtered_items();
        self.status_message = format!("Refreshed library: {} tracks", self.total_tracks);
    }

    /// Seçili öğeyi döndür
    pub fn get_selected(&self) -> Option<PathBuf> {
        self.state
            .selected()
            .and_then(|i| self.filtered_items.get(i))
            .map(|(_, path)| path.clone())
    }

    /// Liste boş mu?
    pub fn is_empty(&self) -> bool {
        self.filtered_items.is_empty()
    }

    /// Mevcut görünüm listesi
    pub fn get_display_items(&self) -> &Vec<(usize, PathBuf)> {
        &self.filtered_items
    }

    pub fn current_track_index(&self) -> Option<usize> {
        self.current_track.as_ref().and_then(|current| {
            self.filtered_items
                .iter()
                .position(|(_, path)| path == current)
        })
    }

    pub fn start_search(&mut self) {
        self.is_searching = true;
        self.status_message = "Search mode: type to filter, Enter to keep, Esc to clear"
            .to_string();
    }

    pub fn append_search_char(&mut self, ch: char) {
        self.search_query.push(ch);
        self.update_filtered_items();
        self.status_message = format!("Filter: {} matches", self.filtered_items.len());
    }

    pub fn backspace_search(&mut self) {
        self.search_query.pop();
        self.update_filtered_items();
        self.status_message = format!("Filter: {} matches", self.filtered_items.len());
    }

    pub fn confirm_search(&mut self) {
        self.is_searching = false;
        self.status_message = if self.search_query.is_empty() {
            format!("Showing all {} tracks", self.filtered_items.len())
        } else {
            format!("Search locked: {} matches", self.filtered_items.len())
        };
    }

    pub fn cancel_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        self.update_filtered_items();
        self.status_message = format!("Cleared filter: {} tracks", self.filtered_items.len());
    }

    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.update_filtered_items();
        self.status_message = format!("Sort: {}", self.sort_mode.label());
    }

    pub fn cycle_repeat_mode(&mut self) {
        self.repeat_mode = self.repeat_mode.next();
        self.status_message = format!("Repeat: {}", self.repeat_mode.label());
    }

    pub fn cycle_theme_mode(&mut self) {
        self.theme_mode = self.theme_mode.next();
        self.status_message = format!("Theme: {}", self.theme_mode.label());
    }

    pub fn is_waveform_enabled_for(&self, track: &Path) -> bool {
        !self.waveform_hidden_tracks.contains(track)
    }

    pub fn toggle_selected_waveform(&mut self) {
        let Some(selected) = self.get_selected() else {
            self.status_message = "No track selected".to_string();
            return;
        };

        if self.waveform_hidden_tracks.remove(&selected) {
            self.status_message = "Waveform enabled for selected track".to_string();
        } else {
            self.waveform_hidden_tracks.insert(selected);
            self.status_message = "Waveform hidden for selected track".to_string();
        }
    }

    pub fn waveform_points(&self, track: &Path, samples: usize) -> Vec<(f64, f64)> {
        if samples == 0 {
            return Vec::new();
        }

        let seed = track
            .to_string_lossy()
            .bytes()
            .fold(0_u64, |acc, byte| acc.wrapping_mul(131).wrapping_add(byte as u64 + 17));
        let phase = (seed % 360) as f64 / 57.2958;
        let pulse = ((seed >> 7) % 11) as f64 + 3.0;
        let shimmer = ((seed >> 13) % 17) as f64 + 5.0;

        (0..samples)
            .map(|index| {
                let x = index as f64;
                let base = x / samples as f64;
                let y = ((base * pulse * std::f64::consts::TAU + phase).sin() * 0.72)
                    + ((base * shimmer * std::f64::consts::PI + phase * 0.5).cos() * 0.28)
                    + ((base * 3.0 + phase * 0.2).sin() * 0.12);
                (x, y.clamp(-1.0, 1.0))
            })
            .collect()
    }

    pub fn scan_roots_summary(&self) -> String {
        self.scan_roots
            .iter()
            .map(|root| root.display().to_string())
            .collect::<Vec<_>>()
            .join("  |  ")
    }

    pub fn selected_track_number(&self) -> usize {
        self.state.selected().map(|i| i + 1).unwrap_or(0)
    }

    pub fn visible_track_count(&self) -> usize {
        self.filtered_items.len()
    }

    pub fn set_error<E: std::fmt::Display>(&mut self, error: E) {
        let message = error.to_string();
        self.status_message = format!("Error: {message}");
        self.last_error = Some(message);
    }

    /// Müzik çalmayı başlat
    pub fn play_track(&mut self, track: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        use rodio::{Decoder, OutputStream, Sink};
        use std::fs::File;
        use std::io::BufReader;

        // Eğer zaten bir şarkı çalıyorsa durdur
        if let Some(ref sink) = self.sink {
            sink.stop();
        }

        // Stream'i ilk kez oluştur veya mevcut olanı kullan
        if self._stream.is_none() {
            let (stream, handle) = OutputStream::try_default()?;
            self._stream = Some(stream);
            self.stream_handle = Some(handle);
        }

        if let Some(ref handle) = self.stream_handle {
            let sink = Sink::try_new(handle)?;
            let file = File::open(&track)?;
            let source = Decoder::new(BufReader::new(file))?;

            sink.append(source);

            self.sink = Some(Arc::new(sink));
            let track_name = display_name(&track).to_string();
            self.current_track = Some(track);
            self.playback_state = PlaybackState::Playing;
            self.status_message = format!("Now playing {track_name}");
            self.last_error = None;
        }

        Ok(())
    }

    /// Müziği duraklat/devam ettir
    pub fn toggle_pause(&mut self) {
        if let Some(ref sink) = self.sink {
            match self.playback_state {
                PlaybackState::Playing => {
                    sink.pause();
                    self.playback_state = PlaybackState::Paused;
                    self.status_message = "Playback paused".to_string();
                }
                PlaybackState::Paused => {
                    sink.play();
                    self.playback_state = PlaybackState::Playing;
                    self.status_message = "Playback resumed".to_string();
                }
                PlaybackState::Stopped => {
                    // Şarkı durduysa hiçbir şey yapma
                }
            }
        }
    }

    /// Müziği tamamen durdur
    pub fn stop_playback(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.stop();
        }
        self.sink = None;
        self.current_track = None;
        self.playback_state = PlaybackState::Stopped;
        self.status_message = "Playback stopped".to_string();
        // Stream'i kapatmıyoruz, çünkü tekrar kullanabiliriz
    }

    pub fn play_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.get_selected() {
            self.play_track(selected)?;
        }
        Ok(())
    }

    pub fn play_next_track(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.filtered_items.is_empty() {
            return Ok(());
        }

        let next_index = match self.current_track_index().or(self.state.selected()) {
            Some(index) => {
                if index + 1 < self.filtered_items.len() {
                    index + 1
                } else if self.repeat_mode == RepeatMode::All {
                    0
                } else {
                    return Ok(());
                }
            }
            None => 0,
        };

        self.state.select(Some(next_index));
        self.play_selected()
    }

    pub fn play_previous_track(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.filtered_items.is_empty() {
            return Ok(());
        }

        let previous_index = match self.current_track_index().or(self.state.selected()) {
            Some(index) => {
                if index > 0 {
                    index - 1
                } else if self.repeat_mode == RepeatMode::All {
                    self.filtered_items.len() - 1
                } else {
                    0
                }
            }
            None => 0,
        };

        self.state.select(Some(previous_index));
        self.play_selected()
    }

    /// Müzik çalma durumunu kontrol et
    pub fn update_playback_status(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let track_finished = self.sink.as_ref().map(|sink| sink.empty()).unwrap_or(false);

        if !track_finished {
            return Ok(());
        }

        match self.repeat_mode {
            RepeatMode::One => {
                if let Some(current) = self.current_track.clone() {
                    self.play_track(current)?;
                }
            }
            RepeatMode::All => {
                let next_index = self.current_track_index().map(|index| {
                    if index + 1 < self.filtered_items.len() {
                        index + 1
                    } else {
                        0
                    }
                });

                if let Some(next_index) = next_index {
                    self.state.select(Some(next_index));
                    self.play_selected()?;
                }
            }
            RepeatMode::Off => {
                self.current_track = None;
                self.playback_state = PlaybackState::Stopped;
                self.sink = None;
                self.status_message = "Track finished".to_string();
            }
        }

        Ok(())
    }
}

fn display_name(path: &Path) -> &str {
    path.file_stem()
        .and_then(|name| name.to_str())
        .or_else(|| path.file_name().and_then(|name| name.to_str()))
        .unwrap_or("Unknown")
}

fn folder_name(path: &Path) -> String {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or(".")
        .to_string()
}

pub fn track_display_name(path: &Path) -> String {
    display_name(path).to_string()
}

pub fn track_folder_name(path: &Path) -> String {
    folder_name(path)
}
