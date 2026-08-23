use drot_kernel::{CommandForm, CommandSpec};

pub fn apply_form_preset(form: &mut CommandForm, command: &CommandSpec, preset_id: &str) {
    match command.id {
        "build.run" => apply_build_run_preset(form, command, preset_id),
        "release.run" => apply_release_run_preset(form, command, preset_id),
        _ => {}
    }
}

pub fn default_preset_id(command_id: &str) -> Option<&'static str> {
    match command_id {
        "build.run" => Some("local-integration"),
        "release.run" => Some("dry-run"),
        _ => None,
    }
}

fn apply_build_run_preset(form: &mut CommandForm, command: &CommandSpec, preset_id: &str) {
    match preset_id {
        "local-integration" => {
            form.set_boolean(command, "step_config", true);
            form.set_boolean(command, "step_defs", true);
            form.set_boolean(command, "step_quality", true);
            form.set_boolean(command, "step_docs", false);
            form.set_boolean(command, "step_dotnet", true);
            form.set_boolean(command, "step_native", true);
            form.set_boolean(command, "step_verify", false);
            form.set_select_by_value(command, "native_scope", "Host only");
            form.set_select_by_value(command, "configuration", "Debug");
        }
        "production-parity" => {
            form.set_boolean(command, "step_config", true);
            form.set_boolean(command, "step_defs", true);
            form.set_boolean(command, "step_quality", true);
            form.set_boolean(command, "step_docs", true);
            form.set_boolean(command, "step_dotnet", true);
            form.set_boolean(command, "step_native", true);
            form.set_boolean(command, "step_verify", true);
            form.set_select_by_value(command, "native_scope", "Host + cross-native");
            form.set_select_by_value(command, "configuration", "Release");
        }
        _ => {}
    }
}

fn apply_release_run_preset(form: &mut CommandForm, command: &CommandSpec, preset_id: &str) {
    match preset_id {
        "dry-run" => {
            form.set_boolean(command, "step_cargo", false);
            form.set_boolean(command, "step_nuget", false);
            form.set_boolean(command, "dry_run", true);
            form.set_select_by_value(command, "configuration", "Release");
        }
        "full-release" => {
            form.set_boolean(command, "step_cargo", true);
            form.set_boolean(command, "step_nuget", true);
            form.set_boolean(command, "dry_run", false);
            form.set_select_by_value(command, "configuration", "Release");
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use drot_kernel::{CommandForm, CommandRegistry, Extension, register_base_commands};

    use crate::registry::DharaStorageExtension;

    use super::{apply_form_preset, default_preset_id};

    fn build_run_command(registry: &CommandRegistry) -> &drot_kernel::CommandSpec {
        registry
            .commands()
            .find(|command| command.id == "build.run")
            .expect("build.run registered")
    }

    fn registry_with_storage() -> CommandRegistry {
        let mut registry = CommandRegistry::new();
        register_base_commands(&mut registry);
        DharaStorageExtension.register(&mut registry);
        registry
    }

    #[test]
    fn default_preset_is_local_integration_for_build_run() {
        assert_eq!(default_preset_id("build.run"), Some("local-integration"));
    }

    #[test]
    fn local_integration_preset_serializes_debug_argv() {
        let registry = registry_with_storage();
        let command = build_run_command(&registry);
        let mut form = CommandForm::from_command_tui(command);
        apply_form_preset(&mut form, command, "local-integration");

        assert_eq!(
            form.select_label_at(command, "configuration"),
            Some("Debug")
        );
        assert_eq!(form.boolean_at(command, "step_verify"), Some(false));

        let args = form.build_args(command).expect("argv");
        assert!(args.contains(&"--configuration".to_owned()));
        assert!(args.contains(&"Debug".to_owned()));
        assert!(args.contains(&"--skip-verify".to_owned()));
        assert!(args.contains(&"--skip-docs".to_owned()));
    }

    #[test]
    fn production_parity_preset_omits_skip_verify() {
        let registry = registry_with_storage();
        let command = build_run_command(&registry);
        let mut form = CommandForm::from_command_tui(command);
        apply_form_preset(&mut form, command, "production-parity");

        let args = form.build_args(command).expect("argv");
        assert!(args.contains(&"--configuration".to_owned()));
        assert!(args.contains(&"Release".to_owned()));
        assert!(!args.contains(&"--skip-verify".to_owned()));
    }
}
