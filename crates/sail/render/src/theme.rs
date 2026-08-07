//! Theme + token paint resolution shared with the lint token path.
//!
//! Values come from the DTCG registry (`sail_lint::tokens`) and optional
//! theme JSON `modes.{light|dark}` overlays. Never scrapes CSS (T-I1).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde_json::Value;

/// Color / mode selection for a render.
#[derive(Debug, Clone)]
pub struct ThemeSelect {
    /// Theme id (e.g. `ship`). Empty → registry base values only.
    pub id: String,
    /// `light` or `dark` (data-mode). Default `light`.
    pub mode: String,
}

impl Default for ThemeSelect {
    fn default() -> Self {
        Self {
            id: String::new(),
            mode: "light".into(),
        }
    }
}

impl ThemeSelect {
    pub fn parse(theme: Option<&str>, mode: Option<&str>) -> Self {
        let raw = theme.unwrap_or("").trim();
        // Accept `ship-dark` / `ship/dark` as id+mode.
        let (id, mode_from_id) = if let Some((a, b)) = raw.split_once('-') {
            if matches!(b, "light" | "dark") {
                (a.to_string(), Some(b.to_string()))
            } else {
                (raw.to_string(), None)
            }
        } else if let Some((a, b)) = raw.split_once('/') {
            (a.to_string(), Some(b.to_string()))
        } else {
            (raw.to_string(), None)
        };
        let mode = mode
            .map(|m| m.trim().to_string())
            .or(mode_from_id)
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| "light".into());
        Self { id, mode }
    }

    /// `theme` attribute value (theme id only).
    pub fn data_theme(&self) -> &str {
        if self.id.is_empty() { "ship" } else { &self.id }
    }

    /// `data-mode` attribute value.
    pub fn data_mode(&self) -> &str {
        &self.mode
    }
}

/// Resolved cssName → concrete CSS value for the selected theme/mode.
#[derive(Debug, Clone, Default)]
pub struct ThemePalette {
    values: HashMap<String, String>,
    pub theme: ThemeSelect,
}

impl ThemePalette {
    /// Build palette from the embedded registry, optionally overlaying a theme file.
    pub fn resolve(select: ThemeSelect, workspace: Option<&Path>) -> Result<Self> {
        let mut values = base_registry_values();
        if !select.id.is_empty() {
            if let Some(path) = find_theme_file(&select.id, workspace) {
                overlay_theme_mode(&mut values, &path, &select.mode)
                    .with_context(|| format!("loading theme {}", path.display()))?;
            }
        }
        // Expand simple `var(--x)` chains against the map (a few passes).
        for _ in 0..8 {
            if !expand_vars_once(&mut values) {
                break;
            }
        }
        Ok(Self {
            values,
            theme: select,
        })
    }

    /// Lookup a token by css name (no `--` prefix).
    pub fn get(&self, css_name: &str) -> Option<&str> {
        self.values.get(css_name).map(|s| s.as_str())
    }

    /// Resolve a CSS value that may contain `var(--token)`.
    pub fn resolve_css_value(&self, value: &str) -> String {
        resolve_value_against(value, &self.values, 0)
    }

    /// Background for the document root (theme `background` or `surface-0`).
    pub fn root_background(&self) -> &str {
        self.get("background")
            .or_else(|| self.get("surface-0"))
            .unwrap_or("oklch(1 0 0)")
    }

    /// Default text color.
    pub fn root_foreground(&self) -> &str {
        self.get("foreground").unwrap_or("oklch(0.15 0.004 50)")
    }

    /// Placeholder chrome colors (unknown subtrees) — muted/border tokens.
    pub fn placeholder_bg(&self) -> &str {
        self.get("muted")
            .or_else(|| self.get("surface-2"))
            .unwrap_or("oklch(0.95 0.003 106)")
    }

    pub fn placeholder_border(&self) -> &str {
        self.get("border")
            .or_else(|| self.get("border-subtle"))
            .unwrap_or("oklch(0.7 0 0)")
    }

    pub fn placeholder_fg(&self) -> &str {
        self.get("muted-foreground")
            .or_else(|| self.get("foreground"))
            .unwrap_or("oklch(0.45 0.02 50)")
    }

    /// Test helper: drop one token mapping (golden-fail probe).
    pub fn remove_mapping(&mut self, css_name: &str) {
        self.values.remove(css_name);
    }
}

