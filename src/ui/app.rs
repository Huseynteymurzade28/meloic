use rand::seq::SliceRandom;
use ratatui::widgets::ListState;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::path::PathBuf;
use std::sync::Arc;

/// Müzik çalma durumu
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
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
}
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Normal,
    Help,
}

impl App {
    /// Yeni bir App instance'ı oluştur
    pub fn new(items: Vec<PathBuf>) -> App {
        let total = items.len();
        App {
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
        }
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
        self.filtered_items = self
            .items
            .iter()
            .enumerate()
            .map(|(i, path)| (i, path.clone()))
            .collect();
    }

    /// Kütüphaneyi yenile
    pub fn refresh(&mut self) {
        // Bu fonksiyon ileride kütüphane tarama logic'i eklendiğinde kullanılabilir
        self.select_first();
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
            self.current_track = Some(track);
            self.playback_state = PlaybackState::Playing;
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
                }
                PlaybackState::Paused => {
                    sink.play();
                    self.playback_state = PlaybackState::Playing;
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
        // Stream'i kapatmıyoruz, çünkü tekrar kullanabiliriz
    }

    /// Müzik çalma durumunu kontrol et
    pub fn update_playback_status(&mut self) {
        if let Some(ref sink) = self.sink {
            if sink.empty() {
                // Şarkı bitti
                self.current_track = None;
                self.playback_state = PlaybackState::Stopped;
                self.sink = None;
            }
        }
    }
}
