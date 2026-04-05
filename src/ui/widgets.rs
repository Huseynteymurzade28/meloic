use crate::ui::app::{App, PlaybackState, ViewMode, track_display_name, track_folder_name};
use crate::ui::help::draw_help_screen;
use ratatui::{
    symbols,
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph, Wrap},
};
use std::fs;
use std::path::Path;

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
                Constraint::Length(4),
                Constraint::Min(10),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Üst bilgi paneli çiz
    draw_header_panel(f, main_chunks[0], app);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    draw_enhanced_music_list(f, body_chunks[0], app);
    draw_sidebar(f, body_chunks[1], app);

    // Alt durum çubuğunu çiz
    draw_status_bar(f, main_chunks[2], app);
}

/// Gelişmiş başlık paneli
fn draw_header_panel(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(26),
                Constraint::Min(28),
                Constraint::Length(40),
            ]
            .as_ref(),
        )
        .split(area);

    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "meloic ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "music dashboard",
            Style::default()
                .fg(theme.secondary),
        ),
    ])])
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent)))
    .alignment(Alignment::Center);
    f.render_widget(title, header_chunks[0]);

    let center = Paragraph::new(Line::from(vec![
        Span::styled("Filter ", Style::default().fg(theme.muted)),
        Span::styled(
            if app.search_query.is_empty() {
                "all tracks".to_string()
            } else {
                format!("{}", app.search_query)
            },
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if app.is_searching { "  [searching]" } else { "" },
            Style::default().fg(theme.warning),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Library View ")
            .border_style(Style::default().fg(theme.muted)),
    );
    f.render_widget(center, header_chunks[1]);

    let stats_text = format!(
        "{} shown / {} total\nsort {}  |  repeat {}  |  theme {}\nroots {}",
        app.visible_track_count(),
        app.total_tracks,
        app.sort_mode.label(),
        app.repeat_mode.label(),
        app.theme_mode.label(),
        app.scan_roots.len()
    );

    let stats = Paragraph::new(vec![Line::from(vec![Span::styled(
        stats_text,
        Style::default().fg(theme.text),
    )])])
    .block(Block::default()
        .borders(Borders::ALL)
        .title("Session")
        .border_style(Style::default().fg(theme.success)))
    .alignment(Alignment::Left)
    .wrap(Wrap { trim: true });
    f.render_widget(stats, header_chunks[2]);
}

/// Gelişmiş müzik listesi
fn draw_enhanced_music_list(f: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme_mode.palette();
    if app.is_empty() {
        let empty_msg = "No tracks match the current view.\n\nAdd audio files or clear the active filter with Esc.";

        let empty_widget = Paragraph::new(empty_msg)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Library ")
                    .style(Style::default().fg(theme.accent)),
            )
            .style(Style::default().fg(theme.warning))
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
            let filename = track_display_name(path);
            let parent = track_folder_name(path);
            let is_selected = Some(display_idx) == selected;
            let is_playing = app.current_track.as_ref() == Some(path);
            let waveform_enabled = app.is_waveform_enabled_for(path);
            let extension = track_extension(path);

            let marker = match (is_playing, is_selected) {
                (true, _) => "PLAY",
                (false, true) => "SELE",
                _ => "    ",
            };

            let number_style = if is_selected {
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            };

            let filename_style = if is_playing {
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(theme.text)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let waveform_badge = if waveform_enabled { "graph on" } else { "graph off" };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{:3}. ", display_idx + 1), number_style),
                    Span::styled(
                        format!("[{}] ", marker),
                        Style::default().fg(if is_playing { theme.success } else { theme.muted }),
                    ),
                    Span::styled(filename, filename_style),
                ]),
                Line::from(vec![
                    Span::styled(parent, Style::default().fg(theme.muted)),
                    Span::styled("  |  ", Style::default().fg(theme.muted)),
                    Span::styled(extension, Style::default().fg(theme.secondary)),
                    Span::styled("  |  ", Style::default().fg(theme.muted)),
                    Span::styled(waveform_badge, Style::default().fg(theme.accent)),
                ]),
            ])
        })
        .collect();

    let list_title = if app.search_query.is_empty() {
        " Library ".to_string()
    } else {
        format!(" Library / {} ", app.search_query)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(theme.secondary)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.selection_bg)
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.state);
}

