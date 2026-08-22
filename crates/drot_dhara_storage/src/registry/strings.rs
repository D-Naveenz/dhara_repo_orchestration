//! Keyed product UI copy for command articles and option fields.

pub(crate) fn s(key: &str) -> &'static str {
    match key {
        // config
        "cmd.config.show.description" => {
            "Inspect the effective Dhara repository configuration and resolved environment."
        }
        "cmd.config.env.init.description" => {
            "Create .env.local from .env.example when the local file is missing."
        }

        // version
        "cmd.version.set.description" => {
            "Set the shared workspace version used by both Cargo and NuGet metadata."
        }
        "cmd.version.set.field.version.label" => "Version",
        "cmd.version.set.field.version.help" => {
            "Semantic version to write into dhara.config.toml and synchronized package metadata."
        }
        "cmd.version.bump.description" => {
            "Bump the shared workspace version using semantic-version part semantics."
        }
        "cmd.version.bump.field.part.label" => "Part",
        "cmd.version.bump.field.part.help" => {
            "Which portion of the shared workspace version should be incremented."
        }

        // defs
        "cmd.defs.pack.description" => {
            "Copy the runtime filedefs.dat from the repository embed path to an output file."
        }
        "cmd.defs.pack.field.output.label" => "Output",
        "cmd.defs.pack.field.output.help" => "Optional output file path.",
        "cmd.defs.build-trid-xml.description" => {
            "Build a filedefs.dat package from TrID XML sources or archives."
        }
        "cmd.defs.build-trid-xml.field.input.label" => "Input",
        "cmd.defs.build-trid-xml.field.input.help" => "Optional TrID XML input path or archive.",
        "cmd.defs.build-trid-xml.field.output.label" => "Output",
        "cmd.defs.build-trid-xml.field.output.help" => "Optional output package path.",
        "cmd.defs.inspect.description" => {
            "Inspect an encoded FlatBuffers package and summarize its metadata and counts."
        }
        "cmd.defs.inspect.field.input.label" => "Input",
        "cmd.defs.inspect.field.input.help" => "Optional package path to inspect.",
        "cmd.defs.inspect-trid-xml.description" => {
            "Preview TrID XML transformation results without writing an output package."
        }
        "cmd.defs.inspect-trid-xml.field.input.label" => "Input",
        "cmd.defs.inspect-trid-xml.field.input.help" => "Optional TrID XML source path.",
        "cmd.defs.normalize.description" => {
            "Normalize an existing FlatBuffers package into the canonical builder format."
        }
        "cmd.defs.normalize.field.input.label" => "Input",
        "cmd.defs.normalize.field.input.help" => "Optional source package path.",
        "cmd.defs.normalize.field.output.label" => "Output",
        "cmd.defs.normalize.field.output.help" => "Optional normalized output path.",
        "cmd.defs.verify.description" => {
            "Compare two FlatBuffers packages for semantic equivalence."
        }
        "cmd.defs.verify.field.left.label" => "Left",
        "cmd.defs.verify.field.left.help" => "Left-hand package path.",
        "cmd.defs.verify.field.right.label" => "Right",
        "cmd.defs.verify.field.right.help" => "Right-hand package path.",
        "cmd.defs.sync-embedded.description" => {
            "Refresh the runtime filedefs.dat package in dhara_storage/resources from the builder source."
        }
        "cmd.defs.sync-embedded.field.input.label" => "Input",
        "cmd.defs.sync-embedded.field.input.help" => "Optional TrID XML archive or directory path.",
        "cmd.defs.sync-embedded.field.output.label" => "Output",
        "cmd.defs.sync-embedded.field.output.help" => "Optional embedded package output path.",
        "cmd.defs.sync-embedded.field.check.label" => "Check only",
        "cmd.defs.sync-embedded.field.check.help" => {
            "Validate whether the embedded package is up to date without writing changes."
        }

        // package / verify
        "cmd.verify.package.description" => {
            "Pack and verify the Dhara.Storage NuGet package, including smoke-consumer validation."
        }
        "cmd.package.pack.description" => {
            "Pack the Dhara.Storage NuGet package with staged native assets."
        }
        "cmd.package.field.configuration.label" => "Configuration",
        "cmd.package.field.configuration.help" => {
            "Build configuration used for verification and packing."
        }
        "cmd.package.field.version.label" => "Version override",
        "cmd.package.field.version.help" => {
            "Optional package version override. Leave empty to use dhara.config.toml."
        }
        "cmd.package.publish.description" => {
            "Verify and optionally publish the Dhara.Storage NuGet package."
        }
        "cmd.package.publish.field.configuration.label" => "Configuration",
        "cmd.package.publish.field.configuration.help" => {
            "Build configuration used during package verification and packing."
        }
        "cmd.package.publish.field.version.label" => "Version override",
        "cmd.package.publish.field.version.help" => {
            "Optional package version override. Leave empty to use dhara.config.toml."
        }
        "cmd.package.publish.field.source.label" => "Source",
        "cmd.package.publish.field.source.help" => "Optional NuGet source URL override.",
        "cmd.package.publish.field.mode.label" => "Mode",
        "cmd.package.publish.field.mode.help" => {
            "Choose whether to publish or perform a dry run only."
        }

        // build.run
        "cmd.build.run.description" => {
            "Run the full local repository build: config sync, definitions, quality, native staging, and package verification."
        }
        "cmd.build.run.field.skip_config.label" => "Skip config",
        "cmd.build.run.field.skip_config.help" => {
            "Do not apply dhara.config.toml drift into manifests."
        }
        "cmd.build.run.field.skip_defs.label" => "Skip definitions",
        "cmd.build.run.field.skip_defs.help" => {
            "Do not refresh core/dhara_storage/resources/filedefs.dat."
        }
        "cmd.build.run.field.skip_quality.label" => "Skip quality",
        "cmd.build.run.field.skip_quality.help" => "Do not run fmt, clippy, doc, and tests.",
        "cmd.build.run.field.skip_docs.label" => "Skip docs",
        "cmd.build.run.field.skip_docs.help" => "Skip cargo doc when quality checks run.",
        "cmd.build.run.field.skip_dotnet.label" => "Skip .NET tests",
        "cmd.build.run.field.skip_dotnet.help" => "Skip dotnet test when quality checks run.",
        "cmd.build.run.field.skip_native.label" => "Skip native staging",
        "cmd.build.run.field.skip_native.help" => {
            "Do not build and stage dhara-sd for host runtimes."
        }
        "cmd.build.run.field.cross_native.label" => "Cross-native",
        "cmd.build.run.field.cross_native.help" => {
            "Also stage cross-native targets buildable on this host (for example win-arm64 on Windows x64). Requires MSVC ARM64 build tools when enabled on Windows."
        }
        "cmd.build.run.field.skip_verify.label" => "Skip verify",
        "cmd.build.run.field.skip_verify.help" => "Do not pack and verify the NuGet package.",
        "cmd.build.run.field.configuration.label" => "Configuration",
        "cmd.build.run.field.configuration.help" => {
            "Build configuration used for native staging and package verification."
        }
        "cmd.build.run.group.preset" => "Preset",
        "cmd.build.run.group.workflow" => "Workflow steps",
        "cmd.build.run.group.advanced" => "Advanced",
        "cmd.build.run.field.preset.label" => "Build preset",
        "cmd.build.run.field.preset.help" => {
            "Quick TUI presets for local integration testing or production parity."
        }
        "cmd.build.run.field.step_config.label" => "Apply configuration sync",
        "cmd.build.run.field.step_config.help" => {
            "Synchronize dhara.config.toml into Cargo and NuGet manifests."
        }
        "cmd.build.run.field.step_defs.label" => "Sync file definitions",
        "cmd.build.run.field.step_defs.help" => {
            "Refresh embedded filedefs.dat when TrID input is available."
        }
        "cmd.build.run.field.step_quality.label" => "Run quality checks",
        "cmd.build.run.field.step_quality.help" => {
            "Run fmt, clippy, doc, Rust tests, and dotnet test."
        }
        "cmd.build.run.field.step_docs.label" => "  cargo doc",
        "cmd.build.run.field.step_docs.help" => "Include cargo doc in quality checks.",
        "cmd.build.run.field.step_dotnet.label" => "  dotnet test",
        "cmd.build.run.field.step_dotnet.help" => "Include dotnet test in quality checks.",
        "cmd.build.run.field.step_native.label" => "Stage native sidecar",
        "cmd.build.run.field.step_native.help" => {
            "Build and stage dhara-sd for selected host runtimes."
        }
        "cmd.build.run.field.step_verify.label" => "Verify NuGet package",
        "cmd.build.run.field.step_verify.help" => {
            "Pack and run consumer smoke checks (Release configuration required)."
        }
        "cmd.build.run.field.native_scope.label" => "Native scope",
        "cmd.build.run.field.native_scope.help" => {
            "Host-only staging for local work, or include cross-native targets."
        }

        // release.run workflow
        "cmd.release.run.group.preset" => "Preset",
        "cmd.release.run.group.steps" => "Release steps",
        "cmd.release.run.group.advanced" => "Advanced",
        "cmd.release.run.field.preset.label" => "Release preset",
        "cmd.release.run.field.preset.help" => {
            "Dry-run validation or full publish workflow."
        }
        "cmd.release.run.field.step_cargo.label" => "Publish Cargo crates",
        "cmd.release.run.field.step_cargo.help" => "Run the Cargo release phase.",
        "cmd.release.run.field.step_nuget.label" => "Publish NuGet package",
        "cmd.release.run.field.step_nuget.help" => "Run NuGet pack and publish.",

        "group.paths" => "Paths",
        "group.package" => "Package",
        "group.mode" => "Mode",
        "group.general" => "General",
        "cmd.release.run.description" => {
            "Run the Cargo-first release workflow, with optional NuGet publishing."
        }
        "cmd.release.run.field.configuration.label" => "Configuration",
        "cmd.release.run.field.configuration.help" => {
            "Build configuration used when NuGet packaging is enabled."
        }
        "cmd.release.run.field.source.label" => "Source",
        "cmd.release.run.field.source.help" => "Optional NuGet source URL override.",
        "cmd.release.run.field.dry_run.label" => "Dry run",
        "cmd.release.run.field.dry_run.help" => {
            "Run Cargo and NuGet release validation without publishing."
        }
        "cmd.release.run.field.skip_cargo.label" => "Skip Cargo",
        "cmd.release.run.field.skip_cargo.help" => {
            "Skip the Cargo release phase when crates were already published."
        }
        "cmd.release.run.field.skip_nuget.label" => "Skip NuGet",
        "cmd.release.run.field.skip_nuget.help" => "Publish or dry-run only the Cargo release.",

        other => {
            debug_assert!(false, "missing product UI string key: {other}");
            "<missing>"
        }
    }
}
