pub mod modals;

use drot_kernel::FormValue;
use drot_kernel::forms::{preset_id, preset_label};
use drot_kernel::{AppState, DiagnosticSeverity, MainTab};
use drot_kernel::{CommandRegistry, CommandSpec, FieldKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};
use ratatui_interact::components::{
    ButtonState, CheckBox, CheckBoxState, InputState, ScrollableContentState, Tab, TabViewAction,
    TabViewState,
};
use ratatui_interact::theme::Theme;
use ratatui_interact::traits::ClickRegionRegistry;
use std::time::Instant;

use crate::embedded::OptionFieldAction;
use crate::focus::TuiFocus;
use crate::strings::{self, t};
use crate::theme as dhara_theme;
use crate::widgets::{
    ComboPart, ComboRenderParams, bios_combo, bios_textbox, button_group, padded_button, panel,
    scroll_body, tab_table, text_marquee::LabelMarquee, text_wrap,
};

pub struct CenterPanelClicks {
    pub registry: ClickRegionRegistry<TabViewAction>,
}

pub fn sync_tab_view_from_state(tab_state: &mut TabViewState, main_tab: MainTab) {
    tab_state.select(tab_index(main_tab));
}

pub fn sync_state_from_tab_view(tab_state: &TabViewState) -> MainTab {
    tab_from_index(tab_state.selected_index)
}

#[allow(clippy::too_many_arguments)] // Center panel owns tabs, scrolls, and options widgets in one pass.
pub fn render_center_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    registry: &CommandRegistry,
    theme: &Theme,
    tab_state: &mut TabViewState,
    shell_focus: &crate::focus::ShellFocus,
    info_scroll: &mut ScrollableContentState,
    trouble_scroll: &mut ScrollableContentState,
    system_scroll: &mut ScrollableContentState,
    form_field: usize,
    editing_form: bool,
    embedded_focus: Option<crate::embedded::EmbeddedFocus>,
    option_input: &InputState,
    option_checkbox: &CheckBoxState,
    option_field_clicks: &mut ClickRegionRegistry<OptionFieldAction>,
    reset_btn: &mut ButtonState,
    shell_clicks: &mut ClickRegionRegistry<TuiFocus>,
    combo_marquee: &mut LabelMarquee,
    now: Instant,
) -> CenterPanelClicks {
    sync_tab_view_from_state(tab_state, state.main_tab);
    tab_state.focused = shell_focus.is_focused(&TuiFocus::MainTabs)
        || shell_focus.is_focused(&TuiFocus::TabContent)
        || shell_focus.is_focused(&TuiFocus::OptionsReset);

    let tabs_focused = shell_focus.is_focused(&TuiFocus::MainTabs)
        || shell_focus.is_focused(&TuiFocus::TabContent)
        || shell_focus.is_focused(&TuiFocus::OptionsReset);
    let panel_inner = panel::render_panel(frame, area, "", tabs_focused, false);

    let layout = tab_table::split_tab_table(panel_inner);
    let tab_labels = strings::tab_labels();
    let tabs: Vec<Tab<'_>> = tab_labels.iter().map(|label| Tab::new(label)).collect();

    let mut click_registry = ClickRegionRegistry::new();
    option_field_clicks.clear();
    tab_table::render_tab_header(
        layout.header,
        &tabs,
        tab_state,
        theme,
        &mut click_registry,
        frame.buffer_mut(),
    );
    tab_table::render_tab_separator(
        frame,
        Rect::new(panel_inner.x, panel_inner.y + 1, panel_inner.width, 1),
    );

    let body = scroll_body::inset_body(layout.body);
    let selected_tab = tab_state.selected_index;
    let content_focused = shell_focus.is_focused(&TuiFocus::TabContent);
    info_scroll.set_focused(content_focused && selected_tab == 0);
    trouble_scroll.set_focused(content_focused && selected_tab == 2);
    system_scroll.set_focused(content_focused && selected_tab == 3);

    match selected_tab {
        0 => render_info_tab(body, frame.buffer_mut(), state, registry, info_scroll),
        1 => render_options_tab(
            frame,
            body,
            state,
            registry,
            form_field,
            editing_form,
            embedded_focus,
            option_input,
            option_checkbox,
            theme,
            content_focused,
            option_field_clicks,
            reset_btn,
            shell_focus,
            shell_clicks,
            combo_marquee,
            now,
        ),
        2 => render_trouble_tab(body, frame.buffer_mut(), state, trouble_scroll),
        3 => render_system_tab(body, frame.buffer_mut(), state, system_scroll),
        _ => {}
    }

    CenterPanelClicks {
        registry: click_registry,
    }
}

