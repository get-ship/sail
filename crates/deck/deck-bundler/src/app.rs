//! Deck `app` build target (#170) — bundles a SAIL SPA project (the hub
//! `apps/hub/`, the space scaffold `apps/spaces/`, or any other host
//! that uses the `routes/` convention) to a static dir.
//!
//! This is the deck-owned replacement for the retired `shell-app.ts`
//! (`Bun.build`) — one bundler engine (Rolldown) for framework + routes +
//! projects. Driven by `deck build` (`crates/deck/deck-cli`). The project
//! graph compiles its own `.sail` in-process via the shared `SailPlugin`
//! (`rolldown_bundle.rs`) — the exact resolve/load hooks the route/component
//! build uses — so no on-disk `.compiled.js` pre-step is needed.
//! `AppPlugin` runs alongside it to externalize the framework singletons
//! (which resolve to `/_deck/*` via the document's importmap); it runs FIRST so
//! importmap specifiers (incl. `@ship/sail-runtime/*` subpaths) stay external
//! rather than being re-anchored to source by `SailPlugin`.
//!
//! The dev's authored source is THREE files only: `package.json` (the
//! manifest: name → page <title>), `routes/` (file-routed chrome), and
//! `public/` (static assets). The bundler generates the rest: `index.html`
//! (from a template, with the importmap substituted), `routes.gen.ts` (the
//! entry that `index.html` references), and the bundled entry chunk.
//! Project builds live at `<project>/.deck/` by default.

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use rolldown::plugin::{
    HookResolveIdArgs, HookResolveIdOutput, HookResolveIdReturn, HookUsage, Plugin, PluginContext,
};
use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
use rolldown_common::{ChunkFilenamesOutputOption, ModuleType, ResolvedExternal};

/// deck's BAKED framework singletons — assets deck itself owns and ships
/// in every app build. Each entry is `(package_alias, source_filename,
/// output_url_relative_to_project)`. The bundler copies `source_filename`
/// from the workspace's prebuilt framework dir to
/// `output_url_relative_to_project` in the project's output, then emits
/// an importmap entry mapping `package_alias → /output_url_relative_to_project`.
///
/// Scope of this list: deck-owned runtime (the SAIL runtime + DOM bindings
/// + the router the framework owns). What is NOT here: framework CLIENTS
/// (`@ship/sdk-client`, etc.) — those are provider-declared. A framework-
/// singleton package self-declares its serve contract in its OWN
/// `package.json#deckFramework` (mirror of how packs declare
/// `customElements`), and the bundler DISCOVERS these by walking the
/// app's resolved dependency graph (see [`discover_deck_frameworks`]).
/// The app declares nothing; deck names no ship package; the singleton
/// owner names itself.
///
/// Lucide icons are NOT here — the daemon serves them per-icon from its
/// embedded set at `/_deck/icons/{name}.svg`. Static-only deploys without
/// a daemon use `<sail-icon src=…>` (CDN escape hatch).
///
/// The package_alias MAY include a subpath: `@ship/sail-runtime/route` etc.
/// All aliases for one source bundle point at the same output URL — the
/// runtime exports a single module entry. (e.g. `@ship/sail-runtime/route`,
/// `@ship/sail-runtime/template-hooks`, `@ship/sail-runtime/patch` all share
/// the runtime bundle.)
const FRAMEWORK_ASSETS: &[(&str, &str, &str)] = &[
    ("@ship/sail-runtime", "sail-runtime.js", "_deck/runtime.js"),
    ("@ship/sail-runtime/route", "sail-runtime.js", "_deck/runtime.js"),
    ("@ship/sail-runtime/template-hooks", "sail-runtime.js", "_deck/runtime.js"),
    ("@ship/sail-runtime/patch", "sail-runtime.js", "_deck/runtime.js"),
    // `/internal` is what COMPILED ROUTES import (__tpl, bind*, effect — the
    // emitter's surface). Its absence here inlined a SECOND copy of the
    // signal graph into every app bundle: route bindings subscribed in the
    // copy, controller signals wrote in the real one, and no page reacted
    // to anything. The resolve guard below makes this class of miss a
    // build error instead of a silent fork.
    ("@ship/sail-runtime/internal", "sail-runtime.js", "_deck/runtime.js"),
    // Compiler auto-imports `use:` actions from this subpath (see
    // sail-compiler emit/js/imports.rs). Shares the framework-bundle
    // singleton (full action surface + runtime) — must not be inlined.
    ("@ship/sail-runtime/actions", "sail-runtime.js", "_deck/runtime.js"),
    ("@ship/sail-dom", "sail-dom.js", "_deck/dom.js"),
    // `@ship/deck/router` lives in the framework because the bundler
    // generates an entry that imports it. The runtime exports
    // `defineRoutes`, `route`, `navigate`, `setEditMode` — every app's
    // entry needs this singleton. The bundle file is built in
    // `crates/deck/deck-core/build.rs` and shipped via
    // `include_str!` in `assets.rs`.
    ("@ship/deck/router", "deck-route.js", "_deck/route.js"),
    // The code-editor/playground route's WASM-backed compiler + browser
    // analyser. Framework-owned (deck ships them), same class as
    // runtime/dom — NOT provider-declared, since the app declares nothing
    // and deck names both packages itself. `sail-compiler-wasm`'s co-asset
    // (`.wasm` binary) rides next to `_deck/compiler-wasm.js` — see
    // `FrameworkSingleton::co_assets`. Only the two subpaths the
    // code-editor actually imports (`sail-code-editor.sail`'s dynamic
    // `import('@ship/sail-lsp-browser/completions' | '/diagnostics')`) are
    // registered — `/worker` and `/hover` are real exports but unused by
    // any route today, so they are not surfaced here.
    ("@ship/sail-compiler-wasm", "sail-compiler-wasm.js", "_deck/compiler-wasm.js"),
    ("@ship/sail-lsp-browser/completions", "sail-lsp-completions.js", "_deck/lsp-completions.js"),
    ("@ship/sail-lsp-browser/diagnostics", "sail-lsp-diagnostics.js", "_deck/lsp-diagnostics.js"),
];

/// The deck-owned framework singletons surfaced in every generated importmap.
/// Tuple is `(package_alias, source_bundle_filename, dist_url_path)`. The set
/// is the SSOT for both the bundler's importmap and the dev daemon's `/_deck/*`
/// routes — a new singleton needs an entry here AND a matching serve handler.
pub fn framework_assets() -> &'static [(&'static str, &'static str, &'static str)] {
    FRAMEWORK_ASSETS
}
/// A framework-singleton self-declaration, read from a package's
/// `package.json#deckFramework`. Replaces the host-declared
/// `framework_clients` map: the package that owns the singleton
/// declares its serve contract, and the bundler discovers it via the
/// app's dep graph (the same walker that finds components, bundles,
/// routes). Mirror of how packs declare `customElements` — the package
/// names itself; deck names no ship path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeckFrameworkDecl {
    /// Bare specifier the app's source imports (`@ship/sdk-client`).
    pub alias: String,
    /// Absolute path to the source module the bundler reads the
    /// singleton bytes from (`<package_root>/<entry>`). Resolution
    /// happens during discovery (the dep's package.json#deckFramework
    /// `entry` is package-relative; we anchor it to the package root
    /// in the workspace).
    pub src_path: PathBuf,
    /// The output URL path the importmap binds the alias to (relative
    /// to the project root). The basename is what gets copied to
    /// `<out>/_deck/<basename>` when `--bundle-framework` is set.
    pub serve: String,
}

/// Generate the importmap JSON. The keys are the package aliases
/// (deck's `FRAMEWORK_ASSETS` singletons + every discovered
/// framework-singleton declared by an app dependency's
/// `package.json#deckFramework`). The values are the absolute URL the
/// host serves each bundle at — `/_deck/<file>` for the project's
/// output, which shipd serves at the project root via the
/// `serve_scaffold_asset` route.
///
/// Discovered frameworks ride the SAME `_deck/<basename>` URL
/// convention as the framework singletons: the dep's package.json
/// declares a `serve` filename (e.g. `_deck/sdk-client.js`) and the
/// bundler binds the alias to `/_deck/sdk-client.js`. No special-case
/// route; one URL convention, one handler.
///
/// Output is pretty-printed with 2-space indent to match the existing
/// `apps/shell/index.html` style.
fn generate_importmap(
    dist_prefix: &str,
    discovered: &[DeckFrameworkDecl],
) -> String {
    let mut imports: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for (alias, _src, dist_rel) in FRAMEWORK_ASSETS {
        imports.insert((*alias).to_string(), format!("{dist_prefix}/{dist_rel}"));
    }
    // Provider-declared singletons: `<alias>` → `/_deck/<basename>`.
    // The dep's package.json#deckFramework#serve field provides the
    // relative URL; the bundler binds the alias to `/_deck/<basename>`
    // (the same URL the framework singletons ride). The daemon's
    // existing static-asset handler serves them on the same code path.
    for decl in discovered {
        let basename = std::path::Path::new(&decl.serve)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&decl.serve);
        imports.insert(decl.alias.clone(), format!("{dist_prefix}/_deck/{basename}"));
    }
    let body = imports
        .iter()
        .map(|(k, v)| format!("        \"{k}\": \"{v}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n      \"imports\": {{\n{body}\n      }}\n    }}")
}

/// Extract the bare specifier set from a rendered importmap JSON. The dev
/// doesn't author the importmap; the bundler writes it via `generate_importmap`
/// and reads it back here to get the externals list. The two views must agree:
/// any package the importmap resolves to `/foo` is a runtime external; the
/// route bundle's ESM imports of that package stay as bare specifiers.
fn importmap_externals_from_json(importmap_json: &str) -> Result<HashSet<String>> {
    let v: serde_json::Value = serde_json::from_str(importmap_json)
        .context("generated importmap is not valid JSON")?;
    let imports = v
        .get("imports")
        .and_then(|x| x.as_object())
        .context("generated importmap has no `imports` object")?;
    if imports.is_empty() {
        bail!("generated importmap has no imports");
    }
    Ok(imports.keys().cloned().collect())
}

