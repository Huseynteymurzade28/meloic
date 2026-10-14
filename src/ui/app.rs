use rand::seq::SliceRandom;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::ListState;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::library;
use crate::metadata::{self, TrackMetadata};
use crate::playlist;
use crate::ui::cover::CoverRenderer;
use crate::waveform::{WaveformService, WaveformState};

const SEEK_SMALL: Duration = Duration::from_secs(5);
const SEEK_LARGE: Duration = Duration::from_secs(30);
const VOLUME_STEP: f32 = 0.05;
const DEFAULT_VOLUME: f32 = 0.8;
pub const LIST_ROW_HEIGHT: u16 = 2;
const STATUS_TTL: Duration = Duration::from_secs(5);

/// Screen regions recorded during the last draw, used to route mouse clicks.
#[derive(Default, Clone, Copy)]
pub struct HitAreas {
    pub list: Rect,
    pub list_offset: usize,
    pub progress_bar: Rect,
}

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
    Catppuccin,
    TokyoNight,
    Nord,
    Gruvbox,
    Solarized,
    RosePine,
    Dracula,
    Material,
    Monokai,
    Neon,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::Catppuccin => Self::TokyoNight,
            Self::TokyoNight => Self::Nord,
            Self::Nord => Self::Gruvbox,
            Self::Gruvbox => Self::Solarized,
            Self::Solarized => Self::RosePine,
            Self::RosePine => Self::Dracula,
            Self::Dracula => Self::Material,
            Self::Material => Self::Monokai,
            Self::Monokai => Self::Neon,
            Self::Neon => Self::Catppuccin,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Catppuccin => "catppuccin",
            Self::TokyoNight => "tokyo-night",
            Self::Nord => "nord",
            Self::Gruvbox => "gruvbox",
            Self::Solarized => "solarized",
            Self::RosePine => "rose-pine",
            Self::Dracula => "dracula",
            Self::Material => "material",
            Self::Monokai => "monokai",
            Self::Neon => "neon",
        }
    }

    pub fn from_label(s: &str) -> Option<Self> {
        match s {
            "catppuccin" => Some(Self::Catppuccin),
            "tokyo-night" => Some(Self::TokyoNight),
            "nord" => Some(Self::Nord),
            "gruvbox" => Some(Self::Gruvbox),
            "solarized" => Some(Self::Solarized),
            "rose-pine" => Some(Self::RosePine),
            "dracula" => Some(Self::Dracula),
            "material" => Some(Self::Material),
            "monokai" => Some(Self::Monokai),
            "neon" => Some(Self::Neon),
            _ => None,
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
            Self::Catppuccin => ThemePalette {
                accent: Color::Rgb(203, 166, 247),
                secondary: Color::Rgb(180, 190, 254),
                success: Color::Rgb(166, 227, 161),
                warning: Color::Rgb(250, 179, 135),
                text: Color::Rgb(205, 214, 244),
                muted: Color::Rgb(108, 112, 134),
                selection_bg: Color::Rgb(69, 71, 90),
            },
            Self::TokyoNight => ThemePalette {
                accent: Color::Rgb(187, 154, 247),
                secondary: Color::Rgb(122, 162, 247),
                success: Color::Rgb(158, 206, 106),
                warning: Color::Rgb(224, 175, 104),
                text: Color::Rgb(192, 202, 245),
                muted: Color::Rgb(86, 95, 137),
                selection_bg: Color::Rgb(54, 74, 130),
            },
            Self::Nord => ThemePalette {
                accent: Color::Rgb(136, 192, 208),
                secondary: Color::Rgb(129, 161, 193),
                success: Color::Rgb(163, 190, 140),
                warning: Color::Rgb(235, 203, 139),
                text: Color::Rgb(236, 239, 244),
                muted: Color::Rgb(76, 86, 106),
                selection_bg: Color::Rgb(59, 66, 82),
            },
            Self::Gruvbox => ThemePalette {
                accent: Color::Rgb(211, 134, 155),
                secondary: Color::Rgb(131, 165, 152),
                success: Color::Rgb(184, 187, 38),
                warning: Color::Rgb(250, 189, 47),
                text: Color::Rgb(235, 219, 178),
                muted: Color::Rgb(124, 111, 100),
                selection_bg: Color::Rgb(60, 56, 54),
            },
            Self::Solarized => ThemePalette {
                accent: Color::Rgb(131, 148, 210),
                secondary: Color::Rgb(42, 192, 182),
                success: Color::Rgb(155, 190, 0),
                warning: Color::Rgb(215, 165, 45),
                text: Color::Rgb(175, 190, 192),
                muted: Color::Rgb(110, 132, 140),
                selection_bg: Color::Rgb(7, 54, 66),
            },
            Self::RosePine => ThemePalette {
                accent: Color::Rgb(196, 167, 231),
                secondary: Color::Rgb(156, 207, 216),
                success: Color::Rgb(246, 193, 119),
                warning: Color::Rgb(235, 188, 186),
                text: Color::Rgb(224, 222, 244),
                muted: Color::Rgb(110, 106, 134),
                selection_bg: Color::Rgb(38, 35, 58),
            },
            Self::Dracula => ThemePalette {
                accent: Color::Rgb(255, 121, 198),
                secondary: Color::Rgb(189, 147, 249),
                success: Color::Rgb(80, 250, 123),
                warning: Color::Rgb(241, 250, 140),
                text: Color::Rgb(248, 248, 242),
                muted: Color::Rgb(98, 114, 164),
                selection_bg: Color::Rgb(68, 71, 90),
            },
            Self::Material => ThemePalette {
                accent: Color::Rgb(128, 203, 196),
                secondary: Color::Rgb(130, 177, 255),
                success: Color::Rgb(195, 232, 141),
                warning: Color::Rgb(255, 203, 107),
                text: Color::Rgb(238, 255, 255),
                muted: Color::Rgb(84, 110, 122),
                selection_bg: Color::Rgb(55, 71, 79),
            },
            Self::Monokai => ThemePalette {
                accent: Color::Rgb(249, 38, 114),
                secondary: Color::Rgb(102, 217, 232),
                success: Color::Rgb(166, 226, 46),
                warning: Color::Rgb(230, 219, 116),
                text: Color::Rgb(248, 248, 242),
                muted: Color::Rgb(117, 113, 94),
                selection_bg: Color::Rgb(73, 72, 62),
            },
            Self::Neon => ThemePalette {
                accent: Color::Rgb(255, 120, 198),
                secondary: Color::Rgb(120, 220, 255),
                success: Color::Rgb(140, 255, 180),
                warning: Color::Rgb(255, 210, 120),
                text: Color::Rgb(230, 235, 245),
                muted: Color::Rgb(120, 130, 150),
                selection_bg: Color::Rgb(62, 76, 104),
            },
        }
    }
}

