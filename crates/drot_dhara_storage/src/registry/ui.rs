use drot_kernel::{ArgBinding, CommandUi, FieldKind, FieldSpec, PresetOption};

use super::strings::s;

pub(crate) const VERSION_PARTS: &[&str] = &["major", "minor", "patch"];
pub(crate) const CONFIGURATIONS: &[&str] = &["Debug", "Release"];
pub(crate) const DRY_RUN_OPTIONS: &[&str] = &["dry-run", "execute"];
pub(crate) const NATIVE_SCOPES: &[&str] = &["Host only", "Host + cross-native"];

pub(crate) const BUILD_RUN_PRESETS: &[PresetOption] = &[
    PresetOption {
        id: "local-integration",
        label: "Local integration",
    },
    PresetOption {
        id: "production-parity",
        label: "Production parity",
    },
    PresetOption {
        id: "custom",
        label: "Custom",
    },
];

pub(crate) const RELEASE_RUN_PRESETS: &[PresetOption] = &[
    PresetOption {
        id: "dry-run",
        label: "Dry run",
    },
    PresetOption {
        id: "full-release",
        label: "Full release",
    },
];

pub(crate) fn ui_for_command(
    id: &'static str,
    summary: &'static str,
    args_summary: &'static str,
) -> CommandUi {
    match id {
        "config.show" => quick_command(s("cmd.config.show.description"), false),
        "config.env.init" => quick_command(s("cmd.config.env.init.description"), false),
        "version.set" => CommandUi {
            description: s("cmd.version.set.description"),
            fields: vec![text_field(
                "version",
                s("cmd.version.set.field.version.label"),
                s("cmd.version.set.field.version.help"),
                ArgBinding::Positional,
                true,
                None,
                Some(s("group.general")),
            )],
            quick_run: true,
            supports_cancel: false,
        },
        "version.bump" => CommandUi {
            description: s("cmd.version.bump.description"),
            fields: vec![combo_field(
                "part",
                s("cmd.version.bump.field.part.label"),
                s("cmd.version.bump.field.part.help"),
                "--part",
                VERSION_PARTS,
                Some("minor"),
                None,
                Some(s("group.general")),
            )],
            quick_run: true,
            supports_cancel: false,
        },
        "defs.pack" => CommandUi {
            description: s("cmd.defs.pack.description"),
            fields: vec![optional_path(
                "output",
                s("cmd.defs.pack.field.output.label"),
                s("cmd.defs.pack.field.output.help"),
                "--output",
                Some(s("group.paths")),
            )],
            quick_run: false,
            supports_cancel: false,
        },
        "defs.build-trid-xml" => CommandUi {
            description: s("cmd.defs.build-trid-xml.description"),
            fields: vec![
                optional_path(
                    "input",
                    s("cmd.defs.build-trid-xml.field.input.label"),
                    s("cmd.defs.build-trid-xml.field.input.help"),
                    "--input",
                    Some(s("group.paths")),
                ),
                optional_path(
                    "output",
                    s("cmd.defs.build-trid-xml.field.output.label"),
                    s("cmd.defs.build-trid-xml.field.output.help"),
                    "--output",
                    Some(s("group.paths")),
                ),
            ],
            quick_run: false,
            supports_cancel: false,
        },
        "defs.inspect" => CommandUi {
            description: s("cmd.defs.inspect.description"),
            fields: vec![optional_path(
                "input",
                s("cmd.defs.inspect.field.input.label"),
                s("cmd.defs.inspect.field.input.help"),
                "--input",
                Some(s("group.paths")),
            )],
            quick_run: false,
            supports_cancel: false,
        },
        "defs.inspect-trid-xml" => CommandUi {
            description: s("cmd.defs.inspect-trid-xml.description"),
            fields: vec![optional_path(
                "input",
                s("cmd.defs.inspect-trid-xml.field.input.label"),
                s("cmd.defs.inspect-trid-xml.field.input.help"),
                "--input",
                Some(s("group.paths")),
            )],
            quick_run: false,
            supports_cancel: false,
        },
        "defs.normalize" => CommandUi {
            description: s("cmd.defs.normalize.description"),
            fields: vec![
                optional_path(
                    "input",
                    s("cmd.defs.normalize.field.input.label"),
                    s("cmd.defs.normalize.field.input.help"),
                    "--input",
                    Some(s("group.paths")),
                ),
                optional_path(
                    "output",
                    s("cmd.defs.normalize.field.output.label"),
                    s("cmd.defs.normalize.field.output.help"),
                    "--output",
                    Some(s("group.paths")),
                ),
            ],
            quick_run: false,
            supports_cancel: false,
        },
        "defs.verify" => CommandUi {
            description: s("cmd.defs.verify.description"),
            fields: vec![
                required_path(
                    "left",
                    s("cmd.defs.verify.field.left.label"),
                    s("cmd.defs.verify.field.left.help"),
                    "--left",
                    Some(s("group.paths")),
                ),
                required_path(
                    "right",
                    s("cmd.defs.verify.field.right.label"),
                    s("cmd.defs.verify.field.right.help"),
                    "--right",
                    Some(s("group.paths")),
                ),
            ],
            quick_run: false,
            supports_cancel: false,
        },
        "defs.sync-embedded" => CommandUi {
            description: s("cmd.defs.sync-embedded.description"),
            fields: vec![
                optional_path(
                    "input",
                    s("cmd.defs.sync-embedded.field.input.label"),
                    s("cmd.defs.sync-embedded.field.input.help"),
                    "--input",
                    Some(s("group.paths")),
                ),
                optional_path(
                    "output",
                    s("cmd.defs.sync-embedded.field.output.label"),
                    s("cmd.defs.sync-embedded.field.output.help"),
                    "--output",
                    Some(s("group.paths")),
                ),
                FieldSpec {
                    key: "check",
                    label: s("cmd.defs.sync-embedded.field.check.label"),
                    help: s("cmd.defs.sync-embedded.field.check.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--check"),
                    required: false,
                    default_value: Some("false"),
                    tui_default_value: None,
                    group: Some(s("group.mode")),
                    invert_switch: false,
                    tui_only: false,
                },
            ],
            quick_run: false,
            supports_cancel: false,
        },
        "verify.package" => package_command(s("cmd.verify.package.description")),
        "package.pack" => package_command(s("cmd.package.pack.description")),
        "package.publish" => CommandUi {
            description: s("cmd.package.publish.description"),
            fields: vec![
                radio_field(
                    "mode",
                    s("cmd.package.publish.field.mode.label"),
                    s("cmd.package.publish.field.mode.help"),
                    DRY_RUN_OPTIONS,
                    Some("dry-run"),
                    None,
                    Some(s("group.mode")),
                ),
                combo_field(
                    "configuration",
                    s("cmd.package.publish.field.configuration.label"),
                    s("cmd.package.publish.field.configuration.help"),
                    "--configuration",
                    CONFIGURATIONS,
                    Some("Release"),
                    None,
                    Some(s("group.package")),
                ),
                text_field(
                    "version",
                    s("cmd.package.publish.field.version.label"),
                    s("cmd.package.publish.field.version.help"),
                    ArgBinding::FlagValue("--version"),
                    false,
                    None,
                    Some(s("group.package")),
                ),
                text_field(
                    "source",
                    s("cmd.package.publish.field.source.label"),
                    s("cmd.package.publish.field.source.help"),
                    ArgBinding::FlagValue("--source"),
                    false,
                    None,
                    Some(s("group.package")),
                ),
            ],
            quick_run: false,
            supports_cancel: true,
        },
        "build.run" => build_run_ui(),
        "release.run" => release_run_ui(),
        _ => CommandUi {
            description: summary,
            fields: {
                let _ = args_summary;
                Vec::new()
            },
            quick_run: false,
            supports_cancel: false,
        },
    }
}

