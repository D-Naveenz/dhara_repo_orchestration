use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui_interact::components::ButtonState;
use ratatui_interact::theme::Theme;
use unicode_width::UnicodeWidthStr;

pub const BUTTON_SLOT_HEIGHT: u16 = 1;
pub const BUTTON_MIN_WIDTH: u16 = 10;
pub const BUTTON_GAP: u16 = 1;

fn button_style(state: &ButtonState, theme: &Theme) -> Style {
    let palette = &theme.palette;
    if !state.enabled {
        return Style::default().fg(palette.text_disabled);
    }
    if state.focused {
        return Style::default()
            .fg(palette.highlight_fg)
            .bg(palette.highlight_bg)
            .add_modifier(Modifier::BOLD);
    }
    Style::default().fg(palette.text)
}

/// Exact bracket label with one space on each side of `label`.
pub fn bracket_label(label: &str) -> String {
    format!("[ {label} ]")
}

/// Display width of `[ label ]`.
pub fn label_width(label: &str) -> u16 {
    bracket_label(label).width() as u16
}

/// One-line bracket button: `[ label ]`, placed at an exact column (no wide Paragraph center).
pub fn render_padded_button(
    area: Rect,
    label: &str,
    state: &ButtonState,
    theme: &Theme,
    buf: &mut ratatui::buffer::Buffer,
) -> Rect {
    if area.width == 0 || area.height == 0 {
        return area;
    }

    let text = bracket_label(label);
    let text_width = text.width() as u16;
    if text_width == 0 {
        return area;
    }

    let paint_width = text_width.min(area.width);
    let x = area.x + area.width.saturating_sub(paint_width) / 2;
    let style = button_style(state, theme);

    // Clear only the button text span so leftover cells do not look like extra padding.
    for col in x..x + paint_width {
        buf[(col, area.y)].set_char(' ').set_style(Style::default());
    }

    buf.set_string(x, area.y, &text, style);
    area
}