fn base_registry_values() -> HashMap<String, String> {
    static CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut m = HashMap::new();
            // Walk known color + size tokens via public lint API.
            for name in sail_lint::tokens::color_values() {
                if let Some(v) = sail_lint::tokens::token_value(name) {
                    m.insert((*name).to_string(), v.to_string());
                }
            }
            // Dimensions + type tokens: radius, space, control heights, shadows,
            // font/text/tracking/leading (text channel needs the full set).
            for prefix in [
                "radius-",
                "space-",
                "control-h-",
                "shadow-",
                "font-",
                "text-",
                "tracking-",
                "leading-",
                "opacity-",
                "border-width",
            ] {
                // suffixes for scales; also try bare names.
                if prefix.ends_with('-') {
                    for s in sail_lint::tokens::suffixes(prefix) {
                        let name = format!("{prefix}{s}");
                        if let Some(v) = sail_lint::tokens::token_value(&name) {
                            m.insert(name, v.to_string());
                        }
                    }
                } else if let Some(v) = sail_lint::tokens::token_value(prefix) {
                    m.insert(prefix.to_string(), v.to_string());
                }
            }
            // Font family aliases used by `fontFamily(body|display|mono)`.
            for name in [
                "body-font",
                "display-font",
                "mono-font",
                "heading-font",
                "label-font",
                "font-body",
                "font-body-app",
                "font-display",
                "font-display-app",
                "font-mono",
                "font-mono-app",
                "font-press-start",
                "font-press-start-app",
            ] {
                if let Some(v) = sail_lint::tokens::token_value(name) {
                    m.entry(name.to_string()).or_insert_with(|| v.to_string());
                }
            }
            // Ensure critical surface tokens even if not in color_values (aliases).
            for name in [
                "background",
                "foreground",
                "primary",
                "primary-foreground",
                "surface-0",
                "surface-1",
                "surface-2",
                "surface-3",
                "border",
                "border-subtle",
                "border-strong",
                "muted",
                "muted-foreground",
                "card",
                "card-foreground",
                "accent",
                "accent-foreground",
                "navbar",
                "navbar-foreground",
                "sidebar",
                "sidebar-foreground",
                "sidebar-border",
                "sidebar-accent",
                "sidebar-accent-foreground",
                "switch",
                "switch-border",
                "switch-thumb",
                "input",
                "ring",
            ] {
                if let Some(v) = sail_lint::tokens::token_value(name) {
                    m.entry(name.to_string()).or_insert_with(|| v.to_string());
                }
            }
            m
        })
        .clone()
}

