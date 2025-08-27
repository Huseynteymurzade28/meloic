use crate::ui::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Yardım ekranını çiz
pub fn draw_help_screen(f: &mut Frame, _app: &App) {
    let area = centered_rect(70, 80, f.size());

    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "🎵 MELOIC - Music Player Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        // Navigation bölümü
        Line::from(vec![Span::styled(
            "📍 NAVIGATION",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/↓ or j/k     - Move up/down in track list"),
        Line::from("  Page Up/Down   - Jump 10 tracks"),
        Line::from("  Home/End       - Go to first/last track"),
        Line::from("  g/G            - Go to top/bottom (vim style)"),
        Line::from(""),
        // Playback bölümü
        Line::from(vec![Span::styled(
            "🎮 PLAYBACK CONTROLS",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Enter/Space    - Play selected track"),
        Line::from("  p              - Pause/Resume current track"),
        Line::from("  x              - Stop playback completely"),
        Line::from("  s              - Shuffle (random track selection)"),
        Line::from(""),
        // Features bölümü
        Line::from(vec![Span::styled(
            "✨ FEATURES",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  r or F5        - Refresh music library"),
        Line::from("  ?/h or F1      - Show/hide this help screen"),
        Line::from(""),
        // System bölümü
        Line::from(vec![Span::styled(
            "⚙️  SYSTEM",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from("  q or Esc       - Quit application"),
        Line::from(""),
        // Supported formats
        Line::from(vec![Span::styled(
            "🎧 SUPPORTED FORMATS",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  MP3, FLAC, WAV, OGG, M4A, AAC"),
        Line::from(""),
        // Footer
        Line::from(vec![Span::styled(
            "💡 TIP: Meloic scans 'music' folder or current directory for audio files",
            Style::default().fg(Color::Gray),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to close this help screen",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let help_widget = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🆘 Help & Controls ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);

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
