//! SailTheme/FontSearch + FontAdd — Google Fonts catalog proxy + vendor pipeline.
//!
//! Delivery: download CSS (unicode-range subsets from Google) → woff2 into
//! `<project>/fonts/` → `@font-face` in `fonts.css` → typography token write →
//! license row in `vendor-manifest.json`. Offline after selection; no runtime CDN.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{Value, json};

use crate::ctx::Ctx;
use crate::error::OpError;

/// Browser UA so Google returns woff2 (not ttf) in the CSS API.
const CSS_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Bundled Google Fonts catalog (family + category + popularity rank).
/// Generated from https://fonts.google.com/metadata/fonts — offline SSOT so
/// FontSearch is a real discoverer, not a 15-name guess list.
const BUNDLED_CATALOG_JSON: &str = include_str!("../data/google-fonts-catalog.json");

/// Tiny last-resort if the bundled JSON is ever missing/corrupt.
const FALLBACK_FAMILIES: &[&str] = &[
    "DM Sans",
    "Syne",
    "JetBrains Mono",
    "Press Start 2P",
    "Fraunces",
    "Inter",
    "Roboto",
    "Source Serif 4",
    "IBM Plex Sans",
    "Space Grotesk",
    "Playfair Display",
    "Libre Baskerville",
    "Fira Code",
    "Outfit",
    "Manrope",
    "Poppins",
    "Montserrat",
    "Open Sans",
    "Lato",
    "Noto Sans",
];

#[derive(Debug, Clone)]
struct FontEntry {
    family: String,
    category: String,
    /// Lower = more popular (Google popularity rank).
    popularity: u32,
    mono: bool,
}

static FAMILY_CATALOG: OnceLock<Vec<FontEntry>> = OnceLock::new();

fn http_client() -> Result<reqwest::blocking::Client, OpError> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(CSS_UA)
        .build()
        .map_err(|e| OpError::Internal(format!("http client: {e}")))
}

/// `reqwest::blocking` owns a private Tokio runtime. Building/dropping it on a Tokio
/// worker panics. When already inside a runtime, run the work on a dedicated OS thread.
fn run_blocking_http<F, T>(f: F) -> Result<T, OpError>
where
    F: FnOnce() -> Result<T, OpError> + Send + 'static,
    T: Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        std::thread::spawn(f)
            .join()
            .map_err(|_| OpError::Internal("blocking http thread panicked".into()))?
    } else {
        f()
    }
}

fn parse_bundled_catalog(raw: &str) -> Vec<FontEntry> {
    let Ok(v) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    let Some(arr) = v.as_array() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let Some(family) = item.get("family").and_then(|x| x.as_str()) else {
            continue;
        };
        let category = item
            .get("category")
            .and_then(|x| x.as_str())
            .unwrap_or("Other")
            .to_string();
        let popularity = item
            .get("popularity")
            .and_then(|x| x.as_u64())
            .unwrap_or(99_999) as u32;
        let mono = item.get("mono").and_then(|x| x.as_bool()).unwrap_or(false)
            || category.eq_ignore_ascii_case("Monospace");
        out.push(FontEntry {
            family: family.to_string(),
            category,
            popularity,
            mono,
        });
    }
    // Popularity ascending (1 = most popular), then name.
    out.sort_by(|a, b| {
        a.popularity
            .cmp(&b.popularity)
            .then_with(|| a.family.cmp(&b.family))
    });
    out
}

fn load_catalog() -> &'static [FontEntry] {
    FAMILY_CATALOG.get_or_init(|| {
        let mut list = parse_bundled_catalog(BUNDLED_CATALOG_JSON);
        if list.is_empty() {
            list = FALLBACK_FAMILIES
                .iter()
                .enumerate()
                .map(|(i, s)| FontEntry {
                    family: (*s).to_string(),
                    category: "Other".into(),
                    popularity: i as u32,
                    mono: s.to_ascii_lowercase().contains("mono")
                        || s.to_ascii_lowercase().contains("code"),
                })
                .collect();
        }
        list
    })
}

