//! DTCG model → `tokens.css` + `themes.css` + `token-registry.json` +
//! `generated/unions.ts`.
//!
//! SSOT is `packages/sail/tokens/model/**` (Design Tokens Format groups + themes).
//! Port of the former Node `generate.mjs` — pure Rust, no Node in the path.
//!
//! Invoke: `cargo run -p sail-compiler -- tokens`

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde_json::{Map, Value};

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

/// Preferred L1 tone axis order. Membership itself comes from DTCG
/// `color.tone.<name>.{fg,bg,...}` ramps when present; this only sorts the set.
const TONE_ORDER: &[&str] = &[
    "default",
    "primary",
    "info",
    "success",
    "warning",
    "destructive",
    "accent",
    "muted",
];

/// Fallback tone membership when the model has no `color.tone.*` ramps.
/// SOURCE string in generated unions documents the intended DTCG path for later.
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
    /// `$extensions.ship.channel` — a Tier-2 expression channel. Channel
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
    #[allow(dead_code)]
    dark: bool,
    /// Mode name when this layer is a modes-in-file emit (Phase D). `None` =
    /// legacy single-block theme (twin light/dark ids).
    mode: Option<String>,
    /// Default mode for the theme id (from `$extensions.ship.defaultMode`).
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
    pub registry_json: PathBuf,
    pub unions_ts: PathBuf,
}

/// Default project theme id scaffolded into `tokens/themes/` (modes-in-file).
pub const DEFAULT_PROJECT_THEME_ID: &str = "ship";

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
    /// Stems of project `tokens/themes/*.json`.
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
    /// Project DTCG theme directories (`apps/<app>/tokens/themes`). Every
    /// `*.json` here is always in the emit set; same `themeId` overrides catalog.
    pub project_theme_dirs: Vec<PathBuf>,
    /// When `Some`, only these **catalog** theme ids are emitted (project themes
    /// still always emit). When `None`, all catalog themes emit.
    pub enabled_catalog: Option<Vec<String>>,
    /// When true, write **only** `themes.css` (app B1 delivery). Skips tokens.css,
    /// registry, unions, themes-meta so app out dirs stay free of package litter.
    pub themes_only: bool,
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
    let aliases = load_compat_aliases(model_dir)?;
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
            registry_json: PathBuf::new(),
            unions_ts: PathBuf::new(),
        });
    }

    let registry = build_registry(&base_tokens, &aliases);
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
        registry_json,
        unions_ts,
    })
}

