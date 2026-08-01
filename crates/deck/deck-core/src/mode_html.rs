//! Inject `<meta name="ship-*">` tags into served HTML shells.
//!
//! shell's index.html ships a static <meta name="deck-mode"
//! content="development"> tag. When shipd serves the same HTML in
//! production mode it rewrites that tag via `inject_mode_meta` so the
//! shell's status bar can render the DEV badge appropriately. The same
//! rewrite-or-insert mechanism surfaces the app's render mode via
//! `inject_render_meta` (`<meta name="deck-render">`) so CSR-vs-SSR is
//! legible client-side rather than implicit.

use crate::DaemonMode;
use deck_config::RenderMode;

/// Short string label for the daemon mode. Used as the `content`
/// attribute of `<meta name="deck-mode">`.
pub fn mode_str(mode: DaemonMode) -> &'static str {
    match mode {
        DaemonMode::Development => "development",
        DaemonMode::Production => "production",
    }
}

/// Short string label for the render mode. Used as the `content`
/// attribute of `<meta name="deck-render">` (`csr` / `ssr`).
pub fn render_str(render: RenderMode) -> &'static str {
    match render {
        RenderMode::Csr => "csr",
        RenderMode::Ssr => "ssr",
    }
}

/// Rewrite (or insert) a `<meta name="{name}" content="{value}" />` tag in an
/// HTML string. If a tag with that `name` already exists (with any `content`),
/// replace the whole tag; otherwise insert a fresh one right after `<head>`
/// so it appears before scripts. No `<head>` ⇒ returned unchanged.
fn inject_named_meta(html: &str, name: &str, value: &str) -> String {
    let needle = format!("<meta name=\"{name}\"");
    let replacement = format!("<meta name=\"{name}\" content=\"{value}\" />");
    if let Some(start) = html.find(&needle) {
        // Find the closing `>` of this tag.
        if let Some(rel_end) = html[start..].find('>') {
            let end = start + rel_end + 1;
            let mut out = String::with_capacity(html.len() + replacement.len());
            out.push_str(&html[..start]);
            out.push_str(&replacement);
            out.push_str(&html[end..]);
            return out;
        }
    }
    // Tag missing — insert after <head>.
    let head_open = "<head>";
    if let Some(idx) = html.find(head_open) {
        let insert_at = idx + head_open.len();
        let mut out = String::with_capacity(html.len() + replacement.len() + 4);
        out.push_str(&html[..insert_at]);
        out.push_str("\n  ");
        out.push_str(&replacement);
        out.push_str(&html[insert_at..]);
        return out;
    }
    // No <head> — return as-is.
    html.to_string()
}

/// Rewrite (or insert) the `<meta name="deck-mode" content="…">` tag
/// in an HTML string to reflect the daemon's current mode.
pub fn inject_mode_meta(html: &str, mode: DaemonMode) -> String {
    inject_named_meta(html, "deck-mode", mode_str(mode))
}

/// Rewrite (or insert) the `<meta name="deck-render" content="csr|ssr">` tag
/// so the app's active render mode is observable client-side.
pub fn inject_render_meta(html: &str, render: RenderMode) -> String {
    inject_named_meta(html, "deck-render", render_str(render))
}

/// Multi-app mount path (e.g. `/_studio`). CSR `mountRouter` strips this
/// prefix before matching generated route tables that still use `/`.
/// Empty or `"/"` → no meta (single-app at root).
pub fn inject_app_route_meta(html: &str, route: &str) -> String {
    let r = route.trim();
    if r.is_empty() || r == "/" {
        return html.to_string();
    }
    // Normalize: no trailing slash except for bare `/` (already returned).
    let r = r.trim_end_matches('/');
    if r.is_empty() {
        return html.to_string();
    }
    inject_named_meta(html, "deck-app-route", r)
}

/// Workspace-relative project path for studio / devtools (`apps/<name>`).
///
/// Walks the absolute project root string for an `/apps/<segment>` suffix.
/// Standalone projects outside a monorepo `apps/` tree return `None` (studio
/// falls back to heuristics).
pub fn sail_project_rel_from_root(project_root: &std::path::Path) -> Option<String> {
    let norm = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let s = norm.to_string_lossy().replace('\\', "/");
    if let Some(i) = s.rfind("/apps/") {
        let rel = s[i + 1..].to_string(); // "apps/foo"
        if rel.contains("..") || rel.is_empty() {
            return None;
        }
        // Require apps/<one-or-more segments> with no trailing junk.
        let rest = rel.strip_prefix("apps/")?;
        if rest.is_empty() || rest.contains("//") {
            return None;
        }
        return Some(rel);
    }
    None
}

/// Inject `sail-project` + `sail-project-themes` meta for the active app so the
/// portable design studio lists/saves the correct project themes dir.
///
/// Values are workspace-relative (`apps/<name>` / `apps/<name>/tokens/themes`).
pub fn inject_project_themes_meta(html: &str, project_root: &std::path::Path) -> String {
    let Some(project) = sail_project_rel_from_root(project_root) else {
        return html.to_string();
    };
    let themes = format!("{project}/tokens/themes");
    let html = inject_named_meta(html, "sail-project", &project);
    inject_named_meta(&html, "sail-project-themes", &themes)
}

/// Walk ancestors for the monorepo root that owns `packages/sail/tokens/model`.
fn workspace_root_for_themes(project_root: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut cur = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    loop {
        if cur.join("packages/sail/tokens/model/themes").is_dir() {
            return Some(cur);
        }
        cur = cur.parent()?.to_path_buf();
    }
}

