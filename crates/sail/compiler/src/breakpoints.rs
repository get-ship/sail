//! Theme viewport breakpoints for the structural `<when {token}>` block.
//!
//! Names and media queries match Style condition keys
//! (`spinnaker_css::conditions`) so paint-level `.when({ tablet: … })`
//! and tree-level `<when {tablet}>` share one scale. Widths come from
//! the active theme's breakpoint table.

/// Resolve a breakpoint token to its media query. `None` when the name is
/// not on the active theme scale (raw px, unknown id, paint-only conditions).
pub fn breakpoint_media(token: &str) -> Option<String> {
    let t = token.trim();
    if !spinnaker::tokens::is_theme_breakpoint(t) {
        return None;
    }
    spinnaker_css::condition_media(t)
}

/// Human-readable list for diagnostics: `mobile, tablet, desktop`.
pub fn breakpoint_token_list() -> String {
    spinnaker::tokens::breakpoint_names_display()
}

/// Sibling `breakpoints.json` replaces the theme table for this file.
/// Used by the 1-bp gauntlet cell so `when: tablet` compiles away.
pub fn with_source_breakpoints<T>(
    source_path: Option<&std::path::Path>,
    f: impl FnOnce() -> T,
) -> T {
    match load_sibling_breakpoints(source_path) {
        Some(table) => {
            let refs: Vec<(&str, &str)> = table
                .iter()
                .map(|(n, w)| (n.as_str(), w.as_str()))
                .collect();
            spinnaker::tokens::with_breakpoint_table(&refs, f)
        }
        None => f(),
    }
}

fn load_sibling_breakpoints(
    source_path: Option<&std::path::Path>,
) -> Option<Vec<(String, String)>> {
    let path = source_path?;
    let sibling = path.parent()?.join("breakpoints.json");
    let text = std::fs::read_to_string(sibling).ok()?;
    let map: std::collections::BTreeMap<String, String> = serde_json::from_str(&text).ok()?;
    if map.is_empty() {
        return None;
    }
    Some(map.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_resolve_to_min_width_media() {
        for (name, _) in spinnaker::tokens::breakpoint_table() {
            let q = breakpoint_media(&name).unwrap_or_else(|| panic!("missing {name}"));
            assert!(
                q.contains("min-width"),
                "{name} media must be min-width: {q}"
            );
        }
    }

    #[test]
    fn rejects_raw_px_and_paint_conditions() {
        assert!(breakpoint_media("768px").is_none());
        assert!(breakpoint_media("(min-width: 768px)").is_none());
        assert!(breakpoint_media("motion-reduce").is_none());
        assert!(breakpoint_media("dark").is_none());
        assert!(breakpoint_media("bp-md").is_none());
    }

    #[test]
    fn matches_style_condition_table() {
        for (name, _) in spinnaker::tokens::breakpoint_table() {
            assert_eq!(
                breakpoint_media(&name),
                spinnaker_css::condition_media(&name),
                "{name} must equal Style condition media"
            );
        }
    }

    #[test]
    fn omitted_theme_breakpoint_is_none() {
        spinnaker::tokens::with_breakpoint_table(&[("mobile", "390px")], || {
            assert!(breakpoint_media("tablet").is_none());
            assert!(breakpoint_media("wide").is_none());
            assert!(breakpoint_media("mobile").unwrap().contains("390px"));
        });
    }

    #[test]
    fn sibling_breakpoints_json_replaces_table() {
        let dir = tempfile::tempdir().unwrap();
        let sail = dir.path().join("cell.sail");
        std::fs::write(&sail, "").unwrap();
        std::fs::write(
            dir.path().join("breakpoints.json"),
            r#"{ "mobile": "390px" }"#,
        )
        .unwrap();
        with_source_breakpoints(Some(&sail), || {
            assert!(breakpoint_media("tablet").is_none());
            assert!(breakpoint_media("mobile").unwrap().contains("390px"));
        });
    }
}