fn draw_sidebar(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Min(10),
            Constraint::Min(7),
        ])
        .split(area);

    draw_now_playing(f, chunks[0], app);
    draw_selection_details(f, chunks[1], app);
    draw_waveform_panel(f, chunks[2], app);
    draw_quick_keys(f, chunks[3], app);
}

fn draw_now_playing(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let lines = if let Some(track) = &app.current_track {
        let status = match app.playback_state {
            PlaybackState::Playing => "playing",
            PlaybackState::Paused => "paused",
            PlaybackState::Stopped => "stopped",
        };

        vec![
            Line::from(vec![
                Span::styled("Status ", Style::default().fg(theme.muted)),
                Span::styled(status, Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(track_display_name(track)),
            Line::from(Span::styled(track_folder_name(track), Style::default().fg(theme.muted))),
            Line::from(vec![
                Span::styled("Mode ", Style::default().fg(theme.muted)),
                Span::styled(app.repeat_mode.label(), Style::default().fg(theme.secondary)),
                Span::styled("  |  ", Style::default().fg(theme.muted)),
                Span::styled(track_extension(track), Style::default().fg(theme.accent)),
            ]),
            Line::from(Span::styled(human_file_size(track), Style::default().fg(theme.text))),
        ]
    } else {
        vec![
            Line::from("Nothing playing"),
            Line::from(Span::styled("Press Enter to start the selected track", Style::default().fg(theme.muted))),
            Line::from(Span::styled(format!("Roots: {}", app.scan_roots_summary()), Style::default().fg(theme.muted))),
        ]
    };

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Now Playing ").border_style(Style::default().fg(theme.success)))
        .wrap(Wrap { trim: true });
    f.render_widget(widget, area);
}

fn draw_selection_details(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let lines = if let Some(selected) = app.get_selected() {
        let wave_status = if app.is_waveform_enabled_for(&selected) {
            "graph visible"
        } else {
            "graph hidden"
        };

        vec![
            Line::from(vec![
                Span::styled("Track ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{}/{}", app.selected_track_number(), app.visible_track_count()),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(track_display_name(&selected)),
            Line::from(Span::styled(track_folder_name(&selected), Style::default().fg(theme.muted))),
            Line::from(vec![
                Span::styled("Format ", Style::default().fg(theme.muted)),
                Span::styled(track_extension(&selected), Style::default().fg(theme.secondary)),
                Span::styled("  |  Size ", Style::default().fg(theme.muted)),
                Span::styled(human_file_size(&selected), Style::default().fg(theme.text)),
            ]),
            Line::from(Span::styled(format!("Waveform {}", wave_status), Style::default().fg(theme.secondary))),
            Line::from(Span::styled(format!("Scanned roots: {}", app.scan_roots_summary()), Style::default().fg(theme.muted))),
            Line::from(Span::styled(selected.to_string_lossy().to_string(), Style::default().fg(theme.muted))),
        ]
    } else {
        vec![Line::from("No track selected")]
    };

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Selection ").border_style(Style::default().fg(theme.secondary)))
        .wrap(Wrap { trim: true });
    f.render_widget(widget, area);
}

fn draw_waveform_panel(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Signal Graph ")
        .border_style(Style::default().fg(theme.accent));

    if let Some(track) = app.get_selected() {
        if app.is_waveform_enabled_for(&track) {
            let inner = block.inner(area);
            f.render_widget(block, area);

            if inner.width < 8 || inner.height < 6 {
                let compact = Paragraph::new("Resize to view graph")
                    .style(Style::default().fg(theme.muted))
                    .alignment(Alignment::Center);
                f.render_widget(compact, inner);
                return;
            }

            let graph_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(2)])
                .split(inner);
            let points = app.waveform_points(&track, graph_chunks[0].width.saturating_sub(2) as usize);
            let datasets = vec![
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
                    .data(&points),
                Dataset::default()
                    .marker(symbols::Marker::Dot)
                    .graph_type(GraphType::Scatter)
                    .style(Style::default().fg(theme.accent))
                    .data(&points),
            ];
            let chart = Chart::new(datasets)
                .x_axis(Axis::default().bounds([0.0, points.len() as f64]))
                .y_axis(Axis::default().bounds([-1.1, 1.1]));
            let caption = Paragraph::new("Graph view for the selected track. Press w to hide or show it.")
                .style(Style::default().fg(theme.muted))
                .wrap(Wrap { trim: true });

            f.render_widget(chart, graph_chunks[0]);
            f.render_widget(caption, graph_chunks[1]);
            return;
        }

        let widget = Paragraph::new(vec![
            Line::from(Span::styled("Signal graph hidden", Style::default().fg(theme.warning))),
            Line::from(""),
            Line::from(Span::styled("This track is keeping the visualizer closed.", Style::default().fg(theme.text))),
            Line::from(Span::styled("Press w to enable the graph again.", Style::default().fg(theme.muted))),
        ])
        .block(block)
        .wrap(Wrap { trim: true });
        f.render_widget(widget, area);
        return;
    }

    let widget = Paragraph::new("No track selected")
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(widget, area);
}

fn draw_quick_keys(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let status_style = if app.last_error.is_some() {
        Style::default().fg(Color::Rgb(255, 160, 160))
    } else {
        Style::default().fg(theme.text)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("/", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" search  "),
            Span::styled("n/b", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" next prev"),
        ]),
        Line::from(vec![
            Span::styled("t", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" sort  "),
            Span::styled("m", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" repeat  "),
            Span::styled("c", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" theme"),
        ]),
        Line::from(vec![
            Span::styled("s", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" random"),
            Span::styled("  w", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" graph"),
        ]),
        Line::from(vec![
            Span::styled("p", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" pause  "),
            Span::styled("x", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" stop  "),
            Span::styled("r", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" refresh"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(app.status_message.clone(), status_style)]),
    ];

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Controls ").border_style(Style::default().fg(theme.accent)))
        .wrap(Wrap { trim: true });
    f.render_widget(widget, area);
}

/// Alt durum çubuğu
fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(32),
            Constraint::Percentage(36),
            Constraint::Percentage(32),
        ])
        .split(area);

    let current_selection = if app.visible_track_count() == 0 {
        "No selection".to_string()
    } else {
        format!("Track {}/{}", app.selected_track_number(), app.visible_track_count())
    };

    let selection_widget = Paragraph::new(current_selection).style(
        Style::default()
            .fg(theme.secondary)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(selection_widget, top_chunks[0]);

    let help_text = if app.is_searching {
        "Search: type to filter, Enter keep, Esc clear"
    } else {
        "Enter play  / search  w graph  c theme  h help  q quit"
    };
    let help_widget = Paragraph::new(help_text)
        .style(
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
        .alignment(Alignment::Center);

    f.render_widget(help_widget, top_chunks[1]);

    let playback_info = match (&app.current_track, &app.playback_state) {
        (Some(track), state) => {
            let filename = track_display_name(track);
            let status_icon = match state {
                crate::ui::app::PlaybackState::Playing => ">",
                crate::ui::app::PlaybackState::Paused => "||",
                crate::ui::app::PlaybackState::Stopped => "[]",
            };
            format!("{} {}", status_icon, filename)
        }
        (None, _) => "No track playing".to_string(),
    };

    let playback_widget = Paragraph::new(playback_info)
        .style(
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Right);

    f.render_widget(playback_widget, top_chunks[2]);
}

fn track_extension(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_uppercase())
        .unwrap_or_else(|| "FILE".to_string())
}

fn human_file_size(path: &Path) -> String {
    let Ok(metadata) = fs::metadata(path) else {
        return "unknown size".to_string();
    };

    let bytes = metadata.len() as f64;
    if bytes >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GB", bytes / 1024.0 / 1024.0 / 1024.0)
    } else if bytes >= 1024.0 * 1024.0 {
        format!("{:.1} MB", bytes / 1024.0 / 1024.0)
    } else if bytes >= 1024.0 {
        format!("{:.1} KB", bytes / 1024.0)
    } else {
        format!("{} B", metadata.len())
    }
}
