//! StyleSurface — the Style SSOT (SPEC-STYLE-ARCH-1 §2.5).
//!
//! Complete, validate, generate, LSP, MCP `sail_surface`, and the authoring
//! skill read this object. Privilege filtering lands in M1
//! (`complete ⊆ surface ∩ privilege`); M0 is the table plus print.

use serde::Serialize;

use crate::tokens;

/// Which form a file may write. Path + file kind resolve this in M1.
/// Config cannot grant Full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Privilege {
    Full,
    Identities,
    None,
}

/// Where a preset comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PresetOrigin {
    /// Compiler IR. Zero atoms. Not DTCG.
    Language,
    /// Floor key every theme must implement. Generate warns in M0, fails in M3.
    ThemeRequired,
    /// Theme/app add, or a migrate-window alias that will die after grep-zero.
    ThemeOptional,
}

/// One concern argument: `type(heading)`, `size(grow)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StylePreset {
    pub concern: &'static str,
    pub name: &'static str,
    pub origin: PresetOrigin,
    pub meaning: &'static str,
}

impl StylePreset {
    pub fn key(self) -> String {
        format!("{}.{}", self.concern, self.name)
    }
}

/// Primitive token the theme owns (paint, type, space, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StyleAtom {
    pub path: String,
    pub scale: String,
}

/// The composed Style table. Nobody authors this — it is derived.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StyleSurface {
    pub atoms: Vec<StyleAtom>,
    pub presets: Vec<StylePreset>,
    /// Floor keys (`type.heading`). Language presets are not in this list.
    pub required: Vec<&'static str>,
    pub privilege: Privilege,
}

/// Language presets. Compiler IR, never DTCG.
pub const LANGUAGE_PRESETS: &[StylePreset] = &[
    StylePreset {
        concern: "size",
        name: "hug",
        origin: PresetOrigin::Language,
        meaning: "intrinsic. flex 0 0 auto, min auto",
    },
    StylePreset {
        concern: "size",
        name: "grow",
        origin: PresetOrigin::Language,
        meaning: "share leftover. flex 1 1 0, min 0, never width 100%",
    },
    StylePreset {
        concern: "size",
        name: "max",
        origin: PresetOrigin::Language,
        meaning: "be the containing block. stretch + min 0, never width 100% in flex",
    },
    StylePreset {
        concern: "overflow",
        name: "hidden",
        origin: PresetOrigin::Language,
        meaning: "clip overflow",
    },
    StylePreset {
        concern: "overflow",
        name: "auto",
        origin: PresetOrigin::Language,
        meaning: "scroll if needed",
    },
    StylePreset {
        concern: "overflow",
        name: "clip",
        origin: PresetOrigin::Language,
        meaning: "clip without a scrollport",
    },
    StylePreset {
        concern: "overflow",
        name: "scroll",
        origin: PresetOrigin::Language,
        meaning: "always a scrollport",
    },
    StylePreset {
        concern: "overflow",
        name: "visible",
        origin: PresetOrigin::Language,
        meaning: "do not clip",
    },
    StylePreset {
        concern: "position",
        name: "relative",
        origin: PresetOrigin::Language,
        meaning: "positioning containing block",
    },
    StylePreset {
        concern: "position",
        name: "absolute",
        origin: PresetOrigin::Language,
        meaning: "out of flow, inset against containing block",
    },
    StylePreset {
        concern: "position",
        name: "sticky",
        origin: PresetOrigin::Language,
        meaning: "stick within scroller",
    },
    StylePreset {
        concern: "position",
        name: "fixed",
        origin: PresetOrigin::Language,
        meaning: "viewport-fixed",
    },
    StylePreset {
        concern: "position",
        name: "static",
        origin: PresetOrigin::Language,
        meaning: "normal flow",
    },
];

