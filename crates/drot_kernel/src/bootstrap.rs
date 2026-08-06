use crate::command::{CommandRegistry, Extension};

/// Registers compile-time product extensions into the command registry.
pub fn register_extensions(
    registry: &mut CommandRegistry,
    extensions: impl IntoIterator<Item = Box<dyn Extension>>,
) {
    for extension in extensions {
        extension.register(registry);
    }
}