fn build_run_ui() -> CommandUi {
    let workflow = s("cmd.build.run.group.workflow");
    CommandUi {
        description: s("cmd.build.run.description"),
        fields: vec![
            preset_field(
                "preset",
                s("cmd.build.run.field.preset.label"),
                s("cmd.build.run.field.preset.help"),
                BUILD_RUN_PRESETS,
                Some(s("cmd.build.run.group.preset")),
            ),
            include_step(
                "step_config",
                s("cmd.build.run.field.step_config.label"),
                s("cmd.build.run.field.step_config.help"),
                "--skip-config",
                Some("true"),
                Some("true"),
                Some(workflow),
            ),
            include_step(
                "step_defs",
                s("cmd.build.run.field.step_defs.label"),
                s("cmd.build.run.field.step_defs.help"),
                "--skip-defs",
                Some("true"),
                Some("true"),
                Some(workflow),
            ),
            include_step(
                "step_quality",
                s("cmd.build.run.field.step_quality.label"),
                s("cmd.build.run.field.step_quality.help"),
                "--skip-quality",
                Some("true"),
                Some("true"),
                Some(workflow),
            ),
            include_step(
                "step_docs",
                s("cmd.build.run.field.step_docs.label"),
                s("cmd.build.run.field.step_docs.help"),
                "--skip-docs",
                Some("true"),
                Some("false"),
                Some(workflow),
            ),
            include_step(
                "step_dotnet",
                s("cmd.build.run.field.step_dotnet.label"),
                s("cmd.build.run.field.step_dotnet.help"),
                "--skip-dotnet",
                Some("true"),
                Some("true"),
                Some(workflow),
            ),
            include_step(
                "step_native",
                s("cmd.build.run.field.step_native.label"),
                s("cmd.build.run.field.step_native.help"),
                "--skip-native",
                Some("true"),
                Some("true"),
                Some(workflow),
            ),
            include_step(
                "step_verify",
                s("cmd.build.run.field.step_verify.label"),
                s("cmd.build.run.field.step_verify.help"),
                "--skip-verify",
                Some("true"),
                Some("false"),
                Some(workflow),
            ),
            combo_field(
                "native_scope",
                s("cmd.build.run.field.native_scope.label"),
                s("cmd.build.run.field.native_scope.help"),
                "--cross-native",
                NATIVE_SCOPES,
                Some("Host only"),
                Some("Host only"),
                Some(s("cmd.build.run.group.advanced")),
            ),
            combo_field(
                "configuration",
                s("cmd.build.run.field.configuration.label"),
                s("cmd.build.run.field.configuration.help"),
                "--configuration",
                CONFIGURATIONS,
                Some("Release"),
                Some("Debug"),
                Some(s("cmd.build.run.group.advanced")),
            ),
        ],
        quick_run: true,
        supports_cancel: true,
    }
}