/// `SailTheme/FontSearch { query, category? }` — filter the full Google Fonts catalog.
/// Empty query returns the most popular families (real discoverer, not a stub list).
pub fn font_search(args: &Value, _ctx: &Ctx) -> Result<Value, OpError> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let category = args
        .get("category")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"))
        .map(|s| s.to_ascii_lowercase());
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n.clamp(1, 80) as usize)
        .unwrap_or(40);

    let catalog = load_catalog();
    let mut hits: Vec<&FontEntry> = catalog
        .iter()
        .filter(|e| {
            if let Some(ref cat) = category {
                let ec = e.category.to_ascii_lowercase();
                if cat == "mono" || cat == "monospace" {
                    if !e.mono && ec != "monospace" {
                        return false;
                    }
                } else if !ec.contains(cat) {
                    return false;
                }
            }
            if query.is_empty() {
                return true;
            }
            e.family.to_ascii_lowercase().contains(&query)
        })
        .take(limit)
        .collect();

    // Keep popularity order for empty query; for non-empty prefer prefix matches first.
    if !query.is_empty() {
        hits.sort_by(|a, b| {
            let a_pref = a.family.to_ascii_lowercase().starts_with(&query);
            let b_pref = b.family.to_ascii_lowercase().starts_with(&query);
            b_pref
                .cmp(&a_pref)
                .then_with(|| a.popularity.cmp(&b.popularity))
                .then_with(|| a.family.cmp(&b.family))
        });
    }

    let families: Vec<String> = hits.iter().map(|e| e.family.clone()).collect();
    let hit_rows: Vec<Value> = hits
        .iter()
        .map(|e| {
            json!({
                "family": e.family,
                "category": e.category,
                "popularity": e.popularity,
                "mono": e.mono,
            })
        })
        .collect();

    Ok(json!({
        "ok": true,
        "query": query,
        "category": category,
        "families": families,
        "hits": hit_rows,
        "count": families.len(),
        "catalogSize": catalog.len(),
        "source": "bundled-google-fonts",
    }))
}

/// `SailTheme/FontAdd { family, axes?, project? }` — vendor subsetted woff2 + tokens + license.
pub fn font_add(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let family = args
        .get("family")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("FontAdd requires family".into()))?
        .to_string();

    if family.contains("..") || family.contains('/') || family.contains('\\') {
        return Err(OpError::InvalidArgument("invalid family name".into()));
    }

    let axes = args
        .get("axes")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let project = resolve_project_arg(args, ctx)?;
    let fonts_dir = project.join("fonts");
    std::fs::create_dir_all(&fonts_dir)
        .map_err(|e| OpError::Internal(format!("mkdir fonts {}: {e}", fonts_dir.display())))?;

    let css_api = google_css_url(&family, axes);
    // Download CSS + woff2 off any ambient Tokio worker (reqwest::blocking private RT).
    let (written_files, face_rules) = run_blocking_http({
        let family = family.clone();
        let fonts_dir = fonts_dir.clone();
        move || download_font_faces(&family, &css_api, &fonts_dir)
    })?;

    // Merge into fonts.css (append rules for this family; rewrite if family already present).
    let css_path = fonts_dir.join("fonts.css");
    let mut css_body = if css_path.is_file() {
        std::fs::read_to_string(&css_path).unwrap_or_default()
    } else {
        String::from("/* App-vendored fonts (SailTheme/FontAdd) — offline after selection. */\n")
    };
    // Drop previous rules for this family (simple marker blocks).
    css_body = strip_family_block(&css_body, &family);
    css_body.push_str(&format!("\n/* family: {family} */\n"));
    for rule in &face_rules {
        css_body.push_str(rule);
        css_body.push('\n');
    }
    std::fs::write(&css_path, css_body)
        .map_err(|e| OpError::Internal(format!("write fonts.css: {e}")))?;

    // License / vendor manifest.
    let manifest_path = fonts_dir.join("vendor-manifest.json");
    let mut manifest: BTreeMap<String, Value> = if manifest_path.is_file() {
        std::fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        BTreeMap::new()
    };
    manifest.insert(
        family.clone(),
        json!({
            "family": family,
            "source": "google-fonts",
            "license": "OFL-1.1",
            "licenseUrl": "https://scripts.sil.org/OFL",
            "files": written_files,
            "css": "fonts.css",
            "axes": axes,
        }),
    );
    let man_body = serde_json::to_string_pretty(&manifest)
        .map_err(|e| OpError::Internal(format!("serialize vendor-manifest: {e}")))?;
    std::fs::write(&manifest_path, format!("{man_body}\n"))
        .map_err(|e| OpError::Internal(format!("write vendor-manifest: {e}")))?;

    // Typography token write: patch active project theme body/display if present.
    let token_path = write_type_role_tokens(ctx, &project, &family)?;

    Ok(json!({
        "ok": true,
        "family": family,
        "project": project_rel(ctx, &project),
        "fontsDir": project_rel(ctx, &fonts_dir),
        "files": written_files,
        "css": project_rel(ctx, &css_path),
        "manifest": project_rel(ctx, &manifest_path),
        "themePath": token_path,
        "offline": true,
    }))
}

