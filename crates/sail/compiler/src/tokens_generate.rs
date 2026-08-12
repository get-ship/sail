//! DTCG model → `tokens.css` + `themes.css` + `token-registry.json` +
//! `generated/unions.ts`.
//!
//! SSOT is the design-token catalog: baked into the binary at compile time
//! (same idea as delivery CSS) and optionally overridden by an on-disk
//! monorepo tree at `packages/sail/tokens/model/**` or `--model`.
//!
//! Invoke: `cargo run -p sail-compiler -- tokens`

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use serde_json::{Map, Value};

/// Compile-time baked catalog (`packages/sail/tokens/model/**` at build).
mod baked_tokens_model {
    include!(concat!(env!("OUT_DIR"), "/baked_tokens_model.rs"));
}

const GENERATED_HEADER: &str = "\
/*
 * GENERATED FILE — do not edit by hand.
 *
 * Source of truth: packages/sail/tokens/model/** (DTCG groups + themes).
 * Regenerate: cargo run -p sail-compiler -- tokens
 *            (or: pnpm --filter @ship/tokens generate)
 *
 * Token layers, in order of abstraction (highest first):
 *   1. Semantic surfaces  — --surface-0..3, --backdrop
 *   2. Tone ramps         — --tone-{name}-{fg|bg|bg-solid|border|ring}
 *   3. Brand palette      — --primary, --accent, --foreground, etc.
 *   4. Primitives         — spacing, radii, typography scale, motion curves
 *
 * Components should read the HIGHEST layer that fits.
 * Themes: model/themes/*.json → themes.css (plus model/themes-chrome.css).
 */
";

const THEMES_HEADER: &str = "\
/*
 * GENERATED FILE — do not edit by hand.
 *
 * Theme token overrides from packages/sail/tokens/model/themes/*.json.
 * Structural flourishes: packages/sail/tokens/model/themes-chrome.css
 * Regenerate: cargo run -p sail-compiler -- tokens
 */
";

const UNIONS_HEADER: &str = "\
/**
 * GENERATED FILE — do not edit by hand.
 *
 * Closed-set TypeScript unions derived from the DTCG model + token registry.
 * Source of truth: packages/sail/tokens/model/** (groups + themes).
 * Regenerate: cargo run -p sail-compiler -- tokens
 *            (or: pnpm --filter @ship/tokens generate)
 *
 * L1 / app props import these instead of hand-maintaining literal unions:
 *   import type { ToneName, CssTokenName, ThemeId } from '@ship/tokens'
 *   // or: import type { ToneName } from '@ship/tokens/unions'
 */

";

const PRODUCTS_HEADER: &str = "\
/*
 * GENERATED FILE — do not edit by hand.
 *
 * Source of truth: packages/sail/tokens/model/products.json (system channels).
 * Regenerate: cargo run -p sail-compiler -- tokens
 *            (or: pnpm --filter @ship/tokens generate)
 *
 * Product brand hues. The two axes compose instead of colliding: a THEME owns
 * lightness and chroma (its character — how muted, how bright, how glassy), a
 * PRODUCT owns hue (the brand). Every brand token in tokens.css and themes.css
 * is written `oklch(L C var(--brand-hue, <theme's own hue>))`, so with no
 * product set the theme looks exactly as it always did, and with one set only
 * the hue moves.
 *
 * Switch the theme and the whole page repaints. Cross from SAIL into Deck and
 * only the accent turns violet.
 */

";

/// Stable theme catalog order. One id per file; light/dark live under `modes`.
const THEME_ORDER: &[&str] = &[
    "ship",
    "neutral",
    "paper",
    "halo",
    "console",
    "contrast",
    "glass",
    "brut",
    "8bit",
    "synthwave",
];

// ── THEMES-1 schema lock (authoring contract) ───────────────────────────────
// Theme DATA MODEL is the forever contract — DTCG JSON is storage only.
// A theme = { name (themeId), modes capability (light|dark|both), scale
// overrides, component-token overrides }. Unknown keys refuse naming valid ones.

/// Allowed top-level keys on a theme document.
const THEME_ROOT_KEYS: &[&str] = &["$description", "$extensions", "modes"];

/// Allowed keys under `$extensions.sail` on a theme document.
const THEME_SAIL_KEYS: &[&str] = &[
    "layer",
    "themeId",
    "defaultMode",
    "supportedModes",
    "density",
    "ordered",
];

/// Allowed paint mode names (`data-mode` values).
const THEME_PAINT_MODES: &[&str] = &["light", "dark"];

/// Allowed top-level groups inside a mode body (scale + component overrides).
/// DTCG `$extensions` on the mode object is also allowed.
const THEME_MODE_GROUP_KEYS: &[&str] = &[
    "$extensions",
    "color",
    "component",
    "control",
    "interaction",
    "motion",
    "radius",
    "scrollbar",
    "shadow",
    "typography",
    "space",
    "icon",
    "density",
    "panel",
    "z",
    "breakpoint",
];

/// Override resolution order (highest wins first). THEMES-1 WI3 / freeze §6.
const THEME_OVERRIDE_ORDER: &[&str] = &["instance", "component", "global", "baked"];

/// Preferred L1 tone axis order. Membership itself comes from DTCG
/// `color.tone.<name>.{fg,bg,...}` ramps when present; this only sorts the set.
const TONE_ORDER: &[&str] = &[
    "primary",
    "info",
    "success",
    "warning",
    "destructive",
    "accent",
    "muted",
];

/// Fallback tone membership (no `'default'` — rest = omit tone).
const TONE_FALLBACK: &[&str] = TONE_ORDER;

/// Special layers loaded from `model/<name>.json`. Dark is a **mode inside
/// themes**, not a global dual layer — `dark.json` is intentionally absent.
const SPECIAL_LAYERS: &[&str] = &[
    "base",
    "reduced-motion",
    "density-compact",
    "density-spacious",
];

/// Semantic brand / surface paint groups under `color.*` (not product accents,
/// tone ramps, gradients, or component packs).
const SEMANTIC_COLOR_GROUPS: &[&str] = &[
    "background",
    "foreground",
    "primary",
    "secondary",
    "muted",
    "brandAccent",
    "destructive",
    "info",
    "success",
    "warning",
    "input",
    "ring",
    "backdrop",
    "divider",
    "scrim",
    "edgeHighlight",
];

#[derive(Debug, Clone)]
struct Token {
    path: String,
    css_name: String,
    r#type: String,
    value: String,
    css_value: String,
    /// `$extensions.sail.channel` — a Tier-2 expression channel. Channel
    /// tokens are partitioned out of the base `:root` block at emission and
    /// re-emitted under the `Expression channels (Tier 2)` banner so the
    /// channel-integrity tests can find (and police) the full set.
    channel: bool,
    /// `$description` — emitted as a doc comment above the declaration in
    /// the channel block (the old hand-authored block's prose, now SSOT).
    description: Option<String>,
}

#[derive(Debug, Clone)]
struct LayerMeta {
    file: String,
    description: String,
    selector: String,
    media: Option<String>,
    layer: String,
    theme_id: Option<String>,
    dark: bool,
    /// Mode name when this layer is a modes-in-file emit (Phase D). `None` =
    /// legacy single-block theme (twin light/dark ids).
    mode: Option<String>,
    /// Default mode for the theme id (from `$extensions.sail.defaultMode`).
    default_mode: Option<String>,
}

#[derive(Debug, Clone)]
struct Layer {
    meta: LayerMeta,
    tokens: Vec<Token>,
}

#[derive(Debug, Clone, Default)]
struct NameIndex {
    path_to_css: HashMap<String, String>,
    css_to_path: HashMap<String, String>,
}

/// Summary of a successful generate run (for CLI logging / tests).
#[derive(Debug, Clone)]
pub struct GenerateReport {
    pub token_count: usize,
    pub theme_count: usize,
    pub color_count: usize,
    pub path_count: usize,
    pub layers: Vec<String>,
    pub tokens_css: PathBuf,
    pub themes_css: PathBuf,
    pub products_css: PathBuf,
    pub registry_json: PathBuf,
    pub unions_ts: PathBuf,
}

/// Product brand-hue system axis (model/products.json) — not theme-overridable.
#[derive(Debug, Clone)]
struct ProductEntry {
    id: String,
    hue: u32,
    label: String,
}

#[derive(Debug, Clone)]
struct ProductsAxis {
    channel: String,
    composition_law: String,
    entries: Vec<ProductEntry>,
}

/// Default project theme id scaffolded into `<project>/themes/` (modes-in-file).
pub const DEFAULT_PROJECT_THEME_ID: &str = "ship";

/// Relative monorepo path to the on-disk DTCG model (catalog development).
pub const MONOREPO_MODEL_REL: &str = "packages/sail/tokens/model";

/// Relative monorepo path to package delivery output.
pub const MONOREPO_TOKENS_SRC_REL: &str = "packages/sail/tokens/src";

/// Catalog allowlist from `theme.enabled` in `sail.config` (T0).
///
/// | Form | Meaning |
/// |------|---------|
/// | omit key / no config | Default Sail theme [`DEFAULT_PROJECT_THEME_ID`] |
/// | `[]` | Explicit opt-out: no catalog themes |
/// | `["ship", …]` | Only those catalog ids |
/// | `"all"` | Explicit full catalog |
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CatalogAllowlist {
    /// Key omitted / missing config — ship default styling (not full catalog).
    #[default]
    Default,
    /// Explicit `"enabled": []` — no catalog themes in app emit.
    Empty,
    /// `"enabled": "all"` — full catalog.
    All,
    /// Explicit catalog id list.
    Ids(Vec<String>),
}

/// App-level `theme` section from `sail.config.json(c)` (T0).
///
/// ```jsonc
/// {
///   "theme": {
///     "default": "ship",
///     "defaultMode": "light",
///     // omit enabled → default Sail theme (ship) ships.
///     // "enabled": [] → explicit no catalog
///     // "enabled": ["ship", "paper"] | "all"
///   }
/// }
/// ```
///
/// **Consume sites:**
/// - `enabled` → [`GenerateOptions::enabled_catalog`] for **app** delivery
///   ([`options_for_project`]). Monorepo shared `@ship/tokens` uses
///   [`options_for_shared_package`] (full catalog + project merge).
/// - `default` / `defaultMode` → host paint + config-loud validation against
///   the effective emit set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThemeConfig {
    /// Preferred default theme id (scaffold / host paint).
    pub default: Option<String>,
    /// Initial `data-mode` (prefer `light` when omitted at host).
    pub default_mode: Option<String>,
    /// Catalog theme allowlist (T0). See [`CatalogAllowlist`].
    pub enabled: CatalogAllowlist,
    /// True when the `enabled` key was present in config (even if `[]`).
    pub enabled_key_present: bool,
}

impl ThemeConfig {
    /// Resolve the default theme id, falling back to [`DEFAULT_PROJECT_THEME_ID`].
    pub fn default_theme_id(&self) -> &str {
        self.default
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_PROJECT_THEME_ID)
    }

    /// Map config `enabled` → app emit filter (`None` = full catalog).
    pub fn enabled_catalog_for_app(&self) -> Option<Vec<String>> {
        match &self.enabled {
            CatalogAllowlist::Default => Some(vec![DEFAULT_PROJECT_THEME_ID.to_string()]),
            CatalogAllowlist::Empty => Some(Vec::new()),
            CatalogAllowlist::All => None,
            CatalogAllowlist::Ids(ids) => Some(ids.clone()),
        }
    }
}

/// Resolved emit-set membership for studio / tests / validation (T1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectiveThemeIds {
    /// Stems of project `themes/*.json` (and legacy `tokens/themes/*.json`).
    pub project: Vec<String>,
    /// Catalog ids that participate (allowlist ∩ existing catalog files).
    pub catalog: Vec<String>,
}

impl EffectiveThemeIds {
    /// Union of project + catalog ids (project ids first, stable).
    pub fn all_ids(&self) -> Vec<String> {
        let mut out = self.project.clone();
        for id in &self.catalog {
            if !out.iter().any(|p| p == id) {
                out.push(id.clone());
            }
        }
        out
    }
}

/// Emit-set options for [`generate_with_options`] (T1).
///
/// - **Package** ([`options_for_shared_package`] / bare [`generate`]): full
///   catalog (`enabled_catalog: None`), optional project merge.
/// - **App** ([`options_for_project`]): only effective set from T0 `enabled`
///   (`Some([])` when thin/omit — never silent full catalog).
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Project DTCG theme directories (`<project>/themes`, legacy
    /// `tokens/themes`). Every `*.json` here is always in the emit set; same
    /// `themeId` overrides catalog.
    pub project_theme_dirs: Vec<PathBuf>,
    /// When `Some`, only these **catalog** theme ids are emitted (project themes
    /// still always emit). When `None`, all catalog themes emit.
    pub enabled_catalog: Option<Vec<String>>,
    /// When true, write **only** `themes.css` (app B1 delivery). Skips tokens.css,
    /// registry, unions, themes-meta so app out dirs stay free of package litter.
    pub themes_only: bool,
}

// ── Baked catalog / model resolution ────────────────────────────────────────

/// Materialize the compile-time catalog to a process-local cache directory.
///
/// Generate still reads a filesystem tree; baking avoids requiring
/// `packages/sail/tokens/model` on disk for external apps.
pub fn materialize_baked_model() -> Result<PathBuf> {
    static DIR: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    let cached = DIR.get_or_init(|| materialize_baked_model_inner().map_err(|e| e.to_string()));
    match cached {
        Ok(p) => Ok(p.clone()),
        Err(e) => bail!("baked tokens model: {e}"),
    }
}

fn materialize_baked_model_inner() -> Result<PathBuf> {
    let base = std::env::temp_dir().join(format!(
        "sail-baked-tokens-model-{}",
        baked_model_fingerprint()
    ));
    let marker = base.join(".complete");
    if marker.is_file() && base.join("base.json").is_file() {
        return Ok(base);
    }
    // Rebuild from scratch if incomplete/stale.
    if base.exists() {
        let _ = fs::remove_dir_all(&base);
    }
    for (rel, body) in baked_tokens_model::BAKED_MODEL_FILES {
        let dest = base.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
        }
        fs::write(&dest, body).with_context(|| format!("write {}", dest.display()))?;
    }
    fs::write(&marker, b"ok\n").with_context(|| format!("write {}", marker.display()))?;
    Ok(base)
}

fn baked_model_fingerprint() -> u64 {
    // Cheap content fingerprint so model edits at compile time land in a new dir.
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    baked_tokens_model::BAKED_MODEL_FILES.len().hash(&mut h);
    for (rel, body) in baked_tokens_model::BAKED_MODEL_FILES {
        rel.hash(&mut h);
        body.len().hash(&mut h);
        // Sample ends so tiny edits still move the fingerprint.
        body.bytes().next().hash(&mut h);
        body.bytes().last().hash(&mut h);
    }
    h.finish()
}

/// Theme ids present in the baked catalog (sorted).
pub fn baked_catalog_theme_ids() -> Vec<String> {
    let mut ids: Vec<String> = baked_tokens_model::BAKED_MODEL_FILES
        .iter()
        .filter_map(|(rel, _)| {
            let rel = rel.replace('\\', "/");
            let rest = rel.strip_prefix("themes/")?;
            if rest.contains('/') {
                return None;
            }
            let stem = rest.strip_suffix(".json")?;
            if stem.is_empty() || stem.starts_with('.') {
                return None;
            }
            Some(stem.to_string())
        })
        .collect();
    ids.sort();
    ids
}

/// Raw JSON body for a baked catalog theme id, if present.
pub fn baked_catalog_theme_json(theme_id: &str) -> Option<&'static str> {
    let want = format!("themes/{theme_id}.json");
    baked_tokens_model::BAKED_MODEL_FILES
        .iter()
        .find(|(rel, _)| *rel == want || rel.replace('\\', "/") == want)
        .map(|(_, body)| *body)
}

/// Resolve the DTCG model directory for generate.
///
/// Precedence:
/// 1. `model_override` when set (CLI `--model` / monorepo catalog development)
/// 2. `<workspace>/packages/sail/tokens/model` when that tree exists on disk
/// 3. Materialized baked catalog (external / no monorepo tree)
pub fn resolve_model_dir(
    workspace_root: Option<&Path>,
    model_override: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(m) = model_override {
        if m.is_dir() {
            return Ok(m.to_path_buf());
        }
        bail!("model directory not found: {}", m.display());
    }
    if let Some(ws) = workspace_root {
        let disk = ws.join(MONOREPO_MODEL_REL);
        if disk.is_dir() {
            return Ok(disk);
        }
    }
    materialize_baked_model()
}

/// True when `workspace_root` hosts the monorepo `@ship/tokens` package tree.
pub fn monorepo_tokens_package_present(workspace_root: &Path) -> bool {
    workspace_root
        .join("packages/sail/tokens/package.json")
        .is_file()
        && workspace_root.join(MONOREPO_MODEL_REL).is_dir()
}

/// Whether dual `sail dev` should also write package delivery into
/// `packages/sail/tokens/src`.
///
/// Package delivery is monorepo-only (catalog development). External apps never
/// write it. Opt out in monorepo: `SAIL_TOKENS_PACKAGE_GENERATE=0`.
pub fn should_generate_package_delivery(workspace_root: &Path) -> bool {
    match std::env::var("SAIL_TOKENS_PACKAGE_GENERATE") {
        Ok(v) if v == "0" || v.eq_ignore_ascii_case("false") => false,
        Ok(v) if v == "1" || v.eq_ignore_ascii_case("true") => {
            monorepo_tokens_package_present(workspace_root)
        }
        _ => monorepo_tokens_package_present(workspace_root),
    }
}

/// Project theme directories: `<project>/themes` then legacy `tokens/themes`.
///
/// Only existing directories are returned. Modern layout is `themes/`;
/// legacy `tokens/themes` remains dual-read for external apps mid-transition.
pub fn project_theme_dirs(project_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let modern = project_dir.join("themes");
    let legacy = project_dir.join("tokens").join("themes");
    if modern.is_dir() {
        dirs.push(modern);
    }
    if legacy.is_dir() {
        dirs.push(legacy);
    }
    dirs
}

/// Prefer `<project>/themes` for new writes; fall back to legacy dir if that is
/// where themes already live and `themes/` does not.
pub fn project_themes_write_dir(project_dir: &Path) -> PathBuf {
    let modern = project_dir.join("themes");
    let legacy = project_dir.join("tokens").join("themes");
    if modern.is_dir() {
        return modern;
    }
    if legacy.is_dir() && !modern.exists() {
        return legacy;
    }
    modern
}

/// Union of theme ids across all project theme dirs (sorted, unique).
pub fn list_project_theme_ids(project_dir: &Path) -> Vec<String> {
    let mut set = BTreeSet::new();
    for dir in project_theme_dirs(project_dir) {
        for id in list_theme_ids_in_dir(&dir) {
            set.insert(id);
        }
    }
    set.into_iter().collect()
}

