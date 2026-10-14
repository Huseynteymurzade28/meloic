use crate::metadata::{self, TrackMetadata};
use crate::ui::app::{
    App, LIST_ROW_HEIGHT, LibraryRow, PlaybackState, SortMode, ThemePalette, ViewMode,
    track_display_name, track_folder_name,
};
use crate::ui::cover::CoverSlot;
use crate::ui::help::draw_help_screen;
use crate::waveform::WaveformState;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap,
    },
};
use std::fs;
use std::path::Path;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Below this width the sidebar is hidden and the library takes the full body.
const SIDEBAR_MIN_WIDTH: u16 = 100;
const BAR_GLYPHS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const EQ_FRAMES: [&str; 8] = ["▁▃▅", "▂▅▃", "▃▇▂", "▅▃▆", "▇▂▄", "▅▄▇", "▃▆▅", "▂▇▃"];

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    draw_main_screen(f, app);
    match app.current_view {
        ViewMode::Help => draw_help_screen(f, app),
        ViewMode::Queue => draw_queue_popup(f, app),
        ViewMode::Info => draw_info_popup(f, app),
        ViewMode::Cover => draw_cover_popup(f, app),
        ViewMode::Normal => {}
    }
}

fn draw_main_screen(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(f.size());

    draw_header(f, main_chunks[0], app);

    if main_chunks[1].width >= SIDEBAR_MIN_WIDTH {
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(main_chunks[1]);
        draw_library(f, body_chunks[0], app);
        draw_sidebar(f, body_chunks[1], app);
    } else {
        draw_library(f, main_chunks[1], app);
    }

    draw_player_bar(f, main_chunks[2], app);
    draw_footer(f, main_chunks[3], app);
}

// ---------------------------------------------------------------- helpers

fn panel<'a>(title: impl Into<String>, color: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            format!(" {} ", title.into()),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
}

fn text_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// Truncate `text` to `max` columns, ending with an ellipsis when cut.
fn fit(text: &str, max: usize) -> String {
    if text_width(text) <= max {
        return text.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut width = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if width + w > max - 1 {
            break;
        }
        out.push(ch);
        width += w;
    }
    out.push('…');
    out
}

/// Join left and right spans with padding so the right part hugs `width`.
fn spread<'a>(mut left: Vec<Span<'a>>, right: Vec<Span<'a>>, width: usize) -> Line<'a> {
    let used: usize = left.iter().chain(&right).map(|s| s.width()).sum();
    left.push(Span::raw(" ".repeat(width.saturating_sub(used))));
    left.extend(right);
    Line::from(left)
}

fn eq_glyph(app: &App) -> &'static str {
    match app.playback_state {
        PlaybackState::Playing => {
            let frame = app.started_at.elapsed().as_millis() / 150;
            EQ_FRAMES[frame as usize % EQ_FRAMES.len()]
        }
        PlaybackState::Paused => "▁▁▁",
        PlaybackState::Stopped => "   ",
    }
}

fn track_title(app: &App, path: &Path) -> String {
    app.metadata_for(path)
        .and_then(|m| m.title.clone())
        .unwrap_or_else(|| track_display_name(path))
}

fn track_artist(app: &App, path: &Path) -> Option<String> {
    app.metadata_for(path).and_then(|m| m.artist.clone())
}

