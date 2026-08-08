use std::time::{Duration, Instant};

use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui_interact::components::{TreeNode, TreeStyle, TreeView, TreeViewState};
use ratatui_interact::theme::Theme;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::theme as dhara_theme;

/// Width of the expand/collapse chevron column (matches `"▶ "` / `"▼ "`).
const CHEVRON_SPACER: &str = "  ";

/// Delay between marquee column steps.
const MARQUEE_STEP: Duration = Duration::from_millis(120);
/// Hold at each end before reversing direction.
const MARQUEE_PAUSE: Duration = Duration::from_millis(800);

/// Ping-pong horizontal scroll for the selected overflowing tree label.
#[derive(Debug, Clone)]
pub struct TreeLabelMarquee {
    path_key: String,
    offset: usize,
    forward: bool,
    next_step: Instant,
    pause_until: Instant,
}

impl Default for TreeLabelMarquee {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeLabelMarquee {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            path_key: String::new(),
            offset: 0,
            forward: true,
            next_step: now,
            pause_until: now + MARQUEE_PAUSE,
        }
    }

    fn reset(&mut self, path_key: &str, now: Instant) {
        self.path_key = path_key.to_owned();
        self.offset = 0;
        self.forward = true;
        self.next_step = now;
        self.pause_until = now + MARQUEE_PAUSE;
    }

    /// Advance ping-pong state for the selected label; returns the column offset to paint.
    pub fn advance(
        &mut self,
        path_key: &str,
        label: &str,
        max_width: usize,
        now: Instant,
    ) -> usize {
        let overflow = label.width().saturating_sub(max_width);
        if overflow == 0 {
            self.reset(path_key, now);
            return 0;
        }

        if self.path_key != path_key {
            self.reset(path_key, now);
            return 0;
        }

        self.offset = self.offset.min(overflow);

        if now < self.pause_until {
            return self.offset;
        }

        if now < self.next_step {
            return self.offset;
        }

        // Catch up one step per frame so fast redraws do not skip the text.
        self.next_step = now + MARQUEE_STEP;

        if self.forward {
            if self.offset >= overflow {
                self.forward = false;
                self.pause_until = now + MARQUEE_PAUSE;
            } else {
                self.offset += 1;
                if self.offset >= overflow {
                    self.forward = false;
                    self.pause_until = now + MARQUEE_PAUSE;
                }
            }
        } else if self.offset == 0 {
            self.forward = true;
            self.pause_until = now + MARQUEE_PAUSE;
        } else {
            self.offset -= 1;
            if self.offset == 0 {
                self.forward = true;
                self.pause_until = now + MARQUEE_PAUSE;
            }
        }

        self.offset
    }
}

struct FlatNode<'a, T> {
    node: &'a TreeNode<T>,
    depth: usize,
    is_last: bool,
    parent_is_last: Vec<bool>,
}

fn flatten_visible<'a, T>(nodes: &'a [TreeNode<T>], state: &TreeViewState) -> Vec<FlatNode<'a, T>>
where
    T: std::fmt::Debug,
{
    let mut result = Vec::new();
    flatten_nodes(nodes, state, 0, &mut result, &[]);
    result
}

fn flatten_nodes<'a, T>(
    nodes: &'a [TreeNode<T>],
    state: &TreeViewState,
    depth: usize,
    result: &mut Vec<FlatNode<'a, T>>,
    parent_is_last: &[bool],
) where
    T: std::fmt::Debug,
{
    let count = nodes.len();
    for (idx, node) in nodes.iter().enumerate() {
        let is_last = idx == count - 1;
        result.push(FlatNode {
            node,
            depth,
            is_last,
            parent_is_last: parent_is_last.to_vec(),
        });
        if node.has_children() && !state.is_collapsed(&node.id) {
            let mut parents = parent_is_last.to_vec();
            parents.push(is_last);
            flatten_nodes(&node.children, state, depth + 1, result, &parents);
        }
    }
}

