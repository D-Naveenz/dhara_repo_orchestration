//! Base command catalog owned by the kernel.
//!
//! Extensions upsert handlers and product-specific metadata onto these stubs.

use crate::command::{CommandRegistry, CommandSpec, CommandUi, SectionSpec};

/// Registers generic DROT command stubs (no handlers until an extension implements them).
pub fn register_base_commands(registry: &mut CommandRegistry) {
    registry.add_section(SectionSpec {
        name: "config",
        prompt: "drot:config> ",
        summary: "Repository configuration commands",
    });
    registry.add_section(SectionSpec {
        name: "version",
        prompt: "drot:version> ",
        summary: "Versioning commands",
    });

    registry.upsert_command(stub(
        "config.show",
        &["config", "show"],
        "config",
        "Show effective repo configuration",
        "",
        "Show the effective repository configuration.",
    ));
    registry.upsert_command(stub(
        "config.env.init",
        &["config", "env", "init"],
        "config",
        "Create .env.local from .env.example",
        "",
        "Create .env.local from .env.example when missing.",
    ));
    registry.upsert_command(stub(
        "version.set",
        &["version", "set"],
        "version",
        "Set the shared workspace version",
        "<version>",
        "Set the shared workspace version across manifests.",
    ));
    registry.upsert_command(stub(
        "version.bump",
        &["version", "bump"],
        "version",
        "Bump the shared workspace version",
        "--part <major|minor|patch>",
        "Bump the shared workspace version by major, minor, or patch.",
    ));
}

fn stub(
    id: &'static str,
    path: &'static [&'static str],
    section: &'static str,
    summary: &'static str,
    args_summary: &'static str,
    description: &'static str,
) -> CommandSpec {
    CommandSpec {
        id,
        path,
        summary,
        args_summary,
        section,
        ui: CommandUi::empty(description),
        handler: None,
        is_disabled: false,
        disabled_reason: None,
    }
}