fn key_hint<'a>(key: &'a str, label: &'a str, theme: ThemePalette) -> Vec<Span<'a>> {
    vec![
        Span::styled(
            key,
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {label}  "), Style::default().fg(theme.muted)),
    ]
}

// ---------------------------------------------------------------- header

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.muted));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let badge = |icon: &'static str, label: String, active: bool| {
        let color = if active { theme.accent } else { theme.muted };
        vec![
            Span::styled(format!("{icon} "), Style::default().fg(color)),
            Span::styled(
                label,
                Style::default().fg(if active { theme.text } else { theme.muted }),
            ),
            Span::raw("   "),
        ]
    };

    let mut right: Vec<Span> = Vec::new();
    right.extend(badge("⇄", "shuffle".into(), app.shuffle));
    right.extend(badge(
        "↻",
        app.repeat_mode.label().into(),
        app.repeat_mode != crate::ui::app::RepeatMode::Off,
    ));
    right.extend(badge("⇅", app.sort_mode.label().into(), true));
    right.push(Span::styled("◐ ", Style::default().fg(theme.accent)));
    right.push(Span::styled(
        app.theme_mode.label(),
        Style::default().fg(theme.secondary),
    ));
    right.push(Span::raw(" "));

    let mut left = vec![
        Span::styled(
            " ♫ meloic",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(theme.muted)),
    ];

    if app.is_searching || !app.search_query.is_empty() {
        left.push(Span::styled("/ ", Style::default().fg(theme.warning)));
        left.push(Span::styled(
            app.search_query.clone(),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ));
        if app.is_searching {
            left.push(Span::styled("▏", Style::default().fg(theme.accent)));
        }
        left.push(Span::styled(
            format!("  {} / {}", app.visible_track_count(), app.total_tracks),
            Style::default().fg(theme.muted),
        ));
    } else {
        left.push(Span::styled(
            format!("{} tracks", app.total_tracks),
            Style::default().fg(theme.text),
        ));
        left.push(Span::styled(
            "   press / to search",
            Style::default().fg(theme.muted),
        ));
    }
    let pending = app.metadata_loader.pending();
    if pending > 0 {
        left.push(Span::styled(
            format!("   reading tags · {pending} left"),
            Style::default().fg(theme.muted),
        ));
    }

    let width = inner.width as usize;
    let right_width: usize = right.iter().map(|s| s.width()).sum();
    let line = if left.iter().map(|s| s.width()).sum::<usize>() + right_width + 2 <= width {
        spread(left, right, width)
    } else {
        Line::from(left)
    };
    f.render_widget(Paragraph::new(line), inner);
}

// ---------------------------------------------------------------- library

fn draw_library(f: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme_mode.palette();
    let title = if app.search_query.is_empty() {
        "Library".to_string()
    } else {
        format!("Library · {}", app.search_query)
    };
    let block = panel(title, theme.secondary).title(
        ratatui::widgets::block::Title::from(Span::styled(
            format!(
                " {}/{} ",
                app.selected_track_number(),
                app.visible_track_count()
            ),
            Style::default().fg(theme.muted),
        ))
        .alignment(Alignment::Right),
    );
    let inner = block.inner(area);

    if app.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No tracks match the current view",
                Style::default().fg(theme.warning),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Esc clears the filter · r rescans the library",
                Style::default().fg(theme.muted),
            )),
        ])
        .block(block)
        .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    // Load tags around the selection right away; the background loader
    // fills in the rest of the library.
    let visible_tracks = (inner.height / LIST_ROW_HEIGHT) as usize + 1;
    let anchor = app.state.selected().unwrap_or(0);
    app.ensure_metadata_range(
        anchor.saturating_sub(visible_tracks),
        visible_tracks * 2 + 1,
    );

    let offset = scroll_offset(app, inner.height);
    app.list_offset = offset;

    let width = inner.width as usize;
    let selected = app.state.selected();
    let eq = eq_glyph(app);

    // Only build the rows that fit on screen.
    let mut items: Vec<ListItem> = Vec::new();
    let mut used = 0;
    for row in app.rows.iter().skip(offset) {
        if used >= inner.height {
            break;
        }
        used += row.height();
        items.push(match row {
            LibraryRow::Header {
                title,
                detail,
                tracks,
            } => group_header(title, detail, *tracks, width, theme),
            LibraryRow::Track(idx) => {
                let path = &app.filtered_items[*idx].1;
                track_item(app, *idx, path, Some(*idx) == selected, eq, width, theme)
            }
        });
    }

    let mut state = ListState::default();
    state.select(app.selected_row().map(|row| row - offset));
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(theme.selection_bg));
    f.render_stateful_widget(list, area, &mut state);

    let mut hit = app.hit_areas.get();
    hit.list = inner;
    app.hit_areas.set(hit);
}