/// Absolute path candidates for a project theme id (modern then legacy).
fn project_theme_file_candidates(project_dir: &Path, theme_id: &str) -> [PathBuf; 2] {
    [
        project_dir.join("themes").join(format!("{theme_id}.json")),
        project_dir
            .join("tokens")
            .join("themes")
            .join(format!("{theme_id}.json")),
    ]
}

/// Read DTCG under `model_dir` and write the three artifacts into `out_dir`.
/// Catalog-only emit (full theme library) — used by `sail-compiler tokens`.
pub fn generate(model_dir: &Path, out_dir: &Path) -> Result<GenerateReport> {
    generate_with_options(model_dir, out_dir, &GenerateOptions::default())
}

/// Like [`generate`], with project theme dirs + optional catalog id filter (C2).
pub fn generate_with_options(
    model_dir: &Path,
    out_dir: &Path,
    opts: &GenerateOptions,
) -> Result<GenerateReport> {
    let (layers, name_index, base_tokens) = load_base_layers(model_dir)?;
    if layers.is_empty() {
        bail!("No model/*.json files found in {}", model_dir.display());
    }
    // MIGRATION-DEBT: dies with CHANNELS-1 / TOKENS-1 (token-alias migration).
    let aliases = load_compat_aliases(model_dir)?;
    let products = load_products(model_dir)?;
    let themes = load_emit_themes(model_dir, &name_index, opts)?;

    let by_layer: HashMap<String, &Layer> =
        layers.iter().map(|l| (l.meta.layer.clone(), l)).collect();

    let mut ordered: Vec<&Layer> = Vec::new();
    if let Some(base) = by_layer.get("base") {
        ordered.push(base);
    }
    for l in &layers {
        if SPECIAL_LAYERS.contains(&l.meta.layer.as_str()) {
            continue;
        }
        ordered.push(l);
    }

    let chrome_path = model_dir.join("chrome.css");
    let chrome = match fs::read_to_string(&chrome_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("warn: model/chrome.css missing — emitting tokens only");
            String::new()
        }
    };

    let mut css_parts: Vec<String> = vec![GENERATED_HEADER.to_string()];
    for l in &ordered {
        if l.meta.layer == "base" {
            // Tier-2 expression channels are partitioned out of the base
            // `:root` block and re-emitted under their own banner so the
            // channel-integrity tests can parse (and police) the full set.
            // Declarations are NOT duplicated: each channel token leaves the
            // base block exactly once.
            let (channels, regular): (Vec<Token>, Vec<Token>) =
                l.tokens.iter().cloned().partition(|t| t.channel);
            let base_regular = Layer {
                meta: l.meta.clone(),
                tokens: regular,
            };
            css_parts.push(emit_layer(&base_regular));
            css_parts.push(String::new());
            if !channels.is_empty() {
                css_parts.push(emit_channel_block(&channels));
            }
        } else {
            css_parts.push(emit_layer(l));
        }
        css_parts.push(String::new());
    }
    if !aliases.is_empty() {
        css_parts.push("/* ─── compat aliases (legacy CSS names) ─── */".into());
        css_parts.push(emit_compat_aliases(":root", &aliases));
        css_parts.push(String::new());
    }
    if !chrome.is_empty() {
        css_parts.push(chrome.trim_end().to_string());
        css_parts.push(String::new());
    }
    if let Some(l) = by_layer.get("reduced-motion") {
        css_parts.push(emit_layer(l));
        css_parts.push(String::new());
    }
    for name in ["density-compact", "density-spacious"] {
        if let Some(l) = by_layer.get(name) {
            css_parts.push(emit_layer(l));
            css_parts.push(String::new());
        }
    }

    let css = collapse_blank_lines(&css_parts.join("\n"));
    fs::create_dir_all(out_dir).with_context(|| format!("create out dir {}", out_dir.display()))?;
    // Package delivery writes tokens.css; app B1 (`themes_only`) skips it.
    let tokens_css = out_dir.join("tokens.css");
    if !opts.themes_only {
        write_with_trailing_newline(&tokens_css, &css)?;
    }

    // themes.css — preserve multi-mode layers (same theme_id, different mode).
    let mut ordered_themes: Vec<&Layer> = Vec::new();
    let mut emitted_keys: HashSet<String> = HashSet::new();
    for id in THEME_ORDER {
        for t in &themes {
            let tid = t.meta.theme_id.as_deref().unwrap_or("");
            if tid != *id {
                continue;
            }
            let key = theme_emit_key(t);
            if emitted_keys.insert(key) {
                ordered_themes.push(t);
            }
        }
    }
    for t in &themes {
        let key = theme_emit_key(t);
        if emitted_keys.insert(key) {
            ordered_themes.push(t);
        }
    }

    let mut theme_parts: Vec<String> = vec![THEMES_HEADER.to_string()];
    for t in &ordered_themes {
        theme_parts.push(emit_layer(t));
        theme_parts.push(String::new());
    }
    // Structural chrome is hand-authored for the full catalog; it is always
    // concatenated even when `enabled_catalog` thins token blocks (Phase D
    // may slim chrome to match the emit set). Token variables still follow
    // the emit set — chrome only adds non-token flourishes.
    let themes_chrome_path = model_dir.join("themes-chrome.css");
    if themes_chrome_path.is_file() {
        let chrome_t = fs::read_to_string(&themes_chrome_path)
            .with_context(|| format!("read {}", themes_chrome_path.display()))?;
        theme_parts.push(chrome_t.trim_end().to_string());
        theme_parts.push(String::new());
    }
    let themes_css_body = collapse_blank_lines(&theme_parts.join("\n"));
    let themes_css = out_dir.join("themes.css");
    write_with_trailing_newline(&themes_css, &themes_css_body)?;

    // Unique theme ids (modes share one id).
    let mut theme_ids: Vec<String> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();
    for t in &ordered_themes {
        if let Some(id) = &t.meta.theme_id {
            if seen_ids.insert(id.clone()) {
                theme_ids.push(id.clone());
            }
        }
    }

    // App B1 delivery: only themes.css (no package tokens/registry/unions litter).
    if opts.themes_only {
        return Ok(GenerateReport {
            token_count: 0,
            theme_count: theme_ids.len(),
            color_count: 0,
            path_count: 0,
            layers: layers.iter().map(|l| l.meta.layer.clone()).collect(),
            tokens_css: PathBuf::new(),
            themes_css,
            products_css: PathBuf::new(),
            registry_json: PathBuf::new(),
            unions_ts: PathBuf::new(),
        });
    }

    // products.css — system brand-hue axis (not theme-overridable).
    let products_css = out_dir.join("products.css");
    if let Some(ref axis) = products {
        write_with_trailing_newline(&products_css, &emit_products_css(axis))?;
    }

    let mut registry = build_registry(&base_tokens, &aliases);
    if let Some(ref axis) = products {
        if let Some(obj) = registry.as_object_mut() {
            obj.insert("products".into(), products_registry_meta(axis));
        }
    }
    let color_count = registry["colors"].as_array().map(|a| a.len()).unwrap_or(0);
    let path_count = registry["paths"].as_object().map(|o| o.len()).unwrap_or(0);
    let token_count = registry["names"].as_array().map(|a| a.len()).unwrap_or(0);
    let registry_json = out_dir.join("token-registry.json");
    let reg_text = serde_json::to_string_pretty(&registry)? + "\n";
    fs::write(&registry_json, reg_text)
        .with_context(|| format!("write {}", registry_json.display()))?;

    let unions_body = build_unions_ts(&base_tokens, &registry, &theme_ids);
    let unions_dir = out_dir.join("generated");
    fs::create_dir_all(&unions_dir).with_context(|| format!("create {}", unions_dir.display()))?;
    let unions_ts = unions_dir.join("unions.ts");
    write_with_trailing_newline(&unions_ts, &unions_body)?;

    // Generated theme meta (id, dark, modes) — thin themes.ts consumes this.
    let themes_meta_body = build_themes_meta_ts(&ordered_themes);
    let themes_meta_ts = unions_dir.join("themes-meta.ts");
    write_with_trailing_newline(&themes_meta_ts, &themes_meta_body)?;

    Ok(GenerateReport {
        token_count,
        // Count unique theme ids (not mode blocks).
        theme_count: theme_ids.len(),
        color_count,
        path_count,
        layers: layers.iter().map(|l| l.meta.layer.clone()).collect(),
        tokens_css,
        themes_css,
        products_css,
        registry_json,
        unions_ts,
    })
}