fn release_run_ui() -> CommandUi {
    let steps = s("cmd.release.run.group.steps");
    CommandUi {
        description: s("cmd.release.run.description"),
        fields: vec![
            preset_field(
                "preset",
                s("cmd.release.run.field.preset.label"),
                s("cmd.release.run.field.preset.help"),
                RELEASE_RUN_PRESETS,
                Some(s("cmd.release.run.group.preset")),
            ),
            include_step(
                "step_cargo",
                s("cmd.release.run.field.step_cargo.label"),
                s("cmd.release.run.field.step_cargo.help"),
                "--skip-cargo",
                Some("true"),
                Some("false"),
                Some(steps),
            ),
            include_step(
                "step_nuget",
                s("cmd.release.run.field.step_nuget.label"),
                s("cmd.release.run.field.step_nuget.help"),
                "--skip-nuget",
                Some("true"),
                Some("false"),
                Some(steps),
            ),
            combo_field(
                "configuration",
                s("cmd.release.run.field.configuration.label"),
                s("cmd.release.run.field.configuration.help"),
                "--configuration",
                CONFIGURATIONS,
                Some("Release"),
                None,
                Some(s("cmd.release.run.group.advanced")),
            ),
            text_field(
                "source",
                s("cmd.release.run.field.source.label"),
                s("cmd.release.run.field.source.help"),
                ArgBinding::FlagValue("--source"),
                false,
                None,
                Some(s("cmd.release.run.group.advanced")),
            ),
            FieldSpec {
                key: "dry_run",
                label: s("cmd.release.run.field.dry_run.label"),
                help: s("cmd.release.run.field.dry_run.help"),
                kind: FieldKind::Boolean,
                binding: ArgBinding::Switch("--dry-run"),
                required: false,
                default_value: Some("false"),
                tui_default_value: Some("true"),
                group: Some(s("cmd.release.run.group.advanced")),
                invert_switch: false,
                tui_only: false,
            },
        ],
        quick_run: false,
        supports_cancel: true,
    }
}

