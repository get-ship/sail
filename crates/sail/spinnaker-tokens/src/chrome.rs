//! Manifest-generated structural CSS (body wash, overlay, scrollbar, color-scheme).

use serde_json::Value;

/// Shared structural rules. Token channels (`--gradient-surface`,
/// `--surface-overlay`, scrollbar vars) are the only paint inputs.
pub fn generate_platform_chrome() -> String {
    String::from(
        r#"/*
 * GENERATED FILE — do not edit by hand.
 * Structural chrome from the theme engine (spinnaker-tokens).
 * Regenerate: cargo run -p sail-compiler --offline -- tokens
 */

/* Surface wash — under content. Default --gradient-surface: none. */
body::before {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: -1;
  background: var(--gradient-surface);
}

/* Surface overlay — texture above content. Default --surface-overlay: none. */
body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 9999;
  background: var(--surface-overlay);
  mix-blend-mode: var(--surface-overlay-blend);
  opacity: var(--surface-overlay-opacity);
}

/* Themed subtree: overlay clips to the host, not the viewport. */
[theme]:not(:root) {
  position: relative;
  isolation: isolate;
}
[theme]:not(:root)::after {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: var(--z-max);
  background: var(--surface-overlay);
  mix-blend-mode: var(--surface-overlay-blend);
  opacity: var(--surface-overlay-opacity);
}
@media (prefers-reduced-motion: reduce) {
  body::after,
  [theme]:not(:root)::after { display: none; }
}

[theme][mode='light'] {
  color-scheme: light;
}
[theme][mode='dark'] {
  color-scheme: dark;
}

/* Scrollbar — spends scrollbar.* tokens. */
html {
  scrollbar-width: thin;
  scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
}
::-webkit-scrollbar {
  width: var(--scrollbar-width);
  height: var(--scrollbar-width);
}
::-webkit-scrollbar-track {
  background: var(--scrollbar-track);
}
::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: var(--scrollbar-radius);
}
"#,
    )
}

/// Per-theme structural rules from `$extensions.sail.chrome` (data, not files).
///
/// Each entry: `{ "selector": "body", "props": { "background-image": "…" } }`.
/// The engine prefixes `:root[theme='id']` / `[theme='id']` so a theme cannot
/// leak global selectors.
pub fn generate_themes_chrome(theme_id: &str, doc: &Value) -> String {
    let Some(entries) = doc
        .pointer("/$extensions/sail/chrome")
        .and_then(|v| v.as_array())
    else {
        return String::new();
    };
    if entries.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str(&format!("/* theme chrome: {theme_id} */\n"));
    for entry in entries {
        let Some(sel) = entry.get("selector").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(props) = entry.get("props").and_then(|v| v.as_object()) else {
            continue;
        };
        if props.is_empty() {
            continue;
        }
        let scoped = scope_selector(theme_id, sel);
        out.push_str(&scoped);
        out.push_str(" {\n");
        for (k, v) in props {
            if k.starts_with('$') {
                continue;
            }
            let val = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                other => other.to_string(),
            };
            out.push_str(&format!("  {k}: {val};\n"));
        }
        out.push_str("}\n");
    }
    out
}

fn scope_selector(theme_id: &str, sel: &str) -> String {
    let trimmed = sel.trim();
    if trimmed.starts_with(":root") || trimmed.contains("[theme") {
        return trimmed.to_string();
    }
    let parts: Vec<&str> = trimmed.split(',').map(str::trim).collect();
    parts
        .into_iter()
        .map(|p| {
            if p == "body" {
                format!(":root[theme='{theme_id}'] body, [theme='{theme_id}'].tc-bg")
            } else {
                format!(":root[theme='{theme_id}'] {p}, [theme='{theme_id}'] {p}")
            }
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

/// Concatenate catalog theme chrome (sorted by id).
pub fn generate_all_themes_chrome<'a>(
    themes: impl IntoIterator<Item = (&'a str, &'a Value)>,
) -> String {
    let mut parts = vec![String::from(
        "/*\n * GENERATED FILE — do not edit by hand.\n \
         * Per-theme structural chrome from $extensions.sail.chrome.\n \
         * Regenerate: cargo run -p sail-compiler --offline -- tokens\n */\n",
    )];
    let mut items: Vec<(&str, &Value)> = themes.into_iter().collect();
    items.sort_by(|a, b| a.0.cmp(b.0));
    for (id, doc) in items {
        let chunk = generate_themes_chrome(id, doc);
        if !chunk.is_empty() {
            parts.push(chunk);
        }
    }
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn platform_chrome_spends_registry_channels() {
        let css = generate_platform_chrome();
        assert!(css.contains("var(--gradient-surface)"));
        assert!(css.contains("var(--surface-overlay)"));
        assert!(css.contains("var(--scrollbar-thumb)"));
        assert!(css.contains("var(--z-max)"));
    }

    #[test]
    fn theme_chrome_is_scoped() {
        let doc = json!({
            "$extensions": {
                "sail": {
                    "chrome": [{
                        "selector": "body",
                        "props": { "background-attachment": "fixed" }
                    }]
                }
            }
        });
        let css = generate_themes_chrome("halo", &doc);
        assert!(css.contains("[theme='halo']"));
        assert!(css.contains("background-attachment: fixed"));
    }
}
