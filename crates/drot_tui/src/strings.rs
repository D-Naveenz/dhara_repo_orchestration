//! Keyed UI chrome strings for the TUI (locale-swappable later).

pub fn t(key: &str) -> &'static str {
    match key {
        "tab.info" => "Info",
        "tab.options" => "Options",
        "tab.troubleshooting" => "Troubleshooting",
        "tab.system" => "System",

        "doc.syntax" => "Syntax",
        "doc.options" => "Options",
        "doc.empty" => "Select a task in the tree to view its description.",

        "options.empty" => "Select a task to edit options.",
        "options.loading" => "Loading form…",
        "options.none" => "No options for this task. Press Run or r.",

        "trouble.empty" => "Warnings and errors appear here during a run.",
        "system.empty" => "Open this tab to load configuration.",

        other => {
            debug_assert!(false, "missing TUI string key: {other}");
            "<missing>"
        }
    }
}

pub fn tab_labels() -> [&'static str; 4] {
    [
        t("tab.info"),
        t("tab.options"),
        t("tab.troubleshooting"),
        t("tab.system"),
    ]
}