fn config_theme_path() -> std::path::PathBuf {
    crate::playlist::config_dir().join("theme")
}

pub fn load_saved_theme() -> ThemeMode {
    std::fs::read_to_string(config_theme_path())
        .ok()
        .and_then(|s| ThemeMode::from_label(s.trim()))
        .unwrap_or(ThemeMode::Catppuccin)
}

pub fn save_theme(theme: ThemeMode) {
    let path = config_theme_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, theme.label());
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

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Normal,
    Help,
    Queue,
    Info,
    Cover,
}

pub struct App {
    pub items: Vec<PathBuf>,
    pub state: ListState,
    pub queue_state: ListState,
    pub show_help: bool,
    pub filtered_items: Vec<(usize, PathBuf)>,
    pub total_tracks: usize,
    pub current_view: ViewMode,
    pub playback_state: PlaybackState,
    pub current_track: Option<PathBuf>,
    pub sink: Option<Arc<Sink>>,
    pub _stream: Option<OutputStream>,
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
    pub volume: f32,
    pub muted: bool,
    pub volume_before_mute: f32,
    pub queue: Vec<PathBuf>,
    pub metadata_cache: HashMap<PathBuf, Option<TrackMetadata>>,
    pub waveform: WaveformService,
    pub album_art_cache: HashMap<PathBuf, Option<Vec<u8>>>,
    pub covers: RefCell<CoverRenderer>,
    pub hit_areas: Cell<HitAreas>,
    pub shuffle: bool,
    pub started_at: Instant,
    status_seen: RefCell<(String, Instant)>,
}

