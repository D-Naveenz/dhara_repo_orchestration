//! Dhara Storage product extension for DROT.

mod build;
mod config;
mod defs;
mod package;
mod presets;

pub use presets::{apply_form_preset, default_preset_id};
mod quality;
mod release;
mod strings;
mod ui;

use std::sync::Arc;

use anyhow::Result;

use drot_kernel::{
    CommandRegistry, CommandResult, CommandSpec, Extension, SectionSpec, ToolContext,
};

pub(crate) struct RegisteredCommand {
    id: &'static str,
    path: &'static [&'static str],
    summary: &'static str,
    args_summary: &'static str,
    section: &'static str,
    handler: fn(&ToolContext, &[String]) -> Result<CommandResult>,
}

/// Storage product extension — upserts handlers onto kernel base commands and adds product commands.
pub struct DharaStorageExtension;

impl Extension for DharaStorageExtension {
    fn register(&self, registry: &mut CommandRegistry) {
        for section in self.sections() {
            registry.add_section(section);
        }

        for command in self.commands() {
            let handler = command.handler;
            registry.upsert_command(CommandSpec {
                id: command.id,
                path: command.path,
                summary: command.summary,
                args_summary: command.args_summary,
                section: command.section,
                ui: ui::ui_for_command(command.id, command.summary, command.args_summary),
                handler: Some(Arc::new(handler)),
                is_disabled: false,
                disabled_reason: None,
            });
        }
    }
}

impl DharaStorageExtension {
    fn sections(&self) -> Vec<SectionSpec> {
        vec![
            build::section(),
            config::section(),
            config::version_section(),
            defs::section(),
            quality::section(),
            package::native_section(),
            package::verify_section(),
            package::section(),
            release::section(),
        ]
    }

    fn commands(&self) -> Vec<RegisteredCommand> {
        let mut commands = Vec::new();
        commands.extend(build::commands());
        commands.extend(config::commands());
        commands.extend(defs::commands());
        commands.extend(quality::commands());
        commands.extend(package::commands());
        commands.extend(release::commands());
        commands
    }
}

pub(crate) fn command(
    id: &'static str,
    path: &'static [&'static str],
    summary: &'static str,
    args_summary: &'static str,
    section: &'static str,
    handler: fn(&ToolContext, &[String]) -> Result<CommandResult>,
) -> RegisteredCommand {
    RegisteredCommand {
        id,
        path,
        summary,
        args_summary,
        section,
        handler,
    }
}

#[cfg(test)]
mod tests {
    use drot_kernel::{CommandRegistry, Extension, register_base_commands};

    use super::DharaStorageExtension;

    #[test]
    fn registration_adds_expected_sections_and_commands() {
        let mut registry = CommandRegistry::new();
        register_base_commands(&mut registry);
        DharaStorageExtension.register(&mut registry);

        let sections = registry
            .sections()
            .map(|section| section.name)
            .collect::<Vec<_>>();
        assert_eq!(
            sections,
            vec![
                "build", "config", "defs", "native", "package", "quality", "release", "verify",
                "version"
            ]
        );

        let commands = registry
            .commands()
            .map(|command| command.id)
            .collect::<Vec<_>>();
        assert!(commands.contains(&"build.run"));
        assert!(commands.contains(&"config.show"));
        assert!(commands.contains(&"defs.inspect-trid-xml"));
        assert!(commands.contains(&"verify.package"));
        assert!(commands.contains(&"release.run"));
        assert!(
            registry
                .commands()
                .all(|command| !command.ui.description.trim().is_empty())
        );
        assert!(
            registry
                .commands()
                .find(|c| c.id == "version.bump")
                .is_some_and(|c| !c.is_effectively_disabled())
        );
    }
}
