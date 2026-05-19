use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::{io, time::Duration};

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
    VolumeUp,
    VolumeDown,
    ToggleMute,
    SeekForward,
    SeekBackward,
    SeekForwardLarge,
    SeekBackwardLarge,
    AddToQueue,
    AddFilteredToQueue,
    ClearQueue,
    SaveQueue,
    LoadQueue,
    ToggleQueueView,
    ToggleInfoView,
    QueueRemoveSelected,
    QueuePlaySelected,
}

pub fn handle_events(
    is_searching: bool,
    in_popup_queue: bool,
    in_popup_info: bool,
) -> io::Result<Option<AppAction>> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(None);
    }
    let Event::Key(key) = event::read()? else {
        return Ok(None);
    };

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

    if in_popup_info {
        return Ok(match (key.code, key.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('i'), _) | (KeyCode::Char('q'), _) => {
                Some(AppAction::ToggleInfoView)
            }
            _ => None,
        });
    }

    if in_popup_queue {
        return Ok(match (key.code, key.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('Q'), _) => Some(AppAction::ToggleQueueView),
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(AppAction::NextItem),
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(AppAction::PreviousItem),
            (KeyCode::Char('d'), _) | (KeyCode::Delete, _) => {
                Some(AppAction::QueueRemoveSelected)
            }
            (KeyCode::Char('c'), _) => Some(AppAction::ClearQueue),
            (KeyCode::Enter, _) => Some(AppAction::QueuePlaySelected),
            _ => None,
        });
    }

    Ok(match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => Some(AppAction::Quit),

        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(AppAction::NextItem),
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(AppAction::PreviousItem),

        (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            Some(AppAction::NextPage)
        }
        (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            Some(AppAction::PreviousPage)
        }

        (KeyCode::Home, _) | (KeyCode::Char('g'), _) => Some(AppAction::GoToTop),
        (KeyCode::End, _) | (KeyCode::Char('G'), _) => Some(AppAction::GoToBottom),

        (KeyCode::Enter, _) | (KeyCode::Char(' '), _) => Some(AppAction::SelectItem),
        (KeyCode::Char('/'), _) => Some(AppAction::StartSearch),

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

        (KeyCode::Char('p'), _) => Some(AppAction::TogglePause),
        (KeyCode::Char('x'), _) => Some(AppAction::StopPlayback),

        // Volume
        (KeyCode::Char('+'), _) | (KeyCode::Char('='), _) => Some(AppAction::VolumeUp),
        (KeyCode::Char('-'), _) | (KeyCode::Char('_'), _) => Some(AppAction::VolumeDown),
        (KeyCode::Char('0'), _) => Some(AppAction::ToggleMute),

        // Seek
        (KeyCode::Left, m) if m.contains(KeyModifiers::SHIFT) => {
            Some(AppAction::SeekBackwardLarge)
        }
        (KeyCode::Right, m) if m.contains(KeyModifiers::SHIFT) => {
            Some(AppAction::SeekForwardLarge)
        }
        (KeyCode::Left, _) => Some(AppAction::SeekBackward),
        (KeyCode::Right, _) => Some(AppAction::SeekForward),

        // Queue / playlist
        (KeyCode::Char('a'), _) => Some(AppAction::AddToQueue),
        (KeyCode::Char('A'), _) => Some(AppAction::AddFilteredToQueue),
        (KeyCode::Char('Q'), _) => Some(AppAction::ToggleQueueView),
        (KeyCode::Char('S'), _) => Some(AppAction::SaveQueue),
        (KeyCode::Char('L'), _) => Some(AppAction::LoadQueue),
        (KeyCode::Char('i'), _) => Some(AppAction::ToggleInfoView),

        _ => None,
    })
}