fn find_theme_file(theme_id: &str, workspace: Option<&Path>) -> Option<PathBuf> {
    let roots: Vec<PathBuf> = workspace.map(|w| vec![w.to_path_buf()]).unwrap_or_else(|| {
        let mut v = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            v.push(cwd);
        }
        v
    });
    for root in roots {
        // Project themes: modern `<root>/themes` then legacy `tokens/themes`.
        for sub in ["themes", "tokens/themes"] {
            let p = root.join(sub).join(format!("{theme_id}.json"));
            if p.is_file() {
                return Some(p);
            }
        }
        // Catalog model (monorepo disk) — not a project theme path.
        for rel in [
            format!("packages/sail/tokens/model/themes/{theme_id}.json"),
            format!("packages/sail/tokens/model/themes/{theme_id}-light.json"),
        ] {
            let c = root.join(&rel);
            if c.is_file() {
                return Some(c);
            }
        }
        // Monorepo apps: apps/*/{themes,tokens/themes}/{id}.json
        let apps = root.join("apps");
        if let Ok(entries) = std::fs::read_dir(&apps) {
            for e in entries.flatten() {
                let app = e.path();
                for sub in ["themes", "tokens/themes"] {
                    let p = app.join(sub).join(format!("{theme_id}.json"));
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
        // Walk up for catalog model only (no named-app hardcode).
        let mut dir = root;
        for _ in 0..6 {
            let c = dir.join(format!("packages/sail/tokens/model/themes/{theme_id}.json"));
            if c.is_file() {
                return Some(c);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    None
}

fn overlay_theme_mode(
    values: &mut HashMap<String, String>,
    theme_path: &Path,
    mode: &str,
) -> Result<()> {
    let text = std::fs::read_to_string(theme_path)
        .with_context(|| format!("read {}", theme_path.display()))?;
    let root: Value = serde_json::from_str(&text).context("parse theme json")?;

    // modes-in-file: modes.light / modes.dark
    let mode_node = root
        .pointer(&format!("/modes/{mode}"))
        .or_else(|| root.pointer("/modes/light"))
        .or_else(|| {
            root.get("modes")
                .and_then(|m| m.as_object())
                .and_then(|o| o.values().next())
        });

    if let Some(node) = mode_node {
        walk_dtcg(node, "", values);
        return Ok(());
    }

    // Flat theme file (single mode): walk whole tree excluding $extensions.
    walk_dtcg(&root, "", values);
    Ok(())
}

fn walk_dtcg(node: &Value, path: &str, values: &mut HashMap<String, String>) {
    let Some(obj) = node.as_object() else {
        return;
    };
    if let Some(val) = obj.get("$value") {
        let css = path_to_css(path);
        if !css.is_empty() {
            let s = value_to_string(val);
            if !s.is_empty() {
                values.insert(css, s);
            }
        }
        return;
    }
    for (k, v) in obj {
        if k.starts_with('$') {
            continue;
        }
        let child = if path.is_empty() {
            k.clone()
        } else {
            format!("{path}.{k}")
        };
        walk_dtcg(v, &child, values);
    }
}

fn path_to_css(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    // Prefer registry path map (color.primary.default → primary).
    if let Some(css) = sail_lint::tokens::path_to_css_name(path) {
        return css.to_string();
    }
    // Fallback: kebab of leaf segments, drop common group prefixes.
    let parts: Vec<&str> = path.split('.').collect();
    // color.primary.default → primary; color.surface.0 → surface-0
    let skip = matches!(
        parts.first().copied(),
        Some("color" | "component" | "interaction")
    );
    let start = if skip { 1 } else { 0 };
    let mut segs: Vec<&str> = parts[start..].to_vec();
    // Drop trailing `.default`
    if segs.last() == Some(&"default") {
        segs.pop();
    }
    // component.card.bg → card (and card-foreground for fg)
    if parts.first() == Some(&"component") && segs.len() >= 2 {
        let host = segs[0];
        let prop = segs[1];
        return match prop {
            "bg" => host.to_string(),
            "fg" => format!("{host}-foreground"),
            _ => format!("{host}-{prop}"),
        };
    }
    segs.join("-")
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

fn expand_vars_once(values: &mut HashMap<String, String>) -> bool {
    let snapshot = values.clone();
    let mut changed = false;
    for v in values.values_mut() {
        let next = resolve_value_against(v, &snapshot, 0);
        if next != *v {
            *v = next;
            changed = true;
        }
    }
    changed
}

fn resolve_value_against(value: &str, map: &HashMap<String, String>, depth: u8) -> String {
    if depth > 12 {
        return value.to_string();
    }
    let mut out = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"var(") {
            let start = i + 4;
            let mut depth_p = 1i32;
            let mut j = start;
            while j < bytes.len() {
                match bytes[j] {
                    b'(' => depth_p += 1,
                    b')' => {
                        depth_p -= 1;
                        if depth_p == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if depth_p == 0 {
                let inner = value[start..j].trim();
                // var(--name, fallback)
                let (name_part, fallback) = match inner.split_once(',') {
                    Some((n, f)) => (n.trim(), Some(f.trim())),
                    None => (inner, None),
                };
                let name = name_part.strip_prefix("--").unwrap_or(name_part);
                if let Some(resolved) = map.get(name) {
                    out.push_str(&resolve_value_against(resolved, map, depth + 1));
                } else if let Some(fb) = fallback {
                    out.push_str(&resolve_value_against(fb, map, depth + 1));
                } else {
                    // Leave unresolved for honest diagnostics.
                    out.push_str(&value[i..=j]);
                }
                i = j + 1;
                continue;
            }
        }
        // UTF-8 safe: continuation bytes ride with their lead byte.
        let len = match bytes[i] {
            0x00..=0x7F => 1,
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            _ => 4,
        };
        out.push_str(&value[i..i + len]);
        i += len;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_surface_and_primary() {
        let p = ThemePalette::resolve(ThemeSelect::default(), None).unwrap();
        assert!(p.get("surface-0").is_some() || p.get("background").is_some());
        assert!(p.get("primary").is_some(), "primary missing");
        assert!(p.get("radius-md").is_some() || p.get("radius-sm").is_some());
    }

    #[test]
    fn parse_theme_mode_from_id() {
        let t = ThemeSelect::parse(Some("ship-dark"), None);
        assert_eq!(t.id, "ship");
        assert_eq!(t.mode, "dark");
    }

    #[test]
    fn ship_dark_differs_from_light() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .ok();
        assert!(root.is_some(), "workspace root required for theme files");
        let light = ThemePalette::resolve(
            ThemeSelect {
                id: "ship".into(),
                mode: "light".into(),
            },
            root.as_deref(),
        )
        .unwrap();
        let dark = ThemePalette::resolve(
            ThemeSelect {
                id: "ship".into(),
                mode: "dark".into(),
            },
            root.as_deref(),
        )
        .unwrap();
        let lb = light.get("background").expect("light background token");
        let db = dark.get("background").expect("dark background token");
        assert_ne!(
            lb, db,
            "ship light/dark background must differ (theme modes overlay)"
        );
        let lf = light.get("foreground").expect("light foreground");
        let df = dark.get("foreground").expect("dark foreground");
        assert_ne!(lf, df, "ship light/dark foreground must differ");
    }
}