impl App {
    pub fn new(
        items: Vec<PathBuf>,
        scan_roots: Vec<PathBuf>,
        initial_queue: Vec<PathBuf>,
        picker: Option<ratatui_image::picker::Picker>,
    ) -> App {
        let total = items.len();
        let mut app = App {
            items: items.clone(),
            state: ListState::default(),
            queue_state: ListState::default(),
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
            theme_mode: load_saved_theme(),
            status_message: format!("Loaded {total} tracks"),
            last_error: None,
            waveform_hidden_tracks: HashSet::new(),
            volume: DEFAULT_VOLUME,
            muted: false,
            volume_before_mute: DEFAULT_VOLUME,
            queue: initial_queue,
            metadata_cache: HashMap::new(),
            waveform: WaveformService::new(),
            album_art_cache: HashMap::new(),
            covers: RefCell::new(CoverRenderer::new(picker)),
            hit_areas: Cell::new(HitAreas::default()),
            shuffle: false,
            started_at: Instant::now(),
            status_seen: RefCell::new((String::new(), Instant::now())),
        };
        app.update_filtered_items();
        app
    }

    pub fn select_first(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(0));
            if let Some(track) = self.get_selected() {
                self.request_track_assets(&track);
            }
        }
    }

    pub fn next(&mut self) {
        match self.current_view {
            ViewMode::Queue => self.queue_next(),
            _ => self.library_next(),
        }
    }

    pub fn previous(&mut self) {
        match self.current_view {
            ViewMode::Queue => self.queue_previous(),
            _ => self.library_previous(),
        }
    }

    fn library_next(&mut self) {
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
        if let Some(track) = self.get_selected() {
            self.request_track_assets(&track);
        }
    }

    fn library_previous(&mut self) {
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
        if let Some(track) = self.get_selected() {
            self.request_track_assets(&track);
        }
    }

    fn queue_next(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        let i = match self.queue_state.selected() {
            Some(i) => {
                if i + 1 >= self.queue.len() {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.queue_state.select(Some(i));
    }

    fn queue_previous(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        let i = match self.queue_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.queue.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.queue_state.select(Some(i));
    }

    pub fn next_page(&mut self) {
        if let Some(selected) = self.state.selected() {
            let new_index =
                std::cmp::min(selected + 10, self.filtered_items.len().saturating_sub(1));
            self.state.select(Some(new_index));
            if let Some(track) = self.get_selected() {
                self.request_track_assets(&track);
            }
        }
    }

    pub fn previous_page(&mut self) {
        if let Some(selected) = self.state.selected() {
            let new_index = selected.saturating_sub(10);
            self.state.select(Some(new_index));
            if let Some(track) = self.get_selected() {
                self.request_track_assets(&track);
            }
        }
    }

    pub fn go_to_top(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(0));
            if let Some(track) = self.get_selected() {
                self.request_track_assets(&track);
            }
        }
    }

    pub fn go_to_bottom(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(self.filtered_items.len() - 1));
            if let Some(track) = self.get_selected() {
                self.request_track_assets(&track);
            }
        }
    }

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
            if let Some(track) = self.get_selected() {
                self.request_track_assets(&track);
            }
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.current_view = if self.show_help {
            ViewMode::Help
        } else {
            ViewMode::Normal
        };
    }

    pub fn toggle_queue_view(&mut self) {
        match self.current_view {
            ViewMode::Queue => {
                self.current_view = ViewMode::Normal;
            }
            _ => {
                self.current_view = ViewMode::Queue;
                if !self.queue.is_empty() && self.queue_state.selected().is_none() {
                    self.queue_state.select(Some(0));
                }
            }
        }
    }

    pub fn toggle_cover_view(&mut self) {
        self.current_view = match self.current_view {
            ViewMode::Cover => ViewMode::Normal,
            _ => {
                if let Some(track) = self.current_track.clone().or_else(|| self.get_selected()) {
                    self.ensure_metadata(&track);
                    self.ensure_album_art(&track);
                }
                ViewMode::Cover
            }
        };
    }

    pub fn toggle_info_view(&mut self) {
        self.current_view = match self.current_view {
            ViewMode::Info => ViewMode::Normal,
            _ => {
                if let Some(track) = self.get_selected() {
                    self.ensure_metadata(&track);
                }
                ViewMode::Info
            }
        };
    }

    /// Esc in the main view: clear an active filter instead of quitting.
    pub fn escape(&mut self) {
        if !self.search_query.is_empty() {
            self.cancel_search();
        }
    }

    pub fn scroll(&mut self, down: bool) {
        if down { self.next() } else { self.previous() }
    }

    /// Left click: select a library row (play it when already selected) or
    /// seek when the progress bar is hit.
    pub fn click(&mut self, column: u16, row: u16) -> Result<(), Box<dyn std::error::Error>> {
        let hit = self.hit_areas.get();
        let contains =
            |r: Rect| column >= r.x && column < r.x + r.width && row >= r.y && row < r.y + r.height;

        if contains(hit.progress_bar) && hit.progress_bar.width > 0 {
            if let Some(total) = self.current_total_duration() {
                let frac = (column - hit.progress_bar.x) as f64 / hit.progress_bar.width as f64;
                self.seek_to(total.mul_f64(frac.clamp(0.0, 1.0)));
            }
            return Ok(());
        }

        if contains(hit.list) {
            let index = hit.list_offset + ((row - hit.list.y) / LIST_ROW_HEIGHT) as usize;
            if index >= self.filtered_items.len() {
                return Ok(());
            }
            if self.state.selected() == Some(index) {
                return self.play_selected();
            }
            self.state.select(Some(index));
            if let Some(track) = self.get_selected() {
                self.request_track_assets(&track);
            }
        }
        Ok(())
    }

    /// Status message plus whether it is still fresh (recently changed).
    pub fn status(&self) -> (String, bool) {
        let mut seen = self.status_seen.borrow_mut();
        if seen.0 != self.status_message {
            *seen = (self.status_message.clone(), Instant::now());
        }
        (self.status_message.clone(), seen.1.elapsed() < STATUS_TTL)
    }

    /// Load tags for rows about to be drawn so the list can show artist/duration.
    pub fn ensure_metadata_range(&mut self, start: usize, count: usize) {
        let end = (start + count).min(self.filtered_items.len());
        let paths: Vec<PathBuf> = self.filtered_items[start.min(end)..end]
            .iter()
            .map(|(_, p)| p.clone())
            .collect();
        for path in paths {
            self.ensure_metadata(&path);
        }
    }

    fn update_filtered_items(&mut self) {
        let previous_selection = self.get_selected();
        let query = self.search_query.to_lowercase();

        let metadata_cache = &self.metadata_cache;
        self.filtered_items = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, path)| {
                if query.is_empty() {
                    return true;
                }
                let meta_match = metadata_cache
                    .get(*path)
                    .and_then(|m| m.as_ref())
                    .map(|m| {
                        [&m.title, &m.artist, &m.album].iter().any(|field| {
                            field
                                .as_deref()
                                .is_some_and(|v| v.to_lowercase().contains(&query))
                        })
                    })
                    .unwrap_or(false);
                meta_match
                    || path
                        .file_name()
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

    pub fn refresh(&mut self) {
        self.items = library::scan_music_roots(&self.scan_roots);
        self.total_tracks = self.items.len();
        self.update_filtered_items();
        self.status_message = format!("Refreshed library: {} tracks", self.total_tracks);
    }

    pub fn get_selected(&self) -> Option<PathBuf> {
        self.state
            .selected()
            .and_then(|i| self.filtered_items.get(i))
            .map(|(_, path)| path.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.filtered_items.is_empty()
    }

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
        self.status_message =
            "Search mode: type to filter, Enter to keep, Esc to clear".to_string();
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

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        self.status_message = format!("Shuffle: {}", if self.shuffle { "on" } else { "off" });
    }

    pub fn cycle_theme_mode(&mut self) {
        self.theme_mode = self.theme_mode.next();
        save_theme(self.theme_mode);
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

    pub fn waveform_state_for(&self, track: &Path) -> Option<&WaveformState> {
        self.waveform.get(track)
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

    pub fn play_track(&mut self, track: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        use rodio::{Decoder, Sink};
        use std::fs::File;
        use std::io::BufReader;

        if let Some(ref sink) = self.sink {
            sink.stop();
        }

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
            sink.set_volume(if self.muted { 0.0 } else { self.volume });

            self.sink = Some(Arc::new(sink));
            self.ensure_metadata(&track);
            let track_name = self
                .metadata_for(&track)
                .and_then(|m| m.title.clone())
                .unwrap_or_else(|| display_name(&track));
            self.ensure_album_art(&track);
            self.waveform.request(&track);
            self.current_track = Some(track);
            self.playback_state = PlaybackState::Playing;
            self.status_message = format!("Now playing {track_name}");
            self.last_error = None;
        }

        Ok(())
    }

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
                PlaybackState::Stopped => {}
            }
        }
    }

    pub fn stop_playback(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.stop();
        }
        self.sink = None;
        self.current_track = None;
        self.playback_state = PlaybackState::Stopped;
        self.status_message = "Playback stopped".to_string();
    }

    pub fn play_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.get_selected() {
            self.play_track(selected)?;
        }
        Ok(())
    }

    pub fn play_next_track(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.queue.is_empty() {
            let next = self.queue.remove(0);
            self.fix_queue_state_after_removal(0);
            return self.play_track(next);
        }

        let Some(next_index) = self.next_library_index() else {
            return Ok(());
        };
        self.state.select(Some(next_index));
        self.play_selected()
    }

    /// Index of the track that should follow the current one, honouring
    /// shuffle and repeat-all. `None` means the end of the list was reached.
    fn next_library_index(&self) -> Option<usize> {
        let len = self.filtered_items.len();
        if len == 0 {
            return None;
        }
        let current = self.current_track_index().or(self.state.selected());

        if self.shuffle && len > 1 {
            let mut rng = rand::thread_rng();
            let candidates: Vec<usize> = (0..len).filter(|i| Some(*i) != current).collect();
            return candidates.choose(&mut rng).copied();
        }

        match current {
            Some(index) if index + 1 < len => Some(index + 1),
            Some(_) if self.repeat_mode == RepeatMode::All => Some(0),
            Some(_) => None,
            None => Some(0),
        }
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

    pub fn update_playback_status(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.waveform.poll();

        // The waveform panel follows the playing track, falling back to the
        // selection; re-request whichever one a skipped decode left empty.
        if let Some(track) = self
            .current_track
            .clone()
            .or_else(|| self.get_selected())
            .filter(|track| self.is_waveform_enabled_for(track))
        {
            self.waveform.request(&track);
        }

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
            _ => {
                if !self.queue.is_empty() {
                    let next = self.queue.remove(0);
                    self.fix_queue_state_after_removal(0);
                    self.play_track(next)?;
                } else if let Some(next_index) = self
                    .current_track_index()
                    .and_then(|_| self.next_library_index())
                {
                    self.state.select(Some(next_index));
                    self.play_selected()?;
                } else {
                    self.current_track = None;
                    self.playback_state = PlaybackState::Stopped;
                    self.sink = None;
                    self.status_message = "Track finished".to_string();
                }
            }
        }

        Ok(())
    }

    // -------- Volume --------

    pub fn volume_up(&mut self) {
        self.muted = false;
        self.volume = (self.volume + VOLUME_STEP).min(1.0);
        self.apply_volume();
        self.status_message = format!("Volume {}%", (self.volume * 100.0).round() as u32);
    }

    pub fn volume_down(&mut self) {
        self.muted = false;
        self.volume = (self.volume - VOLUME_STEP).max(0.0);
        self.apply_volume();
        self.status_message = format!("Volume {}%", (self.volume * 100.0).round() as u32);
    }

    pub fn toggle_mute(&mut self) {
        if self.muted {
            self.muted = false;
            self.volume = self.volume_before_mute;
            self.status_message = format!("Unmuted ({}%)", (self.volume * 100.0).round() as u32);
        } else {
            self.muted = true;
            self.volume_before_mute = self.volume;
            self.status_message = "Muted".to_string();
        }
        self.apply_volume();
    }

    fn apply_volume(&self) {
        if let Some(ref sink) = self.sink {
            sink.set_volume(if self.muted { 0.0 } else { self.volume });
        }
    }

    pub fn volume_percent(&self) -> u32 {
        if self.muted {
            0
        } else {
            (self.volume * 100.0).round() as u32
        }
    }

    // -------- Seek --------

    pub fn seek_forward(&mut self) {
        self.seek_by(SEEK_SMALL, true);
    }

    pub fn seek_backward(&mut self) {
        self.seek_by(SEEK_SMALL, false);
    }

    pub fn seek_forward_large(&mut self) {
        self.seek_by(SEEK_LARGE, true);
    }

    pub fn seek_backward_large(&mut self) {
        self.seek_by(SEEK_LARGE, false);
    }

    fn seek_by(&mut self, delta: Duration, forward: bool) {
        let Some(ref sink) = self.sink else {
            self.status_message = "Nothing playing to seek".to_string();
            return;
        };
        let pos = sink.get_pos();
        let target = if forward {
            pos.saturating_add(delta)
        } else {
            pos.saturating_sub(delta)
        };
        self.seek_to(target);
    }

    pub fn seek_to(&mut self, target: Duration) {
        let Some(ref sink) = self.sink else {
            return;
        };
        let pos = sink.get_pos();
        let total = self.current_total_duration();
        let clamped = if let Some(total) = total {
            if target >= total {
                total.saturating_sub(Duration::from_millis(500))
            } else {
                target
            }
        } else {
            target
        };

        match sink.try_seek(clamped) {
            Ok(_) => {
                self.status_message = format!(
                    "Seek {} -> {}",
                    metadata::format_duration(pos),
                    metadata::format_duration(clamped)
                );
            }
            Err(err) => {
                self.status_message = format!("Seek not supported: {err}");
            }
        }
    }

    pub fn progress_ratio(&self) -> f64 {
        match self.current_total_duration() {
            Some(total) if total.as_secs_f64() > 0.0 => {
                (self.current_position().as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0)
            }
            _ => 0.0,
        }
    }

    pub fn current_position(&self) -> Duration {
        self.sink.as_ref().map(|s| s.get_pos()).unwrap_or_default()
    }

    pub fn current_total_duration(&self) -> Option<Duration> {
        self.current_track
            .as_ref()
            .and_then(|track| self.metadata_cache.get(track))
            .and_then(|m| m.as_ref())
            .and_then(|m| m.duration)
    }

    // -------- Metadata --------

    pub fn ensure_metadata(&mut self, track: &Path) {
        if self.metadata_cache.contains_key(track) {
            return;
        }
        let meta = metadata::read_metadata(track);
        self.metadata_cache.insert(track.to_path_buf(), meta);
    }

    pub fn metadata_for(&self, track: &Path) -> Option<&TrackMetadata> {
        self.metadata_cache.get(track).and_then(|m| m.as_ref())
    }

    pub fn ensure_album_art(&mut self, track: &Path) {
        if self.album_art_cache.contains_key(track) {
            return;
        }
        let art = metadata::read_album_art(track);
        self.album_art_cache.insert(track.to_path_buf(), art);
    }

    pub fn album_art_for(&self, track: &Path) -> Option<&[u8]> {
        self.album_art_cache.get(track)?.as_deref()
    }

    fn request_track_assets(&mut self, track: &Path) {
        self.ensure_metadata(track);
        self.ensure_album_art(track);
        if self.is_waveform_enabled_for(track) {
            self.waveform.request(track);
        }
    }

    // -------- Queue / playlist --------

    pub fn add_selected_to_queue(&mut self) {
        let Some(track) = self.get_selected() else {
            self.status_message = "No track selected".to_string();
            return;
        };
        self.queue.push(track.clone());
        self.status_message = format!(
            "Queued {} (queue size {})",
            display_name(&track),
            self.queue.len()
        );
    }

    pub fn add_filtered_to_queue(&mut self) {
        let mut added = 0;
        for (_, path) in self.filtered_items.clone() {
            self.queue.push(path);
            added += 1;
        }
        self.status_message = format!("Queued {} tracks (size {})", added, self.queue.len());
    }

    pub fn clear_queue(&mut self) {
        let n = self.queue.len();
        self.queue.clear();
        self.queue_state.select(None);
        self.status_message = format!("Cleared queue ({} tracks)", n);
    }

    pub fn remove_queue_selected(&mut self) {
        let Some(i) = self.queue_state.selected() else {
            return;
        };
        if i >= self.queue.len() {
            return;
        }
        let removed = self.queue.remove(i);
        self.fix_queue_state_after_removal(i);
        self.status_message = format!("Removed {} from queue", display_name(&removed));
    }

    fn fix_queue_state_after_removal(&mut self, removed_index: usize) {
        if self.queue.is_empty() {
            self.queue_state.select(None);
            return;
        }
        let new_index = removed_index.min(self.queue.len() - 1);
        if let Some(sel) = self.queue_state.selected() {
            if sel >= self.queue.len() {
                self.queue_state.select(Some(new_index));
            }
        }
    }

    pub fn queue_play_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(i) = self.queue_state.selected() else {
            return Ok(());
        };
        if i >= self.queue.len() {
            return Ok(());
        }
        let track = self.queue.remove(i);
        self.fix_queue_state_after_removal(i);
        self.current_view = ViewMode::Normal;
        self.play_track(track)
    }

    pub fn save_queue(&mut self) {
        let path = playlist::default_queue_path();
        match playlist::write_m3u(&path, &self.queue) {
            Ok(_) => {
                self.status_message =
                    format!("Saved {} tracks to {}", self.queue.len(), path.display());
            }
            Err(e) => self.set_error(e),
        }
    }

    pub fn load_queue(&mut self) {
        let path = playlist::default_queue_path();
        match playlist::parse_m3u(&path) {
            Ok(tracks) => {
                let n = tracks.len();
                self.queue = tracks;
                self.status_message = format!("Loaded {} tracks from {}", n, path.display());
            }
            Err(e) => self.set_error(e),
        }
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn queue_peek(&self, n: usize) -> &[PathBuf] {
        let end = n.min(self.queue.len());
        &self.queue[..end]
    }
}

fn display_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .or_else(|| path.file_name().and_then(|name| name.to_str()))
        .unwrap_or("Unknown");
    strip_video_id_suffix(stem)
}

fn strip_video_id_suffix(name: &str) -> String {
    let trimmed = name.trim_end();
    if !trimmed.ends_with(']') {
        return name.to_string();
    }
    let Some(open) = trimmed.rfind('[') else {
        return name.to_string();
    };
    let inner = &trimmed[open + 1..trimmed.len() - 1];
    if inner.len() == 11
        && inner
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        trimmed[..open].trim_end().to_string()
    } else {
        name.to_string()
    }
}

fn folder_name(path: &Path) -> String {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or(".")
        .to_string()
}

pub fn track_display_name(path: &Path) -> String {
    display_name(path)
}

pub fn track_folder_name(path: &Path) -> String {
    folder_name(path)
}
