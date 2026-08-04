//! `SailTheme` RPCs — List, Read, Write, Fork, Generate, Graph.
//!
//! Writes confined to `apps/*/tokens/themes/**/*.json`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::ctx::Ctx;
use crate::error::OpError;

const CATALOG_THEMES: &str = "packages/sail/tokens/model/themes";
const MODEL_ROOT: &str = "packages/sail/tokens/model";
const MODEL_COMPONENTS: &str = "packages/sail/tokens/model/components";

// ── path policy ─────────────────────────────────────────────────────────────

/// Writable project themes: `apps/<app>/tokens/themes/**/*.json` only.
pub fn is_write_theme_path(rel: &str) -> bool {
    let norm = rel.replace('\\', "/");
    if norm.is_empty() || norm.contains("..") || norm.contains("node_modules") {
        return false;
    }
    if norm.contains("packages/sail/tokens/model") {
        return false;
    }
    if !norm.starts_with("apps/") {
        return false;
    }
    if !norm.contains("/tokens/themes/") {
        return false;
    }
    norm.ends_with(".json")
}

/// Readable: project themes, catalog themes, or package model schema JSON
/// (`base.json` + `components/*.json`). Never density/compat/theme CSS.
pub fn is_read_theme_path(rel: &str) -> bool {
    let norm = rel.replace('\\', "/");
    if is_model_schema_file_path(&norm) {
        return true;
    }
    if norm.ends_with(".json") && is_catalog_theme_file_path(&norm) {
        return true;
    }
    is_write_theme_path(rel)
}

fn is_catalog_theme_file_path(norm: &str) -> bool {
    if norm.is_empty() || norm.contains("..") || norm.contains("node_modules") {
        return false;
    }
    norm.starts_with("packages/sail/tokens/model/themes/") && norm.ends_with(".json")
}

fn is_catalog_theme_dir_path(norm: &str) -> bool {
    let n = norm.trim_end_matches('/');
    n == "packages/sail/tokens/model/themes"
}

/// Package DTCG schema sources the studio merges into a full token surface.
fn is_model_schema_file_path(norm: &str) -> bool {
    if norm.is_empty() || norm.contains("..") || norm.contains("node_modules") {
        return false;
    }
    if norm == format!("{MODEL_ROOT}/base.json") {
        return true;
    }
    norm.starts_with(&format!("{MODEL_COMPONENTS}/"))
        && norm.ends_with(".json")
        && !norm[MODEL_COMPONENTS.len() + 1..].contains('/')
}

fn is_model_components_dir_path(norm: &str) -> bool {
    let n = norm.trim_end_matches('/');
    n == MODEL_COMPONENTS
}

/// Directory listable: project themes, catalog themes, or model/components.
pub fn is_list_theme_dir(rel: &str) -> bool {
    let norm = rel.replace('\\', "/").trim_end_matches('/').to_string();
    if norm.is_empty() || norm.contains("..") || norm.contains("node_modules") {
        return false;
    }
    if is_catalog_theme_dir_path(&norm) || is_model_components_dir_path(&norm) {
        return true;
    }
    if !norm.starts_with("apps/") {
        return false;
    }
    norm.ends_with("/tokens/themes") || norm.contains("/tokens/themes/")
}

fn ensure_under_workspace(root: &Path, target: &Path) -> Result<PathBuf, OpError> {
    let canon_root = root
        .canonicalize()
        .map_err(|e| OpError::Internal(format!("workspace root {}: {e}", root.display())))?;
    if target.exists() {
        let canon = target
            .canonicalize()
            .map_err(|e| OpError::NotFound(format!("canonicalize {}: {e}", target.display())))?;
        if !canon.starts_with(&canon_root) {
            return Err(OpError::PermissionDenied("path escapes workspace".into()));
        }
        return Ok(canon);
    }
    // Parent must exist under workspace for create-on-write.
    let parent = target.parent().unwrap_or(root);
    if !parent.exists() {
        std::fs::create_dir_all(parent)
            .map_err(|e| OpError::Internal(format!("mkdir {}: {e}", parent.display())))?;
    }
    let canon_parent = parent
        .canonicalize()
        .map_err(|e| OpError::InvalidArgument(format!("parent path invalid: {e}")))?;
    if !canon_parent.starts_with(&canon_root) {
        return Err(OpError::PermissionDenied("path escapes workspace".into()));
    }
    Ok(canon_parent.join(target.file_name().unwrap_or_default()))
}

// ── RPCs ────────────────────────────────────────────────────────────────────