/// Compact file-explorer style: 2-col indent, chevron icons, no box-drawing connectors.
fn tree_style(theme: &Theme) -> TreeStyle {
    let mut style = TreeStyle::minimal();
    style.selected_style = dhara_theme::tree_selected_style();
    style.normal_style = Style::default().fg(dhara_theme::TEXT);
    style.connector_style = Style::default().fg(dhara_theme::MUTED);
    style.icon_style = Style::default().fg(dhara_theme::ACCENT);
    style.collapsed_icon = "▶ ";
    style.expanded_icon = "▼ ";
    style.cursor_normal = "";
    style.cursor_selected = "";
    let _ = theme;
    style
}

fn build_prefix<T>(
    style: &TreeStyle,
    flat: &FlatNode<'_, T>,
    widget: &TreeViewState,
) -> (String, Style) {
    let mut prefix = String::new();
    let row_style = style.normal_style;

    for &parent_is_last in &flat.parent_is_last {
        let connector = if parent_is_last {
            style.connector_space
        } else {
            style.connector_vertical
        };
        prefix.push_str(connector);
    }

    if flat.depth > 0 {
        let connector = if flat.is_last {
            style.connector_last
        } else {
            style.connector_branch
        };
        prefix.push_str(connector);
    }

    if flat.node.has_children() {
        let icon = if widget.is_collapsed(&flat.node.id) {
            style.collapsed_icon
        } else {
            style.expanded_icon
        };
        prefix.push_str(icon);
    } else {
        // Reserve chevron width so leaf labels align under parents.
        prefix.push_str(CHEVRON_SPACER);
    }

    (prefix, row_style)
}

/// Truncate on the right to `max_width` display columns, appending `…` when clipped.
fn clip_right(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let full_width = text.width();
    if full_width <= max_width {
        return text.to_owned();
    }

    let ellipsis = '…';
    let ellipsis_width = ellipsis.width().unwrap_or(1);
    if max_width <= ellipsis_width {
        return ellipsis.to_string();
    }

    let budget = max_width - ellipsis_width;
    let mut out = String::new();
    let mut width = 0usize;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if width + w > budget {
            break;
        }
        out.push(ch);
        width += w;
    }
    out.push(ellipsis);
    out
}

/// Skip `offset` display columns, then take up to `max_width` (no ellipsis).
fn clip_window(text: &str, offset: usize, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let mut skipped = 0usize;
    let mut started = offset == 0;
    let mut out = String::new();
    let mut width = 0usize;

    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if !started {
            if skipped + w > offset {
                // Wide char straddles the window start — advance past it.
                skipped += w;
                started = true;
                continue;
            }
            skipped += w;
            if skipped == offset {
                started = true;
            }
            continue;
        }

        if width + w > max_width {
            break;
        }
        out.push(ch);
        width += w;
    }

    out
}

pub fn visible_count<T: std::fmt::Debug>(nodes: &[TreeNode<T>], state: &TreeViewState) -> usize {
    TreeView::new(nodes, state).visible_count()
}