/// Fetch Google CSS, download latin woff2 into `fonts_dir`, return written names + CSS rules.
fn download_font_faces(
    family: &str,
    css_api: &str,
    fonts_dir: &Path,
) -> Result<(Vec<String>, Vec<String>), OpError> {
    let client = http_client()?;
    let css_text = client
        .get(css_api)
        .header("User-Agent", CSS_UA)
        .send()
        .map_err(|e| OpError::Internal(format!("FontAdd CSS fetch: {e}")))?
        .error_for_status()
        .map_err(|e| OpError::Internal(format!("FontAdd CSS status: {e}")))?
        .text()
        .map_err(|e| OpError::Internal(format!("FontAdd CSS body: {e}")))?;

    let faces = parse_google_css(&css_text);
    if faces.is_empty() {
        return Err(OpError::FailedPrecondition(format!(
            "FontAdd: no @font-face/woff2 found for family {family:?} (check spelling)"
        )));
    }

    let slug = slugify(family);
    let mut written_files: Vec<String> = Vec::new();
    let mut face_rules: Vec<String> = Vec::new();

    for (i, face) in faces.iter().enumerate() {
        // Prefer latin; skip non-latin ranges when we already have latin.
        let is_latin = face
            .unicode_range
            .as_deref()
            .map(|u| u.contains("U+0000-00FF") || u.contains("U+0000"))
            .unwrap_or(true);
        if !is_latin
            && faces.iter().any(|f| {
                f.unicode_range
                    .as_deref()
                    .is_some_and(|u| u.contains("U+0000-00FF"))
            })
        {
            // Keep one non-latin optional? Spec: subset unicode-range — keep latin only.
            continue;
        }

        let bytes = client
            .get(&face.url)
            .send()
            .map_err(|e| OpError::Internal(format!("FontAdd woff2 fetch: {e}")))?
            .error_for_status()
            .map_err(|e| OpError::Internal(format!("FontAdd woff2 status: {e}")))?
            .bytes()
            .map_err(|e| OpError::Internal(format!("FontAdd woff2 body: {e}")))?;

        let weight_part = face
            .weight
            .as_deref()
            .unwrap_or("400")
            .replace(' ', "")
            .replace("..", "-");
        let file_name = if faces.len() == 1 {
            format!("{slug}-latin-wght-normal.woff2")
        } else {
            format!("{slug}-latin-{weight_part}-{i}.woff2")
        };
        let path = fonts_dir.join(&file_name);
        std::fs::write(&path, &bytes)
            .map_err(|e| OpError::Internal(format!("write {}: {e}", path.display())))?;
        written_files.push(file_name.clone());

        let format = if face.weight.as_deref().is_some_and(|w| w.contains(' ')) {
            "woff2-variations"
        } else {
            "woff2"
        };
        let weight = face.weight.as_deref().unwrap_or("400");
        let style = face.style.as_deref().unwrap_or("normal");
        let unicode = face.unicode_range.clone().unwrap_or_else(|| {
            "U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD".into()
        });
        face_rules.push(format!(
            "@font-face {{\n  font-family: '{family}';\n  font-style: {style};\n  font-weight: {weight};\n  font-display: swap;\n  src: url('./{file_name}') format('{format}');\n  unicode-range: {unicode};\n}}\n"
        ));
    }

    if written_files.is_empty() {
        return Err(OpError::FailedPrecondition(format!(
            "FontAdd: no latin woff2 downloaded for {family:?}"
        )));
    }

    Ok((written_files, face_rules))
}

