//! The shared design-token registry — the **single source** both the
//! `sail-css-token` lint and the `sx` vocabulary read from.
//!
//! There is exactly one source of truth for design-token *definitions*: the
//! grouped DTCG model at `packages/sail/tokens/model/` (plus
//! `model/themes/*.json`). A generate step emits:
//!   - `packages/sail/tokens/src/tokens.css` (web delivery)
//!   - `packages/sail/tokens/src/themes.css` (theme overrides)
//!   - `packages/sail/tokens/src/token-registry.json` (this registry)
//!
//! Lint reads the **generated registry**, not a CSS scrape. Hand-edit of
//! generated CSS is not SSOT; edit the model and re-run
//! `cargo run -p sail-compiler -- tokens` (or `pnpm --filter @ship/tokens
//! generate`).
//!
//! Token **css names** are stored WITHOUT the `--` prefix (e.g. `"space-4"`,
//! `"text-3xl"`). Grouped DTCG paths (e.g. `space.4`) are recorded on each
//! registry entry as `path` for tooling / J2. `sx`'s allowed spacing/text/radius
//! values are *derived* from this registry (filtered by css-name prefix), so
//! adding a token to the DTCG model (and regenerating) makes `style.fontSize(3xl)`
//! legal automatically.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Deserialize;

/// Generated registry artifact (DTCG emit). Path is relative to this crate.
/// Theme env / complete / check all read this same include.
pub const TOKEN_REGISTRY_JSON: &str =
    include_str!("../../../../packages/sail/tokens/src/token-registry.json");

/// Consumers accept this schema only. No v1 dual-read.
pub const REGISTRY_SCHEMA_V2: &str = "spinnaker.registry.v2";

#[derive(Debug, Deserialize, Clone)]
struct RegistryFile {
    /// `spinnaker.registry.v2` — required. Absent / other = not this artifact.
    #[serde(default)]
    schema: String,
    /// Public CSS custom-property names (no `--` prefix). Includes alias
    /// names (empty after TOKENS-1 app-accent-* scrub).
    names: Vec<String>,
    colors: Vec<String>,
    /// Optional full map — present for value-aware tests / tooling.
    /// Keyed by cssName.
    #[serde(default)]
    tokens: HashMap<String, TokenMeta>,
    /// path → cssName (canonical tokens only). Present from registry v2.
    #[serde(default)]
    paths: HashMap<String, String>,
    /// DTCG composite identities (`border.subtle`, `bg.surface`, …).
    #[serde(default)]
    composites: HashMap<String, CompositeMeta>,
}

#[derive(Debug, Deserialize, Clone)]
struct CompositeMeta {
    #[serde(default)]
    path: String,
    #[serde(default, rename = "type")]
    r#type: String,
    #[serde(default)]
    members: HashMap<String, String>,
}

/// One DTCG composite identity from the generated registry.
#[derive(Debug, Clone)]
pub struct CompositeToken {
    /// Public identity (`border.subtle`, `dialog.border`).
    pub name: &'static str,
    /// DTCG `$type` (`border`, `size`, `elevation`, `type`).
    pub r#type: &'static str,
    /// Storage path (`component.dialog.border` or `border.subtle`).
    pub path: &'static str,
    /// Slot → CSS custom-property name (no `--`).
    pub members: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Deserialize, Clone)]
struct TokenMeta {
    /// DTCG `$type` (color, dimension, …). Retained for tooling; name
    /// membership and the `colors` array are what lint scales use today.
    #[serde(default)]
    #[allow(dead_code)]
    r#type: String,
    #[serde(default)]
    value: String,
    /// Grouped DTCG path (e.g. `space.4`, `color.primary.default`).
    #[serde(default)]
    #[allow(dead_code)]
    path: String,
    /// Public CSS name without `--` (same as the map key for canonical entries).
    #[serde(default, rename = "cssName")]
    #[allow(dead_code)]
    css_name: String,
    /// When set, this entry is an `aliasOf` of another cssName.
    #[serde(default, rename = "aliasOf")]
    #[allow(dead_code)]
    alias_of: String,
}

