use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::{io, time::Duration};

/// Kullanıcı aksiyonlarını temsil eden enum
#[derive(Debug, Clone)]
pub enum AppAction {
    Quit,
    NextItem,
    PreviousItem,
    SelectItem,
    StartSearch,
    SearchInput(char),
    SearchBackspace,
    ConfirmSearch,
    CancelSearch,
    NextPage,
    PreviousPage,
    GoToTop,
    GoToBottom,
    ToggleHelp,
    Refresh,
    Random,
    NextTrack,
    PreviousTrack,
    CycleSort,
    CycleRepeat,
    CycleTheme,
    ToggleWaveform,
    TogglePause,
    StopPlayback,
}

/// Event'ları dinle ve uygun aksiyonu döndür
pub fn handle_events(is_searching: bool) -> io::Result<Option<AppAction>> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if is_searching {
                return Ok(match (key.code, key.modifiers) {
                    (KeyCode::Esc, _) => Some(AppAction::CancelSearch),
                    (KeyCode::Enter, _) => Some(AppAction::ConfirmSearch),
                    (KeyCode::Backspace, _) => Some(AppAction::SearchBackspace),
                    (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        Some(AppAction::SearchInput(ch))
                    }
                    _ => None,
                });
            }

            return Ok(match (key.code, key.modifiers) {
                // Çıkış
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => Some(AppAction::Quit),

                // Navigasyon - Vim tarzı
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(AppAction::NextItem),
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(AppAction::PreviousItem),

                // Sayfa navigasyonu
                (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    Some(AppAction::NextPage)
                }
                (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                    Some(AppAction::PreviousPage)
                }

                // Başa/sona git
                (KeyCode::Home, _) | (KeyCode::Char('g'), _) => Some(AppAction::GoToTop),
                (KeyCode::End, _) | (KeyCode::Char('G'), _) => Some(AppAction::GoToBottom),

                // Seçim
                (KeyCode::Enter, _) | (KeyCode::Char(' '), _) => Some(AppAction::SelectItem),
                (KeyCode::Char('/'), _) => Some(AppAction::StartSearch),

                // Özel fonksiyonlar
                (KeyCode::Char('?'), _) | (KeyCode::Char('h'), _) | (KeyCode::F(1), _) => {
                    Some(AppAction::ToggleHelp)
                }
                (KeyCode::Char('r'), _) | (KeyCode::F(5), _) => Some(AppAction::Refresh),
                (KeyCode::Char('s'), _) => Some(AppAction::Random),
                (KeyCode::Char('n'), _) => Some(AppAction::NextTrack),
                (KeyCode::Char('b'), _) => Some(AppAction::PreviousTrack),
                (KeyCode::Char('t'), _) => Some(AppAction::CycleSort),
                (KeyCode::Char('m'), _) => Some(AppAction::CycleRepeat),
                (KeyCode::Char('c'), _) => Some(AppAction::CycleTheme),
                (KeyCode::Char('w'), _) => Some(AppAction::ToggleWaveform),

                // Müzik kontrolleri
                (KeyCode::Char('p'), _) => Some(AppAction::TogglePause),
                (KeyCode::Char('x'), _) => Some(AppAction::StopPlayback),

                _ => None,
            });
        }
    }
    Ok(None)
}