fn resolve_project_arg(args: &Value, ctx: &Ctx) -> Result<PathBuf, OpError> {
    if let Some(p) = args.get("project").and_then(|v| v.as_str()).map(str::trim) {
        if p.is_empty() || p.contains("..") {
            return Err(OpError::InvalidArgument("invalid project path".into()));
        }
        let pb = PathBuf::from(p);
        let abs = if pb.is_absolute() {
            pb
        } else {
            ctx.workspace_root.join(pb)
        };
        if abs.join("routes").is_dir() || abs.join("package.json").is_file() {
            return Ok(abs);
        }
        return Err(OpError::InvalidArgument(format!(
            "project {} is not an app (need routes/ or package.json)",
            abs.display()
        )));
    }
    resolve_project(ctx)
}

fn resolve_project(ctx: &Ctx) -> Result<PathBuf, OpError> {
    let p = &ctx.project_root;
    let same_as_workspace =
        p.canonicalize().ok().as_ref() == ctx.workspace_root.canonicalize().ok().as_ref();

    // Strong signal: routes/ means this is an app (external or monorepo).
    if p.join("routes").is_dir() {
        return Ok(p.clone());
    }
    // package.json alone only counts when not the monorepo workspace root
    // (workspace roots often carry a package.json without being an app).
    if p.join("package.json").is_file() && !same_as_workspace {
        return Ok(p.clone());
    }

    // Monorepo fallback: first apps/* with routes (sorted — no named-app hardcode).
    let apps = ctx.workspace_root.join("apps");
    if apps.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&apps) {
            let mut paths: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
            paths.sort();
            for path in paths {
                if path.join("routes").is_dir() {
                    return Ok(path);
                }
            }
        }
    }
    Err(OpError::FailedPrecondition(
        "FontAdd needs a project (pass project: \"<app-root>\" or run with project ctx)".into(),
    ))
}

