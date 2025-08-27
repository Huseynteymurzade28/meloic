use crate::ui::app::{App, ViewMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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
                Constraint::Min(10),   // Ana liste
                Constraint::Length(2), // Alt durum çubuğu
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
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30), // Sol: Track info
            Constraint::Min(20),    // Orta: Çalan şarkı
            Constraint::Length(50), // Sağ: Kontroller
        ])
        .split(area);

    // Sol: Seçili track bilgisi
    let current_selection = app
        .state
        .selected()
        .map(|i| format!("Track {}/{}", i + 1, app.filtered_items.len()))
        .unwrap_or_else(|| "No selection".to_string());

    let selection_widget = Paragraph::new(current_selection)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::TOP | Borders::RIGHT));

    f.render_widget(selection_widget, chunks[0]);

    // Orta: Çalan şarkı durumu
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
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::RIGHT));

    f.render_widget(playback_widget, chunks[1]);

    // Sağ: Kontroller
    let controls_text = " Enter: Play • p: Pause/Resume • x: Stop • q: Quit";
    let controls_widget = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));

    f.render_widget(controls_widget, chunks[2]);
}

/// Yardım ekranı
fn draw_help_screen(f: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, f.size());

    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "🎵 MELOIC - Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/↓ or j/k     - Move up/down"),
        Line::from("  Page Up/Down   - Move 10 items"),
        Line::from("  Home/End or g/G - Go to top/bottom"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Playback:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Enter or Space  - Play selected track"),
        Line::from("  p              - Pause/Resume playback"),
        Line::from("  x              - Stop playback completely"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Features:",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  s              - Shuffle (random selection)"),
        Line::from("  r or F5        - Refresh library"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Controls:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from("  q or Esc       - Quit"),
        Line::from("  ? or F1        - Toggle this help"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let help_widget = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(help_widget, area);
}

/// Ortalanmış rectangle hesapla
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
