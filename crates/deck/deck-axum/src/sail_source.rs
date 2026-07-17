//! `GET /_deck/source/{tag}` — render a simple HTML page showing the
//! `.sail` template and `.controller.ts` for a SAIL component.
//!
//! Powers the "View source" deep-link in the component studio. The component
//! source trees under `packages/components/*` (EVERY bucket, discovered
//! generically — no hardcoded bucket names) are read from disk at request time.
//!
//! The component root is derived FROM the mount's project root — the per-request
//! [`deck_core::Resolution`], the SAME single root source the rest of deck
//! resolves against — walking up to the nearest `packages/components/`. There is
//! no compile-time manifest-dir bake (which broke in any installed binary) and
//! deck names no domain-specific bucket here. This is a dev feature; a 404
//! outside a workspace (or with no Resolution carried) is fine.
//!
//! Look-ups go through `is_valid_tag()` — only `sail-[a-z0-9-]+` matches;
//! anything else (path traversal, uppercase, foreign prefixes) returns 400.

use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

/// Tag whitelist — lowercase ASCII, digits, and hyphens after `sail-`.
/// Rejects path-traversal (`..`, `/`, `\\`) and any non-component name.
pub(crate) fn is_valid_tag(tag: &str) -> bool {
    if !tag.starts_with("sail-") {
        return false;
    }
    if tag.len() < 6 || tag.len() > 64 {
        return false;
    }
    tag.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Walk up from `start` (bounded) to the nearest ancestor holding a
/// `packages/components/` dir — where SAIL + consumer component packs live. This
/// derives the component-source root FROM the mount's single project root, not a
/// compile-time constant, so it works in an installed binary and outside the
/// framework build tree.
fn find_components_dir(start: &FsPath) -> Option<PathBuf> {
    let mut cur = start;
    for _ in 0..32 {
        let candidate = cur.join("packages").join("components");
        if candidate.is_dir() {
            return Some(candidate);
        }
        cur = cur.parent()?;
    }
    None
}

/// Read `<pkg>/src/<tag>/<tag><suffix>` under any pack in ANY bucket dir of
/// `components_dir` (`packages/components/<bucket>/<pkg>/…`). Buckets are
/// discovered by directory listing — deck hardcodes no `sail`/`ship` name.
fn lookup_file(components_dir: &FsPath, tag: &str, suffix: &str) -> Option<String> {
    let buckets = std::fs::read_dir(components_dir).ok()?;
    for bucket in buckets.flatten() {
        let Ok(packs) = std::fs::read_dir(bucket.path()) else {
            continue;
        };
        for pack in packs.flatten() {
            let file = pack.path().join("src").join(tag).join(format!("{tag}{suffix}"));
            if let Ok(body) = std::fs::read_to_string(&file) {
                return Some(body);
            }
        }
    }
    None
}

/// GET /_deck/source/{tag}
pub async fn serve_sail_source(
    resolution: Option<Extension<Arc<deck_core::Resolution>>>,
    Path(tag): Path<String>,
) -> Response {
    if !is_valid_tag(&tag) {
        return (StatusCode::BAD_REQUEST, "invalid tag").into_response();
    }
    // Mount-scoped: resolve the component root from THIS request's project root.
    let Some(root) = resolution.and_then(|e| e.0.project_root().map(|p| p.to_path_buf())) else {
        return (StatusCode::NOT_FOUND, "source viewer needs a project root").into_response();
    };
    let Some(components) = find_components_dir(&root) else {
        return (StatusCode::NOT_FOUND, "no packages/components under the project root").into_response();
    };

    let Some(sui_src) = lookup_file(&components, &tag, ".sail") else {
        return (StatusCode::NOT_FOUND, "no such component").into_response();
    };
    let ctrl_src = lookup_file(&components, &tag, ".controller.ts");

    let body = render_html(&tag, &sui_src, ctrl_src.as_deref());
    let mut resp = Html(body).into_response();
    resp.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-cache"),
    );
    resp
}