fn render_info_tab(
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    state: &AppState,
    registry: &CommandRegistry,
    scroll: &mut ScrollableContentState,
) {
    let Some(command) = state.selected_command(registry) else {
        Paragraph::new(t("doc.empty")).render(area, buf);
        return;
    };

    let width = area.width as usize;
    let mut lines: Vec<String> = Vec::new();

    // Title — CLI path (Learn-style primary heading).
    let title = command.path_string();
    lines.extend(text_wrap::wrap_line(&title, width));

    // Lead — one-line summary.
    lines.push(String::new());
    lines.extend(text_wrap::wrap_line(command.summary, width));

    // Description — fuller prose when it adds more than the summary.
    if !command.ui.description.is_empty() && command.ui.description != command.summary {
        lines.push(String::new());
        lines.extend(text_wrap::wrap_paragraphs(command.ui.description, width));
    }

    if command.is_effectively_disabled() {
        lines.push(String::new());
        lines.extend(text_wrap::wrap_line(
            &format!("Disabled — {}", command.disable_message()),
            width,
        ));
    }

    // Syntax
    lines.push(String::new());
    lines.extend(text_wrap::wrap_line(t("doc.syntax"), width));
    let syntax = if command.args_summary.is_empty() {
        command.path_string()
    } else {
        format!("{} {}", command.path_string(), command.args_summary)
    };
    lines.extend(text_wrap::wrap_line(&format!("  {syntax}"), width));

    // CLI flags reference (Options tab is the primary human surface).
    if !command.ui.fields.is_empty() {
        lines.push(String::new());
        lines.extend(text_wrap::wrap_line(t("doc.cli_flags"), width));
        let syntax = if command.args_summary.is_empty() {
            command.path_string()
        } else {
            format!("{} {}", command.path_string(), command.args_summary)
        };
        lines.extend(text_wrap::wrap_line(&format!("  {syntax}"), width));
    }

    scroll.set_lines(lines);
    scroll_body::render_scroll_body(area, scroll, buf);
}

