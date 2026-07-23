mod audit;
mod operation;
mod progress;

pub use audit::*;
pub use operation::{ActivityLabel, CommandOutcome, CommandRun, ELAPSED_UI_THRESHOLD, command_labels};
pub use progress::{
    ProgressSettings, init_progress_settings, reset_build_progress_logging,
};

pub fn interactive_mode_enabled() -> bool {
    progress::progress_settings().run_mode == crate::context::RunMode::Interactive
}
