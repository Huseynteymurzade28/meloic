use crate::ui::app::{App, ViewMode};
use crate::ui::help::draw_help_screen;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Ana UI'ı çiz
pub fn draw_ui(f: &mut Frame, app: &mut App) {
    match app.current_view {
        ViewMode::Help => draw_help_screen(f, app),
        ViewMode::Normal => draw_main_screen(f, app),
    }
}

/// Ana ekranı çiz
fn draw_main_screen(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Başlık + durum çubuğu
                Constraint::Min(8),    // Ana liste 
                Constraint::Length(1), // Alt durum çubuğu (tek satır)
            ]
            .as_ref(),
        )
        .split(f.size());

    // Üst bilgi paneli çiz
    draw_header_panel(f, main_chunks[0], app);

    // Müzik listesini çiz
    draw_enhanced_music_list(f, main_chunks[1], app);

    // Alt durum çubuğunu çiz
    draw_status_bar(f, main_chunks[2], app);
}

/// Gelişmiş başlık paneli
fn draw_header_panel(f: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(30), // Logo ve başlık
                Constraint::Min(20),    // Orta boşluk
                Constraint::Length(40), // İstatistikler
            ]
            .as_ref(),
        )
        .split(area);

    // Sol: Logo ve başlık
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "🎵 ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "MELOIC",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ])])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(title, header_chunks[0]);

    // Sağ: İstatistikler
    let stats_text = format!("📀 {} tracks", app.total_tracks);

    let stats = Paragraph::new(vec![Line::from(vec![Span::styled(
        stats_text,
        Style::default().fg(Color::Yellow),
    )])])
    .block(Block::default().borders(Borders::ALL).title("Library"))
    .alignment(Alignment::Center);
    f.render_widget(stats, header_chunks[2]);
}

/// Gelişmiş müzik listesi
fn draw_enhanced_music_list(f: &mut Frame, area: Rect, app: &mut App) {
    if app.is_empty() {
        let empty_msg = "🎵 No music files found in the 'music' directory\n\n📁 Add some .mp3, .flac, .wav or .ogg files";

        let empty_widget = Paragraph::new(empty_msg)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🎼 Music Library")
                    .style(Style::default().fg(Color::Gray)),
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);

        f.render_widget(empty_widget, area);
        return;
    }

    let display_items = app.get_display_items().clone();
    let selected = app.state.selected();

    let items: Vec<ListItem> = display_items
        .iter()
        .enumerate()
        .map(|(display_idx, (_, path))| {
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown");

            // Dosya uzantısına göre ikon seç
            let icon = match path.extension().and_then(|ext| ext.to_str()) {
                Some("mp3") => "🎵",
                Some("flac") => "🎶",
                Some("wav") => "🔊",
                Some("ogg") => "🎧",
                _ => "🎼",
            };

            // Seçili öğe için özel stil
            let number_style = if Some(display_idx) == selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let filename_style = if Some(display_idx) == selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:3}. ", display_idx + 1), number_style),
                Span::styled(format!("{} ", icon), Style::default().fg(Color::Green)),
                Span::styled(filename, filename_style),
            ]))
        })
        .collect();

    let list_title = "🎼 Music Library".to_string();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.state);
}

/// Alt durum çubuğu
fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    // Tek satır: Seçili track, help mesajı ve çalan şarkı
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Sol: Track info
            Constraint::Percentage(40), // Orta: Help mesajı
            Constraint::Percentage(30), // Sağ: Çalan şarkı
        ])
        .split(area);

    // Sol: Seçili track bilgisi
    let current_selection = app
        .state
        .selected()
        .map(|i| format!("Track {}/{}", i + 1, app.filtered_items.len()))
        .unwrap_or_else(|| "No selection".to_string());

    let selection_widget = Paragraph::new(current_selection).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(selection_widget, top_chunks[0]);

    // Orta: Help mesajı
    let help_text = "Press 'h' for help";
    let help_widget = Paragraph::new(help_text)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    f.render_widget(help_widget, top_chunks[1]);

    // Sağ: Çalan şarkı durumu
    let playback_info = match (&app.current_track, &app.playback_state) {
        (Some(track), state) => {
            let filename = track
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown");
            let status_icon = match state {
                crate::ui::app::PlaybackState::Playing => "▶️",
                crate::ui::app::PlaybackState::Paused => "⏸️",
                crate::ui::app::PlaybackState::Stopped => "⏹️",
            };
            format!("{} {}", status_icon, filename)
        }
        (None, _) => "🎵 No track playing".to_string(),
    };

    let playback_widget = Paragraph::new(playback_info)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Right);

    f.render_widget(playback_widget, top_chunks[2]);
}



