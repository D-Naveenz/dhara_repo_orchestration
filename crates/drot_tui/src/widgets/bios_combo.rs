use std::time::Instant;

use drot_kernel::{FieldSpec, max_combo_option_width};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

use crate::theme as dhara_theme;
use crate::widgets::text_marquee::{LabelMarquee, center_offset, clip_right, clip_window};

const CHROME_WIDTH: u16 = 6;
const OPEN: &str = "【";
const CLOSE: &str = "】";
const LEFT_ARROW: &str = "⮜";
const RIGHT_ARROW: &str = "⮞";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboPart {
    Inner,
    Left,
    Right,
}

pub struct ComboClickRegions {
    pub label: Rect,
    pub left: Rect,
    pub right: Rect,
    pub open: Rect,
    pub text: Rect,
    pub close: Rect,
}

pub struct ComboRenderParams<'a> {
    pub field: &'a FieldSpec,
    pub value: &'a str,
    pub selected: bool,
    pub embedded: Option<ComboPart>,
    pub marquee: &'a mut LabelMarquee,
    pub marquee_key: &'a str,
    pub now: Instant,
}

pub fn render_bios_combo(
    area: Rect,
    label: &str,
    params: &mut ComboRenderParams<'_>,
    buf: &mut Buffer,
) -> ComboClickRegions {
    let text_w = resolve_text_slot_width(params.field, area.width);
    let cluster_w = text_w.saturating_add(CHROME_WIDTH);
    let cluster_x = area.x.saturating_add(area.width.saturating_sub(cluster_w));
    let label_render_w = cluster_x.saturating_sub(area.x);

    let label_style = if params.selected && params.embedded.is_none() {
        Style::default().fg(dhara_theme::ACCENT)
    } else {
        Style::default().fg(dhara_theme::TEXT)
    };
    Paragraph::new(Line::from(Span::styled(
        clip_label(label, label_render_w as usize),
        label_style,
    )))
    .render(Rect::new(area.x, area.y, label_render_w.max(1), 1), buf);

    let cluster = Rect::new(cluster_x, area.y, cluster_w, 1);
    let embedded_inner = params.embedded == Some(ComboPart::Inner);
    paint_cluster_background(cluster, params.selected, embedded_inner, buf);

    let mut x = cluster.x;
    let open_area = Rect::new(x, area.y, 1, 1);
    x += 1;
    let left_area = Rect::new(x, area.y, 1, 1);
    x += 1;
    let pad_left = Rect::new(x, area.y, 1, 1);
    x += 1;
    let text_area = Rect::new(x, area.y, text_w, 1);
    x = x.saturating_add(text_w);
    let pad_right = Rect::new(x, area.y, 1, 1);
    x += 1;
    let right_area = Rect::new(x, area.y, 1, 1);
    x += 1;
    let close_area = Rect::new(x, area.y, 1, 1);

    let bracket_style = bracket_style(params.selected, embedded_inner);
    Paragraph::new(Line::from(Span::styled(OPEN, bracket_style))).render(open_area, buf);
    Paragraph::new(Line::from(Span::styled(" ", Style::default().bg(cluster_bg(params.selected, embedded_inner)))))
        .render(pad_left, buf);
    Paragraph::new(Line::from(Span::styled(" ", Style::default().bg(cluster_bg(params.selected, embedded_inner)))))
        .render(pad_right, buf);
    Paragraph::new(Line::from(Span::styled(CLOSE, bracket_style))).render(close_area, buf);

    Paragraph::new(Line::from(Span::styled(
        LEFT_ARROW,
        arrow_style(ComboPart::Left, params.selected, params.embedded),
    )))
    .render(left_area, buf);
    Paragraph::new(Line::from(Span::styled(
        RIGHT_ARROW,
        arrow_style(ComboPart::Right, params.selected, params.embedded),
    )))
    .render(right_area, buf);

    paint_value_text(text_area, params, buf);

    ComboClickRegions {
        label: Rect::new(area.x, area.y, label_render_w.max(1), 1),
        left: left_area,
        right: right_area,
        open: open_area,
        text: text_area,
        close: close_area,
    }
}

