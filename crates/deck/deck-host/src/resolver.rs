use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use sui_compiler::{
    parse,
    render::{self, RenderOutput},
};

#[derive(Debug, serde::Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ResolvedView {
    pub source: String,
    pub params: HashMap<String, String>,
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ResolveError {
    pub message: String,
}
/// Hard ceiling on a single view artifact read+compiled per request (source
/// `.sui` and any co-located CSS). Each request reads from disk and compiles in
/// process with no size bound otherwise — a multi-MB file would let one request
/// pin a worker on a large parse. 4 MiB is comfortably above any real view yet
/// rejects pathological inputs. Exposed so deck-core's CSS reads share one
/// ceiling (one source of truth).
pub const MAX_VIEW_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
/// Config-driven view resolution layout (#273 — deck hardcodes no path).
///
/// `view_dirs` are the bucket names appended to the project root
/// (e.g. `["views"]`). For each `view_dirs[i]`, the
/// resolver looks for `<root>/<view_dirs[i]>/<name>/<view_leaf>`. All
/// buckets share one `view_leaf` (defaults to `page.sui`), which fixes
/// the #254 `index.sui` vs `page.sui` divergence between
/// buckets. A host that serves additional buckets (Ship's `.ship/views`)
/// supplies them as an OVERRIDE — deck names no host-private path.
/// `first_party_root`, when set, takes precedence over the
/// `view_dirs` buckets — the host declares its own first-party app
/// source root (e.g. `apps/shell/src`) and that bucket wins for any
/// matching view name.
#[derive(Debug, Clone)]
pub struct ViewLayout {
    pub view_dirs: Vec<String>,
    pub view_leaf: String,
    pub first_party_root: Option<PathBuf>,
}
impl ViewLayout {
    /// Conventional defaults — the deck-native `views/` bucket and the
    /// `page.sui` leaf. A host that serves additional buckets (Ship's
    /// `.ship/views`) supplies them as an override; deck names no
    /// host-private path in its zero-config default.
    pub fn conventional() -> Self {
        Self {
            view_dirs: vec!["views".into()],
            view_leaf: "page.sui".into(),
            first_party_root: None,
        }
    }
}

/// Read a view artifact to a string, capped at [`MAX_VIEW_SOURCE_BYTES`].
///
/// Returns `None` when the file is absent, unreadable, or larger than the cap —
/// the size is checked via a cheap `stat` before any bytes are read, so an
/// oversized file is never slurped into memory. Callers treat `None` as "no
/// such artifact" (the co-located CSS reads already chain on `Option`/`Result`).
pub fn read_view_artifact(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() > MAX_VIEW_SOURCE_BYTES {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

pub fn resolve(
    project_root: &Path,
    layout: &ViewLayout,
    name: &str,
    sub_path: Option<&str>,
) -> Result<ResolvedView, ResolveError> {
    let segments: Vec<&str> = sub_path
        .unwrap_or("")
        .split('/')
        .filter(|s| !s.is_empty() && *s != "..")
        .collect();
    let mut attempts: Vec<String> = Vec::new();
    // First-party views: the HOST supplies their root (deck is app-agnostic
    // and knows no app layout — see .artifacts/asset-architecture.md). When
    // set, it takes precedence over the configured `view_dirs` buckets.
    // When unset (a non-ship host), there is no first-party bucket.
    if let Some(fp_base) = layout.first_party_root.as_deref() {
        let fp_root = fp_base.join(name);
        let mut fp_params: HashMap<String, String> = HashMap::new();
        let mut fp_tried: Vec<PathBuf> = Vec::new();
        if let Some(found) = walk_disk_with_leaf(
            &fp_root,
            &segments,
            &layout.view_leaf,
            &mut fp_params,
            &mut fp_tried,
        ) {
            return read_resolved(found, fp_base, fp_params);
        }
    }
    // Walk every configured view-dir bucket in order. First hit wins; the
    // rest are reported as "tried" so a 404 names every bucket consulted.
    // A single leaf convention (`layout.view_leaf`) replaces the old
    // `page.sui`/`index.sui` per-bucket divergence (#254): every configured
    // bucket looks up the same `<name>/<view_leaf>`.
    for bucket in &layout.view_dirs {
        let bucket_root = project_root.join(bucket);
        let view_root = bucket_root.join(name);
        let mut params: HashMap<String, String> = HashMap::new();
        let mut tried: Vec<PathBuf> = Vec::new();
        if let Some(found) = walk_disk_with_leaf(
            &view_root,
            &segments,
            &layout.view_leaf,
            &mut params,
            &mut tried,
        ) {
            return read_resolved(found, &bucket_root, params);
        }
        for p in &tried {
            let rel = p
                .strip_prefix(&bucket_root)
                .map(|r| format!("{bucket}/{}", r.display()))
                .unwrap_or_else(|_| p.display().to_string());
            attempts.push(rel);
        }
    }
    Err(ResolveError {
        message: format!(
            "view not found: {} — looked at: {}",
            join_request(name, sub_path),
            attempts.join(", "),
        ),
    })
}

fn read_resolved(
    leaf: PathBuf,
    allowed_root: &Path,
    params: HashMap<String, String>,
) -> Result<ResolvedView, ResolveError> {
    // Containment guard against symlink escape: the leaf NAME is validated
    // upstream, but a symlink *inside* the views bucket can still point its
    // target outside the project. Canonicalize both the leaf and its bucket
    // root, then assert the resolved leaf stays under the root. A leaf that
    // resolves outside (or whose root can't be canonicalized) is rejected,
    // surfacing as a 404 upstream.
    let canonical = leaf.canonicalize().map_err(|e| ResolveError {
        message: format!("failed to resolve {}: {e}", leaf.display()),
    })?;
    let canonical_root = allowed_root.canonicalize().map_err(|e| ResolveError {
        message: format!("failed to resolve views root {}: {e}", allowed_root.display()),
    })?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ResolveError {
            message: format!(
                "view path escapes its root: {} is not under {}",
                canonical.display(),
                canonical_root.display(),
            ),
        });
    }
    // Size cap before read: each request reads + compiles the source in process
    // with no other bound. Reject an oversized file up front (cheap stat) rather
    // than slurping multiple MB into memory and parsing it.
    if let Ok(meta) = std::fs::metadata(&canonical)
        && meta.len() > MAX_VIEW_SOURCE_BYTES
    {
        return Err(ResolveError {
            message: format!(
                "view source too large: {} is {} bytes (max {})",
                canonical.display(),
                meta.len(),
                MAX_VIEW_SOURCE_BYTES,
            ),
        });
    }
    let source = match std::fs::read_to_string(&canonical) {
        Ok(s) => s,
        Err(e) => {
            return Err(ResolveError {
                message: format!("failed to read {}: {e}", canonical.display()),
            });
        }
    };
    Ok(ResolvedView {
        source,
        params,
        source_path: Some(canonical),
    })
}

