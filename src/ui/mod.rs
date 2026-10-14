pub mod app;
pub mod cover;
pub mod events;
pub mod help;
pub mod widgets;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, path::PathBuf};

use app::App;
use app::ViewMode;
use events::{InputMode, handle_events};
use widgets::draw_ui;

pub fn run_ui(
    tracks: &[PathBuf],
    scan_roots: Vec<PathBuf>,
    initial_queue: Vec<PathBuf>,
) -> io::Result<Option<PathBuf>> {
    let picker = cover::detect_picker();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(tracks.to_vec(), scan_roots, initial_queue, picker);
    app.select_first();

    let result = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> io::Result<Option<PathBuf>> {
    loop {
        if let Err(error) = app.update_playback_status() {
            app.set_error(error);
        }

        terminal.draw(|f| draw_ui(f, &mut app))?;

        let mode = if app.is_searching {
            InputMode::Search
        } else {
            match app.current_view {
                ViewMode::Normal => InputMode::Normal,
                ViewMode::Help => InputMode::Help,
                ViewMode::Queue => InputMode::Queue,
                ViewMode::Info => InputMode::Info,
                ViewMode::Cover => InputMode::Cover,
            }
        };

        match handle_events(mode)? {
            Some(events::AppAction::Quit) => return Ok(None),
            Some(events::AppAction::NextItem) => app.next(),
            Some(events::AppAction::PreviousItem) => app.previous(),
            Some(events::AppAction::StartSearch) => app.start_search(),
            Some(events::AppAction::SearchInput(ch)) => app.append_search_char(ch),
            Some(events::AppAction::SearchBackspace) => app.backspace_search(),
            Some(events::AppAction::ConfirmSearch) => app.confirm_search(),
            Some(events::AppAction::CancelSearch) => app.cancel_search(),
            Some(events::AppAction::NextPage) => app.next_page(),
            Some(events::AppAction::PreviousPage) => app.previous_page(),
            Some(events::AppAction::GoToTop) => app.go_to_top(),
            Some(events::AppAction::GoToBottom) => app.go_to_bottom(),
            Some(events::AppAction::SelectItem) => {
                if let Err(error) = app.play_selected() {
                    app.set_error(error);
                }
            }
            Some(events::AppAction::ToggleHelp) => app.toggle_help(),
            Some(events::AppAction::Refresh) => app.refresh(),
            Some(events::AppAction::Random) => app.select_random(),
            Some(events::AppAction::NextTrack) => {
                if let Err(error) = app.play_next_track() {
                    app.set_error(error);
                }
            }
            Some(events::AppAction::PreviousTrack) => {
                if let Err(error) = app.play_previous_track() {
                    app.set_error(error);
                }
            }
            Some(events::AppAction::CycleSort) => app.cycle_sort_mode(),
            Some(events::AppAction::CycleRepeat) => app.cycle_repeat_mode(),
            Some(events::AppAction::CycleTheme) => app.cycle_theme_mode(),
            Some(events::AppAction::ToggleWaveform) => app.toggle_selected_waveform(),
            Some(events::AppAction::TogglePause) => app.toggle_pause(),
            Some(events::AppAction::StopPlayback) => app.stop_playback(),
            Some(events::AppAction::VolumeUp) => app.volume_up(),
            Some(events::AppAction::VolumeDown) => app.volume_down(),
            Some(events::AppAction::ToggleMute) => app.toggle_mute(),
            Some(events::AppAction::SeekForward) => app.seek_forward(),
            Some(events::AppAction::SeekBackward) => app.seek_backward(),
            Some(events::AppAction::SeekForwardLarge) => app.seek_forward_large(),
            Some(events::AppAction::SeekBackwardLarge) => app.seek_backward_large(),
            Some(events::AppAction::AddToQueue) => app.add_selected_to_queue(),
            Some(events::AppAction::AddFilteredToQueue) => app.add_filtered_to_queue(),
            Some(events::AppAction::ClearQueue) => app.clear_queue(),
            Some(events::AppAction::SaveQueue) => app.save_queue(),
            Some(events::AppAction::LoadQueue) => app.load_queue(),
            Some(events::AppAction::ToggleQueueView) => app.toggle_queue_view(),
            Some(events::AppAction::ToggleInfoView) => app.toggle_info_view(),
            Some(events::AppAction::QueueRemoveSelected) => app.remove_queue_selected(),
            Some(events::AppAction::QueuePlaySelected) => {
                if let Err(error) = app.queue_play_selected() {
                    app.set_error(error);
                }
            }
            Some(events::AppAction::ToggleCoverView) => app.toggle_cover_view(),
            Some(events::AppAction::ToggleShuffle) => app.toggle_shuffle(),
            Some(events::AppAction::Escape) => app.escape(),
            Some(events::AppAction::ScrollDown) => app.scroll(true),
            Some(events::AppAction::ScrollUp) => app.scroll(false),
            Some(events::AppAction::Click(column, row)) => {
                if let Err(error) = app.click(column, row) {
                    app.set_error(error);
                }
            }
            None => {}
        }
    }
}
