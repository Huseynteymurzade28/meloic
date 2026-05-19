use crate::metadata::{self, TrackMetadata};
use crate::ui::app::{App, PlaybackState, ViewMode, track_display_name, track_folder_name};
use crate::ui::help::draw_help_screen;
use crate::waveform::WaveformState;
use image::GenericImageView;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Chart, Clear, Dataset, Gauge, GraphType, LineGauge, List, ListItem,
        Paragraph, Wrap,
    },
};
use std::fs;
use std::path::Path;

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    match app.current_view {
        ViewMode::Help => {
            draw_main_screen(f, app);
            draw_help_screen(f, app);
        }
        ViewMode::Queue => {
            draw_main_screen(f, app);
            draw_queue_popup(f, app);
        }
        ViewMode::Info => {
            draw_main_screen(f, app);
            draw_info_popup(f, app);
        }
        ViewMode::Normal => draw_main_screen(f, app),
    }
}

fn draw_main_screen(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(4),
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    draw_header_panel(f, main_chunks[0], app);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    draw_enhanced_music_list(f, body_chunks[0], app);
    draw_sidebar(f, body_chunks[1], app);

    draw_progress_panel(f, main_chunks[2], app);
    draw_status_bar(f, main_chunks[3], app);
}

fn draw_header_panel(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(26),
                Constraint::Min(28),
                Constraint::Length(42),
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
        Span::styled("music dashboard", Style::default().fg(theme.secondary)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)),
    )
    .alignment(Alignment::Center);
    f.render_widget(title, header_chunks[0]);

    let center = Paragraph::new(Line::from(vec![
        Span::styled("Filter ", Style::default().fg(theme.muted)),
        Span::styled(
            if app.search_query.is_empty() {
                "all tracks".to_string()
            } else {
                app.search_query.to_string()
            },
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if app.is_searching {
                "  [searching]"
            } else {
                ""
            },
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

    let vol_label = if app.muted {
        "muted".to_string()
    } else {
        format!("{}%", app.volume_percent())
    };

    let stats_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} shown / {} total  ", app.visible_track_count(), app.total_tracks),
                Style::default().fg(theme.text),
            ),
            Span::styled(
                format!("queue {}", app.queue_len()),
                Style::default().fg(theme.warning),
            ),
        ]),
        Line::from(vec![
            Span::styled("sort ", Style::default().fg(theme.muted)),
            Span::styled(app.sort_mode.label(), Style::default().fg(theme.secondary)),
            Span::styled("  repeat ", Style::default().fg(theme.muted)),
            Span::styled(app.repeat_mode.label(), Style::default().fg(theme.secondary)),
            Span::styled("  vol ", Style::default().fg(theme.muted)),
            Span::styled(
                vol_label,
                Style::default()
                    .fg(if app.muted { theme.warning } else { theme.success })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("theme ", Style::default().fg(theme.muted)),
            Span::styled("● ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(app.theme_mode.label(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("  roots ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{}", app.scan_roots.len()),
                Style::default().fg(theme.text),
            ),
        ]),
    ];

    let stats = Paragraph::new(stats_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Session ")
                .border_style(Style::default().fg(theme.success)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(stats, header_chunks[2]);
}

fn draw_enhanced_music_list(f: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme_mode.palette();
    if app.is_empty() {
        let empty_msg =
            "No tracks match the current view.\n\nAdd audio files or clear the active filter with Esc.";

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
            let extension = track_extension(path);

            let marker = match (is_playing, is_selected) {
                (true, _) => " ▶ ",
                (false, true) => " › ",
                _ => "   ",
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
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let meta = app.metadata_for(path);
            let artist = meta.map(|m| m.display_artist().to_string()).unwrap_or_default();
            let duration_label = meta
                .and_then(|m| m.duration)
                .map(metadata::format_duration)
                .unwrap_or_default();

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{:3}. ", display_idx + 1), number_style),
                    Span::styled(
                        format!("{} ", marker),
                        Style::default().fg(if is_playing {
                            theme.success
                        } else if is_selected {
                            theme.accent
                        } else {
                            theme.muted
                        }),
                    ),
                    Span::styled(filename, filename_style),
                ]),
                Line::from(vec![
                    Span::styled(parent, Style::default().fg(theme.muted)),
                    Span::styled("  |  ", Style::default().fg(theme.muted)),
                    Span::styled(extension, Style::default().fg(theme.secondary)),
                    Span::styled("  |  ", Style::default().fg(theme.muted)),
                    Span::styled(artist, Style::default().fg(theme.accent)),
                    Span::styled("  ", Style::default()),
                    Span::styled(duration_label, Style::default().fg(theme.warning)),
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
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Min(6),
            Constraint::Length(7),
        ])
        .split(area);

    draw_now_playing(f, chunks[0], app);
    draw_queue_preview(f, chunks[1], app);
    draw_waveform_panel(f, chunks[2], app);
    draw_quick_keys(f, chunks[3], app);
}

fn draw_now_playing(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Now Playing ")
        .border_style(Style::default().fg(theme.success));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Resolve album art bytes (current track preferred, then selected)
    let selected_path = app.get_selected();
    let art_bytes: Option<&[u8]> = app
        .current_track
        .as_deref()
        .and_then(|t| app.album_art_for(t))
        .or_else(|| selected_path.as_deref().and_then(|t| app.album_art_for(t)));

    let art_width = if art_bytes.is_some() && inner.width > 30 && inner.height > 2 {
        let candidate = (inner.height * 2).min(inner.width / 2).min(40);
        candidate
    } else {
        0
    };

    if art_width > 0 {
        let splits = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(art_width), Constraint::Length(1), Constraint::Min(5)])
            .split(inner);

        if let Some(data) = art_bytes {
            draw_halfblock_art(f, splits[0], data);
        }

        draw_track_info_panel(f, splits[2], app, theme);
    } else {
        draw_track_info_panel(f, inner, app, theme);
    }
}