/// List theme JSON basenames under a directory path (workspace-relative).
pub fn list(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let rel = args
        .get("path")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("List requires non-empty `path`".into()))?;

    if !is_list_theme_dir(rel) {
        return Err(OpError::PermissionDenied(format!(
            "path not listable (apps/*/tokens/themes, {CATALOG_THEMES}, or {MODEL_COMPONENTS}): {rel}"
        )));
    }

    let dir = ctx.workspace_root.join(rel);
    let canon_root = ctx
        .workspace_root
        .canonicalize()
        .map_err(|e| OpError::Internal(format!("workspace root missing: {e}")))?;

    if !dir.is_dir() {
        return Ok(json!({ "ok": true, "path": rel, "files": [] }));
    }
    let canon_dir = dir
        .canonicalize()
        .map_err(|e| OpError::InvalidArgument(format!("path invalid: {e}")))?;
    if !canon_dir.starts_with(&canon_root) {
        return Err(OpError::PermissionDenied("path escapes workspace".into()));
    }

    let mut files: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&canon_dir) {
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().into_owned();
            if name.ends_with(".json") && !name.starts_with('.') {
                files.push(name);
            }
        }
    }
    files.sort();
    Ok(json!({ "ok": true, "path": rel, "files": files }))
}

/// Read a theme JSON file (project or catalog).
pub fn read(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let rel = args
        .get("path")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("Read requires non-empty `path`".into()))?;

    if !is_read_theme_path(rel) {
        return Err(OpError::PermissionDenied(format!(
            "path not readable (project themes, catalog themes, or package model schema JSON): {rel}"
        )));
    }

    let target = ctx.workspace_root.join(rel);
    let canon = ensure_under_workspace(&ctx.workspace_root, &target)?;
    let content = std::fs::read_to_string(&canon)
        .map_err(|e| OpError::NotFound(format!("read {rel}: {e}")))?;
    Ok(json!({ "ok": true, "path": rel, "content": content }))
}

/// Write project theme JSON only.
///
/// Enforces `$extensions.ordered` scale monotonicity before disk write
/// (named `NonMonotonicScale` — same refusal for MCP + GUI).
pub fn write(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let rel = args
        .get("path")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("Write requires non-empty `path`".into()))?;
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OpError::InvalidArgument("Write requires string `content`".into()))?;

    if !is_write_theme_path(rel) {
        return Err(OpError::PermissionDenied(format!(
            "path not writable (apps/*/tokens/themes/*.json only): {rel}"
        )));
    }

    // Parse + ordered-scale gate (refuse non-monotonic before write).
    if let Ok(doc) = serde_json::from_str::<Value>(content) {
        validate_ordered_scales(&doc)?;
    }

    let target = ctx.workspace_root.join(rel);
    let resolved = ensure_under_workspace(&ctx.workspace_root, &target)?;
    if let Some(parent) = resolved.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| OpError::Internal(format!("mkdir {}: {e}", parent.display())))?;
    }
    std::fs::write(&resolved, content)
        .map_err(|e| OpError::Internal(format!("write {rel}: {e}")))?;
    Ok(json!({
        "ok": true,
        "path": rel,
        "bytes": content.len(),
    }))
}

/// Default ordered steps for the radius family when `$extensions.ordered` is absent
/// but a `radius` object with step keys is present (design surface + catalog parity).
const DEFAULT_RADIUS_ORDERED: &[&str] = &["sm", "md", "lg", "xl"];

/// Walk DTCG JSON and refuse non-monotonic ordered scale families.
///
/// `$extensions.ordered` is an array of sibling step keys (e.g. `["sm","md","lg","xl"]`).
/// Each step's `$value` must parse as a dimension and be non-decreasing vs the previous.
/// Sentinel steps like `full` (9999px) are excluded by not listing them in `ordered`.
pub fn validate_ordered_scales(doc: &Value) -> Result<(), OpError> {
    walk_ordered_scales(doc, "")
}

fn walk_ordered_scales(node: &Value, path: &str) -> Result<(), OpError> {
    let Some(obj) = node.as_object() else {
        return Ok(());
    };

    // Explicit `$extensions.ordered` on this object.
    if let Some(ordered) = ordered_keys(obj) {
        check_monotonic_steps(obj, &ordered, path)?;
    } else if path == "radius" || path.ends_with(".radius") {
        // Implicit radius family (base model / theme radius overrides).
        let keys: Vec<String> = DEFAULT_RADIUS_ORDERED
            .iter()
            .filter(|k| obj.contains_key(**k))
            .map(|s| (*s).to_string())
            .collect();
        if keys.len() >= 2 {
            check_monotonic_steps(obj, &keys, path)?;
        }
    }

    for (k, child) in obj {
        if k.starts_with('$') {
            continue;
        }
        let next = if path.is_empty() {
            k.clone()
        } else {
            format!("{path}.{k}")
        };
        walk_ordered_scales(child, &next)?;
    }
    // Also walk modes.* theme branches (modes.light.radius …).
    Ok(())
}