/// Contextual custom properties that are legitimate in component `var(--x)`
/// references but are **not** global design tokens, so they carry no value in
/// the DTCG base layer and are not in the generated registry. They are set
/// per-context (at runtime, by a parent primitive, or inside a component) —
/// a curated lint allowlist, not the token system. Keep this list tight;
/// anything with a global value belongs in the DTCG model, not here.
///
/// Public so the LSP shares the exact same list (it completes these too) —
/// one definition, no cross-crate drift.
pub const CONVENTION_TOKENS: &[&str] = &[
    // tone-* channel names removed (no contracts attr channel). Semantic
    // ramp tokens (hover-bg, …) live in the registry, not here.
    // Floating-primitive positioning channels (dropdown / hovercard / popover / tooltip).
    "sail-dd-anchor",
    "sail-dd-default",
    "sail-hc-anchor",
    "sail-hc-default",
    "sail-popover-anchor",
    "sail-popover-default",
    "sail-tooltip-anchor",
    "sail-tooltip-default",
    // App-shell layout channels (rail + top bar).
    "rail-item-justify",
    "rail-label-display",
    "ship-rail-w",
    "ship-rail-w-collapsed",
    "ship-top-bar-h",
    // Misc component-local channels.
    "card-shadow-style",
    "font-sans",
    "toast-duration",
    "z-overlay",
    // Runtime-geometry channels that actions write used to live here so
    // bind/var resolution treated them like tokens. STYLE-BINDING-1 moves
    // them to action_manifest `writes` — `bind(name)` resolves against the
    // action contract, not this allowlist. Keep only non-action convention
    // channels that still need var(--x) legitimacy in author CSS.
];

thread_local! {
    static REGISTRY_OVERRIDE: RefCell<Option<RegistryFile>> = const { RefCell::new(None) };
}

fn parse_registry_v2(json: &str) -> RegistryFile {
    let file: RegistryFile = serde_json::from_str(json)
        .unwrap_or_else(|e| panic!("token-registry.json parse failed: {e}"));
    assert_eq!(
        file.schema, REGISTRY_SCHEMA_V2,
        "consumers read {REGISTRY_SCHEMA_V2} only (got {:?})",
        file.schema
    );
    file
}

fn registry_file() -> &'static RegistryFile {
    static CACHE: OnceLock<RegistryFile> = OnceLock::new();
    CACHE.get_or_init(|| parse_registry_v2(TOKEN_REGISTRY_JSON))
}

/// Run `f` against a registry JSON instead of the baked artifact.
/// Complete and check both read [`color_values`] / [`registry`] from this file.
pub fn with_registry_json<T>(json: &str, f: impl FnOnce() -> T) -> T {
    let parsed = parse_registry_v2(json);
    REGISTRY_OVERRIDE.with(|cell| {
        let prev = cell.replace(Some(parsed));
        let out = f();
        *cell.borrow_mut() = prev;
        out
    })
}

/// Clone the baked v2 registry, add a color name, return JSON.
/// Used by the complete∩check one-artifact test — not a product token.
pub fn inject_registry_color(json: &str, name: &str) -> String {
    use serde_json::{Value, json};
    let mut root: Value =
        serde_json::from_str(json).unwrap_or_else(|e| panic!("inject parse: {e}"));
    assert_eq!(
        root.get("schema").and_then(Value::as_str),
        Some(REGISTRY_SCHEMA_V2),
        "inject requires {REGISTRY_SCHEMA_V2}"
    );
    if let Some(Value::Array(names)) = root.get_mut("names") {
        names.push(Value::String(name.into()));
    }
    if let Some(Value::Array(colors)) = root.get_mut("colors") {
        colors.push(Value::String(name.into()));
    }
    if let Some(Value::Object(tokens)) = root.get_mut("tokens") {
        tokens.insert(
            name.into(),
            json!({
                "type": "color",
                "value": "oklch(0.62 0.18 320)",
                "path": format!("color.{name}.solid"),
                "cssName": name,
            }),
        );
    }
    root.to_string()
}

fn collect_color_names(file: &RegistryFile) -> Vec<&'static str> {
    let mut colors: HashSet<&'static str> = HashSet::new();
    for name in &file.colors {
        if !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            colors.insert(Box::leak(name.clone().into_boxed_str()));
        }
    }
    for kw in crate::scales::KEYWORD_COLORS {
        colors.insert(kw);
    }
    let mut out: Vec<&'static str> = colors.into_iter().collect();
    out.sort_unstable();
    out
}

