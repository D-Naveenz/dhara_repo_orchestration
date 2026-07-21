mod audit;
mod operation;
mod progress;

pub use audit::*;
pub use operation::{
    ActivityLabel, CommandOutcome, CommandRun, ELAPSED_UI_THRESHOLD, command_labels,
    phase_activity_label,
};
pub use progress::{
    dispatch_trid_progress, emit_trid_progress, init_progress_settings,
    reset_build_progress_logging,
};

pub fn interactive_mode_enabled() -> bool {
    progress::progress_settings().run_mode == crate::context::RunMode::Interactive
}
