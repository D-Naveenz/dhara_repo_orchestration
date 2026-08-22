use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::theme as dhara_theme;

pub fn render_group_title(area: Rect, title: &str, buf: &mut Buffer) -> u16 {
    if title.is_empty() {
        return 0;
    }
    Paragraph::new(Line::from(Span::styled(
        title,
        Style::default()
            .fg(dhara_theme::ACCENT)
            .add_modifier(ratatui::style::Modifier::BOLD),
    )))
    .render(area, buf);
    1
}