fn ordered_keys(obj: &serde_json::Map<String, Value>) -> Option<Vec<String>> {
    let ext = obj.get("$extensions")?.as_object()?;
    let arr = ext
        .get("ordered")
        .or_else(|| {
            ext.get("ship")
                .and_then(|s| s.as_object())
                .and_then(|s| s.get("ordered"))
        })?
        .as_array()?;
    let keys: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if keys.len() >= 2 { Some(keys) } else { None }
}

fn check_monotonic_steps(
    obj: &serde_json::Map<String, Value>,
    keys: &[String],
    path: &str,
) -> Result<(), OpError> {
    let mut prev: Option<(String, f64, String)> = None;
    for key in keys {
        let Some(step) = obj.get(key) else {
            continue;
        };
        let Some(raw) = step_value_str(step) else {
            continue;
        };
        let Some(px) = parse_dimension_px(&raw) else {
            // Non-dimension / alias — skip numeric check for that step.
            continue;
        };
        if let Some((prev_key, prev_px, prev_raw)) = &prev {
            if px + f64::EPSILON < *prev_px {
                return Err(OpError::InvalidArgument(format!(
                    "NonMonotonicScale: {path} step {key}={raw} is less than {prev_key}={prev_raw} (ordered requires non-decreasing)"
                )));
            }
        }
        prev = Some((key.clone(), px, raw));
    }
    Ok(())
}

fn step_value_str(step: &Value) -> Option<String> {
    let obj = step.as_object()?;
    let v = obj.get("$value")?;
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Parse dimension-like `$value` to px for ordering. Supports `Npx`, bare number, `Nrem` (16×).
fn parse_dimension_px(raw: &str) -> Option<f64> {
    let s = raw.trim();
    if s.starts_with('{') {
        return None; // alias
    }
    if let Some(n) = s.strip_suffix("px") {
        return n.trim().parse().ok();
    }
    if let Some(n) = s.strip_suffix("rem") {
        return n.trim().parse::<f64>().ok().map(|v| v * 16.0);
    }
    if let Some(n) = s.strip_suffix("em") {
        return n.trim().parse::<f64>().ok().map(|v| v * 16.0);
    }
    s.parse().ok()
}

/// Delete a **project** theme JSON under the write allowlist (catalog paths refused).
pub fn delete(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let rel = args
        .get("path")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("Delete requires non-empty `path`".into()))?;

    if !is_write_theme_path(rel) {
        return Err(OpError::PermissionDenied(format!(
            "path not deletable (apps/*/tokens/themes/*.json only): {rel}"
        )));
    }

    let target = ctx.workspace_root.join(rel);
    let resolved = ensure_under_workspace(&ctx.workspace_root, &target)?;
    if !resolved.is_file() {
        return Err(OpError::NotFound(format!("theme not found: {rel}")));
    }
    std::fs::remove_file(&resolved).map_err(|e| OpError::Internal(format!("delete {rel}: {e}")))?;
    Ok(json!({
        "ok": true,
        "path": rel,
    }))
}

