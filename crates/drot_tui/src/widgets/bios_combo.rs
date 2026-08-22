use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
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
    pub label: Rect,
    pub left: Rect,
    pub value: Rect,
    pub right: Rect,
}

const MIN_VALUE_CHARS: usize = 10;

pub fn render_bios_combo(
    area: Rect,
    label: &str,
    value: &str,
    selected: bool,
    embedded: Option<ComboPart>,
    buf: &mut Buffer,
) -> ComboClickRegions {
    let prefix = if selected { "▸ " } else { "  " };
    let label_text = format!("{prefix}{label}");

    let cluster_w = combo_cluster_width(value, area.width);
    let cluster_x = area.x.saturating_add(area.width.saturating_sub(cluster_w));
    let label_render_w = cluster_x.saturating_sub(area.x);

    Paragraph::new(Line::from(Span::styled(
        truncate_label(&label_text, label_render_w as usize),
        if selected && embedded.is_none() {
            dhara_theme::selected_style()
        } else {
            Style::default().fg(dhara_theme::TEXT)
        },
    )))
    .render(
        Rect::new(area.x, area.y, label_render_w.max(1), 1),
        buf,
    );

    let cluster = Rect::new(cluster_x, area.y, cluster_w, 1);
    paint_cluster_background(cluster, selected, embedded.is_some(), buf);

    let inner_x = cluster.x.saturating_add(1);
    let inner_w = cluster.width.saturating_sub(2).max(3);
    let left_area = Rect::new(inner_x, area.y, 1, 1);
    let right_area = Rect::new(
        cluster.x.saturating_add(cluster.width.saturating_sub(2)),
        area.y,
        1,
        1,
    );
    let value_area = Rect::new(
        inner_x.saturating_add(1),
        area.y,
        inner_w.saturating_sub(2).max(1),
        1,
    );

    let frame_style = if selected || embedded.is_some() {
        Style::default().fg(dhara_theme::ACCENT)
    } else {
        Style::default().fg(dhara_theme::COMBO_BORDER)
    };

    Paragraph::new(Line::from(Span::styled("│", frame_style))).render(
        Rect::new(cluster.x, area.y, 1, 1),
        buf,
    );
    Paragraph::new(Line::from(Span::styled("│", frame_style))).render(
        Rect::new(cluster.x.saturating_add(cluster.width.saturating_sub(1)), area.y, 1, 1),
        buf,
    );

    Paragraph::new(Line::from(Span::styled(
        "◀",
        part_style(ComboPart::Left, selected, embedded),
    )))
    .render(left_area, buf);

    Paragraph::new(Line::from(Span::styled(
        truncate(value, value_area.width as usize),
        if embedded == Some(ComboPart::Value) {
            dhara_theme::selected_style()
        } else {
            Style::default()
                .fg(dhara_theme::TEXT)
                .bg(dhara_theme::COMBO_BG)
                .add_modifier(Modifier::BOLD)
        },
    )))
    .render(value_area, buf);

    Paragraph::new(Line::from(Span::styled(
        "▶",
        part_style(ComboPart::Right, selected, embedded),
    )))
    .render(right_area, buf);

    ComboClickRegions {
        label: Rect::new(area.x, area.y, label_render_w.max(1), 1),
        left: left_area,
        value: value_area,
        right: right_area,
    }
}

fn combo_cluster_width(value: &str, max_width: u16) -> u16 {
    let value_chars = value.chars().count().max(MIN_VALUE_CHARS) as u16;
    // │ + ◀ + value + ▶ + │
    let width = value_chars.saturating_add(4);
    width.clamp(12, max_width.max(12))
}

fn paint_cluster_background(
    cluster: Rect,
    selected: bool,
    embedded: bool,
    buf: &mut Buffer,
) {
    let bg = if embedded {
        dhara_theme::SELECTED_BG
    } else if selected {
        dhara_theme::COMBO_BG_SELECTED
    } else {
        dhara_theme::COMBO_BG
    };
    for x in cluster.x..cluster.x.saturating_add(cluster.width) {
        if let Some(cell) = buf.cell_mut((x, cluster.y)) {
            cell.set_bg(bg);
        }
    }
}

fn part_style(part: ComboPart, selected: bool, embedded: Option<ComboPart>) -> Style {
    if embedded == Some(part) {
        dhara_theme::selected_style()
    } else if selected {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .bg(dhara_theme::COMBO_BG)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(dhara_theme::MUTED)
            .bg(dhara_theme::COMBO_BG)
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
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

fn truncate_label(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        String::new()
    } else {
        truncate(text, max_chars)
    }
}
