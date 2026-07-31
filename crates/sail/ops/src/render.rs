//! `SailRender` RPCs — static themed screenshot from `.sail` source.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::ctx::Ctx;
use crate::error::OpError;
use crate::watch::relativize_watch_path;

#[cfg(test)]
use crate::inspect;

/// Path confinement — same rules as SailInspect (workspace-relative `.sail`).
fn resolve_sail_path(path: &str, ctx: &Ctx) -> Result<(PathBuf, String), OpError> {
    let raw = path.trim();
    if raw.is_empty() {
        return Err(OpError::InvalidArgument("path is required".into()));
    }
    if raw.contains("..") {
        return Err(OpError::PermissionDenied(
            "path must not contain `..`".into(),
        ));
    }
    if !raw.ends_with(".sail") {
        return Err(OpError::InvalidArgument("path must end with .sail".into()));
    }
    if Path::new(raw).is_absolute() {
        return Err(OpError::PermissionDenied(
            "path must be workspace-relative (absolute paths refused)".into(),
        ));
    }
    let under_ws = ctx.workspace_root.join(raw);
    let candidate = if under_ws.exists() {
        under_ws
    } else {
        ctx.project_root.join(raw)
    };
    let canon_root = ctx
        .workspace_root
        .canonicalize()
        .map_err(|e| OpError::Internal(format!("workspace root: {e}")))?;
    if let Ok(partial) = candidate.canonicalize() {
        if !partial.starts_with(&canon_root) {
            return Err(OpError::PermissionDenied("path escapes workspace".into()));
        }
    } else {
        let parent = candidate.parent().unwrap_or(candidate.as_path());
        if let Ok(cp) = parent.canonicalize() {
            if !cp.starts_with(&canon_root) {
                return Err(OpError::PermissionDenied("path escapes workspace".into()));
            }
        }
        return Err(OpError::NotFound(format!("read {raw}: not found")));
    }
    let canon = candidate
        .canonicalize()
        .map_err(|e| OpError::NotFound(format!("read {raw}: {e}")))?;
    if !canon.starts_with(&canon_root) {
        return Err(OpError::PermissionDenied("path escapes workspace".into()));
    }
    let rel = relativize_watch_path(&ctx.workspace_root, &canon);
    Ok((canon, rel))
}

fn parse_viewport(args: &Value) -> (u32, u32) {
    if let Some(s) = args.get("viewport").and_then(|v| v.as_str()) {
        if let Some((w, h)) = s.split_once('x') {
            if let (Ok(w), Ok(h)) = (w.parse(), h.parse()) {
                return (w, h);
            }
        }
    }
    if let Some(vp) = args.get("viewport") {
        let w = vp.get("width").and_then(|v| v.as_u64()).unwrap_or(1440) as u32;
        let h = vp.get("height").and_then(|v| v.as_u64()).unwrap_or(900) as u32;
        return (w.max(1), h.max(1));
    }
    (1440, 900)
}

/// `SailRender/Screenshot`.
pub fn screenshot(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path = args.get("path").and_then(|v| v.as_str());
    let source_inline = args.get("source").and_then(|v| v.as_str());
    let theme = args
        .get("theme")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let viewport = parse_viewport(args);

    let (source, file_label, abs_for_join): (String, String, Option<PathBuf>) =
        match (path, source_inline) {
            (Some(p), None) => {
                let (abs, rel) = resolve_sail_path(p, ctx)?;
                let text = std::fs::read_to_string(&abs)
                    .map_err(|e| OpError::NotFound(format!("read {rel}: {e}")))?;
                (text, rel, Some(abs))
            }
            (None, Some(s)) => (s.to_string(), "_inline.sail".into(), None),
            (Some(p), Some(s)) => {
                // Inline source wins; path still fully confined for file label + join.
                let (_abs, rel) = resolve_sail_path(p, ctx)?;
                (s.to_string(), rel, None)
            }
            (None, None) => {
                return Err(OpError::InvalidArgument(
                    "Screenshot requires `path` and/or `source`".into(),
                ));
            }
        };

    // App FontAdd vendors share `<project>/fonts` with Studio + bundler.
    let app_fonts_dir = {
        let p = ctx.project_root.join("fonts");
        if p.is_dir() { Some(p) } else { None }
    };
    let opts = sail_render::RenderOptions {
        viewport,
        fonts_dir: None,
        app_fonts_dir,
        theme,
        mode,
        workspace: Some(ctx.workspace_root.clone()),
        file: Some(file_label.clone()),
        drop_bg_token: None,
    };

    let out = if let Some(abs) = abs_for_join {
        sail_render::render_file(&abs, opts)
    } else {
        sail_render::render_source(&source, opts)
    }
    .map_err(|e| OpError::Internal(format!("render: {e:#}")))?;

    // Regions already carry SailInspect-compatible node ids (library join).
    // No post-hoc substitution — CLI and ops agree.
    let regions = serde_json::to_value(&out.regions).unwrap_or(json!([]));
    let dynamics = serde_json::to_value(&out.dynamics).unwrap_or(json!([]));

    let png =
        sail_render::png_bytes(&out.image).map_err(|e| OpError::Internal(format!("png: {e:#}")))?;
    let image_b64 = base64_encode(&png);

    let theme_label = format!("{}:{}", out.theme.data_theme(), out.theme.data_mode());

    Ok(json!({
        "imagePng": image_b64,
        "regions": regions,
        "dynamics": dynamics,
        "viewport": { "width": out.viewport.0, "height": out.viewport.1 },
        "theme": theme_label,
        "ms": out.ms,
        "file": file_label,
    }))
}