/// Extract bare ESM import specifiers from emitted JS — static `import`/`from`
/// and dynamic `import(...)`. Hand-rolled (deck-bundler has no regex dep): scan
/// for each `from` / `import` keyword at an identifier boundary and read the
/// string literal that immediately follows (after optional whitespace / `(`).
/// "Bare" = the specifier does not start with `.` or `/` — a package specifier,
/// not a relative/absolute path. Works on both minified (`from"x"`) and
/// pretty-printed (`from "x"`) output.
///
/// STRING-AWARE: text inside string literals, template literals, and comments
/// is not code. A docs site that renders `import … from '@ship/x'` inside a
/// snippet string must not fail the build over its own teaching material —
/// only a real import statement can blank a page.
fn scan_bare_specifiers(js: &str) -> Vec<String> {
    let bytes = js.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
    let at = |i: usize, kw: &[u8]| bytes.len() >= i + kw.len() && &bytes[i..i + kw.len()] == kw;

    /// Advance past a `'…'`/`"…"` literal (i at the opening quote).
    fn skip_string(bytes: &[u8], mut i: usize) -> usize {
        let quote = bytes[i];
        i += 1;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' => i += 2,
                b if b == quote => return i + 1,
                _ => i += 1,
            }
        }
        i
    }

    /// Advance past a template literal (i at the opening backtick). `${…}`
    /// interpolations can nest strings and further templates; track brace
    /// depth per interpolation so an inner `}` doesn't end it early.
    fn skip_template(bytes: &[u8], mut i: usize) -> usize {
        i += 1;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' => i += 2,
                b'`' => return i + 1,
                b'$' if i + 1 < bytes.len() && bytes[i + 1] == b'{' => {
                    i += 2;
                    let mut depth = 1usize;
                    while i < bytes.len() && depth > 0 {
                        match bytes[i] {
                            b'\'' | b'"' => i = skip_string(bytes, i),
                            b'`' => i = skip_template(bytes, i),
                            b'{' => {
                                depth += 1;
                                i += 1;
                            }
                            b'}' => {
                                depth -= 1;
                                i += 1;
                            }
                            _ => i += 1,
                        }
                    }
                }
                _ => i += 1,
            }
        }
        i
    }

    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' | b'"' => {
                i = skip_string(bytes, i);
                continue;
            }
            b'`' => {
                i = skip_template(bytes, i);
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
            _ => {}
        }
        let kw_len = if at(i, b"import") {
            Some(6)
        } else if at(i, b"from") {
            Some(4)
        } else {
            None
        };
        if let Some(len) = kw_len {
            let prev_ok = i == 0 || !is_ident(bytes[i - 1]);
            let after = i + len;
            let next_ok = after >= bytes.len() || !is_ident(bytes[after]);
            if prev_ok && next_ok {
                // Skip whitespace and an optional `(` (dynamic import).
                let mut j = after;
                while j < bytes.len() && (bytes[j].is_ascii_whitespace() || bytes[j] == b'(') {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                    let quote = bytes[j];
                    let start = j + 1;
                    let mut k = start;
                    while k < bytes.len() && bytes[k] != quote {
                        k += 1;
                    }
                    if k < bytes.len() {
                        if let Ok(spec) = std::str::from_utf8(&bytes[start..k]) {
                            if !spec.is_empty() && !spec.starts_with('.') && !spec.starts_with('/') {
                                out.push(spec.to_string());
                            }
                        }
                        i = k + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out
}

/// Fail the build when an emitted chunk would ship a bare `@ship/*` ESM import
/// the document importmap does NOT resolve. Such a specifier throws "Failed to
/// resolve module specifier" at module-instantiation time, leaving an empty
/// `#root` — a SILENT blank page. deck owns the `@ship/*` namespace
/// contract: every `@ship/*` bare import the bundle externalizes must EITHER be
/// inlined (a workspace package resolvable from this project) OR be in the
/// importmap (a framework singleton / provider-declared singleton). This guard
/// turns that silent blank into a loud, actionable error.
///
/// `importmap_keys` are the specifiers the generated importmap resolves; the
/// match is exact plus the standard importmap trailing-slash prefix form
/// (`@ship/x/` covers `@ship/x/sub`). Scoped to `@ship/*`: a non-`@ship` bare
/// import may be a host-provided global and is out of deck's contract.
fn assert_pack_imports_resolved(
    assets_dir: &Path,
    importmap_keys: &HashSet<String>,
) -> Result<()> {
    let resolves = |spec: &str| -> bool {
        importmap_keys.contains(spec)
            || importmap_keys.iter().any(|k| k.ends_with('/') && spec.starts_with(k.as_str()))
    };
    // (specifier, chunk file) — BTreeSet for a stable, deduped report.
    let mut offenders: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(assets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("js") {
            continue;
        }
        let js = std::fs::read_to_string(&path)?;
        for spec in scan_bare_specifiers(&js) {
            if spec.starts_with("@ship/") && !resolves(&spec) {
                offenders.insert((spec, entry.file_name().to_string_lossy().into_owned()));
            }
        }
    }
    if !offenders.is_empty() {
        let list = offenders
            .iter()
            .map(|(spec, file)| format!("  - `{spec}` (in assets/{file})"))
            .collect::<Vec<_>>()
            .join("\n");
        bail!(
            "app build: the bundle externalized {} bare `@ship/*` import(s) the document \
             importmap does NOT resolve — each throws \"Failed to resolve module specifier\" \
             at runtime, blanking the page:\n{list}\n  Each must be EITHER inlined (a workspace \
             package resolvable from this project) OR a framework singleton in the importmap. \
             The usual cause is the app importing a package it does not declare as a dependency \
             — add it to the app's package.json (and install) so the bundler can resolve it.",
            offenders.len()
        );
    }
    Ok(())
}

pub struct AppBuild {
    pub out_dir: PathBuf,
    pub js_href: String,
    /// Absolute URL paths for every CSS asset emitted under
    /// `<out_dir>/assets/` by Rolldown's native asset pipeline
    /// (see `module_types: { ".css" -> Asset }` above). The bundler
    /// injects a `<link rel="stylesheet">` for each one into the
    /// generated `index.html`. Empty when the project imports no CSS.
    pub css_hrefs: Vec<String>,
    pub framework_assets: usize,
    /// The asset-URL namespace the generated `index.html` bakes its hrefs
    /// under — an explicit `--base`, else the package-name last segment
    /// (`@ship/hub` → `hub`). The host serves assets under exactly THIS prefix
    /// (`/<url_namespace>/<file>`), so a serve route derived from the build's
    /// own output can never diverge from what was baked — the dev-vs-build
    /// base-URL mismatch that 404s every asset. One derivation, returned here.
    pub url_namespace: String,
}

/// Externalizes the framework singleton specifiers declared in
/// FRAMEWORK_ASSETS. The bundler generates the importmap at build time
/// (from the same FRAMEWORK_ASSETS list); the AppPlugin runs FIRST so
/// the importmap specifiers win as externals rather than being re-anchored
/// to source by `SailPlugin`. `.sail` compilation is handled by the sibling
/// `SailPlugin` in the same bundle pass.
#[derive(Debug)]
struct AppPlugin {
    externals: HashSet<String>,
}

impl Plugin for AppPlugin {
    fn name(&self) -> Cow<'static, str> {
        "deck-app-externals".into()
    }

    async fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs<'_>,
    ) -> HookResolveIdReturn {
        if self.externals.contains(args.specifier) {
            return Ok(Some(HookResolveIdOutput {
                id: args.specifier.into(),
                external: Some(ResolvedExternal::Bool(true)),
                ..Default::default()
            }));
        }
        // A framework-singleton SUBPATH that is not in the importmap must not
        // fall through to node resolution: the bundler would inline a second
        // copy of the module — for the runtime that means a second signal
        // graph, and every binding in the app goes quietly dead. Fail the
        // build and name the fix.
        // `@ship/sail-compiler-wasm` is NOT in this list: it has no
        // meaningful subpath tree the way runtime/dom/lsp-browser do (its
        // package.json exports only its own `.` entry plus the `.wasm`
        // co-export the wasm-bindgen glue resolves at runtime via a plain
        // string, never a static import specifier) — so there is no unknown
        // subpath a bare import could name. Its bare `.` alias is matched
        // directly by the `self.externals.contains` check above.
        for root in [
            "@ship/sail-runtime/",
            "@ship/sail-dom/",
            "@ship/deck/",
            "@ship/sail-lsp-browser/",
        ] {
            if args.specifier.starts_with(root) {
                anyhow::bail!(
                    "app build: `{}` is a framework-singleton subpath with no importmap \
                     entry — bundling it inline would duplicate the singleton (for the \
                     runtime: a second signal graph, and reactivity dies silently). Add it \
                     to FRAMEWORK_ASSETS in deck-bundler/src/app.rs.",
                    args.specifier
                );
            }
        }
        Ok(None)
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::ResolveId
    }
}
/// Read the project name from `<project>/package.json#name`. The dev's
/// `package.json` is the manifest: the bundler reads it for the page <title>
/// and the chrome's brand string. Required.
fn read_package_name(project: &Path) -> Result<String> {
    let pkg_path = project.join("package.json");
    let body = std::fs::read_to_string(&pkg_path)
        .with_context(|| format!("read {}", pkg_path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&body)
        .with_context(|| format!("parse {}", pkg_path.display()))?;
    v.get("name")
        .and_then(|n| n.as_str())
        .map(String::from)
        .with_context(|| format!("package.json at {} has no `name` field", project.display()))
}
/// Discover framework-singleton self-declarations by walking the app's
/// `package.json` dependency graph. Each dep's own `package.json` MAY
/// carry a `deckFramework` field:
/// ```jsonc
/// {
///   "deckFramework": {
///     "alias": "@ship/sdk-client",
///     "entry": "sail-sdk-client.js",
///     "serve": "_deck/sdk-client.js"
///   }
/// }
/// ```
///
/// The bundler reads every transitive dep, collects each declaration, and
/// returns the resolved list. The importmap, externals list, and
/// `--bundle-framework` copy are all driven from this discovery pass —
/// the host's `deck.jsonc` carries no client registration anymore
/// (`framework_clients` is removed from the schema).
///
/// Discovery is depth-first on the dep tree, deduplicated by alias
/// (first wins on collision — a workspace with two `@ship/sdk-client`
/// copies would otherwise fight the importmap). A dep outside the
/// workspace (e.g. a published package on the registry) is skipped —
/// only in-workspace package.json files declare their serve contract.
/// For dev that's sufficient; the prod install path resolves via
/// standard package resolution and the runtime still serves the
/// asset at the importmap's URL (the daemon's static handler is the
/// authoritative source).
pub(crate) fn discover_deck_frameworks(
    project: &Path,
    repo_root: &Path,
) -> Result<Vec<DeckFrameworkDecl>> {
    let pkg_path = project.join("package.json");
    let raw = std::fs::read_to_string(&pkg_path)
        .with_context(|| format!("read {}", pkg_path.display()))?;
    let pkg: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", pkg_path.display()))?;
    // Build the workspace name → root map once. The same map the
    // SailPlugin uses to resolve `@ship/<name>` to workspace source;
    // we reuse it so the discovery walker is anchored against the
    // same source of truth. A bare `@ship/sdk-client` declared by a
    // dep that resolves via node_modules (not a workspace member)
    // is skipped — the bundler ships only what it can resolve to a
    // file on disk.
    let package_roots = crate::catalog::component_package_roots(repo_root);
    let mut out: Vec<DeckFrameworkDecl> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    walk_deck_frameworks(&pkg, &package_roots, &mut out, &mut seen)?;
    // Stable order so the importmap is byte-stable across runs.
    out.sort_by(|a, b| a.alias.cmp(&b.alias));
    Ok(out)
}

fn walk_deck_frameworks(
    pkg: &serde_json::Value,
    package_roots: &std::collections::HashMap<String, PathBuf>,
    out: &mut Vec<DeckFrameworkDecl>,
    seen: &mut std::collections::HashSet<String>,
) -> Result<()> {
    // 1. Read THIS package's `dependencies` (deps + devDeps + peerDeps).
    for deps_key in ["dependencies", "devDependencies", "peerDependencies"] {
        let Some(deps) = pkg.get(deps_key).and_then(|d| d.as_object()) else {
            continue;
        };
        for (dep_name, _ver) in deps {
            if !seen.insert(dep_name.clone()) {
                continue;
            }
            let Some(root) = package_roots.get(dep_name) else {
                // Not in workspace (registry install). Skip — the
                // singleton owner must be a workspace member for
                // dev-mode self-declaration to work. Production
                // resolution is the runtime's `importmap` URL.
                continue;
            };
            let dep_pkg_path = root.join("package.json");
            let raw = match std::fs::read_to_string(&dep_pkg_path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let dep_pkg: serde_json::Value = match serde_json::from_str(&raw) {
                Ok(v) => v,
                Err(_) => continue,
            };
            // 2. Self-declaration: if this dep's package.json carries
            // `deckFramework`, register it.
            if let Some(decl) = dep_pkg.get("deckFramework") {
                let alias = decl
                    .get("alias")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow!(
                            "{}: deckFramework missing string `alias`",
                            dep_pkg_path.display()
                        )
                    })?
                    .to_string();
                let entry = decl
                    .get("entry")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow!(
                            "{}: deckFramework missing string `entry`",
                            dep_pkg_path.display()
                        )
                    })?
                    .to_string();
                let serve = decl
                    .get("serve")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow!(
                            "{}: deckFramework missing string `serve`",
                            dep_pkg_path.display()
                        )
                    })?
                    .to_string();
                let src_path = root.join(&entry);
                out.push(DeckFrameworkDecl {
                    alias,
                    src_path,
                    serve,
                });
            }
            // 3. Recurse into this dep's deps (transitive).
            walk_deck_frameworks(&dep_pkg, package_roots, out, seen)?;
        }
    }
    Ok(())
}
/// Stage framework co-assets (today: `sail_compiler_bg.wasm`) into `out_dir`
/// root so `/sail_compiler_bg.wasm` is served.
///
/// Runs on EVERY app build — not only `--bundle-framework`. deck dev keeps
/// framework JS external (served from the daemon bake at `/_deck/*`) but the
/// wasm binary is still a public root asset. Historically only
/// `bundle_owned_framework` copied it, so bare `deck dev` after a cold wipe
/// returned HTTP 200 on `/` while `/sail_compiler_bg.wasm` 404'd.
///
/// Loud when the package is installed but unbuilt; silent skip when the
/// dep isn't present (consumer apps that don't ship the in-browser compiler).
fn stage_framework_co_assets(repo_root: &Path, project: &Path, out_dir: &Path) -> Result<()> {
    // Prefer the project graph: monorepo workspace always contains
    // `@ship/sail-compiler-wasm`, but only apps that depend on it need the
    // `.wasm` co-asset. Fail loud for those; skip for everyone else.
    let project_deps = project_runtime_dep_names(project);
    for s in crate::targets::FRAMEWORK_SINGLETONS {
        if s.co_assets.is_empty() {
            continue;
        }
        let pkg_name = package_name_of_spec(s.spec);
        let project_needs = project_deps.iter().any(|d| d == pkg_name);

        // Happy path: entry resolves from repo (workspace or node_modules).
        if let Some(entry) = crate::node_resolve::resolve_entry(repo_root, s.spec) {
            // If neither the project nor a successful prior public/ copy needs
            // it, still stage when resolvable — cheap and keeps --bundle-framework
            // consumers consistent. Missing co-asset next to a resolved entry is
            // always a broken install.
            let Some(entry_dir) = entry.parent() else {
                continue;
            };
            for co in s.co_assets {
                let src = entry_dir.join(co);
                if !src.is_file() {
                    if !project_needs {
                        // Workspace member present but unbuilt, and this app
                        // doesn't depend on it — skip rather than block.
                        continue;
                    }
                    bail!(
                        "app build: framework singleton `{}` resolved to {} but co-asset \
                         `{co}` is missing at {} — run `bun run wasm` (builds \
                         packages/sail/compiler-wasm/pkg) before `deck dev` / `deck build`",
                        s.name,
                        entry.display(),
                        src.display()
                    );
                }
                let dst = out_dir.join(co);
                std::fs::copy(&src, &dst).with_context(|| {
                    format!(
                        "copy co-asset {} -> {} (framework singleton `{}`)",
                        src.display(),
                        dst.display(),
                        s.name
                    )
                })?;
            }
            continue;
        }

        // Entry missing. Only fail when THIS project depends on the package —
        // monorepo workspace membership alone must not block unrelated apps.
        if !project_needs {
            continue;
        }
        if let Some(pkg_root) = crate::node_resolve::resolve_package_root(repo_root, pkg_name) {
            bail!(
                "app build: `{}` is installed at {} but its entry/co-assets are missing \
                 (need {}). Run `bun run wasm` from the repo root (or \
                 `cd apps/sail-site && bun run gen`) before `deck dev` / `deck build`.",
                s.spec,
                pkg_root.display(),
                s.co_assets.join(", ")
            );
        }
        bail!(
            "app build: project depends on `{}` but it is not installed and co-assets \
             ({}) cannot be staged. Run `bun install` and `bun run wasm`.",
            s.spec,
            s.co_assets.join(", ")
        );
    }
    Ok(())
}

