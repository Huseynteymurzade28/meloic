pub mod app;
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
use events::handle_events;
use widgets::draw_ui;

/// Ana UI fonksiyonu - terminal kurulumu ve event loop'u yönetir
pub fn run_ui(tracks: &[PathBuf], scan_roots: Vec<PathBuf>) -> io::Result<Option<PathBuf>> {
    // Terminal kurulumu
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App oluştur ve çalıştır
    let mut app = App::new(tracks.to_vec(), scan_roots);
    app.select_first(); // İlk öğeyi seç

    let result = run_app(&mut terminal, app);

    // Terminal'i eski haline getir
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Ana uygulama döngüsü
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> io::Result<Option<PathBuf>> {
    loop {
        // Müzik durumunu güncelle
        if let Err(error) = app.update_playback_status() {
            app.set_error(error);
        }

        // UI'ı çiz
        terminal.draw(|f| draw_ui(f, &mut app))?;

        // Event'ları handle et
        match handle_events(app.is_searching)? {
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
            None => {} // Hiçbir aksiyon yok, devam et
        }
    }
}
