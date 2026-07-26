//! Dhara Storage product plugin for DROT.

pub mod commands;
pub mod filedefs;
pub mod ops;
pub mod registry;

use std::path::Path;

use drot_kernel::ToolCapability;
use drot_kernel::product::{ProductHooks, set_product_hooks};
use drot_kernel::workspace::{DefsPackageStatus, WorkspaceSnapshot};

pub use registry::DharaStorageCapability;

/// Product plugins contributed by this crate.
pub fn plugins() -> Vec<Box<dyn ToolCapability>> {
    vec![Box::new(DharaStorageCapability)]
}

/// Dhara Storage [`ProductHooks`] implementation.
pub struct StorageProductHooks;

static STORAGE_HOOKS: StorageProductHooks = StorageProductHooks;

impl ProductHooks for StorageProductHooks {
    fn cargo_workspace_deps(&self) -> &'static [&'static str] {
        &["dhara_storage_core", "dhara_storage"]
    }

    fn embedded_defs_relative(&self) -> &'static str {
        "src/core/dhara_storage/resources/filedefs.dat"
    }

    fn embedded_defs_dir_relative(&self) -> &'static str {
        "src/core/dhara_storage/resources"
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
}

/// Registers [`StorageProductHooks`] with the kernel.
pub fn install_hooks() {
    set_product_hooks(&STORAGE_HOOKS);
}
