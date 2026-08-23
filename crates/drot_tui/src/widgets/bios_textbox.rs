use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::theme as dhara_theme;

const PROMPT: &str = ">";
const PAD: &str = " ";
const PLACEHOLDER_CURSOR: &str = "_";
/// Minimum text columns inside the highlight (keeps empty fields readable).
const MIN_TEXT_COLS: u16 = 4;

pub struct TextboxClickRegions {
    pub label: Rect,
    pub input: Rect,
    pub row: Rect,
}

pub struct TextboxRenderResult {
    pub regions: TextboxClickRegions,
    /// Terminal cursor when embedded — Ratatui shows/blinks it after draw.
    pub cursor: Option<Position>,
}

pub fn render_bios_textbox(
    area: Rect,
    label: &str,
    value: &str,
    selected: bool,
    embedded: bool,
    cursor_pos: usize,
    buf: &mut Buffer,
) -> TextboxRenderResult {
    let row_selected = selected;
    let inner_active = selected && embedded;
    let bg = cluster_bg(row_selected, inner_active);

    let label_prefix = if row_selected { "> " } else { "  " };
    let label_style = if row_selected {
        if inner_active {
            Style::default()
                .fg(dhara_theme::ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(dhara_theme::ACCENT)
        }
    } else {
        Style::default().fg(dhara_theme::TEXT)
    };
    let prefix_style = if row_selected {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(dhara_theme::TEXT)
    };

    let label_line = format!("{label} ");
    let label_w = (2 + label_line.width()).min(area.width as usize) as u16;
    Paragraph::new(Line::from(vec![
        Span::styled(label_prefix, prefix_style),
        Span::styled(clip_label(&label_line, label_w.saturating_sub(2) as usize), label_style),
    ]))
    .render(Rect::new(area.x, area.y, label_w.max(1), 1), buf);

    let avail = area.width.saturating_sub(label_w).max(1);
    let layout = cluster_layout(area, label_w, avail, value, inner_active, cursor_pos);

    paint_cluster_background(
        Rect::new(layout.cluster_x, area.y, layout.cluster_w, 1),
        bg,
        buf,
    );

    let pad_style = Style::default().bg(bg);
    let prompt_style = if inner_active {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    } else if row_selected {
        Style::default()
            .fg(dhara_theme::TEXT)
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(dhara_theme::MUTED).bg(bg)
    };
    let text_style = if inner_active {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    } else if row_selected {
        Style::default().fg(dhara_theme::TEXT).bg(bg)
    } else {
        Style::default().fg(dhara_theme::TEXT).bg(bg)
    };

    buf.set_string(layout.pad_left.x, layout.pad_left.y, PAD, pad_style);
    buf.set_string(layout.prompt.x, layout.prompt.y, PROMPT, prompt_style);

    let visible = &layout.visible_text;
    if !visible.is_empty() {
        buf.set_string(layout.text.x, layout.text.y, visible, text_style);
    }

    let cursor = if inner_active {
        Some(Position {
            x: layout.cursor_x,
            y: area.y,
        })
    } else if value.is_empty() {
        // Idle empty field: static `_` cue (not a live caret).
        let cue_style = Style::default().fg(dhara_theme::MUTED).bg(bg);
        buf.set_string(
            layout.cursor_x,
            area.y,
            PLACEHOLDER_CURSOR,
            cue_style,
        );
        None
    } else {
        None
    };

    // Trailing pad after the text/cursor column.
    buf.set_string(layout.pad_right.x, layout.pad_right.y, PAD, pad_style);

    TextboxRenderResult {
        regions: TextboxClickRegions {
            label: Rect::new(area.x, area.y, label_w.max(1), 1),
            input: Rect::new(layout.cluster_x, area.y, layout.cluster_w, 1),
            row: area,
        },
        cursor,
    }
}

struct ClusterLayout {
    cluster_x: u16,
    cluster_w: u16,
    pad_left: Rect,
    prompt: Rect,
    text: Rect,
    pad_right: Rect,
    visible_text: String,
    cursor_x: u16,
}

fn cluster_layout(
    area: Rect,
    label_w: u16,
    avail: u16,
    value: &str,
    inner_active: bool,
    cursor_pos: usize,
) -> ClusterLayout {
    let pad_w = 1u16;
    let prompt_w = display_width(PROMPT) as u16;
    let chrome = pad_w
        .saturating_add(prompt_w)
        .saturating_add(pad_w);

    let max_text = avail.saturating_sub(chrome).max(1);
    let content_w = value.width() as u16;
    // Reserve one column for idle `_` when empty and not editing.
    let idle_cursor_cols = if !inner_active && value.is_empty() {
        1
    } else {
        0
    };
    let desired_text = content_w
        .saturating_add(idle_cursor_cols)
        .max(MIN_TEXT_COLS)
        .min(max_text);

    let cluster_w = chrome.saturating_add(desired_text);
    let cluster_x = area
        .x
        .saturating_add(label_w)
        .saturating_add(avail.saturating_sub(cluster_w));
    let y = area.y;

    let mut x = cluster_x;
    let pad_left = Rect::new(x, y, pad_w, 1);
    x = x.saturating_add(pad_w);
    let prompt = Rect::new(x, y, prompt_w.max(1), 1);
    x = x.saturating_add(prompt_w);
    let text_area = Rect::new(x, y, desired_text, 1);

    let (visible, cursor_col_in_text) =
        visible_window(value, cursor_pos, desired_text as usize, inner_active);
    let cursor_x = text_area.x.saturating_add(cursor_col_in_text as u16);

    x = x.saturating_add(desired_text);
    let pad_right = Rect::new(x, y, pad_w, 1);

    ClusterLayout {
        cluster_x,
        cluster_w,
        pad_left,
        prompt,
        text: text_area,
        pad_right,
        visible_text: visible,
        cursor_x,
    }
}

/// Window text so the caret stays inside the slot; returns (visible, caret column in slot).
fn visible_window(
    value: &str,
    cursor_pos: usize,
    slot_w: usize,
    inner_active: bool,
) -> (String, usize) {
    if slot_w == 0 {
        return (String::new(), 0);
    }
    if !inner_active {
        let clipped = clip_right(value, slot_w.saturating_sub(if value.is_empty() { 1 } else { 0 }));
        let col = if value.is_empty() { 0 } else { clipped.width().min(slot_w) };
        return (clipped, col.min(slot_w.saturating_sub(1)));
    }

    let chars: Vec<char> = value.chars().collect();
    let cursor = cursor_pos.min(chars.len());
    // Prefer keeping the caret visible: scroll so cursor is near the end of the window.
    let mut start = 0usize;
    if chars.len() > slot_w {
        start = cursor.saturating_sub(slot_w.saturating_sub(1));
        if start + slot_w > chars.len() {
            start = chars.len().saturating_sub(slot_w);
        }
    }
    let end = (start + slot_w).min(chars.len());
    let visible: String = chars[start..end].iter().collect();
    let caret_col = cursor.saturating_sub(start).min(slot_w.saturating_sub(1));
    // When caret is at end and there is room, place it just after the last char.
    let caret_col = if cursor == chars.len() && visible.width() < slot_w {
        visible.width()
    } else {
        caret_col
    };
    (visible, caret_col)
}

fn cluster_bg(selected: bool, inner_active: bool) -> ratatui::style::Color {
    if inner_active || selected {
        dhara_theme::COMBO_BG_SELECTED
    } else {
        dhara_theme::COMBO_BG
    }
}

fn paint_cluster_background(cluster: Rect, bg: ratatui::style::Color, buf: &mut Buffer) {
    for x in cluster.x..cluster.x.saturating_add(cluster.width) {
        if let Some(cell) = buf.cell_mut((x, cluster.y)) {
            cell.set_bg(bg);
        }
    }
}

fn display_width(text: &str) -> usize {
    text.width().max(1)
}

fn clip_right(text: &str, max_cols: usize) -> String {
    if text.width() <= max_cols {
        return text.to_owned();
    }
    if max_cols == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut w = 0usize;
    for ch in text.chars() {
        let cw = ch.width().unwrap_or(0);
        if w + cw > max_cols.saturating_sub(1) {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out.push('…');
    out
}

fn clip_label(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_owned()
    } else if max_chars == 0 {
        String::new()
    } else {
        text.chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>()
            + "…"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_idle_reserves_placeholder_column() {
        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        let result = render_bios_textbox(area, "Source", "", true, false, 0, &mut buf);
        assert!(result.cursor.is_none());
        assert!(result.regions.input.width >= MIN_TEXT_COLS + 3);
    }

    #[test]
    fn embedded_returns_terminal_cursor() {
        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        let result = render_bios_textbox(area, "Source", "ab", true, true, 1, &mut buf);
        assert!(result.cursor.is_some());
    }

    #[test]
    fn cluster_grows_with_text() {
        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        let short = render_bios_textbox(area, "Source", "a", false, false, 0, &mut buf);
        let long = render_bios_textbox(area, "Source", "abcdefghijkl", false, false, 0, &mut buf);
        assert!(long.regions.input.width > short.regions.input.width);
    }
}
