use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::{Result, bail};
use tracing::warn;

use crate::logging::{CommandOutcome, CommandRun, LoggingOptions, ensure_logging};
use crate::output::emit_warn_line;

pub use crate::context::{CommandResult, ReportField, RunMode, StructuredReport, ToolContext};

pub type CommandHandler =
    Arc<dyn Fn(&ToolContext, &[String]) -> anyhow::Result<CommandResult> + Send + Sync + 'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionSpec {
    pub name: &'static str,
    pub prompt: &'static str,
    pub summary: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgBinding {
    Positional,
    FlagValue(&'static str),
    Switch(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresetOption {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Path,
    BrowsablePath { dialog_title: &'static str },
    Boolean,
    /// Legacy select; rendered as [`FieldKind::Combo`] in the TUI.
    Select(&'static [&'static str]),
    /// BIOS-style cycle control in the TUI.
    Combo(&'static [&'static str]),
    /// Mutually exclusive options displayed as `[*]` / `[ ]`.
    Radio(&'static [&'static str]),
    /// TUI-only preset picker; values are not serialized to CLI.
    Preset(&'static [PresetOption]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub kind: FieldKind,
    pub binding: ArgBinding,
    pub required: bool,
    pub default_value: Option<&'static str>,
    /// Optional TUI-only default; falls back to [`Self::default_value`].
    pub tui_default_value: Option<&'static str>,
    /// Visual grouping key for bordered button groups in the Options tab.
    pub group: Option<&'static str>,
    /// When true, a checked boolean means *include* the step (omit `--skip-*` when true).
    pub invert_switch: bool,
    /// When true, field is omitted from [`crate::forms::CommandForm::build_args`].
    pub tui_only: bool,
}

impl FieldSpec {
    pub const fn boolean(
        key: &'static str,
        label: &'static str,
        help: &'static str,
        flag: &'static str,
        default_value: Option<&'static str>,
    ) -> Self {
        Self {
            key,
            label,
            help,
            kind: FieldKind::Boolean,
            binding: ArgBinding::Switch(flag),
            required: false,
            default_value,
            tui_default_value: None,
            group: None,
            invert_switch: false,
            tui_only: false,
        }
    }

    pub const fn combo(
        key: &'static str,
        label: &'static str,
        help: &'static str,
        flag: &'static str,
        options: &'static [&'static str],
        default_value: Option<&'static str>,
    ) -> Self {
        Self {
            key,
            label,
            help,
            kind: FieldKind::Combo(options),
            binding: ArgBinding::FlagValue(flag),
            required: false,
            default_value,
            tui_default_value: None,
            group: None,
            invert_switch: false,
            tui_only: false,
        }
    }

    pub fn effective_default(&self, tui: bool) -> Option<&'static str> {
        if tui {
            self.tui_default_value.or(self.default_value)
        } else {
            self.default_value
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandUi {
    pub description: &'static str,
    pub fields: Vec<FieldSpec>,
    pub quick_run: bool,
    pub supports_cancel: bool,
}

/// Unified command specification shared by CLI help and the TUI.
#[derive(Clone)]
pub struct CommandSpec {
    pub id: &'static str,
    pub path: &'static [&'static str],
    pub summary: &'static str,
    pub args_summary: &'static str,
    pub section: &'static str,
    pub ui: CommandUi,
    /// Instruction set; [`None`] means the command has no action yet.
    pub handler: Option<CommandHandler>,
    /// Explicit disable even when a handler is present.
    pub is_disabled: bool,
    /// Operator-facing reason when disabled; preferred over default messages.
    pub disabled_reason: Option<&'static str>,
}

/// Compile-time product extension that registers or upserts commands into the registry.
pub trait Extension {
    fn register(&self, registry: &mut CommandRegistry);
}

#[derive(Clone, Default)]
pub struct CommandRegistry {
    sections: BTreeMap<&'static str, SectionSpec>,
    commands: Vec<CommandSpec>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_section(&mut self, section: SectionSpec) {
        self.sections.insert(section.name, section);
    }

    /// Appends a command. Prefer [`Self::upsert_command`] when an extension may replace a base stub.
    pub fn add_command(&mut self, command: CommandSpec) {
        self.upsert_command(command);
    }

    /// Inserts or replaces a command by [`CommandSpec::id`] (later registration wins).
    pub fn upsert_command(&mut self, command: CommandSpec) {
        if let Some(existing) = self.commands.iter_mut().find(|c| c.id == command.id) {
            *existing = command;
        } else {
            self.commands.push(command);
        }
    }

    pub fn sections(&self) -> impl Iterator<Item = &SectionSpec> {
        self.sections.values()
    }

    pub fn commands(&self) -> impl Iterator<Item = &CommandSpec> {
        self.commands.iter()
    }

    pub fn commands_for_section<'a>(
        &'a self,
        section: &'static str,
    ) -> impl Iterator<Item = &'a CommandSpec> + 'a {
        self.commands
            .iter()
            .filter(move |command| command.section == section)
    }

    pub fn resolve<'a>(&'a self, args: &'a [String]) -> Option<(&'a CommandSpec, &'a [String])> {
        self.commands
            .iter()
            .filter(|command| args.len() >= command.path.len())
            .filter(|command| {
                command
                    .path
                    .iter()
                    .zip(args.iter())
                    .all(|(expected, actual)| expected == actual)
            })
            .max_by_key(|command| command.path.len())
            .map(|command| (command, &args[command.path.len()..]))
    }

    pub fn execute(&self, context: &ToolContext, args: &[String]) -> Result<CommandResult> {
        let Some((command, rest)) = self.resolve(args) else {
            bail!("unknown command path: {}", args.join(" "));
        };

        ensure_logging(LoggingOptions::from_context(context))?;

        if command.is_effectively_disabled() {
            let reason = command.disable_message();
            let warning = format!("command {} is disabled — {reason}", command.id);
            warn!(target: "drot::audit", "{warning}");
            emit_warn_line(&warning);
            return Ok(CommandResult {
                exit_code: 1,
                report: None,
                message: Some(warning),
            });
        }

        let handler = command
            .handler
            .as_ref()
            .expect("enabled command must have a handler");

        let run = CommandRun::begin(command.id);
        let result = handler(context, rest);
        run.complete(CommandOutcome::from_execute(command.id, &result));

        result
    }

    pub fn help_text(&self) -> String {
        let mut output = String::from("Dhara tool commands:\n");
        for section in self.sections.values() {
            output.push_str(&format!("\n{}:\n", section.name));
            for command in self.commands_for_section(section.name) {
                let path = command.path.join(" ");
                let mut summary = command.summary.to_owned();
                if command.is_effectively_disabled() {
                    summary.push_str(" (disabled)");
                    let reason = command.disable_message();
                    summary.push_str(" — ");
                    summary.push_str(reason);
                }
                if command.args_summary.is_empty() {
                    output.push_str(&format!("  {path:<28} {summary}\n"));
                } else {
                    output.push_str(&format!(
                        "  {:<28} {summary}\n",
                        format!("{path} {}", command.args_summary),
                    ));
                }
            }
        }
        output
    }
}