fn quick_command(description: &'static str, supports_cancel: bool) -> CommandUi {
    CommandUi {
        description,
        fields: Vec::new(),
        quick_run: true,
        supports_cancel,
    }
}

fn package_command(description: &'static str) -> CommandUi {
    CommandUi {
        description,
        fields: vec![
            combo_field(
                "configuration",
                s("cmd.package.field.configuration.label"),
                s("cmd.package.field.configuration.help"),
                "--configuration",
                CONFIGURATIONS,
                Some("Release"),
                None,
                Some(s("group.package")),
            ),
            text_field(
                "version",
                s("cmd.package.field.version.label"),
                s("cmd.package.field.version.help"),
                ArgBinding::FlagValue("--version"),
                false,
                None,
                Some(s("group.package")),
            ),
        ],
        quick_run: true,
        supports_cancel: true,
    }
}

fn preset_field(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    options: &'static [PresetOption],
    group: Option<&'static str>,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Preset(options),
        binding: ArgBinding::Switch("__preset"),
        required: false,
        default_value: Some(options.first().map(|o| o.id).unwrap_or("")),
        tui_default_value: None,
        group,
        invert_switch: false,
        tui_only: true,
    }
}

fn include_step(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    flag: &'static str,
    cli_default: Option<&'static str>,
    tui_default: Option<&'static str>,
    group: Option<&'static str>,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Boolean,
        binding: ArgBinding::Switch(flag),
        required: false,
        default_value: cli_default,
        tui_default_value: tui_default,
        group,
        invert_switch: true,
        tui_only: false,
    }
}

fn combo_field(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    flag: &'static str,
    options: &'static [&'static str],
    default_value: Option<&'static str>,
    tui_default_value: Option<&'static str>,
    group: Option<&'static str>,
) -> FieldSpec {
    let binding = if flag == "--cross-native" {
        ArgBinding::Switch(flag)
    } else {
        ArgBinding::FlagValue(flag)
    };
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Combo(options),
        binding,
        required: false,
        default_value,
        tui_default_value,
        group,
        invert_switch: false,
        tui_only: false,
    }
}

fn radio_field(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    options: &'static [&'static str],
    default_value: Option<&'static str>,
    tui_default_value: Option<&'static str>,
    group: Option<&'static str>,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Radio(options),
        binding: ArgBinding::FlagValue("__mode"),
        required: false,
        default_value,
        tui_default_value,
        group,
        invert_switch: false,
        tui_only: false,
    }
}

fn text_field(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    binding: ArgBinding,
    required: bool,
    default_value: Option<&'static str>,
    group: Option<&'static str>,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Text,
        binding,
        required,
        default_value,
        tui_default_value: None,
        group,
        invert_switch: false,
        tui_only: false,
    }
}

fn required_path(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    flag: &'static str,
    group: Option<&'static str>,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Path,
        binding: ArgBinding::FlagValue(flag),
        required: true,
        default_value: None,
        tui_default_value: None,
        group,
        invert_switch: false,
        tui_only: false,
    }
}

fn optional_path(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    flag: &'static str,
    group: Option<&'static str>,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Path,
        binding: ArgBinding::FlagValue(flag),
        required: false,
        default_value: None,
        tui_default_value: None,
        group,
        invert_switch: false,
        tui_only: false,
    }
}