/// Required theme presets. Compiler owns the keys. Theme supplies atom values.
pub const REQUIRED_THEME_PRESETS: &[StylePreset] = &[
    // type — display-sm is the floor name (registry already has it; do not dual-name).
    req("type", "body", "body typeface program"),
    req("type", "body-sm", "small body"),
    req(
        "type",
        "body-semibold",
        "semibold body — emphasis without label styling",
    ),
    req(
        "type",
        "heading",
        "heading typeface program — tags <h1>–<h6> spend this",
    ),
    req("type", "label", "eyebrow: xs / semibold / wide / uppercase"),
    req("type", "mono", "monospace program"),
    req("type", "mono-sm", "small mono"),
    req("type", "display-sm", "display typeface program"),
    // size
    req("size", "control-2xs", "control height recipe"),
    req("size", "control-xs", "control height recipe"),
    req("size", "control-sm", "control height recipe"),
    req("size", "control-md", "control height recipe"),
    req("size", "control-lg", "control height recipe"),
    req("size", "control-xl", "control height recipe"),
    req("pad", "page", "document inset"),
    req("size", "icon-3xs", "icon box"),
    req("size", "icon-2xs", "icon box"),
    req("size", "icon-xs", "icon box"),
    req("size", "icon-sm", "icon box"),
    req("size", "icon-md", "icon box"),
    req("size", "icon-lg", "icon box"),
    req("size", "icon-xl", "icon box"),
    req("size", "icon-2xl", "icon box"),
    req("size", "icon-3xl", "icon box"),
    // border
    req("border", "default", "default stroke"),
    req("border", "subtle", "quiet stroke"),
    req("border", "strong", "emphatic stroke"),
    // bg — J2 surface steps (app / surface / subtle / inset / overlay)
    req("bg", "app", "page/root face"),
    req("bg", "surface", "raised surface"),
    req("bg", "subtle", "recessed face"),
    req("bg", "inset", "input/combobox face"),
    req("bg", "overlay", "popover/menu face"),
    // fg
    req("fg", "foreground", "ink"),
    req("fg", "muted", "secondary ink"),
    // elevation
    req("elevation", "none", "no shadow"),
    req("elevation", "xs", "shadow step"),
    req("elevation", "sm", "shadow step"),
    req("elevation", "md", "shadow step"),
    req("elevation", "lg", "shadow step"),
    req("elevation", "xl", "shadow step"),
    // radius — L2 writes radius(md)
    req("radius", "sm", "corner step"),
    req("radius", "md", "corner step"),
    req("radius", "lg", "corner step"),
    req("radius", "xl", "corner step"),
    req("radius", "full", "pill"),
];

/// Empty: migrate-window aliases die after corpus grep-zero (retirement law).
const MIGRATE_ALIASES: &[StylePreset] = &[];

const fn req(concern: &'static str, name: &'static str, meaning: &'static str) -> StylePreset {
    StylePreset {
        concern,
        name,
        origin: PresetOrigin::ThemeRequired,
        meaning,
    }
}

/// Dotted floor keys. Parallel to [`REQUIRED_THEME_PRESETS`].
pub const REQUIRED_KEYS: &[&str] = &[
    "type.body",
    "type.body-sm",
    "type.body-semibold",
    "type.heading",
    "type.label",
    "type.mono",
    "type.mono-sm",
    "type.display-sm",
    "size.control-2xs",
    "size.control-xs",
    "size.control-sm",
    "size.control-md",
    "size.control-lg",
    "size.control-xl",
    "pad.page",
    "size.icon-3xs",
    "size.icon-2xs",
    "size.icon-xs",
    "size.icon-sm",
    "size.icon-md",
    "size.icon-lg",
    "size.icon-xl",
    "size.icon-2xl",
    "size.icon-3xl",
    "border.default",
    "border.subtle",
    "border.strong",
    "bg.app",
    "bg.surface",
    "bg.subtle",
    "bg.inset",
    "bg.overlay",
    "fg.foreground",
    "fg.muted",
    "elevation.none",
    "elevation.xs",
    "elevation.sm",
    "elevation.md",
    "elevation.lg",
    "elevation.xl",
    "radius.sm",
    "radius.md",
    "radius.lg",
    "radius.xl",
    "radius.full",
];

impl StyleSurface {
    /// Floor only: language + required + migrate aliases. No registry merge.
    pub fn floor() -> Self {
        let mut presets = Vec::with_capacity(
            LANGUAGE_PRESETS.len() + REQUIRED_THEME_PRESETS.len() + MIGRATE_ALIASES.len(),
        );
        presets.extend_from_slice(LANGUAGE_PRESETS);
        presets.extend_from_slice(REQUIRED_THEME_PRESETS);
        presets.extend_from_slice(MIGRATE_ALIASES);
        Self {
            atoms: atoms_from_registry(),
            presets,
            required: REQUIRED_KEYS.to_vec(),
            privilege: Privilege::Full,
        }
    }

    /// Floor plus theme-optional composites already in the registry that are
    /// not floor/language/alias. Theme adds (`type.kicker`) appear here.
    pub fn merged() -> Self {
        static CACHE: std::sync::OnceLock<StyleSurface> = std::sync::OnceLock::new();
        CACHE.get_or_init(Self::merge_now).clone()
    }

