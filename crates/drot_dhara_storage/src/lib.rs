//! Dhara Storage product extension for DROT.

pub mod commands;
pub mod filedefs;
pub mod ops;
pub mod registry;

use drot_kernel::command::CommandSpec;
use drot_kernel::forms::CommandForm;

use std::path::Path;

use drot_kernel::Extension;
use drot_kernel::product::{ProductHooks, set_product_hooks};
use drot_kernel::workspace::{DefsPackageStatus, WorkspaceSnapshot};

pub use registry::DharaStorageExtension;

/// Stable id logged in session bookends for this extension.
pub const EXTENSION_ID: &str = "dhara_storage";

/// Product extensions contributed by this crate (exactly one for hosts that enable it).
pub fn extension() -> Vec<Box<dyn Extension>> {
    vec![Box::new(DharaStorageExtension)]
}

/// Dhara Storage [`ProductHooks`] implementation.
pub struct StorageProductHooks;

static STORAGE_HOOKS: StorageProductHooks = StorageProductHooks;

impl ProductHooks for StorageProductHooks {
    fn cargo_workspace_deps(&self) -> &'static [&'static str] {
        &["dhara_storage_core", "dhara_storage"]
    }

    fn embedded_defs_relative(&self) -> &'static str {
        "core/dhara_storage/resources/filedefs.dat"
    }

    fn embedded_defs_dir_relative(&self) -> &'static str {
        "core/dhara_storage/resources"
    }

    fn analyze_defs_package(&self, defs_path: &Path) -> WorkspaceSnapshot {
        if !defs_path.is_file() {
            return WorkspaceSnapshot {
                defs_path: defs_path.to_path_buf(),
                defs_status: DefsPackageStatus::Missing,
                package_revision: None,
                definitions_release: None,
                package_version: None,
                definition_count: None,
            };
        }

        match filedefs::load_package(defs_path) {
            Ok(loaded) => WorkspaceSnapshot {
                defs_path: defs_path.to_path_buf(),
                defs_status: DefsPackageStatus::Present,
                package_revision: Some(loaded.package.package_revision),
                definitions_release: Some(loaded.package.definitions_release.clone()),
                package_version: Some(loaded.package.package_version.clone()),
                definition_count: Some(loaded.package.definitions.len()),
            },
            Err(error) => WorkspaceSnapshot {
                defs_path: defs_path.to_path_buf(),
                defs_status: DefsPackageStatus::Invalid {
                    reason: error.to_string(),
                },
                package_revision: None,
                definitions_release: None,
                package_version: None,
                definition_count: None,
            },
        }
    }

    fn initialize_tui_form(&self, form: &mut CommandForm, command: &CommandSpec) {
        initialize_tui_form(form, command);
    }

    fn apply_tui_preset(&self, form: &mut CommandForm, command: &CommandSpec, preset_id: &str) {
        registry::apply_form_preset(form, command, preset_id);
    }
}

/// Registers [`StorageProductHooks`] with the kernel.
pub fn install_hooks() {
    set_product_hooks(&STORAGE_HOOKS);
}

/// Applies the default TUI preset for commands that define workflow presets.
pub fn initialize_tui_form(form: &mut CommandForm, command: &CommandSpec) {
    if let Some(preset_id) = registry::default_preset_id(command.id) {
        registry::apply_form_preset(form, command, preset_id);
    }
}