/// First row to draw so the selection stays visible, keeping the header of
/// the selected track's group on screen when possible.
fn scroll_offset(app: &App, height: u16) -> usize {
    let Some(selected) = app.selected_row() else {
        return 0;
    };
    let rows_height = |from: usize| -> u16 {
        app.rows[from..=selected]
            .iter()
            .map(LibraryRow::height)
            .sum()
    };
    // Everything up to the selection fits: no need to scroll at all.
    if rows_height(0) <= height {
        return 0;
    }

    // Scrolling down: advance until the selected row fits entirely.
    let mut offset = app.list_offset.min(selected);
    while offset < selected && rows_height(offset) > height {
        offset += 1;
    }
    // Never start right below a group header when there is room for it.
    // This also covers rows shifting as tags load in the background.
    while offset > 0
        && matches!(app.rows[offset - 1], LibraryRow::Header { .. })
        && rows_height(offset - 1) <= height
    {
        offset -= 1;
    }
    offset
}

fn group_header<'a>(
    title: &str,
    detail: &str,
    tracks: usize,
    width: usize,
    theme: ThemePalette,
) -> ListItem<'a> {
    let count = format!("{tracks} track{} ", if tracks == 1 { "" } else { "s" });
    let room = width.saturating_sub(text_width(&count) + 4);
    let title = fit(title, room);
    let detail = fit(detail, room.saturating_sub(text_width(&title) + 3));
    let mut left = vec![
        Span::styled("▍ ", Style::default().fg(theme.accent)),
        Span::styled(
            title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    if !detail.is_empty() {
        left.push(Span::styled(
            format!("  {detail}"),
            Style::default().fg(theme.muted),
        ));
    }
    ListItem::new(spread(
        left,
        vec![Span::styled(count, Style::default().fg(theme.muted))],
        width,
    ))
}

fn track_item<'a>(
    app: &App,
    idx: usize,
    path: &Path,
    is_selected: bool,
    eq: &str,
    width: usize,
    theme: ThemePalette,
) -> ListItem<'a> {
    let is_playing = app.current_track.as_deref() == Some(path);
    let meta = app.metadata_for(path);

    // Album view numbers tracks by their tag; other views by list position.
    let number = match (app.sort_mode, meta.and_then(|m| m.track_number)) {
        (SortMode::Album, Some(n)) => n as usize,
        _ => idx + 1,
    };
    let (marker, marker_color) = if is_playing {
        (eq.to_string(), theme.success)
    } else if is_selected {
        ("  ›".to_string(), theme.accent)
    } else {
        (format!("{number:>3}"), theme.muted)
    };

    let duration = meta
        .and_then(|m| m.duration)
        .map(metadata::format_duration)
        .unwrap_or_default();

    let title_style = if is_playing {
        Style::default()
            .fg(theme.success)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let title_room = width.saturating_sub(6 + text_width(&duration) + 1);
    let line1 = spread(
        vec![
            Span::styled(format!("{marker}  "), Style::default().fg(marker_color)),
            Span::styled(fit(&track_title(app, path), title_room), title_style),
        ],
        vec![
            Span::styled(duration, Style::default().fg(theme.muted)),
            Span::raw(" "),
        ],
        width,
    );

    // Skip whatever the group header above already says.
    let artist = track_artist(app, path);
    let album = meta
        .and_then(|m| m.album.clone())
        .unwrap_or_else(|| track_folder_name(path));
    let subtitle = match (app.sort_mode, artist) {
        (SortMode::Artist, _) => album,
        (SortMode::Album, Some(artist)) => artist,
        (SortMode::Album, None) => String::new(),
        (_, Some(artist)) => format!("{artist} · {album}"),
        (_, None) => album,
    };
    let ext = track_extension(path);
    let sub_room = width.saturating_sub(5 + text_width(&ext) + 1);
    let line2 = spread(
        vec![
            Span::raw("     "),
            Span::styled(
                fit(&subtitle, sub_room),
                Style::default().fg(if is_selected {
                    theme.accent
                } else {
                    theme.muted
                }),
            ),
        ],
        vec![
            Span::styled(ext, Style::default().fg(theme.muted)),
            Span::raw(" "),
        ],
        width,
    );

    ListItem::new(vec![line1, line2])
}

// ---------------------------------------------------------------- sidebar