    fn merge_now() -> Self {
        let mut surface = Self::floor();
        let known: std::collections::BTreeSet<String> =
            surface.presets.iter().map(|p| p.key()).collect();
        for composite in tokens::all_composites() {
            let Some((concern, name)) = split_composite_name(composite) else {
                continue;
            };
            let key = format!("{concern}.{name}");
            if known.contains(&key) {
                continue;
            }
            // Host packs (bg.button) stay visible as optional until M3 deletes them.
            surface.presets.push(StylePreset {
                concern: leak_str(&concern),
                name: leak_str(&name),
                origin: PresetOrigin::ThemeOptional,
                meaning: "theme or host composite",
            });
        }
        surface
    }

    pub fn presets_for(&self, concern: &str) -> impl Iterator<Item = StylePreset> + '_ {
        let concern = concern.to_string();
        self.presets
            .iter()
            .copied()
            .filter(move |p| p.concern == concern)
    }

    pub fn find(&self, concern: &str, name: &str) -> Option<StylePreset> {
        self.presets
            .iter()
            .copied()
            .find(|p| p.concern == concern && p.name == name)
    }

    /// Required theme presets that have no matching composite in `present`.
    /// Language presets are not required of the theme. Radius steps live on
    /// the radius scale (not as `radius.sm` composites).
    pub fn missing_required(&self, present: &[&str]) -> Vec<&'static str> {
        self.required
            .iter()
            .copied()
            .filter(|key| !required_key_is_present(key, present))
            .collect()
    }

    pub fn missing_required_from_registry(&self) -> Vec<&'static str> {
        let present: Vec<&str> = tokens::all_composites().iter().map(|c| c.name).collect();
        self.missing_required(&present)
    }

    /// Human table for `sail-compiler surface`.
    pub fn format_table(&self) -> String {
        let mut out = String::new();
        out.push_str("StyleSurface  (SPEC-STYLE-ARCH-1)\n");
        out.push_str(
            "privilege: path+kind (Full stdlib / Identities L2 / None routes). \
             Config cannot grant Full. Product trees enforced.\n",
        );
        out.push_str(&format!(
            "atoms: {}  (from token registry)\n\n",
            self.atoms.len()
        ));

        out.push_str("## Language presets (compiler, not DTCG)\n");
        for p in self
            .presets
            .iter()
            .filter(|p| p.origin == PresetOrigin::Language)
        {
            out.push_str(&format!(
                "  {:>10}({:<10})  {}\n",
                p.concern, p.name, p.meaning
            ));
        }

        out.push_str("\n## Required theme presets\n");
        let present: std::collections::BTreeSet<&str> =
            tokens::all_composites().iter().map(|c| c.name).collect();
        for p in self
            .presets
            .iter()
            .filter(|p| p.origin == PresetOrigin::ThemeRequired)
        {
            let key = p.key();
            let present_keys: Vec<&str> = present.iter().copied().collect();
            let status = if required_key_is_present(&key, &present_keys) {
                "present"
            } else {
                "MISSING"
            };
            out.push_str(&format!(
                "  {:>10}.{:<12}  {status:<8}  {}\n",
                p.concern, p.name, p.meaning
            ));
        }

        out.push_str("\n## Optional / migrate\n");
        for p in self
            .presets
            .iter()
            .filter(|p| p.origin == PresetOrigin::ThemeOptional)
        {
            out.push_str(&format!(
                "  {:>10}.{:<12}  {}\n",
                p.concern, p.name, p.meaning
            ));
        }
        out
    }
}

fn split_composite_name(composite: &tokens::CompositeToken) -> Option<(String, String)> {
    let concern = composite.r#type;
    if concern.is_empty() {
        return None;
    }
    if let Some(rest) = composite.name.strip_prefix(&format!("{concern}.")) {
        return Some((concern.to_string(), rest.to_string()));
    }
    if let Some(rest) = composite.name.strip_suffix(&format!(".{concern}")) {
        return Some((concern.to_string(), rest.to_string()));
    }
    None
}

fn required_key_is_present(key: &str, present: &[&str]) -> bool {
    if present
        .iter()
        .any(|p| *p == key || composite_matches_key(p, key))
    {
        return true;
    }
    // Radius floor is the scale, not DTCG composites.
    if let Some(name) = key.strip_prefix("radius.") {
        return crate::tokens::radius_values().contains(&name);
    }
    false
}

fn composite_matches_key(composite_name: &str, key: &str) -> bool {
    if composite_name == key {
        return true;
    }
    let Some((concern, name)) = key.split_once('.') else {
        return false;
    };
    composite_name == format!("{name}.{concern}")
}