fn draw_halfblock_art(f: &mut Frame, area: Rect, data: &[u8]) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let Ok(img) = image::load_from_memory(data) else {
        return;
    };
    let cols = area.width as u32;
    let rows = area.height as u32;
    let pixel_rows = rows * 2;
    let img = img.resize_to_fill(cols, pixel_rows, image::imageops::FilterType::Triangle);
    let (actual_w, _) = img.dimensions();

    let mut lines: Vec<Line<'static>> = Vec::new();
    for row in 0..rows {
        let mut spans = Vec::new();
        for col in 0..actual_w {
            let top = img.get_pixel(col, row * 2);
            let bot_y = row * 2 + 1;
            let bot = img.get_pixel(col, bot_y.min(pixel_rows - 1));
            let fg = Color::Rgb(top[0], top[1], top[2]);
            let bg = Color::Rgb(bot[0], bot[1], bot[2]);
            spans.push(Span::styled("▀", Style::default().fg(fg).bg(bg)));
        }
        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

fn draw_track_info_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: crate::ui::app::ThemePalette,
) {
    let lines = if let Some(track) = &app.current_track {
        let status_icon = match app.playback_state {
            PlaybackState::Playing => "▶",
            PlaybackState::Paused => "⏸",
            PlaybackState::Stopped => "⏹",
        };
        let status_color = match app.playback_state {
            PlaybackState::Playing => theme.success,
            PlaybackState::Paused => theme.warning,
            PlaybackState::Stopped => theme.muted,
        };
        let meta = app.metadata_for(track);
        let title = meta
            .and_then(|m| m.title.clone())
            .unwrap_or_else(|| track_display_name(track));
        let artist = meta
            .map(|m| m.display_artist().to_string())
            .unwrap_or_else(|| "Unknown artist".to_string());
        let album = meta
            .map(|m| m.display_album().to_string())
            .unwrap_or_else(|| "Unknown album".to_string());
        let duration = meta
            .and_then(|m| m.duration)
            .map(metadata::format_duration)
            .unwrap_or_default();

        vec![
            Line::from(vec![
                Span::styled(
                    format!(" {} ", status_icon),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("vol ", Style::default().fg(theme.muted)),
                Span::styled(
                    if app.muted {
                        "MUTE".to_string()
                    } else {
                        format!("{}%", app.volume_percent())
                    },
                    Style::default()
                        .fg(if app.muted { theme.warning } else { theme.secondary })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                format!(" {}", title),
                Style::default()
                    .fg(theme.text)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled(" by ", Style::default().fg(theme.muted)),
                Span::styled(artist, Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled(" on ", Style::default().fg(theme.muted)),
                Span::styled(album, Style::default().fg(theme.secondary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(track_extension(track), Style::default().fg(theme.warning)),
                Span::styled("  ", Style::default().fg(theme.muted)),
                Span::styled(duration, Style::default().fg(theme.muted)),
            ]),
        ]
    } else {
        let selected_art = app.get_selected();
        let has_art = selected_art
            .as_deref()
            .and_then(|t| app.album_art_for(t))
            .is_some();
        vec![
            Line::from(Span::styled(
                " Nothing playing",
                Style::default().fg(theme.muted),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Press Enter to play",
                Style::default().fg(theme.text),
            )),
            Line::from(""),
            Line::from(Span::styled(
                if has_art { " Album art ready" } else { " No album art" },
                Style::default().fg(theme.muted),
            )),
            Line::from(Span::styled(
                format!(" Vol {}%", app.volume_percent()),
                Style::default().fg(theme.secondary),
            )),
        ]
    };

    let widget = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(widget, area);
}

fn draw_queue_preview(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let title = format!(" Queue ({}) ", app.queue_len());

    let lines: Vec<Line> = if app.queue.is_empty() {
        vec![
            Line::from(Span::styled(
                "Queue is empty",
                Style::default().fg(theme.muted),
            )),
            Line::from(Span::styled(
                "Press 'a' to add the selected track",
                Style::default().fg(theme.muted),
            )),
            Line::from(Span::styled(
                "Press 'A' to enqueue all filtered tracks",
                Style::default().fg(theme.muted),
            )),
            Line::from(Span::styled(
                "Press 'Q' to open the queue manager",
                Style::default().fg(theme.muted),
            )),
        ]
    } else {
        let mut lines = Vec::new();
        for (i, track) in app.queue_peek(5).iter().enumerate() {
            let name = track_display_name(track);
            lines.push(Line::from(vec![
                Span::styled(format!("{}. ", i + 1), Style::default().fg(theme.muted)),
                Span::styled(name, Style::default().fg(theme.text)),
            ]));
        }
        if app.queue_len() > 5 {
            lines.push(Line::from(Span::styled(
                format!("+ {} more...", app.queue_len() - 5),
                Style::default().fg(theme.muted),
            )));
        }
        lines
    };

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(theme.warning)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(widget, area);
}

fn draw_waveform_panel(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Waveform ")
        .border_style(Style::default().fg(theme.accent));

    let Some(track) = app.get_selected() else {
        let widget = Paragraph::new("No track selected")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(widget, area);
        return;
    };

    if !app.is_waveform_enabled_for(&track) {
        let widget = Paragraph::new(vec![
            Line::from(Span::styled(
                "Waveform hidden",
                Style::default().fg(theme.warning),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press w to enable the graph again.",
                Style::default().fg(theme.muted),
            )),
        ])
        .block(block)
        .wrap(Wrap { trim: true });
        f.render_widget(widget, area);
        return;
    }

    let state = app.waveform_state_for(&track);
    let samples = match state {
        Some(WaveformState::Ready(data)) => data.clone(),
        Some(WaveformState::Loading) => {
            let widget = Paragraph::new("Decoding waveform...")
                .block(block)
                .style(Style::default().fg(theme.muted))
                .alignment(Alignment::Center);
            f.render_widget(widget, area);
            return;
        }
        Some(WaveformState::Failed) => {
            let widget = Paragraph::new("Failed to decode audio for waveform")
                .block(block)
                .style(Style::default().fg(theme.warning))
                .alignment(Alignment::Center);
            f.render_widget(widget, area);
            return;
        }
        None => {
            let widget = Paragraph::new("Waveform not requested yet")
                .block(block)
                .style(Style::default().fg(theme.muted))
                .alignment(Alignment::Center);
            f.render_widget(widget, area);
            return;
        }
    };

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 8 || inner.height < 4 {
        let compact = Paragraph::new("Resize to view graph")
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center);
        f.render_widget(compact, inner);
        return;
    }

    let n = samples.len() as f64;
    let positive: Vec<(f64, f64)> = samples
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v as f64))
        .collect();
    let negative: Vec<(f64, f64)> = samples
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, -(*v as f64)))
        .collect();

    let is_current = app.current_track.as_ref() == Some(&track);
    let progress_line: Vec<(f64, f64)> = if is_current {
        if let Some(total) = app.current_total_duration() {
            let pos = app.current_position();
            let frac = if total.as_secs_f64() > 0.0 {
                (pos.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let x = frac * n;
            vec![(x, -1.1), (x, 1.1)]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let mut datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.success))
            .data(&positive),
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.secondary))
            .data(&negative),
    ];
    if !progress_line.is_empty() {
        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Block)
                .graph_type(GraphType::Line)
                .style(
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
                .data(&progress_line),
        );
    }

    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, n.max(1.0)]))
        .y_axis(Axis::default().bounds([-1.1, 1.1]));

    f.render_widget(chart, inner);
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
            Span::styled("← →", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" seek  "),
            Span::styled("+ -", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" vol  "),
            Span::styled("0", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" mute"),
        ]),
        Line::from(vec![
            Span::styled("a", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" queue+  "),
            Span::styled("Q", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" queue  "),
            Span::styled("i", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" info"),
        ]),
        Line::from(vec![
            Span::styled("S/L", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" save/load  "),
            Span::styled("h", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" help"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(app.status_message.clone(), status_style)]),
    ];

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Controls ")
                .border_style(Style::default().fg(theme.accent)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(widget, area);
}

