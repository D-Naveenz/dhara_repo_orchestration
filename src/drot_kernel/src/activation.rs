use std::io::{self, IsTerminal, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::context::RunMode;
use crate::repo_config::{apply_config_drift, detect_config_drift, ConfigDriftItem};

/// Applies manifest drift immediately when `yes` is set.
///
/// Returns pending drift items for the interactive activation modal when confirmation is required.
pub fn run_activation(
    repo_root: &Path,
    yes: bool,
    run_mode: RunMode,
) -> Result<Option<Vec<ConfigDriftItem>>> {
    let drifts = detect_config_drift(repo_root)?;
    if drifts.is_empty() {
        return Ok(None);
    }

    if yes {
        apply_config_drift(repo_root, &drifts)?;
        return Ok(None);
    }

    match run_mode {
        RunMode::Interactive => Ok(Some(drifts)),
        RunMode::Direct => prompt_direct_activation(repo_root, &drifts),
    }
}

fn prompt_direct_activation(
    repo_root: &Path,
    drifts: &[ConfigDriftItem],
) -> Result<Option<Vec<ConfigDriftItem>>> {
    if !io::stdin().is_terminal() {
        bail!(non_interactive_drift_message(drifts));
    }

    eprintln!("Configuration drift detected (dhara.config.toml is truth):");
    for item in drifts {
        eprintln!("  - {}", item.summary);
    }
    eprint!("Apply changes? [y/N]: ");
    io::stderr()
        .flush()
        .context("failed to flush activation prompt")?;

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("failed to read activation confirmation")?;
    let answer = answer.trim().to_ascii_lowercase();
    if answer == "y" || answer == "yes" {
        apply_config_drift(repo_root, drifts)?;
        return Ok(None);
    }

    bail!("activation declined; pass --yes to apply configuration drift without prompting");
}

pub fn non_interactive_drift_message(drifts: &[ConfigDriftItem]) -> String {
    let mut message =
        String::from("configuration drift detected; pass --yes to apply without prompting:\n");
    for item in drifts {
        message.push_str("  - ");
        message.push_str(&item.summary);
        message.push('\n');
    }
    message
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::repo_config::{
        detect_config_drift, ConfigDriftItem, ConfigDriftKind, CONFIG_PATH, ROOT_CARGO_TOML_PATH,
    };

    fn write_minimal_repo(repo_root: &std::path::Path) {
        fs::write(
            repo_root.join(CONFIG_PATH),
            "[versions]\nworkspace = \"0.2.0\"\n\n[nuget]\npackage_id = \"Dhara.Storage\"\nsource = \"https://api.nuget.org/v3/index.json\"\nauthors = [\"Author\"]\ndescription = \"desc\"\ntags = [\"t\"]\nreadme = \"src/bindings/csharp/Dhara.Storage/README.md\"\nrepository_url = \"https://example.com\"\nproject_url = \"https://example.com\"\n\n[ci]\nsmoke_project = \"src/bindings/csharp/Dhara.Storage.ConsumerSmoke/Dhara.Storage.ConsumerSmoke.csproj\"\npackage_project = \"src/bindings/csharp/Dhara.Storage/Dhara.Storage.csproj\"\ntests_project = \"src/bindings/csharp/Dhara.Storage.Tests/Dhara.Storage.Tests.csproj\"\nnative_runtimes = [\"linux-x64\"]\nhost_runtime_smoke = \"linux-x64\"\naot_runtime_smoke = \"linux-x64\"\n\n[publish]\nenvironment = \"nuget-production\"\napi_key_env = \"NUGET_API_KEY\"\n\n[targets.rust_targets]\nlinux-x64 = \"x86_64-unknown-linux-gnu\"\n",
        )
        .unwrap();
        fs::create_dir_all(repo_root.join("src/bindings/csharp/Dhara.Storage")).unwrap();
        fs::write(
            repo_root.join("src/bindings/csharp/Dhara.Storage/Dhara.Storage.csproj"),
            "<Project />",
        )
        .unwrap();
        fs::write(
            repo_root.join(ROOT_CARGO_TOML_PATH),
            "[workspace]\n[workspace.package]\nversion = \"0.1.0\"\n[workspace.dependencies]\ndhara_storage_dal = { version = \"0.1.0\", path = \"src/core/dhara_storage_dal\" }\ndhara_storage = { version = \"0.1.0\", path = \"src/core/dhara_storage\" }\n",
        )
        .unwrap();
        fs::write(repo_root.join(".env.example"), "NUGET_API_KEY=\n").unwrap();
        fs::write(
            repo_root.join("src/bindings/csharp/Dhara.Storage/README.md"),
            "# pkg",
        )
        .unwrap();
        fs::create_dir_all(repo_root.join("src/core/dhara_storage_dal/resources")).unwrap();
        fs::write(
            repo_root.join("src/core/dhara_storage_dal/resources/filedefs.dat"),
            b"dat",
        )
        .unwrap();
    }

    #[test]
    fn activation_yes_applies_without_prompt() {
        let temp = tempdir().unwrap();
        write_minimal_repo(temp.path());

        let pending = run_activation(temp.path(), true, RunMode::Direct).unwrap();
        assert!(pending.is_none());
        let cargo = fs::read_to_string(temp.path().join(ROOT_CARGO_TOML_PATH)).unwrap();
        assert!(cargo.contains("version = \"0.2.0\""));
        let remaining = detect_config_drift(temp.path()).unwrap();
        assert!(!remaining
            .iter()
            .any(|item| item.kind == ConfigDriftKind::WorkspaceCargoToml));
    }

    #[test]
    fn activation_non_tty_without_yes_fails() {
        let drifts = vec![ConfigDriftItem {
            kind: ConfigDriftKind::WorkspaceCargoToml,
            summary: format!("{ROOT_CARGO_TOML_PATH} workspace version -> 0.2.0"),
        }];
        let error = prompt_direct_activation(tempdir().unwrap().path(), &drifts).unwrap_err();
        assert!(error.to_string().contains("--yes"));
    }

    #[test]
    fn non_interactive_message_lists_drifts() {
        let drifts = vec![ConfigDriftItem {
            kind: ConfigDriftKind::WorkspaceCargoToml,
            summary: "workspace cargo drift".to_owned(),
        }];
        let message = non_interactive_drift_message(&drifts);
        assert!(message.contains("workspace cargo drift"));
        assert!(message.contains("--yes"));
    }
}
