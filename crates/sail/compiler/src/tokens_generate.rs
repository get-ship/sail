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

use anyhow::{anyhow, bail, Context, Result};
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

/// Stable theme catalog order (matches former generate.mjs).
const THEME_ORDER: &[&str] = &[
    "ship-dark",
    "paper",
    "halo",
    "console",
    "contrast",
    "neutral-light",
    "neutral-dark",
    "glass",
    "brut",
    "8bit",
    "synthwave",
    "ship-light",
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

const SPECIAL_LAYERS: &[&str] = &[
    "base",
    "reduced-motion",
    "dark",
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

/// Read DTCG under `model_dir` and write the three artifacts into `out_dir`.
pub fn generate(model_dir: &Path, out_dir: &Path) -> Result<GenerateReport> {
    let (layers, name_index, base_tokens) = load_base_layers(model_dir)?;
    if layers.is_empty() {
        bail!("No model/*.json files found in {}", model_dir.display());
    }
    let aliases = load_compat_aliases(model_dir)?;
    let themes = load_theme_layers(model_dir, &name_index)?;

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
    for name in ["density-compact", "density-spacious", "dark"] {
        if let Some(l) = by_layer.get(name) {
            css_parts.push(emit_layer(l));
            css_parts.push(String::new());
        }
    }
    if by_layer.contains_key("dark") && !aliases.is_empty() {
        css_parts.push("/* ─── compat aliases under .dark ─── */".into());
        css_parts.push(emit_compat_aliases(".dark", &aliases));
        css_parts.push(String::new());
    }

    let css = collapse_blank_lines(&css_parts.join("\n"));
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;
    let tokens_css = out_dir.join("tokens.css");
    write_with_trailing_newline(&tokens_css, &css)?;

    // themes.css
    let mut theme_by_id: HashMap<String, &Layer> = HashMap::new();
    for t in &themes {
        if let Some(id) = &t.meta.theme_id {
            theme_by_id.insert(id.clone(), t);
        }
    }
    let mut ordered_themes: Vec<&Layer> = Vec::new();
    for id in THEME_ORDER {
        if let Some(t) = theme_by_id.get(*id) {
            ordered_themes.push(t);
        }
    }
    for t in &themes {
        let id = t.meta.theme_id.as_deref().unwrap_or("");
        if !THEME_ORDER.contains(&id) {
            ordered_themes.push(t);
        }
    }

    let mut theme_parts: Vec<String> = vec![THEMES_HEADER.to_string()];
    for t in &ordered_themes {
        theme_parts.push(emit_layer(t));
        theme_parts.push(String::new());
    }
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

    let registry = build_registry(&base_tokens, &aliases);
    let color_count = registry["colors"].as_array().map(|a| a.len()).unwrap_or(0);
    let path_count = registry["paths"].as_object().map(|o| o.len()).unwrap_or(0);
    let token_count = registry["names"].as_array().map(|a| a.len()).unwrap_or(0);
    let registry_json = out_dir.join("token-registry.json");
    let reg_text = serde_json::to_string_pretty(&registry)? + "\n";
    fs::write(&registry_json, reg_text)
        .with_context(|| format!("write {}", registry_json.display()))?;

    let theme_ids: Vec<String> = ordered_themes
        .iter()
        .filter_map(|t| t.meta.theme_id.clone())
        .collect();
    let unions_body = build_unions_ts(&base_tokens, &registry, &theme_ids);
    let unions_dir = out_dir.join("generated");
    fs::create_dir_all(&unions_dir)
        .with_context(|| format!("create {}", unions_dir.display()))?;
    let unions_ts = unions_dir.join("unions.ts");
    write_with_trailing_newline(&unions_ts, &unions_body)?;

    Ok(GenerateReport {
        token_count,
        theme_count: ordered_themes.len(),
        color_count,
        path_count,
        layers: layers.iter().map(|l| l.meta.layer.clone()).collect(),
        tokens_css,
        themes_css,
        registry_json,
        unions_ts,
    })
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
    serde_json::from_str(&text).with_context(|| format!("parse JSON {}", path.display()))
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
    LayerMeta {
        file: file.to_string(),
        description,
        selector,
        media,
        layer,
        theme_id,
        dark,
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
        deep_merge_objects(base_doc, &doc).with_context(|| {
            format!(
                "merge component source {} into base",
                path.display()
            )
        })?;
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

fn load_theme_layers(model_dir: &Path, name_index: &NameIndex) -> Result<Vec<Layer>> {
    let themes_dir = model_dir.join("themes");
    if !themes_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<String> = fs::read_dir(&themes_dir)
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
        let mut meta = layer_meta(&doc, &format!("themes/{file}"), fallback);
        if meta.theme_id.is_none() {
            meta.theme_id = Some(fallback.to_string());
        }
        if meta.selector.is_empty() || meta.selector == ":root" {
            let id = meta.theme_id.as_deref().unwrap_or(fallback);
            meta.selector =
                format!(":root[data-theme='{id}'],\n[data-theme-preview='{id}']");
        }
        meta.layer = "theme".into();
        let raw = flatten_tokens(&doc, &[], None);
        let mut tokens = Vec::with_capacity(raw.len());
        for mut t in raw {
            t.css_value = resolve_value(&t.value, name_index)?;
            tokens.push(t);
        }
        themes.push(Layer { meta, tokens });
    }
    Ok(themes)
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

    let (control_h_tokens, control_h_steps) = scale_from_prefix(
        &names,
        "control-h-",
        &["2xs", "xs", "sm", "md", "lg", "xl"],
    );
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
            &model.join("dark.json"),
            r#"{
  "$description": "dark",
  "$extensions": { "ship": { "selector": ".dark", "layer": "dark" } },
  "color": {
    "primary": {
      "$value": "oklch(0.7 0.1 250)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "primary" } }
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
      "selector": ":root[data-theme='paper'],\n[data-theme-preview='paper']"
    }
  },
  "color": {
    "primary": {
      "$value": "oklch(0.4 0.05 75)",
      "$type": "color",
      "$extensions": { "ship": { "cssName": "primary" } }
    }
  }
}
"#,
        );
        write_file(&model.join("chrome.css"), "/* chrome */\nbody { margin: 0; }\n");
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
        assert!(tokens_css.contains(".dark {"));
        assert!(tokens_css.contains("/* chrome */"));

        let themes_css = fs::read_to_string(&report.themes_css).unwrap();
        assert!(themes_css.contains(":root[data-theme='paper']"));
        assert!(themes_css.contains("--primary:"));
        assert!(themes_css.contains("/* themes chrome */"));

        let reg: Value =
            serde_json::from_str(&fs::read_to_string(&report.registry_json).unwrap()).unwrap();
        assert_eq!(reg["version"], 2);
        assert!(reg["names"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some("space-4")));
        assert!(reg["names"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some("app-primary")));
        assert_eq!(reg["tokens"]["app-primary"]["aliasOf"], "primary");
        assert_eq!(reg["paths"]["space.4"], "space-4");
        assert!(reg["colors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some("primary")));

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
      "selector": ":root[data-theme='glass']"
    }
  },
  "component": {
    "card": {
      "bg": {
        "$value": "oklch(1 0 0 / 55%)",
        "$type": "color",
        "$extensions": { "ship": { "cssName": "card" } }
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
        assert!(block.contains("--control-frame: 0 0 0 0 transparent;"), "{css}");
        assert!(block.contains("Control framing — transparent-shadow no-op."));
        // … and NOT duplicated in the base block.
        assert_eq!(css.matches("--control-frame:").count(), 1, "{css}");
        // Non-channel tokens stay in the base block, before the banner.
        let fg = css.find("--foreground:").unwrap();
        assert!(fg < banner, "{css}");
    }
}
