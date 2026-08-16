//! App-vendored fonts + framework used-family materialization (SPEC-FONTS-1).
//!
//! Layout under the project (written by ops FontAdd):
//! ```text
//! <project>/fonts/
//!   vendor-manifest.json
//!   fonts.css          # @font-face rules with url('./….woff2')
//!   <family>-….woff2
//! ```
//!
//! Framework faces live in `@ship/tokens` (`fonts-manifest.json` + vendored
//! woff2). A role is USED when reachable from the compiled token/style graph
//! (theme defaults count — `html { font-family: var(--font-body) }` seeds
//! `body` even if no author writes `fontFamily(body)`). Unused families are
//! stripped from sail-base `@font-face` before the CSS asset plugin emits.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Project-relative fonts directory produced by SailTheme/FontAdd.
pub const APP_FONTS_DIR: &str = "fonts";

const FONTS_MANIFEST_JSON: &str =
    include_str!("../../../../packages/sail/tokens/src/fonts-manifest.json");

#[derive(Debug, Clone, Deserialize)]
struct ManifestFile {
    faces: Vec<FontFace>,
}

/// One vendored / Fontsource-pinned face.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontFace {
    pub family: String,
    pub role: String,
    pub file: String,
    pub package_id: String,
    pub version: String,
    pub license: String,
}

fn manifest() -> &'static ManifestFile {
    static CACHE: OnceLock<ManifestFile> = OnceLock::new();
    CACHE.get_or_init(|| {
        serde_json::from_str(FONTS_MANIFEST_JSON)
            .unwrap_or_else(|e| panic!("fonts-manifest.json parse failed: {e}"))
    })
}

/// Framework FONTS catalog (id / version / license / role / file).
pub fn framework_faces() -> &'static [FontFace] {
    &manifest().faces
}

/// Basename → face.
pub fn face_by_file(file: &str) -> Option<&'static FontFace> {
    framework_faces().iter().find(|f| f.file == file)
}

/// Known framework woff2 basenames.
pub fn framework_font_files() -> HashSet<String> {
    framework_faces().iter().map(|f| f.file.clone()).collect()
}

/// Copy `<project>/fonts/**` → `<out_dir>/fonts/**` when present.
pub fn stage_app_fonts(project: &Path, out_dir: &Path) -> Result<()> {
    let src = project.join(APP_FONTS_DIR);
    if !src.is_dir() {
        return Ok(());
    }
    let dest = out_dir.join(APP_FONTS_DIR);
    copy_dir_recursive(&src, &dest)
        .with_context(|| format!("copy {} → {}", src.display(), dest.display()))?;
    Ok(())
}

pub(crate) fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            std::fs::copy(&from, &to).with_context(|| format!("copy file {}", from.display()))?;
        }
    }
    Ok(())
}

/// Formerly emitted `<link href="/{ns}/fonts/fonts.css">`. That path 404s in
/// dev (`serve_scaffold_asset` rejects nested segments). Fonts load via entry
/// `import './fonts/fonts.css'` instead. Kept as `None` so callers compile;
/// staged `.sail/fonts/**` is for offline render only.
pub fn app_fonts_css_href(_project: &Path, _url_namespace: &str) -> Option<String> {
    None
}

fn tokens_src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../packages/sail/tokens/src")
}

/// Theme ids in the emit set. Empty = thin (no catalog remaps). `"all"` = every catalog theme.
pub fn enabled_theme_ids(project: Option<&Path>) -> Vec<String> {
    let Some(project) = project else {
        return Vec::new();
    };
    for name in ["sail.config.jsonc", "sail.config.json"] {
        let path = project.join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let cleaned = if name.ends_with('c') {
            strip_jsonc(&text)
        } else {
            text
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&cleaned) else {
            continue;
        };
        return theme_enabled_from_value(&v);
    }
    Vec::new()
}

