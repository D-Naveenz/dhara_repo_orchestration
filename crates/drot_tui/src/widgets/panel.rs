use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};

use crate::theme as dhara_theme;

pub fn render_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    focused: bool,
    fill_panel_bg: bool,
) -> Rect {
    let border_style = if focused {
        dhara_theme::border_style().fg(dhara_theme::ACCENT)
    } else {
        dhara_theme::border_style()
    };
    let style = if fill_panel_bg {
        dhara_theme::panel_style()
    } else {
        dhara_theme::border_only_style()
    };

    let block = if title.trim().is_empty() {
        // Closed box — no title gap when the panel is untitled.
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(style)
    } else {
        Block::default()
            .title(format!(" {title} "))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(style)
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}
