use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

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

pub fn render_group_border(area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(dhara_theme::BORDER))
        .style(Style::default().bg(dhara_theme::PANEL_BG))
        .render(area, buf);
}