fn draw_sidebar(f: &mut Frame, area: Rect, app: &App) {
    // Give the cover roughly half the sidebar; the waveform takes what is left.
    let now_height = (area.height * 55 / 100).clamp(12, 26);
    let show_queue = area.height >= now_height + 8 + 6;
    let mut constraints = vec![Constraint::Length(now_height)];
    if show_queue {
        constraints.push(Constraint::Length(8));
    }
    constraints.push(Constraint::Min(5));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    draw_now_playing(f, chunks[0], app);
    if show_queue {
        draw_queue_preview(f, chunks[1], app);
    }
    draw_waveform_panel(f, chunks[chunks.len() - 1], app);
}

fn draw_now_playing(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let title = if app.current_track.is_some() {
        "Now Playing"
    } else {
        "Selected"
    };
    let block = panel(title, theme.success).title(
        ratatui::widgets::block::Title::from(Span::styled(
            " v enlarge ",
            Style::default().fg(theme.muted),
        ))
        .alignment(Alignment::Right),
    );
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Current track preferred, otherwise whatever is highlighted.
    let Some(track) = app.current_track.clone().or_else(|| app.get_selected()) else {
        return;
    };

    if inner.width < 30 || inner.height < 3 {
        draw_track_details(f, inner, app, &track, theme);
        return;
    }

    // Square cover on the left (up to ~55% of the width), details beside it.
    let cover_box = Rect {
        width: inner.width * 55 / 100,
        ..inner
    };
    let art = Rect {
        x: inner.x,
        ..app.covers.borrow().square_in(cover_box)
    };
    draw_cover(f, art, app, CoverSlot::Sidebar, &track);

    let info_x = art.x + art.width + 2;
    let info_area = Rect {
        x: info_x,
        width: inner.right().saturating_sub(info_x),
        ..inner
    };
    draw_track_details(f, info_area, app, &track, theme);
}

fn draw_cover(f: &mut Frame, area: Rect, app: &App, slot: CoverSlot, track: &Path) {
    let drawn = app
        .album_art_for(track)
        .is_some_and(|data| app.covers.borrow_mut().render(f, area, slot, track, data));
    if !drawn {
        draw_art_placeholder(f, area, app.theme_mode.palette());
    }
}

fn draw_art_placeholder(f: &mut Frame, area: Rect, theme: ThemePalette) {
    let mid = area.height / 2;
    let lines: Vec<Line> = (0..area.height)
        .map(|row| {
            if row == mid {
                Line::from("♫")
            } else {
                Line::from("")
            }
        })
        .collect();
    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .style(Style::default().bg(theme.selection_bg).fg(theme.muted));
    f.render_widget(widget, area);
}

fn draw_track_details(f: &mut Frame, area: Rect, app: &App, track: &Path, theme: ThemePalette) {
    let width = area.width as usize;
    let meta = app.metadata_for(track);
    let title = track_title(app, track);
    let artist = meta
        .map(|m| m.display_artist().to_string())
        .unwrap_or_else(|| "Unknown artist".to_string());
    let album = meta
        .map(|m| m.display_album().to_string())
        .unwrap_or_else(|| track_folder_name(track));

    let mut tech = vec![track_extension(track)];
    if let Some(rate) = meta.and_then(|m| m.sample_rate) {
        tech.push(format!("{:.1} kHz", rate as f64 / 1000.0));
    }
    if let Some(duration) = meta.and_then(|m| m.duration) {
        tech.push(metadata::format_duration(duration));
    }

    let is_current = app.current_track.as_deref() == Some(track);
    let (state_icon, state_label, state_color) = match (is_current, &app.playback_state) {
        (true, PlaybackState::Playing) => ("▶", "playing", theme.success),
        (true, PlaybackState::Paused) => ("⏸", "paused", theme.warning),
        _ => ("⏎", "press Enter to play", theme.muted),
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            fit(&title, width),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            fit(&artist, width),
            Style::default().fg(theme.accent),
        )),
        Line::from(Span::styled(
            fit(&album, width),
            Style::default().fg(theme.secondary),
        )),
        Line::from(""),
        Line::from(Span::styled(
            fit(&tech.join(" · "), width),
            Style::default().fg(theme.muted),
        )),
    ];
    if let Some(year) = meta.and_then(|m| m.year) {
        let genre = meta.and_then(|m| m.genre.clone());
        let text = match genre {
            Some(genre) => format!("{year} · {genre}"),
            None => year.to_string(),
        };
        lines.push(Line::from(Span::styled(
            fit(&text, width),
            Style::default().fg(theme.muted),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(format!("{state_icon} "), Style::default().fg(state_color)),
        Span::styled(state_label, Style::default().fg(state_color)),
    ]));

    f.render_widget(Paragraph::new(lines), area);
}