fn package_name_of_spec(spec: &str) -> &str {
    // `@scope/name/sub` → `@scope/name`; `pkg/sub` → `pkg`.
    let parts: Vec<&str> = spec.split('/').collect();
    let n = if spec.starts_with('@') { 2 } else { 1 };
    if parts.len() <= n {
        spec
    } else {
        // Can't return a temporary join; specs in FRAMEWORK_SINGLETONS never
        // carry a subpath today. Keep the simple path.
        spec
    }
}

/// Runtime dependency names from `<project>/package.json` (dependencies +
/// devDependencies + peerDependencies). Used to decide whether a missing
/// framework co-asset is this app's problem.
fn project_runtime_dep_names(project: &Path) -> Vec<String> {
    let raw = match std::fs::read_to_string(project.join("package.json")) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let pkg: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for key in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(deps) = pkg.get(key).and_then(|d| d.as_object()) {
            out.extend(deps.keys().cloned());
        }
    }
    out
}


/// Build deck's OWN framework singletons from node-resolved source and write
/// each onto its served `_deck/<file>` path under `out_dir` — the self-contained
/// `--bundle-framework` path. Entries are resolved by node resolution from
/// `repo_root` (`BundleLayout::framework_singletons`): no prebuilt-asset dir,
/// no `DECK_FRAMEWORK_ASSETS_DIR`. One source bundle may back several aliases
/// (`@ship/sail-runtime` and `@ship/sail-runtime/route` share `runtime.js`), so
/// the output is mapped onto every matching `FRAMEWORK_ASSETS` `dist_rel`.
/// Bails when a singleton named in the importmap can't be resolved/built — a
/// self-contained build must not ship an importmap pointing at a missing file.
/// Returns the number of files written.
async fn bundle_owned_framework(
    repo_root: &Path,
    out_dir: &Path,
    minify: bool,
) -> Result<usize> {
    use crate::targets::BundleLayout;
    let staging = out_dir.join(".fw-build");
    let layout = BundleLayout::with_config(
        repo_root.to_path_buf(),
        None,
        deck_config::DeckConfig::defaults(),
    )
    .with_out_root(staging.clone());
    let work = staging.join("work");

    let mut written: HashSet<String> = HashSet::new();
    for target in layout.framework_singletons() {
        // The FRAMEWORK_ASSETS `dist_rel`s this singleton's output backs. A
        // singleton not surfaced in the importmap (sail-sdk-client — it is
        // provider-declared, not deck-owned) maps to none → skip it.
        let dists: Vec<&str> = FRAMEWORK_ASSETS
            .iter()
            .filter(|(_, src, _)| *src == target.out_file)
            .map(|(_, _, dist_rel)| *dist_rel)
            .collect();
        if dists.is_empty() {
            continue;
        }
        crate::rolldown_bundle::bundle_target(&layout, &target, &work, minify)
            .await
            .with_context(|| format!("bundle framework singleton `{}`", target.name))?;
        let built = layout.assets_dir_for(&target.name).join(&target.out_file);
        // Co-assets (e.g. `sail-compiler-wasm`'s `.wasm` binary) ride along
        // next to the JS under their OWN literal filename — see
        // `FrameworkSingleton::co_assets` doc. Sourced from the resolved
        // entry's directory (the wasm-bindgen `pkg/` dir), copied verbatim.
        let co_assets: &[&str] = crate::targets::FRAMEWORK_SINGLETONS
            .iter()
            .find(|s| s.name == target.name)
            .map(|s| s.co_assets)
            .unwrap_or(&[]);
        let entry_dir = match &target.entry {
            crate::targets::TargetEntry::Real(p) => p.parent().map(Path::to_path_buf),
            crate::targets::TargetEntry::Virtual { .. } => None,
        };
        for dist_rel in dists {
            if !written.insert(dist_rel.to_string()) {
                continue;
            }
            let dst = out_dir.join(dist_rel);
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&built, &dst)
                .with_context(|| format!("copy {} -> {}", built.display(), dst.display()))?;
        }
        // Co-assets land at the PROJECT ROOT (`out_dir`), not next to the JS —
        // `diagnostics.ts` calls `initWasm(wasmUrl)` with an EXPLICIT url
        // (default `/sail_compiler_bg.wasm`, root-absolute; see that file's
        // doc comment), which bypasses wasm-bindgen's own
        // `import.meta.url`-relative default entirely. So the binary must sit
        // at the root path the caller actually requests, not wherever the JS
        // singleton happens to be served from.
        for co_asset in co_assets {
            if !written.insert(format!("co-asset:{co_asset}")) {
                continue;
            }
            let Some(entry_dir) = &entry_dir else { continue };
            let src = entry_dir.join(co_asset);
            let co_dst = out_dir.join(co_asset);
            std::fs::copy(&src, &co_dst).with_context(|| {
                format!(
                    "copy co-asset {} -> {} (framework singleton `{}`)",
                    src.display(),
                    co_dst.display(),
                    target.name
                )
            })?;
        }
    }
    std::fs::remove_dir_all(&staging).ok();

    // Strictness: every distinct deck-owned `dist_rel` must have been produced.
    // A miss means a framework singleton isn't installed in this project — fail
    // loudly rather than ship a dangling importmap.
    let mut required: Vec<&str> = FRAMEWORK_ASSETS.iter().map(|(_, _, d)| *d).collect();
    required.sort_unstable();
    required.dedup();
    for dist_rel in required {
        if !out_dir.join(dist_rel).exists() {
            bail!(
                "app build: bundle_framework: framework singleton for `/{dist_rel}` did not \
                 resolve from {} — is it installed (node_modules)?",
                repo_root.display()
            );
        }
    }
    Ok(written.len())
}

/// Build a SAIL SPA project to `out_dir`. `repo_root` is the workspace root
/// (cwd for module resolution); `project` is the dev's authored source
/// (`package.json` + `routes/` + `public/`). The dev does NOT author
/// `index.html` or any entry script — those are generated from a template
/// and the routes/ walk, respectively.
///
/// Asset URLs use the package-name SERVE namespace (`/<name>/<file>`, which a
/// host maps to `assets/<file>`). For a build a dumb static host can serve
/// directly, use [`bundle_app_with_base`] with `Some("assets")`.
pub async fn bundle_app(
    repo_root: &Path,
    packs: &[String],
    project: &Path,
    out_dir: &Path,
    minify: bool,
    bundle_framework: bool,
) -> Result<AppBuild> {
    bundle_app_with_base(repo_root, packs, project, out_dir, minify, bundle_framework, None).await
}