fn render_html(tag: &str, sui_src: &str, ctrl_src: Option<&str>) -> String {
    let ctrl = ctrl_src
        .map(html_escape)
        .unwrap_or_else(|| "<em>(no controller.ts)</em>".to_string());
    format!(
        "<!doctype html><meta charset=\"utf-8\">\
<title>source · {tag}</title>\
<style>\
body{{font:14px/1.5 ui-monospace,monospace;background:#0e0e10;color:#e6e6e6;margin:0;padding:1.5rem}}\
h1{{font:600 1.1rem ui-sans-serif,system-ui;margin:0 0 1rem;color:#9cdcfe}}\
h2{{font:600 .9rem ui-sans-serif,system-ui;margin:1.5rem 0 .5rem;color:#9cdcfe}}\
pre{{background:#1a1a1d;padding:1rem;border-radius:6px;overflow:auto;margin:0}}\
</style>\
<h1>&lt;{tag}&gt; source</h1>\
<h2>page.sail</h2>\
<pre>{sui_src_esc}</pre>\
<h2>controller.ts</h2>\
<pre>{ctrl}</pre>",
        sui_src_esc = html_escape(sui_src),
    )
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_tag_passes() {
        assert!(is_valid_tag("sail-button"));
        assert!(is_valid_tag("sail-icon"));
        assert!(is_valid_tag("sail-pipeline-dag"));
    }

    #[test]
    fn invalid_tag_rejected() {
        assert!(!is_valid_tag("ship-button"));
        assert!(!is_valid_tag("sail-../etc"));
        assert!(!is_valid_tag("sail-../etc/passwd"));
        assert!(!is_valid_tag("sail-Button"));
        assert!(!is_valid_tag("sail-"));
        assert!(!is_valid_tag(&format!("sail-{}", "a".repeat(64))));
    }

    #[test]
    fn html_escape_handles_special_chars() {
        assert_eq!(html_escape("<a>"), "&lt;a&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"q\""), "&quot;q&quot;");
    }

    /// The component root is derived from the (runtime) project root by walking
    /// up to `packages/components/` — NOT a compile-time `CARGO_MANIFEST_DIR`
    /// bake — and lookups scan EVERY bucket generically (here an arbitrary
    /// `acme` bucket, proving no `sail`/`ship` hardcode).
    #[test]
    fn lookup_reads_any_bucket_from_the_runtime_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A pack in a non-sail/non-ship bucket: packages/components/acme/widgets/…
        let src = root.join("packages/components/acme/widgets/src/sail-thing");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("sail-thing.sail"), "<div>hi</div>").unwrap();

        // Derive the components dir from a DEEPLY nested project root (walk up).
        let nested = root.join("apps/demo");
        std::fs::create_dir_all(&nested).unwrap();
        let components = super::find_components_dir(&nested).expect("finds packages/components upward");
        assert_eq!(components, root.join("packages").join("components"));

        assert_eq!(
            super::lookup_file(&components, "sail-thing", ".sail").as_deref(),
            Some("<div>hi</div>"),
            "reads from an arbitrary bucket — no hardcoded sail/ship"
        );
        assert!(
            super::lookup_file(&components, "sail-missing", ".sail").is_none(),
            "absent tag → None"
        );
    }

    /// Source-scan guard: the compile-time hardcodes this thread removed must
    /// stay gone — no `env!` manifest-dir bake, no `packages/components/<domain>`
    /// bucket literal in this module. Needles are split via `concat!` so this
    /// test can't match its OWN source.
    #[test]
    fn no_compile_time_root_or_domain_hardcode() {
        let src = include_str!("sail_source.rs");
        let manifest_bake = concat!("env!(\"CARGO_", "MANIFEST_DIR\")");
        let sail_bucket = concat!("components/", "sail");
        let ship_bucket = concat!("components/", "ship");
        assert!(!src.contains(manifest_bake), "no compile-time manifest-dir bake");
        assert!(!src.contains(sail_bucket), "no hardcoded sail bucket path");
        assert!(!src.contains(ship_bucket), "no hardcoded ship bucket path");
    }
}
