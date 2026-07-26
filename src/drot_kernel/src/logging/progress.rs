use std::cell::Cell;
use std::io::IsTerminal;
use std::sync::OnceLock;
use std::time::Instant;

use crate::context::{RunMode, ToolContext};

static PROGRESS_SETTINGS: OnceLock<ProgressSettings> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub struct ProgressSettings {
    pub trace: bool,
    pub run_mode: RunMode,
}

impl ProgressSettings {
    pub fn from_context(context: &ToolContext) -> Self {
        Self {
            trace: context.trace,
            run_mode: context.run_mode,
        }
    }

    pub fn console_enabled(self) -> bool {
        self.run_mode == RunMode::Direct && std::io::stderr().is_terminal()
    }
}

pub fn init_progress_settings(context: &ToolContext) {
    let _ = PROGRESS_SETTINGS.set(ProgressSettings::from_context(context));
}

pub(crate) fn progress_settings() -> ProgressSettings {
    PROGRESS_SETTINGS
        .get()
        .copied()
        .unwrap_or(ProgressSettings {
            trace: false,
            run_mode: RunMode::Direct,
        })
}

thread_local! {
    static LAST_CONSOLE_UPDATE: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// Clears per-command progress logging TLS (used at the start of each command run).
pub fn reset_build_progress_logging() {
    LAST_CONSOLE_UPDATE.with(|last| last.set(None));
}
