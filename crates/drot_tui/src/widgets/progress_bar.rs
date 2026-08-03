//! One-line Unicode-capped Gauge-style progress bar with centered percentage.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::theme as dhara_theme;

const CAP_LEFT: char = '';
const CAP_RIGHT: char = '';

/// Paint a single-row capsule progress bar; `ratio` in `0.0..=1.0`.
///
/// Middle uses Gauge-style spaces (`fg`+`bg` = fill) instead of block/`░` glyphs.
pub fn render_capsule_progress(area: Rect, ratio: f64, complete: bool, buf: &mut Buffer) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let fill_color = dhara_theme::SUCCESS;
    let track_color = dhara_theme::PROGRESS_TRACK;
    let y = area.y;
    let _ = complete; // green for both running and complete (matches prior bar)

    let ratio = ratio.clamp(0.0, 1.0);
    let percent = ((ratio * 100.0).round() as u16).min(100);
    let label = format!("{percent}%");

    if area.width < 3 {
        buf[(area.x, y)]
            .set_char(' ')
            .set_style(Style::default().fg(fill_color).bg(fill_color));
        return;
    }

    let track_width = (area.width as usize).saturating_sub(2);
    let filled_cols = ((ratio * track_width as f64).round() as usize).min(track_width);

    let left_style = if filled_cols > 0 || percent == 100 {
        Style::default().fg(fill_color)
    } else {
        Style::default().fg(track_color)
    };
    buf[(area.x, y)].set_char(CAP_LEFT).set_style(left_style);

    for i in 0..track_width {
        let style = if i < filled_cols {
            Style::default().fg(fill_color).bg(fill_color)
        } else {
            Style::default().fg(track_color).bg(track_color)
        };
        buf[(area.x + 1 + i as u16, y)]
            .set_char(' ')
            .set_style(style);
    }

    let right_style = if filled_cols >= track_width {
        Style::default().fg(fill_color)
    } else {
        Style::default().fg(track_color)
    };
    buf[(area.x + area.width - 1, y)]
        .set_char(CAP_RIGHT)
        .set_style(right_style);

    // Centered percentage overpaint.
    let label_width = label.chars().count() as u16;
    if label_width > 0 && label_width <= area.width {
        let start = area.x + (area.width.saturating_sub(label_width)) / 2;
        for (offset, ch) in label.chars().enumerate() {
            let x = start + offset as u16;
            let track_index = x.saturating_sub(area.x + 1) as usize;
            let on_fill = if x <= area.x {
                filled_cols > 0
            } else if x >= area.x + area.width - 1 {
                filled_cols >= track_width
            } else {
                track_index < filled_cols
            };
            let fg = if on_fill {
                Color::Black
            } else {
                dhara_theme::TEXT
            };
            let bg = if on_fill { fill_color } else { track_color };
            buf[(x, y)]
                .set_char(ch)
                .set_style(Style::default().fg(fg).bg(bg));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paints_full_width_without_panic() {
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        render_capsule_progress(area, 0.0, false, &mut buf);
        render_capsule_progress(area, 0.42, false, &mut buf);
        render_capsule_progress(area, 1.0, true, &mut buf);
    }
}
