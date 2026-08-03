use std::env;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use drot_kernel::{
    CommandRegistry, ParseMode, RootArgs, RunMode, ToolContext, activation::run_activation,
    ensure_workspace_state, log_session_end, parse_root_args, paths::resolve_exe_root,
    register_plugins, resolve_and_persist_repository, try_early_repository, workers,
};

pub fn run() -> Result<()> {
    drot_dhara_storage::install_hooks();

    let cli = parse_root_args(env::args().skip(1).collect(), ParseMode::Direct)?;

    let mut registry = CommandRegistry::new();
    register_plugins(&mut registry, drot_dhara_storage::plugins());

    if cli.show_version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if cli.show_help || cli.command.is_empty() {
        print!("{}", help_text(&registry));
        return Ok(());
    }

    let exe_root =
        resolve_exe_root(env::current_exe().context("failed to resolve current executable")?)?;

    let run_mode = RunMode::Direct;
    let effective_workers = workers::init_global_thread_pool(cli.workers)?;

    let repo_root = resolve_repository_for_direct(&exe_root, cli.repository.clone())?;
    let _pending = run_activation(&repo_root, cli.yes, run_mode)?.unwrap_or_default();
    let context = build_context(
        repo_root,
        exe_root.clone(),
        run_mode,
        &cli,
        effective_workers,
    );
    ensure_workspace_state(&context);

    let command_id = registry
        .resolve(&cli.command)
        .map(|(command, _)| command.id)
        .unwrap_or("unknown");
    let result = match registry.execute(&context, &cli.command) {
        Ok(result) => {
            log_session_end(result.exit_code, Some(command_id), None);
            result
        }
        Err(error) => {
            log_session_end(1, Some(command_id), Some(&error.to_string()));
            return Err(error);
        }
    };
    result.print(&context);
    if result.exit_code != 0 {
        std::process::exit(result.exit_code);
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

fn resolve_repository_for_direct(
    exe_root: &std::path::Path,
    cli_override: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(repo) = try_early_repository(exe_root, cli_override)? {
        return Ok(repo);
    }

    if io::stdin().is_terminal() {
        let path = prompt_repository_path()?;
        return resolve_and_persist_repository(exe_root, path, true);
    }

    bail!(
        "repository path is required; pass -r/--repository <path> or run drot_tui to create {}/runtime.toml",
        exe_root.display()
    );
}

fn prompt_repository_path() -> Result<PathBuf> {
    let mut stderr = io::stderr();
    write!(stderr, "Repository path (folder or dhara.config.toml): ")?;
    stderr.flush()?;

    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .context("failed to read repository path from stdin")?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        bail!("repository path is required");
    }
    Ok(PathBuf::from(trimmed))
}

fn help_text(registry: &CommandRegistry) -> String {
    format!(
        "Usage: drot [global-options] <command> [command-options]\n\n\
         Direct CLI for CI, agents, and scripts. For the interactive TUI, run drot_tui.\n\n\
         Global options (may appear before or after the command):\n\
           -r, --repository <path>  repository directory or dhara.config.toml (overrides runtime cache)\n\
           --package-dir <path>\n\
           --output-dir <path>\n\
           --logs-dir <path>\n\
           -m, --min         file log WARN only (console stays INFO)\n\
           -t, --trace       file log DEBUG (console stays INFO)\n\
           -w, --workers <n>  cap Rayon worker threads (default 4; env TOOL_MAX_WORKERS)\n\
           -y, --yes         apply configuration drift without prompting\n\
           -h, --help\n\
           --version\n\n\
         Repository resolution:\n\
           1. -r/--repository when provided\n\
           2. exe_path/runtime.toml when valid\n\
           3. interactive prompt on TTY\n\n\
         {}",
        registry.help_text()
    )
}