/// Fork a catalog (or project) theme into a new project theme file.
///
/// Args: `source_path`, `dest_path` (must be write-allowlisted), optional `content`
/// override (if omitted, reads source).
pub fn fork(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let source_path = args
        .get("source_path")
        .or_else(|| args.get("source"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("Fork requires non-empty `source_path`".into()))?;
    let dest_path = args
        .get("dest_path")
        .or_else(|| args.get("dest"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("Fork requires non-empty `dest_path`".into()))?;

    if !is_read_theme_path(source_path) {
        return Err(OpError::PermissionDenied(format!(
            "source not readable: {source_path}"
        )));
    }
    if !is_write_theme_path(dest_path) {
        return Err(OpError::PermissionDenied(format!(
            "dest not writable (apps/*/tokens/themes/*.json only): {dest_path}"
        )));
    }

    let content = if let Some(c) = args.get("content").and_then(|v| v.as_str()) {
        c.to_string()
    } else {
        let src = ctx.workspace_root.join(source_path);
        let canon = ensure_under_workspace(&ctx.workspace_root, &src)?;
        std::fs::read_to_string(&canon)
            .map_err(|e| OpError::NotFound(format!("read source {source_path}: {e}")))?
    };

    // Optional themeId rewrite via `new_id` — validate shape server-side (T2 §4.4).
    let content = if let Some(new_id) = args.get("new_id").and_then(|v| v.as_str()) {
        sail_compiler::tokens_generate::validate_theme_id(new_id)
            .map_err(|e| OpError::InvalidArgument(format!("Fork new_id invalid: {e}")))?;
        rewrite_theme_id(&content, new_id).map_err(|e| {
            OpError::InvalidArgument(format!("Fork cannot rewrite themeId to {new_id:?}: {e}"))
        })?
    } else {
        content
    };

    write(&json!({ "path": dest_path, "content": content }), ctx)?;
    Ok(json!({
        "ok": true,
        "source_path": source_path,
        "dest_path": dest_path,
        "bytes": content.len(),
    }))
}

fn rewrite_theme_id(content: &str, new_id: &str) -> Result<String, String> {
    let mut v: Value =
        serde_json::from_str(content).map_err(|e| format!("source is not valid JSON: {e}"))?;
    let obj = v
        .as_object_mut()
        .ok_or_else(|| "source JSON root must be an object".to_string())?;
    let ext = obj.entry("$extensions").or_insert_with(|| json!({}));
    let ext_obj = ext
        .as_object_mut()
        .ok_or_else(|| "$extensions must be an object".to_string())?;
    let ship = ext_obj.entry("ship").or_insert_with(|| json!({}));
    let ship_obj = ship
        .as_object_mut()
        .ok_or_else(|| "$extensions.ship must be an object".to_string())?;
    ship_obj.insert("themeId".into(), json!(new_id));
    serde_json::to_string_pretty(&v)
        .map(|s| s + "\n")
        .map_err(|e| format!("serialize rewritten theme: {e}"))
}

/// Generate tokens.
///
/// Args:
/// - `mode`: `"package"` | `"app"` (default: infer from `out`, else package)
/// - `project`: project root for theme merge / app emit
/// - `out`: optional out dir override (must stay under workspace)
/// - `workspace`, `model`: path overrides (must stay under workspace)
///
/// **package** — full catalog + project merge → `packages/sail/tokens/src`
/// **app** — T0 effective set only → `<project>/.deck/themes` (B1 themes.css only)
pub fn generate(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let workspace = resolve_under_workspace(
        args.get("workspace").and_then(|v| v.as_str()),
        &ctx.workspace_root,
        /* must_exist */ true,
    )?;
    let model_dir = if let Some(m) = args.get("model").and_then(|v| v.as_str()) {
        resolve_path_under(&workspace, m)?
    } else {
        workspace.join("packages/sail/tokens/model")
    };

    // Project merge when `project` is set in args, else if ctx.project_root
    // differs from workspace and looks like an app (dev server path).
    let project_opt = if let Some(p) = args.get("project").and_then(|v| v.as_str()) {
        Some(resolve_path_under(&workspace, p)?)
    } else if ctx.project_root != ctx.workspace_root
        && (ctx.project_root.join("routes").is_dir()
            || ctx.project_root.join("package.json").is_file())
    {
        Some(confine_existing(&workspace, &ctx.project_root)?)
    } else {
        None
    };

    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            // Infer: explicit out under .deck/themes → app; else package.
            if let Some(out) = args.get("out").and_then(|v| v.as_str()) {
                let n = out.replace('\\', "/");
                if n.contains("/.deck/themes") || n.ends_with("/.deck/themes") {
                    return "app";
                }
            }
            "package"
        });

    let report = match mode {
        "app" => {
            let project_dir = project_opt.ok_or_else(|| {
                OpError::InvalidArgument(
                    "SailTheme/Generate mode=app requires `project` (or ctx.project_root app)"
                        .into(),
                )
            })?;
            let out_dir = if let Some(o) = args.get("out").and_then(|v| v.as_str()) {
                resolve_path_under(&workspace, o)?
            } else {
                sail_compiler::tokens_generate::app_themes_out_dir(&project_dir)
            };
            // Prefer dedicated helper (themes_only + correct out dir).
            if args.get("out").is_none() {
                sail_compiler::tokens_generate::generate_app_delivery(&workspace, &project_dir)
                    .map_err(|e| OpError::Internal(format!("tokens generate (app): {e}")))?
            } else {
                let mut opts =
                    sail_compiler::tokens_generate::options_for_project(&workspace, &project_dir)
                        .map_err(|e| {
                        OpError::FailedPrecondition(format!(
                            "theme config for {}: {e}",
                            project_dir.display()
                        ))
                    })?;
                opts.themes_only = true;
                sail_compiler::tokens_generate::generate_with_options(&model_dir, &out_dir, &opts)
                    .map_err(|e| OpError::Internal(format!("tokens generate (app): {e}")))?
            }
        }
        "package" => {
            let out_dir = if let Some(o) = args.get("out").and_then(|v| v.as_str()) {
                resolve_path_under(&workspace, o)?
            } else {
                workspace.join("packages/sail/tokens/src")
            };
            if let Some(project_dir) = project_opt {
                let opts = sail_compiler::tokens_generate::options_for_shared_package(
                    &workspace,
                    &project_dir,
                )
                .map_err(|e| {
                    OpError::FailedPrecondition(format!(
                        "theme config for {}: {e}",
                        project_dir.display()
                    ))
                })?;
                sail_compiler::tokens_generate::generate_with_options(&model_dir, &out_dir, &opts)
                    .map_err(|e| OpError::Internal(format!("tokens generate (package): {e}")))?
            } else {
                sail_compiler::tokens_generate::generate(&model_dir, &out_dir)
                    .map_err(|e| OpError::Internal(format!("tokens generate: {e}")))?
            }
        }
        other => {
            return Err(OpError::InvalidArgument(format!(
                "SailTheme/Generate mode must be \"package\" or \"app\" (got {other:?})"
            )));
        }
    };

    Ok(json!({
        "ok": true,
        "mode": mode,
        "token_count": report.token_count,
        "theme_count": report.theme_count,
        "tokens_css": report.tokens_css.display().to_string(),
        "themes_css": report.themes_css.display().to_string(),
        "unions_ts": report.unions_ts.display().to_string(),
    }))
}

