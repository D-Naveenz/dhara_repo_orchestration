use std::path::Path;

use anyhow::Result;

use drot_kernel::CommandResult;
use drot_kernel::repo_config::DharaRepoConfig;

use crate::ops::nuget::PackageOptions;

pub fn verify_package(
    repo_root: &Path,
    tool_root: &Path,
    config: &DharaRepoConfig,
    options: &PackageOptions,
) -> Result<CommandResult> {
    crate::ops::nuget::verify(repo_root, tool_root, config, options)
}
