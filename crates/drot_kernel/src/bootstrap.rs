use crate::command::{CommandRegistry, ToolCapability};

pub fn register_plugins(
    registry: &mut CommandRegistry,
    plugins: impl IntoIterator<Item = Box<dyn ToolCapability>>,
) {
    for plugin in plugins {
        plugin.register(registry);
    }
}