#[allow(clippy::too_many_arguments)] // Tree renderer takes layout, labels, and theme together.
pub fn render_clipped_tree<T: std::fmt::Debug>(
    area: Rect,
    nodes: &[TreeNode<T>],
    state: &TreeViewState,
    label: impl Fn(&TreeNode<T>) -> &str,
    path_key: impl Fn(&TreeNode<T>) -> &str,
    is_disabled: impl Fn(&TreeNode<T>) -> bool,
    marquee: &mut TreeLabelMarquee,
    theme: &Theme,
    buf: &mut ratatui::buffer::Buffer,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let style = tree_style(theme);
    let visible = flatten_visible(nodes, state);
    let scroll = state.scroll as usize;
    let viewport_height = area.height as usize;
    let now = Instant::now();

    for (view_idx, flat_node) in visible
        .iter()
        .enumerate()
        .skip(scroll)
        .take(viewport_height)
    {
        let is_selected = view_idx == state.selected_index;
        let disabled = is_disabled(flat_node.node);
        let row_y = area.y + (view_idx - scroll) as u16;
        let row_area = Rect::new(area.x, row_y, area.width, 1);
        let row_style = if is_selected {
            if disabled {
                dhara_theme::tree_disabled_selected_style()
            } else {
                style.selected_style
            }
        } else if disabled {
            dhara_theme::tree_disabled_style()
        } else {
            style.normal_style
        };

        if is_selected {
            let fg = if disabled {
                dhara_theme::MUTED
            } else {
                dhara_theme::WARNING
            };
            for x in row_area.x..row_area.x + row_area.width {
                buf[(x, row_y)].set_bg(dhara_theme::SELECTED_BG).set_fg(fg);
            }
        }

        let (prefix, prefix_style) = build_prefix(&style, flat_node, state);
        let prefix_paint = if disabled {
            dhara_theme::tree_disabled_style()
        } else {
            prefix_style
        };
        let prefix_width = prefix.width();
        buf.set_string(row_area.x, row_area.y, &prefix, prefix_paint);

        let label_width = row_area.width.saturating_sub(prefix_width as u16) as usize;
        if label_width == 0 {
            continue;
        }

        let text = label(flat_node.node);
        let clipped = if is_selected {
            let offset = marquee.advance(path_key(flat_node.node), text, label_width, now);
            if text.width() > label_width {
                clip_window(text, offset, label_width)
            } else {
                text.to_owned()
            }
        } else {
            clip_right(text, label_width)
        };
        buf.set_string(
            row_area.x + prefix_width as u16,
            row_area.y,
            &clipped,
            row_style,
        );
    }
}

pub fn handle_tree_wheel<T: std::fmt::Debug>(
    widget: &mut TreeViewState,
    nodes: &[TreeNode<T>],
    inner: Rect,
    mouse: &MouseEvent,
) -> bool {
    if inner.width == 0
        || inner.height == 0
        || mouse.column < inner.x
        || mouse.column >= inner.x + inner.width
        || mouse.row < inner.y
        || mouse.row >= inner.y + inner.height
    {
        return false;
    }

    let count = visible_count(nodes, widget);
    if count == 0 {
        return false;
    }

    match mouse.kind {
        MouseEventKind::ScrollUp => {
            if widget.scroll > 0 {
                widget.scroll -= 1;
            } else if widget.selected_index > 0 {
                widget.selected_index -= 1;
            }
            widget.ensure_visible(count);
            true
        }
        MouseEventKind::ScrollDown => {
            let max_scroll = count.saturating_sub(inner.height as usize) as u16;
            if widget.scroll < max_scroll {
                widget.scroll += 1;
            } else if widget.selected_index + 1 < count {
                widget.selected_index += 1;
            }
            widget.ensure_visible(count);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_window_skips_offset_and_limits_width() {
        assert_eq!(clip_window("abcdefghij", 0, 4), "abcd");
        assert_eq!(clip_window("abcdefghij", 3, 4), "defg");
        assert_eq!(clip_window("abcdefghij", 8, 4), "ij");
        assert_eq!(clip_window("short", 0, 20), "short");
    }

    #[test]
    fn marquee_resets_on_path_change_and_steps_forward() {
        let mut m = TreeLabelMarquee::new();
        let t0 = Instant::now();
        assert_eq!(m.advance("a", "abcdefghijklmnopqrstuvwxyz", 5, t0), 0);

        let t1 = t0 + MARQUEE_PAUSE + MARQUEE_STEP;
        assert_eq!(m.advance("a", "abcdefghijklmnopqrstuvwxyz", 5, t1), 1);

        let t2 = t1 + MARQUEE_STEP;
        assert_eq!(m.advance("b", "abcdefghijklmnopqrstuvwxyz", 5, t2), 0);
    }
}