fn draw_queue_preview(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let block = panel(format!("Up Next · {}", app.queue_len()), theme.warning);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let width = inner.width as usize;

    let lines: Vec<Line> = if app.queue.is_empty() {
        vec![
            Line::from(Span::styled(
                "Queue is empty",
                Style::default().fg(theme.muted),
            )),
            Line::from(vec![
                Span::styled("a", Style::default().fg(theme.warning)),
                Span::styled(" add selected  ", Style::default().fg(theme.muted)),
                Span::styled("A", Style::default().fg(theme.warning)),
                Span::styled(" add all shown", Style::default().fg(theme.muted)),
            ]),
        ]
    } else {
        let shown = inner.height.saturating_sub(1).max(1) as usize;
        let mut lines: Vec<Line> = app
            .queue_peek(shown)
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let label = match track_artist(app, track) {
                    Some(artist) => format!("{} — {}", track_title(app, track), artist),
                    None => track_title(app, track),
                };
                Line::from(vec![
                    Span::styled(format!("{:>2}  ", i + 1), Style::default().fg(theme.muted)),
                    Span::styled(
                        fit(&label, width.saturating_sub(4)),
                        Style::default().fg(theme.text),
                    ),
                ])
            })
            .collect();
        if app.queue_len() > shown {
            lines.push(Line::from(Span::styled(
                format!("    + {} more · Q to manage", app.queue_len() - shown),
                Style::default().fg(theme.muted),
            )));
        }
        lines
    };

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_waveform_panel(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let block = panel("Waveform", theme.accent);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let track = app.current_track.clone().or_else(|| app.get_selected());
    let message = |text: &str, color: Color| {
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(text.to_string(), Style::default().fg(color))),
        ])
        .alignment(Alignment::Center)
    };

    let Some(track) = track else {
        f.render_widget(message("No track selected", theme.muted), inner);
        return;
    };
    if !app.is_waveform_enabled_for(&track) {
        f.render_widget(message("Hidden · press w to show", theme.muted), inner);
        return;
    }
    let samples = match app.waveform_state_for(&track) {
        Some(WaveformState::Ready(data)) => data,
        Some(WaveformState::Loading) | None => {
            f.render_widget(message("Decoding…", theme.muted), inner);
            return;
        }
        Some(WaveformState::Failed) => {
            f.render_widget(message("Could not decode audio", theme.warning), inner);
            return;
        }
    };
    if inner.width < 4 || inner.height < 1 || samples.is_empty() {
        return;
    }

    let cols = inner.width as usize;
    let rows = inner.height as usize;
    let peak = samples
        .iter()
        .cloned()
        .fold(0.0_f32, f32::max)
        .max(f32::EPSILON);
    let levels: Vec<f32> = (0..cols)
        .map(|c| {
            let start = c * samples.len() / cols;
            let end = ((c + 1) * samples.len() / cols)
                .max(start + 1)
                .min(samples.len());
            samples[start..end].iter().cloned().fold(0.0, f32::max) / peak
        })
        .collect();

    let played_cols = if app.current_track.as_deref() == Some(track.as_path()) {
        (app.progress_ratio() * cols as f64).round() as usize
    } else {
        0
    };

    let lines: Vec<Line> = (0..rows)
        .map(|r| {
            let floor = (rows - 1 - r) * 8;
            let spans: Vec<Span> = levels
                .iter()
                .enumerate()
                .map(|(c, level)| {
                    let eighths = ((level * (rows * 8) as f32).round() as usize).max(1);
                    let glyph = BAR_GLYPHS[eighths.saturating_sub(floor).min(8)];
                    let color = if c < played_cols {
                        theme.accent
                    } else if played_cols > 0 {
                        theme.muted
                    } else {
                        theme.secondary
                    };
                    Span::styled(glyph.to_string(), Style::default().fg(color))
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

// ---------------------------------------------------------------- player bar

fn draw_player_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let (state_label, color) = match (&app.current_track, &app.playback_state) {
        (Some(_), PlaybackState::Playing) => (format!("{} Playing", eq_glyph(app)), theme.success),
        (Some(_), PlaybackState::Paused) => ("⏸ Paused".to_string(), theme.warning),
        _ => ("⏹ Stopped".to_string(), theme.muted),
    };
    let block = panel(state_label, color);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),
            Constraint::Length(3),
            Constraint::Length(20),
        ])
        .split(inner);
    let main = chunks[0];
    let width = main.width as usize;

    // Line 1: what is playing.
    let now = match &app.current_track {
        Some(track) => {
            let mut spans = vec![Span::styled(
                track_title(app, track),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            )];
            if let Some(artist) = track_artist(app, track) {
                spans.push(Span::styled("  —  ", Style::default().fg(theme.muted)));
                spans.push(Span::styled(artist, Style::default().fg(theme.accent)));
            }
            spans
        }
        None => vec![Span::styled(
            "Nothing playing — pick a track and press Enter",
            Style::default().fg(theme.muted),
        )],
    };
    let now_text: String = now.iter().map(|s| s.content.as_ref()).collect();
    let now_line = if text_width(&now_text) > width {
        Line::from(Span::styled(fit(&now_text, width), now[0].style))
    } else {
        Line::from(now)
    };
    f.render_widget(Paragraph::new(now_line), Rect { height: 1, ..main });

    // Line 2: elapsed ━━━━●──── total
    let (elapsed, total) = match (&app.current_track, app.current_total_duration()) {
        (Some(_), Some(total)) => (
            metadata::format_duration(app.current_position()),
            metadata::format_duration(total),
        ),
        (Some(_), None) => (
            metadata::format_duration(app.current_position()),
            "?".into(),
        ),
        (None, _) => ("0:00".to_string(), "0:00".to_string()),
    };
    let prefix = format!("{elapsed} ");
    let suffix = format!(" {total}");
    let bar_width = width.saturating_sub(text_width(&prefix) + text_width(&suffix));
    let filled = (app.progress_ratio() * bar_width as f64).round() as usize;
    let mut bar = vec![Span::styled(
        prefix.clone(),
        Style::default().fg(theme.text),
    )];
    if bar_width > 0 {
        let knob = filled.min(bar_width - 1);
        bar.push(Span::styled(
            "━".repeat(knob),
            Style::default().fg(theme.accent),
        ));
        bar.push(Span::styled(
            if app.current_track.is_some() {
                "●"
            } else {
                "─"
            },
            Style::default().fg(theme.text),
        ));
        bar.push(Span::styled(
            "─".repeat(bar_width - knob - 1),
            Style::default().fg(theme.selection_bg),
        ));
    }
    bar.push(Span::styled(suffix, Style::default().fg(theme.muted)));
    let progress_row = Rect {
        y: main.y + 1,
        height: 1,
        ..main
    };
    if main.height > 1 {
        f.render_widget(Paragraph::new(Line::from(bar)), progress_row);
    }

    let mut hit = app.hit_areas.get();
    hit.progress_bar = Rect {
        x: progress_row.x + text_width(&prefix) as u16,
        width: bar_width as u16,
        ..progress_row
    };
    app.hit_areas.set(hit);

    // Right: volume.
    let vol = chunks[2];
    let vol_label = if app.muted {
        "muted".to_string()
    } else {
        format!("{}%", app.volume_percent())
    };
    let slots = 10;
    let lit = (app.volume_percent() as usize * slots)
        .div_ceil(100)
        .min(slots);
    let vol_color = if app.muted {
        theme.muted
    } else {
        theme.warning
    };
    let vol_lines = vec![
        Line::from(vec![
            Span::styled("vol ", Style::default().fg(theme.muted)),
            Span::styled(
                vol_label,
                Style::default().fg(vol_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("━".repeat(lit), Style::default().fg(vol_color)),
            Span::styled(
                "─".repeat(slots - lit),
                Style::default().fg(theme.selection_bg),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(vol_lines).alignment(Alignment::Right), vol);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme_mode.palette();
    let (status, fresh) = app.status();
    let status_color = if status.starts_with("Error") {
        theme.warning
    } else if fresh {
        theme.text
    } else {
        theme.muted
    };

    let hints: Vec<Span> = if app.is_searching {
        [("type", "filter"), ("⏎", "keep"), ("esc", "clear")]
            .into_iter()
            .flat_map(|(k, l)| key_hint(k, l, theme))
            .collect()
    } else {
        [
            ("⏎", "play"),
            ("p", "pause"),
            ("n/b", "next/prev"),
            ("a", "queue"),
            ("z", "shuffle"),
            ("/", "search"),
            ("?", "help"),
            ("q", "quit"),
        ]
        .into_iter()
        .flat_map(|(k, l)| key_hint(k, l, theme))
        .collect()
    };

    let width = area.width as usize;
    let hints_width: usize = hints.iter().map(|s| s.width()).sum();
    let status_span = |room: usize| {
        Span::styled(
            format!(" {}", fit(&status, room.saturating_sub(1))),
            Style::default().fg(status_color),
        )
    };
    let line = if hints_width + 20 <= width {
        spread(vec![status_span(width - hints_width - 2)], hints, width)
    } else {
        Line::from(status_span(width))
    };
    f.render_widget(Paragraph::new(line), area);
}

// ---------------------------------------------------------------- popups

fn draw_queue_popup(f: &mut Frame, app: &mut App) {
    let theme = app.theme_mode.palette();
    let area = centered_rect(70, 70, f.size());
    f.render_widget(Clear, area);

    let outer = panel(format!("Queue · {} tracks", app.queue_len()), theme.warning);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(inner);

    if app.queue.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Queue is empty",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Close this popup, highlight a track and press a to enqueue it,"),
            Line::from("or A to enqueue every track in the current view."),
        ])
        .style(Style::default().fg(theme.muted))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(empty, chunks[0]);
    } else {
        let width = chunks[0].width as usize;
        let items: Vec<ListItem> = app
            .queue
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let duration = app
                    .metadata_for(path)
                    .and_then(|m| m.duration)
                    .map(metadata::format_duration)
                    .unwrap_or_default();
                let artist = track_artist(app, path).unwrap_or_else(|| track_folder_name(path));
                let room = width.saturating_sub(6 + text_width(&duration) + 1);
                ListItem::new(spread(
                    vec![
                        Span::styled(format!("{:>3}  ", i + 1), Style::default().fg(theme.muted)),
                        Span::styled(
                            fit(&track_title(app, path), room / 2 + room % 2),
                            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("  ", Style::default()),
                        Span::styled(fit(&artist, room / 2), Style::default().fg(theme.accent)),
                    ],
                    vec![
                        Span::styled(duration, Style::default().fg(theme.muted)),
                        Span::raw(" "),
                    ],
                    width,
                ))
            })
            .collect();

        let list = List::new(items).highlight_style(Style::default().bg(theme.selection_bg));
        f.render_stateful_widget(list, chunks[0], &mut app.queue_state);
    }

    let mut hints: Vec<Span> = Vec::new();
    for (k, l) in [
        ("j/k", "move"),
        ("⏎", "play"),
        ("d", "remove"),
        ("c", "clear"),
        ("esc", "close"),
    ] {
        hints.extend(key_hint(k, l, theme));
    }
    let help = Paragraph::new(vec![
        Line::from(hints),
        Line::from(Span::styled(
            "S / L in the main view saves / loads ~/.config/meloic/queue.m3u",
            Style::default().fg(theme.muted),
        )),
    ])
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.selection_bg)),
    );
    f.render_widget(help, chunks[1]);
}