impl CommandSpec {
    pub fn path_string(&self) -> String {
        self.path.join(" ")
    }

    /// True when manually disabled or when no instruction set is registered.
    pub fn is_effectively_disabled(&self) -> bool {
        self.is_disabled || self.handler.is_none()
    }

    /// Human-readable disable reason for help, Info, and execute warnings.
    pub fn disable_message(&self) -> &'static str {
        if let Some(reason) = self.disabled_reason {
            return reason;
        }
        if self.handler.is_none() {
            return "not implemented by this extension";
        }
        "disabled by the developers"
    }
}

impl CommandUi {
    pub fn empty(description: &'static str) -> Self {
        Self {
            description,
            fields: Vec::new(),
            quick_run: false,
            supports_cancel: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use crate::{ReportField, RunMode, StructuredReport, ToolContext};

    use super::*;

    fn noop(_: &ToolContext, _: &[String]) -> Result<CommandResult> {
        Ok(CommandResult::success())
    }

    fn report_handler(_: &ToolContext, args: &[String]) -> Result<CommandResult> {
        Ok(CommandResult::with_report(StructuredReport {
            title: "dispatch".to_owned(),
            fields: vec![ReportField {
                label: "args".to_owned(),
                value: args.join(" "),
            }],
        }))
    }

    fn context() -> ToolContext {
        ToolContext {
            repo_root: PathBuf::from("."),
            tool_root: PathBuf::from("."),
            run_mode: RunMode::Direct,
            min: false,
            trace: false,
            workers: 4,
            package_dir: None,
            output_dir: None,
            logs_dir: None,
        }
    }

    fn spec(
        id: &'static str,
        path: &'static [&'static str],
        section: &'static str,
        summary: &'static str,
        handler: Option<CommandHandler>,
    ) -> CommandSpec {
        CommandSpec {
            id,
            path,
            summary,
            args_summary: "",
            section,
            ui: CommandUi::empty(summary),
            handler,
            is_disabled: false,
            disabled_reason: None,
        }
    }

    #[test]
    fn resolves_longest_matching_path() {
        let mut registry = CommandRegistry::new();
        registry.add_section(SectionSpec {
            name: "config",
            prompt: "cfg> ",
            summary: "Config commands",
        });
        registry.add_command(spec(
            "config",
            &["config"],
            "config",
            "Config root",
            Some(Arc::new(noop)),
        ));
        registry.add_command(spec(
            "config.show",
            &["config", "show"],
            "config",
            "Show config",
            Some(Arc::new(noop)),
        ));

        let args = vec!["config".to_owned(), "show".to_owned(), "--x".to_owned()];
        let (command, rest) = registry.resolve(&args).expect("command should resolve");
        assert_eq!(command.id, "config.show");
        assert_eq!(rest, &["--x".to_owned()]);
    }

    #[test]
    fn execute_dispatches_to_registered_handler() {
        let mut registry = CommandRegistry::new();
        registry.add_section(SectionSpec {
            name: "verify",
            prompt: "verify> ",
            summary: "Verification commands",
        });
        registry.add_command(CommandSpec {
            id: "verify.package",
            path: &["verify", "package"],
            summary: "Verify package",
            args_summary: "[--configuration <name>]",
            section: "verify",
            ui: CommandUi::empty("Verify package"),
            handler: Some(Arc::new(report_handler)),
            is_disabled: false,
            disabled_reason: None,
        });

        let result = registry
            .execute(
                &context(),
                &[
                    "verify".to_owned(),
                    "package".to_owned(),
                    "--configuration".to_owned(),
                    "Release".to_owned(),
                ],
            )
            .expect("command should execute");

        assert_eq!(result.exit_code, 0);
        let report = result.report.expect("report should be returned");
        assert_eq!(report.title, "dispatch");
        assert_eq!(report.fields[0].value, "--configuration Release");
    }

    #[test]
    fn help_text_groups_commands_by_section() {
        let mut registry = CommandRegistry::new();
        registry.add_section(SectionSpec {
            name: "config",
            prompt: "cfg> ",
            summary: "Config commands",
        });
        registry.add_command(spec(
            "config.show",
            &["config", "show"],
            "config",
            "Show config",
            Some(Arc::new(noop)),
        ));

        let help = registry.help_text();
        assert!(help.contains("Dhara tool commands:"));
        assert!(help.contains("config:"));
        assert!(help.contains("config show"));
        assert!(help.contains("Show config"));
    }

    #[test]
    fn upsert_replaces_handler_and_disable_flags() {
        let mut registry = CommandRegistry::new();
        registry.add_section(SectionSpec {
            name: "version",
            prompt: "v> ",
            summary: "Version",
        });
        registry.upsert_command(spec(
            "version.bump",
            &["version", "bump"],
            "version",
            "Bump",
            None,
        ));
        registry.upsert_command(CommandSpec {
            id: "version.bump",
            path: &["version", "bump"],
            summary: "Bump version",
            args_summary: "--part <major|minor|patch>",
            section: "version",
            ui: CommandUi::empty("Bump"),
            handler: Some(Arc::new(noop)),
            is_disabled: false,
            disabled_reason: None,
        });

        let command = registry
            .commands()
            .find(|c| c.id == "version.bump")
            .expect("upserted");
        assert!(!command.is_effectively_disabled());
        assert!(command.handler.is_some());
        assert_eq!(command.summary, "Bump version");
    }

    #[test]
    fn disabled_command_does_not_invoke_handler() {
        let mut registry = CommandRegistry::new();
        registry.add_section(SectionSpec {
            name: "version",
            prompt: "v> ",
            summary: "Version",
        });
        registry.add_command(CommandSpec {
            id: "version.bump",
            path: &["version", "bump"],
            summary: "Bump",
            args_summary: "",
            section: "version",
            ui: CommandUi::empty("Bump"),
            handler: Some(Arc::new(|_, _| {
                panic!("handler must not run when disabled");
            })),
            is_disabled: true,
            disabled_reason: Some("disabled by the developers"),
        });

        let result = registry
            .execute(&context(), &["version".to_owned(), "bump".to_owned()])
            .expect("disabled execute returns a result");
        assert_eq!(result.exit_code, 1);
        assert!(
            result
                .message
                .as_deref()
                .unwrap_or("")
                .contains("disabled by the developers")
        );
    }

    #[test]
    fn missing_handler_is_effectively_disabled() {
        let command = spec("version.set", &["version", "set"], "version", "Set", None);
        assert!(command.is_effectively_disabled());
        assert_eq!(
            command.disable_message(),
            "not implemented by this extension"
        );
    }

    #[test]
    fn help_marks_disabled_commands() {
        let mut registry = CommandRegistry::new();
        registry.add_section(SectionSpec {
            name: "version",
            prompt: "v> ",
            summary: "Version",
        });
        registry.add_command(spec(
            "version.set",
            &["version", "set"],
            "version",
            "Set version",
            None,
        ));
        let help = registry.help_text();
        assert!(help.contains("(disabled)"));
        assert!(help.contains("not implemented by this extension"));
    }
}