fn base64_encode(bytes: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(T[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(T[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ctx;

    fn workspace_ctx() -> Ctx {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for _ in 0..8 {
            if dir.join("Cargo.toml").is_file() && dir.join("packages/sail/tokens").is_dir() {
                return Ctx::new(&dir, &dir, &dir);
            }
            if !dir.pop() {
                break;
            }
        }
        Ctx::from_env()
    }

    #[test]
    fn screenshot_inline_painted_no_leak() {
        let ctx = workspace_ctx();
        let args = json!({
            "source": r#"<div sx={style.bg(primary).pad(2)}>Hi {ticks()}</div>"#,
            "viewport": "200x100",
            "theme": "ship",
            "mode": "light"
        });
        let out = screenshot(&args, &ctx).expect("screenshot");
        assert!(out.get("imagePng").and_then(|v| v.as_str()).unwrap().len() > 32);
        assert!(out["ms"].as_u64().unwrap_or(9999) < 2000);
        let dyns = out["dynamics"].as_array().unwrap();
        assert!(
            dyns.iter()
                .any(|d| d["reason"].as_str().unwrap_or("").contains("ticks")),
            "{dyns:?}"
        );
        // Dynamics carry parent-joined node ids (not dyn.expr provisional).
        for d in dyns {
            let id = d["nodeId"].as_str().unwrap_or("");
            assert!(
                !id.contains("dyn.expr"),
                "dynamics should join to author node ids, got {id}"
            );
        }
    }

    #[test]
    fn screenshot_path_regions_resolve_to_inspect_ids() {
        let ctx = workspace_ctx();
        let rel = "crates/sail/render/tests/fixtures/paint.sail";
        let abs = ctx.workspace_root.join(rel);
        assert!(
            abs.is_file(),
            "committed fixture missing: {} (workspace {})",
            abs.display(),
            ctx.workspace_root.display()
        );
        let args = json!({
            "path": rel,
            "viewport": "400x300",
            "theme": "ship",
            "mode": "light"
        });
        let out = screenshot(&args, &ctx).expect("screenshot");
        let regions = out["regions"].as_array().expect("regions");
        assert!(!regions.is_empty());

        let model = inspect::model(
            &json!({"path": rel, "viewport": {"width": 400, "height": 300}}),
            &ctx,
        )
        .expect("model");
        let tree_ids: std::collections::BTreeSet<_> = model["tree"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|n| n["id"].as_str())
            .collect();
        let layout_ids: std::collections::BTreeSet<_> = model["layout"]["boxes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|b| b["id"].as_str())
            .collect();
        // True join: renderer region ids must appear in inspect tree/layout —
        // not a wholesale replacement of inspect boxes.
        for r in regions {
            let id = r["nodeId"].as_str().unwrap();
            assert!(
                layout_ids.contains(id) || tree_ids.contains(id),
                "region id {id} not in inspect model (join failed)"
            );
        }
        // Dynamics also resolve.
        for d in out["dynamics"].as_array().unwrap() {
            let id = d["nodeId"].as_str().unwrap();
            assert!(
                layout_ids.contains(id) || tree_ids.contains(id),
                "dynamic id {id} not in inspect model"
            );
        }
    }

    #[test]
    fn screenshot_refuses_path_escape() {
        let ctx = workspace_ctx();
        let err = screenshot(&json!({"path": "../secret.sail"}), &ctx).unwrap_err();
        assert!(matches!(err, OpError::PermissionDenied(_)), "{err}");
        let err = screenshot(&json!({"path": "/etc/passwd.sail"}), &ctx).unwrap_err();
        assert!(matches!(err, OpError::PermissionDenied(_)), "{err}");
    }

    #[test]
    fn screenshot_path_plus_source_confines_path() {
        let ctx = workspace_ctx();
        let err = screenshot(
            &json!({
                "path": "../escape.sail",
                "source": "<div>x</div>"
            }),
            &ctx,
        )
        .unwrap_err();
        assert!(matches!(err, OpError::PermissionDenied(_)), "{err}");
    }
}
