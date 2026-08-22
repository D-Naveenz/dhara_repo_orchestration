use drot_kernel::{ArgBinding, CommandUi, FieldKind, FieldSpec};

use super::strings::s;

pub(crate) const VERSION_PARTS: &[&str] = &["major", "minor", "patch"];
pub(crate) const CONFIGURATIONS: &[&str] = &["Release"];
pub(crate) const DRY_RUN_OPTIONS: &[&str] = &["dry-run", "execute"];

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
            fields: vec![FieldSpec {
                key: "version",
                label: s("cmd.version.set.field.version.label"),
                help: s("cmd.version.set.field.version.help"),
                kind: FieldKind::Text,
                binding: ArgBinding::Positional,
                required: true,
                default_value: None,
            }],
            quick_run: true,
            supports_cancel: false,
        },
        "version.bump" => CommandUi {
            description: s("cmd.version.bump.description"),
            fields: vec![FieldSpec {
                key: "part",
                label: s("cmd.version.bump.field.part.label"),
                help: s("cmd.version.bump.field.part.help"),
                kind: FieldKind::Select(VERSION_PARTS),
                binding: ArgBinding::FlagValue("--part"),
                required: true,
                default_value: Some("minor"),
            }],
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
                ),
                optional_path(
                    "output",
                    s("cmd.defs.build-trid-xml.field.output.label"),
                    s("cmd.defs.build-trid-xml.field.output.help"),
                    "--output",
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
                ),
                optional_path(
                    "output",
                    s("cmd.defs.normalize.field.output.label"),
                    s("cmd.defs.normalize.field.output.help"),
                    "--output",
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
                ),
                required_path(
                    "right",
                    s("cmd.defs.verify.field.right.label"),
                    s("cmd.defs.verify.field.right.help"),
                    "--right",
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
                ),
                optional_path(
                    "output",
                    s("cmd.defs.sync-embedded.field.output.label"),
                    s("cmd.defs.sync-embedded.field.output.help"),
                    "--output",
                ),
                FieldSpec {
                    key: "check",
                    label: s("cmd.defs.sync-embedded.field.check.label"),
                    help: s("cmd.defs.sync-embedded.field.check.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--check"),
                    required: false,
                    default_value: Some("false"),
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
                FieldSpec {
                    key: "configuration",
                    label: s("cmd.package.publish.field.configuration.label"),
                    help: s("cmd.package.publish.field.configuration.help"),
                    kind: FieldKind::Select(CONFIGURATIONS),
                    binding: ArgBinding::FlagValue("--configuration"),
                    required: true,
                    default_value: Some("Release"),
                },
                FieldSpec {
                    key: "version",
                    label: s("cmd.package.publish.field.version.label"),
                    help: s("cmd.package.publish.field.version.help"),
                    kind: FieldKind::Text,
                    binding: ArgBinding::FlagValue("--version"),
                    required: false,
                    default_value: None,
                },
                FieldSpec {
                    key: "source",
                    label: s("cmd.package.publish.field.source.label"),
                    help: s("cmd.package.publish.field.source.help"),
                    kind: FieldKind::Text,
                    binding: ArgBinding::FlagValue("--source"),
                    required: false,
                    default_value: None,
                },
                FieldSpec {
                    key: "mode",
                    label: s("cmd.package.publish.field.mode.label"),
                    help: s("cmd.package.publish.field.mode.help"),
                    kind: FieldKind::Select(DRY_RUN_OPTIONS),
                    binding: ArgBinding::FlagValue("__mode"),
                    required: true,
                    default_value: Some("dry-run"),
                },
            ],
            quick_run: false,
            supports_cancel: true,
        },
        "build.run" => CommandUi {
            description: s("cmd.build.run.description"),
            fields: vec![
                FieldSpec {
                    key: "skip_config",
                    label: s("cmd.build.run.field.skip_config.label"),
                    help: s("cmd.build.run.field.skip_config.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-config"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "skip_defs",
                    label: s("cmd.build.run.field.skip_defs.label"),
                    help: s("cmd.build.run.field.skip_defs.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-defs"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "skip_quality",
                    label: s("cmd.build.run.field.skip_quality.label"),
                    help: s("cmd.build.run.field.skip_quality.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-quality"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "skip_docs",
                    label: s("cmd.build.run.field.skip_docs.label"),
                    help: s("cmd.build.run.field.skip_docs.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-docs"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "skip_dotnet",
                    label: s("cmd.build.run.field.skip_dotnet.label"),
                    help: s("cmd.build.run.field.skip_dotnet.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-dotnet"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "skip_native",
                    label: s("cmd.build.run.field.skip_native.label"),
                    help: s("cmd.build.run.field.skip_native.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-native"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "cross_native",
                    label: s("cmd.build.run.field.cross_native.label"),
                    help: s("cmd.build.run.field.cross_native.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--cross-native"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "skip_verify",
                    label: s("cmd.build.run.field.skip_verify.label"),
                    help: s("cmd.build.run.field.skip_verify.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-verify"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "configuration",
                    label: s("cmd.build.run.field.configuration.label"),
                    help: s("cmd.build.run.field.configuration.help"),
                    kind: FieldKind::Select(CONFIGURATIONS),
                    binding: ArgBinding::FlagValue("--configuration"),
                    required: true,
                    default_value: Some("Release"),
                },
            ],
            quick_run: true,
            supports_cancel: true,
        },
        "release.run" => CommandUi {
            description: s("cmd.release.run.description"),
            fields: vec![
                FieldSpec {
                    key: "configuration",
                    label: s("cmd.release.run.field.configuration.label"),
                    help: s("cmd.release.run.field.configuration.help"),
                    kind: FieldKind::Select(CONFIGURATIONS),
                    binding: ArgBinding::FlagValue("--configuration"),
                    required: true,
                    default_value: Some("Release"),
                },
                FieldSpec {
                    key: "source",
                    label: s("cmd.release.run.field.source.label"),
                    help: s("cmd.release.run.field.source.help"),
                    kind: FieldKind::Text,
                    binding: ArgBinding::FlagValue("--source"),
                    required: false,
                    default_value: None,
                },
                FieldSpec {
                    key: "dry_run",
                    label: s("cmd.release.run.field.dry_run.label"),
                    help: s("cmd.release.run.field.dry_run.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--dry-run"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "skip_cargo",
                    label: s("cmd.release.run.field.skip_cargo.label"),
                    help: s("cmd.release.run.field.skip_cargo.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-cargo"),
                    required: false,
                    default_value: Some("false"),
                },
                FieldSpec {
                    key: "skip_nuget",
                    label: s("cmd.release.run.field.skip_nuget.label"),
                    help: s("cmd.release.run.field.skip_nuget.help"),
                    kind: FieldKind::Boolean,
                    binding: ArgBinding::Switch("--skip-nuget"),
                    required: false,
                    default_value: Some("false"),
                },
            ],
            quick_run: false,
            supports_cancel: true,
        },
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
            FieldSpec {
                key: "configuration",
                label: s("cmd.package.field.configuration.label"),
                help: s("cmd.package.field.configuration.help"),
                kind: FieldKind::Select(CONFIGURATIONS),
                binding: ArgBinding::FlagValue("--configuration"),
                required: true,
                default_value: Some("Release"),
            },
            FieldSpec {
                key: "version",
                label: s("cmd.package.field.version.label"),
                help: s("cmd.package.field.version.help"),
                kind: FieldKind::Text,
                binding: ArgBinding::FlagValue("--version"),
                required: false,
                default_value: None,
            },
        ],
        quick_run: true,
        supports_cancel: true,
    }
}

fn required_path(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    flag: &'static str,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Path,
        binding: ArgBinding::FlagValue(flag),
        required: true,
        default_value: None,
    }
}

fn optional_path(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    flag: &'static str,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        help,
        kind: FieldKind::Path,
        binding: ArgBinding::FlagValue(flag),
        required: false,
        default_value: None,
    }
}
