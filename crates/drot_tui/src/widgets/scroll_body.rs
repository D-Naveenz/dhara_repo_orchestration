use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui_interact::components::{ScrollableContent, ScrollableContentState};

use crate::theme as dhara_theme;

pub fn render_scroll_body(
    area: Rect,
    scroll: &mut ScrollableContentState,
    buf: &mut ratatui::buffer::Buffer,
) {
    // Keep borderless: do not call `.theme()`, which re-enables Borders::ALL.
    ScrollableContent::new(scroll)
        .style(dhara_theme::scroll_body_style())
        .render(area, buf);
}

/// Inset a tab body so text does not touch the panel border.
pub fn inset_body(area: Rect) -> Rect {
    let x = area.x.saturating_add(1);
    let y = area.y.saturating_add(1);
    let width = area.width.saturating_sub(2);
    let height = area.height.saturating_sub(2);
    Rect::new(x, y, width, height)
}