/// Inject `<meta name="sail-effective-themes" content="id1,id2,…">` so the
/// product theme picker lists only the T0 effective emit set (project themes ∪
/// `theme.enabled` allowlist), matching `SailTheme/EffectiveIds`.
///
/// When the workspace cannot be resolved or effective ids fail, HTML is
/// unchanged (client falls back to full `THEMES` or ops fetch).
pub fn inject_effective_themes_meta(html: &str, project_root: &std::path::Path) -> String {
    let Some(workspace) = workspace_root_for_themes(project_root) else {
        return html.to_string();
    };
    let Ok(ids) = sail_compiler::tokens_generate::effective_theme_ids(&workspace, project_root)
    else {
        return html.to_string();
    };
    let mut safe: Vec<String> = Vec::new();
    for id in ids.all_ids() {
        // Same shape as compiler validate_theme_id — never stamp path-like ids.
        if sail_compiler::tokens_generate::validate_theme_id(&id).is_ok() {
            safe.push(id);
        }
    }
    // Empty content = explicit empty emit set (picker shows no catalog rows).
    inject_named_meta(html, "sail-effective-themes", &safe.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_existing_tag() {
        let html = "<head>\n<meta name=\"deck-mode\" content=\"development\" />\n</head>";
        let out = inject_mode_meta(html, DaemonMode::Production);
        assert!(out.contains("content=\"production\""));
        assert!(!out.contains("content=\"development\""));
    }

    #[test]
    fn inserts_when_missing() {
        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_mode_meta(html, DaemonMode::Development);
        assert!(out.contains("<meta name=\"deck-mode\" content=\"development\""));
    }

    #[test]
    fn no_head_returns_unchanged() {
        let html = "<html><body>x</body></html>";
        let out = inject_mode_meta(html, DaemonMode::Production);
        assert_eq!(out, html);
    }

    #[test]
    fn render_meta_inserted_when_missing() {
        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_render_meta(html, RenderMode::Ssr);
        assert!(out.contains("<meta name=\"deck-render\" content=\"ssr\""));
    }

    #[test]
    fn render_meta_rewrites_existing_tag() {
        let html = "<head>\n<meta name=\"deck-render\" content=\"csr\" />\n</head>";
        let out = inject_render_meta(html, RenderMode::Ssr);
        assert!(out.contains("content=\"ssr\""));
        assert!(!out.contains("content=\"csr\""));
    }

    /// The two meta injectors are independent — injecting render leaves an
    /// already-present deck-mode tag intact (different `name`).
    #[test]
    fn render_and_mode_meta_coexist() {
        let html = "<head>\n<meta name=\"deck-mode\" content=\"development\" />\n</head>";
        let out = inject_render_meta(html, RenderMode::Csr);
        assert!(out.contains("name=\"deck-mode\" content=\"development\""));
        assert!(out.contains("name=\"deck-render\" content=\"csr\""));
    }

    #[test]
    fn sail_project_rel_from_apps_path() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("apps").join("sail-site");
        std::fs::create_dir_all(&app).unwrap();
        let rel = sail_project_rel_from_root(&app).expect("apps path");
        assert_eq!(rel, "apps/sail-site");
    }

    #[test]
    fn inject_project_themes_meta_stamps_both() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("apps").join("my-app");
        std::fs::create_dir_all(&app).unwrap();
        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_project_themes_meta(html, &app);
        assert!(
            out.contains("name=\"sail-project\" content=\"apps/my-app\""),
            "{out}"
        );
        assert!(
            out.contains("name=\"sail-project-themes\" content=\"apps/my-app/tokens/themes\""),
            "{out}"
        );
    }

    #[test]
    fn inject_project_themes_meta_noop_outside_apps() {
        let dir = tempfile::tempdir().unwrap();
        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_project_themes_meta(html, dir.path());
        assert!(!out.contains("sail-project"));
        assert_eq!(out, html);
    }

    #[test]
    fn inject_effective_themes_meta_stamps_allowlist() {
        // Self-contained fixture: minimal catalog stems under packages/.../themes
        // so the test always runs (no real monorepo / symlink / OS skip).
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path();
        let catalog = workspace.join("packages/sail/tokens/model/themes");
        std::fs::create_dir_all(&catalog).unwrap();
        for id in ["neutral", "paper", "synthwave", "glass"] {
            // list_theme_ids_in_dir only needs *.json stems; body unused.
            std::fs::write(catalog.join(format!("{id}.json")), "{}\n").unwrap();
        }
        let app = workspace.join("apps").join("thin-app");
        std::fs::create_dir_all(app.join("tokens/themes")).unwrap();
        // Thin allowlist — only neutral + paper from catalog.
        std::fs::write(
            app.join("sail.config.jsonc"),
            r#"{ "theme": { "default": "neutral", "enabled": ["neutral", "paper"] } }"#,
        )
        .unwrap();

        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_effective_themes_meta(html, &app);
        assert!(
            out.contains("name=\"sail-effective-themes\""),
            "missing meta: {out}"
        );
        let needle = "name=\"sail-effective-themes\" content=\"";
        let start = out.find(needle).expect("content attr") + needle.len();
        let end = out[start..].find('"').expect("close quote") + start;
        let content = &out[start..end];
        let parts: Vec<&str> = content.split(',').filter(|s| !s.is_empty()).collect();
        assert!(parts.contains(&"neutral"), "{content}");
        assert!(parts.contains(&"paper"), "{content}");
        assert!(
            !parts.contains(&"synthwave"),
            "full catalog leaked: {content}"
        );
        assert!(!parts.contains(&"glass"), "full catalog leaked: {content}");
    }

    #[test]
    fn inject_effective_themes_meta_noop_without_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_effective_themes_meta(html, dir.path());
        assert!(!out.contains("sail-effective-themes"));
        assert_eq!(out, html);
    }
}
