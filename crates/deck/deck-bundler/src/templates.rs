//! `index.html` template for deck projects.
//!
//! The dev doesn't author `index.html` — deck generates it. This template
//! is the SSOT for the dev-facing SPA entry shape. The bundler renders one
//! `<project>/.deck/index.html` per project by substituting:
//!   * `{{project_name}}`     — `package.json#name`
//!   * `{{importmap}}`         — the generated importmap JSON
//!   * `{{entry_module}}`      — the generated `routes.gen.ts` (or whatever
//!                                the bundler decides to emit as the entry)
//!
//! The template is intentionally minimal. The dev's project adds static
//! assets via `apps/<x>/public/` (copied verbatim to `<x>/.deck/public/`).
//! Anything fancier (theme pre-paint, font preloading, deck-mode metadata)
//! is a separate concern — the template can grow as needed.
//!
//! The importmap MUST come before any module script: browsers freeze the
//! importmap at first module fetch, so it lives at the top of `<head>` to
//! win the race against `routes.gen.ts` and any per-view bundles.

/// The DOM id of the SPA mount point. Single source for the `index.html`
/// `<div id>` AND the `mountRouter({ root })` call the route emitter writes
/// (`routes.rs`) — change it here and both move together, no hardcoded
/// literal drifting on either side.
pub const ROOT_ID: &str = "root";

/// The minimal `index.html` template for a deck project.
pub const INDEX_HTML_TEMPLATE: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{{project_name}}</title>

  <!-- Importmap: bare specifier → daemon-served ESM URL. The bundler
       writes this from FRAMEWORK_ASSETS at build time. -->
  <script type="importmap">
{{importmap}}
  </script>

  <!-- Base reset. Kills the UA's default 8px body margin (which otherwise
       frames a full-bleed app in a white gutter — invisible in light mode,
       ugly in dark) and lets the document surface adopt the theme tokens, so
       dark mode is dark edge-to-edge and overscroll matches. Apps that load no
       tokens fall back to the UA canvas. -->
  <style>
    html { background: var(--background); color: var(--foreground); }
    body { margin: 0; min-height: 100vh; }
  </style>
</head>
<body>
  <div id="{{root_id}}"></div>
  <script type="module" src="/{{entry_module}}"></script>
</body>
</html>
"#;

/// Render the template with substitutions.
///
/// `importmap_json` must be a pre-serialized JSON object (i.e. the
/// `importmap` field's value as a JSON string with two-space indent).
/// `entry_module` is the file name (no path) of the generated entry —
/// `routes.gen.ts` is the current convention.
pub fn render_index_html(
    project_name: &str,
    importmap_json: &str,
    entry_module: &str,
) -> String {
    INDEX_HTML_TEMPLATE
        .replace("{{root_id}}", ROOT_ID)
        .replace("{{project_name}}", project_name)
        .replace("{{importmap}}", importmap_json)
        .replace("{{entry_module}}", entry_module)
}