/// Emit `generated/themes-meta.ts` — catalog meta + THEMES-1 schema lock.
fn build_themes_meta_ts(layers: &[&Layer]) -> String {
    // Group modes by theme id.
    let mut by_id: BTreeMap<String, (bool, Option<String>, Vec<String>)> = BTreeMap::new();
    for t in layers {
        let Some(id) = &t.meta.theme_id else {
            continue;
        };
        let entry = by_id
            .entry(id.clone())
            .or_insert_with(|| (false, t.meta.default_mode.clone(), Vec::new()));
        if let Some(mode) = &t.meta.mode {
            if !entry.2.contains(mode) {
                entry.2.push(mode.clone());
            }
            // dark flag: true if default mode is dark, or single-mode dark theme
            if t.meta.default_mode.as_deref() == Some(mode.as_str()) {
                entry.0 = t.meta.dark;
            }
            if entry.1.is_none() {
                entry.1 = t.meta.default_mode.clone();
            }
        } else {
            entry.0 = t.meta.dark;
        }
    }
    // Rank modes light → dark → alpha (same as emit order).
    for entry in by_id.values_mut() {
        entry
            .2
            .sort_by(|a, b| mode_rank(a).cmp(&mode_rank(b)).then_with(|| a.cmp(b)));
    }

    let mut out = String::new();
    out.push_str(
        "/**\n * GENERATED FILE — do not edit by hand.\n *\n \
         * Theme catalog meta + THEMES-1 schema lock from\n \
         * packages/sail/tokens/model/themes/**.\n \
         * Includes modes-in-file (Phase D) + mode capability (light|dark|both).\n \
         * Regenerate: sail tokens\n */\n\n",
    );

    // ── Schema lock (THEMES-1 WI1) ──────────────────────────────────────────
    out.push_str("/** Allowed top-level keys on a theme document. */\n");
    out.push_str(&format!(
        "export const THEME_ROOT_KEYS = [{}] as const\n",
        THEME_ROOT_KEYS
            .iter()
            .map(|k| format!("'{k}'"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str("export type ThemeRootKey = (typeof THEME_ROOT_KEYS)[number]\n\n");

    out.push_str("/** Allowed keys under `$extensions.sail` on a theme. */\n");
    out.push_str(&format!(
        "export const THEME_SAIL_KEYS = [{}] as const\n",
        THEME_SAIL_KEYS
            .iter()
            .map(|k| format!("'{k}'"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str("export type ThemeSailKey = (typeof THEME_SAIL_KEYS)[number]\n\n");

    out.push_str("/** Allowed paint mode names (`data-mode` values). */\n");
    out.push_str(&format!(
        "export const THEME_PAINT_MODES = [{}] as const\n",
        THEME_PAINT_MODES
            .iter()
            .map(|k| format!("'{k}'"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str("export type ThemePaintMode = (typeof THEME_PAINT_MODES)[number]\n\n");

    out.push_str(
        "/** Allowed top-level groups inside a mode body (scale + component overrides). */\n",
    );
    out.push_str(&format!(
        "export const THEME_MODE_GROUP_KEYS = [{}] as const\n",
        THEME_MODE_GROUP_KEYS
            .iter()
            .map(|k| format!("'{k}'"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str("export type ThemeModeGroupKey = (typeof THEME_MODE_GROUP_KEYS)[number]\n\n");

    out.push_str(
        "/**\n * Override resolution order (highest wins first).\n \
         * Freeze §6 / inv-theme-override-order: instance (style/channels) >\n \
         * component tokens > global theme > baked defaults.\n */\n",
    );
    out.push_str(&format!(
        "export const THEME_OVERRIDE_ORDER = [{}] as const\n",
        THEME_OVERRIDE_ORDER
            .iter()
            .map(|k| format!("'{k}'"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str("export type ThemeOverrideLayer = (typeof THEME_OVERRIDE_ORDER)[number]\n\n");

    out.push_str("/** Per-theme declared mode capability (VSCode model — not universal). */\n");
    out.push_str("export type ModeCapability = 'light' | 'dark' | 'both'\n\n");

    out.push_str("export interface ThemeMeta {\n");
    out.push_str("  id: string\n");
    out.push_str("  /** Whether the default mode is a dark surface (data-mode dark). */\n");
    out.push_str("  dark: boolean\n");
    out.push_str("  /** Mode names when modes-in-file; empty = legacy single-block theme. */\n");
    out.push_str("  modes: readonly string[]\n");
    out.push_str("  defaultMode: string | null\n");
    out.push_str("  /** Declared capability derived from modes: light | dark | both. */\n");
    out.push_str("  capability: ModeCapability\n");
    out.push_str("}\n\n");
    out.push_str("export const THEME_META: readonly ThemeMeta[] = [\n");
    for (id, (dark, default_mode, modes)) in &by_id {
        let modes_lit = if modes.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                modes
                    .iter()
                    .map(|m| format!("'{m}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let dm = match default_mode {
            Some(m) => format!("'{m}'"),
            None => "null".into(),
        };
        let cap = mode_capability(modes);
        out.push_str(&format!(
            "  {{ id: '{id}', dark: {dark}, modes: {modes_lit}, defaultMode: {dm}, capability: '{cap}' }},\n"
        ));
    }
    out.push_str("] as const\n\n");
    out.push_str(
        "const BY_ID: Record<string, ThemeMeta> = Object.fromEntries(\n  THEME_META.map((t) => [t.id, t]),\n)\n\n",
    );
    out.push_str(
        "export function themeMeta(id: string): ThemeMeta | undefined {\n  return BY_ID[id]\n}\n\n",
    );
    out.push_str("/** Capability for a theme id (defaults to both when unknown). */\n");
    out.push_str("export function modeCapability(id: string): ModeCapability {\n");
    out.push_str("  return themeMeta(id)?.capability ?? 'both'\n");
    out.push_str("}\n");
    out
}

fn write_with_trailing_newline(path: &Path, body: &str) -> Result<()> {
    let mut out = body.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(path, out).with_context(|| format!("write {}", path.display()))
}

/// Match JS: `.replace(/\n{3,}/g, '\n\n')`. Operates on char boundaries so
/// multi-byte UTF-8 in comments/chrome is preserved.
fn collapse_blank_lines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' {
            let mut n = 1usize;
            while chars.peek() == Some(&'\n') {
                chars.next();
                n += 1;
            }
            let keep = if n >= 3 { 2 } else { n };
            for _ in 0..keep {
                out.push('\n');
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn is_token_leaf(node: &Value) -> bool {
    node.as_object()
        .map(|o| o.contains_key("$value"))
        .unwrap_or(false)
}

fn is_group(node: &Value) -> bool {
    node.as_object()
        .map(|o| !o.contains_key("$value"))
        .unwrap_or(false)
}

/// Generator-owned public short name for a DTCG path.
///
/// Design identity is the path; CSS/Style public names (`primary`, `chip`, …)
/// are derived here — not stamped as `$extensions.*.cssName` in the model.
/// Rules reproduce the historical cssName map (bit-identical).
pub fn public_name(path: &str) -> String {
    if path == "component.error-boundary.icon-fg" {
        return "error-boundary-icon".into();
    }
    if path == "radius.default" {
        return "radius".into();
    }
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
        return String::new();
    }
    let root = parts[0];
    if root == "component" && parts.len() >= 2 {
        let pack = parts[1];
        let rest = &parts[2..];
        if rest.is_empty() {
            return pack.to_string();
        }
        if rest.len() == 1 && rest[0] == "bg" {
            return pack.to_string();
        }
        if rest.len() == 1 && rest[0] == "fg" {
            return format!("{pack}-foreground");
        }
        if rest.len() == 1 && rest[0] == "track" && matches!(pack, "meter" | "progress" | "switch")
        {
            return pack.to_string();
        }
        let mapped: Vec<String> = rest
            .iter()
            .map(|seg| {
                if *seg == "fg" {
                    "foreground".to_string()
                } else if let Some(stem) = seg.strip_suffix("-fg") {
                    format!("{stem}-foreground")
                } else {
                    camel_to_kebab(seg)
                }
            })
            .collect();
        return format!("{pack}-{}", mapped.join("-"));
    }
    if root == "color" {
        let rest = &parts[1..];
        if !rest.is_empty() && rest[0] == "brandAccent" {
            if rest.len() == 2 && rest[1] == "default" {
                return "accent".into();
            }
            let tail: Vec<String> = rest[1..].iter().map(|s| camel_to_kebab(s)).collect();
            return format!("accent-{}", tail.join("-"));
        }
        if rest.len() == 2 && rest[0] == "mix" {
            let leaf = rest[1];
            if let Some(stem) = leaf.strip_suffix("Mix") {
                return format!("mix-{}", camel_to_kebab(stem));
            }
        }
        let mut segs: Vec<String> = rest.iter().map(|s| camel_to_kebab(s)).collect();
        if segs.len() >= 2 && segs.last().map(|s| s.as_str()) == Some("default") {
            segs.pop();
        }
        return segs.join("-");
    }
    if matches!(root, "typography" | "motion" | "interaction") {
        return parts[1..]
            .iter()
            .map(|s| camel_to_kebab(s))
            .collect::<Vec<_>>()
            .join("-");
    }
    if root == "control" && parts.len() >= 2 && parts[1] == "press" {
        return parts[1..]
            .iter()
            .map(|s| camel_to_kebab(s))
            .collect::<Vec<_>>()
            .join("-");
    }
    if root == "icon" && parts.len() == 2 {
        return format!("icon-size-{}", parts[1]);
    }
    parts
        .iter()
        .map(|s| camel_to_kebab(s))
        .collect::<Vec<_>>()
        .join("-")
}

fn camel_to_kebab(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + 4);
    for (i, &b) in bytes.iter().enumerate() {
        let is_upper = b.is_ascii_uppercase();
        if is_upper {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = bytes.get(i + 1).copied().unwrap_or(0);
            let prev_lower_or_digit = i > 0 && (prev.is_ascii_lowercase() || prev.is_ascii_digit());
            let next_lower = next.is_ascii_lowercase();
            let prev_upper = i > 0 && prev.is_ascii_uppercase();
            if prev_lower_or_digit || (prev_upper && next_lower) {
                out.push('-');
            }
            out.push((b as char).to_ascii_lowercase());
        } else {
            out.push(b as char);
        }
    }
    out
}

/// Flatten a DTCG document into ordered leaves.
fn flatten_tokens(doc: &Value, path_prefix: &[String], inherited_type: Option<&str>) -> Vec<Token> {
    let mut out = Vec::new();
    let Some(obj) = doc.as_object() else {
        return out;
    };
    for (key, node) in obj {
        if key.starts_with('$') {
            continue;
        }
        if !node.is_object() {
            continue;
        }
        if is_token_leaf(node) {
            let mut path_parts = path_prefix.to_vec();
            path_parts.push(key.clone());
            let path = path_parts.join(".");
            // Public short name is generator policy — ignore any leftover cssName stamps.
            let css_name = public_name(&path);
            let ty = node
                .get("$type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| inherited_type.map(|s| s.to_string()))
                .unwrap_or_else(|| "string".into());
            let value = match node.get("$value") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                Some(Value::Bool(b)) => b.to_string(),
                Some(Value::Null) => "null".into(),
                Some(other) => other.to_string(),
                None => continue,
            };
            let channel = node
                .pointer("/$extensions/sail/channel")
                .or_else(|| node.pointer("/$extensions/ship/channel"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let description = node
                .get("$description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            out.push(Token {
                path,
                css_name,
                r#type: ty,
                value,
                css_value: String::new(),
                channel,
                description,
            });
        } else if is_group(node) {
            let group_type = node
                .get("$type")
                .and_then(|v| v.as_str())
                .or(inherited_type);
            let mut next = path_prefix.to_vec();
            next.push(key.clone());
            out.extend(flatten_tokens(node, &next, group_type));
        }
    }
    out
}

fn build_name_index(tokens: &[Token]) -> NameIndex {
    let mut idx = NameIndex::default();
    for t in tokens {
        idx.path_to_css.insert(t.path.clone(), t.css_name.clone());
        idx.css_to_path.insert(t.css_name.clone(), t.path.clone());
    }
    idx
}

/// Resolve DTCG `$value` → CSS. Aliases `{path}` / `{cssName}` → `var(--cssName)`.
///
/// SPEC-D1 failure modes: dangling alias and alias cycles are hard errors
/// (never silent kebab fallback).
fn resolve_value(value: &str, name_index: &NameIndex) -> Result<String> {
    let alias = parse_alias(value);
    if let Some(ref_name) = alias {
        let css_name = resolve_alias_target(&ref_name, name_index).with_context(|| {
            format!("dangling alias `{{{ref_name}}}` — target identity does not exist in the model")
        })?;
        return Ok(format!("var(--{css_name})"));
    }
    Ok(value.to_string())
}

/// Map `{path}` / `{cssName}` to a known public cssName, or None if dangling.
fn resolve_alias_target(ref_name: &str, name_index: &NameIndex) -> Option<String> {
    if let Some(c) = name_index.path_to_css.get(ref_name) {
        return Some(c.clone());
    }
    if name_index.css_to_path.contains_key(ref_name) {
        return Some(ref_name.to_string());
    }
    None
}

/// Detect alias cycles across token `$value` edges. Hard error (SPEC-D1).
fn check_alias_cycles(tokens: &[Token]) -> Result<()> {
    let path_set: HashSet<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    let css_to_path: HashMap<&str, &str> = tokens
        .iter()
        .map(|t| (t.css_name.as_str(), t.path.as_str()))
        .collect();
    let mut edges: HashMap<String, String> = HashMap::new();
    for t in tokens {
        let Some(alias) = parse_alias(&t.value) else {
            continue;
        };
        let target = if path_set.contains(alias.as_str()) {
            alias
        } else if let Some(p) = css_to_path.get(alias.as_str()) {
            (*p).to_string()
        } else {
            continue; // dangling handled in resolve_value
        };
        edges.insert(t.path.clone(), target);
    }
    let mut visiting: HashSet<String> = HashSet::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();
    for node in edges.keys().cloned().collect::<Vec<_>>() {
        check_cycle_dfs(&node, &edges, &mut visiting, &mut visited, &mut stack)?;
    }
    Ok(())
}

fn check_cycle_dfs(
    node: &str,
    edges: &HashMap<String, String>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) -> Result<()> {
    if visited.contains(node) {
        return Ok(());
    }
    if !visiting.insert(node.to_string()) {
        let cycle_start = stack.iter().position(|n| n == node).unwrap_or(0);
        let mut cycle: Vec<&str> = stack[cycle_start..].iter().map(|s| s.as_str()).collect();
        cycle.push(node);
        anyhow::bail!("alias cycle detected: {}", cycle.join(" → "));
    }
    stack.push(node.to_string());
    if let Some(next) = edges.get(node) {
        check_cycle_dfs(next, edges, visiting, visited, stack)?;
    }
    stack.pop();
    visiting.remove(node);
    visited.insert(node.to_string());
    Ok(())
}

fn parse_alias(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'{' || bytes[bytes.len() - 1] != b'}' {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    if inner.is_empty() {
        return None;
    }
    if !inner
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'-')
    {
        return None;
    }
    Some(inner.to_string())
}

fn load_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    // Allow JSONC in project/catalog DTCG (comments + trailing commas) so
    // theme SSOT edits match sail.config authoring comfort.
    let stripped = strip_jsonc_comments(&text);
    serde_json::from_str(&stripped).with_context(|| format!("parse JSON {}", path.display()))
}

fn layer_meta(doc: &Value, file: &str, fallback_layer: &str) -> LayerMeta {
    let ext = doc
        .pointer("/$extensions/sail")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let description = doc
        .get("$description")
        .and_then(|v| v.as_str())
        .unwrap_or(file)
        .to_string();
    let selector = ext
        .get("selector")
        .and_then(|v| v.as_str())
        .unwrap_or(":root")
        .to_string();
    let media = ext
        .get("media")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let layer = ext
        .get("layer")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_layer)
        .to_string();
    let theme_id = ext
        .get("themeId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let dark = ext.get("dark").and_then(|v| v.as_bool()).unwrap_or(false);
    let default_mode = ext
        .get("defaultMode")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    LayerMeta {
        file: file.to_string(),
        description,
        selector,
        media,
        layer,
        theme_id,
        dark,
        mode: None,
        default_mode,
    }
}

fn load_base_layers(model_dir: &Path) -> Result<(Vec<Layer>, NameIndex, Vec<Token>)> {
    let mut files: Vec<String> = fs::read_dir(model_dir)
        .with_context(|| format!("read dir {}", model_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|f| f.ends_with(".json") && f != "compat-aliases.json" && f != "products.json")
        .collect();
    files.sort();

    let base_file = if files.iter().any(|f| f == "base.json") {
        "base.json".to_string()
    } else {
        files
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("no layer JSON files in {}", model_dir.display()))?
    };
    let mut base_doc = load_json(&model_dir.join(&base_file))?;
    // Component token packs live under model/components/*.json and merge into
    // the base layer (paths like component.card.bg → public cssNames).
    merge_component_sources(model_dir, &mut base_doc)?;
    let base_tokens_raw = flatten_tokens(&base_doc, &[], None);
    check_alias_cycles(&base_tokens_raw)?;
    let name_index = build_name_index(&base_tokens_raw);

    let mut layers = Vec::new();
    for file in &files {
        let mut doc = load_json(&model_dir.join(file))?;
        let fallback = file.trim_end_matches(".json");
        let meta = layer_meta(&doc, file, fallback);
        if meta.layer == "base" {
            // Use the merged base document (includes components/).
            doc = base_doc.clone();
        }
        let raw = flatten_tokens(&doc, &[], None);
        let mut tokens = Vec::with_capacity(raw.len());
        for mut t in raw {
            t.css_value = resolve_value(&t.value, &name_index)?;
            tokens.push(t);
        }
        layers.push(Layer { meta, tokens });
    }

    let base_tokens = layers
        .iter()
        .find(|l| l.meta.layer == "base")
        .map(|l| l.tokens.clone())
        .or_else(|| layers.first().map(|l| l.tokens.clone()))
        .unwrap_or_default();

    Ok((layers, name_index, base_tokens))
}

/// Deep-merge `model/components/*.json` into the base DTCG document.
/// Each file may define a `component` group (or any top-level groups).
fn merge_component_sources(model_dir: &Path, base_doc: &mut Value) -> Result<()> {
    let components_dir = model_dir.join("components");
    if !components_dir.is_dir() {
        return Ok(());
    }
    let mut files: Vec<String> = fs::read_dir(&components_dir)
        .with_context(|| format!("read dir {}", components_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|f| f.ends_with(".json"))
        .collect();
    files.sort();
    for file in files {
        let path = components_dir.join(&file);
        let doc = load_json(&path)?;
        deep_merge_objects(base_doc, &doc)
            .with_context(|| format!("merge component source {} into base", path.display()))?;
    }
    Ok(())
}

/// Merge `overlay` into `base` for JSON objects (DTCG groups). Non-object
/// leaves in `overlay` replace. `$`-metadata keys from overlay win when present.
fn deep_merge_objects(base: &mut Value, overlay: &Value) -> Result<()> {
    let Some(over_map) = overlay.as_object() else {
        *base = overlay.clone();
        return Ok(());
    };
    if !base.is_object() {
        *base = Value::Object(serde_json::Map::new());
    }
    let base_map = base
        .as_object_mut()
        .ok_or_else(|| anyhow!("base document root must be a JSON object"))?;
    for (k, v) in over_map {
        if k.starts_with('$') {
            base_map.insert(k.clone(), v.clone());
            continue;
        }
        match base_map.get_mut(k) {
            Some(existing) if existing.is_object() && v.is_object() => {
                deep_merge_objects(existing, v)?;
            }
            _ => {
                base_map.insert(k.clone(), v.clone());
            }
        }
    }
    Ok(())
}

/// Composite key so modes-in-file layers do not collapse on theme id.
fn theme_emit_key(layer: &Layer) -> String {
    let id = layer
        .meta
        .theme_id
        .as_deref()
        .unwrap_or(layer.meta.file.as_str());
    match &layer.meta.mode {
        Some(mode) => format!("{id}#{mode}"),
        None => id.to_string(),
    }
}

/// Stable mode order: light, dark, then remaining alpha (matches load-time rank).
fn mode_rank(mode: &str) -> u8 {
    match mode {
        "light" => 0,
        "dark" => 1,
        _ => 2,
    }
}

/// Sort emit keys so `id#light` precedes `id#dark` (not lexicographic).
fn sort_theme_emit_keys(keys: &mut [String]) {
    keys.sort_by(|a, b| {
        let (a_id, a_mode) = a
            .split_once('#')
            .map(|(i, m)| (i, Some(m)))
            .unwrap_or((a.as_str(), None));
        let (b_id, b_mode) = b
            .split_once('#')
            .map(|(i, m)| (i, Some(m)))
            .unwrap_or((b.as_str(), None));
        a_id.cmp(b_id).then_with(|| match (a_mode, b_mode) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(am), Some(bm)) => mode_rank(am).cmp(&mode_rank(bm)).then_with(|| am.cmp(bm)),
        })
    });
}

/// Build the theme emit set: filtered catalog + project themes (project wins).
fn load_emit_themes(
    model_dir: &Path,
    name_index: &NameIndex,
    opts: &GenerateOptions,
) -> Result<Vec<Layer>> {
    let mut catalog = load_theme_layers_from_dir(
        &model_dir.join("themes"),
        name_index,
        /* label_prefix */ "themes",
    )?;
    if let Some(enabled) = &opts.enabled_catalog {
        let allow: HashSet<&str> = enabled.iter().map(String::as_str).collect();
        catalog.retain(|t| {
            t.meta
                .theme_id
                .as_deref()
                .map(|id| allow.contains(id))
                .unwrap_or(false)
        });
    }

    // Index by emit key (id or id#mode) for override; preserve THEME_ORDER.
    let mut by_key: BTreeMap<String, Layer> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for layer in catalog {
        let key = theme_emit_key(&layer);
        if !by_key.contains_key(&key) {
            order.push(key.clone());
        }
        by_key.insert(key, layer);
    }

    for dir in &opts.project_theme_dirs {
        if !dir.is_dir() {
            continue;
        }
        let project = load_theme_layers_from_dir(dir, name_index, "project-themes")?;
        for layer in project {
            let key = theme_emit_key(&layer);
            if !by_key.contains_key(&key) {
                order.push(key.clone());
            }
            by_key.insert(key, layer);
        }
    }

    // Stable order: THEME_ORDER theme ids first (all their modes), then extras.
    let mut ordered: Vec<Layer> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for id in THEME_ORDER {
        // Collect keys for this theme id (bare + modes).
        let mut keys: Vec<String> = by_key
            .keys()
            .filter(|k| *k == id || k.starts_with(&format!("{id}#")))
            .cloned()
            .collect();
        sort_theme_emit_keys(&mut keys);
        for key in keys {
            if let Some(layer) = by_key.remove(&key) {
                seen.insert(key);
                ordered.push(layer);
            }
        }
    }
    for key in order {
        if seen.contains(&key) {
            continue;
        }
        if let Some(layer) = by_key.remove(&key) {
            ordered.push(layer);
        }
    }
    for (_, layer) in by_key {
        ordered.push(layer);
    }
    Ok(ordered)
}

fn load_theme_layers_from_dir(
    themes_dir: &Path,
    name_index: &NameIndex,
    label_prefix: &str,
) -> Result<Vec<Layer>> {
    if !themes_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<String> = fs::read_dir(themes_dir)
        .with_context(|| format!("read dir {}", themes_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|f| f.ends_with(".json"))
        .collect();
    files.sort();

    let mut themes = Vec::new();
    for file in files {
        let doc = load_json(&themes_dir.join(&file))?;
        let fallback = file.trim_end_matches(".json");
        let label = format!("{label_prefix}/{file}");

        // Envelope first so modes-in-file / twin-id errors stay specific.
        validate_theme_doc_schema(&doc, &label)?;

        // Modes-in-file only — one theme id, mode blocks under `modes`.
        // No twin-id / flat-theme dual path.
        // Single-mode themes are allowed (e.g. glass light-only, synthwave dark-only)
        // via a single key under `modes` and/or `$extensions.sail.supportedModes`.
        let modes = doc
            .get("modes")
            .and_then(|v| v.as_object())
            .filter(|m| !m.is_empty());
        let Some(modes) = modes else {
            bail!(
                "theme {label}: modes-in-file required (non-empty object `modes`). \
                 Twin light/dark theme ids are not supported. \
                 Single-mode themes declare one key under modes (and optional \
                 $extensions.sail.supportedModes)."
            );
        };

        // Resolve declared mode set: keys under `modes`, optionally intersected
        // with `$extensions.sail.supportedModes` (must be a subset of modes keys).
        let supported_ext: Option<Vec<String>> = doc
            .pointer("/$extensions/sail/supportedModes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str())
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .filter(|v: &Vec<String>| !v.is_empty());

        let mut mode_names: Vec<String> = if let Some(ref supported) = supported_ext {
            for m in supported {
                if !modes.contains_key(m) {
                    bail!(
                        "theme {label}: $extensions.sail.supportedModes includes {m:?} \
                         but modes.{m} is missing (found modes: {:?})",
                        modes.keys().cloned().collect::<Vec<_>>()
                    );
                }
            }
            supported.clone()
        } else {
            modes.keys().cloned().collect()
        };

        if mode_names.is_empty() {
            bail!("theme {label}: no modes declared after supportedModes filter");
        }

        // Only light|dark (and future named modes) — reject unknown paint modes
        // that would never apply via data-mode.
        for m in &mode_names {
            if m != "light" && m != "dark" {
                bail!(
                    "theme {label}: unsupported mode name {m:?} \
                     (only \"light\" and \"dark\" are valid data-mode values)"
                );
            }
        }

        // Path parity among **declared** modes only (single-mode ⇒ no check).
        // Multi-mode: every mode must declare the same $value leaves so
        // data-mode switches cannot leave sticky CSS vars from another mode.
        if mode_names.len() >= 2 {
            let path_sets: Vec<(String, std::collections::BTreeSet<String>)> = mode_names
                .iter()
                .map(|name| {
                    let paths: std::collections::BTreeSet<String> = flatten_tokens(
                        modes.get(name).unwrap_or(&Value::Object(Map::new())),
                        &[],
                        None,
                    )
                    .into_iter()
                    .map(|t| t.path)
                    .collect();
                    (name.clone(), paths)
                })
                .collect();
            let reference = &path_sets[0].1;
            for (name, paths) in path_sets.iter().skip(1) {
                if paths != reference {
                    let only_ref: Vec<_> = reference.difference(paths).cloned().collect();
                    let only_this: Vec<_> = paths.difference(reference).cloned().collect();
                    bail!(
                        "theme {label}: mode path parity required among declared modes \
                         (same $value leaves). mismatch {ref_mode} vs {name}: \
                         only in {ref_mode} ({n_r}): {only_ref:?}; \
                         only in {name} ({n_t}): {only_this:?}",
                        ref_mode = path_sets[0].0,
                        n_r = only_ref.len(),
                        n_t = only_this.len(),
                    );
                }
            }
        }

        let mut base_meta = layer_meta(&doc, &label, fallback);
        if base_meta.theme_id.is_none() {
            base_meta.theme_id = Some(fallback.to_string());
        }
        let id = base_meta
            .theme_id
            .clone()
            .unwrap_or_else(|| fallback.to_string());
        // Reject residual twin id filenames / themeIds.
        if id.ends_with("-light") || id.ends_with("-dark") {
            bail!(
                "theme {label}: twin theme id '{id}' is not allowed — use one id with modes.light/modes.dark"
            );
        }
        let default_mode = base_meta.default_mode.clone().or_else(|| {
            if mode_names.contains(&"light".to_string()) {
                Some("light".into())
            } else {
                mode_names.first().cloned()
            }
        });
        // defaultMode must be a declared mode
        if let Some(ref dm) = default_mode {
            if !mode_names.iter().any(|m| m == dm) {
                bail!(
                    "theme {label}: defaultMode {dm:?} is not among declared modes {mode_names:?}"
                );
            }
        }
        // Paint gate after envelope / twin-id / mode-set (slots, aliases, contrast).
        crate::theme_gate::validate_theme_gate(&doc, &label)?;

        let multi_mode = mode_names.len() >= 2;

        // Stable mode order: light, dark, then remaining alpha.
        mode_names.sort_by(|a, b| {
            let rank = |m: &str| match m {
                "light" => 0,
                "dark" => 1,
                _ => 2,
            };
            rank(a).cmp(&rank(b)).then_with(|| a.cmp(b))
        });
        for mode_name in mode_names {
            let mode_doc = modes
                .get(&mode_name)
                .cloned()
                .unwrap_or(Value::Object(Map::new()));
            let mode_ext_dark = mode_doc
                .pointer("/$extensions/sail/dark")
                .and_then(|v| v.as_bool())
                .unwrap_or(mode_name == "dark");
            let mut meta = base_meta.clone();
            meta.file = format!("{label}#mode={mode_name}");
            meta.mode = Some(mode_name.clone());
            meta.default_mode = default_mode.clone();
            meta.dark = mode_ext_dark;
            meta.layer = "theme".into();
            // Selector family (hard cutover — theme attr only):
            //   • `:root[theme=…]` — app product theme on documentElement
            //   • `[theme=…]` — scoped hosts
            // Nested hosts without data-mode inherit document mode via
            // `:root[data-mode=X] [theme=id]:not([data-mode])` — no JS.
            // Do not emit a global `[theme=id]:not([data-mode])` on the
            // default-mode block (that pinned nested hosts to default).
            let is_default = default_mode.as_deref() == Some(mode_name.as_str());
            meta.selector = if is_default && !multi_mode {
                // Only mode: a nested host without data-mode always gets these tokens
                // (paper island inside a dark document).
                format!(
                    ":root[theme='{id}'][data-mode='{mode_name}'],\n\
                     :root[theme='{id}']:not([data-mode]),\n\
                     [theme='{id}'][data-mode='{mode_name}'],\n\
                     [theme='{id}']:not([data-mode])"
                )
            } else if is_default {
                format!(
                    ":root[theme='{id}'][data-mode='{mode_name}'],\n\
                     :root[theme='{id}']:not([data-mode]),\n\
                     [theme='{id}'][data-mode='{mode_name}'],\n\
                     :root[data-mode='{mode_name}'] [theme='{id}']:not([data-mode]),\n\
                     :root:not([data-mode]) [theme='{id}']:not([data-mode])"
                )
            } else {
                format!(
                    ":root[theme='{id}'][data-mode='{mode_name}'],\n\
                     [theme='{id}'][data-mode='{mode_name}'],\n\
                     :root[data-mode='{mode_name}'] [theme='{id}']:not([data-mode])"
                )
            };
            let raw = flatten_tokens(&mode_doc, &[], None);
            let mut tokens = Vec::with_capacity(raw.len());
            for mut t in raw {
                t.css_value = resolve_value(&t.value, name_index)?;
                tokens.push(t);
            }
            themes.push(Layer { meta, tokens });
        }
    }
    Ok(themes)
}

/// Load `theme` from `project_dir/sail.config.jsonc` or `.json` (T0).
///
/// - Missing file / missing `theme` section → [`ThemeConfig::default`] (thin).
/// - Unparseable body → **error** (fail loud).
pub fn load_theme_config(project_dir: &Path) -> Result<ThemeConfig> {
    let Some((path, text)) = read_sail_config_text(project_dir) else {
        return Ok(ThemeConfig::default());
    };
    parse_theme_config(&text).with_context(|| format!("parse theme config {}", path.display()))
}

/// Parse a `sail.config` body (JSON or JSONC) into [`ThemeConfig`].
///
/// Unparseable JSONC/JSON is an error (config-loud). Wrong-typed `enabled`
/// (not array and not the string `"all"`) is a **hard error**.
pub fn parse_theme_config(text: &str) -> Result<ThemeConfig> {
    let stripped = strip_jsonc_comments(text);
    let value: Value = serde_json::from_str(&stripped)
        .map_err(|e| anyhow!("invalid JSON/JSONC in sail.config: {e}"))?;
    let Some(theme) = value.get("theme") else {
        return Ok(ThemeConfig::default());
    };
    if !theme.is_object() {
        bail!("theme config: \"theme\" must be an object");
    }
    let default = theme
        .get("default")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let default_mode = theme
        .get("defaultMode")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let (enabled, enabled_key_present) = match theme.get("enabled") {
        // Omit → default Sail theme (ship), not an empty app.
        None => (CatalogAllowlist::Default, false),
        Some(v) => {
            if let Some(s) = v.as_str() {
                if s == "all" {
                    (CatalogAllowlist::All, true)
                } else {
                    bail!(
                        "theme config: \"enabled\" string must be the literal \"all\" \
                         (got {s:?}); use an array of catalog ids for a thin allowlist"
                    );
                }
            } else if let Some(arr) = v.as_array() {
                if arr.iter().any(|item| !item.is_string()) {
                    bail!(
                        "theme config: \"enabled\" array elements must all be strings \
                         (catalog theme ids)"
                    );
                }
                let ids: Vec<String> = arr
                    .iter()
                    .filter_map(|x| x.as_str())
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect();
                // Explicit [] = opt out of catalog; omit key keeps default ship.
                if ids.is_empty() {
                    (CatalogAllowlist::Empty, true)
                } else {
                    (CatalogAllowlist::Ids(ids), true)
                }
            } else {
                bail!(
                    "theme config: \"enabled\" must be an array of catalog ids or the \
                     string \"all\" (got {})",
                    v
                );
            }
        }
    };
    Ok(ThemeConfig {
        default,
        default_mode,
        enabled,
        enabled_key_present,
    })
}

/// Dual-read `sail.config.jsonc` then `sail.config.json` under `project_dir`.
/// Returns `(path, text)` for the first file found (jsonc wins when both exist).
fn read_sail_config_text(project_dir: &Path) -> Option<(PathBuf, String)> {
    for name in ["sail.config.jsonc", "sail.config.json"] {
        let path = project_dir.join(name);
        if let Ok(text) = fs::read_to_string(&path) {
            return Some((path, text));
        }
    }
    None
}

/// Theme id shape used by scaffold / config: `^[a-z][a-z0-9-]*$`.
pub fn validate_theme_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("theme id is required");
    }
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        bail!("theme id is required");
    };
    if !first.is_ascii_lowercase() {
        bail!("theme id must start with a lowercase letter — got {id:?}");
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        bail!("theme id must match /^[a-z][a-z0-9-]*$/ — got {id:?}");
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        bail!("theme id must not contain path separators — got {id:?}");
    }
    Ok(())
}

/// Map declared paint modes → capability (`light` | `dark` | `both`).
pub fn mode_capability(modes: &[String]) -> &'static str {
    let has_light = modes.iter().any(|m| m == "light");
    let has_dark = modes.iter().any(|m| m == "dark");
    match (has_light, has_dark) {
        (true, true) => "both",
        (false, true) => "dark",
        _ => "light",
    }
}

fn closest_schema_key<'a>(unknown: &str, valid: &[&'a str]) -> Option<&'a str> {
    let mut best: Option<(usize, &'a str)> = None;
    for &k in valid {
        let d = schema_levenshtein(unknown, k);
        let max_d = (k.len().max(unknown.len()) / 3).max(2);
        if d <= max_d && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, k));
        }
    }
    best.map(|(_, s)| s)
}

fn schema_levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

fn unknown_key_error(label: &str, where_: &str, key: &str, valid: &[&str]) -> anyhow::Error {
    let valid_list = valid.join(", ");
    match closest_schema_key(key, valid) {
        Some(s) => anyhow!(
            "theme {label}: unknown {where_} key \"{key}\" \
             (closest valid key: \"{s}\"; valid: {valid_list})"
        ),
        None => anyhow!("theme {label}: unknown {where_} key \"{key}\" (valid: {valid_list})"),
    }
}

/// THEMES-1 schema lock: refuse out-of-schema theme documents.
///
/// Checks root keys, `$extensions.sail` keys, paint mode names, and mode-body
/// scale/component group keys. Nested DTCG token leaves are not re-enumerated
/// (registry paths own that surface); this locks the **authoring envelope**.
pub fn validate_theme_doc_schema(doc: &Value, label: &str) -> Result<()> {
    let Some(obj) = doc.as_object() else {
        bail!("theme {label}: root must be a JSON object");
    };

    for key in obj.keys() {
        if !THEME_ROOT_KEYS.contains(&key.as_str()) {
            return Err(unknown_key_error(label, "root", key, THEME_ROOT_KEYS));
        }
    }

    if let Some(ext) = obj.get("$extensions") {
        let Some(ext_obj) = ext.as_object() else {
            bail!("theme {label}: $extensions must be an object");
        };
        for key in ext_obj.keys() {
            if key != "sail" {
                return Err(unknown_key_error(label, "$extensions", key, &["sail"]));
            }
        }
        if let Some(ship) = ext_obj.get("sail") {
            let Some(ship_obj) = ship.as_object() else {
                bail!("theme {label}: $extensions.sail must be an object");
            };
            for key in ship_obj.keys() {
                if !THEME_SAIL_KEYS.contains(&key.as_str()) {
                    return Err(unknown_key_error(
                        label,
                        "$extensions.sail",
                        key,
                        THEME_SAIL_KEYS,
                    ));
                }
            }
            if let Some(dm) = ship_obj.get("defaultMode").and_then(|v| v.as_str()) {
                if !THEME_PAINT_MODES.contains(&dm) {
                    return Err(unknown_key_error(
                        label,
                        "defaultMode",
                        dm,
                        THEME_PAINT_MODES,
                    ));
                }
            }
            if let Some(arr) = ship_obj.get("supportedModes").and_then(|v| v.as_array()) {
                for item in arr {
                    let Some(m) = item.as_str() else {
                        bail!("theme {label}: supportedModes entries must be strings");
                    };
                    if !THEME_PAINT_MODES.contains(&m) {
                        return Err(unknown_key_error(
                            label,
                            "supportedModes",
                            m,
                            THEME_PAINT_MODES,
                        ));
                    }
                }
            }
            if let Some(layer) = ship_obj.get("layer").and_then(|v| v.as_str()) {
                if layer != "theme" {
                    bail!(
                        "theme {label}: $extensions.sail.layer must be \"theme\" (got {layer:?})"
                    );
                }
            }
            if let Some(id) = ship_obj.get("themeId").and_then(|v| v.as_str()) {
                // Catalog allows digit-leading ids (e.g. "8bit"); fork/scaffold
                // still use the stricter validate_theme_id for new ids.
                if id.is_empty()
                    || id.contains("..")
                    || id.contains('/')
                    || id.contains('\\')
                    || !id
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                {
                    bail!(
                        "theme {label}: invalid $extensions.sail.themeId {id:?} \
                         (expected /^[a-z0-9][a-z0-9-]*$/)"
                    );
                }
            }
            if let Some(d) = ship_obj.get("density").and_then(|v| v.as_str()) {
                if d != "compact" && d != "default" && d != "spacious" {
                    bail!(
                        "theme {label}: $extensions.sail.density must be \
                         compact|default|spacious (got {d:?})"
                    );
                }
            }
        }
    }

    if let Some(modes) = obj.get("modes") {
        let Some(modes_obj) = modes.as_object() else {
            bail!("theme {label}: modes must be an object");
        };
        for (mode_name, body) in modes_obj {
            if !THEME_PAINT_MODES.contains(&mode_name.as_str()) {
                return Err(unknown_key_error(
                    label,
                    "modes",
                    mode_name,
                    THEME_PAINT_MODES,
                ));
            }
            let Some(body_obj) = body.as_object() else {
                bail!("theme {label}: modes.{mode_name} must be an object");
            };
            for key in body_obj.keys() {
                if !THEME_MODE_GROUP_KEYS.contains(&key.as_str()) {
                    return Err(unknown_key_error(
                        label,
                        &format!("modes.{mode_name}"),
                        key,
                        THEME_MODE_GROUP_KEYS,
                    ));
                }
            }
        }
    }

    Ok(())
}

/// True if `path` resolves under `root` (after canonicalize). Used to block
/// path-escape on scaffold copy destinations/sources.
fn path_is_under(path: &Path, root: &Path) -> Result<bool> {
    let root =
        fs::canonicalize(root).with_context(|| format!("canonicalize {}", root.display()))?;
    let path = if path.exists() {
        fs::canonicalize(path).with_context(|| format!("canonicalize {}", path.display()))?
    } else {
        // Parent must exist (we create dest dir before this check).
        let parent = path
            .parent()
            .ok_or_else(|| anyhow!("path has no parent: {}", path.display()))?;
        let parent = fs::canonicalize(parent)
            .with_context(|| format!("canonicalize {}", parent.display()))?;
        parent.join(path.file_name().unwrap_or_default())
    };
    Ok(path.starts_with(&root))
}

/// Std-only JSONC stripper (// + /* */ comments, trailing commas). Mirrors
/// `deck_config::strip_jsonc_comments` so sail-compiler stays free of that dep.
fn strip_jsonc_comments(input: &str) -> String {
    let mut stage = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if in_string {
            stage.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            stage.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc == '\n' {
                            stage.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }
        stage.push(c);
    }
    let src = stage.as_bytes();
    let mut cleaned: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    let mut in_string2 = false;
    let mut escape2 = false;
    while i < src.len() {
        let b = src[i];
        if in_string2 {
            cleaned.push(b);
            if escape2 {
                escape2 = false;
            } else if b == b'\\' {
                escape2 = true;
            } else if b == b'"' {
                in_string2 = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string2 = true;
            cleaned.push(b);
            i += 1;
            continue;
        }
        if b == b',' {
            let mut j = i + 1;
            while j < src.len()
                && (src[j] == b' ' || src[j] == b'\t' || src[j] == b'\r' || src[j] == b'\n')
            {
                j += 1;
            }
            if j < src.len() && (src[j] == b']' || src[j] == b'}') {
                i += 1;
                continue;
            }
        }
        cleaned.push(b);
        i += 1;
    }
    String::from_utf8(cleaned)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

/// Copy one Sail catalog theme into `<project>/themes/<id>.json`.
///
/// Uses `theme_id` (default [`DEFAULT_PROJECT_THEME_ID`]). When
/// `catalog_themes_dir` is `None`, sources the baked catalog (no monorepo
/// tree required). Does not overwrite an existing project file (modern or
/// legacy path). Returns the destination path.
pub fn scaffold_default_project_theme(
    project_dir: &Path,
    catalog_themes_dir: Option<&Path>,
    theme_id: Option<&str>,
) -> Result<PathBuf> {
    let id = theme_id
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_PROJECT_THEME_ID);
    validate_theme_id(id)?;

    // Already present under modern or legacy project themes — leave alone.
    for existing in project_theme_file_candidates(project_dir, id) {
        if existing.is_file() {
            return Ok(existing);
        }
    }

    let dest_dir = project_themes_write_dir(project_dir);
    fs::create_dir_all(&dest_dir).with_context(|| format!("mkdir {}", dest_dir.display()))?;
    let dest = dest_dir.join(format!("{id}.json"));
    if !path_is_under(&dest, &dest_dir)? {
        bail!("project theme path escapes themes dir: {}", dest.display());
    }

    if let Some(catalog_dir) = catalog_themes_dir {
        if !catalog_dir.is_dir() {
            bail!("catalog themes dir not found: {}", catalog_dir.display());
        }
        let src = catalog_dir.join(format!("{id}.json"));
        if !src.is_file() {
            bail!(
                "catalog theme not found: {} (baked catalog or model/themes/{id}.json)",
                src.display()
            );
        }
        if !path_is_under(&src, catalog_dir)? {
            bail!("catalog theme path escapes catalog dir: {}", src.display());
        }
        fs::copy(&src, &dest)
            .with_context(|| format!("copy {} → {}", src.display(), dest.display()))?;
        return Ok(dest);
    }

    let Some(body) = baked_catalog_theme_json(id) else {
        bail!("catalog theme not found in baked catalog: {id}");
    };
    fs::write(&dest, body).with_context(|| format!("write {}", dest.display()))?;
    Ok(dest)
}

/// Write a thin `sail.config.jsonc` theme section when the project has no
/// sail.config yet. Does not overwrite an existing config file.
///
/// Thin `enabled: [id]` + `defaultMode: "light"` — app delivery ships only
/// that catalog id (+ project themes). Package generate stays full catalog
/// ([`options_for_shared_package`]).
pub fn scaffold_theme_config(project_dir: &Path, theme_id: &str) -> Result<Option<PathBuf>> {
    validate_theme_id(theme_id)?;
    for name in ["sail.config.jsonc", "sail.config.json"] {
        if project_dir.join(name).is_file() {
            return Ok(None);
        }
    }
    let path = project_dir.join("sail.config.jsonc");
    let body = format!(
        "{{\n\
         \x20\x20// Sail app config — theme (product) + deck (bundler/serve).\n\
         \x20\x20// `default` / `defaultMode` drive first paint (theme / data-mode).\n\
         \x20\x20// `enabled` is the catalog allowlist for app CSS emit:\n\
         \x20\x20//   omit → default theme ship; [] → no catalog; [\"id\"] → those; \"all\" → full.\n\
         \x20\x20// Project themes under themes/ always emit. Package generate\n\
         \x20\x20// (shared @ship/tokens, monorepo only) emits full catalog + overrides.\n\
         \x20\x20// Serve keys nest under `deck` only (SPEC-SAIL-CONFIG-1) — never top-level.\n\
         \x20\x20\"theme\": {{\n\
         \x20\x20\x20\x20\"default\": \"{theme_id}\",\n\
         \x20\x20\x20\x20\"defaultMode\": \"light\",\n\
         \x20\x20\x20\x20\"enabled\": [\"{theme_id}\"]\n\
         \x20\x20}},\n\
         \x20\x20\"deck\": {{\n\
         \x20\x20\x20\x20\"render\": \"csr\"\n\
         \x20\x20}}\n\
         }}\n"
    );
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(Some(path))
}

/// App-local delivery directory for filtered `themes.css` (T1 B1).
///
/// `<project>/.deck/themes/` — a **foreign top-level dir** under the SPA out tree.
/// `publish_build(..., prune=true)` only prunes top-level **files** not in the
/// stage manifest and contents of build-owned dirs (e.g. `assets/`). It does
/// **not** remove foreign top-level dirs, so this path survives non-theme HMR
/// rebuilds (unlike a bare `.deck/themes.css` top-level file, which was pruned).
///
/// Bundler aliases `@ship/tokens/themes.css` → [`app_themes_css_path`] when present.
pub fn app_themes_out_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(".deck").join("themes")
}

/// App-local themes.css path (T1 B1): `<project>/.deck/themes/themes.css`.
pub fn app_themes_css_path(project_dir: &Path) -> PathBuf {
    app_themes_out_dir(project_dir).join("themes.css")
}

/// List theme JSON stems under `dir` (sorted). Missing dir → empty.
pub fn list_theme_ids_in_dir(dir: &Path) -> Vec<String> {
    let mut ids = Vec::new();
    let Ok(rd) = fs::read_dir(dir) else {
        return ids;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        if !path
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| x.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !stem.is_empty() && !stem.starts_with('.') {
            ids.push(stem.to_string());
        }
    }
    ids.sort();
    ids
}

/// Effective theme set for a project (T0/T1 algorithm).
///
/// ```text
/// project_ids = stems of project themes/*.json (+ legacy tokens/themes)
/// catalog_allow = all | list | default [ship] | explicit []
/// effective_catalog = catalog_allow ∩ catalog (disk model or baked)
/// emit_set = project ∪ effective_catalog (project wins on id at emit)
/// ```
///
/// `workspace_root` is optional: when the monorepo model tree is present it is
/// preferred for live catalog edits; otherwise the baked catalog is used.
pub fn effective_theme_ids(workspace_root: &Path, project_dir: &Path) -> Result<EffectiveThemeIds> {
    effective_theme_ids_opt(Some(workspace_root), project_dir)
}

/// Like [`effective_theme_ids`] with optional workspace (external apps pass `None`).
pub fn effective_theme_ids_opt(
    workspace_root: Option<&Path>,
    project_dir: &Path,
) -> Result<EffectiveThemeIds> {
    let cfg = load_theme_config(project_dir)?;
    let project = list_project_theme_ids(project_dir);
    let catalog_all = catalog_theme_ids(workspace_root)?;
    let catalog = match &cfg.enabled {
        CatalogAllowlist::Default => catalog_all
            .into_iter()
            .filter(|id| id == DEFAULT_PROJECT_THEME_ID)
            .collect(),
        CatalogAllowlist::Empty => Vec::new(),
        CatalogAllowlist::All => catalog_all,
        CatalogAllowlist::Ids(ids) => {
            let allow: HashSet<&str> = ids.iter().map(String::as_str).collect();
            catalog_all
                .into_iter()
                .filter(|id| allow.contains(id.as_str()))
                .collect()
        }
    };
    Ok(EffectiveThemeIds { project, catalog })
}

/// Catalog theme ids from disk model (when present) or the baked catalog.
pub fn catalog_theme_ids(workspace_root: Option<&Path>) -> Result<Vec<String>> {
    if let Some(ws) = workspace_root {
        let disk = ws.join(MONOREPO_MODEL_REL).join("themes");
        if disk.is_dir() {
            return Ok(list_theme_ids_in_dir(&disk));
        }
    }
    Ok(baked_catalog_theme_ids())
}

/// Config-loud validation: `default` ∈ effective set; `defaultMode` exists on
/// the theme when the DTCG file is loadable.
pub fn validate_theme_config(
    workspace_root: &Path,
    project_dir: &Path,
    cfg: &ThemeConfig,
) -> Result<()> {
    validate_theme_config_opt(Some(workspace_root), project_dir, cfg)
}

/// Like [`validate_theme_config`] with optional workspace (baked catalog ok).
pub fn validate_theme_config_opt(
    workspace_root: Option<&Path>,
    project_dir: &Path,
    cfg: &ThemeConfig,
) -> Result<()> {
    let effective = effective_theme_ids_opt(workspace_root, project_dir)?;
    let all = effective.all_ids();
    if let Some(default) = cfg.default.as_deref().filter(|s| !s.is_empty()) {
        if !all.iter().any(|id| id == default) {
            if all.is_empty() {
                bail!(
                    "theme config: default theme {default:?} is not in the effective set \
                     (no project themes and empty catalog allowlist). \
                     Scaffold a theme (`sail scaffold theme`) or set theme.enabled \
                     to include a catalog id / \"all\""
                );
            }
            bail!(
                "theme config: default theme {default:?} is not in the effective set \
                 (project: {:?}, catalog: {:?}). Add a project theme file, include the \
                 id in theme.enabled, or change theme.default",
                effective.project,
                effective.catalog
            );
        }
        // defaultMode must exist on the theme when we can load modes.
        if let Some(mode) = cfg.default_mode.as_deref().filter(|s| !s.is_empty()) {
            let modes = load_theme_mode_names(workspace_root, project_dir, default)?;
            if !modes.is_empty() && !modes.iter().any(|m| m == mode) {
                bail!(
                    "theme config: defaultMode {mode:?} is not a mode on theme {default:?} \
                     (modes: {modes:?})"
                );
            }
        }
    }
    Ok(())
}

/// Mode keys for a theme id (project file wins, else catalog). Empty if unknown.
fn load_theme_mode_names(
    workspace_root: Option<&Path>,
    project_dir: &Path,
    theme_id: &str,
) -> Result<Vec<String>> {
    validate_theme_id(theme_id)?;
    // Project files first (modern then legacy).
    for path in project_theme_file_candidates(project_dir, theme_id) {
        if let Some(modes) = read_theme_mode_keys(&path)? {
            return Ok(modes);
        }
    }
    // Disk catalog when monorepo model is present.
    if let Some(ws) = workspace_root {
        let catalog = ws
            .join(MONOREPO_MODEL_REL)
            .join("themes")
            .join(format!("{theme_id}.json"));
        if let Some(modes) = read_theme_mode_keys(&catalog)? {
            return Ok(modes);
        }
    }
    // Baked catalog body.
    if let Some(body) = baked_catalog_theme_json(theme_id) {
        return mode_keys_from_theme_json(body);
    }
    Ok(Vec::new())
}

fn read_theme_mode_keys(path: &Path) -> Result<Option<Vec<String>>> {
    if !path.is_file() {
        return Ok(None);
    }
    let text =
        fs::read_to_string(path).with_context(|| format!("read theme {}", path.display()))?;
    mode_keys_from_theme_json(&text).map(Some)
}

fn mode_keys_from_theme_json(text: &str) -> Result<Vec<String>> {
    let stripped = strip_jsonc_comments(text);
    let doc: Value =
        serde_json::from_str(&stripped).with_context(|| "parse theme JSON for mode keys")?;
    if let Some(modes) = doc.get("modes").and_then(|m| m.as_object()) {
        let mut keys: Vec<String> = modes.keys().cloned().collect();
        keys.sort_by(|a, b| mode_rank(a).cmp(&mode_rank(b)).then_with(|| a.cmp(b)));
        return Ok(keys);
    }
    Ok(Vec::new())
}

/// Project theme dirs + T0 `enabled` as app emit options (thin on omit).
///
/// Prefer [`options_for_shared_package`] when writing monorepo
/// `packages/sail/tokens/src` so thin `enabled` cannot strip the shared library.
///
/// Validates `default` / `defaultMode` against the effective set (config-loud).
pub fn options_for_project(workspace_root: &Path, project_dir: &Path) -> Result<GenerateOptions> {
    options_for_project_opt(Some(workspace_root), project_dir)
}

/// Like [`options_for_project`] with optional workspace (baked catalog).
pub fn options_for_project_opt(
    workspace_root: Option<&Path>,
    project_dir: &Path,
) -> Result<GenerateOptions> {
    let theme_cfg = load_theme_config(project_dir)?;
    validate_theme_config_opt(workspace_root, project_dir, &theme_cfg)?;
    Ok(GenerateOptions {
        project_theme_dirs: project_theme_dirs(project_dir),
        enabled_catalog: theme_cfg.enabled_catalog_for_app(),
        themes_only: false,
    })
}

/// Options for generating into the monorepo shared `@ship/tokens` package.
///
/// Always **full catalog** + project theme merge (project wins on id). Thin
/// `theme.enabled` from sail.config is ignored so a scaffolded allowlist cannot
/// rewrite workspace-wide `themes.css` for every consumer.
///
/// Still validates theme config (default/defaultMode) so cold-start fails loud.
pub fn options_for_shared_package(
    workspace_root: &Path,
    project_dir: &Path,
) -> Result<GenerateOptions> {
    let mut opts = options_for_project(workspace_root, project_dir)?;
    opts.enabled_catalog = None;
    Ok(opts)
}

/// Generate **app delivery** themes into `<project>/.deck/themes/` (T1 B1).
///
/// Uses the baked catalog (or monorepo disk model when present) + project
/// themes; writes **only** `themes.css`. Does **not** touch package delivery.
pub fn generate_app_delivery(workspace_root: &Path, project_dir: &Path) -> Result<GenerateReport> {
    generate_app_delivery_opt(Some(workspace_root), project_dir)
}

/// Like [`generate_app_delivery`] with optional workspace (external apps).
pub fn generate_app_delivery_opt(
    workspace_root: Option<&Path>,
    project_dir: &Path,
) -> Result<GenerateReport> {
    let model_dir = resolve_model_dir(workspace_root, None)?;
    let out_dir = app_themes_out_dir(project_dir);
    let mut opts = options_for_project_opt(workspace_root, project_dir)?;
    opts.themes_only = true;
    generate_with_options(&model_dir, &out_dir, &opts)
}

/// Generate **package** delivery into `packages/sail/tokens/src` (full catalog).
///
/// Monorepo catalog development only — callers should gate with
/// [`should_generate_package_delivery`] / [`monorepo_tokens_package_present`].
pub fn generate_package_delivery(
    workspace_root: &Path,
    project_dir: Option<&Path>,
) -> Result<GenerateReport> {
    let model_dir = resolve_model_dir(Some(workspace_root), None)?;
    let out_dir = workspace_root.join(MONOREPO_TOKENS_SRC_REL);
    if let Some(project) = project_dir {
        let opts = options_for_shared_package(workspace_root, project)?;
        generate_with_options(&model_dir, &out_dir, &opts)
    } else {
        generate(&model_dir, &out_dir)
    }
}

/// True when the project has theme SSOT that should drive a cold-start generate
/// (`themes/*.json`, legacy `tokens/themes/*.json`, or sail.config `theme`).
pub fn project_wants_theme_generate(project_dir: &Path) -> bool {
    for dir in project_theme_dirs(project_dir) {
        if let Ok(rd) = fs::read_dir(&dir) {
            let has_json = rd.filter_map(|e| e.ok()).any(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.eq_ignore_ascii_case("json"))
            });
            if has_json {
                return true;
            }
        }
    }
    match load_theme_config(project_dir) {
        Ok(cfg) => match &cfg.enabled {
            // Explicit opt-out: only generate if defaults still need validation
            // (or project themes already returned true above).
            CatalogAllowlist::Empty => cfg.default.is_some() || cfg.default_mode.is_some(),
            // Default ship / all / explicit list → app should emit CSS.
            CatalogAllowlist::Default | CatalogAllowlist::All | CatalogAllowlist::Ids(_) => true,
        },
        // Malformed config: still run generate so the error surfaces on the
        // cold-start path rather than only after an edit.
        Err(_) => read_sail_config_text(project_dir).is_some(),
    }
}

fn load_compat_aliases(model_dir: &Path) -> Result<BTreeMap<String, String>> {
    let p = model_dir.join("compat-aliases.json");
    if !p.is_file() {
        return Ok(BTreeMap::new());
    }
    let doc = match load_json(&p) {
        Ok(d) => d,
        Err(_) => return Ok(BTreeMap::new()),
    };
    let Some(aliases) = doc.get("aliases").and_then(|v| v.as_object()) else {
        return Ok(BTreeMap::new());
    };
    let mut out = BTreeMap::new();
    for (k, v) in aliases {
        if let Some(s) = v.as_str() {
            out.insert(k.clone(), s.to_string());
        }
    }
    Ok(out)
}

/// Load product brand-hue system channels from `model/products.json`.
fn load_products(model_dir: &Path) -> Result<Option<ProductsAxis>> {
    let p = model_dir.join("products.json");
    if !p.is_file() {
        return Ok(None);
    }
    let doc = load_json(&p)?;
    let channel = doc
        .pointer("/$extensions/sail/channel")
        .and_then(|v| v.as_str())
        .unwrap_or("brand-hue")
        .to_string();
    let composition_law = doc
        .pointer("/$extensions/sail/compositionLaw")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let Some(products) = doc.get("products").and_then(|v| v.as_object()) else {
        bail!("products.json missing products object");
    };
    // Stable product order matches historical products.css (ship → sail → deck → …).
    const ORDER: &[&str] = &["ship", "sail", "deck", "components", "ai"];
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    for id in ORDER {
        if let Some(entry) = products.get(*id) {
            let hue = entry
                .get("brandHue")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow!("products.{id}.brandHue missing or not a number"))?
                as u32;
            let label = entry
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            entries.push(ProductEntry {
                id: (*id).to_string(),
                hue,
                label,
            });
            seen.insert((*id).to_string());
        }
    }
    for (id, entry) in products {
        if seen.contains(id) {
            continue;
        }
        let hue = entry
            .get("brandHue")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("products.{id}.brandHue missing or not a number"))?
            as u32;
        let label = entry
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        entries.push(ProductEntry {
            id: id.clone(),
            hue,
            label,
        });
    }
    Ok(Some(ProductsAxis {
        channel,
        composition_law,
        entries,
    }))
}

fn emit_products_css(axis: &ProductsAxis) -> String {
    let mut lines = vec![PRODUCTS_HEADER.trim_end().to_string(), String::new()];
    // Pad product ids so selectors align like the historical hand file.
    let id_width = axis.entries.iter().map(|e| e.id.len()).max().unwrap_or(0);
    for e in &axis.entries {
        let pad = " ".repeat(id_width.saturating_sub(e.id.len()));
        let comment = if e.label.is_empty() {
            String::new()
        } else {
            format!("  /* {} */", e.label)
        };
        lines.push(format!(
            ":root[data-product='{}']{} {{ --{}: {}; }}{}",
            e.id, pad, axis.channel, e.hue, comment
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn products_registry_meta(axis: &ProductsAxis) -> Value {
    let mut entries = Map::new();
    for e in &axis.entries {
        let mut m = Map::new();
        m.insert("hue".into(), Value::Number(e.hue.into()));
        if !e.label.is_empty() {
            m.insert("label".into(), Value::String(e.label.clone()));
        }
        entries.insert(e.id.clone(), Value::Object(m));
    }
    let mut out = Map::new();
    out.insert("layer".into(), Value::String("system".into()));
    out.insert("axis".into(), Value::String("product".into()));
    out.insert("channel".into(), Value::String(axis.channel.clone()));
    if !axis.composition_law.is_empty() {
        out.insert(
            "compositionLaw".into(),
            Value::String(axis.composition_law.clone()),
        );
    }
    out.insert("entries".into(), Value::Object(entries));
    Value::Object(out)
}

fn emit_block(
    selector: &str,
    tokens: &[Token],
    indent: &str,
    color_scheme: Option<&str>,
) -> String {
    if tokens.is_empty() {
        return String::new();
    }
    let mut lines = vec![format!("{selector} {{")];
    if let Some(scheme) = color_scheme {
        lines.push(format!("{indent}color-scheme: {scheme};"));
    }
    for t in tokens {
        lines.push(format!("{indent}--{}: {};", t.css_name, t.css_value));
    }
    lines.push("}".into());
    lines.join("\n")
}

fn emit_compat_aliases(selector: &str, aliases: &BTreeMap<String, String>) -> String {
    if aliases.is_empty() {
        return String::new();
    }
    let mut lines = vec![
        format!("{selector} {{"),
        "  /* Legacy aliases (PE compat) */".into(),
    ];
    // BTreeMap iterates sorted — matches localeCompare for ASCII names.
    for (old_name, new_name) in aliases {
        lines.push(format!("  --{old_name}: var(--{new_name});"));
    }
    lines.push("}".into());
    lines.join("\n")
}

/// Banner for the Tier-2 expression-channel block. The tests in
/// `tests/channel_integrity.rs` locate this block BY THIS BANNER STRING and
/// then require every declaration inside it to have a generated reader —
/// keep the heading verbatim.
const CHANNELS_BANNER: &str = "\
/* ─── Expression channels (Tier 2) ──────────────────────────────────
 *
 * Design-token \"theme-bending\" channels: the properties a theme most
 * often wants to bend but otherwise cannot — stroke weight, control
 * framing, interaction motion, corner treatment, display type, casing,
 * gradient and glow.
 *
 * The contract for every name in this block is enforced by the tests in
 * `crates/sail/compiler/tests/channel_integrity.rs` and
 * `channel_discipline.rs`:
 *   1. The default is a NO-OP — it reproduces today's rendering
 *      exactly, so declaring the channel changes nothing on its own.
 *      channel_integrity additionally rejects hard literals that would
 *      break composition (e.g. `none` inside a `box-shadow` list).
 *   2. A generated reader consumes it. A channel nothing reads is a
 *      variable in a costume; the `channel-readers` generator check
 *      fails the build if any name here goes unread.
 *   3. Where a channel governs a single CSS property, the skin lints in
 *      channel_discipline stop a skin writing that property literally
 *      instead of through the channel. Today that coverage is
 *      `text-transform` and `border-image`; `border-width` is tracked as a
 *      ratchet rather than an outright ban. Other channels are held
 *      honest by checks (1) and (2) above.
 *
 * Themes opt in; components never change. That is the whole thesis —
 * a theme should be a token block, not a patch.
 *
 * SSOT: `$extensions.sail.channel` + `$description` on the token in
 * packages/sail/tokens/model/**. Do not hand-edit this block.
 */";

/// Emit the Tier-2 expression-channel `:root` block: banner, then each
/// channel token with its `$description` as a doc comment.
fn emit_channel_block(tokens: &[Token]) -> String {
    let mut lines = vec![CHANNELS_BANNER.to_string(), ":root {".into()];
    for (i, t) in tokens.iter().enumerate() {
        if let Some(desc) = &t.description {
            if i > 0 {
                lines.push(String::new());
            }
            let mut desc_lines = desc.lines();
            if let Some(first) = desc_lines.next() {
                let rest: Vec<&str> = desc_lines.collect();
                if rest.is_empty() {
                    lines.push(format!("  /* {first} */"));
                } else {
                    lines.push(format!("  /* {first}"));
                    for line in rest {
                        lines.push(format!("   * {line}"));
                    }
                    lines.push("   */".into());
                }
            }
        }
        lines.push(format!("  --{}: {};", t.css_name, t.css_value));
    }
    lines.push("}".into());
    lines.join("\n")
}

fn emit_layer(layer: &Layer) -> String {
    let mut parts = Vec::new();
    let theme_suffix = layer
        .meta
        .theme_id
        .as_ref()
        .map(|id| format!(":{id}"))
        .unwrap_or_default();
    parts.push(format!(
        "/* ─── {}{} ({}) ─── */",
        layer.meta.layer, theme_suffix, layer.meta.file
    ));
    if !layer.meta.description.is_empty() {
        parts.push(format!("/* {} */", layer.meta.description));
    }
    let color_scheme = if layer.meta.theme_id.is_some() {
        Some(if layer.meta.dark { "dark" } else { "light" })
    } else {
        None
    };
    let block = emit_block(&layer.meta.selector, &layer.tokens, "  ", color_scheme);
    if let Some(media) = &layer.meta.media {
        parts.push(format!("@media {media} {{"));
        let indented: String = block
            .lines()
            .map(|l| {
                if l.is_empty() {
                    String::new()
                } else {
                    format!("  {l}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(indented);
        parts.push("}".into());
    } else {
        parts.push(block);
    }
    parts.join("\n")
}

fn build_registry(base_tokens: &[Token], aliases: &BTreeMap<String, String>) -> Value {
    // Preserve insertion order of base tokens then aliases (like JS object).
    let mut tokens: Map<String, Value> = Map::new();
    for t in base_tokens {
        let mut meta = Map::new();
        meta.insert("path".into(), Value::String(t.path.clone()));
        meta.insert("cssName".into(), Value::String(t.css_name.clone()));
        meta.insert("type".into(), Value::String(t.r#type.clone()));
        meta.insert("value".into(), Value::String(t.css_value.clone()));
        tokens.insert(t.css_name.clone(), Value::Object(meta));
    }

    let mut colors: HashSet<String> = HashSet::new();
    for (name, meta) in &tokens {
        if meta.get("type").and_then(|v| v.as_str()) == Some("color") {
            colors.insert(name.clone());
        }
    }
    for _ in 0..4 {
        let snapshot: Vec<(String, String)> = tokens
            .iter()
            .filter_map(|(name, meta)| {
                if colors.contains(name) {
                    return None;
                }
                let val = meta.get("value").and_then(|v| v.as_str())?;
                let target = parse_var_ref(val)?;
                if colors.contains(&target) {
                    Some((name.clone(), target))
                } else {
                    None
                }
            })
            .collect();
        for (name, _) in snapshot {
            colors.insert(name.clone());
            if let Some(Value::Object(meta)) = tokens.get_mut(&name) {
                meta.insert("type".into(), Value::String("color".into()));
            }
        }
    }

    let mut alias_entries = Map::new();
    for (old_name, new_name) in aliases {
        alias_entries.insert(old_name.clone(), Value::String(new_name.clone()));
        if colors.contains(new_name) {
            colors.insert(old_name.clone());
        }
        if tokens.contains_key(new_name) && !tokens.contains_key(old_name) {
            let src = tokens.get(new_name).cloned().unwrap();
            let path = src
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let ty = src
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("string")
                .to_string();
            let mut meta = Map::new();
            meta.insert("path".into(), Value::String(path));
            meta.insert("cssName".into(), Value::String(old_name.clone()));
            meta.insert("type".into(), Value::String(ty));
            meta.insert("value".into(), Value::String(format!("var(--{new_name})")));
            meta.insert("aliasOf".into(), Value::String(new_name.clone()));
            tokens.insert(old_name.clone(), Value::Object(meta));
        }
    }

    let mut names: Vec<String> = tokens.keys().cloned().collect();
    names.sort();
    let mut paths = Map::new();
    for n in &names {
        let meta = &tokens[n];
        if meta.get("aliasOf").is_some() {
            continue;
        }
        if let Some(path) = meta.get("path").and_then(|v| v.as_str()) {
            paths.insert(path.to_string(), Value::String(n.clone()));
        }
    }
    let mut colors_vec: Vec<String> = colors.into_iter().collect();
    colors_vec.sort();

    let mut out = Map::new();
    out.insert("version".into(), Value::Number(2.into()));
    out.insert(
        "source".into(),
        Value::String("packages/sail/tokens/model".into()),
    );
    out.insert("generated".into(), Value::Bool(true));
    out.insert(
        "names".into(),
        Value::Array(names.into_iter().map(Value::String).collect()),
    );
    out.insert(
        "colors".into(),
        Value::Array(colors_vec.into_iter().map(Value::String).collect()),
    );
    out.insert("tokens".into(), Value::Object(tokens));
    out.insert("paths".into(), Value::Object(paths));
    out.insert("aliases".into(), Value::Object(alias_entries));
    Value::Object(out)
}

fn parse_var_ref(value: &str) -> Option<String> {
    // /^var\(--([a-zA-Z0-9_-]+)\)/
    let rest = value.strip_prefix("var(--")?;
    let end = rest.find(')')?;
    let name = &rest[..end];
    if name.is_empty() {
        return None;
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return None;
    }
    // Must be exact match: var(--name) with nothing after
    if &rest[end..] != ")" {
        return None;
    }
    Some(name.to_string())
}

/// Registry `names` / `colors` arrays as sorted `String`s.
fn registry_string_list(registry: &Value, key: &str) -> Vec<String> {
    registry
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Tone allowlist for host props + Style bound groups.
/// Membership mirrors curated semantic color groups (not `color.tone.*` ramps).
/// `'default'` is not a Tone member — rest paint = component tokens when tone is omitted.
fn tone_names_from_tokens(_tokens: &[Token]) -> (Vec<String>, &'static str) {
    (
        TONE_FALLBACK.iter().map(|s| (*s).to_string()).collect(),
        "dtcg-groups:color.{primary,info,success,warning,destructive,brandAccent→accent,muted}",
    )
}

/// Build the Style/author `TONE` table: group → members → publicName.
fn tone_group_table(tokens: &[Token]) -> Vec<(String, Vec<(String, String)>)> {
    let (names, _) = tone_names_from_tokens(tokens);
    let path_to_css: HashMap<&str, &str> = tokens
        .iter()
        .map(|t| (t.path.as_str(), t.css_name.as_str()))
        .collect();
    let mut out = Vec::new();
    for name in &names {
        let dtcg_stem = if name == "accent" {
            "brandAccent"
        } else {
            name.as_str()
        };
        let mut members = Vec::new();
        // default member
        let default_paths = [
            format!("color.{dtcg_stem}.default"),
            format!("color.{dtcg_stem}"),
        ];
        let default_public = default_paths
            .iter()
            .find_map(|p| path_to_css.get(p.as_str()).copied())
            .unwrap_or(name.as_str());
        members.push(("default".to_string(), default_public.to_string()));
        // foreground
        let fg_path = format!("color.{dtcg_stem}.foreground");
        let fg_public = path_to_css
            .get(fg_path.as_str())
            .copied()
            .unwrap_or_else(|| {
                // brandAccent.foreground → accent-foreground via public_name
                let _ = dtcg_stem;
                if name == "accent" {
                    "accent-foreground"
                } else {
                    // leak a static by formatting into members below
                    ""
                }
            });
        let fg = if fg_public.is_empty() {
            format!("{name}-foreground")
        } else {
            fg_public.to_string()
        };
        members.push(("foreground".to_string(), fg));
        // hover only when present
        let hover_path = format!("color.{dtcg_stem}.hover");
        if let Some(h) = path_to_css.get(hover_path.as_str()) {
            members.push(("hover".to_string(), (*h).to_string()));
        } else if name == "primary" {
            // ensure primary-hover if registry has it under public name
            if tokens.iter().any(|t| t.css_name == "primary-hover") {
                members.push(("hover".to_string(), "primary-hover".to_string()));
            }
        }
        out.push((name.clone(), members));
    }
    out
}

/// Prefer `preferred` order for known members; append unknowns sorted.
fn order_with_preferred(set: &BTreeSet<String>, preferred: &[&str]) -> Vec<String> {
    let mut out = Vec::with_capacity(set.len());
    let mut remaining = set.clone();
    for p in preferred {
        if remaining.remove(*p) {
            out.push((*p).to_string());
        }
    }
    out.extend(remaining);
    out
}

/// Strip a stable prefix from cssNames. Returns (full token names ordered by
/// step, step suffixes ordered by `preferred_steps` then remaining sorted).
fn scale_from_prefix(
    names: &[String],
    prefix: &str,
    preferred_steps: &[&str],
) -> (Vec<String>, Vec<String>) {
    let mut steps = BTreeSet::new();
    for n in names {
        if let Some(step) = n.strip_prefix(prefix) {
            if !step.is_empty() {
                steps.insert(step.to_string());
            }
        }
    }
    let step_list = if preferred_steps.is_empty() {
        steps.into_iter().collect()
    } else {
        order_with_preferred(&steps, preferred_steps)
    };
    let full: Vec<String> = step_list.iter().map(|s| format!("{prefix}{s}")).collect();
    (full, step_list)
}

/// Semantic brand/surface paint cssNames under documented color groups.
fn semantic_color_names(tokens: &[Token]) -> Vec<String> {
    let mut set = BTreeSet::new();
    for t in tokens {
        let Some(rest) = t.path.strip_prefix("color.") else {
            continue;
        };
        if t.r#type != "color" {
            // Still include ring width etc.? No — paint only.
            continue;
        }
        let group = rest.split('.').next().unwrap_or("");
        if SEMANTIC_COLOR_GROUPS.contains(&group) {
            set.insert(t.css_name.clone());
        }
    }
    set.into_iter().collect()
}

fn ts_quote(s: &str) -> String {
    // Token / theme ids are ascii [A-Za-z0-9_-]; still escape for safety.
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out.push('\'');
    out
}

fn emit_const_and_type(
    out: &mut String,
    const_name: &str,
    type_name: &str,
    members: &[String],
    doc: &str,
) {
    if !doc.is_empty() {
        out.push_str("/** ");
        out.push_str(doc);
        out.push_str(" */\n");
    }
    out.push_str("export const ");
    out.push_str(const_name);
    out.push_str(" = [\n");
    for m in members {
        out.push_str("  ");
        out.push_str(&ts_quote(m));
        out.push_str(",\n");
    }
    out.push_str("] as const\n");
    out.push_str("export type ");
    out.push_str(type_name);
    out.push_str(" = (typeof ");
    out.push_str(const_name);
    out.push_str(")[number]\n\n");
}

fn emit_type_union(out: &mut String, type_name: &str, members: &[String], doc: &str) {
    if !doc.is_empty() {
        out.push_str("/** ");
        out.push_str(doc);
        out.push_str(" */\n");
    }
    out.push_str("export type ");
    out.push_str(type_name);
    out.push_str(" =\n");
    if members.is_empty() {
        out.push_str("  never\n\n");
        return;
    }
    for m in members {
        out.push_str("  | ");
        out.push_str(&ts_quote(m));
        out.push('\n');
    }
    out.push('\n');
}

/// Emit `generated/unions.ts` body (closed-set types for L1 / app props).
fn build_unions_ts(base_tokens: &[Token], registry: &Value, theme_ids: &[String]) -> String {
    let names = registry_string_list(registry, "names");
    let colors = registry_string_list(registry, "colors");
    let (tone_names, tone_source) = tone_names_from_tokens(base_tokens);
    let semantic = semantic_color_names(base_tokens);

    let (control_h_tokens, control_h_steps) =
        scale_from_prefix(&names, "control-h-", &["2xs", "xs", "sm", "md", "lg", "xl"]);
    let (text_tokens, text_steps) = scale_from_prefix(
        &names,
        "text-",
        &[
            "xs", "sm", "md", "base", "lg", "xl", "2xl", "3xl", "4xl", "display",
        ],
    );
    let (space_tokens, _space_steps) = scale_from_prefix(&names, "space-", &[]);
    let radius_tokens: Vec<String> = names
        .iter()
        .filter(|n| n.as_str() == "radius" || n.starts_with("radius-"))
        .cloned()
        .collect();
    let duration_tokens: Vec<String> = names
        .iter()
        .filter(|n| n.starts_with("duration-"))
        .cloned()
        .collect();

    let mut out = String::new();
    out.push_str(UNIONS_HEADER);

    // Tone — curated semantic paint groups (no `'default'`; rest = omit tone).
    out.push_str("/** Where {@link TONE_NAMES} membership is sourced from. */\n");
    out.push_str("export const TONE_NAMES_SOURCE = ");
    out.push_str(&ts_quote(tone_source));
    out.push_str(" as const\n\n");
    emit_const_and_type(
        &mut out,
        "TONE_NAMES",
        "Tone",
        &tone_names,
        "Semantic paint tone groups (Style bound `tone` / host prop). No `'default'`.",
    );
    out.push_str("/** @deprecated Use {@link Tone}. */\n");
    out.push_str("export type ToneName = Tone\n\n");
    // TONE table: group → members → publicName
    let tone_table = tone_group_table(base_tokens);
    out.push_str("/** Tone group → members → public token name (Style / tokens.css). */\n");
    out.push_str("export const TONE = {\n");
    for (group, members) in &tone_table {
        out.push_str("  ");
        out.push_str(group);
        out.push_str(": {\n");
        for (member, public) in members {
            out.push_str("    ");
            out.push_str(member);
            out.push_str(": ");
            out.push_str(&ts_quote(public));
            out.push_str(",\n");
        }
        out.push_str("  },\n");
    }
    out.push_str("} as const\n");
    out.push_str("export type ToneGroup = keyof typeof TONE\n");
    out.push_str(
        "export type ToneMember<G extends ToneGroup = ToneGroup> = keyof (typeof TONE)[G]\n\n",
    );

    emit_const_and_type(
        &mut out,
        "THEME_IDS",
        "ThemeId",
        theme_ids,
        "Theme catalog ids from packages/sail/tokens/model/themes/*.json.",
    );

    emit_const_and_type(
        &mut out,
        "CONTROL_HEIGHT_STEPS",
        "ControlHeight",
        &control_h_steps,
        "Control height scale steps (suffix of control-h-*).",
    );
    emit_const_and_type(
        &mut out,
        "CONTROL_HEIGHT_TOKENS",
        "ControlHeightToken",
        &control_h_tokens,
        "Control height token cssNames (control-h-*).",
    );

    emit_const_and_type(
        &mut out,
        "TEXT_SIZE_STEPS",
        "TextSize",
        &text_steps,
        "Typography size scale steps (suffix of text-*).",
    );
    emit_const_and_type(
        &mut out,
        "TEXT_SIZE_TOKENS",
        "TextSizeToken",
        &text_tokens,
        "Typography size token cssNames (text-*).",
    );

    emit_const_and_type(
        &mut out,
        "SPACE_TOKENS",
        "SpaceToken",
        &space_tokens,
        "Spacing scale token cssNames (space-*).",
    );

    emit_const_and_type(
        &mut out,
        "RADIUS_TOKENS",
        "RadiusToken",
        &radius_tokens,
        "Radius scale token cssNames.",
    );

    emit_const_and_type(
        &mut out,
        "DURATION_TOKENS",
        "DurationToken",
        &duration_tokens,
        "Motion duration token cssNames (duration-*).",
    );

    emit_const_and_type(
        &mut out,
        "SEMANTIC_COLOR_TOKENS",
        "SemanticColorName",
        &semantic,
        "Brand / semantic surface paint tokens under color.* (excl. tone ramps & product accents).",
    );

    // Large closed sets: type unions only (no multi-hundred runtime arrays).
    emit_type_union(
        &mut out,
        "ColorTokenName",
        &colors,
        "Every color-typed token cssName in the registry (paint tokens).",
    );
    emit_type_union(
        &mut out,
        "CssTokenName",
        &names,
        "Every public token cssName in the registry (without the -- prefix).",
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
    }

    #[test]
    fn flatten_and_resolve_alias() {
        let doc: Value = serde_json::json!({
            "space": {
                "$type": "dimension",
                "4": {
                    "$value": "16px",
                    "$extensions": { "sail": { "cssName": "space-4" } }
                }
            },
            "color": {
                "primary": {
                    "default": {
                        "$value": "oklch(0.5 0.1 250)",
                        "$type": "color",
                        "$extensions": { "sail": { "cssName": "primary" } }
                    }
                },
                "brand": {
                    "$value": "{color.primary.default}",
                    "$type": "color",
                    "$extensions": { "sail": { "cssName": "brand" } }
                }
            }
        });
        let tokens = flatten_tokens(&doc, &[], None);
        assert_eq!(tokens.len(), 3);
        let space = tokens.iter().find(|t| t.path == "space.4").unwrap();
        assert_eq!(space.css_name, "space-4");
        assert_eq!(space.value, "16px");
        let idx = build_name_index(&tokens);
        let resolved = resolve_value("{color.primary.default}", &idx).unwrap();
        assert_eq!(resolved, "var(--primary)");
        let by_css = resolve_value("{primary}", &idx).unwrap();
        assert_eq!(by_css, "var(--primary)");
    }

    #[test]
    fn generate_writes_artifacts_and_theme_block() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{
  "$description": "test base",
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "space": {
    "4": {
      "$value": "16px",
      "$type": "dimension",
      "$extensions": { "sail": { "cssName": "space-4" } }
    }
  },
  "control": {
    "h-sm": {
      "$value": "28px",
      "$type": "dimension",
      "$extensions": { "sail": { "cssName": "control-h-sm" } }
    },
    "h-md": {
      "$value": "34px",
      "$type": "dimension",
      "$extensions": { "sail": { "cssName": "control-h-md" } }
    }
  },
  "color": {
    "primary": {
      "$value": "oklch(0.5 0.1 250)",
      "$type": "color",
      "$extensions": { "sail": { "cssName": "primary" } }
    },
    "tone": {
      "default": {
        "fg": {
          "$value": "oklch(0.2 0 0)",
          "$type": "color",
          "$extensions": { "sail": { "cssName": "foreground" } }
        },
        "bg": {
          "$value": "oklch(0.9 0 0)",
          "$type": "color",
          "$extensions": { "sail": { "cssName": "muted" } }
        }
      },
      "primary": {
        "fg": {
          "$value": "{primary}",
          "$type": "color",
          "$extensions": { "sail": { "cssName": "primary" } }
        },
        "bg": {
          "$value": "{primary}",
          "$type": "color",
          "$extensions": { "sail": { "cssName": "hover-bg" } }
        }
      }
    }
  }
}
"#,
        );
        write_file(
            &model.join("compat-aliases.json"),
            r#"{ "aliases": { "app-primary": "primary" } }"#,
        );
        write_file(
            &model.join("themes/paper.json"),
            r#"{
  "$description": "paper theme",
  "$extensions": {
    "sail": {
      "layer": "theme",
      "themeId": "paper",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.4 0.05 75)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.7 0.08 75)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  }
}
"#,
        );
        write_file(
            &model.join("chrome.css"),
            "/* chrome */\nbody { margin: 0; }\n",
        );
        write_file(
            &model.join("themes-chrome.css"),
            "/* themes chrome */\nbody[theme='paper'] { background: white; }\n",
        );

        let report = generate(&model, &out).unwrap();
        assert!(report.tokens_css.is_file());
        assert!(report.themes_css.is_file());
        assert!(report.registry_json.is_file());
        assert!(report.unions_ts.is_file());
        assert_eq!(report.theme_count, 1);

        let tokens_css = fs::read_to_string(&report.tokens_css).unwrap();
        assert!(tokens_css.contains("GENERATED FILE"));
        assert!(tokens_css.contains("--space-4: 16px;"));
        assert!(tokens_css.contains("--app-primary: var(--primary);"));
        assert!(
            !tokens_css.contains(".dark {"),
            "dark.json dual layer must not emit"
        );
        assert!(tokens_css.contains("/* chrome */"));

        let themes_css = fs::read_to_string(&report.themes_css).unwrap();
        assert!(
            themes_css.contains("theme='paper'][data-mode='light'")
                || themes_css.contains(":root[theme='paper']"),
            "paper modes selectors missing: {themes_css}"
        );
        assert!(themes_css.contains("--primary:"));
        assert!(themes_css.contains("/* themes chrome */"));

        let reg: Value =
            serde_json::from_str(&fs::read_to_string(&report.registry_json).unwrap()).unwrap();
        assert_eq!(reg["version"], 2);
        assert!(
            reg["names"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str() == Some("space-4"))
        );
        assert!(
            reg["names"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str() == Some("app-primary"))
        );
        assert_eq!(reg["tokens"]["app-primary"]["aliasOf"], "primary");
        assert_eq!(reg["paths"]["space.4"], "space-4");
        assert!(
            reg["colors"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str() == Some("primary"))
        );

        let unions = fs::read_to_string(&report.unions_ts).unwrap();
        assert!(unions.contains("GENERATED FILE"));
        assert!(unions.contains("export type CssTokenName ="));
        assert!(unions.contains("export type ColorTokenName ="));
        assert!(unions.contains("export type ToneName ="));
        assert!(unions.contains("export type ThemeId ="));
        assert!(unions.contains("export const TONE_NAMES"));
        assert!(unions.contains("TONE_NAMES_SOURCE"));
        assert!(unions.contains("'default'"));
        assert!(unions.contains("'primary'"));
        assert!(unions.contains("'paper'"));
        assert!(unions.contains("'control-h-md'"));
        assert!(unions.contains("'sm'"));
        assert!(unions.contains("export type SpaceToken ="));
        assert!(unions.contains("export type SemanticColorName ="));

        // Re-run is stable.
        let again = generate(&model, &out).unwrap();
        assert_eq!(
            fs::read_to_string(&report.tokens_css).unwrap(),
            fs::read_to_string(&again.tokens_css).unwrap()
        );
        assert_eq!(
            fs::read_to_string(&report.registry_json).unwrap(),
            fs::read_to_string(&again.registry_json).unwrap()
        );
        assert_eq!(
            fs::read_to_string(&report.unions_ts).unwrap(),
            fs::read_to_string(&again.unions_ts).unwrap()
        );
    }

    #[test]
    fn tone_names_are_dtcg_groups_without_default() {
        let tokens = vec![Token {
            path: "color.primary".into(),
            css_name: "primary".into(),
            r#type: "color".into(),
            value: "oklch(0.5 0.1 250)".into(),
            css_value: "oklch(0.5 0.1 250)".into(),
            channel: false,
            description: None,
        }];
        let (names, source) = tone_names_from_tokens(&tokens);
        assert_eq!(
            names,
            TONE_FALLBACK
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>()
        );
        assert!(!names.iter().any(|n| n == "default"), "no default tone");
        assert!(source.contains("dtcg-groups"), "source={source}");
    }

    #[test]
    fn public_name_matches_historical_map() {
        // Spot-check rules against known historical cssNames (bit-identical policy).
        assert_eq!(public_name("color.primary.default"), "primary");
        assert_eq!(
            public_name("color.primary.foreground"),
            "primary-foreground"
        );
        assert_eq!(public_name("color.brandAccent.default"), "accent");
        assert_eq!(
            public_name("color.brandAccent.foreground"),
            "accent-foreground"
        );
        assert_eq!(public_name("component.chip.bg"), "chip");
        assert_eq!(public_name("component.chip.fg"), "chip-foreground");
        assert_eq!(public_name("component.chip.border"), "chip-border");
        assert_eq!(public_name("radius.default"), "radius");
        assert_eq!(public_name("space.4"), "space-4");
        assert_eq!(
            public_name("component.error-boundary.icon-fg"),
            "error-boundary-icon"
        );
        assert_eq!(public_name("control.press.translate"), "press-translate");
        assert_eq!(public_name("icon.sm"), "icon-size-sm");
        assert_eq!(public_name("color.mix.surfaceMix"), "mix-surface");
    }

    #[test]
    fn component_sources_merge_into_base_with_stable_css_names() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "color": {
    "surface": {
      "1": {
        "$value": "oklch(0.98 0 0)",
        "$type": "color",
        "$extensions": { "sail": { "cssName": "surface-1" } }
      }
    },
    "foreground": {
      "$value": "oklch(0.2 0 0)",
      "$type": "color",
      "$extensions": { "sail": { "cssName": "foreground" } }
    }
  }
}
"#,
        );
        write_file(
            &model.join("components/card.json"),
            r#"{
  "component": {
    "card": {
      "bg": {
        "$value": "{color.surface.1}",
        "$type": "color",
        "$extensions": { "sail": { "cssName": "card" } }
      },
      "fg": {
        "$value": "{color.foreground}",
        "$type": "color",
        "$extensions": { "sail": { "cssName": "card-foreground" } }
      }
    }
  }
}
"#,
        );
        write_file(
            &model.join("themes/glass.json"),
            r#"{
  "$extensions": {
    "sail": {
      "layer": "theme",
      "themeId": "glass",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "primary": {
          "default": { "$value": "oklch(0.45 0.12 250)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      },
      "component": {
        "card": {
          "bg": {
            "$value": "oklch(1 0 0 / 55%)",
            "$type": "color",
            "$extensions": { "sail": { "cssName": "card" } }
          }
        }
      }
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "primary": {
          "default": { "$value": "oklch(0.45 0.12 250)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      },
      "component": {
        "card": {
          "bg": {
            "$value": "oklch(0.2 0 0 / 55%)",
            "$type": "color",
            "$extensions": { "sail": { "cssName": "card" } }
          }
        }
      }
    }
  }
}
"#,
        );

        let report = generate(&model, &out).unwrap();
        let tokens_css = fs::read_to_string(&report.tokens_css).unwrap();
        assert!(
            tokens_css.contains("--card: var(--surface-1);"),
            "card defaults through global surface: {tokens_css}"
        );
        assert!(tokens_css.contains("--card-foreground: var(--foreground);"));

        let themes_css = fs::read_to_string(&report.themes_css).unwrap();
        assert!(
            themes_css.contains("--card: oklch(1 0 0 / 55%);"),
            "theme overrides component.card only: {themes_css}"
        );

        let reg: Value =
            serde_json::from_str(&fs::read_to_string(&report.registry_json).unwrap()).unwrap();
        assert_eq!(reg["paths"]["component.card.bg"], "card");
        assert_eq!(reg["paths"]["component.card.fg"], "card-foreground");
        assert!(reg["paths"].get("color.card.foreground").is_none());
        assert!(reg["paths"].get("misc.card").is_none());
    }

    #[test]
    fn channel_tokens_are_partitioned_under_the_tier_2_banner() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "color": {
    "foreground": {
      "$value": "oklch(0.2 0 0)",
      "$type": "color",
      "$extensions": { "sail": { "cssName": "foreground" } }
    }
  },
  "control": {
    "frame": {
      "$value": "0 0 0 0 transparent",
      "$type": "string",
      "$description": "Control framing — transparent-shadow no-op.",
      "$extensions": { "sail": { "cssName": "control-frame", "channel": true } }
    }
  }
}
"#,
        );

        let report = generate(&model, &out).unwrap();
        let css = fs::read_to_string(&report.tokens_css).unwrap();

        // Banner present, declaration inside the banner's :root block …
        let banner = css.find("Expression channels (Tier 2)").expect("banner");
        let block_end = css[banner..].find("\n}").map(|e| banner + e).unwrap();
        let block = &css[banner..block_end];
        assert!(
            block.contains("--control-frame: 0 0 0 0 transparent;"),
            "{css}"
        );
        assert!(block.contains("Control framing — transparent-shadow no-op."));
        // … and NOT duplicated in the base block.
        assert_eq!(css.matches("--control-frame:").count(), 1, "{css}");
        // Non-channel tokens stay in the base block, before the banner.
        let fg = css.find("--foreground:").unwrap();
        assert!(fg < banner, "{css}");
    }

    #[test]
    fn parse_theme_config_jsonc_and_defaults() {
        let cfg = parse_theme_config(
            r#"{
  // comment
  "theme": {
    "default": "paper",
    "defaultMode": "light",
    "enabled": ["paper", "glass"],
  },
}"#,
        )
        .unwrap();
        assert_eq!(cfg.default.as_deref(), Some("paper"));
        assert_eq!(cfg.default_mode.as_deref(), Some("light"));
        assert_eq!(
            cfg.enabled,
            CatalogAllowlist::Ids(vec!["paper".into(), "glass".into()])
        );
        assert!(cfg.enabled_key_present);
        assert_eq!(cfg.default_theme_id(), "paper");
        assert_eq!(
            cfg.enabled_catalog_for_app().as_deref(),
            Some(["paper".to_string(), "glass".to_string()].as_slice())
        );

        let empty = parse_theme_config("{}").unwrap();
        assert_eq!(empty, ThemeConfig::default());
        assert_eq!(empty.default_theme_id(), DEFAULT_PROJECT_THEME_ID);
        assert_eq!(empty.enabled, CatalogAllowlist::Default);
        assert!(!empty.enabled_key_present);
        // Omit enabled → default Sail theme (ship), not full catalog and not empty.
        assert_eq!(
            empty.enabled_catalog_for_app().as_deref(),
            Some([DEFAULT_PROJECT_THEME_ID.to_string()].as_slice())
        );

        let no_enabled = parse_theme_config(r#"{ "theme": { "default": "ship" } }"#).unwrap();
        assert_eq!(no_enabled.default.as_deref(), Some("ship"));
        assert_eq!(no_enabled.enabled, CatalogAllowlist::Default);
        assert!(!no_enabled.enabled_key_present);

        let all = parse_theme_config(r#"{ "theme": { "enabled": "all" } }"#).unwrap();
        assert_eq!(all.enabled, CatalogAllowlist::All);
        assert!(all.enabled_key_present);
        assert!(all.enabled_catalog_for_app().is_none());
    }

    #[test]
    fn theme_config_malformed_and_wrong_typed_enabled() {
        assert!(
            parse_theme_config("{ not json").is_err(),
            "malformed must fail loud"
        );
        assert!(
            parse_theme_config(r#"{ "theme": "paper" }"#).is_err(),
            "theme must be object"
        );
        // Wrong-typed enabled → hard error (T0).
        assert!(
            parse_theme_config(r#"{ "theme": { "enabled": true } }"#).is_err(),
            "bool enabled must error"
        );
        assert!(
            parse_theme_config(r#"{ "theme": { "enabled": ["paper", 1] } }"#).is_err(),
            "non-string element must error"
        );
        assert!(
            parse_theme_config(r#"{ "theme": { "enabled": "ship" } }"#).is_err(),
            "string other than \"all\" must error"
        );
        let empty_list = parse_theme_config(r#"{ "theme": { "enabled": [] } }"#).unwrap();
        assert_eq!(empty_list.enabled, CatalogAllowlist::Empty);
        assert!(empty_list.enabled_key_present);
        assert_eq!(
            empty_list.enabled_catalog_for_app().as_deref(),
            Some([].as_slice())
        );
    }

    #[test]
    fn dual_read_prefers_jsonc_over_json() {
        let dir = tempfile::tempdir().unwrap();
        write_file(
            &dir.path().join("sail.config.json"),
            r#"{ "theme": { "default": "from-json" } }"#,
        );
        write_file(
            &dir.path().join("sail.config.jsonc"),
            r#"{ "theme": { "default": "from-jsonc" } }"#,
        );
        let cfg = load_theme_config(dir.path()).unwrap();
        assert_eq!(cfg.default.as_deref(), Some("from-jsonc"));
    }

    #[test]
    fn validate_theme_id_rejects_path_escape() {
        assert!(validate_theme_id("ship").is_ok());
        assert!(validate_theme_id("../evil").is_err());
        assert!(validate_theme_id("has space").is_err());
        assert!(validate_theme_id("Ship").is_err());
        assert!(validate_theme_id("").is_err());
    }

    #[test]
    fn empty_enabled_emits_project_only_unknown_id_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        let project = dir.path().join("app");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "color": {
        "primary": {
          "default": { "$value": "oklch(0.5 0.1 250)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
}
"#,
        );
        write_file(
            &model.join("themes/paper.json"),
            r#"{
  "$extensions": { "sail": { "layer": "theme", "themeId": "paper", "defaultMode": "light" } },
  "modes": { "light": { "color": {
        "primary": {
          "default": { "$value": "oklch(0.4 0.05 75)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      } }, "dark": { "$extensions": { "sail": { "dark": true } }, "color": {
        "primary": {
          "default": { "$value": "oklch(0.7 0.1 75)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      } } }
}
"#,
        );
        write_file(
            &project.join("tokens/themes/custom.json"),
            r#"{
  "$extensions": { "sail": { "layer": "theme", "themeId": "custom", "defaultMode": "light" } },
  "modes": { "light": { "color": {
        "primary": {
          "default": { "$value": "oklch(0.45 0.2 140)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      } }, "dark": { "$extensions": { "sail": { "dark": true } }, "color": {
        "primary": {
          "default": { "$value": "oklch(0.6 0.15 140)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      } } }
}
"#,
        );
        write_file(
            &project.join("sail.config.jsonc"),
            r#"{ "theme": { "enabled": [], "default": "custom" } }"#,
        );
        let opts = options_for_project(dir.path(), &project).unwrap();
        assert_eq!(opts.enabled_catalog.as_deref(), Some([].as_slice()));
        let report = generate_with_options(&model, &out, &opts).unwrap();
        assert_eq!(report.theme_count, 1);
        let css = fs::read_to_string(&report.themes_css).unwrap();
        assert!(css.contains("theme='custom'"));
        assert!(!css.contains("theme='paper'"));

        // Unknown id in enabled: silent drop (no catalog match); project still emits.
        write_file(
            &project.join("sail.config.jsonc"),
            r#"{ "theme": { "enabled": ["no-such-theme"], "default": "custom" } }"#,
        );
        let opts2 = options_for_project(dir.path(), &project).unwrap();
        let report2 = generate_with_options(&model, &dir.path().join("src2"), &opts2).unwrap();
        assert_eq!(report2.theme_count, 1, "project custom only");

        // Shared package options ignore thin enabled → full catalog + project.
        let shared = options_for_shared_package(dir.path(), &project).unwrap();
        assert!(shared.enabled_catalog.is_none());
        let full = generate_with_options(&model, &dir.path().join("src3"), &shared).unwrap();
        assert_eq!(full.theme_count, 2, "paper + custom");

        // Omit enabled → default ship from catalog (when present); not full catalog.
        let bare = dir.path().join("bare");
        fs::create_dir_all(&bare).unwrap();
        write_file(
            &model.join("themes/ship.json"),
            r#"{
  "$extensions": { "sail": { "layer": "theme", "themeId": "ship", "defaultMode": "light" } },
  "modes": { "light": { "color": {
        "primary": {
          "default": { "$value": "oklch(0.5 0.1 250)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      } }, "dark": { "$extensions": { "sail": { "dark": true } }, "color": {
        "primary": {
          "default": { "$value": "oklch(0.3 0.05 250)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      } } }
}
"#,
        );
        let bare_opts = options_for_project(dir.path(), &bare).unwrap();
        assert!(bare_opts.project_theme_dirs.is_empty());
        assert_eq!(
            bare_opts.enabled_catalog.as_deref(),
            Some([DEFAULT_PROJECT_THEME_ID.to_string()].as_slice())
        );
        let bare_report =
            generate_with_options(&model, &dir.path().join("src4"), &bare_opts).unwrap();
        assert_eq!(
            bare_report.theme_count, 1,
            "omit enabled ships default ship"
        );
        let bare_css = fs::read_to_string(&bare_report.themes_css).unwrap();
        assert!(bare_css.contains("theme='ship'"));
        assert!(!bare_css.contains("theme='paper'"));

        // Explicit "all" → full catalog on app path.
        write_file(
            &bare.join("sail.config.jsonc"),
            r#"{ "theme": { "enabled": "all" } }"#,
        );
        let all_opts = options_for_project(dir.path(), &bare).unwrap();
        assert!(all_opts.enabled_catalog.is_none());
        let all_report =
            generate_with_options(&model, &dir.path().join("src5"), &all_opts).unwrap();
        assert_eq!(
            all_report.theme_count, 2,
            "paper + ship from catalog under enabled=all"
        );
    }

    #[test]
    fn project_themes_and_enabled_catalog_drive_emit_set() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        let project = dir.path().join("app");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "color": {
        "primary": {
          "default": { "$value": "oklch(0.5 0.1 250)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
}
"#,
        );
        write_file(
            &model.join("themes/paper.json"),
            r#"{
  "$extensions": {
    "sail": {
      "layer": "theme",
      "themeId": "paper",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.4 0.05 75)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.4 0.05 75)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  }
}
"#,
        );
        write_file(
            &model.join("themes/glass.json"),
            r#"{
  "$extensions": {
    "sail": {
      "layer": "theme",
      "themeId": "glass",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.8 0.05 200)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.8 0.05 200)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  }
}
"#,
        );
        // Project overrides paper + adds a custom theme.
        write_file(
            &project.join("tokens/themes/paper.json"),
            r#"{
  "$extensions": {
    "sail": {
      "layer": "theme",
      "themeId": "paper",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.12 0.34 99)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.12 0.34 99)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  }
}
"#,
        );
        write_file(
            &project.join("tokens/themes/custom.json"),
            r#"{
  "$extensions": {
    "sail": {
      "layer": "theme",
      "themeId": "custom",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.45 0.2 140)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.45 0.2 140)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  }
}
"#,
        );
        write_file(
            &project.join("sail.config.jsonc"),
            r#"{
  "theme": {
    "default": "paper",
    // only paper from catalog; glass must not emit
    "enabled": ["paper"]
  }
}
"#,
        );

        let opts = options_for_project(dir.path(), &project).unwrap();
        assert_eq!(
            opts.enabled_catalog.as_deref(),
            Some(["paper".to_string()].as_slice())
        );
        assert_eq!(opts.project_theme_dirs.len(), 1);
        // paper is both project + enabled catalog; custom is project-only.

        let report = generate_with_options(&model, &out, &opts).unwrap();
        assert_eq!(report.theme_count, 2, "paper + custom");
        let themes_css = fs::read_to_string(&report.themes_css).unwrap();
        assert!(
            themes_css.contains("oklch(0.12 0.34 99)"),
            "project paper leaf must win: {themes_css}"
        );
        assert!(
            themes_css.contains(":root[theme='custom']") || themes_css.contains("[theme='custom']"),
            "project-only theme must emit: {themes_css}"
        );
        assert!(
            !themes_css.contains("theme='glass'"),
            "disabled catalog theme must not emit: {themes_css}"
        );
        // Catalog-only generate still emits full catalog.
        let full = generate(&model, &dir.path().join("src-full")).unwrap();
        assert_eq!(full.theme_count, 2);
        let full_css = fs::read_to_string(&full.themes_css).unwrap();
        assert!(full_css.contains("theme='glass'"));
        assert!(
            full_css.contains("oklch(0.4 0.05 75)"),
            "catalog paper value"
        );
    }

    #[test]
    fn scaffold_copies_default_theme_once() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = dir.path().join("catalog");
        let project = dir.path().join("app");
        write_file(
            &catalog.join("ship.json"),
            r#"{ "$extensions": { "sail": { "themeId": "ship", "defaultMode": "light" } }, "modes": { "light": {}, "dark": {} } }"#,
        );
        let dest = scaffold_default_project_theme(&project, Some(&catalog), None).unwrap();
        assert!(dest.is_file());
        assert_eq!(dest.file_name().and_then(|n| n.to_str()), Some("ship.json"));
        // Modern external layout: <project>/themes/<id>.json
        assert!(
            dest.ends_with("themes/ship.json") || dest.ends_with("themes\\ship.json"),
            "dest={}",
            dest.display()
        );
        // Second call does not overwrite (even if source changes).
        write_file(&catalog.join("ship.json"), r#"{ "changed": true }"#);
        let again = scaffold_default_project_theme(&project, Some(&catalog), None).unwrap();
        assert_eq!(again, dest);
        let body = fs::read_to_string(&dest).unwrap();
        assert!(body.contains("ship"));
        assert!(!body.contains("changed"));

        let cfg_path = scaffold_theme_config(&project, DEFAULT_PROJECT_THEME_ID)
            .unwrap()
            .expect("writes new config");
        assert!(cfg_path.is_file());
        let cfg = load_theme_config(&project).unwrap();
        assert_eq!(cfg.default.as_deref(), Some(DEFAULT_PROJECT_THEME_ID));
        assert_eq!(cfg.default_mode.as_deref(), Some("light"));
        assert_eq!(
            cfg.enabled,
            CatalogAllowlist::Ids(vec![DEFAULT_PROJECT_THEME_ID.to_string()])
        );
        assert!(cfg.enabled_key_present);
        // Existing config is left alone.
        assert!(scaffold_theme_config(&project, "paper").unwrap().is_none());
        // Bad id rejected.
        assert!(scaffold_default_project_theme(&project, Some(&catalog), Some("../x")).is_err());
    }

    #[test]
    fn scaffold_from_baked_catalog_without_disk_model() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("external-app");
        let dest = scaffold_default_project_theme(&project, None, Some("ship")).unwrap();
        assert!(dest.is_file());
        assert!(dest.ends_with("themes/ship.json") || dest.ends_with("themes\\ship.json"));
        let body = fs::read_to_string(&dest).unwrap();
        assert!(body.contains("ship") || body.contains("themeId"), "{body}");
        // Baked ids include known catalog stems.
        let ids = baked_catalog_theme_ids();
        assert!(ids.contains(&"ship".to_string()), "{ids:?}");
        assert!(ids.contains(&"paper".to_string()), "{ids:?}");
    }

    /// No monorepo model under the project: baked catalog + project themes
    /// still emit app delivery CSS.
    #[test]
    fn external_app_delivery_uses_baked_catalog() {
        let dir = tempfile::tempdir().unwrap();
        // Isolated "workspace" with no packages/sail/tokens tree.
        let fake_ws = dir.path().join("not-a-monorepo");
        let project = fake_ws.join("my-app");
        fs::create_dir_all(&project).unwrap();
        assert!(!fake_ws.join("packages/sail/tokens/model").exists());
        scaffold_default_project_theme(&project, None, Some("ship")).unwrap();
        write_file(
            &project.join("sail.config.jsonc"),
            r#"{ "theme": { "default": "ship", "defaultMode": "light", "enabled": ["ship"] } }"#,
        );
        let report = generate_app_delivery(&fake_ws, &project).unwrap();
        assert_eq!(report.theme_count, 1);
        assert!(report.themes_css.is_file());
        let css = fs::read_to_string(&report.themes_css).unwrap();
        assert!(
            css.contains("theme='ship'") || css.contains("theme=\"ship\""),
            "baked ship theme must paint: {css}"
        );
        assert!(
            !css.contains("theme='synthwave'"),
            "thin enabled must not emit full catalog"
        );
        // Package delivery must not run for non-monorepo workspaces.
        assert!(!should_generate_package_delivery(&fake_ws));
        // Effective ids from baked catalog alone.
        let ids = effective_theme_ids_opt(None, &project).unwrap();
        assert!(ids.catalog.contains(&"ship".to_string()), "{ids:?}");
        assert!(ids.project.contains(&"ship".to_string()), "{ids:?}");
    }

    /// Monorepo-path proof: real catalog model + project theme leaf → CSS.
    #[test]
    fn monorepo_project_theme_leaf_reaches_themes_css() {
        // crates/sail/compiler → repo root
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest
            .ancestors()
            .nth(3)
            .expect("repo root above crates/sail/compiler");
        let model = root.join("packages/sail/tokens/model");
        if !model.join("themes/ship.json").is_file() {
            // Worktree without tokens package — skip rather than fail CI layouts.
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("app");
        let out = dir.path().join("out");
        let marker = "oklch(0.111 0.222 33.3)";
        // Scaffold default theme then override a primary leaf with a unique marker.
        let dest = scaffold_default_project_theme(
            &project,
            Some(&model.join("themes")),
            Some(DEFAULT_PROJECT_THEME_ID),
        )
        .unwrap();
        let mut doc: Value = serde_json::from_str(&fs::read_to_string(&dest).unwrap()).unwrap();
        // Modes-in-file (`ship`) nests tokens under modes.<default>; twins use root.
        let mode_ptr = "/modes/light/color/primary/default/$value";
        if doc.pointer(mode_ptr).is_some() {
            *doc.pointer_mut(mode_ptr).unwrap() = Value::String(marker.to_string());
        } else {
            panic!("default theme must be modes-in-file with color.primary.default");
        }
        fs::write(&dest, serde_json::to_string_pretty(&doc).unwrap() + "\n").unwrap();
        write_file(
            &project.join("sail.config.jsonc"),
            &format!(
                r#"{{ "theme": {{ "default": "{DEFAULT_PROJECT_THEME_ID}", "defaultMode": "light", "enabled": ["{DEFAULT_PROJECT_THEME_ID}"] }} }}"#
            ),
        );

        // Thin options (project-local out): single theme block; no synthwave.
        let thin = options_for_project(root, &project).unwrap();
        assert_eq!(
            thin.enabled_catalog.as_deref(),
            Some([DEFAULT_PROJECT_THEME_ID.to_string()].as_slice())
        );
        let report = generate_with_options(&model, &out, &thin).unwrap();
        assert_eq!(report.theme_count, 1, "thin enabled + project same id");
        let css = fs::read_to_string(&report.themes_css).unwrap();
        assert!(
            css.contains(marker),
            "project DTCG leaf must appear in generated themes.css: marker missing"
        );
        assert!(
            css.contains("theme='ship'") || css.contains("theme=\"ship\""),
            "default theme selector must emit"
        );
        assert!(
            !css.contains("theme='synthwave'"),
            "thin app emit must not ship unused catalog theme synthwave"
        );

        // Shared package options: full catalog still includes project override leaf.
        let shared = options_for_shared_package(root, &project).unwrap();
        assert!(shared.enabled_catalog.is_none());
        let shared_out = dir.path().join("shared-out");
        let shared_report = generate_with_options(&model, &shared_out, &shared).unwrap();
        assert!(
            shared_report.theme_count >= 2,
            "shared package keeps full catalog"
        );
        let shared_css = fs::read_to_string(&shared_report.themes_css).unwrap();
        assert!(
            shared_css.contains(marker),
            "project override still merges into shared full-catalog emit"
        );
        assert!(
            shared_css.contains("theme='synthwave'") || shared_css.contains("theme=\"synthwave\""),
            "package emit keeps full catalog including synthwave"
        );

        // B1: app delivery writes filtered themes under project/.deck/themes/
        // (themes.css only — no tokens/registry/unions litter).
        let app_report = generate_app_delivery(root, &project).unwrap();
        assert_eq!(app_report.theme_count, 1);
        assert_eq!(app_report.themes_css, app_themes_css_path(&project));
        assert!(
            app_report
                .themes_css
                .to_string_lossy()
                .replace('\\', "/")
                .contains("/.deck/themes/themes.css"),
            "app path must be .deck/themes/themes.css (prune-safe foreign dir)"
        );
        assert!(
            !app_themes_out_dir(&project).join("tokens.css").is_file(),
            "app delivery must not write tokens.css"
        );
        assert!(
            !app_themes_out_dir(&project)
                .join("token-registry.json")
                .is_file(),
            "app delivery must not write registry"
        );
        let app_css = fs::read_to_string(&app_report.themes_css).unwrap();
        assert!(app_css.contains(marker));
        assert!(!app_css.contains("theme='synthwave'"));
    }

    #[test]
    fn project_wants_theme_generate_detects_ssot() {
        let dir = tempfile::tempdir().unwrap();
        // No config → CatalogAllowlist::Default (ship) still wants generate so
        // bare apps get default Sail styling without opt-in.
        assert!(
            project_wants_theme_generate(dir.path()),
            "default ship allowlist should cold-start generate"
        );
        write_file(
            &dir.path().join("tokens/themes/x.json"),
            r#"{ "$extensions": { "sail": { "themeId": "x", "defaultMode": "light" } }, "modes": { "light": {}, "dark": {} } }"#,
        );
        assert!(project_wants_theme_generate(dir.path()));
        let bare = tempfile::tempdir().unwrap();
        write_file(
            &bare.path().join("sail.config.jsonc"),
            r#"{ "theme": { "default": "ship" } }"#,
        );
        assert!(project_wants_theme_generate(bare.path()));
        // Explicit empty catalog + no project themes → no generate needed for paint.
        let empty = tempfile::tempdir().unwrap();
        write_file(
            &empty.path().join("sail.config.jsonc"),
            r#"{ "theme": { "enabled": [] } }"#,
        );
        assert!(!project_wants_theme_generate(empty.path()));
    }

    #[test]
    fn reject_twin_theme_ids_and_require_modes() {
        // Modes-in-file + twin-id rules sit after schema lock. Fixtures must use
        // the modern envelope (root: $description | $extensions | modes) so we
        // exercise twin/modes, not unknown-root-key.
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{ "$extensions": { "sail": { "selector": ":root", "layer": "base" } } }"#,
        );

        // Missing `modes` (valid root keys only) → modes-in-file required.
        write_file(
            &model.join("themes/ship.json"),
            r#"{
  "$extensions": { "sail": { "themeId": "ship", "layer": "theme" } }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("modes-in-file"),
            "expected modes-in-file error, got: {err}"
        );
        let _ = fs::remove_file(model.join("themes/ship.json"));

        // Twin theme id with modes envelope still refuses (one id + modes.*).
        write_file(
            &model.join("themes/ship-light.json"),
            r#"{
  "$extensions": {
    "sail": {
      "themeId": "ship-light",
      "defaultMode": "light",
      "layer": "theme"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.5 0.1 50)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("twin"),
            "expected twin theme id error, got: {err}"
        );
    }

    #[test]
    fn allow_single_mode_theme() {
        // Owner override: single-mode themes (e.g. glass light-only) are valid.
        // Path parity is among declared modes only — no dual-mode requirement.
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{ "$extensions": { "sail": { "selector": ":root", "layer": "base" } } }"#,
        );
        write_file(
            &model.join("themes/glass.json"),
            r#"{
  "$extensions": {
    "sail": {
      "themeId": "glass",
      "defaultMode": "light",
      "supportedModes": ["light"]
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.62 0.16 50)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  }
}
"#,
        );
        generate(&model, &out).expect("single-mode theme must generate");
        let css = fs::read_to_string(out.join("themes.css")).unwrap();
        assert!(
            css.contains("theme='glass'][data-mode='light']"),
            "expected light selector, got: {css}"
        );
        assert!(
            !css.contains("theme='glass'][data-mode='dark']"),
            "single-mode glass must not emit dark block"
        );
        let meta = fs::read_to_string(out.join("generated/themes-meta.ts")).unwrap();
        assert!(
            meta.contains("id: 'glass'") && meta.contains("modes: ['light']"),
            "themes-meta must list only light for glass, got: {meta}"
        );
        assert!(
            meta.contains("capability: 'light'"),
            "single-mode glass must emit capability light, got: {meta}"
        );
        assert!(
            meta.contains("THEME_ROOT_KEYS")
                && meta.contains("THEME_OVERRIDE_ORDER")
                && meta.contains("ModeCapability"),
            "themes-meta must emit THEMES-1 schema lock constants, got: {meta}"
        );
    }

    #[test]
    fn reject_unknown_theme_root_key_names_valid() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{ "$extensions": { "sail": { "selector": ":root", "layer": "base" } } }"#,
        );
        write_file(
            &model.join("themes/bad.json"),
            r#"{
  "$extensions": { "sail": { "themeId": "bad", "defaultMode": "light" } },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.5 0.1 50)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  },
  "palette": { "oops": true }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("unknown root key") && err.contains("palette"),
            "expected unknown root key error, got: {err}"
        );
        assert!(
            err.contains("valid:") || err.contains("closest valid key"),
            "must name valid keys, got: {err}"
        );
    }

    #[test]
    fn reject_unknown_mode_group_key_names_valid() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{ "$extensions": { "sail": { "selector": ":root", "layer": "base" } } }"#,
        );
        write_file(
            &model.join("themes/bad-group.json"),
            r#"{
  "$extensions": { "sail": { "themeId": "bad-group", "defaultMode": "light" } },
  "modes": {
    "light": {
      "colour": {
        "primary": {
          "$value": "oklch(0.5 0.1 50)",
          "$type": "color",
          "$extensions": { "sail": { "cssName": "primary" } }
        }
      }
    }
  }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("unknown modes.light key") && err.contains("colour"),
            "expected unknown mode group error, got: {err}"
        );
        assert!(
            err.contains("color") || err.contains("closest valid key"),
            "must suggest color, got: {err}"
        );
    }

    #[test]
    fn validate_theme_doc_schema_ok_for_minimal() {
        let doc: Value = serde_json::from_str(
            r#"{
  "$extensions": { "sail": { "themeId": "x", "defaultMode": "dark", "layer": "theme" } },
  "modes": { "dark": { "color": {} } }
}"#,
        )
        .unwrap();
        validate_theme_doc_schema(&doc, "x.json").expect("minimal dark theme ok");
        assert_eq!(mode_capability(&["dark".into()]), "dark");
        assert_eq!(mode_capability(&["light".into()]), "light");
        assert_eq!(mode_capability(&["light".into(), "dark".into()]), "both");
    }

    #[test]
    fn reject_supported_modes_missing_body() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{ "$extensions": { "sail": { "selector": ":root", "layer": "base" } } }"#,
        );
        write_file(
            &model.join("themes/broken.json"),
            r#"{
  "$extensions": {
    "sail": {
      "themeId": "broken",
      "defaultMode": "dark",
      "supportedModes": ["dark"]
    }
  },
  "modes": { "light": {} }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("supportedModes") && err.contains("dark"),
            "expected supportedModes/missing body error, got: {err}"
        );
    }

    #[test]
    fn reject_modes_path_parity_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{ "$extensions": { "sail": { "selector": ":root", "layer": "base" } } }"#,
        );
        write_file(
            &model.join("themes/uneven.json"),
            r#"{
  "$extensions": { "sail": { "themeId": "uneven", "defaultMode": "light" } },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.7 0.1 100)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.3 0.1 100)",
          "$type": "color",
          "$extensions": { "sail": { "cssName": "primary" } }
        },
        "surface": {
          "0": {
            "$value": "oklch(0.2 0.02 250)",
            "$type": "color",
            "$extensions": { "sail": { "cssName": "surface-0" } }
          }
        }
      }
    }
  }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("parity") && err.contains("surface"),
            "expected path parity error mentioning surface, got: {err}"
        );
    }

    #[test]
    fn modes_in_file_emit_data_theme_and_data_mode() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "color": {
        "primary": {
          "default": { "$value": "oklch(0.5 0.1 250)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
}
"#,
        );
        write_file(
            &model.join("themes/ship.json"),
            r#"{
  "$extensions": {
    "sail": { "layer": "theme", "themeId": "ship", "defaultMode": "light" }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.7 0.16 58)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.77 0.16 70)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
    }
  }
}
"#,
        );
        let report = generate(&model, &out).unwrap();
        assert_eq!(report.theme_count, 1, "one theme id, two modes");
        let css = fs::read_to_string(&report.themes_css).unwrap();
        assert!(
            css.contains("theme='ship'][data-mode='light'")
                || css.contains(r#"theme='ship'][data-mode='light'"#),
            "light mode selector missing: {css}"
        );
        assert!(
            css.contains("theme='ship'][data-mode='dark'"),
            "dark mode selector missing: {css}"
        );
        assert!(
            css.contains("[theme='ship'][data-mode='light']") || css.contains("[theme='ship']"),
            "author IR [theme=id] selector missing: {css}"
        );
        assert!(
            css.contains(":root[data-mode='dark'] [theme='ship']:not([data-mode])"),
            "dual-mode nested host must inherit document dark: {css}"
        );
        assert!(
            css.contains(":root[data-mode='light'] [theme='ship']:not([data-mode])")
                || css.contains(":root:not([data-mode]) [theme='ship']:not([data-mode])"),
            "dual-mode nested host must inherit document light/cold: {css}"
        );
        assert!(
            css.contains("color-scheme: light") && css.contains("color-scheme: dark"),
            "theme mode blocks must set color-scheme: {css}"
        );
        // Global `[theme=id]:not([data-mode])` would pin nested dual-mode hosts
        // to the default mode. Only ancestor-qualified forms are allowed.
        let global_bare = css.lines().any(|l| {
            l.trim() == "[theme='ship']:not([data-mode])"
                || l.trim() == "[theme='ship']:not([data-mode]),"
        });
        assert!(
            !global_bare,
            "dual-mode must not emit a global [theme=id]:not([data-mode]) selector: {css}"
        );
        let meta = fs::read_to_string(out.join("generated/themes-meta.ts")).unwrap();
        assert!(meta.contains("id: 'ship'"));
        assert!(meta.contains("'light'"));
        assert!(meta.contains("'dark'"));
        assert!(meta.contains("defaultMode: 'light'"));
    }

    #[test]
    fn load_json_accepts_jsonc_comments_in_theme_files() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "color": {
        "primary": {
          "default": { "$value": "oklch(0.5 0.1 250)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
}
"#,
        );
        write_file(
            &model.join("themes/paper.json"),
            r#"{
  // project-style comment in theme DTCG
  "$extensions": {
    "sail": {
      "layer": "theme",
      "themeId": "paper",
      "defaultMode": "light",
    },
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.4 0.05 75)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" },
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        },
      },
    },
    "dark": {
      "$extensions": { "sail": { "dark": true } },
      "color": {
        "primary": {
          "default": { "$value": "oklch(0.7 0.08 75)", "$type": "color" },
          "foreground": { "$value": "oklch(0.18 0 0)", "$type": "color" },
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" },
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        },
      },
    },
  },
}
"#,
        );
        let report = generate(&model, &out).unwrap();
        assert_eq!(report.theme_count, 1);
    }
    #[test]
    fn dangling_alias_is_hard_error() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "color": {
        "primary": {
          "default": { "$value": "{color.does.not.exist}", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        },
        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
        "muted": {
          "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
          "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
        },
        "brandAccent": {
          "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
          "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
        }
      }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("dangling alias") || err.contains("does.not.exist"),
            "expected dangling alias diagnostic, got: {err}"
        );
    }

    #[test]
    fn alias_cycle_is_hard_error() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "sail": { "selector": ":root", "layer": "base" } },
  "color": {
    "a": { "$value": "{color.b}", "$type": "color" },
    "b": { "$value": "{color.a}", "$type": "color" }
  }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("alias cycle"),
            "expected alias cycle diagnostic, got: {err}"
        );
    }
}