/// DTCG graph: tiers, alias edges, per-theme values — never CSS (T-I1).
pub fn graph(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let theme_id = args
        .get("themeId")
        .or_else(|| args.get("theme_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    graph_internal(theme_id, ctx)
}

/// Build alias graph from catalog (and optional project) theme JSON only.
pub fn graph_internal(theme_id: Option<&str>, ctx: &Ctx) -> Result<Value, OpError> {
    let id = theme_id.unwrap_or("ship");
    // Refuse path-like / traversal theme ids.
    if id.contains("..")
        || id.contains('/')
        || id.contains('\\')
        || id.contains('.') && id.contains('/')
    {
        return Err(OpError::InvalidArgument(format!("invalid themeId: {id}")));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(OpError::InvalidArgument(format!("invalid themeId: {id}")));
    }

    let catalog_rel = format!("{CATALOG_THEMES}/{id}.json");
    let mut docs: Vec<(String, Value)> = Vec::new();

    // Prefer project theme override when present.
    let project_candidate = find_project_theme(&ctx.workspace_root, id);
    if let Some(rel) = project_candidate {
        if is_read_theme_path(&rel) {
            if let Ok(v) = read_json_theme(ctx, &rel) {
                docs.push((rel, v));
            }
        }
    }
    if is_read_theme_path(&catalog_rel) {
        if let Ok(v) = read_json_theme(ctx, &catalog_rel) {
            docs.push((catalog_rel.clone(), v));
        }
    }
    // Always include base model for alias targets (JSON only).
    let base_rel = format!("{MODEL_ROOT}/base.json");
    if is_read_theme_path(&base_rel) {
        if let Ok(v) = read_json_theme(ctx, &base_rel) {
            docs.push((base_rel, v));
        }
    }

    if docs.is_empty() {
        return Err(OpError::NotFound(format!(
            "theme JSON not found for themeId={id}"
        )));
    }

    let mut nodes: BTreeMap<String, Value> = BTreeMap::new();
    let mut edges: Vec<Value> = Vec::new();
    let mut tiers: BTreeSet<String> = BTreeSet::new();

    for (source_file, doc) in &docs {
        walk_dtcg_graph(doc, "", source_file, &mut nodes, &mut edges, &mut tiers);
    }

    // Zero CSS reads: only .json paths above.
    Ok(json!({
        "ok": true,
        "themeId": id,
        "modelVersion": 1,
        "tiers": tiers.into_iter().collect::<Vec<_>>(),
        "nodes": nodes.into_values().collect::<Vec<_>>(),
        "edges": edges,
        "sources": docs.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>(),
    }))
}

fn find_project_theme(workspace: &Path, theme_id: &str) -> Option<String> {
    let apps = workspace.join("apps");
    let rd = std::fs::read_dir(apps).ok()?;
    for ent in rd.flatten() {
        let themes = ent
            .path()
            .join("tokens/themes")
            .join(format!("{theme_id}.json"));
        if themes.is_file() {
            let rel = themes
                .strip_prefix(workspace)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");
            return Some(rel);
        }
    }
    None
}

fn read_json_theme(ctx: &Ctx, rel: &str) -> Result<Value, OpError> {
    // Hard refuse CSS paths.
    if rel.ends_with(".css") || rel.contains(".css") {
        return Err(OpError::PermissionDenied(
            "theme graph refuses CSS paths".into(),
        ));
    }
    if !rel.ends_with(".json") {
        return Err(OpError::PermissionDenied(
            "theme graph reads JSON only".into(),
        ));
    }
    if !is_read_theme_path(rel) {
        return Err(OpError::PermissionDenied(format!(
            "path not readable: {rel}"
        )));
    }
    let target = ctx.workspace_root.join(rel);
    let canon = ensure_under_workspace(&ctx.workspace_root, &target)?;
    let text = std::fs::read_to_string(&canon)
        .map_err(|e| OpError::NotFound(format!("read {rel}: {e}")))?;
    serde_json::from_str(&text).map_err(|e| OpError::InvalidArgument(format!("json {rel}: {e}")))
}

fn walk_dtcg_graph(
    node: &Value,
    path: &str,
    source_file: &str,
    nodes: &mut BTreeMap<String, Value>,
    edges: &mut Vec<Value>,
    tiers: &mut BTreeSet<String>,
) {
    let Some(obj) = node.as_object() else {
        return;
    };
    if let Some(ext) = obj.get("$extensions").and_then(|v| v.as_object()) {
        if let Some(ship) = ext.get("ship").and_then(|v| v.as_object()) {
            if let Some(layer) = ship.get("layer").and_then(|v| v.as_str()) {
                tiers.insert(layer.to_string());
            }
        }
    }
    if obj.contains_key("$value") {
        let value = obj.get("$value").cloned().unwrap_or(Value::Null);
        let ty = obj
            .get("$type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut css_name = None;
        if let Some(ext) = obj.get("$extensions").and_then(|v| v.as_object()) {
            if let Some(ship) = ext.get("ship").and_then(|v| v.as_object()) {
                css_name = ship
                    .get("cssName")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }
        let value_str = match &value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        nodes.insert(
            path.to_string(),
            json!({
                "path": path,
                "type": ty,
                "value": value_str,
                "cssName": css_name,
                "source": source_file,
            }),
        );
        if let Some(alias) = parse_alias_ref(&value_str) {
            edges.push(json!({
                "from": path,
                "to": alias,
                "kind": "alias",
            }));
        }
        return;
    }
    for (k, child) in obj {
        if k.starts_with('$') {
            continue;
        }
        let next = if path.is_empty() {
            k.clone()
        } else {
            format!("{path}.{k}")
        };
        walk_dtcg_graph(child, &next, source_file, nodes, edges, tiers);
    }
}

fn parse_alias_ref(value: &str) -> Option<String> {
    let s = value.trim();
    if let Some(inner) = s.strip_prefix('{').and_then(|x| x.strip_suffix('}')) {
        let inner = inner.trim();
        if !inner.is_empty() && !inner.contains('{') {
            return Some(inner.to_string());
        }
    }
    None
}

/// List effective theme ids for a project (`project` arg or ctx.project_root).
pub fn effective_ids(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let workspace = resolve_under_workspace(
        args.get("workspace").and_then(|v| v.as_str()),
        &ctx.workspace_root,
        /* must_exist */ true,
    )?;
    let project = if let Some(p) = args.get("project").and_then(|v| v.as_str()) {
        resolve_path_under(&workspace, p)?
    } else {
        confine_existing(&workspace, &ctx.project_root)?
    };
    let ids = sail_compiler::tokens_generate::effective_theme_ids(&workspace, &project)
        .map_err(|e| OpError::FailedPrecondition(format!("effective theme ids: {e}")))?;
    Ok(json!({
        "ok": true,
        "project": ids.project,
        "catalog": ids.catalog,
        "all": ids.all_ids(),
    }))
}

/// Resolve `arg` relative to `workspace`, or use `workspace` when arg is None.
/// When `must_exist`, path must exist under workspace.
fn resolve_under_workspace(
    arg: Option<&str>,
    workspace_root: &Path,
    must_exist: bool,
) -> Result<PathBuf, OpError> {
    let path = match arg {
        Some(s) if !s.trim().is_empty() => {
            let pb = PathBuf::from(s.trim());
            if pb.is_absolute() {
                pb
            } else {
                workspace_root.join(pb)
            }
        }
        _ => workspace_root.to_path_buf(),
    };
    if must_exist {
        confine_existing(workspace_root, &path)
    } else {
        // Parent confinement for create-on-write dirs.
        ensure_under_workspace(workspace_root, &path)
    }
}

/// Join relative path under workspace (or accept absolute) and confine.
fn resolve_path_under(workspace: &Path, raw: &str) -> Result<PathBuf, OpError> {
    if raw.trim().is_empty() {
        return Err(OpError::InvalidArgument("path must be non-empty".into()));
    }
    if raw.contains("..") {
        return Err(OpError::PermissionDenied("path must not contain ..".into()));
    }
    let pb = PathBuf::from(raw.trim());
    let abs = if pb.is_absolute() {
        pb
    } else {
        workspace.join(pb)
    };
    if abs.exists() {
        confine_existing(workspace, &abs)
    } else {
        ensure_under_workspace(workspace, &abs)
    }
}

fn confine_existing(workspace: &Path, path: &Path) -> Result<PathBuf, OpError> {
    let canon_root = workspace
        .canonicalize()
        .map_err(|e| OpError::Internal(format!("workspace root {}: {e}", workspace.display())))?;
    let canon = path
        .canonicalize()
        .map_err(|e| OpError::InvalidArgument(format!("path invalid {}: {e}", path.display())))?;
    if !canon.starts_with(&canon_root) {
        return Err(OpError::PermissionDenied("path escapes workspace".into()));
    }
    Ok(canon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn graph_returns_alias_edges_and_refuses_css() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        std::fs::create_dir_all(ws.join("packages/sail/tokens/model/themes")).unwrap();
        let theme = r#"{
          "$extensions": { "ship": { "layer": "theme", "themeId": "toy" } },
          "modes": {
            "light": {
              "color": {
                "a": { "$value": "oklch(0.5 0 0)", "$type": "color" },
                "b": { "$value": "{color.a}", "$type": "color" }
              }
            }
          }
        }"#;
        std::fs::write(ws.join("packages/sail/tokens/model/themes/toy.json"), theme).unwrap();
        // Also base for optional merge
        std::fs::write(
            ws.join("packages/sail/tokens/model/base.json"),
            r#"{"x":{"$value":"1px","$type":"dimension"}}"#,
        )
        .unwrap();
        let ctx = Ctx::new(ws, ws.join("apps/demo"), ws);
        let g = graph(&json!({"themeId": "toy"}), &ctx).unwrap();
        assert_eq!(g["ok"], true);
        let edges = g["edges"].as_array().unwrap();
        assert!(
            edges
                .iter()
                .any(|e| e["kind"] == "alias" && e["to"] == "color.a"),
            "{edges:?}"
        );
        for s in g["sources"].as_array().unwrap() {
            let p = s.as_str().unwrap();
            assert!(p.ends_with(".json"), "{p}");
            assert!(!p.ends_with(".css"));
        }
        // Direct CSS refuse
        let bad = read_json_theme(&ctx, "packages/sail/tokens/src/tokens.css");
        assert!(matches!(bad, Err(OpError::PermissionDenied(_))));
    }

    #[test]
    fn write_refuses_non_monotonic_radius_scale() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        std::fs::create_dir_all(ws.join("apps/demo/tokens/themes")).unwrap();
        let ctx = Ctx::new(ws, ws.join("apps/demo"), ws);

        let bad = r#"{
          "$extensions": { "ship": { "themeId": "bad-radius" } },
          "radius": {
            "sm": { "$value": "8px", "$type": "dimension" },
            "md": { "$value": "4px", "$type": "dimension" },
            "lg": { "$value": "10px", "$type": "dimension" },
            "xl": { "$value": "14px", "$type": "dimension" },
            "$extensions": { "ordered": ["sm", "md", "lg", "xl"] }
          }
        }"#;
        let err = write(
            &json!({
                "path": "apps/demo/tokens/themes/bad-radius.json",
                "content": bad
            }),
            &ctx,
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("NonMonotonicScale"),
            "expected NonMonotonicScale, got {msg}"
        );
        assert!(!ws.join("apps/demo/tokens/themes/bad-radius.json").exists());

        let good = r#"{
          "$extensions": { "ship": { "themeId": "good-radius" } },
          "radius": {
            "sm": { "$value": "4px", "$type": "dimension" },
            "md": { "$value": "6px", "$type": "dimension" },
            "lg": { "$value": "10px", "$type": "dimension" },
            "xl": { "$value": "16px", "$type": "dimension" },
            "$extensions": { "ordered": ["sm", "md", "lg", "xl"] }
          }
        }"#;
        let ok = write(
            &json!({
                "path": "apps/demo/tokens/themes/good-radius.json",
                "content": good
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(ok["ok"], true);
    }

    #[test]
    fn validate_ordered_scales_named_error() {
        let doc = json!({
            "radius": {
                "sm": { "$value": "10px" },
                "md": { "$value": "2px" },
                "$extensions": { "ordered": ["sm", "md"] }
            }
        });
        let err = validate_ordered_scales(&doc).unwrap_err();
        assert!(err.to_string().contains("NonMonotonicScale"));
    }

    #[test]
    fn write_policy_apps_tokens_themes_json_only() {
        assert!(is_write_theme_path(
            "apps/sail-site/tokens/themes/custom.json"
        ));
        assert!(!is_write_theme_path("apps/sail-site/tokens/base.json"));
        assert!(!is_write_theme_path(
            "packages/sail/tokens/model/themes/ship.json"
        ));
        assert!(!is_write_theme_path(
            "apps/sail-site/tokens/themes/../secrets.json"
        ));
        assert!(!is_write_theme_path("apps/foo/tokens/themes/bar.txt"));
        assert!(is_read_theme_path(
            "packages/sail/tokens/model/themes/ship.json"
        ));
        assert!(is_read_theme_path("packages/sail/tokens/model/base.json"));
        assert!(is_read_theme_path(
            "packages/sail/tokens/model/components/card.json"
        ));
        assert!(!is_read_theme_path(
            "packages/sail/tokens/model/density-compact.json"
        ));
        assert!(is_list_theme_dir("apps/sail-site/tokens/themes"));
        assert!(is_list_theme_dir(CATALOG_THEMES));
        assert!(is_list_theme_dir(MODEL_COMPONENTS));
        assert!(!is_list_theme_dir("apps/sail-site/src"));
        assert!(!is_list_theme_dir(MODEL_ROOT));
    }

    #[test]
    fn write_refuses_catalog_and_escape() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        std::fs::create_dir_all(ws.join("apps/demo/tokens/themes")).unwrap();
        std::fs::create_dir_all(ws.join("packages/sail/tokens/model/themes")).unwrap();
        std::fs::write(
            ws.join("packages/sail/tokens/model/themes/ship.json"),
            r#"{"$extensions":{"ship":{"themeId":"ship"}}}"#,
        )
        .unwrap();

        let ctx = Ctx::new(ws, ws.join("apps/demo"), ws);

        let denied = write(
            &json!({
                "path": "packages/sail/tokens/model/themes/ship.json",
                "content": "{}"
            }),
            &ctx,
        );
        assert!(matches!(denied, Err(OpError::PermissionDenied(_))));

        let ok = write(
            &json!({
                "path": "apps/demo/tokens/themes/custom.json",
                "content": r#"{"ok":true}"#
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(ok["ok"], true);
        assert_eq!(
            std::fs::read_to_string(ws.join("apps/demo/tokens/themes/custom.json")).unwrap(),
            r#"{"ok":true}"#
        );
    }

    #[test]
    fn list_read_roundtrip() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        let themes = ws.join("apps/demo/tokens/themes");
        std::fs::create_dir_all(&themes).unwrap();
        std::fs::write(themes.join("a.json"), r#"{"a":1}"#).unwrap();
        std::fs::write(themes.join("b.json"), r#"{"b":2}"#).unwrap();

        let ctx = Ctx::new(ws, ws.join("apps/demo"), ws);
        let listed = list(&json!({"path": "apps/demo/tokens/themes"}), &ctx).unwrap();
        assert_eq!(listed["ok"], true);
        let files = listed["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);

        let r = read(&json!({"path": "apps/demo/tokens/themes/a.json"}), &ctx).unwrap();
        assert_eq!(r["content"], r#"{"a":1}"#);
    }

    #[test]
    fn fork_validates_new_id_and_fails_loud_on_bad_json() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        std::fs::create_dir_all(ws.join("apps/demo/tokens/themes")).unwrap();
        std::fs::create_dir_all(ws.join("packages/sail/tokens/model/themes")).unwrap();
        std::fs::write(
            ws.join("packages/sail/tokens/model/themes/ship.json"),
            r#"{"$extensions":{"ship":{"themeId":"ship"}},"modes":{"light":{},"dark":{}}}"#,
        )
        .unwrap();
        let ctx = Ctx::new(ws, ws.join("apps/demo"), ws);

        let bad_id = fork(
            &json!({
                "source_path": "packages/sail/tokens/model/themes/ship.json",
                "dest_path": "apps/demo/tokens/themes/bad.json",
                "new_id": "Not-Valid"
            }),
            &ctx,
        );
        assert!(matches!(bad_id, Err(OpError::InvalidArgument(_))));

        let bad_json = fork(
            &json!({
                "source_path": "packages/sail/tokens/model/themes/ship.json",
                "dest_path": "apps/demo/tokens/themes/x.json",
                "new_id": "ok-id",
                "content": "not-json"
            }),
            &ctx,
        );
        assert!(matches!(bad_json, Err(OpError::InvalidArgument(_))));

        let ok = fork(
            &json!({
                "source_path": "packages/sail/tokens/model/themes/ship.json",
                "dest_path": "apps/demo/tokens/themes/my-fork.json",
                "new_id": "my-fork"
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(ok["ok"], true);
        let body =
            std::fs::read_to_string(ws.join("apps/demo/tokens/themes/my-fork.json")).unwrap();
        assert!(
            body.contains("\"themeId\": \"my-fork\"") || body.contains("\"themeId\":\"my-fork\"")
        );
    }

    #[test]
    fn generate_paths_confined_under_workspace() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        std::fs::create_dir_all(ws.join("apps/demo")).unwrap();
        let ctx = Ctx::new(ws, ws.join("apps/demo"), ws);
        let outside = tempdir().unwrap();
        let escaped = generate(
            &json!({
                "mode": "package",
                "out": outside.path().join("evil").to_string_lossy(),
            }),
            &ctx,
        );
        assert!(matches!(escaped, Err(OpError::PermissionDenied(_))));
    }
}
