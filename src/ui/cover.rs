//! Album cover rendering.
//!
//! Terminals with a graphics protocol (kitty, sixel, iTerm2) get the real
//! image at full pixel resolution through `ratatui-image`. Everything else
//! falls back to unicode halfblocks, two pixels per cell.

use image::{DynamicImage, GenericImageView, imageops::FilterType};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use ratatui_image::{
    Resize, StatefulImage,
    picker::{Picker, ProtocolType},
    protocol::StatefulProtocol,
};
use std::path::{Path, PathBuf};

/// Cell size in "pixels" used by the halfblock fallback: one wide, two tall.
const HALFBLOCK_FONT: (u16, u16) = (1, 2);

/// Ask the terminal which graphics protocol it speaks. Must run before raw
/// mode / the event reader start, since it reads the reply from stdin.
/// Set `MELOIC_NO_GRAPHICS=1` to force the halfblock renderer.
pub fn detect_picker() -> Option<Picker> {
    if std::env::var_os("MELOIC_NO_GRAPHICS").is_some_and(|v| !v.is_empty()) {
        return None;
    }
    let mut picker = Picker::from_termios().ok()?;
    if picker.font_size.0 == 0 || picker.font_size.1 == 0 {
        return None;
    }
    match picker.guess_protocol() {
        ProtocolType::Halfblocks => None,
        // ratatui-image 1.x does not wrap kitty sequences for tmux passthrough.
        ProtocolType::Kitty if picker.is_tmux => None,
        _ => Some(picker),
    }
}

/// Independent places a cover can be shown at once. Each one maps to its own
/// kitty image id, so the sidebar and the popup never overwrite each other.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CoverSlot {
    Sidebar = 0,
    Popup = 1,
}

enum Rendered {
    Graphic(Box<dyn StatefulProtocol>),
    Blocks(Vec<Line<'static>>),
}

struct CachedCover {
    track: PathBuf,
    size: (u16, u16),
    rendered: Rendered,
}

pub struct CoverRenderer {
    picker: Option<Picker>,
    slots: [Option<CachedCover>; 2],
}

impl CoverRenderer {
    pub fn new(picker: Option<Picker>) -> Self {
        Self {
            picker,
            slots: [None, None],
        }
    }

    pub fn protocol_label(&self) -> &'static str {
        match self.picker.map(|p| p.protocol_type) {
            Some(ProtocolType::Kitty) => "kitty",
            Some(ProtocolType::Sixel) => "sixel",
            Some(ProtocolType::Iterm2) => "iterm2",
            _ => "halfblocks",
        }
    }

    fn font_size(&self) -> (u16, u16) {
        self.picker.map(|p| p.font_size).unwrap_or(HALFBLOCK_FONT)
    }

    /// Largest square (in pixels) that fits `area`, centred, in cells.
    pub fn square_in(&self, area: Rect) -> Rect {
        let (fw, fh) = self.font_size();
        let side = (area.width as u32 * fw as u32).min(area.height as u32 * fh as u32);
        let width = ((side / fw as u32) as u16).min(area.width);
        let height = ((side / fh as u32) as u16).min(area.height);
        Rect {
            x: area.x + (area.width - width) / 2,
            y: area.y + (area.height - height) / 2,
            width,
            height,
        }
    }

    /// Draw the cover for `track` into `area`. Returns false when the image
    /// data could not be decoded, so the caller can draw a placeholder.
    pub fn render(
        &mut self,
        f: &mut Frame,
        area: Rect,
        slot: CoverSlot,
        track: &Path,
        data: &[u8],
    ) -> bool {
        if area.width == 0 || area.height == 0 {
            return true;
        }
        let size = (area.width, area.height);
        let index = slot as usize;
        let fresh = matches!(
            &self.slots[index],
            Some(c) if c.track == track && c.size == size
        );

        if !fresh {
            let Ok(image) = image::load_from_memory(data) else {
                self.slots[index] = None;
                return false;
            };
            let (fw, fh) = self.font_size();
            let image = fit_pixels(
                &image,
                area.width as u32 * fw as u32,
                area.height as u32 * fh as u32,
            );
            let rendered = match self.picker {
                Some(picker) => Rendered::Graphic(self.new_protocol(picker, slot, image)),
                None => Rendered::Blocks(halfblock_lines(&image, area.width, area.height)),
            };
            self.slots[index] = Some(CachedCover {
                track: track.to_path_buf(),
                size,
                rendered,
            });
        }

        match self.slots[index].as_mut().map(|c| &mut c.rendered) {
            Some(Rendered::Graphic(protocol)) => {
                let widget = StatefulImage::new(None).resize(Resize::Fit(None));
                f.render_stateful_widget(widget, area, protocol);
            }
            Some(Rendered::Blocks(lines)) => {
                f.render_widget(Paragraph::new(lines.clone()), area);
            }
            None => {}
        }
        true
    }

    /// ratatui-image hands out kitty ids from a private counter that saturates
    /// at 255, so start each slot from a fresh copy of the picker and bump it
    /// to a fixed id instead (sidebar = 1, popup = 2).
    fn new_protocol(
        &self,
        mut picker: Picker,
        slot: CoverSlot,
        image: DynamicImage,
    ) -> Box<dyn StatefulProtocol> {
        for _ in 0..slot as usize {
            picker.new_resize_protocol(DynamicImage::new_rgb8(1, 1));
        }
        picker.new_resize_protocol(image)
    }
}

/// Scale (up or down) to fit inside `max_w` x `max_h`, keeping aspect ratio.
fn fit_pixels(image: &DynamicImage, max_w: u32, max_h: u32) -> DynamicImage {
    let (w, h) = image.dimensions();
    if w == 0 || h == 0 || max_w == 0 || max_h == 0 {
        return image.clone();
    }
    let scale = (max_w as f64 / w as f64).min(max_h as f64 / h as f64);
    let target_w = ((w as f64 * scale).round() as u32).max(1);
    let target_h = ((h as f64 * scale).round() as u32).max(1);
    if (target_w, target_h) == (w, h) {
        return image.clone();
    }
    image.resize_exact(target_w, target_h, FilterType::Lanczos3)
}

/// Each cell shows two stacked pixels via '▀' (fg = top, bg = bottom).
fn halfblock_lines(image: &DynamicImage, width: u16, height: u16) -> Vec<Line<'static>> {
    let (img_w, img_h) = image.dimensions();
    let rows = (img_h.div_ceil(2)).min(height as u32);
    let cols = img_w.min(width as u32);
    let pad = " ".repeat(((width as u32 - cols) / 2) as usize);
    let top_pad = (height as u32 - rows) / 2;

    let mut lines: Vec<Line<'static>> = (0..top_pad).map(|_| Line::from("")).collect();
    lines.extend((0..rows).map(|row| {
        let mut spans = vec![Span::raw(pad.clone())];
        spans.extend((0..cols).map(|col| {
            let top = image.get_pixel(col, (row * 2).min(img_h - 1));
            let bot = image.get_pixel(col, (row * 2 + 1).min(img_h - 1));
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(top[0], top[1], top[2]))
                    .bg(Color::Rgb(bot[0], bot[1], bot[2])),
            )
        }));
        Line::from(spans)
    }));
    lines
}
