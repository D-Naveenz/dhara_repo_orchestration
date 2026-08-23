use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
use anyhow::Context;
use anyhow::{Result, bail};

/// Set on child processes re-launched under the Visual Studio Developer environment.
pub const INSIDE_MSVC_ENV_VAR: &str = "DHARA_TOOL_INSIDE_MSVC";

/// Commands that do not require MSVC tooling; skip eager DevShell re-exec on Windows.
#[cfg(windows)]
const DEVSHELL_SKIP_COMMANDS: &[&[&str]] = &[
    &["config", "show"],
    &["config", "env", "init"],
    &["version", "set"],
    &["version", "bump"],
];

/// Returns true when the current process appears to run inside a VS Developer environment.
pub fn is_vs_dev_environment() -> bool {
    env::var_os(INSIDE_MSVC_ENV_VAR).is_some()
        || env::var_os("VSCMD_VER").is_some()
        || env::var_os("VCINSTALLDIR").is_some()
}

/// Locates the latest Visual Studio installation via vswhere.
pub fn locate_vs_install() -> Result<PathBuf> {
    locate_visual_studio_install()
}

/// On Windows, re-executes the current binary under Enter-VsDevShell when not already in a VS dev env.
///
/// Returns `Ok(true)` when the parent should exit (re-exec spawned). Returns `Ok(false)` when no
/// re-exec was needed. Non-Windows always returns `Ok(false)`.
pub fn reexec_under_devshell_if_needed(args: &[String]) -> Result<bool> {
    #[cfg(not(windows))]
    {
        let _ = args;
        Ok(false)
    }

    #[cfg(windows)]
    {
        if is_vs_dev_environment() {
            return Ok(false);
        }
        if should_skip_devshell_bootstrap(args) {
            return Ok(false);
        }

        let exe = env::current_exe().context("failed to resolve drot executable path")?;
        let vs_install = locate_visual_studio_install()?;
        let script = build_devshell_launch_script(&exe, args, &vs_install)?;
        let status = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .context("failed to spawn PowerShell for Visual Studio DevShell bootstrap")?;

        if !status.success() {
            bail!(
                "Visual Studio DevShell bootstrap failed with status {status}. \
                 Install Visual Studio Build Tools with C++ workload or run from a Developer PowerShell."
            );
        }
        Ok(true)
    }
}

/// Runs a shell command line inside the Visual Studio Developer environment.
pub fn run_with_msvc_env(command: &str) -> Result<()> {
    #[cfg(not(windows))]
    {
        let _ = command;
        bail!("MSVC environment is only supported on Windows");
    }

    #[cfg(windows)]
    {
        let vs_install = locate_visual_studio_install()?;
        let escaped = command.replace('\'', "''");
        let script = format!(
            r#"$vsPath = '{vs}'; Import-Module (Join-Path $vsPath 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll'); Enter-VsDevShell -VsInstallPath $vsPath -SkipAutomaticLocation; $env:{inside}='1'; cmd.exe /d /c "{escaped}""#,
            vs = vs_install.display(),
            inside = INSIDE_MSVC_ENV_VAR,
            escaped = escaped.replace('"', "\\\""),
        );

        let status = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .context("failed to spawn PowerShell for MSVC command")?;

        if !status.success() {
            bail!("MSVC command failed with status {status}: {command}");
        }
        Ok(())
    }
}

#[cfg(windows)]
fn should_skip_devshell_bootstrap(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }
    if args
        .iter()
        .any(|arg| arg == "--help" || arg == "-h" || arg == "--version" || arg == "-V")
    {
        return true;
    }
    DEVSHELL_SKIP_COMMANDS.iter().any(|prefix| {
        args.len() >= prefix.len() && prefix.iter().zip(args.iter()).all(|(e, a)| *e == a)
    })
}

#[cfg(windows)]
fn build_devshell_launch_script(exe: &Path, args: &[String], vs_install: &Path) -> Result<String> {
    let mut launch = format!("& '{}'", exe.display().to_string().replace('\'', "''"));
    for arg in args {
        let escaped = arg.replace('\'', "''");
        launch.push_str(&format!(" '{}'", escaped));
    }
    Ok(format!(
        r#"$vsPath = '{vs}'; Import-Module (Join-Path $vsPath 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll'); Enter-VsDevShell -VsInstallPath $vsPath -SkipAutomaticLocation; $env:{inside}='1'; {launch}"#,
        vs = vs_install.display(),
        inside = INSIDE_MSVC_ENV_VAR,
        launch = launch,
    ))
}

fn locate_visual_studio_install() -> Result<PathBuf> {
    #[cfg(not(windows))]
    {
        bail!("Visual Studio discovery is only supported on Windows");
    }

    #[cfg(windows)]
    {
        let program_files_x86 = env::var("ProgramFiles(x86)")
            .context("ProgramFiles(x86) is not set; MSVC tooling requires Windows")?;
        let vswhere = Path::new(&program_files_x86)
            .join("Microsoft Visual Studio")
            .join("Installer")
            .join("vswhere.exe");
        if !vswhere.is_file() {
            bail!(
                "vswhere.exe was not found at {}; install Visual Studio build tools",
                vswhere.display()
            );
        }

        let output = Command::new(&vswhere)
            .args([
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-property",
                "installationPath",
            ])
            .output()
            .with_context(|| format!("failed to run {}", vswhere.display()))?;

        if !output.status.success() {
            bail!("vswhere failed with status {}", output.status);
        }

        let install = String::from_utf8(output.stdout)
            .context("vswhere output was not valid UTF-8")?
            .trim()
            .trim_matches('"')
            .to_owned();
        if install.is_empty() {
            bail!("Visual Studio with MSVC build tools was not found");
        }

        Ok(PathBuf::from(install))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(not(windows))]
    fn run_with_msvc_env_is_windows_only() {
        let error = super::run_with_msvc_env("echo test").unwrap_err();
        assert!(error.to_string().contains("MSVC"));
    }

    #[test]
    fn inside_msvc_var_name_is_stable() {
        assert_eq!(super::INSIDE_MSVC_ENV_VAR, "DHARA_TOOL_INSIDE_MSVC");
    }
}