#[allow(clippy::too_many_arguments)] // Options tab wires form widgets and click regions together.
fn render_options_tab(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    registry: &CommandRegistry,
    form_field: usize,
    editing_form: bool,
    embedded_focus: Option<crate::embedded::EmbeddedFocus>,
    option_input: &InputState,
    _option_checkbox: &CheckBoxState,
    theme: &Theme,
    content_focused: bool,
    option_field_clicks: &mut ClickRegionRegistry<OptionFieldAction>,
    reset_btn: &mut ButtonState,
    shell_focus: &crate::focus::ShellFocus,
    shell_clicks: &mut ClickRegionRegistry<TuiFocus>,
    combo_marquee: &mut LabelMarquee,
    now: Instant,
) {
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
    let fields_area = chunks[0];
    let footer = chunks[1];

    let running = state.active_run.is_some();
    let can_reset = state.selected_command(registry).is_some() && !running;
    reset_btn.set_enabled(can_reset);
    reset_btn.set_focused(shell_focus.is_focused(&TuiFocus::OptionsReset));

    let reset_w = padded_button::label_width("Reset").max(padded_button::BUTTON_MIN_WIDTH);
    let reset_w = reset_w.min(footer.width);
    let reset_area = Rect::new(
        footer.x + footer.width.saturating_sub(reset_w),
        footer.y,
        reset_w,
        1,
    );
    padded_button::render_padded_button(reset_area, "Reset", reset_btn, theme, frame.buffer_mut());
    shell_clicks.register(reset_area, TuiFocus::OptionsReset);

    let Some(command) = state.selected_command(registry) else {
        Paragraph::new(t("options.empty")).render(fields_area, frame.buffer_mut());
        return;
    };
    let Some(form) = state.forms.get(command.id) else {
        Paragraph::new(t("options.loading")).render(fields_area, frame.buffer_mut());
        return;
    };

    if command.ui.fields.is_empty() {
        Paragraph::new(t("options.none")).render(fields_area, frame.buffer_mut());
        return;
    }

    let mut y = fields_area.y;
    let mut current_group: Option<&str> = None;
    const GROUP_INDENT: u16 = 2;

    for (index, field) in command.ui.fields.iter().enumerate() {
        if y >= fields_area.y + fields_area.height {
            break;
        }

        if current_group != field.group {
            if current_group.is_some() {
                y += 1;
            }
            current_group = field.group;
            if let Some(title) = field.group {
                if y < fields_area.y + fields_area.height {
                    button_group::render_group_title(
                        Rect::new(fields_area.x, y, fields_area.width, 1),
                        title,
                        frame.buffer_mut(),
                    );
                    y += 1;
                }
            }
        }

        let nest = u16::from(field.tui_nest).saturating_mul(GROUP_INDENT);
        let row = Rect::new(
            fields_area
                .x
                .saturating_add(GROUP_INDENT)
                .saturating_add(nest),
            y,
            fields_area
                .width
                .saturating_sub(GROUP_INDENT)
                .saturating_sub(nest),
            1,
        );
        render_form_field(
            frame,
            row,
            index,
            field,
            &form.values[index],
            index == form_field,
            content_focused,
            editing_form,
            embedded_focus,
            option_input,
            theme,
            option_field_clicks,
            command.id,
            combo_marquee,
            now,
        );
        y += 1;
    }
}

fn register_combo_clicks(
    option_field_clicks: &mut ClickRegionRegistry<OptionFieldAction>,
    regions: &bios_combo::ComboClickRegions,
    index: usize,
) {
    let inner = OptionFieldAction::ComboPart {
        field: index,
        part: ComboPart::Inner,
    };
    // Buttons first — registry returns the first matching region.
    option_field_clicks.register(
        regions.left,
        OptionFieldAction::ComboPart {
            field: index,
            part: ComboPart::Left,
        },
    );
    option_field_clicks.register(
        regions.right,
        OptionFieldAction::ComboPart {
            field: index,
            part: ComboPart::Right,
        },
    );
    option_field_clicks.register(regions.inner, inner);
    option_field_clicks.register(regions.text, inner);
    option_field_clicks.register(regions.label, OptionFieldAction::Field(index));
}

