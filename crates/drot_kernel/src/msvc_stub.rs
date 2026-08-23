//! Non-Windows stubs for the MSVC / VS DevShell API surface.

use std::path::PathBuf;

use anyhow::{Result, bail};

/// Set on child processes re-launched under the Visual Studio Developer environment.
pub const INSIDE_MSVC_ENV_VAR: &str = "DHARA_TOOL_INSIDE_MSVC";

/// Always false outside Windows.
pub fn is_vs_dev_environment() -> bool {
    false
}

/// Visual Studio discovery is Windows-only.
pub fn locate_vs_install() -> Result<PathBuf> {
    bail!("Visual Studio discovery is only supported on Windows")
}

/// No DevShell re-exec outside Windows.
pub fn reexec_under_devshell_if_needed(args: &[String]) -> Result<bool> {
    let _ = args;
    Ok(false)
}

/// MSVC command hosting is Windows-only.
pub fn run_with_msvc_env(command: &str) -> Result<()> {
    let _ = command;
    bail!("MSVC environment is only supported on Windows")
}

#[cfg(test)]
mod tests {
    #[test]
    fn run_with_msvc_env_is_windows_only() {
        let error = super::run_with_msvc_env("echo test").unwrap_err();
        assert!(error.to_string().contains("MSVC"));
    }

    #[test]
    fn inside_msvc_var_name_is_stable() {
        assert_eq!(super::INSIDE_MSVC_ENV_VAR, "DHARA_TOOL_INSIDE_MSVC");
    }
}