fn draw_cover_popup(f: &mut Frame, app: &App) {
    let theme = app.theme_mode.palette();
    let area = centered_rect(80, 92, f.size());
    f.render_widget(Clear, area);

    let block = panel("Cover", theme.accent).title(
        ratatui::widgets::block::Title::from(Span::styled(
            format!(
                " {} · v / esc to close ",
                app.covers.borrow().protocol_label()
            ),
            Style::default().fg(theme.muted),
        ))
        .alignment(Alignment::Right),
    );
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(track) = app.current_track.clone().or_else(|| app.get_selected()) else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),
            Constraint::Length(1),
            Constraint::Length(4),
        ])
        .split(inner);

    let art = app.covers.borrow().square_in(chunks[0]);
    draw_cover(f, art, app, CoverSlot::Popup, &track);

    let meta = app.metadata_for(&track);
    let width = chunks[2].width as usize;
    let mut details = vec![
        Span::styled(
            meta.map(|m| m.display_artist().to_string())
                .unwrap_or_else(|| "Unknown artist".into()),
            Style::default().fg(theme.accent),
        ),
        Span::styled("  ·  ", Style::default().fg(theme.muted)),
        Span::styled(
            meta.map(|m| m.display_album().to_string())
                .unwrap_or_else(|| track_folder_name(&track)),
            Style::default().fg(theme.secondary),
        ),
    ];
    if let Some(year) = meta.and_then(|m| m.year) {
        details.push(Span::styled(
            format!("  ·  {year}"),
            Style::default().fg(theme.muted),
        ));
    }

    let is_current = app.current_track.as_deref() == Some(track.as_path());
    let progress = if is_current {
        let width = width.min(60);
        let filled = (app.progress_ratio() * width as f64).round() as usize;
        Line::from(vec![
            Span::styled("━".repeat(filled), Style::default().fg(theme.accent)),
            Span::styled(
                "─".repeat(width - filled),
                Style::default().fg(theme.selection_bg),
            ),
        ])
    } else {
        Line::from(Span::styled(
            "press Enter in the library to play",
            Style::default().fg(theme.muted),
        ))
    };

    let lines = vec![
        Line::from(Span::styled(
            fit(&track_title(app, &track), width),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(details),
        Line::from(""),
        progress,
    ];
    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        chunks[2],
    );
}

