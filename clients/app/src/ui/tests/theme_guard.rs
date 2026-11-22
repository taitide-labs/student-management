use std::collections::BTreeSet;

use serde_json::Value;

#[test]
fn light_and_dark_theme_keys_are_symmetric_and_non_empty() {
    let raw = include_str!("../../../theme.json");
    let root: Value = serde_json::from_str(raw).expect("theme.json must be valid JSON");
    let css_vars = root
        .get("cssVars")
        .expect("theme.json must contain cssVars");
    let light = css_vars
        .get("light")
        .and_then(Value::as_object)
        .expect("light palette object");
    let dark = css_vars
        .get("dark")
        .and_then(Value::as_object)
        .expect("dark palette object");

    let light_keys: BTreeSet<&str> = light.keys().map(String::as_str).collect();
    let dark_keys: BTreeSet<&str> = dark.keys().map(String::as_str).collect();
    assert_eq!(light_keys, dark_keys, "light and dark keys must match");

    for required in [
        "background",
        "foreground",
        "card",
        "popover",
        "primary",
        "secondary",
        "muted",
        "muted-foreground",
        "accent",
        "destructive",
        "border",
        "input",
        "ring",
        "sidebar",
        "sidebar-foreground",
        "sidebar-accent",
        "sidebar-border",
        "success",
        "warning",
        "radius-card",
        "radius-inset",
    ] {
        assert!(
            light_keys.contains(required),
            "missing required token: {required}"
        );
    }

    for (key, value) in light.iter().chain(dark.iter()) {
        let text = value.as_str().unwrap_or_default().trim();
        assert!(!text.is_empty(), "empty value for theme token: {key}");
    }
}