fn resolve_text_slot_width(field: &FieldSpec, row_width: u16) -> u16 {
    let auto = max_combo_option_width(&field.kind).max(1) as u16;
    let requested = field.tui_combo_width.unwrap_or(auto);
    let max_text = row_width.saturating_sub(CHROME_WIDTH).max(1);
    requested.min(max_text)
}

fn paint_value_text(area: Rect, params: &mut ComboRenderParams<'_>, buf: &mut Buffer) {
    let slot_w = area.width as usize;
    let value = params.value;
    let overflow = value.width() > slot_w;
    let embedded_inner = params.embedded == Some(ComboPart::Inner);
    let style = if embedded_inner {
        dhara_theme::selected_style()
    } else if params.selected {
        Style::default()
            .fg(dhara_theme::TEXT)
            .bg(dhara_theme::COMBO_BG)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(dhara_theme::TEXT)
            .bg(dhara_theme::COMBO_BG)
    };

    let display = if overflow && params.selected && embedded_inner {
        let offset = params.marquee.advance(params.marquee_key, value, slot_w, params.now);
        clip_window(value, offset, slot_w)
    } else if overflow {
        clip_right(value, slot_w)
    } else {
        value.to_owned()
    };

    let x_offset = if overflow {
        0
    } else {
        center_offset(&display, slot_w)
    };
    buf.set_string(
        area.x.saturating_add(x_offset as u16),
        area.y,
        &display,
        style,
    );
}

fn cluster_bg(selected: bool, embedded: bool) -> ratatui::style::Color {
    if embedded {
        dhara_theme::SELECTED_BG
    } else if selected {
        dhara_theme::COMBO_BG_SELECTED
    } else {
        dhara_theme::COMBO_BG
    }
}

fn paint_cluster_background(cluster: Rect, selected: bool, embedded: bool, buf: &mut Buffer) {
    let bg = cluster_bg(selected, embedded);
    for x in cluster.x..cluster.x.saturating_add(cluster.width) {
        if let Some(cell) = buf.cell_mut((x, cluster.y)) {
            cell.set_bg(bg);
        }
    }
}

fn bracket_style(selected: bool, embedded: bool) -> Style {
    if embedded {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .bg(dhara_theme::SELECTED_BG)
            .add_modifier(Modifier::BOLD)
    } else if selected {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .bg(dhara_theme::COMBO_BG_SELECTED)
    } else {
        Style::default()
            .fg(dhara_theme::COMBO_BORDER)
            .bg(dhara_theme::COMBO_BG)
    }
}

fn arrow_style(part: ComboPart, selected: bool, embedded: Option<ComboPart>) -> Style {
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
    use drot_kernel::{ArgBinding, FieldKind, FieldSpec};

    use super::*;

    fn combo_field(width: Option<u16>) -> FieldSpec {
        FieldSpec {
            key: "configuration",
            label: "Configuration",
            help: "",
            kind: FieldKind::Combo(&["Debug", "Release"]),
            binding: ArgBinding::FlagValue("--configuration"),
            required: false,
            default_value: Some("Release"),
            tui_default_value: None,
            group: None,
            invert_switch: false,
            tui_only: false,
            tui_combo_width: width,
        }
    }

    #[test]
    fn intrinsic_text_width_uses_longest_option() {
        let field = combo_field(None);
        assert_eq!(resolve_text_slot_width(&field, 80), "Release".chars().count() as u16);
    }

    #[test]
    fn specified_text_width_is_honored() {
        let field = combo_field(Some(12));
        assert_eq!(resolve_text_slot_width(&field, 80), 12);
    }

    #[test]
    fn cluster_width_includes_six_chrome_columns() {
        let field = combo_field(Some(8));
        assert_eq!(
            resolve_text_slot_width(&field, 80).saturating_add(CHROME_WIDTH),
            14
        );
    }
}
