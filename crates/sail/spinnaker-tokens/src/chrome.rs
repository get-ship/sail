//! Manifest-generated structural CSS (body wash, overlay, scrollbar, color-scheme).

use serde_json::Value;

/// Shared structural rules. Token channels (`--gradient-surface`,
/// scrollbar vars) are the only paint inputs. Platform chrome is empty — deck owns document chrome.
pub fn generate_platform_chrome() -> String {
    String::new()
}

/// Per-theme structural rules from `$extensions.sail.chrome` (data, not files).
///
/// Each entry is either:
/// - `{ "selector": "body", "props": { "background-image": "…" }, "media"?: "…" }`
/// - `{ "keyframes": "name", "frames": { "0%": { … }, "100%": { … } } }`
///
/// The engine prefixes `:root[theme='id']` / `[theme='id']` so a theme cannot
/// leak global selectors (already-scoped `:root` / `[theme` selectors pass through).
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
        if let Some(name) = entry.get("keyframes").and_then(|v| v.as_str()) {
            if let Some(chunk) = emit_keyframes(name, entry.get("frames")) {
                out.push_str(&chunk);
            }
            continue;
        }
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
        let block = emit_rule(&scoped, props);
        if let Some(media) = entry.get("media").and_then(|v| v.as_str()) {
            out.push_str(&format!("@media {media} {{\n"));
            out.push_str(&indent_block(&block));
            out.push_str("}\n");
        } else {
            out.push_str(&block);
        }
    }
    out
}

fn emit_keyframes(name: &str, frames: Option<&Value>) -> Option<String> {
    let frames = frames.and_then(|v| v.as_object())?;
    if frames.is_empty() {
        return None;
    }
    let mut out = format!("@keyframes {name} {{\n");
    for (at, props) in frames {
        let Some(obj) = props.as_object() else {
            continue;
        };
        if obj.is_empty() {
            continue;
        }
        out.push_str(&format!("  {at} {{\n"));
        for (k, v) in obj {
            if k.starts_with('$') {
                continue;
            }
            out.push_str(&format!("    {k}: {};\n", json_css_value(v)));
        }
        out.push_str("  }\n");
    }
    out.push_str("}\n");
    Some(out)
}

fn emit_rule(selector: &str, props: &serde_json::Map<String, Value>) -> String {
    let mut out = String::new();
    out.push_str(selector);
    out.push_str(" {\n");
    for (k, v) in props {
        if k.starts_with('$') {
            continue;
        }
        out.push_str(&format!("  {k}: {};\n", json_css_value(v)));
    }
    out.push_str("}\n");
    out
}

fn indent_block(block: &str) -> String {
    block
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("  {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn json_css_value(v: &Value) -> String {
    match v {
        Value::String(s) => resolve_token_ref(s),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// `{shadow-accent}` in chrome data → `var(--shadow-accent)` in CSS.
fn resolve_token_ref(s: &str) -> String {
    let t = s.trim();
    if let Some(inner) = t.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
        let name = inner.trim();
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return format!("var(--{name})");
        }
    }
    t.to_string()
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
        assert!(
            css.is_empty(),
            "platform chrome should be empty — deck owns document chrome"
        );
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

    #[test]
    fn theme_chrome_wraps_media_and_emits_keyframes() {
        let doc = json!({
            "$extensions": {
                "sail": {
                    "chrome": [
                        {
                            "selector": "body",
                            "media": "(prefers-reduced-motion: no-preference)",
                            "props": { "animation": "glass-mesh 36s ease-in-out infinite alternate" }
                        },
                        {
                            "keyframes": "glass-mesh",
                            "frames": {
                                "0%": { "background-position": "0% 0%" },
                                "100%": { "background-position": "8% 4%" }
                            }
                        }
                    ]
                }
            }
        });
        let css = generate_themes_chrome("glass", &doc);
        assert!(css.contains("@media (prefers-reduced-motion: no-preference)"));
        assert!(css.contains("[theme='glass']"));
        assert!(css.contains("@keyframes glass-mesh"));
        assert!(css.contains("background-position: 0% 0%"));
    }
}