fn collect_registry_names(file: &RegistryFile) -> HashSet<&'static str> {
    let mut out: HashSet<&'static str> = CONVENTION_TOKENS.iter().copied().collect();
    for name in &file.names {
        if !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            out.insert(Box::leak(name.clone().into_boxed_str()));
        }
    }
    out
}

/// Every valid design-token name (no `--` prefix): names from the generated
/// registry, plus the [`CONVENTION_TOKENS`] allowlist. Loaded once.
pub fn registry() -> &'static HashSet<&'static str> {
    REGISTRY_OVERRIDE.with(|cell| {
        if let Some(file) = cell.borrow().as_ref() {
            return &*Box::leak(Box::new(collect_registry_names(file)));
        }
        static CACHE: OnceLock<HashSet<&'static str>> = OnceLock::new();
        CACHE.get_or_init(|| collect_registry_names(registry_file()))
    })
}

/// The suffix values of a scale family, e.g. `suffixes("space-")` →
/// `["1",…,"8"]`, `suffixes("text-")` → `["2xl","3xl","4xl","base",…]`.
/// Sorted for deterministic diagnostics. Derived from [`registry`], so the
/// scale can never drift from the tokens.
pub fn suffixes(prefix: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = registry()
        .iter()
        .filter_map(|&t| t.strip_prefix(prefix).filter(|s| !s.is_empty()))
        .collect();
    out.sort_unstable();
    out
}

/// Allowed `sx` spacing values: the `--space-N` scale plus the literal `0`
/// (no `--space-0` token exists; `p:0` must mean zero padding, not a var).
pub fn space_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v = vec!["0"];
        v.extend(suffixes("space-"));
        v
    })
}

/// Allowed `sx` text values: the full `--text-*` scale (xs … 4xl, base).
pub fn text_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| suffixes("text-"))
}

/// Every token whose VALUE is a color — the surface `sx`'s `bg`/`fg`/`border`
/// draw from. Taken from the generated registry's `colors` list (DTCG `$type:
/// color` + alias resolution at generate time). Derived, so a new tone channel
/// or surface is usable in sx the moment it lands in the model and is
/// regenerated — no hand-maintained color list to drift.
pub fn color_values() -> &'static [&'static str] {
    REGISTRY_OVERRIDE.with(|cell| {
        if let Some(file) = cell.borrow().as_ref() {
            return &*Box::leak(collect_color_names(file).into_boxed_slice());
        }
        static C: OnceLock<Vec<&'static str>> = OnceLock::new();
        C.get_or_init(|| collect_color_names(registry_file()))
            .as_slice()
    })
}

/// Component color leaves: `(storage path, public cssName)`.
/// Used to build L1 surface graphs and exclude surface ids from app paint.
pub fn component_color_paths() -> &'static [(String, String)] {
    static C: OnceLock<Vec<(String, String)>> = OnceLock::new();
    C.get_or_init(|| {
        let file = registry_file();
        let color_set: HashSet<&str> = file.colors.iter().map(|s| s.as_str()).collect();
        let mut out: Vec<(String, String)> = Vec::new();
        for (path, css) in &file.paths {
            if path.starts_with("component.") && color_set.contains(css.as_str()) {
                out.push((path.clone(), css.clone()));
            }
        }
        // Also pull from tokens meta when paths map is incomplete.
        for (css_name, meta) in &file.tokens {
            if meta.path.starts_with("component.")
                && color_set.contains(css_name.as_str())
                && !out.iter().any(|(p, _)| p == &meta.path)
            {
                out.push((meta.path.clone(), css_name.clone()));
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    })
}

/// Dimensional size tokens the sx `w`/`h`/`min-*`/`max-*` props draw from — the
/// icon, control-height, and panel-clamp families that the primitives now depend
/// on. The value IS the token name (`max-h:panel-max-h-md` → `var(--panel-max-h-md)`),
/// so a new size family is usable in sx the moment it lands in the model.
pub fn size_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v: Vec<&'static str> = registry()
            .iter()
            .copied()
            .filter(|t| {
                // Only the DIMENSIONAL panel subfamilies join the sizing
                // vocabulary. The panel-pad-* component tokens are padding
                // knobs consumed by skins, not sx sizes — a broad `panel-`
                // prefix would leak them into `w:`/`h:`.
                t.starts_with("icon-size-")
                    || t.starts_with("control-h-")
                    || t.starts_with("breakpoint-")
                    || t.starts_with("panel-max-")
                    || t.starts_with("panel-min-")
                    || t.starts_with("panel-ch-")
                    || t.starts_with("panel-vh-")
                    || t.starts_with("panel-w-")
            })
            .collect();
        v.sort_unstable();
        v
    })
}

