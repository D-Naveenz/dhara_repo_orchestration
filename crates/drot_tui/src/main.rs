use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use drot_cli::{CommandRegistry, DharaStorageCapability, RunMode, ToolCapability, ToolContext};
use drot_kernel::{
    activation::run_activation, ensure_workspace_state, paths::resolve_exe_root,
    resolve_and_persist_repository, stale_cached_repository, try_cached_repository, workers,
};
use drot_tui::{TuiBootParams, can_launch_tui, run_tui};

fn main() -> Result<()> {
    if !can_launch_tui() {
        bail!("drot_tui requires an interactive terminal (stdin and stdout must be TTYs)");
    }

    let cli = parse_root_args(env::args().skip(1).collect())?;

    let mut registry = CommandRegistry::new();
    DharaStorageCapability.register(&mut registry);

    if cli.show_version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if cli.show_help {
        print!("{}", help_text());
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
        let pending_activation =
            run_activation(&repo_root, cli.yes, run_mode)?.unwrap_or_default();
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

fn try_early_repository(exe_root: &Path, cli_override: Option<PathBuf>) -> Result<Option<PathBuf>> {
    if let Some(path) = cli_override {
        return Ok(Some(resolve_and_persist_repository(exe_root, path, true)?));
    }

    Ok(try_cached_repository(exe_root))
}

#[derive(Debug, Clone)]
struct RootArgs {
    repository: Option<PathBuf>,
    min: bool,
    trace: bool,
    workers: Option<usize>,
    package_dir: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    logs_dir: Option<PathBuf>,
    show_help: bool,
    show_version: bool,
    yes: bool,
}

fn parse_root_args(args: Vec<String>) -> Result<RootArgs> {
    let mut parsed = RootArgs {
        repository: None,
        min: false,
        trace: false,
        workers: None,
        package_dir: None,
        output_dir: None,
        logs_dir: None,
        show_help: false,
        show_version: false,
        yes: false,
    };

    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        match token.as_str() {
            "-h" | "--help" => {
                parsed.show_help = true;
                index += 1;
            }
            "--version" => {
                parsed.show_version = true;
                index += 1;
            }
            "-m" | "--min" => {
                parsed.min = true;
                index += 1;
            }
            "-t" | "--trace" => {
                parsed.trace = true;
                index += 1;
            }
            "-y" | "--yes" => {
                parsed.yes = true;
                index += 1;
            }
            "-r" | "--repository" => {
                parsed.repository = Some(PathBuf::from(next_value(&args, index, "--repository")?));
                index += 2;
            }
            "-w" | "--workers" => {
                let value = next_value(&args, index, "--workers")?;
                parsed.workers = Some(
                    value
                        .parse()
                        .with_context(|| format!("'{value}' is not a valid worker count"))?,
                );
                index += 2;
            }
            "--package-dir" => {
                parsed.package_dir =
                    Some(PathBuf::from(next_value(&args, index, "--package-dir")?));
                index += 2;
            }
            "--output-dir" => {
                parsed.output_dir = Some(PathBuf::from(next_value(&args, index, "--output-dir")?));
                index += 2;
            }
            "--logs-dir" => {
                parsed.logs_dir = Some(PathBuf::from(next_value(&args, index, "--logs-dir")?));
                index += 2;
            }
            _ if token.starts_with("--repository=") => {
                parsed.repository = Some(PathBuf::from(token.trim_start_matches("--repository=")));
                index += 1;
            }
            _ if token.starts_with("-r=") => {
                parsed.repository = Some(PathBuf::from(token.trim_start_matches("-r=")));
                index += 1;
            }
            _ if token.starts_with("--package-dir=") => {
                parsed.package_dir =
                    Some(PathBuf::from(token.trim_start_matches("--package-dir=")));
                index += 1;
            }
            _ if token.starts_with("--output-dir=") => {
                parsed.output_dir = Some(PathBuf::from(token.trim_start_matches("--output-dir=")));
                index += 1;
            }
            _ if token.starts_with("--logs-dir=") => {
                parsed.logs_dir = Some(PathBuf::from(token.trim_start_matches("--logs-dir=")));
                index += 1;
            }
            _ if token.starts_with("--workers=") => {
                let value = token.trim_start_matches("--workers=");
                parsed.workers = Some(
                    value
                        .parse()
                        .with_context(|| format!("'{value}' is not a valid worker count"))?,
                );
                index += 1;
            }
            other => {
                bail!("unexpected argument '{other}' (drot_tui does not take subcommands; use drot for CLI)");
            }
        }
    }

    Ok(parsed)
}

fn next_value<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str> {
    args.get(index + 1)
        .map(String::as_str)
        .with_context(|| format!("{option} requires a value"))
}

fn help_text() -> String {
    "Usage: drot_tui [global-options]\n\n\
     Interactive TUI for Dhara Repository Orchestration. For scripts/CI, use drot.\n\n\
     Global options:\n\
       -r, --repository <path>  repository directory or dhara.config.toml\n\
       --package-dir <path>\n\
       --output-dir <path>\n\
       --logs-dir <path>\n\
       -m, --min         file log WARN only\n\
       -t, --trace       file log DEBUG\n\
       -w, --workers <n>  cap Rayon worker threads\n\
       -y, --yes         apply configuration drift without prompting\n\
       -h, --help\n\
       --version\n"
        .to_owned()
}