#[allow(clippy::too_many_arguments)]
fn render_form_field(
    frame: &mut Frame,
    row: Rect,
    index: usize,
    field: &drot_kernel::FieldSpec,
    value: &FormValue,
    selected: bool,
    content_focused: bool,
    editing_form: bool,
    embedded_focus: Option<crate::embedded::EmbeddedFocus>,
    option_input: &InputState,
    theme: &Theme,
    option_field_clicks: &mut ClickRegionRegistry<OptionFieldAction>,
    command_id: &'static str,
    combo_marquee: &mut LabelMarquee,
    now: Instant,
) {
    let _ = editing_form;
    let embedded_combo = match embedded_focus {
        Some(crate::embedded::EmbeddedFocus::Combo { field_index, part })
            if field_index == index =>
        {
            Some(part)
        }
        _ => None,
    };
    let embedded_text = matches!(
        embedded_focus,
        Some(crate::embedded::EmbeddedFocus::Text { field_index }) if field_index == index
    );

    match (&field.kind, value) {
        (FieldKind::Boolean, FormValue::Boolean(checked)) => {
            let mut cb = CheckBoxState::new(*checked);
            cb.set_focused(selected && content_focused);
            let region = CheckBox::new(field.label, &cb)
                .theme(theme)
                .render_stateful(row, frame.buffer_mut());
            option_field_clicks.register(region.area, OptionFieldAction::Field(index));
        }
        (FieldKind::Radio(options), FormValue::Select(sel)) => {
            let label = options.get(*sel).copied().unwrap_or("");
            let row_selected = selected && content_focused;
            let marquee_key = format!("{command_id}:{index}");
            let mut params = ComboRenderParams {
                field,
                value: label,
                selected: row_selected,
                embedded: embedded_combo,
                marquee: combo_marquee,
                marquee_key: &marquee_key,
                now,
            };
            let regions =
                bios_combo::render_bios_combo(row, field.label, &mut params, frame.buffer_mut());
            register_combo_clicks(option_field_clicks, &regions, index);
        }
        (
            FieldKind::Combo(_) | FieldKind::Select(_) | FieldKind::Preset(_),
            FormValue::Select(sel),
        ) => {
            let label = preset_label(&field.kind, *sel);
            let row_selected = selected && content_focused;
            let marquee_key = format!("{command_id}:{index}");
            let mut params = ComboRenderParams {
                field,
                value: label,
                selected: row_selected,
                embedded: embedded_combo,
                marquee: combo_marquee,
                marquee_key: &marquee_key,
                now,
            };
            let regions =
                bios_combo::render_bios_combo(row, field.label, &mut params, frame.buffer_mut());
            register_combo_clicks(option_field_clicks, &regions, index);
        }
        (
            FieldKind::Text | FieldKind::Path | FieldKind::BrowsablePath { .. },
            FormValue::Text(text),
        ) => {
            let display = if embedded_text {
                option_input.text()
            } else {
                text.as_str()
            };
            let cursor_pos = if embedded_text {
                option_input.cursor_pos
            } else {
                display.chars().count()
            };
            let result = bios_textbox::render_bios_textbox(
                row,
                field.label,
                display,
                selected && content_focused,
                embedded_text && content_focused,
                cursor_pos,
                frame.buffer_mut(),
            );
            if let Some(pos) = result.cursor {
                frame.set_cursor_position(pos);
            }
            option_field_clicks.register(result.regions.input, OptionFieldAction::Field(index));
            option_field_clicks.register(result.regions.label, OptionFieldAction::Field(index));
            option_field_clicks.register(result.regions.row, OptionFieldAction::Field(index));
        }
        _ => {
            Paragraph::new(Line::styled(
                format!("  {}: (unsupported)", field.label),
                Style::default().fg(dhara_theme::TEXT),
            ))
            .render(row, frame.buffer_mut());
        }
    }
}

fn render_trouble_tab(
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    state: &AppState,
    scroll: &mut ScrollableContentState,
) {
    if state.troubleshooting_lines.is_empty() {
        Paragraph::new(t("trouble.empty")).render(area, buf);
        return;
    }
    let width = area.width as usize;
    let mut lines: Vec<String> = Vec::new();
    for line in &state.troubleshooting_lines {
        if line.continuation {
            let text = format!("      {}", line.text);
            lines.extend(text_wrap::wrap_line(&text, width));
            continue;
        }
        let prefix = match line.severity {
            DiagnosticSeverity::Warn => "WARN",
            DiagnosticSeverity::Error => "ERR ",
        };
        let text = format!("[{prefix}] {}", line.text);
        lines.extend(text_wrap::wrap_line(&text, width));
    }
    scroll.set_lines(lines);
    scroll_body::render_scroll_body(area, scroll, buf);
}

