use crate::commands::build_run_command;
use drot_kernel::SectionSpec;

use super::{RegisteredCommand, command};

pub fn section() -> SectionSpec {
    SectionSpec {
        name: "build",
        prompt: "dhara:build> ",
        summary: "End-to-end repository build workflows",
    }
}

pub fn commands() -> Vec<RegisteredCommand> {
    vec![command(
        "build.run",
        &["build", "run"],
        "Run the full local repository build workflow",
        "[--skip-config] [--skip-defs] [--skip-quality] [--skip-docs] [--skip-dotnet] [--skip-native] [--skip-verify] [--configuration <name>]",
        "build",
        build_run_command,
    )]
}