/// Allowed `sx` radius values: the `--radius-*` scale plus the literal `none`
/// (→ `0`). No phantom `xs` — the scale is whatever the tokens declare.
pub fn radius_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v = vec!["none"];
        v.extend(suffixes("radius-"));
        v
    })
}

fn composite_index() -> &'static HashMap<&'static str, CompositeToken> {
    static CACHE: OnceLock<HashMap<&'static str, CompositeToken>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let file = registry_file();
        let mut out = HashMap::new();
        for (name, meta) in &file.composites {
            if name.is_empty() {
                continue;
            }
            let members: Vec<(&'static str, &'static str)> = meta
                .members
                .iter()
                .map(|(slot, css)| {
                    (
                        Box::leak(slot.clone().into_boxed_str()) as &'static str,
                        Box::leak(css.clone().into_boxed_str()) as &'static str,
                    )
                })
                .collect();
            let token = CompositeToken {
                name: Box::leak(name.clone().into_boxed_str()),
                r#type: Box::leak(meta.r#type.clone().into_boxed_str()),
                path: Box::leak(meta.path.clone().into_boxed_str()),
                members: Box::leak(members.into_boxed_slice()),
            };
            out.insert(token.name, token);
        }
        out
    })
}

/// Composite identity by public name (`border.subtle`, `card.border`).
pub fn find_composite(name: &str) -> Option<&'static CompositeToken> {
    composite_index().get(name)
}

/// Every composite whose `$type` is `ty`, sorted by public name.
pub fn composites_of_type(ty: &str) -> Vec<&'static CompositeToken> {
    let mut out: Vec<&'static CompositeToken> = composite_index()
        .values()
        .filter(|c| c.r#type == ty)
        .collect();
    out.sort_by_key(|c| c.name);
    out
}

/// Every composite identity, sorted by public name.
pub fn all_composites() -> Vec<&'static CompositeToken> {
    let mut out: Vec<&'static CompositeToken> = composite_index().values().collect();
    out.sort_by_key(|c| c.name);
    out
}

/// Composite whose member emit label is `css` (`dialog-border`, `border-subtle-color`).
pub fn composite_for_member_css(css: &str) -> Option<&'static CompositeToken> {
    composite_index()
        .values()
        .find(|c| c.members.iter().any(|(_, name)| *name == css))
}

/// Resolve a DTCG path (`space.4`, `color.primary.default`) to the public
/// cssName (`space-4`, `primary`). Registry v2 `paths` map; falls back to
/// kebab of the path when the map is absent (flat registries).
pub fn path_to_css_name(path: &str) -> Option<&'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let file = registry_file();
        let mut m = HashMap::new();
        for (path, css) in &file.paths {
            let k: &'static str = Box::leak(path.clone().into_boxed_str());
            let v: &'static str = Box::leak(css.clone().into_boxed_str());
            m.insert(k, v);
        }
        // Also index TokenMeta.path when present.
        for (css_name, meta) in &file.tokens {
            if !meta.path.is_empty() {
                let k: &'static str = Box::leak(meta.path.clone().into_boxed_str());
                let v: &'static str = Box::leak(css_name.clone().into_boxed_str());
                m.entry(k).or_insert(v);
            }
        }
        m
    });
    map.get(path).copied()
}

thread_local! {
    static BREAKPOINT_OVERRIDE: RefCell<Option<Vec<(String, String)>>> =
        const { RefCell::new(None) };
}

/// Active theme breakpoint table: `(name, width)` in declaration order.
///
/// Default is the registry `breakpoint-*` tokens. Tests and a 1-bp compile
/// install a replacement via [`with_breakpoint_table`].
pub fn breakpoint_table() -> Vec<(String, String)> {
    if let Some(over) = BREAKPOINT_OVERRIDE.with(|c| c.borrow().clone()) {
        return over;
    }
    static CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    CACHE.get_or_init(registry_breakpoint_table).clone()
}

