use std::path::Path;
use std::sync::OnceLock;

use crate::workspace::WorkspaceSnapshot;

pub trait ProductHooks: Send + Sync {
    /// Cargo workspace dependency names to sync on version bump (e.g. dhara_storage_core, dhara_storage).
    fn cargo_workspace_deps(&self) -> &'static [&'static str];
    /// Relative path from repo root to embedded defs package file.
    fn embedded_defs_relative(&self) -> &'static str;
    /// Relative path from repo root to embedded defs directory.
    fn embedded_defs_dir_relative(&self) -> &'static str;
    /// Analyze a defs package on disk into a workspace snapshot.
    fn analyze_defs_package(&self, defs_path: &Path) -> WorkspaceSnapshot;
}

static HOOKS: OnceLock<&'static dyn ProductHooks> = OnceLock::new();

pub fn set_product_hooks(hooks: &'static dyn ProductHooks) {
    let _ = HOOKS.set(hooks);
}

pub fn product_hooks() -> Option<&'static dyn ProductHooks> {
    HOOKS.get().copied()
}

pub fn require_product_hooks() -> &'static dyn ProductHooks {
    product_hooks().expect("product hooks not registered; call set_product_hooks before activation")
}
