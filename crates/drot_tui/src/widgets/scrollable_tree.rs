use std::time::Instant;

use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui_interact::components::{TreeNode, TreeStyle, TreeView, TreeViewState};
use ratatui_interact::theme::Theme;
use unicode_width::UnicodeWidthStr;

use crate::theme as dhara_theme;
use crate::widgets::text_marquee::{LabelMarquee, clip_right, clip_window};

/// Width of the expand/collapse chevron column (matches `"▶ "` / `"▼ "`).
const CHEVRON_SPACER: &str = "  ";

pub type TreeLabelMarquee = LabelMarquee;

struct FlatNode<'a, T> {
    node: &'a TreeNode<T>,
    #[allow(dead_code)]
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

    if flat.node.has_children() {
        let connector = if flat.is_last {
            style.connector_last
        } else {
            style.connector_branch
        };
        prefix.push_str(connector);
        let icon = if widget.is_collapsed(&flat.node.id) {
            style.collapsed_icon
        } else {
            style.expanded_icon
        };
        prefix.push_str(icon);
    } else {
        prefix.push_str(CHEVRON_SPACER);
    }

    (prefix, row_style)
}

pub fn visible_count<T: std::fmt::Debug>(nodes: &[TreeNode<T>], state: &TreeViewState) -> usize {
    TreeView::new(nodes, state).visible_count()
}

#[allow(clippy::too_many_arguments)]
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
    let flat_nodes = flatten_visible(nodes, state);
    let count = flat_nodes.len();
    if count == 0 {
        return;
    }

    let scroll = state.scroll as usize;
    let visible_rows = area.height as usize;
    let now = Instant::now();

    for (row_idx, flat_node) in flat_nodes
        .iter()
        .skip(scroll)
        .take(visible_rows)
        .enumerate()
    {
        let row_y = area.y + row_idx as u16;
        let row_area = Rect::new(area.x, row_y, area.width, 1);
        let is_selected = scroll + row_idx == state.selected_index;
        let disabled = is_disabled(flat_node.node);

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