fn registry_breakpoint_table() -> Vec<(String, String)> {
    let file = registry_file();
    let mut rows: Vec<(String, String)> = file
        .tokens
        .iter()
        .filter_map(|(css, meta)| {
            let name = css.strip_prefix("breakpoint-")?;
            if name.is_empty() {
                return None;
            }
            Some((name.to_string(), meta.value.clone()))
        })
        .collect();
    rows.sort_by(|a, b| {
        width_px(&a.1)
            .cmp(&width_px(&b.1))
            .then_with(|| a.0.cmp(&b.0))
    });
    rows
}

fn width_px(value: &str) -> u32 {
    value
        .trim()
        .trim_end_matches("px")
        .parse::<f32>()
        .ok()
        .map(|n| n as u32)
        .unwrap_or(u32::MAX)
}

/// Run `f` against an explicit theme breakpoint table.
pub fn with_breakpoint_table<T>(table: &[(&str, &str)], f: impl FnOnce() -> T) -> T {
    let owned: Vec<(String, String)> = table
        .iter()
        .map(|(n, w)| ((*n).to_string(), (*w).to_string()))
        .collect();
    BREAKPOINT_OVERRIDE.with(|cell| {
        let prev = cell.replace(Some(owned));
        let out = f();
        *cell.borrow_mut() = prev;
        out
    })
}

/// True when `name` is a breakpoint the active theme declared.
pub fn is_theme_breakpoint(name: &str) -> bool {
    let kebab = crate::intent::when_key_kebab(name);
    breakpoint_table()
        .iter()
        .any(|(n, _)| n == name || n == &kebab)
}

/// Width for a theme-declared breakpoint (`"768px"`), if present.
pub fn breakpoint_width(name: &str) -> Option<String> {
    let kebab = crate::intent::when_key_kebab(name);
    breakpoint_table()
        .into_iter()
        .find(|(n, _)| n == name || n == &kebab)
        .map(|(_, w)| w)
}

/// Human list for diagnostics: `mobile, tablet, desktop, wide`.
pub fn breakpoint_names_display() -> String {
    let names: Vec<String> = breakpoint_table().into_iter().map(|(n, _)| n).collect();
    if names.is_empty() {
        "(none)".into()
    } else {
        names.join(", ")
    }
}