fn draw_info_popup(f: &mut Frame, app: &App) {
    let theme = app.theme_mode.palette();
    let area = centered_rect(60, 70, f.size());
    f.render_widget(Clear, area);

    let block = panel("Track Info", theme.secondary);

    let Some(track) = app.get_selected() else {
        f.render_widget(Paragraph::new("No track selected").block(block), area);
        return;
    };

    let unknown = TrackMetadata::default();
    let m = app.metadata_for(&track).unwrap_or(&unknown);
    let dash = || "—".to_string();

    let field = |label: &str, value: String, color: Color| {
        Line::from(vec![
            Span::styled(format!("  {label:<12}"), Style::default().fg(theme.muted)),
            Span::styled(value, Style::default().fg(color)),
        ])
    };
    let heading = |text: &str| {
        Line::from(Span::styled(
            format!("  {text}"),
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        ))
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", m.display_title(&track_display_name(&track))),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        heading("TAGS"),
        field("Artist", m.display_artist().to_string(), theme.accent),
        field("Album", m.display_album().to_string(), theme.secondary),
        field(
            "Year",
            m.year.map(|y| y.to_string()).unwrap_or_else(dash),
            theme.text,
        ),
        field(
            "Track",
            m.track_number.map(|t| t.to_string()).unwrap_or_else(dash),
            theme.text,
        ),
        field("Genre", m.genre.clone().unwrap_or_else(dash), theme.warning),
        Line::from(""),
        heading("AUDIO"),
        field(
            "Duration",
            m.duration
                .map(metadata::format_duration)
                .unwrap_or_else(dash),
            theme.success,
        ),
        field(
            "Sample rate",
            m.sample_rate
                .map(|s| format!("{s} Hz"))
                .unwrap_or_else(dash),
            theme.text,
        ),
        field(
            "Bit depth",
            m.bit_depth.map(|b| format!("{b} bit")).unwrap_or_else(dash),
            theme.text,
        ),
        field(
            "Channels",
            m.channels
                .map(|c| match c {
                    1 => "mono".to_string(),
                    2 => "stereo".to_string(),
                    n => format!("{n} ch"),
                })
                .unwrap_or_else(dash),
            theme.text,
        ),
        field("Format", track_extension(&track), theme.accent),
        field("Size", human_file_size(&track), theme.text),
        Line::from(""),
        heading("PATH"),
        Line::from(Span::styled(
            format!("  {}", track.to_string_lossy()),
            Style::default().fg(theme.muted),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  i / esc to close",
            Style::default().fg(theme.muted),
        )),
    ];

    let widget = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
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

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
