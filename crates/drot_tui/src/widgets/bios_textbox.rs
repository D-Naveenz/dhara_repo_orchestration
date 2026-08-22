use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::theme as dhara_theme;

pub struct TextboxClickRegions {
    pub input: Rect,
    pub row: Rect,
}

pub fn render_bios_textbox(
    area: Rect,
    label: &str,
    value: &str,
    selected: bool,
    embedded: bool,
    buf: &mut Buffer,
) -> TextboxClickRegions {
    let prompt = if selected {
        Span::styled("> ", Style::default().fg(dhara_theme::ACCENT).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    let label_style = if selected {
        Style::default().fg(dhara_theme::ACCENT)
    } else {
        Style::default().fg(dhara_theme::TEXT)
    };
    let label_text = format!("{label} ");
    let prefix_width: u16 = 2;
    let label_width = label_text.chars().count() as u16;

    Paragraph::new(Line::from(vec![
        prompt,
        Span::styled(label_text, label_style),
    ]))
    .render(
        Rect::new(area.x, area.y, (prefix_width + label_width).min(area.width), 1),
        buf,
    );

    let input_x = area.x.saturating_add((prefix_width + label_width).min(area.width));
    let input_w = area
        .width
        .saturating_sub((prefix_width + label_width).min(area.width))
        .max(4);
    let input_prompt = ">_";
    let display = if value.is_empty() {
        input_prompt.to_owned()
    } else {
        format!("{input_prompt}{value}")
    };

    let style = if embedded {
        dhara_theme::selected_style()
    } else if selected {
        Style::default().fg(dhara_theme::ACCENT)
    } else {
        Style::default().fg(dhara_theme::TEXT)
    };

    let line = format!("【{}】", truncate(&display, input_w.saturating_sub(2) as usize));
    Paragraph::new(Line::from(Span::styled(line, style))).render(
        Rect::new(input_x, area.y, input_w, 1),
        buf,
    );

    TextboxClickRegions {
        input: Rect::new(input_x, area.y, input_w, 1),
        row: area,
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