fn atoms_from_registry() -> Vec<StyleAtom> {
    let mut out: Vec<StyleAtom> = tokens::all_composites()
        .into_iter()
        .flat_map(|c| {
            c.members.iter().map(|(slot, css)| StyleAtom {
                path: format!("{}.{}", c.path, slot),
                scale: css.to_string(),
            })
        })
        .collect();
    // Primitive color / space / text names are the rest of the atom table.
    for name in tokens::color_values() {
        out.push(StyleAtom {
            path: format!("color.{name}"),
            scale: "color".into(),
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.dedup_by(|a, b| a.path == b.path);
    out
}

/// Language size participation. Not DTCG. Not a width token.
pub fn is_language_size_preset(name: &str) -> bool {
    matches!(name, "hug" | "grow" | "max")
}

fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// Resolve privilege from a source path. Config cannot grant Full here
/// (spinnaker cannot read `sail.config`). Slot form on project CEs is
/// `style_slots` in sail-lint. This helper is FileKind only:
/// - `crates/sail/compiler/stdlib/**` → Full
/// - `.sail` stem `page` | `index` | `_layout` under `routes/` → None
/// - other `.sail` → Identities (includes hyphenated route-local CEs)
/// - else → None
///
/// Do **not** treat every `routes/**` file as None.
/// `routes/pricing/pricing-hero.sail` is Identities.
///
/// M1 enforcement is scoped to product trees so compiler test fixtures
/// (including the L1 matrix) stay compilable.
pub fn privilege_for_path(path: &std::path::Path) -> Privilege {
    let p = path.to_string_lossy().replace('\\', "/");
    if is_stdlib_path(&p) {
        return Privilege::Full;
    }
    if p.ends_with(".sail") {
        if is_route_document_path(path, &p) {
            return Privilege::None;
        }
        return Privilege::Identities;
    }
    Privilege::None
}

/// Product trees PLAN-ARCH flips on. Test fixtures are not these trees.
/// Site / studio / blocks / l0 only. Fixtures stay off.
pub fn privilege_enforced_for_path(path: &std::path::Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    if is_stdlib_path(&p) {
        return false;
    }
    if p.contains("/tests/") || p.contains("/test/") || p.contains("/fixtures/") {
        return false;
    }
    // First-party Sail product trees only. A consumer repo that happens to
    // contain an `apps/` directory (helm/apps/helm, any OSS app) is NOT
    // PLAN-ARCH L3. Matching every `/apps/` made Style-in-routes a floor
    // error outside the monorepo.
    let under_apps = p.contains("/apps/sail-site/")
        || p.contains("/apps/studio/")
        || p.starts_with("apps/sail-site/")
        || p.starts_with("apps/studio/");
    let under_blocks =
        p.contains("/packages/sail/blocks/") || p.starts_with("packages/sail/blocks/");
    let under_l0 = is_l0_path(&p);
    under_apps || under_blocks || under_l0
}

fn is_l0_path(p: &str) -> bool {
    p.contains("/packages/sail/l0/") || p.starts_with("packages/sail/l0/")
}

fn is_stdlib_path(p: &str) -> bool {
    p.contains("/crates/sail/compiler/stdlib/") || p.starts_with("crates/sail/compiler/stdlib/")
}

/// Document `.sail` under `routes/` — stem check twins `classify()`.
fn is_route_document_path(path: &std::path::Path, p: &str) -> bool {
    let under_routes = p.contains("/routes/") || p.starts_with("routes/");
    under_routes && is_document_sail_stem(path)
}

fn is_document_sail_stem(path: &std::path::Path) -> bool {
    matches!(
        path.file_stem().and_then(|s| s.to_str()),
        Some("page" | "index" | "_layout")
    )
}

/// Warn lines for generate (M0). M3 flips these to hard fail.
pub fn generate_missing_warnings() -> Vec<String> {
    let missing = StyleSurface::floor().missing_required_from_registry();
    missing
        .into_iter()
        .map(|key| format!("error: required theme preset `{key}` missing from registry"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_keys_match_required_presets() {
        assert_eq!(REQUIRED_KEYS.len(), REQUIRED_THEME_PRESETS.len());
        for (key, preset) in REQUIRED_KEYS.iter().zip(REQUIRED_THEME_PRESETS) {
            assert_eq!(*key, preset.key());
        }
    }

    #[test]
    fn privilege_follows_file_kind() {
        use std::path::Path;
        assert_eq!(
            privilege_for_path(Path::new("crates/sail/compiler/stdlib/button/button.sail")),
            Privilege::Full
        );
        assert_eq!(
            privilege_for_path(Path::new("apps/sail-site/routes/docs/page.sail")),
            Privilege::None
        );
        assert_eq!(
            privilege_for_path(Path::new(
                "packages/sail/blocks/routes/pricing/pricing-hero.sail"
            )),
            Privilege::Identities
        );
        assert_eq!(
            privilege_for_path(Path::new("apps/studio/src/studio-panel/studio-panel.sail")),
            Privilege::Identities
        );
        assert!(privilege_enforced_for_path(Path::new(
            "apps/sail-site/routes/docs/page.sail"
        )));
        assert!(privilege_enforced_for_path(Path::new(
            "apps/studio/src/studio-panel/studio-panel.sail"
        )));
        assert!(privilege_enforced_for_path(Path::new(
            "packages/sail/blocks/routes/pricing/pricing-hero.sail"
        )));
        assert!(privilege_enforced_for_path(Path::new(
            "packages/sail/l0/routes/poison/poison-trap.sail"
        )));
        assert!(privilege_enforced_for_path(Path::new(
            "packages/sail/l0/src/l0-viewer/l0-viewer.sail"
        )));
        assert_eq!(
            privilege_for_path(Path::new("packages/sail/l0/routes/page.sail")),
            Privilege::None
        );
        assert!(!privilege_enforced_for_path(Path::new(
            "crates/sail/compiler/tests/fixtures/l1-matrix/routes/button/page.sail"
        )));
    }

    #[test]
    fn type_heading_is_required_theme_preset() {
        let surface = StyleSurface::floor();
        assert!(
            surface.required.contains(&"type.heading"),
            "required: {:?}",
            surface.required
        );
        let heading = surface.find("type", "heading").expect("type.heading");
        assert_eq!(heading.origin, PresetOrigin::ThemeRequired);
    }

    #[test]
    fn size_grow_is_language_not_dtcg() {
        let surface = StyleSurface::floor();
        let grow = surface.find("size", "grow").expect("size.grow");
        assert_eq!(grow.origin, PresetOrigin::Language);
        assert!(
            !surface.required.contains(&"size.grow"),
            "language presets are not required of the theme"
        );
        assert!(
            tokens::find_composite("size.grow").is_none(),
            "size.grow must not be a DTCG composite"
        );
    }

    #[test]
    fn size_full_is_gone() {
        let surface = StyleSurface::floor();
        assert!(
            surface.find("size", "full").is_none(),
            "size.full retired — use size(grow)|size(max) or free 100%"
        );
    }

    #[test]
    fn missing_required_names_app_and_surfaces() {
        let missing = StyleSurface::floor().missing_required_from_registry();
        for key in [
            "bg.app",
            "bg.inset",
            "bg.overlay",
            "fg.foreground",
            "size.control-md",
        ] {
            assert!(
                !missing.contains(&key),
                "{key} must be on the floor now: {missing:?}"
            );
        }
        assert!(
            !missing.contains(&"type.heading"),
            "type.heading exists in the registry: {missing:?}"
        );
        assert!(missing.is_empty(), "M3 floor must be complete: {missing:?}");
    }

    #[test]
    fn complete_type_offers_heading_from_surface() {
        let (src, off) = at_caret("style({ type(¦) })");
        let items = crate::style_complete_at(&src, None, off, None);
        let labs: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(
            labs.contains(&"heading"),
            "type(¦) must offer heading: {labs:?}"
        );
        assert!(
            labs.contains(&"label"),
            "type(¦) must offer label: {labs:?}"
        );
        assert!(labs.contains(&"body"), "type(¦) must offer body: {labs:?}");
        let heading = items.iter().find(|i| i.label == "heading").unwrap();
        let detail = heading.detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("type.heading") || detail.contains("preset"),
            "heading complete should cite the surface preset, got {detail:?}"
        );
    }

    #[test]
    fn complete_offers_size_grow_as_language_preset() {
        let (src, off) = at_caret("style({ size(¦) })");
        let items = crate::style_complete_at(&src, None, off, None);
        let labs: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labs.contains(&"grow"), "size(grow): {labs:?}");
        assert!(labs.contains(&"hug"), "size(hug): {labs:?}");
        assert!(labs.contains(&"max"), "size(max): {labs:?}");
    }

    fn at_caret(src: &str) -> (String, usize) {
        let i = src.find('¦').expect("caret");
        let mut s = String::with_capacity(src.len() - '¦'.len_utf8());
        s.push_str(&src[..i]);
        s.push_str(&src[i + '¦'.len_utf8()..]);
        (s, i)
    }
}
