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

const OPEN: &str = "【";
const CLOSE: &str = "】";
const LEFT_ARROW: &str = "⮜";
const RIGHT_ARROW: &str = "⮞";
const PAD: &str = " ";

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

struct ClusterLayout {
    cluster_x: u16,
    cluster_w: u16,
    open: Rect,
    left: Rect,
    text: Rect,
    pad_before_right: Rect,
    right: Rect,
    pad_before_close: Rect,
    close: Rect,
}

pub fn render_bios_combo(
    area: Rect,
    label: &str,
    params: &mut ComboRenderParams<'_>,
    buf: &mut Buffer,
) -> ComboClickRegions {
    let text_w = resolve_text_slot_width(params.field, area.width);
    let layout = cluster_layout(area, text_w);
    let label_render_w = layout.cluster_x.saturating_sub(area.x);
    let embedded_inner = params.embedded == Some(ComboPart::Inner);
    let inner_active = params.selected && embedded_inner;
    let bg = cluster_bg(params.selected, inner_active);

    paint_label(
        Rect::new(area.x, area.y, label_render_w.max(1), 1),
        label,
        params.selected,
        inner_active,
        buf,
    );

    paint_cluster_background(
        Rect::new(layout.cluster_x, area.y, layout.cluster_w, 1),
        bg,
        buf,
    );

    paint_span(layout.open, OPEN, bracket_style(params.selected, inner_active, bg), buf);
    paint_span(
        layout.left,
        LEFT_ARROW,
        arrow_style(params.selected, inner_active, bg),
        buf,
    );
    paint_span(layout.pad_before_right, PAD, Style::default().bg(bg), buf);
    paint_span(
        layout.right,
        RIGHT_ARROW,
        arrow_style(params.selected, inner_active, bg),
        buf,
    );
    paint_span(
        layout.pad_before_close,
        PAD,
        Style::default().bg(bg),
        buf,
    );
    paint_span(
        layout.close,
        CLOSE,
        bracket_style(params.selected, inner_active, bg),
        buf,
    );
    paint_value_text(layout.text, params, inner_active, bg, buf);

    ComboClickRegions {
        label: Rect::new(area.x, area.y, label_render_w.max(1), 1),
        left: layout.left,
        right: layout.right,
        open: layout.open,
        text: layout.text,
        close: layout.close,
    }
}

fn cluster_layout(area: Rect, text_w: u16) -> ClusterLayout {
    let open_w = display_width(OPEN) as u16;
    let left_w = display_width(LEFT_ARROW) as u16;
    let pad_w = display_width(PAD) as u16;
    let right_w = display_width(RIGHT_ARROW) as u16;
    let close_w = display_width(CLOSE) as u16;
    let cluster_w = open_w
        .saturating_add(left_w)
        .saturating_add(text_w)
        .saturating_add(pad_w)
        .saturating_add(right_w)
        .saturating_add(pad_w)
        .saturating_add(close_w);
    let cluster_x = area.x.saturating_add(area.width.saturating_sub(cluster_w));
    let y = area.y;

    let mut x = cluster_x;
    let open = Rect::new(x, y, open_w.max(1), 1);
    x = x.saturating_add(open_w);
    let left = Rect::new(x, y, left_w.max(1), 1);
    x = x.saturating_add(left_w);
    let text = Rect::new(x, y, text_w, 1);
    x = x.saturating_add(text_w);
    let pad_before_right = Rect::new(x, y, pad_w.max(1), 1);
    x = x.saturating_add(pad_w);
    let right = Rect::new(x, y, right_w.max(1), 1);
    x = x.saturating_add(right_w);
    let pad_before_close = Rect::new(x, y, pad_w.max(1), 1);
    x = x.saturating_add(pad_w);
    let close = Rect::new(x, y, close_w.max(1), 1);

    ClusterLayout {
        cluster_x,
        cluster_w,
        open,
        left,
        text,
        pad_before_right,
        right,
        pad_before_close,
        close,
    }
}

fn display_width(text: &str) -> usize {
    text.width().max(1)
}

fn chrome_width_for_row(_row_width: u16) -> u16 {
    (display_width(OPEN)
        + display_width(LEFT_ARROW)
        + display_width(PAD)
        + display_width(PAD)
        + display_width(RIGHT_ARROW)
        + display_width(CLOSE)) as u16
}

fn resolve_text_slot_width(field: &FieldSpec, row_width: u16) -> u16 {
    let auto = max_combo_option_width(&field.kind).max(1) as u16;
    let requested = field.tui_combo_width.unwrap_or(auto);
    let max_text = row_width.saturating_sub(chrome_width_for_row(row_width)).max(1);
    requested.min(max_text)
}

fn paint_label(area: Rect, label: &str, selected: bool, inner_active: bool, buf: &mut Buffer) {
    let style = if selected {
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
    Paragraph::new(Line::from(Span::styled(
        clip_label(label, area.width as usize),
        style,
    )))
    .render(area, buf);
}

fn paint_span(area: Rect, text: &str, style: Style, buf: &mut Buffer) {
    buf.set_string(area.x, area.y, text, style);
}

fn paint_value_text(
    area: Rect,
    params: &mut ComboRenderParams<'_>,
    inner_active: bool,
    bg: ratatui::style::Color,
    buf: &mut Buffer,
) {
    let slot_w = area.width as usize;
    let value = params.value;
    let overflow = value.width() > slot_w;
    let style = if inner_active {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    } else if params.selected {
        Style::default()
            .fg(dhara_theme::TEXT)
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(dhara_theme::TEXT).bg(bg)
    };

    let display = if overflow && inner_active {
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

fn bracket_style(selected: bool, inner_active: bool, bg: ratatui::style::Color) -> Style {
    if inner_active {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    } else if selected {
        Style::default().fg(dhara_theme::COMBO_BORDER).bg(bg)
    } else {
        Style::default().fg(dhara_theme::COMBO_BORDER).bg(bg)
    }
}

/// Arrow buttons: **focus** when inner is active (accent, bold); **relax** otherwise (muted).
fn arrow_style(selected: bool, inner_active: bool, bg: ratatui::style::Color) -> Style {
    if inner_active {
        Style::default()
            .fg(dhara_theme::ACCENT)
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    } else if selected {
        Style::default().fg(dhara_theme::MUTED).bg(bg)
    } else {
        Style::default().fg(dhara_theme::MUTED).bg(dhara_theme::COMBO_BG)
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
    fn chrome_width_accounts_for_wide_brackets() {
        let chrome = chrome_width_for_row(80);
        assert!(chrome >= 8);
    }
}