fn project_rel(ctx: &Ctx, path: &Path) -> String {
    path.strip_prefix(&ctx.workspace_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn google_css_url(family: &str, axes: Option<&str>) -> String {
    let encoded = family.replace(' ', "+");
    // Variable-friendly request: opsz/wght ranges when unspecified.
    let axis = axes.unwrap_or("wght@100..900");
    format!("https://fonts.googleapis.com/css2?family={encoded}:{axis}&display=swap")
}

#[derive(Debug, Clone)]
struct FaceRef {
    url: String,
    weight: Option<String>,
    style: Option<String>,
    unicode_range: Option<String>,
}

fn parse_google_css(css: &str) -> Vec<FaceRef> {
    let mut faces = Vec::new();
    // Split on @font-face
    for block in css.split("@font-face").skip(1) {
        let end = block.find('}').unwrap_or(block.len());
        let body = &block[..end];
        let url = extract_url(body);
        let Some(url) = url else { continue };
        if !url.contains(".woff2") {
            continue;
        }
        faces.push(FaceRef {
            url,
            weight: extract_prop(body, "font-weight"),
            style: extract_prop(body, "font-style"),
            unicode_range: extract_prop(body, "unicode-range"),
        });
    }
    faces
}

fn extract_url(body: &str) -> Option<String> {
    let key = "url(";
    let i = body.find(key)?;
    let rest = &body[i + key.len()..];
    let rest = rest.trim_start_matches(['\'', '"']);
    let end = rest.find(['\'', '"', ')']).unwrap_or(rest.len());
    let u = rest[..end].trim();
    if u.starts_with("http") {
        Some(u.to_string())
    } else {
        None
    }
}

fn extract_prop(body: &str, prop: &str) -> Option<String> {
    for line in body.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(prop) {
            let rest = rest.trim_start_matches(':').trim();
            let val = rest.trim_end_matches(';').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn slugify(family: &str) -> String {
    family
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn strip_family_block(css: &str, family: &str) -> String {
    let marker = format!("/* family: {family} */");
    let mut out = String::new();
    let mut lines = css.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() == marker {
            // Skip until next family marker or EOF — but keep following markers.
            while let Some(n) = lines.peek() {
                if n.trim().starts_with("/* family:") {
                    break;
                }
                lines.next();
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Patch project theme typography fontFamily body role when a theme exists.
fn write_type_role_tokens(
    ctx: &Ctx,
    project: &Path,
    family: &str,
) -> Result<Option<String>, OpError> {
    // Modern `themes/` then legacy `tokens/themes/` (EXT-1 E4 dual layout).
    let theme_dirs = sail_compiler::tokens_generate::project_theme_dirs(project);
    let mut theme_file: Option<PathBuf> = None;
    for themes_dir in &theme_dirs {
        if let Ok(rd) = std::fs::read_dir(themes_dir) {
            let mut jsons: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("json"))
                .collect();
            jsons.sort();
            if let Some(p) = jsons.into_iter().next() {
                theme_file = Some(p);
                break;
            }
        }
    }
    let Some(path) = theme_file else {
        return Ok(None);
    };
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| OpError::Internal(format!("read theme {}: {e}", path.display())))?;
    let mut doc: Value = serde_json::from_str(&raw)
        .map_err(|e| OpError::InvalidArgument(format!("theme JSON: {e}")))?;

    // Best-effort: set modes.*.font.family.body or fontFamily.body stacks.
    if let Some(modes) = doc.get_mut("modes").and_then(|m| m.as_object_mut()) {
        for (_mode, mode_val) in modes.iter_mut() {
            patch_font_family(mode_val, family);
        }
    } else {
        patch_font_family(&mut doc, family);
    }

    let out = serde_json::to_string_pretty(&doc)
        .map_err(|e| OpError::Internal(format!("serialize theme: {e}")))?;
    std::fs::write(&path, format!("{out}\n"))
        .map_err(|e| OpError::Internal(format!("write theme: {e}")))?;
    Ok(Some(project_rel(ctx, &path)))
}

fn patch_font_family(node: &mut Value, family: &str) {
    // Common DTCG shapes used by studio theme-doc.
    if let Some(obj) = node.as_object_mut() {
        // font.family.body.$value
        if let Some(font) = obj.get_mut("font").and_then(|f| f.as_object_mut()) {
            if let Some(fam) = font.get_mut("family").and_then(|f| f.as_object_mut()) {
                for role in ["body", "display", "sans"] {
                    if let Some(slot) = fam.get_mut(role) {
                        set_value_stack(slot, family);
                    }
                }
            }
        }
        if let Some(typo) = obj.get_mut("typography").and_then(|t| t.as_object_mut()) {
            if let Some(ff) = typo.get_mut("fontFamily").and_then(|f| f.as_object_mut()) {
                for role in ["body", "display", "sans"] {
                    if let Some(slot) = ff.get_mut(role) {
                        set_value_stack(slot, family);
                    }
                }
            }
        }
    }
}

fn set_value_stack(slot: &mut Value, family: &str) {
    if let Some(obj) = slot.as_object_mut() {
        if obj.contains_key("$value") {
            obj.insert("$value".into(), json!([family, "system-ui", "sans-serif"]));
            return;
        }
    }
    *slot = json!({
        "$type": "fontFamilies",
        "$value": [family, "system-ui", "sans-serif"]
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_fraunces() {
        assert_eq!(slugify("Fraunces"), "fraunces");
        assert_eq!(slugify("DM Sans"), "dm-sans");
    }

    #[test]
    fn parse_css_extracts_woff2() {
        let css = r#"
@font-face {
  font-family: 'Fraunces';
  font-style: normal;
  font-weight: 100 900;
  font-display: swap;
  src: url(https://fonts.gstatic.com/s/fraunces/v31/latin.woff2) format('woff2');
  unicode-range: U+0000-00FF;
}
"#;
        let faces = parse_google_css(css);
        assert_eq!(faces.len(), 1);
        assert!(faces[0].url.contains("fraunces"));
        assert_eq!(faces[0].weight.as_deref(), Some("100 900"));
    }

    #[test]
    fn strip_family_block_removes_prior() {
        let css =
            "/* App */\n/* family: A */\n@font-face { a }\n/* family: B */\n@font-face { b }\n";
        let out = strip_family_block(css, "A");
        assert!(!out.contains("family: A"));
        assert!(out.contains("family: B"));
    }
}
