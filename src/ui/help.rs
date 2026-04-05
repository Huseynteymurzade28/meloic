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
            "MELOIC - Music Player Help",
            Style::default()
                .fg(Color::Rgb(120, 220, 255))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "NAVIGATION",
            Style::default()
                .fg(Color::Rgb(255, 220, 120))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/↓ or j/k     - Move up/down in track list"),
        Line::from("  Page Up/Down   - Jump 10 tracks"),
        Line::from("  Home/End       - Go to first/last track"),
        Line::from("  g/G            - Go to top/bottom (vim style)"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "PLAYBACK",
            Style::default()
                .fg(Color::Rgb(120, 220, 170))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Enter/Space    - Play selected track"),
        Line::from("  p              - Pause/Resume current track"),
        Line::from("  x              - Stop playback completely"),
        Line::from("  s              - Shuffle (random track selection)"),
        Line::from("  n / b          - Play next / previous track"),
        Line::from("  m              - Cycle repeat mode (off/all/one)"),
        Line::from("  w              - Toggle graph panel for selected track"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "LIBRARY",
            Style::default()
                .fg(Color::Rgb(255, 120, 198))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  r or F5        - Refresh music library"),
        Line::from("  /              - Start live search/filter mode"),
        Line::from("  t              - Cycle sort mode"),
        Line::from("  c              - Cycle UI theme"),
        Line::from("  ?/h or F1      - Show/hide this help screen"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "SEARCH MODE",
            Style::default().fg(Color::Rgb(255, 160, 120))
            .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Type           - Filter tracks instantly"),
        Line::from("  Backspace      - Remove last character"),
        Line::from("  Enter          - Keep current filter and exit search"),
        Line::from("  Esc            - Clear filter and exit search"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "SYSTEM",
            Style::default().fg(Color::Rgb(255, 160, 120))
            .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  q or Esc       - Quit application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "SUPPORTED FORMATS",
            Style::default()
                .fg(Color::Rgb(100, 150, 255))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  MP3, FLAC, WAV, OGG, M4A, AAC"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "TIP: Meloic scans ~/Music, /Music, local Music/music folders, and the current directory",
            Style::default().fg(Color::Rgb(150, 150, 150)),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press h, ? or F1 to close this help screen",
            Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )]),
    ];

    let help_widget = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help & Controls ")
                .title_style(
                    Style::default()
                        .fg(Color::Rgb(120, 220, 255))
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(Color::Rgb(255, 120, 198))),
        )
        .style(Style::default().fg(Color::Rgb(255, 255, 255)))
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