fn draw_progress_panel(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(18)])
        .split(area);

    let (ratio, label) = match (&app.current_track, app.current_total_duration()) {
        (Some(_), Some(total)) => {
            let pos = app.current_position();
            let r = if total.as_secs_f64() > 0.0 {
                (pos.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (
                r,
                format!(
                    "{} / {}",
                    metadata::format_duration(pos),
                    metadata::format_duration(total)
                ),
            )
        }
        (Some(_), None) => {
            let pos = app.current_position();
            (0.0, format!("{} / ?", metadata::format_duration(pos)))
        }
        (None, _) => (0.0, "—".to_string()),
    };

    let gauge = LineGauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Progress ")
                .border_style(Style::default().fg(theme.success)),
        )
        .filled_style(Style::default().fg(theme.success).bg(theme.selection_bg))
        .ratio(ratio)
        .label(label);
    f.render_widget(gauge, chunks[0]);

    let vol_ratio = if app.muted {
        0.0
    } else {
        app.volume_percent() as f64 / 100.0
    };
    let vol_label = if app.muted {
        "muted".to_string()
    } else {
        format!("{}%", app.volume_percent())
    };
    let vol_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Volume ")
                .border_style(Style::default().fg(theme.warning)),
        )
        .gauge_style(
            Style::default()
                .fg(if app.muted { theme.muted } else { theme.warning })
                .bg(theme.selection_bg)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(vol_ratio)
        .label(vol_label);
    f.render_widget(vol_gauge, chunks[1]);
}

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
        format!(
            "Track {}/{}",
            app.selected_track_number(),
            app.visible_track_count()
        )
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
        "Enter play  ← → seek  + - vol  a queue  Q manage  i info  h help  q quit"
    };
    let help_widget = Paragraph::new(help_text)
        .style(
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(help_widget, top_chunks[1]);

    let playback_info = match (&app.current_track, &app.playback_state) {
        (Some(track), state) => {
            let filename = track_display_name(track);
            let status_icon = match state {
                PlaybackState::Playing => ">",
                PlaybackState::Paused => "||",
                PlaybackState::Stopped => "[]",
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

fn draw_queue_popup(f: &mut Frame, app: &mut App) {
    let theme = app.theme_mode.palette();
    let area = centered_rect(70, 70, f.size());
    f.render_widget(Clear, area);

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Queue Manager ({}) ", app.queue_len()))
        .border_style(Style::default().fg(theme.warning));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(4)])
        .split(inner);

    if app.queue.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from("Queue is empty"),
            Line::from(""),
            Line::from("Close this popup (Esc / Q), highlight a track in the library,"),
            Line::from("then press 'a' to enqueue it or 'A' to enqueue all filtered tracks."),
        ])
        .style(Style::default().fg(theme.muted))
        .alignment(Alignment::Center);
        f.render_widget(empty, chunks[0]);
    } else {
        let items: Vec<ListItem> = app
            .queue
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let meta = app.metadata_for(path);
                let title = meta
                    .and_then(|m| m.title.clone())
                    .unwrap_or_else(|| track_display_name(path));
                let artist = meta
                    .map(|m| m.display_artist().to_string())
                    .unwrap_or_default();
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{:3}. ", i + 1),
                            Style::default().fg(theme.muted),
                        ),
                        Span::styled(
                            title,
                            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("     ", Style::default()),
                        Span::styled(artist, Style::default().fg(theme.accent)),
                        Span::styled("  ", Style::default()),
                        Span::styled(
                            track_folder_name(path),
                            Style::default().fg(theme.muted),
                        ),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(theme.selection_bg)
                    .fg(theme.text)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        f.render_stateful_widget(list, chunks[0], &mut app.queue_state);
    }

    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("j/k ↑/↓", Style::default().fg(theme.warning)),
            Span::raw(" navigate    "),
            Span::styled("Enter", Style::default().fg(theme.warning)),
            Span::raw(" play this item (drops earlier items)    "),
            Span::styled("d", Style::default().fg(theme.warning)),
            Span::raw(" remove"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(theme.warning)),
            Span::raw(" clear queue    "),
            Span::styled("Esc / Q", Style::default().fg(theme.warning)),
            Span::raw(" close popup"),
        ]),
        Line::from(Span::styled(
            "Use S in the main view to save this queue to ~/.config/meloic/queue.m3u, L to reload it.",
            Style::default().fg(theme.muted),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.muted)),
    )
    .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[1]);
}

