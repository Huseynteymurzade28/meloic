use crate::ui::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn draw_help_screen(f: &mut Frame, app: &App) {
    let theme = app.theme_mode.palette();
    let area = centered_rect(88, 92, f.size());
    f.render_widget(Clear, area);

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            "  ♪  MELOIC — Keyboard Reference  ♪  ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(theme.accent));
    let outer_inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(2)])
        .split(outer_inner);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let left_block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.muted));
    let left_inner = left_block.inner(cols[0]);
    f.render_widget(left_block, cols[0]);

    let left = Paragraph::new(build_left_column(theme))
        .style(Style::default().fg(theme.text))
        .wrap(Wrap { trim: false });
    f.render_widget(left, left_inner);

    let right = Paragraph::new(build_right_column(theme))
        .style(Style::default().fg(theme.text))
        .wrap(Wrap { trim: false });
    f.render_widget(right, cols[1]);

    let footer_line = Line::from(vec![
        Span::styled("  ♪ ", Style::default().fg(theme.accent)),
        Span::styled(
            "MP3  FLAC  WAV  OGG  M4A  AAC    ",
            Style::default().fg(theme.muted),
        ),
        Span::styled(
            "  Press h, ? or F1 to close  ",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let footer = Paragraph::new(footer_line).alignment(Alignment::Center);
    f.render_widget(footer, rows[1]);
}

fn section(title: &str, theme: crate::ui::app::ThemePalette) -> Line<'static> {
    let title = title.to_string();
    Line::from(vec![
        Span::styled(
            "  ",
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    ])
}

fn divider(theme: crate::ui::app::ThemePalette) -> Line<'static> {
    Line::from(Span::styled(
        "  ─────────────────────────────────────",
        Style::default().fg(theme.muted),
    ))
}

fn key_line(keys: &str, desc: &str, theme: crate::ui::app::ThemePalette) -> Line<'static> {
    let keys = keys.to_string();
    let desc = desc.to_string();
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{:<12}", keys),
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(theme.text)),
    ])
}

fn blank() -> Line<'static> {
    Line::from("")
}

fn build_left_column(theme: crate::ui::app::ThemePalette) -> Vec<Line<'static>> {
    vec![
        blank(),
        section("NAVIGATION", theme),
        divider(theme),
        key_line("↑ ↓  j k", "Move selection up / down", theme),
        key_line("PgUp / PgDn", "Jump 10 entries", theme),
        key_line("Ctrl+u / Ctrl+d", "Jump 10 (vim style)", theme),
        key_line("g  /  G", "First / last entry", theme),
        key_line("Enter  Space", "Play highlighted track", theme),
        key_line("/", "Live search — Esc to clear", theme),
        blank(),
        section("VOLUME", theme),
        divider(theme),
        key_line("+ / =", "Volume up 5%", theme),
        key_line("- / _", "Volume down 5%", theme),
        key_line("0", "Mute / unmute", theme),
        blank(),
        section("QUEUE & PLAYLISTS", theme),
        divider(theme),
        key_line("a", "Add selected track to queue", theme),
        key_line("A", "Add all filtered tracks", theme),
        key_line("Q", "Open queue manager popup", theme),
        key_line("S  /  L", "Save / load queue (M3U)", theme),
        key_line("d", "Remove track (inside popup)", theme),
        key_line("c", "Clear queue (inside popup)", theme),
        blank(),
    ]
}

fn build_right_column(theme: crate::ui::app::ThemePalette) -> Vec<Line<'static>> {
    vec![
        blank(),
        section("PLAYBACK", theme),
        divider(theme),
        key_line("p", "Pause / resume", theme),
        key_line("x", "Stop playback", theme),
        key_line("n  /  b", "Next / previous track", theme),
        key_line("s", "Shuffle (random track)", theme),
        key_line("m", "Cycle repeat: off → all → one", theme),
        blank(),
        section("SEEK", theme),
        divider(theme),
        key_line("→  /  ←", "Seek ±5 seconds", theme),
        key_line("Shift+→  ←", "Seek ±30 seconds", theme),
        blank(),
        section("LIBRARY & DISPLAY", theme),
        divider(theme),
        key_line("r  /  F5", "Rescan music roots", theme),
        key_line("t", "Cycle sort mode", theme),
        key_line("c", "Cycle theme (10 themes)", theme),
        key_line("i", "Track info popup", theme),
        key_line("w", "Toggle waveform panel", theme),
        key_line("?  h  F1", "Toggle this help screen", theme),
        key_line("q  /  Esc", "Quit meloic", theme),
        blank(),
    ]
}

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