fn walk_disk_with_leaf(
    dir: &Path,
    segments: &[&str],
    leaf_name: &str,
    params: &mut HashMap<String, String>,
    tried: &mut Vec<PathBuf>,
) -> Option<PathBuf> {
    if segments.is_empty() {
        let idx = dir.join(leaf_name);
        tried.push(idx.clone());
        return idx.is_file().then_some(idx);
    }

    if !dir.is_dir() {
        let mut idx = dir.to_path_buf();
        for seg in segments {
            idx.push(seg);
        }
        idx.push(leaf_name);
        tried.push(idx);
        return None;
    }

    let (head, tail) = segments.split_first().unwrap();

    let static_dir = dir.join(head);
    if static_dir.is_dir()
        && let Some(found) = walk_disk_with_leaf(&static_dir, tail, leaf_name, params, tried)
    {
        return Some(found);
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !name_str.starts_with('[') || !name_str.ends_with(']') {
                continue;
            }
            let param = name_str.trim_start_matches('[').trim_end_matches(']');
            if param.is_empty() {
                continue;
            }
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            params.insert(param.to_string(), (*head).to_string());
            if let Some(found) = walk_disk_with_leaf(&path, tail, leaf_name, params, tried) {
                return Some(found);
            }
            params.remove(param);
        }
    }

    None
}

fn join_request(name: &str, sub_path: Option<&str>) -> String {
    match sub_path.filter(|s| !s.is_empty()) {
        Some(sub) => format!("{name}/{sub}"),
        None => name.to_string(),
    }
}

struct CacheEntry {
    mtime: SystemTime,
    rendered: RenderOutput,
}

fn cache() -> &'static Mutex<HashMap<(PathBuf, String), CacheEntry>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<(PathBuf, String), CacheEntry>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn render_resolved(
    resolved: &ResolvedView,
    view_name: &str,
) -> Result<RenderOutput, String> {
    let path = resolved.source_path.as_ref().ok_or_else(|| {
        "render_resolved called without a source_path".to_string()
    })?;
    let mtime = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map_err(|e| format!("stat {}: {e}", path.display()))?;
    let key = (path.clone(), view_name.to_string());
    {
        let guard = cache().lock().expect("sui_resolver cache poisoned");
        if let Some(entry) = guard.get(&key)
            && entry.mtime == mtime
        {
            return Ok(entry.rendered.clone());
        }
    }
    let rendered = compile(&resolved.source, view_name)?;
    let mut guard = cache().lock().expect("sui_resolver cache poisoned");
    guard.insert(
        key,
        CacheEntry {
            mtime,
            rendered: rendered.clone(),
        },
    );
    Ok(rendered)
}

fn compile(source: &str, view_name: &str) -> Result<RenderOutput, String> {
    let program = parse::parse(source).map_err(|e| e.render(source))?;
    let opts = render::RenderOpts {
        view_name: view_name.to_string(),
    };
    render::render_default(&program, &opts).map_err(|e| e.render(source))
}

#[cfg(test)]
#[path = "resolver_tests.rs"]
mod tests;