fn draw_info_popup(f: &mut Frame, app: &App) {
    let theme = app.theme_mode.palette();
    let area = centered_rect(60, 70, f.size());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Track Info ")
        .border_style(Style::default().fg(theme.secondary));

    let Some(track) = app.get_selected() else {
        let widget = Paragraph::new("No track selected").block(block);
        f.render_widget(widget, area);
        return;
    };

    let meta = app.metadata_for(&track);
    let unknown = TrackMetadata::default();
    let m = meta.unwrap_or(&unknown);

    let duration = m
        .duration
        .map(metadata::format_duration)
        .unwrap_or_else(|| "—".to_string());
    let sample_rate = m
        .sample_rate
        .map(|s| format!("{} Hz", s))
        .unwrap_or_else(|| "—".to_string());
    let bit_depth = m
        .bit_depth
        .map(|b| format!("{} bit", b))
        .unwrap_or_else(|| "—".to_string());
    let channels = m
        .channels
        .map(|c| match c {
            1 => "mono".to_string(),
            2 => "stereo".to_string(),
            n => format!("{} ch", n),
        })
        .unwrap_or_else(|| "—".to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled("Title  ", Style::default().fg(theme.muted)),
            Span::styled(
                m.display_title(&track_display_name(&track)).to_string(),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Artist ", Style::default().fg(theme.muted)),
            Span::styled(m.display_artist().to_string(), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("Album  ", Style::default().fg(theme.muted)),
            Span::styled(m.display_album().to_string(), Style::default().fg(theme.secondary)),
        ]),
        Line::from(vec![
            Span::styled("Year   ", Style::default().fg(theme.muted)),
            Span::styled(
                m.year.map(|y| y.to_string()).unwrap_or_else(|| "—".into()),
                Style::default().fg(theme.text),
            ),
            Span::styled("    Track ", Style::default().fg(theme.muted)),
            Span::styled(
                m.track_number.map(|t| t.to_string()).unwrap_or_else(|| "—".into()),
                Style::default().fg(theme.text),
            ),
        ]),
        Line::from(vec![
            Span::styled("Genre  ", Style::default().fg(theme.muted)),
            Span::styled(
                m.genre.clone().unwrap_or_else(|| "—".into()),
                Style::default().fg(theme.warning),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "AUDIO PROPERTIES",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Duration   ", Style::default().fg(theme.muted)),
            Span::styled(duration, Style::default().fg(theme.success)),
        ]),
        Line::from(vec![
            Span::styled("Sample rate ", Style::default().fg(theme.muted)),
            Span::styled(sample_rate, Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled("Bit depth   ", Style::default().fg(theme.muted)),
            Span::styled(bit_depth, Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled("Channels    ", Style::default().fg(theme.muted)),
            Span::styled(channels, Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled("Format      ", Style::default().fg(theme.muted)),
            Span::styled(track_extension(&track), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("Size        ", Style::default().fg(theme.muted)),
            Span::styled(human_file_size(&track), Style::default().fg(theme.text)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "PATH",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(Span::styled(
            track.to_string_lossy().to_string(),
            Style::default().fg(theme.muted),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press i or Esc to close",
            Style::default().fg(theme.muted),
        )),
    ];

    let widget = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(widget, area);
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

