use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::theme as dhara_theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboPart {
    Left,
    Value,
    Right,
}

pub struct ComboClickRegions {
    pub left: Rect,
    pub value: Rect,
    pub right: Rect,
    pub row: Rect,
}

pub fn render_bios_combo(
    area: Rect,
    label: &str,
    value: &str,
    selected: bool,
    embedded: Option<ComboPart>,
    buf: &mut Buffer,
) -> ComboClickRegions {
    let prefix = if selected { "▸ " } else { "  " };
    let label_text = format!("{prefix}{label} ");
    let label_width = label_text.chars().count() as u16;
    let left_w: u16 = 3; // ⮜ + pad
    let right_w: u16 = 3;
    let bracket_w: u16 = 1;

    let x = area.x;
    let y = area.y;

    Paragraph::new(Line::from(Span::styled(
        label_text,
        if selected {
            dhara_theme::selected_style()
        } else {
            Style::default().fg(dhara_theme::TEXT)
        },
    )))
    .render(Rect::new(x, y, label_width.min(area.width), 1), buf);

    let combo_x = x.saturating_add(label_width.min(area.width));
    let combo_w = area.width.saturating_sub(label_width.min(area.width));
    let value_w = combo_w.saturating_sub(left_w + right_w + bracket_w * 2).max(1);

    let left_area = Rect::new(combo_x, y, left_w.min(combo_w), 1);
    let lb = Rect::new(combo_x.saturating_add(left_w), y, bracket_w, 1);
    let value_area = Rect::new(
        combo_x.saturating_add(left_w + bracket_w),
        y,
        value_w,
        1,
    );
    let rb = Rect::new(
        combo_x.saturating_add(left_w + bracket_w + value_w),
        y,
        bracket_w,
        1,
    );
    let right_area = Rect::new(
        combo_x.saturating_add(left_w + bracket_w * 2 + value_w),
        y,
        right_w,
        1,
    );

    let accent = |part: ComboPart| {
        if embedded == Some(part) {
            dhara_theme::selected_style()
        } else if selected {
            Style::default().fg(dhara_theme::ACCENT)
        } else {
            Style::default().fg(dhara_theme::TEXT)
        }
    };

    Paragraph::new(Line::from(Span::styled("⮜", accent(ComboPart::Left))))
        .render(left_area, buf);
    Paragraph::new(Line::from(Span::styled("【", Style::default().fg(dhara_theme::MUTED))))
        .render(lb, buf);
    Paragraph::new(Line::from(Span::styled(
        truncate(value, value_w as usize),
        if embedded == Some(ComboPart::Value) {
            dhara_theme::selected_style()
        } else {
            Style::default().fg(dhara_theme::TEXT)
        },
    )))
    .render(value_area, buf);
    Paragraph::new(Line::from(Span::styled("】", Style::default().fg(dhara_theme::MUTED))))
        .render(rb, buf);
    Paragraph::new(Line::from(Span::styled("⮞", accent(ComboPart::Right))))
        .render(right_area, buf);

    ComboClickRegions {
        left: left_area,
        value: Rect::new(lb.x, y, lb.width.saturating_add(value_area.width).saturating_add(rb.width), 1),
        right: right_area,
        row: area,
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_owned()
    } else if max_chars == 0 {
        String::new()
    } else {
        text.chars().take(max_chars.saturating_sub(1)).collect::<String>() + "…"
    }
}