fn render_system_tab(
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    state: &AppState,
    scroll: &mut ScrollableContentState,
) {
    let text = state
        .system_configs_text
        .as_deref()
        .unwrap_or(t("system.empty"));
    let width = area.width as usize;
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            lines.push(String::new());
        } else {
            lines.extend(text_wrap::wrap_line(line, width));
        }
    }
    scroll.set_lines(lines);
    scroll_body::render_scroll_body(area, scroll, buf);
}

pub fn tab_from_index(index: usize) -> MainTab {
    match index {
        1 => MainTab::Options,
        2 => MainTab::Troubleshooting,
        3 => MainTab::SystemConfigs,
        _ => MainTab::Info,
    }
}

pub fn tab_index(tab: MainTab) -> usize {
    match tab {
        MainTab::Info => 0,
        MainTab::Options => 1,
        MainTab::Troubleshooting => 2,
        MainTab::SystemConfigs => 3,
    }
}

pub fn cycle_form_field(command: &CommandSpec, form_field: &mut usize, delta: isize) {
    let count = command.ui.fields.len();
    if count == 0 {
        return;
    }
    let next = (*form_field as isize + delta).rem_euclid(count as isize) as usize;
    *form_field = next;
}

pub fn sync_option_widgets_from_form(
    state: &AppState,
    registry: &CommandRegistry,
    form_field: usize,
    option_input: &mut InputState,
    option_checkbox: &mut CheckBoxState,
) {
    let Some(command) = state.selected_command(registry) else {
        return;
    };
    let Some(form) = state.forms.get(command.id) else {
        return;
    };
    let Some(field) = command.ui.fields.get(form_field) else {
        return;
    };
    match (&form.values[form_field], &field.kind) {
        (
            FormValue::Text(text),
            FieldKind::Text | FieldKind::Path | FieldKind::BrowsablePath { .. },
        ) => {
            *option_input = InputState::new(text);
        }
        (FormValue::Boolean(value), FieldKind::Boolean) => {
            *option_checkbox = CheckBoxState::new(*value);
        }
        _ => {}
    }
}

pub fn apply_option_widgets_to_form(
    state: &mut AppState,
    registry: &CommandRegistry,
    form_field: usize,
    option_input: &InputState,
    option_checkbox: &CheckBoxState,
) {
    let Some(command) = state.selected_command(registry).cloned() else {
        return;
    };
    let Some(form) = state.forms.get_mut(command.id) else {
        return;
    };
    let Some(field) = command.ui.fields.get(form_field) else {
        return;
    };
    match (&mut form.values[form_field], &field.kind) {
        (
            FormValue::Text(value),
            FieldKind::Text | FieldKind::Path | FieldKind::BrowsablePath { .. },
        ) => {
            *value = option_input.text().to_owned();
        }
        (FormValue::Boolean(value), FieldKind::Boolean) => {
            *value = option_checkbox.checked;
        }
        _ => {}
    }
}

pub fn apply_preset_if_selected(
    state: &mut AppState,
    registry: &CommandRegistry,
    field_index: usize,
) {
    let Some(command) = state.selected_command(registry).cloned() else {
        return;
    };
    let Some(field) = command.ui.fields.get(field_index) else {
        return;
    };
    if !matches!(field.kind, FieldKind::Preset(_)) {
        return;
    }
    let Some(form) = state.forms.get(command.id) else {
        return;
    };
    let FormValue::Select(index) = form.values[field_index] else {
        return;
    };
    let Some(selected_preset) = preset_id(&field.kind, index) else {
        return;
    };
    if selected_preset == "custom" {
        return;
    }
    let Some(form) = state.forms.get_mut(command.id) else {
        return;
    };
    if let Some(hooks) = drot_kernel::product_hooks() {
        hooks.apply_tui_preset(form, &command, selected_preset);
    }
}
