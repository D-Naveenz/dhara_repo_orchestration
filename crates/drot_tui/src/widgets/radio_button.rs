use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::theme as dhara_theme;

pub fn render_radio_row(
    area: Rect,
    label: &str,
    checked: bool,
    selected: bool,
    buf: &mut Buffer,
) {
    let marker = if checked { "[*]" } else { "[ ]" };
    let prefix = if selected { "▸ " } else { "  " };
    let text = format!("{prefix}{marker} {label}");
    let style = if selected {
        dhara_theme::selected_style()
    } else if checked {
        Style::default().fg(dhara_theme::SUCCESS)
    } else {
        Style::default().fg(dhara_theme::TEXT)
    };
    Paragraph::new(Line::from(Span::styled(text, style))).render(area, buf);
}
