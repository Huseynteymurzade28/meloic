use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::{io, time::Duration};

/// Kullanıcı aksiyonlarını temsil eden enum
#[derive(Debug, Clone)]
pub enum AppAction {
    Quit,
    NextItem,
    PreviousItem,
    SelectItem,
    NextPage,
    PreviousPage,
    GoToTop,
    GoToBottom,
    ToggleHelp,
    Refresh,
    Random,
    TogglePause,
    StopPlayback,
}

/// Event'ları dinle ve uygun aksiyonu döndür
pub fn handle_events() -> io::Result<Option<AppAction>> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
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

                // Özel fonksiyonlar
                (KeyCode::Char('?'), _) | (KeyCode::Char('h'), _) | (KeyCode::F(1), _) => {
                    Some(AppAction::ToggleHelp)
                }
                (KeyCode::Char('r'), _) | (KeyCode::F(5), _) => Some(AppAction::Refresh),
                (KeyCode::Char('s'), _) => Some(AppAction::Random),

                // Müzik kontrolleri
                (KeyCode::Char('p'), _) => Some(AppAction::TogglePause),
                (KeyCode::Char('x'), _) => Some(AppAction::StopPlayback),

                _ => None,
            });
        }
    }
    Ok(None)
}
