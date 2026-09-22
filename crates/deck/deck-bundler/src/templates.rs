//! `index.html` template for deck projects.
//!
//! The dev doesn't author `index.html` — deck generates it. This template
//! is the SSOT for the dev-facing SPA entry shape. The bundler renders one
//! `<project>/dist/index.html` per project by substituting:
//!   * `{{project_name}}`     — `package.json#name`
//!   * `{{importmap}}`         — the generated importmap JSON
//!   * `{{entry_module}}`      — the generated `routes.gen.ts` (or whatever
//!                                the bundler decides to emit as the entry)
//!
//! The template is intentionally minimal. The dev's project adds static
//! assets via `apps/<x>/public/` (copied verbatim to `<x>/dist/public/`).
//! Anything fancier (theme pre-paint, font preloading, deck-mode metadata)
//! is a separate concern — the template can grow as needed.
//!
//! The importmap MUST come before any module script: browsers freeze the
//! importmap at first module fetch, so it lives at the top of `<head>` to
//! win the race against `routes.gen.ts` and any per-route bundles.

/// The DOM id of the SPA mount point. Single source for the `index.html`
/// `<div id>` AND the `mountRouter({ root })` call the route emitter writes
/// (`routes.rs`) — change it here and both move together, no hardcoded
/// literal drifting on either side.
pub const ROOT_ID: &str = "root";

/// Pre-paint FOUC guard body — owned by `@ship/tokens`
/// (`packages/sail/runtime/src/prepaint-theme.js` / `PREPAINT_THEME_SCRIPT`).
/// Deck pastes; does **not** hand-write storage keys or seam attr names.
const PREPAINT_THEME_SCRIPT: &str =
    include_str!("../../../../packages/sail/runtime/src/prepaint-theme.js");

/// The minimal `index.html` template for a deck project.
///
/// `{{prepaint_theme_script}}` is replaced with the tokens-owned
/// `PREPAINT_THEME_SCRIPT` body. Other placeholders are project-specific.
pub const INDEX_HTML_TEMPLATE: &str = r#"<!doctype html>
<html lang="{{lang}}">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{{title}}</title>
{{description_meta}}

  <!-- Favicon. A project drops `favicon.svg` (preferred) and/or `favicon.ico`
       into `public/`; both are copied to the output root, and an absent file
       simply 404s harmlessly. -->
  <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
  <link rel="icon" href="/favicon.ico" sizes="any" />

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

  <!-- Pre-paint theme + mode (FOUC guard). Body owned by @ship/tokens
       (bootTheme / PREPAINT_THEME_SCRIPT) — deck pastes, never encodes
       storage keys or theme|data-mode seam knowledge. -->
  <script>
{{prepaint_theme_script}}
  </script>
</head>
<body>
  <div id="{{root_id}}"></div>
  <script type="module" src="/{{entry_module}}"></script>
</body>
</html>
"#;

/// SPA shell title / lang / description from `sail.config` `app`.
/// `RouteShape.head` is an optional table apply slot on navigate — not a `<scope>` member.
#[derive(Debug, Clone)]
pub struct SiteDocument {
    pub title: String,
    pub description: Option<String>,
    pub lang: String,
}

impl SiteDocument {
    pub fn from_project(project: &std::path::Path, package_name: &str) -> Self {
        let mut site = Self {
            title: package_name.to_string(),
            description: None,
            lang: "en".into(),
        };
        let Some((_, cfg)) = sail_lint::config::load_in(project) else {
            return site;
        };
        if !cfg.parse_ok() || cfg.type_error().is_some() {
            return site;
        }
        let Some(s) = &cfg.app else {
            return site;
        };
        if let Some(t) = s.title.as_deref().filter(|t| !t.is_empty()) {
            site.title = t.to_string();
        }
        if let Some(d) = s.description.as_deref().filter(|d| !d.is_empty()) {
            site.description = Some(d.to_string());
        }
        if let Some(l) = s.lang.as_deref().filter(|l| !l.is_empty()) {
            site.lang = l.to_string();
        }
        site
    }
}

/// Render the template with substitutions.
///
/// `importmap_json` must be a pre-serialized JSON object (i.e. the
/// `importmap` field's value as a JSON string with two-space indent).
/// `entry_module` is the file name (no path) of the generated entry —
/// `routes.gen.ts` is the current convention.
pub fn render_index_html(project_name: &str, importmap_json: &str, entry_module: &str) -> String {
    render_index_html_with_site(
        importmap_json,
        entry_module,
        &SiteDocument {
            title: project_name.to_string(),
            description: None,
            lang: "en".into(),
        },
    )
}

pub fn render_index_html_with_site(
    importmap_json: &str,
    entry_module: &str,
    site: &SiteDocument,
) -> String {
    let description_meta = site
        .description
        .as_deref()
        .map(|d| {
            format!(
                "  <meta name=\"description\" content=\"{}\" />\n",
                html_attr(d)
            )
        })
        .unwrap_or_default();
    INDEX_HTML_TEMPLATE
        .replace("{{root_id}}", ROOT_ID)
        .replace("{{title}}", &html_attr(&site.title))
        .replace("{{lang}}", &html_attr(&site.lang))
        .replace("{{description_meta}}", &description_meta)
        .replace("{{importmap}}", importmap_json)
        .replace("{{entry_module}}", entry_module)
        .replace(
            "{{prepaint_theme_script}}",
            PREPAINT_THEME_SCRIPT.trim_end(),
        )
}

fn html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepaint_script_comes_from_tokens_not_hardcoded_keys_in_template() {
        // Template must not embed storage keys / seam attrs itself — only the
        // placeholder. The pasted body carries ship-site-theme from tokens.
        assert!(
            !INDEX_HTML_TEMPLATE.contains("ship-site-theme"),
            "template must paste tokens prepaint, not hardcode storage keys"
        );
        assert!(INDEX_HTML_TEMPLATE.contains("{{prepaint_theme_script}}"));
        let html = render_index_html("demo", "{}", "entry.gen.ts");
        assert!(html.contains("<title>demo</title>"));
        assert!(html.contains("lang=\"en\""));
        assert!(
            html.contains("ship-site-theme"),
            "rendered HTML must include tokens prepaint body"
        );
        assert!(
            html.contains("setAttribute('theme'"),
            "prepaint must stamp theme attr: {html}"
        );
        assert!(
            html.contains("data-mode"),
            "prepaint must stamp data-mode: {html}"
        );
        assert!(
            html.contains("sail-default-theme"),
            "prepaint must read config default meta: {html}"
        );
    }

    #[test]
    fn site_defaults_land_in_shell() {
        let html = render_index_html_with_site(
            "{}",
            "entry.gen.ts",
            &SiteDocument {
                title: "Sail Docs".into(),
                description: Some("Install Sail.".into()),
                lang: "en".into(),
            },
        );
        assert!(html.contains("<title>Sail Docs</title>"));
        assert!(html.contains("name=\"description\""));
        assert!(html.contains("content=\"Install Sail.\""));
        assert!(html.contains("lang=\"en\""));
    }
}
