use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use drot_kernel::{
    CommandRegistry, ParseMode, RootArgs, RunMode, ToolContext, activation::run_activation,
    ensure_workspace_state, parse_root_args, paths::resolve_exe_root, register_plugins,
    stale_cached_repository, try_early_repository, workers,
};
use drot_tui::{TuiBootParams, can_launch_tui, run_tui};

fn main() -> Result<()> {
    if !can_launch_tui() {
        bail!("drot_tui requires an interactive terminal (stdin and stdout must be TTYs)");
    }

    drot_dhara_storage::install_hooks();

    let cli = parse_root_args(env::args().skip(1).collect(), ParseMode::Interactive)?;

    let mut registry = CommandRegistry::new();
    register_plugins(&mut registry, drot_dhara_storage::plugins());

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
        let pending_activation = run_activation(&repo_root, cli.yes, run_mode)?.unwrap_or_default();
        let context = build_context(
            repo_root,
            exe_root.clone(),
            run_mode,
            &cli,
            effective_workers,
        );
        ensure_workspace_state(&context);
        run_tui(
            &registry,
            exe_root,
            boot,
            Some(context),
            pending_activation,
            None,
        )?;
    } else {
        let stale_hint = stale_cached_repository(&exe_root);
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