/// Look up a token's generated CSS value (base layer). Used by tests that pin
/// scale alignment (e.g. Tailwind space ramp). Returns `None` if unknown.
pub fn token_value(name: &str) -> Option<&'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let file = registry_file();
        let mut m = HashMap::new();
        for (k, meta) in &file.tokens {
            let key: &'static str = Box::leak(k.clone().into_boxed_str());
            let val: &'static str = Box::leak(meta.value.clone().into_boxed_str());
            m.insert(key, val);
        }
        m
    });
    map.get(name).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composites_include_border_identities() {
        let names: Vec<&str> = all_composites().iter().map(|c| c.name).collect();
        for want in ["border.subtle", "border.default", "border.strong"] {
            assert!(names.contains(&want), "missing `{want}`: {names:?}");
        }
        let subtle = find_composite("border.subtle").expect("border.subtle");
        assert_eq!(subtle.r#type, "border");
        assert!(
            subtle.members.iter().any(|(s, _)| *s == "color"),
            "{:?}",
            subtle.members
        );
        assert_eq!(
            composite_for_member_css("border-subtle-color").map(|c| c.name),
            Some("border.subtle")
        );
    }

    #[test]
    fn text_scale_includes_the_full_registry_range() {
        let t = text_values();
        // the exact values that sx used to be missing — proof of no-drift.
        for want in ["xs", "sm", "md", "base", "lg", "xl", "2xl", "3xl", "4xl"] {
            assert!(t.contains(&want), "text scale missing `{want}`: {t:?}");
        }
    }

    #[test]
    fn space_has_zero_plus_scale_no_phantom() {
        let s = space_values();
        assert!(s.contains(&"0"), "p:0 must be allowed (literal zero)");
        for n in ["1", "4", "8"] {
            assert!(s.contains(&n), "space missing `{n}`");
        }
        // The ramp above 8 was added in job-E-space-ramp: an agent reaching
        // for a 48px thumb or a 180px canvas can now SPEND a token (and
        // sx accepts those names because the lint reads the registry).
        for n in ["12", "16", "24", "32", "48", "64"] {
            assert!(
                s.contains(&n),
                "space missing `{n}` — the Tailwind-aligned upper ramp didn't reach the registry"
            );
        }
        // The alignment: 7 and 8 used to be silent values (32, 40); the
        // agent's prior was Tailwind's (28, 32). The space scale's emitted
        // CSS values now match, by hand (see `space_lower_aligns_with_tailwind`).
        assert_eq!(
            crate::scales::lower("space", "7").as_deref(),
            Some("var(--space-7)"),
        );
        assert_eq!(
            crate::scales::lower("space", "8").as_deref(),
            Some("var(--space-8)"),
        );
    }

    #[test]
    fn space_lower_aligns_with_tailwind() {
        // Values come from the generated registry (DTCG model), not a CSS scrape.
        for (name, want_px) in [("7", "28px"), ("8", "32px")] {
            let got = token_value(&format!("space-{name}")).unwrap_or("");
            assert_eq!(got, want_px, "--space-{name} drifted from Tailwind");
        }
        // The full Tailwind-aligned space ramp.
        for (n, want_px) in [
            ("1", "4px"),
            ("4", "16px"),
            ("6", "24px"),
            ("7", "28px"),
            ("8", "32px"),
            ("10", "40px"),
            ("12", "48px"),
            ("16", "64px"),
            ("24", "96px"),
            ("48", "192px"),
            ("64", "256px"),
        ] {
            let got = token_value(&format!("space-{n}")).unwrap_or("");
            assert_eq!(
                got, want_px,
                "--space-{n} drifted from Tailwind ({want_px})"
            );
        }
    }

    #[test]
    fn radius_has_none_plus_scale_without_xs() {
        let r = radius_values();
        assert!(r.contains(&"none"));
        assert!(r.contains(&"full") && r.contains(&"lg"));
        assert!(
            !r.contains(&"xs"),
            "radius `xs` was a phantom — not in token registry"
        );
    }

    #[test]
    fn color_values_derive_the_full_surface_not_a_curated_list() {
        let c = color_values();
        // base palette + tonal channels + surfaces — all from registry colors
        for want in [
            "primary",
            "foreground",
            "success",
            "surface",
            "muted-foreground",
            "hover-bg",
            "danger",
            "accent-foreground",
        ] {
            assert!(c.contains(&want), "color surface missing `{want}`");
        }
        // non-colors must NOT leak into the color surface
        assert!(!c.contains(&"space-4") && !c.contains(&"text-2xl") && !c.contains(&"radius-lg"));
        assert!(
            c.len() > 40,
            "expected the real ~85-token surface, got {}",
            c.len()
        );
    }

    #[test]
    fn parses_global_tokens_and_keeps_conventions() {
        let reg = registry();
        // from generated registry
        assert!(reg.contains("primary"), "brand token from registry");
        assert!(reg.contains("surface"), "semantic surface from registry");
        assert_eq!(registry_file().schema, REGISTRY_SCHEMA_V2);
        assert!(reg.contains("space-4") && reg.contains("text-2xl"));
        // curated convention allowlist (no global value, must still be accepted)
        assert!(reg.contains("sail-tooltip-anchor"));
        // a var() *usage* inside a value must never be mistaken for a declaration
        assert!(
            !reg.contains("foreground)"),
            "var() usages must not leak in"
        );
    }

    #[test]
    fn registry_json_is_non_empty_and_matches_name_set() {
        let file = registry_file();
        assert!(
            file.names.len() > 100,
            "expected full token set, got {}",
            file.names.len()
        );
        assert!(!file.colors.is_empty());
        let reg = registry();
        for name in &file.names {
            assert!(reg.contains(name.as_str()), "registry missing {name}");
        }
    }

    #[test]
    fn registry_entries_carry_dtcg_paths() {
        let file = registry_file();
        // Canonical space/primary tokens must expose path + cssName for J2.
        let space = file.tokens.get("space-4").expect("space-4 in tokens map");
        assert_eq!(space.path, "space.4", "space-4 path");
        assert!(
            space.css_name.is_empty() || space.css_name == "space-4",
            "cssName should be space-4 or omitted, got {}",
            space.css_name
        );
        let primary = file.tokens.get("primary").expect("primary in tokens map");
        assert!(
            primary.path.starts_with("color."),
            "primary should live under color.* path, got {}",
            primary.path
        );
        // path index (v2) maps path → cssName
        if !file.paths.is_empty() {
            assert_eq!(
                file.paths.get("space.4").map(String::as_str),
                Some("space-4")
            );
        }
    }
}