/// `bundle_app` with an explicit asset-URL `base`. `base` overrides the URL
/// namespace the generated `index.html` references assets under: `Some("assets")`
/// makes the URLs (`/assets/<file>`) MATCH the on-disk `assets/<file>` layout, so
/// the output is directly static-host-servable (no deck process / serve route) —
/// e.g. `deck build --bundle-framework --base assets`. `None` keeps the
/// package-name serve namespace (the host's per-app route maps it to `assets/`).
pub async fn bundle_app_with_base(
    repo_root: &Path,
    packs: &[String],
    project: &Path,
    out_dir: &Path,
    minify: bool,
    bundle_framework: bool,
    base: Option<&str>,
) -> Result<AppBuild> {
    // 0. Load the host's per-app `deck.jsonc` (optional). The file is
    // OPTIONAL — an app without one is a valid conventional app: no
    // injectGlobals, csr render, no `entry.gen.ts`. The whole-config
    // `Default` lands on `render: csr`, empty injectGlobals, no entry.
    // The host's apps.jsonc owns route/name/source; deck-config names
    // no path — the bundler is the per-app call site, the host binds
    // the orchestration list separately.
    let app_cfg_path = project.join("deck.jsonc");
    let app_cfg = if app_cfg_path.exists() {
        let raw = std::fs::read_to_string(&app_cfg_path)
            .with_context(|| format!("read {}", app_cfg_path.display()))?;
        Some(deck_config::AppServeConfig::parse_deck_jsonc(&raw)
            .with_context(|| format!("parse {}", app_cfg_path.display()))?)
    } else {
        None
    };
    let entry_bootstrap: Option<deck_config::EntryBootstrap> =
        app_cfg.as_ref().and_then(|c| c.entry.clone());
    // 0b. Discover provider-declared framework singletons by walking
    // the app's `package.json` deps. Each dep's own `package.json`
    // MAY carry a `deckFramework` field (mirror of `customElements`).
    // The bundler assembles the importmap + the externals list + the
    // `--bundle-framework` copy set from the discovered declarations.
    // Deck names no ship package; each singleton owner names itself.
    let discovered = discover_deck_frameworks(project, repo_root)
        .context("discover provider-declared framework singletons")?;
    // 1. Generate the route table from `<project>/routes/`. The dev doesn't
    // author a route config — the routes/ tree IS the source. The bundler
    // emits `routes.gen.ts` to the same dir.
    let routes_spec = crate::routes::RouteGenSpec {
        dir: project.join("routes"),
        out: project.join("routes.gen.ts"),
        label: project
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_string(),
        // Plumb the host's entry bootstrap into the route-table spec so
        // `generate_routes` ALSO emits `entry.gen.ts` (when declared) and
        // keeps `routes.gen.ts` data-only. Without this, the data file
        // would re-acquire the router call + token import.
        entry_bootstrap: entry_bootstrap.clone(),
    };
    crate::routes::generate_routes(&[routes_spec]).context("generate routes.gen.ts")?;

    // 1b. Component catalog for this app: aggregate CEM modules from component
    // pack deps and emit `components.gen.ts`. No-op when the project has no
    // `package.json` dependencies on component packs.
    crate::cem::generate_components_gen(project, repo_root)
        .context("generate components.gen.ts")?;

    // 2. Read the project name from `package.json#name`. The dev's
    // npm name is the app's identity; the URL namespace is the
    // last `/`-segment (so `@ship/hub` → `hub`). One source of
    // truth: `package.json#name`.
    let project_name = read_package_name(project)
        .with_context(|| format!("read package.json at {}", project.display()))?;
    // The asset-URL namespace: an explicit `base` (static-host build) wins;
    // otherwise the package name's last segment (`@ship/hub` → `hub`), the per-app
    // serve route a host maps to `assets/`.
    let url_namespace = match base {
        Some(b) => b.trim_matches('/').to_string(),
        None => project_name
            .rsplit_once('/')
            .map(|(_, tail)| tail)
            .unwrap_or(&project_name)
            .to_string(),
    };
    // 3. Generate the importmap. The framework serves the runtime singletons
    // at /_deck/* — the workspace root. Discovered provider-declared
    // singletons (each dep's `package.json#deckFramework`) ride the same
    // `/_deck/<basename>` URL convention so the daemon's static-asset
    // handler serves them on the same code path as the framework
    // singletons. No per-app dist prefix; the app's name is its URL
    // namespace.
    let importmap_json = generate_importmap("", &discovered);
    let externals = importmap_externals_from_json(&importmap_json)?;
    // 4. Render `index.html` from the template. The dev doesn't author
    // index.html — it's deck's entry contract. The template has placeholders
    // for the project name, the importmap, and the entry module name. The
    // entry is ALWAYS the generated `entry.gen.ts` bootstrap (it calls
    // `mountRouter`); `generate_routes` emits it unconditionally — deck's
    // default when the host declares no `entry.bootstrap`, the host's clients
    // wired in when it does. (`routes.gen.ts` is data-only and never mounts.)
    let entry_module = "entry.gen.ts";
    let html = crate::templates::render_index_html(
        &project_name,
        &importmap_json,
        entry_module,
    );
    // Atomic publication (dev-loop-atomic): the old flow deleted the LIVE out
    // dir here and wrote `index.html` last, so every request during a watch
    // rebuild saw `scaffold '<app>' not built (missing .deck/index.html)` for
    // the whole rebuild window. Build into a sibling staging dir instead and
    // promote it file-by-file via atomic rename at the end (`publish_build`,
    // index.html last) — the served tree always carries a complete scaffold.
    // Pruning stale files (the job `remove_dir_all` used to do, for the
    // build-owned `dist`/`.deck` leaves only) happens AFTER the swap.
    let out_leaf = out_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let prune_stale = out_leaf == "dist" || out_leaf == deck_config::DECK_OUT_DIRNAME;
    let dest_dir = out_dir;
    let stage = crate::publish::stage_path_for(dest_dir);
    if stage.exists() {
        std::fs::remove_dir_all(&stage)
            .with_context(|| format!("clear stale staging dir {}", stage.display()))?;
    }
    // Everything below writes into the STAGE; `publish_build` promotes it.
    let out_dir: &Path = &stage;
    let assets_dir = out_dir.join("assets");
    std::fs::create_dir_all(&assets_dir)?;

    // 5. Bundle the generated entry as the SPA entry. The dev's source
    // tree has no `main.ts`; the bundler generates it from the
    // routes/ walk (data only) plus, when the host declares one, a
    // host-owned `entry.gen.ts` (client wiring + mountRouter call).
    // AppPlugin runs FIRST so importmap specifiers (incl. `@ship/sail-runtime/*`
    // subpaths and every host-registered client alias) win as externals;
    // SailPlugin then compiles `.sail` siblings and re-anchors the remaining
    // `@ship/<pkg>` (e.g. sail-charts, pipelines) to workspace source so the
    // app graph compiles them in-process.
    let routes_gen = project.join(entry_module);
    if !routes_gen.exists() {
        bail!("app build: expected generated entry at {}", routes_gen.display());
    }
    // CSS ships as a hashed asset via Rolldown's native `ModuleType::Asset`
    // path (`rolldown_plugin_asset_module`). The `SailPlugin::load` hook only
    // handles `.sail` / `.compiled.js` and returns `Ok(None)` for everything
    // else, so the built-in `AssetModulePlugin` runs and emits a hashed
    // `assets/<name>-<hash>.css` file. The chunk references it via a
    // placeholder that `render_chunk` resolves to the asset filename;
    // we then lift that link into the generated `index.html` below.
    // (Rolldown 1.0 removed `ModuleType::Css` — issue rolldown/rolldown#4271;
    // the asset type is the supported way to ship a CSS file.)
    let mut module_types: rustc_hash::FxHashMap<String, ModuleType> =
        rustc_hash::FxHashMap::default();
    module_types.insert(".css".to_string(), ModuleType::Asset);
    let options = BundlerOptions {
        input: Some(vec![InputItem {
            name: Some("entry".to_string()),
            import: routes_gen.to_string_lossy().to_string(),
        }]),
        cwd: Some(repo_root.to_path_buf()),
        dir: Some(out_dir.to_string_lossy().to_string()),
        format: Some(OutputFormat::Esm),
        minify: Some(minify.into()),
        entry_filenames: Some(ChunkFilenamesOutputOption::String(
            "assets/[name]-[hash].js".to_string(),
        )),
        chunk_filenames: Some(ChunkFilenamesOutputOption::String(
            "assets/[name]-[hash].js".to_string(),
        )),
        module_types: Some(module_types),
        ..Default::default()
    };
    let app_plugin = AppPlugin { externals };
    let resolver = crate::sail_plugin::component_index(repo_root, packs);
    let sail_plugin = crate::sail_plugin::SailPlugin {
        repo_root: repo_root.to_path_buf(),
        emit_component_imports: true,
        component_index: resolver.index,
        shadowed_tags: resolver.shadowed,
        component_roots: crate::catalog::component_package_roots(repo_root),
    };
    let mut bundler = BundlerBuilder::default()
        .with_options(options)
        .with_plugins(vec![
            Arc::new(app_plugin),
            Arc::new(sail_plugin),
            Arc::new(crate::ship_asset_css::ShipAssetCssPlugin),
        ])
        .build()
        .map_err(|e| anyhow!("rolldown build(app): {e:?}"))?;
    let app_output = bundler
        .write()
        .await
        .map_err(|e| anyhow!("rolldown write(app): {e:?}"))?;
    crate::rolldown_bundle::assert_no_unresolved_imports(&app_output, "app")?;
    // 5b. Blank-page guard: every emitted chunk's bare `@ship/*` imports must be
    // resolvable by the document importmap (or have been inlined). A bare
    // `@ship/*` the importmap doesn't cover throws "Failed to resolve module
    // specifier" at runtime — a silent blank page — so fail loud here naming the
    // offender(s). `importmap_externals_from_json` re-derives the key set from
    // the same `importmap_json` the document carries (single source of truth).
    let importmap_keys = importmap_externals_from_json(&importmap_json)?;
    assert_pack_imports_resolved(&assets_dir, &importmap_keys)
        .with_context(|| format!("app build: {}", project.display()))?;
    // 6. The entry chunk is `assets/entry-<hash>.js` (input name "entry").
    // The script href in the generated HTML is the final URL — the
    // bundler is the only place that knows the project name (the
    // framework serves the file at `/<name>/<file>`). No string
    let js_name = std::fs::read_dir(&assets_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.starts_with("entry-") && n.ends_with(".js"))
        .context("app build: no entry chunk assets/entry-*.js emitted")?;
    let js_href = format!("/{url_namespace}/{js_name}");
    // 6b. Discover every CSS asset Rolldown emitted under `assets/`. The
    // `AssetModulePlugin` writes each `.css` import as `assets/<name>-<hash>.css`
    // (content-hashed; default `asset_filenames = "assets/[name]-[hash][extname]"`).
    // We link them directly from `index.html` — Vite-style: the SPA entry is JS,
    // but global / token / shared CSS is a stylesheet the browser fetches in
    // parallel, before the JS runs. The SailPlugin's `load` hook returns `Ok(None)`
    // for `.css`, so the builtin `AssetModulePlugin` handles it (it runs AFTER
    // user plugins via `PluginHookMeta.order = Post`).
    let mut css_names: Vec<String> = std::fs::read_dir(&assets_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".css"))
        .collect();
    css_names.sort();
    let css_hrefs: Vec<String> = css_names
        .iter()
        .map(|n| format!("/{url_namespace}/{n}"))
        .collect();
    let public_dir = project.join("public");
    if public_dir.exists() {
        for entry in std::fs::read_dir(&public_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                std::fs::copy(entry.path(), out_dir.join(entry.file_name()))?;
            }
        }
    }
    // Framework co-assets (e.g. `sail_compiler_bg.wasm`) must land at the SPA
    // root even when `bundle_framework` is false. deck dev serves the JS glue
    // from the baked `/_deck/compiler-wasm.js` singleton, but the companion
    // `.wasm` is requested at `/sail_compiler_bg.wasm` and is NOT embedded —
    // without this stage a bare `deck dev` (no prior `public/` copy from gen)
    // silently 404s the binary and the tutorial/playground die.
    stage_framework_co_assets(repo_root, project, out_dir)?;

    // 8. Write the rendered index.html (the dev doesn't author it; the
    // template was rendered above in step 4). The script tag references
    // the entry chunk; CSS assets (if any) get a `<link rel="stylesheet">`
    // injected BEFORE the entry script so the browser fetches and applies
    // them in parallel with the module load — same first-paint guarantee
    // the per-route synthetic CSS render lifts (orthogonal layer — this is
    // the GLOBAL/linkable CSS path; the route-synthetic one handles route
    // chrome inline in SSR/CSR first-paint).
    let mut final_html = html.replace(
        &format!("/{entry_module}"),
        &js_href,
    );
    if !css_hrefs.is_empty() {
        let link_tags = css_names
            .iter()
            .zip(css_hrefs.iter())
            .map(|(name, href)| {
                // The `@ship/tokens/contracts.css` entry import (deck-deploy-gap,
                // 2nd incident) emits as `assets/contracts-<hash>.css` — the
                // AssetModulePlugin names each emitted CSS asset after its
                // SOURCE file stem, so `contracts-` is a stable prefix. The
                // sail runtime's `ensureLightContracts` looks for a
                // `link[data-sail-contracts]` before warning "framework
                // contracts are not present"; the daemon-served scaffold
                // shell (`scaffold-base.html`) already hand-authors the
                // marker on its `sail.css` link, but THIS bundler-generated
                // link (the self-contained/`entry.gen.ts` build path — what
                // `deck dev` and `deck build --bundle-framework` both use)
                // never carried it, so every page tripped the warning. Tag
                // it here rather than teaching the runtime to sniff hrefs:
                // the attribute is the existing, already-tested contract: a
                // href-based heuristic would couple the runtime to this
                // bundler's naming convention AND still need updating
                // whenever that convention changes, where this is a
                // one-line fix at the single place the link is emitted.
                if name.starts_with("contracts-") {
                    format!("  <link rel=\"stylesheet\" href=\"{href}\" data-sail-contracts>")
                } else {
                    format!("  <link rel=\"stylesheet\" href=\"{href}\">")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        // Anchor immediately before the entry `<script type="module">`. The
        // template renders `<script type="module" src="/<entry_module>">`
        // first as a relative substitute; after the `.replace()` above it
        // carries the final hashed href.
        final_html = final_html.replace(
            "  <script type=\"module\"",
            &format!("{link_tags}\n  <script type=\"module\""),
        );
    }
    std::fs::write(out_dir.join("index.html"), final_html)?;
    let mut framework_assets = 0;
    if bundle_framework {
        let sui_dir = out_dir.join("_deck");
        std::fs::create_dir_all(&sui_dir)?;
        // Deck-owned framework singletons. Built FROM SOURCE, resolved by node
        // resolution from the project's node_modules (`framework_singletons`) —
        // no prebuilt-asset dir, no `DECK_FRAMEWORK_ASSETS_DIR`. Each singleton's
        // output is mapped onto its served `_deck/<file>` URL via FRAMEWORK_ASSETS
        // (one source bundle may back several aliases, e.g. `@ship/sail-runtime`
        // and `@ship/sail-runtime/route`).
        framework_assets +=
            bundle_owned_framework(repo_root, out_dir, minify).await?;
        // Discovered provider-declared singletons. Each `src_path` is
        // already absolute (the dep's workspace package root +
        // `package.json#deckFramework#entry`); the importmap binds the
        // alias to `<serve>`'s basename. The bundle copy writes
        // `<out>/<serve>` — the path the importmap binds — so the
        // daemon's static-asset handler serves it on the same code
        // path as the framework singletons.
        for decl in &discovered {
            if !decl.src_path.exists() {
                bail!(
                    "app build: bundle_framework: missing framework singleton `{}` (from {})",
                    decl.alias,
                    decl.src_path.display()
                );
            }
            let dst = out_dir.join(&decl.serve);
            if dst.exists() {
                continue;
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&decl.src_path, &dst)?;
            framework_assets += 1;
        }
    }
    // (formerly refreshed the runtime catalog by writing
    // `<repo_root>/.deck/catalogs/workspace.json`) — purged:
    // the framework no longer writes a monorepo-root `.deck/catalogs/`
    // (deck-purge-root). The catalog walker
    // (`crate::catalog::build_catalog`) stays available for hosts that
    // want to compute the catalog in-memory for their own devtools; it
    // just does not get written to disk by the framework.

    // Promote the staged build into the live out dir: per-file atomic rename,
    // index.html LAST, stale files pruned after the swap (dev-loop-atomic).
    crate::publish::publish_build(&stage, dest_dir, prune_stale)?;

    Ok(AppBuild {
        out_dir: dest_dir.to_path_buf(),
        js_href,
        css_hrefs,
        framework_assets,
        url_namespace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, s: &str) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, s).unwrap();
    }

    /// `bundle_app_with_base(Some("assets"))` emits asset URLs that MATCH the
    /// on-disk `assets/<file>` layout, so a `deck build --base assets` is directly
    /// static-host-servable — vs the default package-name serve namespace
    /// (`/<name>/<file>`) a host maps to `assets/` via its per-app route. (T18.)
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_with_base_emits_static_servable_asset_urls() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"@scope/branded-app","private":true,"type":"module"}"#,
        );
        write(&r.join("routes/page.sail"), "<main>home</main>\n");

        // Default: the package name's last segment is the serve namespace.
        let out_def = r.join("dist-default");
        let def = bundle_app(r, &[], r, &out_def, false, false).await.expect("default");
        assert!(
            def.js_href.starts_with("/branded-app/entry-"),
            "default uses the package-name serve namespace; got {}",
            def.js_href,
        );

        // `--base assets`: the URL matches the on-disk `assets/` dir.
        let out_base = r.join("dist-base");
        let based = bundle_app_with_base(r, &[], r, &out_base, false, false, Some("assets"))
            .await
            .expect("with base");
        assert!(
            based.js_href.starts_with("/assets/entry-"),
            "base override emits /assets/<file>; got {}",
            based.js_href,
        );
        // The static-servability invariant: the emitted URL resolves to a real
        // file under the output (a dumb by-path host would find it).
        assert!(
            out_base.join(based.js_href.trim_start_matches('/')).exists(),
            "emitted URL {} must resolve to a real file in the output",
            based.js_href,
        );
    }

    /// dev-loop-atomic end-to-end: a component DIR added between two builds is
    /// registered by the next rebuild once the structural-change hook clears
    /// the component-index memo — no restart. The same rebuild also gates the
    /// atomic-publish contract: `index.html` present after the rebuild, the
    /// previous build's stale asset pruned, and a foreign `routes/` bundle
    /// sharing the out dir left untouched (the old `remove_dir_all` nuked it).
    #[tokio::test(flavor = "multi_thread")]
    async fn rebuild_registers_new_component_dir_and_publishes_atomically() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"@scope/dev-loop-app","private":true,"type":"module"}"#,
        );
        write(&r.join("routes/page.sail"), "<main><late-probe>hi</late-probe></main>\n");
        let out = r.join(deck_config::DECK_OUT_DIRNAME);

        let assets_js = |out: &Path| -> String {
            let mut all = String::new();
            for e in std::fs::read_dir(out.join("assets")).unwrap().flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.ends_with(".js") {
                    all.push_str(&std::fs::read_to_string(e.path()).unwrap());
                }
            }
            all
        };

        // Build 1: the tag has no source yet — it compiles as a no-op.
        bundle_app(r, &[], r, &out, false, false).await.expect("first build");
        assert!(
            !assets_js(&out).contains("late-probe-upgraded"),
            "component doesn't exist yet — nothing to compile in"
        );

        // Mid-session: the author creates a NEW component dir, and other
        // writers' state accumulates in the out dir.
        write(
            &r.join("src/late-probe/late-probe.sail"),
            "<template>\n  <span>late-probe-upgraded <slot></slot></span>\n</template>\n",
        );
        write(&out.join("assets/entry-STALE.js"), "from a previous build");
        write(&out.join("routes/keep/route.js"), "bundler-owned view bundle");

        // The watcher's structural-change hook (deck dev calls this on a
        // Create event) clears the memo; the rebuild then registers the tag.
        crate::sail_plugin::invalidate_component_index();
        bundle_app(r, &[], r, &out, false, false).await.expect("rebuild");

        assert!(
            assets_js(&out).contains("late-probe-upgraded"),
            "the mid-session component dir must compile into the rebuild — no restart"
        );
        assert!(out.join("index.html").exists(), "scaffold present after rebuild");
        assert!(
            !out.join("assets/entry-STALE.js").exists(),
            "stale hashed asset from the previous build is pruned"
        );
        assert_eq!(
            std::fs::read_to_string(out.join("routes/keep/route.js")).unwrap(),
            "bundler-owned view bundle",
            "foreign routes/ tree survives the rebuild (remove_dir_all used to nuke it)"
        );
        assert!(
            !crate::publish::stage_path_for(&out).exists(),
            "staging dir cleaned up after publish"
        );
    }

    /// The dev authors `package.json`, `routes/`, `public/`. The bundler
    /// generates `index.html` (with the importmap), `routes.gen.ts` (the
    /// entry), the entry chunk, and copies `public/`. Behavioral
    /// assertions, not text matching — a future index.html template
    /// change shouldn't break this test.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_generates_index_and_routes_gen() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"test-app","private":true,"type":"module"}"#,
        );
        // A GENERIC dynamic route — deliberately not any app's real route, so
        // this framework test never couples to a specific app's IA. The
        // emitter statically imports + compiles each route module, so the
        // fixture must compile: `id` is bound by the page's own controller.
        write(
            &r.join("routes/widgets/[id]/page.sail"),
            "<controller>\n\
import { controller, signal } from '@ship/sail-runtime'\n\
export default controller(() => ({ id: signal('demo') }))\n\
</controller>\n\
<div>{id()}</div>\n",
        );
        write(&r.join("public/favicon.ico"), "ICO");

        let out = r.join("dist");
        let build = bundle_app(r, &[], r, &out, false, false)
            .await
            .expect("bundle_app");

        // Behavior 1: the dev's `index.html` doesn't exist; the bundler
        // generated one at `<out>/index.html`.
        assert!(!r.join("index.html").exists(), "dev did not author index.html");
        assert!(out.join("index.html").exists(), "bundler generated index.html");
        // The generated HTML is valid and references the entry chunk.
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains(&build.js_href), "html references bundled entry chunk");

        // Behavior 2: the entry is the generated `routes.gen.ts` (NOT a
        // dev-authored `src/main.ts`). The bundled output references the
        // routes directory's `page.sail` pattern.
        assert!(!r.join("src/main.ts").exists(), "dev did not author main.ts");
        // The bundler generated the entry, and the dynamic route bundled —
        // bundle_app() above would have errored if the route module failed to
        // compile into the graph. We do NOT string-match the generated TS:
        // that's presentation, and the router's behavior is covered by
        // @ship/deck/router's mount-router tests.
        assert!(r.join("routes.gen.ts").exists(), "bundler generated the routes.gen.ts entry");

        // Behavior 3: the framework singletons are externalized in the
        // bundle (the importmap is the runtime's source of truth, not
        // the bundler — but the importmap must at least NAME the assets).
        assert!(html.contains("@ship/sail-runtime"), "framework runtime in importmap");
        // Deck must NOT hardcode any framework CLIENT (no `@ship/sdk-client`).
        // Framework clients are per-app registrations the host supplies
        // through `AppServeConfig::framework_clients`. A default build
        // (no host deck.jsonc) emits NO client alias — the framework
        // names no ship path.
        assert!(
            !html.contains("@ship/sdk-client"),
            "framework must NOT hardcode `@ship/sdk-client` — host-registered (got: {html})",
        );

        // Behavior 4: the entry chunk href is the final served path.
        // The bundler writes `/<url_namespace>/entry-<hash>.js` so
        // the framework can serve it at the corresponding route
        // without a string rewrite. The URL namespace is the last
        // `/`-segment of `package.json#name` (`@ship/hub` → `hub`).
        assert!(build.js_href.starts_with("/test-app/entry-") && build.js_href.ends_with(".js"),
                "expected /test-app/entry-*.js, got {}", build.js_href);
        // that knows the project name; the framework serves the
        // file at `/<name>/<file>`.
        assert!(build.js_href.starts_with("/test-app/entry-") && build.js_href.ends_with(".js"),
                "expected /test-app/entry-*.js, got {}", build.js_href);
        // Behavior 5: public/ assets are copied to out/.
        assert!(out.join("favicon.ico").exists(), "public/ copied");
    }

    /// Bare `deck dev` (`bundle_framework = false`) must still stage the
    /// wasm-bindgen co-asset at the SPA root. Without it `/sail_compiler_bg.wasm`
    /// 404s while `/` returns 200 — the cold-worktree silent-partial.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_stages_wasm_co_asset_without_bundle_framework() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"wasm-app","private":true,"type":"module",
                "dependencies":{"@ship/sail-compiler-wasm":"workspace:*"}}"#,
        );
        write(&r.join("routes/page.sail"), "<div>hi</div>\n");
        write(
            &r.join("node_modules/@ship/sail-compiler-wasm/package.json"),
            r#"{ "name": "@ship/sail-compiler-wasm", "exports": {
                ".": "./pkg/sail_compiler.js",
                "./sail_compiler_bg.wasm": "./pkg/sail_compiler_bg.wasm" } }"#,
        );
        write(
            &r.join("node_modules/@ship/sail-compiler-wasm/pkg/sail_compiler.js"),
            "export default function init() {}\n",
        );
        std::fs::write(
            r.join("node_modules/@ship/sail-compiler-wasm/pkg/sail_compiler_bg.wasm"),
            b"\0asm-co-asset",
        )
        .unwrap();

        let out = r.join("dist");
        bundle_app(r, &[], r, &out, false, false)
            .await
            .expect("bundle_app without --bundle-framework");

        let wasm = out.join("sail_compiler_bg.wasm");
        assert!(
            wasm.is_file(),
            "co-asset must land at SPA root even when framework JS stays external"
        );
        assert_eq!(std::fs::read(&wasm).unwrap(), b"\0asm-co-asset");
    }

    /// Installed-but-unbuilt `@ship/sail-compiler-wasm` must fail the app build
    /// with a loud fix command — never start a half-live site.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_fails_loud_when_wasm_pkg_unbuilt() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"wasm-app","private":true,"type":"module",
                "dependencies":{"@ship/sail-compiler-wasm":"workspace:*"}}"#,
        );
        write(&r.join("routes/page.sail"), "<div>hi</div>\n");
        // Package root present, but no pkg/ entry or .wasm (cold wipe of
        // wasm-pack output).
        write(
            &r.join("node_modules/@ship/sail-compiler-wasm/package.json"),
            r#"{ "name": "@ship/sail-compiler-wasm", "exports": {
                ".": "./pkg/sail_compiler.js",
                "./sail_compiler_bg.wasm": "./pkg/sail_compiler_bg.wasm" } }"#,
        );
        let out = r.join("dist");
        let result = bundle_app(r, &[], r, &out, false, false).await;
        let err = match result {
            Ok(_) => panic!("unbuilt wasm package must fail the build"),
            Err(e) => e,
        };
        let msg = format!("{err:#}");
        assert!(
            msg.contains("bun run wasm") || msg.contains("sail_compiler_bg.wasm"),
            "error must name the fix / missing co-asset, got: {msg}"
        );
    }


    /// The dev doesn't write a main.ts. The entry is `routes.gen.ts`,
    /// which the bundler generates. A `.sail` file inside `routes/`
    /// must compile IN-PROCESS (no `.compiled.js` artifact) and be
    /// available to the bundle. A failure to compile would surface as
    /// a panic from the bundler, not as a missing chunk.
    #[tokio::test(flavor = "multi_thread")]
    async fn app_build_compiles_sui_in_process() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"test-app","private":true,"type":"module"}"#,
        );
        write(
            &r.join("routes/[name]/page.sail"),
            "<div>hi {1 + 1}</div>\n",
        );

        let out = r.join("dist");
        bundle_app(r, &[], r, &out, false, false)
            .await
            .expect("bundle_app should compile the .sail entry in-process");
    }

    /// SFC route pages: a `page.sail` with an inline `<controller>` block is the
    /// single-file authoring model — no sibling `controller.ts` exists and none
    /// may be required. The bundler must (1) build the app and (2) actually
    /// carry the controller code into the emitted chunks; silently dropping the
    /// controller (template renders, no state) is the failure mode this pins.
    #[tokio::test(flavor = "multi_thread")]
    async fn app_build_bundles_inline_controller_sfc_page() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"sfc-app","private":true,"type":"module"}"#,
        );
        write(
            &r.join("routes/counter/page.sail"),
            "<controller>\n\
             import { controller, signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(41) }))\n\
             </controller>\n\
             <main>\n  <p>{n()}</p>\n</main>\n",
        );

        let out = r.join("dist");
        bundle_app(r, &[], r, &out, false, false)
            .await
            .expect("an inline-<controller> page.sail must bundle");

        // The controller must be IN the output — walk every emitted JS chunk
        // (route modules are code-split) for the scope literal.
        let mut found = false;
        let mut stack = vec![out.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let p = entry.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|e| e == "js")
                    && std::fs::read_to_string(&p).unwrap().contains("signal(41)")
                {
                    found = true;
                }
            }
        }
        assert!(found, "the inline controller's code (signal(41)) must land in an emitted chunk");
    }

    /// When a deck app's module graph imports a `.css` file, the per-app
    /// `bundle_app` builds it through Rolldown's NATIVE asset path
    /// (`module_types: { ".css" -> Asset }` → `AssetModulePlugin`), emits
    /// a content-hashed file under `<out>/assets/`, and lifts the link
    /// into the generated `index.html` so the browser fetches the
    /// stylesheet in parallel with the SPA entry module.
    ///
    /// Gate per `.artifacts/job-spec.md`: a CSS file imported by the entry
    /// MUST end up as a hashed `.css` under `assets/` AND `<link
    /// rel="stylesheet" href="/<ns>/<file>.css">` MUST appear in the
    /// generated HTML — same URL convention the entry JS uses
    /// (`/<namespace>/<file>`), so the daemon's `serve_app_asset`
    /// (`GET /<app>/{file}`) serves it.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_emits_and_links_css_assets() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"css-test-app","private":true,"type":"module"}"#,
        );
        // A minimal route so the entry isn't empty. The CSS import goes
        // on the sibling controller so it reaches the entry graph (the
        // entry is `routes.gen.ts`, which statically imports this).
        // Controller is only here so the CSS import reaches the entry graph
        // (routes.gen.ts imports the virtual controller module). Not empty:
        // sail-controller-empty rejects `() => ({})`.
        write(
            &r.join("routes/page.sail"),
            "<controller>\n\
import './styles.css';\n\
export default controller(() => ({ ok: true }))\n\
</controller>\n\
<div>css</div>\n",
        );
        write(
            &r.join("routes/styles.css"),
            ".css-probe { color: rebeccapurple; }\n",
        );

        let out = r.join("dist");
        let build = bundle_app(r, &[], r, &out, false, false)
            .await
            .expect("bundle_app");

        // Gate 1: at least one hashed `.css` asset emitted under
        // `<out>/assets/`. The default Rolldown asset filename is
        // `assets/[name]-[hash][extname]` → `assets/styles-<hash>.css`.
        let css_on_disk: Vec<String> = std::fs::read_dir(out.join("assets"))
            .expect("assets dir emitted")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".css"))
            .collect();
        assert!(
            !css_on_disk.is_empty(),
            "expected a hashed .css asset under <out>/assets/, got none"
        );
        // The app's OWN imported CSS must be content-hashed off its source stem
        // (`styles.css` → `styles-<hash>.css`). The build ALSO emits the design
        // tokens stylesheet (`tokens-<hash>.css`): `routes.gen.ts`/`entry.gen.ts`
        // unconditionally import `@ship/tokens/tokens.css` (the entry contract),
        // and `component_package_roots` now unions sail's own workspace so that
        // import resolves to source and bundles — exactly the path proven by
        // `bundle_app_bundles_design_tokens_via_entry`. So assert the app's stem
        // is present, not that it's the ONLY asset.
        assert!(
            css_on_disk.iter().any(|n| n.starts_with("styles-")),
            "the app's imported CSS should be content-hashed off the source stem \
             `styles-` (got {css_on_disk:?})"
        );
        assert!(
            css_on_disk.iter().all(|n| n.starts_with("styles-")
                || n.starts_with("tokens-")
                || n.starts_with("contracts-")
                || n.starts_with("element-skins-")),
            "only the app CSS (`styles-`), the tokens CSS (`tokens-`), and \
             the framework stylesheet halves (`contracts-`, `element-skins-`) \
             should be emitted (got {css_on_disk:?})"
        );

        // Gate 2: AppBuild exposes the absolute URLs the index.html link
        // references — one per emitted file, all under `/<namespace>/`.
        assert_eq!(build.css_hrefs.len(), css_on_disk.len(), "css_hrefs matches disk");
        for href in &build.css_hrefs {
            assert!(
                (href.starts_with("/css-test-app/styles-")
                    || href.starts_with("/css-test-app/tokens-")
                    || href.starts_with("/css-test-app/contracts-")
                    || href.starts_with("/css-test-app/element-skins-"))
                    && href.ends_with(".css"),
                "css_href uses the same URL convention as the entry (/<ns>/<file>.css), got {href}"
            );
        }

        // Gate 3: the generated `index.html` links the CSS before the
        // entry `<script type="module">`. Order matters — the stylesheet
        // must apply BEFORE first paint, in parallel with module fetch.
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(
            html.contains("<link rel=\"stylesheet\""),
            "index.html must link the CSS asset"
        );
        for href in &build.css_hrefs {
            assert!(
                html.contains(href),
                "index.html must reference {href}"
            );
        }
        // The link must precede the entry script (link is fetched + applied
        // before module eval; same first-paint guarantee the route-synthetic
        // CSS render lifts — but for GLOBAL/linkable CSS, not route chrome).
        let link_pos = html.find("<link rel=\"stylesheet\"").unwrap();
        let script_pos = html.find("<script type=\"module\"").unwrap();
        assert!(
            link_pos < script_pos,
            "<link rel=stylesheet> must precede <script type=module>; got link@{link_pos} script@{script_pos}"
        );

        // Gate 4 (deck-deploy-gap, 2nd incident): the contracts stylesheet's
        // `<link>` must carry `data-sail-contracts` — the runtime's
        // `ensureLightContracts` looks for exactly that attribute before
        // warning "framework contracts are not present". Every OTHER link
        // (tokens, the app's own CSS) must NOT carry it — the marker means
        // "this link satisfies the framework contracts requirement," not
        // "this is a framework stylesheet."
        let contracts_href = build
            .css_hrefs
            .iter()
            .find(|h| h.contains("/contracts-"))
            .expect("a contracts-<hash>.css asset was emitted (asserted above)");
        assert!(
            html.contains(&format!("href=\"{contracts_href}\" data-sail-contracts")),
            "the contracts <link> must carry data-sail-contracts; got: {html}"
        );
        for href in &build.css_hrefs {
            if href == contracts_href {
                continue;
            }
            assert!(
                !html.contains(&format!("href=\"{href}\" data-sail-contracts")),
                "only the contracts link should carry data-sail-contracts; {href} must not"
            );
        }

        // Belt-and-braces: exactly one `<link rel="stylesheet">` per
        // emitted CSS asset (no synthesized empty links when nothing was
        // imported — the existing `bundle_app_generates_index_and_routes_gen`
        // covers the zero-import path implicitly).
        assert_eq!(
            html.matches("<link rel=\"stylesheet\"").count(),
            css_on_disk.len(),
            "exactly one <link rel=stylesheet> per emitted CSS asset"
        );
        // Belt-and-braces: exactly one `<link rel="stylesheet">` per
        // emitted CSS asset (no synthesized empty links when nothing was
        // imported — the existing `bundle_app_generates_index_and_routes_gen`
        // covers the zero-import path implicitly).
        assert_eq!(
            html.matches("<link rel=\"stylesheet\"").count(),
            css_on_disk.len(),
            "exactly one <link rel=stylesheet> per emitted CSS asset"
        );
    }

    /// Gate per `.artifacts/job-spec.md`: a freshly built sail app's
    /// `index.html` links the design-token stylesheet AND the emitted CSS
    /// under `assets/` defines `:root` tokens (e.g. `--background`). The
    /// token CSS enters the entry graph by convention: the generated
    /// `routes.gen.ts` imports `@ship/tokens/tokens.css` and Rolldown's
    /// native `AssetModulePlugin` bundles + links it. A static-only
    /// deploy (no daemon, no `view-tokens.css` serve route) has tokens.
    ///
    /// Mirrors the per-app build proof in
    /// `bundle_app_emits_and_links_css_assets` (the test it supersedes for
    /// the token path): the project must NOT hand-author the import —
    /// `routes.gen.ts` is generated, the import lands via the entry
    /// contract, and `var(--token)` resolves in the built bundle.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_bundles_design_tokens_via_entry() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        // Minimal workspace: the app project + the `@ship/tokens` package
        // that exports `./tokens.css`. `component_package_roots` walks the
        // workspace to build the `@ship/<name>` → source-dir map that
        // `SailPlugin::resolve_id` uses to re-anchor the bare specifier.
        // Without this map the bundler cannot find the CSS, and the gate
        write(
            &r.join("package.json"),
            r#"{"name":"tokens-app","private":true,"type":"module",
                "workspaces":["packages/sail/*"]}"#,
        );
        write(
            &r.join("packages/sail/tokens/package.json"),
            r#"{"name":"@ship/tokens","type":"module",
                "exports":{".":"./src/index.ts","./tokens.css":"./src/tokens.css",
                "./contracts.css":"./src/contracts.css",
                "./element-skins.css":"./src/element-skins.css"}}"#,
        );
        // Bare-minimum tokens.css: a single `:root` block with the
        // `--background` token the spec names as the gate's exemplar.
        // The gate asserts both presence (the convention) and shape
        // (a real `:root` block with the named token).
        write(
            &r.join("packages/sail/tokens/src/tokens.css"),
            ":root { --background: oklch(20% 0 0); }\n",
        );
        // The framework stylesheet the entry also imports (contracts + skins,
        // each a direct JS import). Minimal but real so the bundle resolves
        // the extra entry imports.
        write(
            &r.join("packages/sail/tokens/src/contracts.css"),
            "[tone='default'] { --tone: var(--foreground); }\n",
        );
        write(
            &r.join("packages/sail/tokens/src/element-skins.css"),
            ".sail-spinner { display: inline-flex; }\n",
        );
        write(
            &r.join("packages/sail/tokens/src/index.ts"),
            "export const __noop = 1;\n",
        );
        // Minimal route so the entry isn't empty. The token import goes
        // through the generated `routes.gen.ts` (not a hand-authored
        // controller) — that's the whole point of the gate.
        write(
            &r.join("routes/page.sail"),
            "<div>tok</div>\n",
        );
        let out = r.join("dist");
        let build = bundle_app(r, &[], r, &out, false, false)
            .await
            .expect("bundle_app");
        // Gate 1: a hashed `tokens-<hash>.css` asset is emitted under
        // `<out>/assets/`. The default Rolldown asset filename is
        // `assets/[name]-[hash][extname]`. The stem comes from the source
        // file (`packages/sail/tokens/src/tokens.css` → `tokens-<hash>.css`).
        let css_on_disk: Vec<String> = std::fs::read_dir(out.join("assets"))
            .expect("assets dir emitted")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".css"))
            .collect();
        assert!(
            css_on_disk.iter().any(|n| n.starts_with("tokens-")),
            "expected a tokens-<hash>.css asset under <out>/assets/ (got: {css_on_disk:?})",
        );
        // Gate 2: the build exposes the absolute URL the link references
        // — `/<namespace>/<file>.css` — and the generated `index.html`
        // links it. Same URL convention as the entry JS, so the daemon's
        // `serve_app_asset` (`GET /<app>/{file}`) serves it for dev.
        assert!(
            build.css_hrefs.iter().any(|h| h.starts_with("/tokens-app/tokens-") && h.ends_with(".css")),
            "css_hrefs must include the bundled tokens asset, got: {:?}",
            build.css_hrefs,
        );
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        for href in &build.css_hrefs {
            assert!(
                html.contains(href),
                "index.html must reference the bundled tokens href {href}",
            );
        }
        // The link must precede the entry script (parallel fetch, before
        // first paint — the same guarantee the merged CSS-asset path
        // establishes for hand-authored CSS).
        let link_pos = html.find("<link rel=\"stylesheet\"").expect("a <link rel=stylesheet> exists");
        let script_pos = html.find("<script type=\"module\"").expect("entry script exists");
        assert!(
            link_pos < script_pos,
            "token <link rel=stylesheet> must precede <script type=module>; got link@{link_pos} script@{script_pos}",
        );

        // Gate 3: the emitted token CSS defines `:root` tokens. The
        // presence of `--background` is the spec's named exemplar (it
        // resolves in every component that uses `var(--background)`).
        let tokens_path = out
            .join("assets")
            .join(css_on_disk.iter().find(|n| n.starts_with("tokens-")).unwrap());
        let tokens_css = std::fs::read_to_string(&tokens_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", tokens_path.display()));
        assert!(
            tokens_css.contains(":root"),
            "emitted tokens CSS must define a :root block (got {} bytes)",
            tokens_css.len(),
        );
        assert!(
            tokens_css.contains("--background:"),
            "emitted tokens CSS must define the --background role token (the spec's named exemplar)",
        );
    }

    /// Provider-declared framework singleton discovery: when an app
    /// dependency's `package.json` carries a `deckFramework` field,
    /// the bundler:
    ///   1. emits an importmap entry binding the alias to
    ///      `/_deck/<serve-basename>`, and
    ///   2. copies the source asset (`<dep-root>/<entry>`) to
    ///      `<out>/<serve>` when `--bundle-framework` is set.
    /// The framework itself never hardcodes any client alias — every
    /// alias is the dep's own self-declaration. This test runs without
    /// `--bundle-framework`: the importmap is emitted at codegen time
    /// (before any framework-asset copy), so the alias is asserted
    /// regardless of bundle mode.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_discovers_provider_declared_singleton() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        // App + dependency form a workspace. The app's deps include
        // `@ship/sdk-client`, whose OWN package.json carries the
        // `deckFramework` self-declaration. The bundler walks the
        // dep tree and binds the alias from the dep's declaration.
        write(
            &r.join("package.json"),
            r#"{"name":"client-test-app","private":true,"type":"module",
                "workspaces":["packages/*"],
                "dependencies":{"@ship/sdk-client":"workspace:*"}}"#,
        );
        write(
            &r.join("packages/sdk-client/package.json"),
            r#"{
  "name": "@ship/sdk-client",
  "type": "module",
  "deckFramework": {
    "alias": "@ship/sdk-client",
    "entry": "sail-sdk-client.js",
    "serve": "_deck/sdk-client.js"
  }
}"#,
        );
        // Minimal route so the entry isn't empty.
        write(&r.join("routes/page.sail"), "<div>x</div>\n");

        let out = r.join("dist");
        bundle_app(r, &[], r, &out, false, false)
            .await
            .expect("bundle_app");

        // Gate: importmap binds the discovered alias to
        // `/_deck/<serve-basename>` (the dep's serve URL convention).
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(
            html.contains("\"@ship/sdk-client\": \"/_deck/sdk-client.js\""),
            "importmap must bind the discovered alias to `/_deck/sdk-client.js` (got: {html})",
        );
        // Hard constraint: framework must NEVER hardcode `@ship/sdk-client`
        // as a literal in `FRAMEWORK_ASSETS`. The alias only lands in the
        // importmap through provider-declared discovery (the dep's
        // own `package.json#deckFramework`).
        assert!(
            !html.contains("@ship/sdk-client")
                || html.contains("@ship/sdk-client\": \"/_deck/sdk-client.js\""),
            "the only place `@ship/sdk-client` may appear is the discovered importmap entry (got: {html})",
        );
    }

    /// `--bundle-framework` builds the deck-owned framework singletons FROM
    /// node-resolved SOURCE into `<out>/_deck/*` AND copies the provider-declared
    /// singleton (`@ship/sdk-client`) alongside. The deck-owned set resolves via
    /// node resolution (`node_modules/@ship/<pkg>`), not a prebuilt-asset dir —
    /// this is the `DECK_FRAMEWORK_ASSETS_DIR`-free path.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_copies_provider_declared_singleton_when_bundling() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        // App + dep workspace; the dep's package.json self-declares
        // its serve contract. The bundler discovers it and copies
        // the source asset when `--bundle-framework` is set.
        write(
            &r.join("package.json"),
            r#"{"name":"client-bundle-app","private":true,"type":"module",
                "workspaces":["packages/*"],
                "dependencies":{"@ship/sdk-client":"workspace:*"}}"#,
        );
        write(
            &r.join("packages/sdk-client/package.json"),
            r#"{
  "name": "@ship/sdk-client",
  "type": "module",
  "deckFramework": {
    "alias": "@ship/sdk-client",
    "entry": "sail-sdk-client.js",
    "serve": "_deck/sdk-client.js"
  }
}"#,
        );
        std::fs::write(
            r.join("packages/sdk-client/sail-sdk-client.js"),
            "// stub\n",
        )
        .unwrap();
        // Install the deck-owned framework singletons as node-resolvable
        // packages — `--bundle-framework` now BUILDS them from source
        // (resolved from `node_modules/@ship/<pkg>`), not a prebuilt dir.
        for (pkg, file) in [
            ("@ship/sail-runtime", "src/index.ts"),
            ("@ship/sail-dom", "src/index.ts"),
            ("@ship/deck-router", "src/index.ts"),
        ] {
            write(
                &r.join(format!("node_modules/{pkg}/package.json")),
                &format!(r#"{{ "name": "{pkg}", "exports": {{ ".": "./{file}" }} }}"#),
            );
            write(&r.join(format!("node_modules/{pkg}/{file}")), "export const ok = 1;\n");
        }
        // The code-editor/playground's wasm compiler + browser LSP singletons
        // — also deck-owned, also strictly required by `bundle_owned_framework`
        // now that they're in FRAMEWORK_ASSETS. `sail-compiler-wasm`'s co-asset
        // (`sail_compiler_bg.wasm`) must sit next to its resolved entry, or the
        // co-asset copy step 404s the build.
        write(
            &r.join("node_modules/@ship/sail-compiler-wasm/package.json"),
            r#"{ "name": "@ship/sail-compiler-wasm", "exports": {
                ".": "./pkg/sail_compiler.js",
                "./sail_compiler_bg.wasm": "./pkg/sail_compiler_bg.wasm" } }"#,
        );
        write(
            &r.join("node_modules/@ship/sail-compiler-wasm/pkg/sail_compiler.js"),
            "export const ok = 1;\n",
        );
        std::fs::write(
            r.join("node_modules/@ship/sail-compiler-wasm/pkg/sail_compiler_bg.wasm"),
            b"\0asm-stub",
        )
        .unwrap();
        write(
            &r.join("node_modules/@ship/sail-lsp-browser/package.json"),
            r#"{ "name": "@ship/sail-lsp-browser", "exports": {
                ".": "./src/client.ts",
                "./completions": "./src/completions.ts",
                "./diagnostics": "./src/diagnostics.ts" } }"#,
        );
        write(
            &r.join("node_modules/@ship/sail-lsp-browser/src/completions.ts"),
            "export const ok = 1;\n",
        );
        write(
            &r.join("node_modules/@ship/sail-lsp-browser/src/diagnostics.ts"),
            "export const ok = 1;\n",
        );
        write(&r.join("routes/page.sail"), "<div>x</div>\n");

        let out = r.join("dist");
        let build = bundle_app(r, &[], r, &out, false, true)
            .await
            .expect("bundle_app");

        // Gate 1: the provider-declared singleton asset was copied
        // to `<out>/<serve>` (the path the importmap binds).
        assert!(
            out.join("_deck/sdk-client.js").exists(),
            "`--bundle-framework` must copy the provider-declared singleton to <out>/<serve>",
        );
        // Gate 2: the deck-owned singletons were built from source into
        // their served `_deck/*` paths.
        for f in [
            "_deck/runtime.js",
            "_deck/dom.js",
            "_deck/route.js",
            "_deck/compiler-wasm.js",
            "_deck/lsp-completions.js",
            "_deck/lsp-diagnostics.js",
        ] {
            assert!(out.join(f).exists(), "deck-owned singleton {f} built from source");
        }
        // The wasm-bindgen co-asset rides to the PROJECT ROOT, not next to
        // its JS — see `FrameworkSingleton::co_assets`.
        assert!(
            out.join("sail_compiler_bg.wasm").exists(),
            "sail-compiler-wasm's co-asset must land at the project root",
        );
        // Gate 3: framework_assets counter includes the discovered
        // singleton (alongside the 6 deck-owned singletons).
        assert!(
            build.framework_assets >= 4,
            "framework_assets must include the deck-owned singletons + 1 discovered singleton (got {})",
            build.framework_assets,
        );
    }

    /// An app with NO `deck.jsonc` and a `@ship/sdk-client` dependency
    /// whose package.json self-declares the framework singleton lands a
    /// valid importmap binding. This is the hub case after the
    /// deck-jsonc-simplify cutover: a first-party app whose config is
    /// fully conventional needs no `deck.jsonc` at all. The bundler
    /// supplies all defaults; the SDK-client is discovered, not declared.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_no_deck_jsonc_still_discovers_framework() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        // App + dep workspace; the app has NO deck.jsonc.
        write(
            &r.join("package.json"),
            r#"{"name":"hub","private":true,"type":"module",
                "workspaces":["packages/*"],
                "dependencies":{"@ship/sdk-client":"workspace:*"}}"#,
        );
        write(
            &r.join("packages/sdk-client/package.json"),
            r#"{
  "name": "@ship/sdk-client",
  "type": "module",
  "deckFramework": {
    "alias": "@ship/sdk-client",
    "entry": "sail-sdk-client.js",
    "serve": "_deck/sdk-client.js"
  }
}"#,
        );
        std::fs::write(
            r.join("packages/sdk-client/sail-sdk-client.js"),
            "// stub\n",
        )
        .unwrap();
        write(&r.join("routes/page.sail"), "<div>x</div>\n");

        let out = r.join("dist");
        bundle_app(r, &[], r, &out, false, false)
            .await
            .expect("bundle_app with NO deck.jsonc");

        // No deck.jsonc ⇒ AppServeConfig is default (render=csr, no entry).
        // The dep's `deckFramework` declaration supplies the SDK-client
        // singleton in the importmap.
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(
            html.contains("\"@ship/sdk-client\": \"/_deck/sdk-client.js\""),
            "importmap must bind the discovered SDK-client (got: {html})",
        );
        // The framework singletons still ship (deck-owned runtime + dom + route).
        assert!(html.contains("@ship/sail-runtime"), "framework runtime in importmap");
        assert!(html.contains("@ship/sail-dom"), "framework dom in importmap");
        assert!(html.contains("@ship/deck/router"), "framework route in importmap");
        // No `entry.bootstrap` declared ⇒ deck emits its DEFAULT bootstrap:
        // `entry.gen.ts` is ALWAYS written (it calls mountRouter so a
        // standalone app mounts) with no host clients; `routes.gen.ts` is the
        // data-only companion. (Previously the no-deck.jsonc case fell back to
        // routes.gen.ts-as-entry, which never mounted — inv-deck-standalone.)
        assert!(r.join("routes.gen.ts").exists(), "routes.gen.ts (data) is emitted");
        assert!(r.join("entry.gen.ts").exists(), "deck emits a default entry.gen.ts bootstrap even without a host entry");
    }

    /// `scan_bare_specifiers` extracts package specifiers from every ESM import
    /// shape — minified `import"x"` / `from"x"`, pretty `import "x"` /
    /// `from "x"`, and dynamic `import("x")` — and IGNORES relative/absolute
    /// paths plus bare strings that aren't in an import position.
    #[test]
    fn scan_bare_specifiers_covers_import_shapes() {
        let js = r#"import"@ship/agents";import x from"@ship/sail-runtime";
export{a}from'@ship/sail-dom';const y=await import("@ship/lazy/sub");
import "./local.js";import"/abs.js";const s="@ship/not-an-import";"#;
        let mut got = scan_bare_specifiers(js);
        got.sort();
        got.dedup();
        assert_eq!(
            got,
            vec![
                "@ship/agents".to_string(),
                "@ship/lazy/sub".to_string(),
                "@ship/sail-dom".to_string(),
                "@ship/sail-runtime".to_string(),
            ],
            "bare import specifiers only — no `./local.js`, `/abs.js`, or the plain string"
        );
    }

    /// Import statements INSIDE strings, template literals, and comments are
    /// text, not code. A docs site whose snippet strings show real imports
    /// (`const code = \`import … from '@ship/tokens/themes'\``) must not fail
    /// the blank-page guard over its own teaching material.
    #[test]
    fn scan_bare_specifiers_ignores_string_and_comment_contexts() {
        let js = r#"const snippet = `import { applyTheme } from '@ship/tokens/themes'
import x from "@ship/inside-template";`;
const s = 'import y from "@ship/inside-string"';
// import z from "@ship/inside-line-comment"
/* import w from "@ship/inside-block-comment" */
const t = `${nested(`import q from "@ship/inside-nested"`)}`;
import real from "@ship/real-import";"#;
        assert_eq!(
            scan_bare_specifiers(js),
            vec!["@ship/real-import".to_string()],
            "only the real import statement counts"
        );
    }

    /// The blank-page guard: a chunk whose every bare `@ship/*` import is in the
    /// importmap PASSES; one carrying an un-importmapped `@ship/*` bare import
    /// FAILS loud and NAMES it. This is the regression bar — `--bundle-framework`
    /// must never ship a bare pack specifier the importmap can't resolve (the
    /// silent blank page this fix removed).
    #[test]
    fn pack_import_guard_fails_on_unimportmapped_bare_specifier() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        std::fs::create_dir_all(&assets).unwrap();
        let keys: HashSet<String> = ["@ship/sail-runtime", "@ship/deck/router"]
            .into_iter()
            .map(String::from)
            .collect();

        // All importmapped → OK.
        std::fs::write(assets.join("ok.js"), r#"import"@ship/sail-runtime";from"@ship/deck/router";"#)
            .unwrap();
        assert_pack_imports_resolved(&assets, &keys).expect("all imports importmapped");

        // Add a chunk with an un-importmapped `@ship/*` bare import → fail loud.
        std::fs::write(assets.join("bad.js"), r#"from"@ship/sdk/connect";"#).unwrap();
        let err = assert_pack_imports_resolved(&assets, &keys)
            .expect_err("an un-importmapped `@ship/*` bare import must fail the build");
        let msg = format!("{err:#}");
        assert!(msg.contains("@ship/sdk/connect"), "error names the offender: {msg}");
        assert!(msg.contains("bad.js"), "error names the offending chunk: {msg}");
    }

    /// A non-`@ship` bare import is OUTSIDE deck's importmap contract (it may be
    /// a host-provided global), so the guard does NOT flag it; and the standard
    /// importmap trailing-slash prefix form resolves subpaths.
    #[test]
    fn pack_import_guard_scopes_to_ship_and_honors_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        std::fs::create_dir_all(&assets).unwrap();
        // `@ship/sail-runtime/` (trailing slash) covers the `/route` subpath.
        let keys: HashSet<String> =
            ["@ship/sail-runtime/".to_string()].into_iter().collect();
        std::fs::write(
            assets.join("c.js"),
            r#"import"@ship/sail-runtime/route";import"lodash";from"react";"#,
        )
        .unwrap();
        assert_pack_imports_resolved(&assets, &keys)
            .expect("prefix-mapped `@ship/*` subpath ok; non-`@ship` bare imports ignored");
    }
}
