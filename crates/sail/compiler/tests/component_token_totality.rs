//! Living-contract after the component.* pack cut.
//!
//! History of the old totality matrix lives in git. The tree carries one
//! form: theme + bg/fg composites. These tests pin absence, not the retired
//! pack names.

use std::collections::BTreeSet;

const REGISTRY: &str = include_str!("../../../../packages/sail/tokens/src/token-registry.json");

#[test]
fn component_dot_paths_stay_cut() {
    let v: serde_json::Value = serde_json::from_str(REGISTRY).expect("registry json");
    let mut revived = Vec::new();
    if let Some(paths) = v.get("paths").and_then(|p| p.as_object()) {
        for path in paths.keys() {
            if path.starts_with("component.") {
                revived.push(path.clone());
            }
        }
    }
    assert!(
        revived.is_empty(),
        "component.* registry paths revived after the pack cut:\n  {}",
        revived.join("\n  ")
    );
}

#[test]
fn tokens1_cut_orphans_stay_gone() {
    let v: serde_json::Value = serde_json::from_str(REGISTRY).expect("registry");
    let names: BTreeSet<String> = v["names"]
        .as_array()
        .expect("names")
        .iter()
        .filter_map(|n| n.as_str().map(str::to_string))
        .collect();
    const CUT: &[&str] = &[
        "body-tracking",
        "glow-info",
        "glow-success",
        "glow-warning",
        "glow-destructive",
        "glow-default",
        "glow-muted",
        "app-accent-agents",
        "leading-normal-plus",
        "menu-min-w",
        "tab-selected",
        "toast-region-width",
        "toast-region-z",
    ];
    let mut revived = Vec::new();
    for c in CUT {
        if names.contains(*c) {
            revived.push(*c);
        }
    }
    assert!(
        revived.is_empty(),
        "TOKENS-1 cut orphans revived in registry: {revived:?}"
    );
}