fn theme_enabled_from_value(v: &serde_json::Value) -> Vec<String> {
    let Some(theme) = v.get("theme") else {
        return Vec::new();
    };
    match theme.get("enabled") {
        Some(serde_json::Value::String(s)) if s == "all" => vec!["all".into()],
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn strip_jsonc(input: &str) -> String {
    jsonc_to_json(input)
}

fn jsonc_to_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
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
            out.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    for nc in chars.by_ref() {
                        if nc == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut prev = '\0';
                    for nc in chars.by_ref() {
                        if prev == '*' && nc == '/' {
                            break;
                        }
                        prev = nc;
                    }
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

/// Used framework woff2 basenames for a project (or framework-only when `project` is None).
pub fn used_font_files_for_project(project: Option<&Path>) -> HashSet<String> {
    let dir = tokens_src_dir();
    let tokens = std::fs::read_to_string(dir.join("tokens.css")).unwrap_or_default();
    let base = std::fs::read_to_string(dir.join("sail-base.css")).unwrap_or_default();
    let enabled = enabled_theme_ids(project);
    let themes = if enabled.is_empty() {
        String::new()
    } else {
        std::fs::read_to_string(dir.join("themes.css")).unwrap_or_default()
    };
    // Bindings in fonts.roles are not usage. Used-set is token-graph
    // reachability (theme defaults + compiled style extra_roles).
    used_framework_font_files(&tokens, &themes, &base, &enabled, &[])
}

/// Token-graph used-set. A family ships iff a reachable var resolves to it.
pub fn used_framework_font_files(
    tokens_css: &str,
    themes_css: &str,
    base_css: &str,
    enabled_themes: &[String],
    extra_roles: &[String],
) -> HashSet<String> {
    let mut defs = parse_custom_props(tokens_css);
    if !enabled_themes.is_empty() {
        let filtered = filter_theme_css(themes_css, enabled_themes);
        for (k, v) in parse_custom_props(&filtered) {
            defs.insert(k, v);
        }
    }
    let mut used_vars: BTreeSet<String> = extract_var_names(base_css);
    for role in extra_roles {
        used_vars.insert(format!("{role}-font"));
        used_vars.insert(format!("font-{role}"));
        used_vars.insert(format!("font-{role}-app"));
    }
    // Document-floor seeds are already in sail-base; keep walking aliases.
    let mut stack: Vec<String> = used_vars.iter().cloned().collect();
    while let Some(name) = stack.pop() {
        let Some(value) = defs.get(&name) else {
            continue;
        };
        for dep in extract_var_names(value) {
            if used_vars.insert(dep.clone()) {
                stack.push(dep);
            }
        }
    }
    let mut files = HashSet::new();
    let faces = framework_faces();
    for name in &used_vars {
        if let Some(value) = defs.get(name) {
            for face in faces {
                if value.contains(&face.family) {
                    files.insert(face.file.clone());
                }
            }
        }
        // Role-named vars (`--font-body-app`, `--heading-font`, `--font-press-start`).
        for face in faces {
            if name == &face.role
                || name == &format!("{}-font", face.role)
                || name == &format!("font-{}", face.role)
                || name == &format!("font-{}-app", face.role)
            {
                files.insert(face.file.clone());
            }
        }
    }
    files
}

fn parse_custom_props(css: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' && is_ident_start(bytes[i + 2]) {
            let name_start = i + 2;
            let mut j = name_start;
            while j < bytes.len() && is_ident_cont(bytes[j]) {
                j += 1;
            }
            let mut k = j;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if bytes.get(k) == Some(&b':') {
                k += 1;
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                let val_start = k;
                while k < bytes.len() && bytes[k] != b';' && bytes[k] != b'}' {
                    k += 1;
                }
                let name = css[name_start..j].to_string();
                let value = css[val_start..k].trim().to_string();
                out.insert(name, value);
                i = k;
                continue;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn extract_var_names(css: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes = css.as_bytes();
    let needle = b"var(--";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let start = i + needle.len();
            let mut j = start;
            while j < bytes.len() && is_ident_cont(bytes[j]) {
                j += 1;
            }
            if j > start {
                out.insert(css[start..j].to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_lowercase() || b == b'_'
}

fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn filter_theme_css(themes_css: &str, enabled: &[String]) -> String {
    if enabled.iter().any(|t| t == "all") {
        return themes_css.to_string();
    }
    if enabled.is_empty() {
        return String::new();
    }
    // Keep a rule block if any enabled id appears as [theme='id'] / [theme="id"].
    let mut out = String::new();
    for chunk in themes_css.split("/* ─── theme:") {
        if chunk.is_empty() {
            continue;
        }
        let id = chunk.split_whitespace().next().unwrap_or("");
        if enabled.iter().any(|e| e == id) || enabled_selector_hit(chunk, enabled) {
            out.push_str("/* ─── theme:");
            out.push_str(chunk);
        }
    }
    if out.is_empty() {
        // Fallback: include any substring match of [theme='…']
        for id in enabled {
            let needle_a = format!("[theme='{id}']");
            let needle_b = format!("[theme=\"{id}\"]");
            if themes_css.contains(&needle_a) || themes_css.contains(&needle_b) {
                return themes_css.to_string();
            }
        }
    }
    out
}

fn enabled_selector_hit(chunk: &str, enabled: &[String]) -> bool {
    enabled.iter().any(|id| {
        chunk.contains(&format!("[theme='{id}']")) || chunk.contains(&format!("[theme=\"{id}\"]"))
    })
}

/// Drop `@font-face` rules whose woff2 basename is a known framework face
/// not in `used_files`. Other `@font-face` (app-vendored) stay.
pub fn strip_unused_framework_faces(css: &str, used_files: &HashSet<String>) -> String {
    let known = framework_font_files();
    let bytes = css.as_bytes();
    let needle = b"@font-face";
    let mut out = String::with_capacity(css.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + needle.len() <= bytes.len() && &bytes[i..i + needle.len()] == needle {
            let block_start = i;
            let Some(open) = bytes[i..].iter().position(|&b| b == b'{') else {
                out.push_str(&css[i..]);
                break;
            };
            let mut depth = 0i32;
            let mut j = i + open;
            while j < bytes.len() {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            j += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            let block = &css[block_start..j];
            let drop = font_face_file(block)
                .is_some_and(|file| known.contains(&file) && !used_files.contains(&file));
            if !drop {
                out.push_str(block);
            }
            i = j;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn font_face_file(block: &str) -> Option<String> {
    let bytes = block.as_bytes();
    let needle = b"url(";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let mut k = i + needle.len();
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k < bytes.len() && (bytes[k] == b'"' || bytes[k] == b'\'') {
                let q = bytes[k];
                k += 1;
                let start = k;
                while k < bytes.len() && bytes[k] != q {
                    k += 1;
                }
                let url = &block[start..k];
                let base = url.rsplit('/').next().unwrap_or(url);
                if base.ends_with(".woff2") || base.ends_with(".woff") {
                    return Some(base.to_string());
                }
            }
        }
        i += 1;
    }
    None
}

/// Audit row for RESULT: family, role, package, version, license, bytes, used.
#[derive(Debug, Clone)]
pub struct FontAuditRow {
    pub family: String,
    pub role: String,
    pub package_id: String,
    pub version: String,
    pub license: String,
    pub file: String,
    pub bytes: u64,
    pub used: bool,
}

pub fn audit_used_fonts(project: Option<&Path>) -> Vec<FontAuditRow> {
    let used = used_font_files_for_project(project);
    let dir = tokens_src_dir().join("fonts");
    framework_faces()
        .iter()
        .map(|face| {
            let path = dir.join(&face.file);
            let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            FontAuditRow {
                family: face.family.clone(),
                role: face.role.clone(),
                package_id: face.package_id.clone(),
                version: face.version.clone(),
                license: face.license.clone(),
                file: face.file.clone(),
                bytes,
                used: used.contains(&face.file),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn stages_woff2_and_css() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("app");
        let fonts = project.join("fonts");
        fs::create_dir_all(&fonts).unwrap();
        fs::write(fonts.join("fonts.css"), "@font-face { font-family: X; }\n").unwrap();
        fs::write(fonts.join("x.woff2"), b"woff").unwrap();
        let out = tmp.path().join("out");
        stage_app_fonts(&project, &out).unwrap();
        assert!(out.join("fonts/fonts.css").is_file());
        assert!(out.join("fonts/x.woff2").is_file());
        assert_eq!(app_fonts_css_href(&project, "assets"), None);
    }

    #[test]
    fn manifest_has_provenance() {
        let faces = framework_faces();
        assert!(faces.len() >= 4);
        for face in faces {
            assert!(face.package_id.starts_with("@fontsource"));
            assert!(!face.version.is_empty());
            assert_eq!(face.license, "OFL-1.1");
            assert!(!face.role.is_empty());
        }
    }

    #[test]
    fn document_floor_excludes_press_start() {
        let dir = tokens_src_dir();
        let tokens = fs::read_to_string(dir.join("tokens.css")).unwrap();
        let base = fs::read_to_string(dir.join("sail-base.css")).unwrap();
        let used = used_framework_font_files(&tokens, "", &base, &[], &[]);
        assert!(
            used.iter().any(|f| f.contains("dm-sans")),
            "body (DM Sans) must be used: {used:?}"
        );
        assert!(
            used.iter().any(|f| f.contains("syne")),
            "heading/display (Syne) must be used: {used:?}"
        );
        assert!(
            used.iter().any(|f| f.contains("jetbrains-mono")),
            "mono must be used: {used:?}"
        );
        assert!(
            !used.iter().any(|f| f.contains("press-start")),
            "Press Start 2P is unused without 8bit: {used:?}"
        );
    }

    #[test]
    fn eightbit_theme_includes_press_start() {
        let dir = tokens_src_dir();
        let tokens = fs::read_to_string(dir.join("tokens.css")).unwrap();
        let themes = fs::read_to_string(dir.join("themes.css")).unwrap();
        let base = fs::read_to_string(dir.join("sail-base.css")).unwrap();
        let used = used_framework_font_files(&tokens, &themes, &base, &["8bit".into()], &[]);
        assert!(
            used.iter().any(|f| f.contains("press-start")),
            "8bit remaps heading to press-start: {used:?}"
        );
    }

    #[test]
    fn unused_configured_role_does_not_ship() {
        let dir = tokens_src_dir();
        let tokens = fs::read_to_string(dir.join("tokens.css")).unwrap();
        let base = fs::read_to_string(dir.join("sail-base.css")).unwrap();
        // Extra configured role that nothing in the graph references.
        let used = used_framework_font_files(&tokens, "", &base, &[], &["unused".into()]);
        assert!(
            !used.iter().any(|f| f.contains("press-start")),
            "unused configured role must not pull press-start: {used:?}"
        );
    }

    #[test]
    fn strip_drops_unused_face_keeps_used() {
        let css = r#"
@font-face {
  font-family: "DM Sans Variable";
  src: url('./fonts/dm-sans-latin-wght-normal.woff2') format('woff2-variations');
}
@font-face {
  font-family: "Press Start 2P";
  src: url('./fonts/press-start-2p-latin-normal.woff2') format('woff2');
}
html { font-family: var(--font-body); }
"#;
        let mut used = HashSet::new();
        used.insert("dm-sans-latin-wght-normal.woff2".into());
        let out = strip_unused_framework_faces(css, &used);
        assert!(out.contains("dm-sans-latin-wght-normal.woff2"));
        assert!(!out.contains("press-start-2p-latin-normal.woff2"));
        assert!(out.contains("font-family: var(--font-body)"));
    }

    #[test]
    fn no_runtime_font_cdn_in_framework_css() {
        let dir = tokens_src_dir();
        for name in ["sail-base.css", "tokens.css", "themes.css"] {
            let css = fs::read_to_string(dir.join(name)).unwrap();
            assert!(
                !css.contains("fonts.googleapis.com") && !css.contains("fonts.gstatic.com"),
                "{name} must not reference a font CDN"
            );
        }
    }
}
