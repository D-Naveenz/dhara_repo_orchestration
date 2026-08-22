use anyhow::{Result, bail};

use crate::command::{ArgBinding, CommandSpec, FieldKind, FieldSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormValue {
    Text(String),
    Boolean(bool),
    Select(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandForm {
    pub command_id: &'static str,
    pub selected_field: usize,
    pub values: Vec<FormValue>,
}

impl CommandForm {
    pub fn from_command(command: &CommandSpec) -> Self {
        Self::from_command_with_defaults(command, false)
    }

    pub fn from_command_tui(command: &CommandSpec) -> Self {
        Self::from_command_with_defaults(command, true)
    }

    fn from_command_with_defaults(command: &CommandSpec, tui: bool) -> Self {
        let values = command
            .ui
            .fields
            .iter()
            .map(|field| init_field_value(field, tui))
            .collect();

        Self {
            command_id: command.id,
            selected_field: 0,
            values,
        }
    }

    pub fn field_index(&self, command: &CommandSpec, key: &str) -> Option<usize> {
        command
            .ui
            .fields
            .iter()
            .position(|field| field.key == key)
    }

    pub fn set_boolean(&mut self, command: &CommandSpec, key: &str, value: bool) {
        if let Some(index) = self.field_index(command, key) {
            if let Some(FormValue::Boolean(current)) = self.values.get_mut(index) {
                *current = value;
            }
        }
    }

    pub fn set_select_by_value(&mut self, command: &CommandSpec, key: &str, option: &str) {
        let Some(index) = self.field_index(command, key) else {
            return;
        };
        let Some(field) = command.ui.fields.get(index) else {
            return;
        };
        let options = select_options(&field.kind);
        if let Some(sel) = options.iter().position(|candidate| *candidate == option) {
            if let Some(FormValue::Select(current)) = self.values.get_mut(index) {
                *current = sel;
            }
        }
    }

    pub fn set_select_index(&mut self, command: &CommandSpec, key: &str, index: usize) {
        if let Some(field_index) = self.field_index(command, key) {
            if let Some(FormValue::Select(current)) = self.values.get_mut(field_index) {
                *current = index;
            }
        }
    }

    pub fn boolean_at(&self, command: &CommandSpec, key: &str) -> Option<bool> {
        let index = self.field_index(command, key)?;
        match self.values.get(index)? {
            FormValue::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    pub fn select_label_at<'a>(&self, command: &CommandSpec, key: &str) -> Option<&'a str> {
        let index = self.field_index(command, key)?;
        let field = command.ui.fields.get(index)?;
        let FormValue::Select(sel) = self.values.get(index)? else {
            return None;
        };
        select_options(&field.kind).get(*sel).copied()
    }

    pub fn selected_value(&self) -> Option<&FormValue> {
        self.values.get(self.selected_field)
    }

    pub fn selected_value_mut(&mut self) -> Option<&mut FormValue> {
        self.values.get_mut(self.selected_field)
    }

    pub fn move_next(&mut self, field_count: usize) {
        if field_count == 0 {
            self.selected_field = 0;
            return;
        }
        self.selected_field = (self.selected_field + 1) % field_count;
    }

    pub fn move_previous(&mut self, field_count: usize) {
        if field_count == 0 {
            self.selected_field = 0;
            return;
        }
        self.selected_field = (self.selected_field + field_count - 1) % field_count;
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(FormValue::Text(value)) = self.selected_value_mut() {
            value.push(ch);
        }
    }

    pub fn backspace(&mut self) {
        if let Some(FormValue::Text(value)) = self.selected_value_mut() {
            value.pop();
        }
    }

    pub fn toggle_bool(&mut self) {
        if let Some(FormValue::Boolean(value)) = self.selected_value_mut() {
            *value = !*value;
        }
    }

    pub fn cycle_next_option(&mut self, command: &CommandSpec) {
        let selected_field = self.selected_field;
        if let (Some(FormValue::Select(index)), Some(field)) = (
            self.selected_value_mut(),
            command.ui.fields.get(selected_field),
        ) {
            let count = option_count(&field.kind);
            if count == 0 {
                *index = 0;
            } else {
                *index = (*index + 1) % count;
            }
        }
    }

    pub fn cycle_previous_option(&mut self, command: &CommandSpec) {
        let selected_field = self.selected_field;
        if let (Some(FormValue::Select(index)), Some(field)) = (
            self.selected_value_mut(),
            command.ui.fields.get(selected_field),
        ) {
            let count = option_count(&field.kind);
            if count == 0 {
                *index = 0;
            } else {
                *index = (*index + count - 1) % count;
            }
        }
    }

    pub fn set_radio_index(&mut self, command: &CommandSpec, field_index: usize, option_index: usize) {
        let Some(field) = command.ui.fields.get(field_index) else {
            return;
        };
        if !matches!(field.kind, FieldKind::Radio(_)) {
            return;
        }
        if let Some(FormValue::Select(current)) = self.values.get_mut(field_index) {
            *current = option_index;
        }
    }

    pub fn display_value(&self, command: &CommandSpec, index: usize) -> String {
        match (
            self.values.get(index),
            command.ui.fields.get(index).map(|field| &field.kind),
        ) {
            (Some(FormValue::Text(value)), _) => value.clone(),
            (Some(FormValue::Boolean(value)), _) => {
                if *value {
                    "yes".to_owned()
                } else {
                    "no".to_owned()
                }
            }
            (Some(FormValue::Select(selected)), Some(kind)) => select_options(kind)
                .get(*selected)
                .copied()
                .unwrap_or_default()
                .to_owned(),
            _ => String::new(),
        }
    }

    pub fn build_args(&self, command: &CommandSpec) -> Result<Vec<String>> {
        let mut args = Vec::new();
        for (field, value) in command.ui.fields.iter().zip(self.values.iter()) {
            if field.tui_only {
                continue;
            }
            match (&field.binding, value, &field.kind) {
                (ArgBinding::Positional, FormValue::Text(text), _) => {
                    let trimmed = text.trim();
                    if field.required && trimmed.is_empty() {
                        bail!("{} is required", field.label);
                    }
                    if !trimmed.is_empty() {
                        args.push(trimmed.to_owned());
                    }
                }
                (ArgBinding::FlagValue(flag), FormValue::Text(text), _) => {
                    let trimmed = text.trim();
                    if field.required && trimmed.is_empty() {
                        bail!("{} is required", field.label);
                    }
                    if !trimmed.is_empty() {
                        args.push((*flag).to_owned());
                        args.push(trimmed.to_owned());
                    }
                }
                (ArgBinding::FlagValue("__mode"), FormValue::Select(index), kind) => {
                    if let Some(option) = select_options(kind).get(*index) {
                        args.push(format!("--{option}"));
                    }
                }
                (ArgBinding::FlagValue(flag), FormValue::Select(index), kind) => {
                    if let Some(option) = select_options(kind).get(*index) {
                        args.push((*flag).to_owned());
                        args.push((*option).to_owned());
                    }
                }
                (ArgBinding::Switch(flag), FormValue::Boolean(enabled), _) => {
                    let emit = if field.invert_switch {
                        !*enabled
                    } else {
                        *enabled
                    };
                    if emit {
                        args.push((*flag).to_owned());
                    }
                }
                (ArgBinding::Switch(flag), FormValue::Select(index), kind) => {
                    let on = *index > 0;
                    if on {
                        args.push((*flag).to_owned());
                    }
                    let _ = kind;
                }
                _ => {}
            }
        }

        Ok(args)
    }
}

fn init_field_value(field: &FieldSpec, tui: bool) -> FormValue {
    match &field.kind {
        FieldKind::Text | FieldKind::Path | FieldKind::BrowsablePath { .. } => {
            FormValue::Text(field.effective_default(tui).unwrap_or_default().to_owned())
        }
        FieldKind::Boolean => FormValue::Boolean(field.effective_default(tui) == Some("true")),
        FieldKind::Select(options)
        | FieldKind::Combo(options)
        | FieldKind::Radio(options) => {
            let default = field
                .effective_default(tui)
                .unwrap_or(options.first().copied().unwrap_or(""));
            let index = options
                .iter()
                .position(|option| *option == default)
                .unwrap_or(0);
            FormValue::Select(index)
        }
        FieldKind::Preset(options) => {
            let default = field
                .effective_default(tui)
                .or_else(|| options.first().map(|option| option.id))
                .unwrap_or("");
            let index = options
                .iter()
                .position(|option| option.id == default)
                .unwrap_or(0);
            FormValue::Select(index)
        }
    }
}

pub fn select_options(kind: &FieldKind) -> &'static [&'static str] {
    match kind {
        FieldKind::Select(options) | FieldKind::Combo(options) | FieldKind::Radio(options) => {
            options
        }
        FieldKind::Text
        | FieldKind::Path
        | FieldKind::BrowsablePath { .. }
        | FieldKind::Boolean
        | FieldKind::Preset(_) => &[],
    }
}

pub fn preset_options(kind: &FieldKind) -> Option<&'static [crate::command::PresetOption]> {
    match kind {
        FieldKind::Preset(options) => Some(options),
        _ => None,
    }
}

pub fn preset_id(kind: &FieldKind, index: usize) -> Option<&'static str> {
    preset_options(kind)?.get(index).map(|option| option.id)
}

pub fn option_count(kind: &FieldKind) -> usize {
    preset_options(kind)
        .map(|options| options.len())
        .unwrap_or_else(|| select_options(kind).len())
}

pub fn preset_label(kind: &FieldKind, index: usize) -> &str {
    if let Some(options) = preset_options(kind) {
        return options
            .get(index)
            .map(|option| option.label)
            .unwrap_or("");
    }
    select_options(kind).get(index).copied().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;

    use crate::command::{
        ArgBinding, CommandResult, CommandSpec, CommandUi, FieldKind, FieldSpec, ToolContext,
    };

    use super::CommandForm;

    fn noop(_: &ToolContext, _: &[String]) -> Result<CommandResult> {
        Ok(CommandResult::success())
    }

    fn command(fields: Vec<FieldSpec>) -> CommandSpec {
        CommandSpec {
            id: "verify.package",
            path: &["verify", "package"],
            summary: "Verify package",
            args_summary: "",
            section: "verify",
            ui: CommandUi {
                description: "Verify package",
                fields,
                quick_run: true,
                supports_cancel: true,
            },
            handler: Some(Arc::new(noop)),
            is_disabled: false,
            disabled_reason: None,
        }
    }

    #[test]
    fn build_args_serializes_flag_and_positional_fields() {
        let command = command(vec![
            FieldSpec {
                key: "version",
                label: "Version",
                help: "",
                kind: FieldKind::Text,
                binding: ArgBinding::Positional,
                required: true,
                default_value: Some("0.4.0"),
                tui_default_value: None,
                group: None,
                invert_switch: false,
                tui_only: false,
            },
            FieldSpec {
                key: "configuration",
                label: "Configuration",
                help: "",
                kind: FieldKind::Combo(&["Debug", "Release"]),
                binding: ArgBinding::FlagValue("--configuration"),
                required: true,
                default_value: Some("Release"),
                tui_default_value: None,
                group: None,
                invert_switch: false,
                tui_only: false,
            },
            FieldSpec {
                key: "check",
                label: "Check",
                help: "",
                kind: FieldKind::Boolean,
                binding: ArgBinding::Switch("--check"),
                required: false,
                default_value: Some("true"),
                tui_default_value: None,
                group: None,
                invert_switch: false,
                tui_only: false,
            },
        ]);

        let form = CommandForm::from_command(&command);
        let args = form.build_args(&command).unwrap();
        assert_eq!(
            args,
            vec![
                "0.4.0".to_owned(),
                "--configuration".to_owned(),
                "Release".to_owned(),
                "--check".to_owned()
            ]
        );
    }

    #[test]
    fn invert_switch_emits_skip_when_step_unchecked() {
        let command = command(vec![FieldSpec {
            key: "step_verify",
            label: "Verify package",
            help: "",
            kind: FieldKind::Boolean,
            binding: ArgBinding::Switch("--skip-verify"),
            required: false,
            default_value: Some("true"),
            tui_default_value: Some("false"),
            group: None,
            invert_switch: true,
            tui_only: false,
        }]);

        let tui_form = CommandForm::from_command_tui(&command);
        let args = tui_form.build_args(&command).unwrap();
        assert!(args.contains(&"--skip-verify".to_owned()));

        let cli_form = CommandForm::from_command(&command);
        let args = cli_form.build_args(&command).unwrap();
        assert!(!args.contains(&"--skip-verify".to_owned()));
    }

    #[test]
    fn tui_default_value_initializes_form() {
        let command = command(vec![FieldSpec {
            key: "step_verify",
            label: "Verify package",
            help: "",
            kind: FieldKind::Boolean,
            binding: ArgBinding::Switch("--skip-verify"),
            required: false,
            default_value: Some("true"),
            tui_default_value: Some("false"),
            group: None,
            invert_switch: true,
            tui_only: false,
        }]);

        let form = CommandForm::from_command_tui(&command);
        assert_eq!(form.boolean_at(&command, "step_verify"), Some(false));
    }

    #[test]
    fn build_args_requires_missing_required_values() {
        let command = command(vec![FieldSpec {
            key: "version",
            label: "Version",
            help: "",
            kind: FieldKind::Text,
            binding: ArgBinding::Positional,
            required: true,
            default_value: None,
            tui_default_value: None,
            group: None,
            invert_switch: false,
            tui_only: false,
        }]);

        let form = CommandForm::from_command(&command);
        let error = form.build_args(&command).unwrap_err().to_string();
        assert!(error.contains("Version is required"));
    }
}
