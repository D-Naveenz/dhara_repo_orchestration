use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::runtime_cache::{resolve_and_persist_repository, try_cached_repository};

/// Shared root CLI flags for `drot` (Direct CLI and interactive TUI).
#[derive(Debug, Clone)]
pub struct RootArgs {
    pub repository: Option<PathBuf>,
    pub min: bool,
    pub trace: bool,
    pub workers: Option<usize>,
    pub package_dir: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub logs_dir: Option<PathBuf>,
    pub show_help: bool,
    pub show_version: bool,
    pub yes: bool,
    /// Subcommand tokens; empty for TUI / interactive mode.
    pub command: Vec<String>,
}

/// How unknown tokens after global options are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    /// Collect unknown tokens into [`RootArgs::command`].
    Direct,
    /// Reject unknown tokens (TUI has no subcommands).
    Interactive,
}

/// Resolve repository from `-r` / `--repository` or a valid runtime cache entry.
pub fn try_early_repository(
    exe_root: &Path,
    cli_override: Option<PathBuf>,
) -> Result<Option<PathBuf>> {
    if let Some(path) = cli_override {
        return Ok(Some(resolve_and_persist_repository(exe_root, path, true)?));
    }

    Ok(try_cached_repository(exe_root))
}

pub fn parse_root_args(args: Vec<String>, mode: ParseMode) -> Result<RootArgs> {
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
        command: Vec::new(),
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
            other => match mode {
                ParseMode::Direct => {
                    parsed.command.push(token.clone());
                    index += 1;
                }
                ParseMode::Interactive => {
                    bail!(
                        "unexpected argument '{other}' (interactive mode does not take subcommands; omit the subcommand for the TUI or pass a command for Direct CLI)"
                    );
                }
            },
        }
    }

    Ok(parsed)
}

fn next_value<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str> {
    args.get(index + 1)
        .map(String::as_str)
        .with_context(|| format!("{option} requires a value"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::repo_config::CONFIG_PATH;
    use crate::runtime_cache::{resolve_and_persist_repository, try_cached_repository};

    use super::{ParseMode, parse_root_args, try_early_repository};

    #[test]
    fn repository_flag_parsed() {
        let parsed = parse_root_args(
            vec![
                "-r".to_owned(),
                "/repo".to_owned(),
                "config".to_owned(),
                "show".to_owned(),
            ],
            ParseMode::Direct,
        )
        .unwrap();
        assert_eq!(parsed.repository, Some("/repo".into()));
    }

    #[test]
    fn repository_long_flag_parsed() {
        let parsed = parse_root_args(
            vec![
                "--repository=/repo".to_owned(),
                "config".to_owned(),
                "show".to_owned(),
            ],
            ParseMode::Direct,
        )
        .unwrap();
        assert_eq!(parsed.repository, Some("/repo".into()));
    }

    #[test]
    fn explicit_repository_wins_and_persists_cache() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("repo");
        let exe = temp.path().join("bin");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&exe).unwrap();
        fs::write(root.join(CONFIG_PATH), "[versions]\n").unwrap();

        let resolved = try_early_repository(&exe, Some(root.clone()))
            .unwrap()
            .unwrap();
        assert!(try_cached_repository(&exe).is_some());
        assert_eq!(
            resolved,
            resolve_and_persist_repository(&exe, root, false).unwrap()
        );
    }

    #[test]
    fn trace_flag_may_follow_subcommand() {
        let parsed = parse_root_args(
            vec![
                "defs".to_owned(),
                "inspect-trid-xml".to_owned(),
                "--trace".to_owned(),
            ],
            ParseMode::Direct,
        )
        .unwrap();

        assert!(parsed.trace);
        assert_eq!(parsed.command, vec!["defs", "inspect-trid-xml"]);
    }

    #[test]
    fn min_flag_parsed() {
        let parsed = parse_root_args(
            vec!["defs".to_owned(), "inspect".to_owned(), "--min".to_owned()],
            ParseMode::Direct,
        )
        .unwrap();
        assert!(parsed.min);
    }

    #[test]
    fn min_short_flag_parsed() {
        let parsed = parse_root_args(
            vec!["-m".to_owned(), "defs".to_owned(), "inspect".to_owned()],
            ParseMode::Direct,
        )
        .unwrap();
        assert!(parsed.min);
    }

    #[test]
    fn trace_short_flag_parsed() {
        let parsed = parse_root_args(
            vec![
                "-t".to_owned(),
                "defs".to_owned(),
                "inspect-trid-xml".to_owned(),
            ],
            ParseMode::Direct,
        )
        .unwrap();
        assert!(parsed.trace);
    }

    #[test]
    fn workers_flag_parsed() {
        let parsed = parse_root_args(
            vec![
                "-w".to_owned(),
                "2".to_owned(),
                "defs".to_owned(),
                "inspect".to_owned(),
            ],
            ParseMode::Direct,
        )
        .unwrap();
        assert_eq!(parsed.workers, Some(2));
    }

    #[test]
    fn yes_flag_parsed() {
        let parsed = parse_root_args(
            vec!["--yes".to_owned(), "config".to_owned(), "show".to_owned()],
            ParseMode::Direct,
        )
        .unwrap();
        assert!(parsed.yes);
    }

    #[test]
    fn interactive_rejects_subcommand_tokens() {
        let err = parse_root_args(
            vec!["config".to_owned(), "show".to_owned()],
            ParseMode::Interactive,
        )
        .unwrap_err();
        assert!(err.to_string().contains("unexpected argument"));
    }
}
