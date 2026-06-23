//! `GET /_deck/source/{tag}` — render a simple HTML page showing the
//! `.sui` template and `.controller.ts` for a SUI component.
//!
//! Powers the "View source" deep-link in the component studio. The component
//! source trees (the primitives bucket at `packages/components/sui/` and the
//! domain bucket at `packages/components/ship/`) are read from disk at request
//! time. This is a dev feature; a 404 outside the workspace is fine.
//!
//! Look-ups go through `is_valid_tag()` — only `sui-[a-z0-9-]+` matches;
//! anything else (path traversal, uppercase, foreign prefixes) returns 400.

use std::sync::LazyLock;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

/// Workspace root — three parents up from this crate's `CARGO_MANIFEST_DIR`
/// (`crates/deck/deck-axum` → repo root), walked once at first access.
static REPO_ROOT: LazyLock<Option<std::path::PathBuf>> = LazyLock::new(|| {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..3 {
        if !d.pop() {
            return None;
        }
    }
    if d.join("package.json").exists() && d.join("packages").is_dir() {
        Some(d)
    } else {
        None
    }
});

/// Tag whitelist — lowercase ASCII, digits, and hyphens after `sui-`.
/// Rejects path-traversal (`..`, `/`, `\\`) and any non-component name.
pub(crate) fn is_valid_tag(tag: &str) -> bool {
    if !tag.starts_with("sui-") {
        return false;
    }
    if tag.len() < 5 || tag.len() > 64 {
        return false;
    }
    tag.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn lookup_file(tag: &str, suffix: &str) -> Option<String> {
    let root = REPO_ROOT.as_ref()?;
    for bucket in ["sui", "ship"] {
        let pkg_dir = std::fs::read_dir(root.join("packages/components").join(bucket)).ok()?;
        for entry in pkg_dir.flatten() {
            let src = entry.path().join("src").join(tag);
            let file = src.join(format!("{tag}{suffix}"));
            if let Ok(body) = std::fs::read_to_string(&file) {
                return Some(body);
            }
        }
    }
    None
}

/// GET /_deck/source/{tag}
pub async fn serve_sui_source(Path(tag): Path<String>) -> Response {
    if !is_valid_tag(&tag) {
        return (StatusCode::BAD_REQUEST, "invalid tag").into_response();
    }

    let Some(sui_src) = lookup_file(&tag, ".sui") else {
        return (StatusCode::NOT_FOUND, "no such component").into_response();
    };
    let ctrl_src = lookup_file(&tag, ".controller.ts");

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
<h2>page.sui</h2>\
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
        assert!(is_valid_tag("sui-button"));
        assert!(is_valid_tag("sui-icon"));
        assert!(is_valid_tag("sui-pipeline-dag"));
    }

    #[test]
    fn invalid_tag_rejected() {
        assert!(!is_valid_tag("ship-button"));
        assert!(!is_valid_tag("sui-../etc"));
        assert!(!is_valid_tag("sui-../etc/passwd"));
        assert!(!is_valid_tag("sui-Button"));
        assert!(!is_valid_tag("sui-"));
        assert!(!is_valid_tag(&format!("sui-{}", "a".repeat(64))));
    }

    #[test]
    fn html_escape_handles_special_chars() {
        assert_eq!(html_escape("<a>"), "&lt;a&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"q\""), "&quot;q&quot;");
    }
}
