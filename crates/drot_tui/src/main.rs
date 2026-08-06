use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use drot_kernel::{
    CommandRegistry, LoggingOptions, ParseMode, RootArgs, RunMode, ToolContext,
    activation::run_activation, begin_operator_session, ensure_workspace_state, parse_root_args,
    paths::resolve_exe_root, register_base_commands, register_extensions, set_linked_extension,
    stale_cached_repository, try_early_repository, workers,
};
use drot_tui::{TuiBootParams, can_launch_tui, run_tui};

fn main() -> Result<()> {
    if !can_launch_tui() {
        bail!("drot_tui requires an interactive terminal (stdin and stdout must be TTYs)");
    }

    #[cfg(feature = "extension-dhara-storage")]
    {
        drot_dhara_storage::install_hooks();
        set_linked_extension(drot_dhara_storage::EXTENSION_ID);
    }
    #[cfg(not(feature = "extension-dhara-storage"))]
    {
        set_linked_extension("none");
    }

    let cli = parse_root_args(env::args().skip(1).collect(), ParseMode::Interactive)?;

    let mut registry = CommandRegistry::new();
    register_base_commands(&mut registry);
    #[cfg(feature = "extension-dhara-storage")]
    {
        register_extensions(&mut registry, drot_dhara_storage::extension());
    }

    if cli.show_version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // TUI does not dump CLI help; point users at the in-app help once loaded.
    if cli.show_help {
        eprintln!(
            "drot_tui: open the TUI and use in-app help for commands.\n\
             Global options: -r/--repository, --package-dir, --output-dir, --logs-dir,\n\
             -m/--min, -t/--trace, -w/--workers, -y/--yes, --version.\n\
             For scripted CLI help, run: drot --help"
        );
        return Ok(());
    }

    let exe_root =
        resolve_exe_root(env::current_exe().context("failed to resolve current executable")?)?;

    let run_mode = RunMode::Interactive;
    let effective_workers = workers::init_global_thread_pool(cli.workers)?;

    let boot = TuiBootParams {
        min: cli.min,
        trace: cli.trace,
        workers: effective_workers,
        yes: cli.yes,
        package_dir: cli.package_dir.clone(),
        output_dir: cli.output_dir.clone(),
        logs_dir: cli.logs_dir.clone(),
    };

    if let Some(repo_root) = try_early_repository(&exe_root, cli.repository.clone())? {
        let context = build_context(
            repo_root.clone(),
            exe_root.clone(),
            run_mode,
            &cli,
            effective_workers,
        );
        let session = begin_operator_session(LoggingOptions::from_context(&context))
            .context("failed to initialize operator logging")?;
        let pending_activation = run_activation(&repo_root, cli.yes, run_mode)?.unwrap_or_default();
        ensure_workspace_state(&context);
        let result = run_tui(
            &registry,
            exe_root,
            boot,
            Some(context),
            pending_activation,
            None,
        );
        match result {
            Ok(()) => session.finish(0, None, None),
            Err(error) => {
                session.finish(1, None, Some(&error.to_string()));
                return Err(error);
            }
        }
    } else {
        let stale_hint = stale_cached_repository(&exe_root);
        // Logging starts when the TUI activates a repository (see run_tui / activation path).
        run_tui(&registry, exe_root, boot, None, Vec::new(), stale_hint)?;
    }

    Ok(())
}

fn build_context(
    repo_root: PathBuf,
    exe_root: PathBuf,
    run_mode: RunMode,
    cli: &RootArgs,
    workers: usize,
) -> ToolContext {
    ToolContext {
        repo_root,
        tool_root: exe_root,
        run_mode,
        min: cli.min,
        trace: cli.trace,
        workers,
        package_dir: cli.package_dir.clone(),
        output_dir: cli.output_dir.clone(),
        logs_dir: cli.logs_dir.clone(),
    }
}
