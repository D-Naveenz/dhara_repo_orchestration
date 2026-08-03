use drot_kernel::RunPhase;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Widget;
use ratatui_interact::components::{ButtonState, Spinner, SpinnerState};
use ratatui_interact::theme::Theme;
use ratatui_interact::traits::ClickRegionRegistry;

use drot_kernel::AppState;

use crate::focus::TuiFocus;
use crate::theme::ValidationTone;
use crate::widgets::{padded_button, panel, progress_bar, status_line};

pub fn render_action_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    shell_focus: &crate::focus::ShellFocus,
    run_btn: &mut ButtonState,
    spinner: &mut SpinnerState,
    shell_clicks: &mut ClickRegionRegistry<TuiFocus>,
) {
    let focused = shell_focus.is_focused(&TuiFocus::ActionRun);
    let inner = panel::render_panel(frame, area, "Actions", focused, true);

    let layout = Layout::vertical([
        Constraint::Length(1), // progress
        Constraint::Length(1), // status
        Constraint::Length(1), // spacer
        Constraint::Length(padded_button::BUTTON_SLOT_HEIGHT),
    ])
    .split(inner);

    render_progress(frame, layout[0], state);
    render_status(frame, layout[1], state, spinner, theme);
    // layout[2] is intentional blank spacer

    let running = state.active_run.is_some();
    let cancelable = state.active_run.as_ref().is_some_and(|run| run.cancelable);

    let (label, enabled) = if running {
        ("Cancel", cancelable)
    } else {
        ("Run", true)
    };
    run_btn.set_enabled(enabled);
    run_btn.set_focused(focused);

    let btn_width = padded_button::label_width(label)
        .max(padded_button::BUTTON_MIN_WIDTH)
        .min(layout[3].width);
    let btn_x = layout[3].x + layout[3].width.saturating_sub(btn_width) / 2;
    let btn_area = Rect::new(btn_x, layout[3].y, btn_width, 1);

    let buf = frame.buffer_mut();
    padded_button::render_padded_button(btn_area, label, run_btn, theme, buf);
    shell_clicks.register(btn_area, TuiFocus::ActionRun);
}

fn render_progress(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let ratio = match state.progress.as_ref() {
        Some(snapshot) if snapshot.phase == RunPhase::Analyzing => 0.0,
        Some(snapshot) => f64::from(snapshot.overall),
        None => 0.0,
    };
    let complete = state
        .progress
        .as_ref()
        .is_some_and(|snapshot| snapshot.phase == RunPhase::Complete);

    progress_bar::render_capsule_progress(area, ratio, complete, frame.buffer_mut());
}

fn format_status_line(state: &AppState) -> String {
    if let Some(snapshot) = state.progress.as_ref() {
        if snapshot.phase == RunPhase::Analyzing && !snapshot.analyzing_message.is_empty() {
            return snapshot.analyzing_message.clone();
        }
        if !snapshot.step_label.is_empty() {
            if let Some(secs) = snapshot.elapsed_secs {
                return format!("{}… ({secs}s)", snapshot.step_label.trim_end_matches('…'));
            }
            return snapshot.step_label.clone();
        }
        if !snapshot.activity_label.is_empty() {
            if let Some(secs) = snapshot.elapsed_secs {
                return format!("{}… ({secs}s)", snapshot.activity_label);
            }
            return format!("{}…", snapshot.activity_label);
        }
    }
    state.status_message.clone()
}

fn status_tone(state: &AppState) -> ValidationTone {
    match state.status_tone {
        drot_kernel::StatusTone::Ready => ValidationTone::Muted,
        drot_kernel::StatusTone::Running => ValidationTone::Muted,
        drot_kernel::StatusTone::Success => ValidationTone::Success,
        drot_kernel::StatusTone::Failed => ValidationTone::Error,
        drot_kernel::StatusTone::Warning => ValidationTone::Muted,
    }
}

fn render_status(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    spinner: &mut SpinnerState,
    theme: &Theme,
) {
    let status = format_status_line(state);
    if state.active_run.is_some() && area.width > 4 {
        let spin_area = Rect::new(area.x, area.y, 2, 1);
        Spinner::new(spinner)
            .theme(theme)
            .render(spin_area, frame.buffer_mut());
        status_line::render_status_line(
            Rect::new(area.x + 2, area.y, area.width.saturating_sub(2), 1),
            &status,
            status_tone(state),
            frame.buffer_mut(),
        );
    } else {
        status_line::render_status_line(area, &status, status_tone(state), frame.buffer_mut());
    }
}
