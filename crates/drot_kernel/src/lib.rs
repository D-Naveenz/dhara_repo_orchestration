pub mod activation;
pub mod args;
pub mod base_commands;
pub mod bootstrap;
pub mod command;
pub mod context;
pub mod forms;
pub mod interactive;
pub mod logging;
pub mod msvc;
pub mod operation_progress;
pub mod output;
pub mod paths;
pub mod product;
pub mod repo_config;
pub mod runner;
pub mod runtime_cache;
pub mod subprocess;
pub mod workers;
pub mod workspace;

pub use args::{ParseMode, RootArgs, parse_root_args, try_early_repository};
pub use base_commands::register_base_commands;
pub use bootstrap::register_extensions;
pub use command::{
    ArgBinding, CommandHandler, CommandRegistry, CommandSpec, CommandUi, Extension, FieldKind,
    FieldSpec, PresetOption, SectionSpec,
};
pub use context::{CommandResult, ReportField, RunMode, StructuredReport, ToolContext};
pub use forms::{CommandForm, FormValue, preset_id, preset_label, preset_options, select_options};
pub use interactive::{
    ActivationPrompt, AppState, DiagnosticLine, DiagnosticSeverity, FAVORITES_GROUP, MainTab,
    NavTree, QUICK_ACTIONS, StatusTone, TreeNode, TreeViewState, VisibleTreeRow,
};
pub use logging::{
    ActivityLabel, CommandOutcome, CommandRun, ELAPSED_UI_THRESHOLD, LoggingOptions,
    LoggingRuntime, SessionGuard, begin_operator_session, command_labels, current_log_path,
    ensure_logging, format_command_args, init_logging, linked_extension, log_activation_debug,
    log_activation_info, log_file_path, log_module_step_debug, log_module_step_error,
    log_module_step_warn, log_session_begin, log_session_end, set_linked_extension,
    summarize_command_result, write_session_record,
};
pub use operation_progress::{
    OperationProgressGuard, ProgressSession, ProgressSnapshot, ProgressStep, RunPhase,
    adjust_step_weight, begin_analyzing, begin_single_shot, clear_run_activity, clear_run_clock,
    commit_plan, complete_progress, has_committed_progress_plan, install_run_clock, plan_step,
    register_interactive_progress_sender, set_command_activity, set_command_milestone,
    set_step_message, set_step_total, tick_step, unregister_interactive_progress_sender,
};
pub use output::{
    OutputCaptureGuard, OutputEvent, OutputStream, cancel_active_subprocess, emit_stderr_line,
    emit_stdout_line, emit_warn_line,
};
pub use paths::{is_repo_root, normalize_repository_input, resolve_exe_root};
pub use product::{ProductHooks, product_hooks, require_product_hooks, set_product_hooks};
pub use repo_config::{
    CARGO_REGISTRY_TOKEN_ENV, CONFIG_PATH, CiConfig, ConfigDriftItem, ConfigDriftKind,
    DEFAULT_ENV_EXAMPLE_CONTENT, DharaRepoConfig, ENV_EXAMPLE_PATH, ENV_LOCAL_PATH,
    NUGET_API_KEY_ENV, NuGetConfig, ProductConfig, ROOT_CARGO_TOML_PATH, ShowOutput, TargetsConfig,
    VersionConfig, VersionPart, apply_config_drift, bump_version, detect_config_drift,
    ensure_repo_scaffolding, init_env, load_config, load_env, package_projects, parse_env_content,
    read_csproj_package_id, set_version, show, sync_cargo_toml, sync_csproj, validate_config,
    verify_release,
};
pub use runner::{RunCompletion, RunHandle, cancel_run, start_run};
pub use runtime_cache::{
    RuntimeCache, load_runtime_cache, resolve_and_persist_repository, runtime_cache_path,
    save_runtime_cache, stale_cached_repository, try_cached_repository,
};
pub use subprocess::{
    inspect_package_entries, run_command, run_command_expect_failure, run_command_with_env,
    run_command_with_env_redacted, write_nuget_config,
};
pub use workspace::{
    DefsPackageStatus, PackageMeta, WorkspaceSnapshot, ensure_workspace_state,
    next_package_revision, next_package_revision_for_build, record_package_written,
    refresh_workspace_state, workspace_snapshot,
};
