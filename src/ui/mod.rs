pub mod app;
pub mod events;
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
pub fn run_ui(tracks: &Vec<PathBuf>) -> io::Result<Option<PathBuf>> {
    // Terminal kurulumu
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App oluştur ve çalıştır
    let mut app = App::new(tracks.clone());
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
        app.update_playback_status();

        // UI'ı çiz
        terminal.draw(|f| draw_ui(f, &mut app))?;

        // Event'ları handle et
        match handle_events()? {
            Some(events::AppAction::Quit) => return Ok(None),
            Some(events::AppAction::NextItem) => app.next(),
            Some(events::AppAction::PreviousItem) => app.previous(),
            Some(events::AppAction::NextPage) => app.next_page(),
            Some(events::AppAction::PreviousPage) => app.previous_page(),
            Some(events::AppAction::GoToTop) => app.go_to_top(),
            Some(events::AppAction::GoToBottom) => app.go_to_bottom(),
            Some(events::AppAction::SelectItem) => {
                if let Some(selected) = app.get_selected() {
                    // Müziği doğrudan çal, UI'dan çıkma
                    if let Err(_) = app.play_track(selected) {
                        // Hata durumunda sessizce devam et
                    }
                }
            }
            Some(events::AppAction::ToggleHelp) => app.toggle_help(),
            Some(events::AppAction::Refresh) => app.refresh(),
            Some(events::AppAction::Random) => app.select_random(),
            Some(events::AppAction::TogglePause) => app.toggle_pause(),
            Some(events::AppAction::StopPlayback) => app.stop_playback(),
            None => {} // Hiçbir aksiyon yok, devam et
        }
    }
}