/// Emit `generated/themes-meta.ts` — catalog meta for apply/pickers (Phase D).
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
         * Theme catalog meta from packages/sail/tokens/model/themes/**.\n \
         * Includes modes-in-file (Phase D). Regenerate: sail tokens\n */\n\n",
    );
    out.push_str("export interface ThemeMeta {\n");
    out.push_str("  id: string\n");
    out.push_str("  /** Whether the default mode is a dark surface (data-mode dark). */\n");
    out.push_str("  dark: boolean\n");
    out.push_str("  /** Mode names when modes-in-file; empty = legacy single-block theme. */\n");
    out.push_str("  modes: readonly string[]\n");
    out.push_str("  defaultMode: string | null\n");
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
        out.push_str(&format!(
            "  {{ id: '{id}', dark: {dark}, modes: {modes_lit}, defaultMode: {dm} }},\n"
        ));
    }
    out.push_str("] as const\n\n");
    out.push_str(
        "const BY_ID: Record<string, ThemeMeta> = Object.fromEntries(\n  THEME_META.map((t) => [t.id, t]),\n)\n\n",
    );
    out.push_str(
        "export function themeMeta(id: string): ThemeMeta | undefined {\n  return BY_ID[id]\n}\n",
    );
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
            let css_name = node
                .pointer("/$extensions/ship/cssName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| path_parts.join("-"));
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
                .pointer("/$extensions/ship/channel")
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
fn resolve_value(value: &str, name_index: &NameIndex) -> Result<String> {
    let alias = parse_alias(value);
    if let Some(ref_name) = alias {
        let css_name = if let Some(c) = name_index.path_to_css.get(&ref_name) {
            c.clone()
        } else if name_index.css_to_path.contains_key(&ref_name) {
            ref_name
        } else {
            ref_name.replace('.', "-")
        };
        return Ok(format!("var(--{css_name})"));
    }
    Ok(value.to_string())
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
        .pointer("/$extensions/ship")
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
        .filter(|f| f.ends_with(".json") && f != "compat-aliases.json")
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

        // Modes-in-file only — one theme id, mode blocks under `modes`.
        // No twin-id / flat-theme dual path.
        // Single-mode themes are allowed (e.g. glass light-only, synthwave dark-only)
        // via a single key under `modes` and/or `$extensions.ship.supportedModes`.
        let modes = doc
            .get("modes")
            .and_then(|v| v.as_object())
            .filter(|m| !m.is_empty());
        let Some(modes) = modes else {
            bail!(
                "theme {label}: modes-in-file required (non-empty object `modes`). \
                 Twin light/dark theme ids are not supported. \
                 Single-mode themes declare one key under modes (and optional \
                 $extensions.ship.supportedModes)."
            );
        };

        // Resolve declared mode set: keys under `modes`, optionally intersected
        // with `$extensions.ship.supportedModes` (must be a subset of modes keys).
        let supported_ext: Option<Vec<String>> = doc
            .pointer("/$extensions/ship/supportedModes")
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
                        "theme {label}: $extensions.ship.supportedModes includes {m:?} \
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
                .pointer("/$extensions/ship/dark")
                .and_then(|v| v.as_bool())
                .unwrap_or(mode_name == "dark");
            let mut meta = base_meta.clone();
            meta.file = format!("{label}#mode={mode_name}");
            meta.mode = Some(mode_name.clone());
            meta.default_mode = default_mode.clone();
            meta.dark = mode_ext_dark;
            meta.layer = "theme".into();
            // Selector family (Job 5d / D7–D11):
            //   • `:root[data-theme=…]` — app product theme on documentElement
            //   • `[theme=…]` — author IR for scoped hosts (Studio home cards,
            //     draft cold path, nested previews)
            //   • `[data-theme-preview=…]` — temporary dual-emit for residual
            //     consumers (sail-site picker, tutorial, etc.); not Studio SSOT
            // Default mode also matches bare theme / data-theme (no data-mode)
            // so cold loads without applyTheme still paint the default mode.
            let is_default = default_mode.as_deref() == Some(mode_name.as_str());
            meta.selector = if is_default {
                format!(
                    ":root[data-theme='{id}'][data-mode='{mode_name}'],\n\
                     :root[data-theme='{id}']:not([data-mode]),\n\
                     [theme='{id}'][data-mode='{mode_name}'],\n\
                     [theme='{id}']:not([data-mode]),\n\
                     [data-theme-preview='{id}'][data-mode='{mode_name}'],\n\
                     [data-theme-preview='{id}']:not([data-mode])"
                )
            } else {
                format!(
                    ":root[data-theme='{id}'][data-mode='{mode_name}'],\n\
                     [theme='{id}'][data-mode='{mode_name}'],\n\
                     [data-theme-preview='{id}'][data-mode='{mode_name}']"
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

/// Copy one Sail catalog theme into `project_dir/tokens/themes/<id>.json` (C5).
///
/// Uses `theme_id` (default [`DEFAULT_PROJECT_THEME_ID`]) from
/// `catalog_themes_dir`. Does not overwrite an existing project file.
/// Returns the destination path.
pub fn scaffold_default_project_theme(
    project_dir: &Path,
    catalog_themes_dir: &Path,
    theme_id: Option<&str>,
) -> Result<PathBuf> {
    let id = theme_id
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_PROJECT_THEME_ID);
    validate_theme_id(id)?;
    if !catalog_themes_dir.is_dir() {
        bail!(
            "catalog themes dir not found: {}",
            catalog_themes_dir.display()
        );
    }
    let src = catalog_themes_dir.join(format!("{id}.json"));
    if !src.is_file() {
        bail!(
            "catalog theme not found: {} (need packages/sail/tokens/model/themes/{id}.json)",
            src.display()
        );
    }
    if !path_is_under(&src, catalog_themes_dir)? {
        bail!("catalog theme path escapes catalog dir: {}", src.display());
    }
    let dest_dir = project_dir.join("tokens/themes");
    fs::create_dir_all(&dest_dir).with_context(|| format!("mkdir {}", dest_dir.display()))?;
    let dest = dest_dir.join(format!("{id}.json"));
    if !path_is_under(&dest, &dest_dir)? {
        bail!(
            "project theme path escapes tokens/themes: {}",
            dest.display()
        );
    }
    if dest.is_file() {
        return Ok(dest);
    }
    fs::copy(&src, &dest)
        .with_context(|| format!("copy {} → {}", src.display(), dest.display()))?;
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
         \x20\x20// Sail app config — theme section (T0).\n\
         \x20\x20// `default` / `defaultMode` drive first paint (data-theme / data-mode).\n\
         \x20\x20// `enabled` is the catalog allowlist for app CSS emit:\n\
         \x20\x20//   omit → default theme ship; [] → no catalog; [\"id\"] → those; \"all\" → full.\n\
         \x20\x20// Project themes under tokens/themes/ always emit. Package generate\n\
         \x20\x20// (shared @ship/tokens) always emits full catalog + project overrides.\n\
         \x20\x20\"theme\": {{\n\
         \x20\x20\x20\x20\"default\": \"{theme_id}\",\n\
         \x20\x20\x20\x20\"defaultMode\": \"light\",\n\
         \x20\x20\x20\x20\"enabled\": [\"{theme_id}\"]\n\
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
/// project_ids = stems of project tokens/themes/*.json
/// catalog_allow = all | list | default [ship] | explicit []
/// effective_catalog = catalog_allow ∩ existing catalog files
/// emit_set = project ∪ effective_catalog (project wins on id at emit)
/// ```
pub fn effective_theme_ids(workspace_root: &Path, project_dir: &Path) -> Result<EffectiveThemeIds> {
    let cfg = load_theme_config(project_dir)?;
    let project = list_theme_ids_in_dir(&project_dir.join("tokens/themes"));
    let catalog_dir = workspace_root.join("packages/sail/tokens/model/themes");
    let catalog_all = list_theme_ids_in_dir(&catalog_dir);
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

/// Config-loud validation: `default` ∈ effective set; `defaultMode` exists on
/// the theme when the DTCG file is loadable.
pub fn validate_theme_config(
    workspace_root: &Path,
    project_dir: &Path,
    cfg: &ThemeConfig,
) -> Result<()> {
    let effective = effective_theme_ids(workspace_root, project_dir)?;
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
    workspace_root: &Path,
    project_dir: &Path,
    theme_id: &str,
) -> Result<Vec<String>> {
    validate_theme_id(theme_id)?;
    let candidates = [
        project_dir.join(format!("tokens/themes/{theme_id}.json")),
        workspace_root.join(format!("packages/sail/tokens/model/themes/{theme_id}.json")),
    ];
    for path in candidates {
        if !path.is_file() {
            continue;
        }
        let text =
            fs::read_to_string(&path).with_context(|| format!("read theme {}", path.display()))?;
        let doc: Value = serde_json::from_str(&text)
            .with_context(|| format!("parse theme JSON {}", path.display()))?;
        if let Some(modes) = doc.get("modes").and_then(|m| m.as_object()) {
            let mut keys: Vec<String> = modes.keys().cloned().collect();
            keys.sort_by(|a, b| mode_rank(a).cmp(&mode_rank(b)).then_with(|| a.cmp(b)));
            return Ok(keys);
        }
        // No modes object — nothing to validate against.
        return Ok(Vec::new());
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
    let theme_cfg = load_theme_config(project_dir)?;
    validate_theme_config(workspace_root, project_dir, &theme_cfg)?;
    let mut project_theme_dirs = Vec::new();
    let themes_dir = project_dir.join("tokens/themes");
    if themes_dir.is_dir() {
        project_theme_dirs.push(themes_dir);
    }
    Ok(GenerateOptions {
        project_theme_dirs,
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
/// Uses [`options_for_project`] (effective set only) and writes **only**
/// `themes.css` (`themes_only`). Does **not** touch the shared package
/// `packages/sail/tokens/src`. Survives SPA `publish_build` prune (foreign dir).
pub fn generate_app_delivery(workspace_root: &Path, project_dir: &Path) -> Result<GenerateReport> {
    let model_dir = workspace_root.join("packages/sail/tokens/model");
    let out_dir = app_themes_out_dir(project_dir);
    let mut opts = options_for_project(workspace_root, project_dir)?;
    opts.themes_only = true;
    generate_with_options(&model_dir, &out_dir, &opts)
}

/// Generate **package** delivery into `packages/sail/tokens/src` (full catalog).
pub fn generate_package_delivery(
    workspace_root: &Path,
    project_dir: Option<&Path>,
) -> Result<GenerateReport> {
    let model_dir = workspace_root.join("packages/sail/tokens/model");
    let out_dir = workspace_root.join("packages/sail/tokens/src");
    if let Some(project) = project_dir {
        let opts = options_for_shared_package(workspace_root, project)?;
        generate_with_options(&model_dir, &out_dir, &opts)
    } else {
        generate(&model_dir, &out_dir)
    }
}

/// True when the project has theme SSOT that should drive a cold-start generate
/// (`tokens/themes/*.json` or a project-root `sail.config` with a `theme` key).
pub fn project_wants_theme_generate(project_dir: &Path) -> bool {
    let themes = project_dir.join("tokens/themes");
    if themes.is_dir() {
        if let Ok(rd) = fs::read_dir(&themes) {
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

fn emit_block(selector: &str, tokens: &[Token], indent: &str) -> String {
    if tokens.is_empty() {
        return String::new();
    }
    let mut lines = vec![format!("{selector} {{")];
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
 * SSOT: `$extensions.ship.channel` + `$description` on the token in
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
    let block = emit_block(&layer.meta.selector, &layer.tokens, "  ");
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

/// L1 host `tone` prop axis (component-library convention — not a token family).
/// Historically derived from removed `color.tone.*` ramp tokens; membership is
/// the fixed [`TONE_FALLBACK`] list so props/docs stay aligned without ramp tokens.
fn tone_names_from_tokens(_tokens: &[Token]) -> (Vec<String>, &'static str) {
    (
        TONE_FALLBACK.iter().map(|s| (*s).to_string()).collect(),
        "l1-convention:TONE_FALLBACK (host prop axis; not design-token ramps)",
    )
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

    // Tone — data set (not a language feature); membership from DTCG or fallback.
    out.push_str("/** Where {@link TONE_NAMES} membership is sourced from. */\n");
    out.push_str("export const TONE_NAMES_SOURCE = ");
    out.push_str(&ts_quote(tone_source));
    out.push_str(" as const\n\n");
    emit_const_and_type(
        &mut out,
        "TONE_NAMES",
        "ToneName",
        &tone_names,
        "L1 host tone prop axis (component convention). Not design-token ramps.",
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
                    "$extensions": { "ship": { "cssName": "space-4" } }
                }
            },
            "color": {
                "primary": {
                    "default": {
                        "$value": "oklch(0.5 0.1 250)",
                        "$type": "color",
                        "$extensions": { "ship": { "cssName": "primary" } }
                    }
                },
                "brand": {
                    "$value": "{color.primary.default}",
                    "$type": "color",
                    "$extensions": { "ship": { "cssName": "brand" } }
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
  "$extensions": { "ship": { "selector": ":root", "layer": "base" } },
  "space": {
    "4": {
      "$value": "16px",
      "$type": "dimension",
      "$extensions": { "ship": { "cssName": "space-4" } }
    }
  },
  "control": {
    "h-sm": {
      "$value": "28px",
      "$type": "dimension",
      "$extensions": { "ship": { "cssName": "control-h-sm" } }
    },
    "h-md": {
      "$value": "34px",
      "$type": "dimension",
      "$extensions": { "ship": { "cssName": "control-h-md" } }
    }
  },
  "color": {
    "primary": {
      "$value": "oklch(0.5 0.1 250)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "primary" } }
    },
    "tone": {
      "default": {
        "fg": {
          "$value": "oklch(0.2 0 0)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "foreground" } }
        },
        "bg": {
          "$value": "oklch(0.9 0 0)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "muted" } }
        }
      },
      "primary": {
        "fg": {
          "$value": "{primary}",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        },
        "bg": {
          "$value": "{primary}",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "hover-bg" } }
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
    "ship": {
      "layer": "theme",
      "themeId": "paper",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.4 0.05 75)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        }
      }
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.7 0.08 75)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
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
            "/* themes chrome */\nbody[data-theme='paper'] { background: white; }\n",
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
            themes_css.contains("data-theme='paper'][data-mode='light'")
                || themes_css.contains(":root[data-theme='paper']"),
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
    fn tone_names_are_l1_host_convention_not_token_ramps() {
        // color.tone.* ramps are gone — ToneName is the fixed L1 host prop list.
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
        assert!(source.contains("l1-convention"), "source={source}");
        assert!(source.contains("not design-token"), "source={source}");
        // Even if a caller stuffed tone paths into the token list, membership
        // stays the fixed convention (no dual-path derivation from ramps).
        let with_stale_ramp = vec![Token {
            path: "color.tone.primary.bg".into(),
            css_name: "tone-primary-bg".into(),
            r#type: "color".into(),
            value: "oklch(0.9 0.05 250)".into(),
            css_value: "oklch(0.9 0.05 250)".into(),
            channel: false,
            description: None,
        }];
        let (names2, source2) = tone_names_from_tokens(&with_stale_ramp);
        assert_eq!(names2, names);
        assert_eq!(source2, source);
    }

    #[test]
    fn component_sources_merge_into_base_with_stable_css_names() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{
  "$extensions": { "ship": { "selector": ":root", "layer": "base" } },
  "color": {
    "surface": {
      "1": {
        "$value": "oklch(0.98 0 0)",
        "$type": "color",
        "$extensions": { "ship": { "cssName": "surface-1" } }
      }
    },
    "foreground": {
      "$value": "oklch(0.2 0 0)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "foreground" } }
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
        "$extensions": { "ship": { "cssName": "card" } }
      },
      "fg": {
        "$value": "{color.foreground}",
        "$type": "color",
        "$extensions": { "ship": { "cssName": "card-foreground" } }
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
    "ship": {
      "layer": "theme",
      "themeId": "glass",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "component": {
        "card": {
          "bg": {
            "$value": "oklch(1 0 0 / 55%)",
            "$type": "color",
            "$extensions": { "ship": { "cssName": "card" } }
          }
        }
      }
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "component": {
        "card": {
          "bg": {
            "$value": "oklch(0.2 0 0 / 55%)",
            "$type": "color",
            "$extensions": { "ship": { "cssName": "card" } }
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
  "$extensions": { "ship": { "selector": ":root", "layer": "base" } },
  "color": {
    "foreground": {
      "$value": "oklch(0.2 0 0)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "foreground" } }
    }
  },
  "control": {
    "frame": {
      "$value": "0 0 0 0 transparent",
      "$type": "string",
      "$description": "Control framing — transparent-shadow no-op.",
      "$extensions": { "ship": { "cssName": "control-frame", "channel": true } }
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
  "$extensions": { "ship": { "selector": ":root", "layer": "base" } },
  "color": {
    "primary": {
      "$value": "oklch(0.5 0.1 250)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "primary" } }
    }
  }
}
"#,
        );
        write_file(
            &model.join("themes/paper.json"),
            r#"{
  "$extensions": { "ship": { "layer": "theme", "themeId": "paper", "defaultMode": "light" } },
  "modes": { "light": { "color": { "primary": { "$value": "oklch(0.4 0.05 75)", "$type": "color", "$extensions": { "ship": { "cssName": "primary" } } } } }, "dark": { "$extensions": { "ship": { "dark": true } }, "color": { "primary": { "$value": "oklch(0.7 0.1 75)", "$type": "color", "$extensions": { "ship": { "cssName": "primary" } } } } } }
}
"#,
        );
        write_file(
            &project.join("tokens/themes/custom.json"),
            r#"{
  "$extensions": { "ship": { "layer": "theme", "themeId": "custom", "defaultMode": "light" } },
  "modes": { "light": { "color": { "primary": { "$value": "oklch(0.55 0.2 140)", "$type": "color", "$extensions": { "ship": { "cssName": "primary" } } } } }, "dark": { "$extensions": { "ship": { "dark": true } }, "color": { "primary": { "$value": "oklch(0.6 0.15 140)", "$type": "color", "$extensions": { "ship": { "cssName": "primary" } } } } } }
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
        assert!(css.contains("data-theme='custom'"));
        assert!(!css.contains("data-theme='paper'"));

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
  "$extensions": { "ship": { "layer": "theme", "themeId": "ship", "defaultMode": "light" } },
  "modes": { "light": { "color": { "primary": { "$value": "oklch(0.5 0.1 250)", "$type": "color", "$extensions": { "ship": { "cssName": "primary" } } } } }, "dark": { "$extensions": { "ship": { "dark": true } }, "color": { "primary": { "$value": "oklch(0.3 0.05 250)", "$type": "color", "$extensions": { "ship": { "cssName": "primary" } } } } } }
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
        assert!(bare_css.contains("data-theme='ship'"));
        assert!(!bare_css.contains("data-theme='paper'"));

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
  "$extensions": { "ship": { "selector": ":root", "layer": "base" } },
  "color": {
    "primary": {
      "$value": "oklch(0.5 0.1 250)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "primary" } }
    }
  }
}
"#,
        );
        write_file(
            &model.join("themes/paper.json"),
            r#"{
  "$extensions": {
    "ship": {
      "layer": "theme",
      "themeId": "paper",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.4 0.05 75)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        }
      }
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.4 0.05 75)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
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
    "ship": {
      "layer": "theme",
      "themeId": "glass",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.8 0.05 200)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        }
      }
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.8 0.05 200)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
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
    "ship": {
      "layer": "theme",
      "themeId": "paper",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.12 0.34 99)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        }
      }
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.12 0.34 99)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
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
    "ship": {
      "layer": "theme",
      "themeId": "custom",
      "defaultMode": "light"
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.55 0.2 140)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        }
      }
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.55 0.2 140)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
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
            themes_css.contains(":root[data-theme='custom']"),
            "project-only theme must emit: {themes_css}"
        );
        assert!(
            !themes_css.contains("data-theme='glass'"),
            "disabled catalog theme must not emit: {themes_css}"
        );
        // Catalog-only generate still emits full catalog.
        let full = generate(&model, &dir.path().join("src-full")).unwrap();
        assert_eq!(full.theme_count, 2);
        let full_css = fs::read_to_string(&full.themes_css).unwrap();
        assert!(full_css.contains("data-theme='glass'"));
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
            r#"{ "$extensions": { "ship": { "themeId": "ship", "defaultMode": "light" } }, "modes": { "light": {}, "dark": {} } }"#,
        );
        let dest = scaffold_default_project_theme(&project, &catalog, None).unwrap();
        assert!(dest.is_file());
        assert_eq!(dest.file_name().and_then(|n| n.to_str()), Some("ship.json"));
        // Second call does not overwrite (even if source changes).
        write_file(&catalog.join("ship.json"), r#"{ "changed": true }"#);
        let again = scaffold_default_project_theme(&project, &catalog, None).unwrap();
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
        assert!(scaffold_default_project_theme(&project, &catalog, Some("../x")).is_err());
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
            &model.join("themes"),
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
            css.contains("data-theme='ship'") || css.contains("data-theme=\"ship\""),
            "default theme selector must emit"
        );
        assert!(
            !css.contains("data-theme='synthwave'"),
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
            shared_css.contains("data-theme='synthwave'")
                || shared_css.contains("data-theme=\"synthwave\""),
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
        assert!(!app_css.contains("data-theme='synthwave'"));
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
            r#"{ "$extensions": { "ship": { "themeId": "x", "defaultMode": "light" } }, "modes": { "light": {}, "dark": {} } }"#,
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
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{ "$extensions": { "ship": { "selector": ":root", "layer": "base" } } }"#,
        );
        // Flat twin-style file must fail.
        write_file(
            &model.join("themes/ship-light.json"),
            r#"{
  "$extensions": { "ship": { "themeId": "ship-light", "layer": "theme" } },
  "color": { "primary": { "$value": "oklch(0.5 0.1 50)", "$type": "color", "$extensions": { "ship": { "cssName": "primary" } } } }
}
"#,
        );
        let err = generate(&model, &out).unwrap_err().to_string();
        assert!(
            err.contains("modes-in-file") || err.contains("twin"),
            "expected modes/twin error, got: {err}"
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
            r#"{ "$extensions": { "ship": { "selector": ":root", "layer": "base" } } }"#,
        );
        write_file(
            &model.join("themes/glass.json"),
            r#"{
  "$extensions": {
    "ship": {
      "themeId": "glass",
      "defaultMode": "light",
      "supportedModes": ["light"]
    }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.62 0.16 50)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
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
            css.contains("data-theme='glass'][data-mode='light']"),
            "expected light selector, got: {css}"
        );
        assert!(
            !css.contains("data-theme='glass'][data-mode='dark']"),
            "single-mode glass must not emit dark block"
        );
        let meta = fs::read_to_string(out.join("generated/themes-meta.ts")).unwrap();
        assert!(
            meta.contains("id: 'glass'") && meta.contains("modes: ['light']"),
            "themes-meta must list only light for glass, got: {meta}"
        );
    }

    #[test]
    fn reject_supported_modes_missing_body() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("model");
        let out = dir.path().join("src");
        write_file(
            &model.join("base.json"),
            r#"{ "$extensions": { "ship": { "selector": ":root", "layer": "base" } } }"#,
        );
        write_file(
            &model.join("themes/broken.json"),
            r#"{
  "$extensions": {
    "ship": {
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
            r#"{ "$extensions": { "ship": { "selector": ":root", "layer": "base" } } }"#,
        );
        write_file(
            &model.join("themes/uneven.json"),
            r#"{
  "$extensions": { "ship": { "themeId": "uneven", "defaultMode": "light" } },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.7 0.1 100)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        }
      }
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.3 0.1 100)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        },
        "surface": {
          "0": {
            "$value": "oklch(0.2 0.02 250)",
            "$type": "color",
            "$extensions": { "ship": { "cssName": "surface-0" } }
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
  "$extensions": { "ship": { "selector": ":root", "layer": "base" } },
  "color": {
    "primary": {
      "$value": "oklch(0.5 0.1 250)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "primary" } }
    }
  }
}
"#,
        );
        write_file(
            &model.join("themes/ship.json"),
            r#"{
  "$extensions": {
    "ship": { "layer": "theme", "themeId": "ship", "defaultMode": "light" }
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.7 0.16 58)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
        }
      }
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.77 0.16 70)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } }
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
            css.contains("data-theme='ship'][data-mode='light'")
                || css.contains(r#"data-theme='ship'][data-mode='light'"#),
            "light mode selector missing: {css}"
        );
        assert!(
            css.contains("data-theme='ship'][data-mode='dark'"),
            "dark mode selector missing: {css}"
        );
        assert!(
            css.contains("[theme='ship'][data-mode='light']")
                || css.contains("[theme='ship']"),
            "author IR [theme=id] selector missing: {css}"
        );
        assert!(
            css.contains(":not([data-mode])"),
            "default mode must match bare data-theme / theme"
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
  "$extensions": { "ship": { "selector": ":root", "layer": "base" } },
  "color": {
    "primary": {
      "$value": "oklch(0.5 0.1 250)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "primary" } }
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
    "ship": {
      "layer": "theme",
      "themeId": "paper",
      "defaultMode": "light",
    },
  },
  "modes": {
    "light": {
      "color": {
        "primary": {
          "$value": "oklch(0.4 0.05 75)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } },
        },
      },
    },
    "dark": {
      "$extensions": { "ship": { "dark": true } },
      "color": {
        "primary": {
          "$value": "oklch(0.7 0.08 75)",
          "$type": "color",
          "$extensions": { "ship": { "cssName": "primary" } },
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
}
