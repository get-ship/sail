//! Workspace discovery for the catalog aggregator (#163 catalog port).
//!
//! Port of `@ship/sail-catalog`'s `workspace.ts`: read the root `package.json`
//! `workspaces.packages` globs, expand each (single trailing `*` only), and
//! return every package dir with a parsable `package.json` that has a `name`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct WorkspacePackage {
    /// npm name (`@ship/charts`).
    pub name: String,
    /// Absolute path to the package root.
    pub root: PathBuf,
    /// `package.json.private === true`.
    pub private: bool,
    /// `package.json.customElements` (relative to root), if any.
    pub custom_elements: Option<String>,
    /// `package.json.deck.bundles[]` — consumer bundles this package declares
    /// for itself (the dissolved `deck.jsonc targets`). Empty for packages that
    /// declare none. The package OWNS its bundle definition; the bundler
    /// discovers it (see [`discover_bundle_declarations`]).
    pub bundles: Vec<BundleDecl>,
}

/// One consumer-bundle self-declaration, read from a package's `package.json`
/// `deck.bundles[]`. Replaces the central `deck.jsonc` `TargetDecl`: the package
/// that owns the entry declares the bundle, and the bundler discovers it the
/// same way it discovers `views/*`. `entry` is relative to the declaring
/// package's root (not the repo root).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleDecl {
    /// Logical name + output basename (`<name>.js`), e.g. `sail-marketing`.
    pub name: String,
    /// Entry module, relative to the declaring package's root.
    pub entry: PathBuf,
    /// Module specifiers kept external (not bundled). The build phase
    /// additionally forces `@ship/sail-runtime` external for every target.
    #[serde(default)]
    pub externals: Vec<String>,
    /// Eager (false) vs lazy (true) intent. Both bundle identically; `lazy`
    /// only gates the optional dev-loop skip (`DECK_DEV_SKIP_LAZY=1`) that omits
    /// heavy on-demand packs (pixi/pdfjs/codemirror) from a dev bake. Unset env
    /// (the default + every production bake) builds eager and lazy alike, so
    /// output is identical regardless of this flag.
    #[serde(default)]
    pub lazy: bool,
}

pub(super) fn read_json(path: &Path) -> Option<Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// The `workspaces.packages` glob list (array or `{ packages: [...] }` form).
fn workspace_globs(repo_root: &Path) -> Vec<String> {
    let Some(manifest) = read_json(&repo_root.join("package.json")) else {
        return Vec::new();
    };
    let ws = &manifest["workspaces"];
    if let Some(arr) = ws.as_array() {
        return arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    }
    ws["packages"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// Expand one glob against the repo root. Supports a single trailing `*`
/// (`packages/*`); literal paths return `[self]`. A stricter pattern (`pkg-*`)
/// is an explicit error so the aggregator never silently drops packages.
fn expand_glob(repo_root: &Path, glob: &str) -> Result<Vec<PathBuf>, String> {
    if !glob.contains('*') {
        let abs = repo_root.join(glob);
        return Ok(if abs.exists() { vec![abs] } else { vec![] });
    }
    let (parent_rel, pattern) = match glob.rfind('/') {
        Some(i) => (&glob[..i], &glob[i + 1..]),
        None => (".", glob),
    };
    if pattern != "*" {
        return Err(format!(
            "workspace glob \"{glob}\" uses unsupported pattern \"{pattern}\""
        ));
    }
    let parent_abs = repo_root.join(parent_rel);
    let Ok(entries) = std::fs::read_dir(&parent_abs) else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if e.path().is_dir() {
            out.push(e.path());
        }
    }
    Ok(out)
}

/// Walk `workspaces.packages` and return every package dir with a parsable
/// `package.json` carrying a `name`. Sorted by name (byte-stable).
///
/// GLOBAL discovery — every workspace pack is returned. For SCOPED discovery
/// (per-project component access control) see [`discover_workspace_packages_scoped`].
pub fn discover_workspace_packages(repo_root: &Path) -> Result<Vec<WorkspacePackage>, String> {
    discover_workspace_packages_scoped(repo_root, None)
}

/// Discover workspace packages, optionally SCOPED to a declared set.
///
/// * `packs == None` (or empty): GLOBAL — every workspace pack is returned,
///   identical to [`discover_workspace_packages`]. Zero behavior change for
///   existing projects.
/// * `packs == Some([..non-empty..])`: SCOPED — only packages whose
///   `package.json` `name` is in the declared set are returned. A declared pack
///   that doesn't resolve to a discovered package is silently absent; its tags
///   never enter the resolver index. This is the access-control mechanic:
///   undeclared packs' tags become `@sail-tag` no-ops (and the bundler emits an
///   author diagnostic naming them — see `catalog::shadowed_tags`).
///
/// The scope is matched by workspace `name` — a stand-in for a registry lookup
/// (installed-version resolution lands with the registry; the scoping mechanic
/// is identical either way).
pub fn discover_workspace_packages_scoped(
    repo_root: &Path,
    packs: Option<&[String]>,
) -> Result<Vec<WorkspacePackage>, String> {
    // An empty declared set is treated as "no scope" → global behavior.
    let scope: Option<std::collections::HashSet<&str>> = match packs {
        Some(p) if !p.is_empty() => Some(p.iter().map(String::as_str).collect()),
        _ => None,
    };

    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for g in workspace_globs(repo_root) {
        for dir in expand_glob(repo_root, &g)? {
            if !seen.insert(dir.clone()) {
                continue;
            }
            let Some(manifest) = read_json(&dir.join("package.json")) else {
                continue;
            };
            let Some(name) = manifest["name"].as_str() else {
                continue;
            };
            // Scope gate: with a declared set present, drop any pack not in it.
            if let Some(scope) = &scope {
                if !scope.contains(name) {
                    continue;
                }
            }
            let bundles = manifest
                .get("deck")
                .and_then(|d| d.get("bundles"))
                .and_then(|b| serde_json::from_value::<Vec<BundleDecl>>(b.clone()).ok())
                .unwrap_or_default();
            out.push(WorkspacePackage {
                name: name.to_string(),
                root: dir,
                private: manifest["private"].as_bool() == Some(true),
                custom_elements: manifest["customElements"].as_str().map(String::from),
                bundles,
            });
        }
    }
    // Component packages imported as DEPENDENCIES (file: link or node_modules) —
    // the "import components from another place" path: an app consumes a
    // component library it declares as a dependency, resolved like any package
    // (file: now, marketplace/CDN+install later), not only from the local
    // workspace. Additive: deduped by name against workspace members and
    // scope-gated, so in-monorepo consumers are unaffected.
    let ws_names: std::collections::HashSet<String> = out.iter().map(|p| p.name.clone()).collect();
    for pkg in discover_dependency_packages(repo_root) {
        if ws_names.contains(&pkg.name) {
            continue;
        }
        if let Some(scope) = &scope {
            if !scope.contains(pkg.name.as_str()) {
                continue;
            }
        }
        out.push(pkg);
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Resolve a dependency spec to its package dir: `file:<path>` (relative to
/// `repo_root`) or `<repo_root>/node_modules/<name>`.
fn resolve_dependency_dir(repo_root: &Path, name: &str, spec: &str) -> Option<PathBuf> {
    let dir = match spec.strip_prefix("file:") {
        Some(p) => repo_root.join(p),
        None => repo_root.join("node_modules").join(name),
    };
    let dir = dir.canonicalize().ok()?;
    dir.join("package.json").exists().then_some(dir)
}

/// Discover component packages declared as dependencies. Only deps carrying a
/// `package.json#customElements` marker count — that marker is what makes a
/// package a component library. The app declares the dependency; the bundler
/// resolves it (file: link now; a registry/CDN install populates the same
/// node_modules later).
fn discover_dependency_packages(repo_root: &Path) -> Vec<WorkspacePackage> {
    let Some(manifest) = read_json(&repo_root.join("package.json")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for field in ["dependencies", "devDependencies"] {
        let Some(deps) = manifest.get(field).and_then(|d| d.as_object()) else {
            continue;
        };
        for (name, spec) in deps {
            let Some(spec) = spec.as_str() else { continue };
            let Some(dir) = resolve_dependency_dir(repo_root, name, spec) else { continue };
            let Some(pkg) = read_json(&dir.join("package.json")) else { continue };
            let Some(ce) = pkg["customElements"].as_str() else { continue };
            let bundles = pkg
                .get("deck")
                .and_then(|d| d.get("bundles"))
                .and_then(|b| serde_json::from_value::<Vec<BundleDecl>>(b.clone()).ok())
                .unwrap_or_default();
            out.push(WorkspacePackage {
                name: pkg["name"].as_str().unwrap_or(name).to_string(),
                root: dir,
                private: pkg["private"].as_bool() == Some(true),
                custom_elements: Some(ce.to_string()),
                bundles,
            });
        }
    }
    out
}

/// Every consumer-bundle self-declaration in the workspace, paired with its
/// declaring package's absolute root (to resolve the package-relative `entry`).
/// This is the discovery half of the dissolved `deck.jsonc targets`: instead of
/// a central registry, each package declares its bundles and the bundler walks
/// the workspace to find them — the same model used for `views/*`. Order follows
/// `discover_workspace_packages` (byte-stable, sorted by package name).
pub fn discover_bundle_declarations(repo_root: &Path) -> Vec<(PathBuf, BundleDecl)> {
    discover_workspace_packages(repo_root)
        .unwrap_or_default()
        .into_iter()
        .flat_map(|p| {
            let root = p.root;
            p.bundles.into_iter().map(move |b| (root.clone(), b))
        })
        .collect()
}

/// Sui's own workspace root — three parents up from this crate's
/// `CARGO_MANIFEST_DIR` (`<sui>/crates/deck/deck-bundler` → `<sui>`), validated
/// once at first access. Sui carved the framework `@ship/*` packages out of the
/// consuming project (`@ship/tokens`, `@ship/sail-*`, the `packages/components/
/// sui/*` packs); a consuming project depends on them via `file:` deps, so they
/// no longer live under the project's `repo_root` and a `repo_root`-only
/// workspace walk misses them. Resolving sui's root from the build crate's own
/// location lets the bundler union those packages into the `@ship/<name>` →
/// source map regardless of which project it is building.
///
/// `None` when the derived path doesn't look like sui's workspace (e.g. the
/// crate was relocated, or a prod build runs from an install layout) — callers
/// fall back to project-only resolution and never panic.
fn sui_workspace_root() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for _ in 0..3 {
            if !d.pop() {
                return None;
            }
        }
        // Confirm it's sui's workspace: a `package.json` + `packages/` dir,
        // AND it actually declares the carved `@ship/tokens` package. The
        // tokens check is the cheap, specific guard that this is sui (not some
        // other ancestor workspace) without a full discovery walk.
        if d.join("package.json").exists()
            && d.join("packages/sail/tokens/package.json").exists()
        {
            Some(d)
        } else {
            None
        }
    })
    .as_deref()
}

/// npm package name → absolute package root, for resolving `@ship/<name>`
/// component imports to workspace source. This is the bun/npm workspace package
/// map — `@ship/<name>` already resolves to its package dir via `workspaces` —
/// surfaced for the bundler's resolver (`SailPlugin::component_roots`). It
/// REPLACES the hand-maintained `componentSourceRoots` search order: in dev the
/// map is the source of truth; in prod, standard package resolution
/// (node_modules) applies and this map is simply empty.
///
/// The map UNIONS two workspaces:
///   1. the consuming project's `repo_root` (its own `@ship/*` — `@ship/sdk`,
///      `@ship/agents`, the `packages/components/ship/*` packs, etc.);
///   2. sui's own workspace ([`sui_workspace_root`]) — the carved framework
///      packages (`@ship/tokens`, `@ship/sail-*`, `packages/components/sail/*`).
///
/// PRECEDENCE on a name collision: **sui wins**. Sui owns the framework `@ship/*`
/// names; after the carve they live only under sui's root, so a stale or absent
/// entry under the project's root must never shadow the real sui source. Sui's
/// entries are therefore overlaid AFTER the project's, overwriting on collision.
/// (Today the two sets are disjoint — project owns `@ship/sdk`/`@ship/agents`/…,
/// sui owns `@ship/tokens`/`@ship/sail-*` — so this rule only ever fires
/// defensively.) If sui's root can't be derived the union degrades to
/// project-only resolution.
///
/// Cached per `repo_root` (shared `Arc`, mirroring `component_index`) so the
/// per-build plugin construction is a cheap clone, not a fresh workspace walk.
pub fn component_package_roots(repo_root: &Path) -> Arc<HashMap<String, PathBuf>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<HashMap<String, PathBuf>>>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    {
        let guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(map) = guard.get(repo_root) {
            return map.clone();
        }
    }
    // Project workspace first.
    let mut roots: HashMap<String, PathBuf> = discover_workspace_packages(repo_root)
        .unwrap_or_default()
        .into_iter()
        .map(|p| (p.name, p.root))
        .collect();
    // Overlay the ENCLOSING JS workspace (the bun/npm workspace root above the
    // project). A pure-JS consumer (ship-apps) is built with `repo_root` set to
    // the APP subdir (`ship-apps/apps/helm`) — not the workspace root — so the
    // project-only walk above sees only the app's own declared deps and MISSES
    // its SIBLING workspace packs (`@ship/agents`, …). Without those in the map,
    // `SailPlugin` can't re-anchor a bare `import "@ship/agents"` to source, it
    // falls through to a bare external the importmap never covers, and the app
    // renders blank. Resolve the enclosing workspace root and union its packages
    // (skip when it IS the project root — the sui monorepo, where they coincide).
    if let Some(ws_root) = crate::paths::topmost_workspace_root(repo_root) {
        if ws_root != repo_root {
            for p in discover_workspace_packages(&ws_root).unwrap_or_default() {
                roots.insert(p.name, p.root);
            }
        }
    }
    // Overlay sui's own workspace (sui wins on collision). Skip when sui IS the
    // project root — its packages are already in `roots`, and re-walking would
    // only be redundant work for an identical result.
    if let Some(sui_root) = sui_workspace_root() {
        if sui_root != repo_root {
            for p in discover_workspace_packages(sui_root).unwrap_or_default() {
                roots.insert(p.name, p.root);
            }
        }
    }
    let map = Arc::new(roots);
    cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(repo_root.to_path_buf(), map.clone());
    map
}

/// npm name → daemon pack-id. `@ship/charts` → `charts`; `widgets` → `widgets`.
pub fn pack_id_for_name(npm_name: &str) -> String {
    if let Some(rest) = npm_name.strip_prefix('@') {
        if let Some(slash) = rest.find('/') {
            return rest[slash + 1..].to_string();
        }
    }
    npm_name.to_string()
}

/// Recursively walk `dir`, collecting `(tag, abs_path)` pairs for component
/// `.sui` files. A `.sui` is a component iff its stem contains a hyphen and is
/// neither "page" nor "index" — the HTML custom-element tag-name rule (HTML spec
/// §4.13.3 requires a hyphen). Views use `page.sui` or `index.sui` and are
/// excluded by this guard. `dist`, `node_modules`, `.git`, and `target` are
/// skipped for speed.
fn scan_sui_components(dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    const SKIP: [&str; 5] = ["node_modules", "dist", "target", ".git", deck_config::DECK_OUT_DIRNAME];
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if SKIP.contains(&name.as_str()) {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            scan_sui_components(&p, out);
        } else if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            if p.extension().and_then(|x| x.to_str()) != Some("sui") {
                continue;
            }
            // Guard: stem must contain a hyphen and not be "page" or "index".
            if stem.contains('-') && stem != "page" && stem != "index" {
                out.push((stem.to_string(), p));
            }
        }
    }
}

/// Return all `(tag, abs_path)` component pairs discovered by source scan of
/// `pkg_root/src/`. Only `.sui` files whose stem contains a hyphen and is not
/// "page" or "index" are returned; views and other files are excluded by the
/// guard.
pub fn source_tags_for_package(pkg_root: &Path) -> Vec<(String, PathBuf)> {
    let src = pkg_root.join("src");
    if !src.is_dir() {
        return vec![];
    }
    let mut out = Vec::new();
    scan_sui_components(&src, &mut out);
    out
}

/// Build a `<tag>` → source `.sui` index from a registry-composed root's
/// `components/` tree — the per-space component layer.
///
/// This is the monorepo-decoupled discovery: instead of reading the root
/// `package.json` workspaces, scan the `components/` dir of a space's composed
/// root (`ship_platform::registry::compose::composed_root_for`), which holds
/// each declared dep's components namespaced as
/// `components/<pkg.path>/<tag>/<tag>.sui`. The same custom-element hyphen rule
/// applies (component iff hyphenated stem ∉ {page,index}), so views never leak.
/// First tag wins on collision (byte-stable: WalkDir is lexical).
pub fn composed_component_index(composed_root: &Path) -> std::collections::HashMap<String, PathBuf> {
    let mut out = std::collections::HashMap::new();
    let components = composed_root.join("components");
    if !components.is_dir() {
        return out;
    }
    let mut pairs = Vec::new();
    scan_sui_components(&components, &mut pairs);
    for (tag, path) in pairs {
        out.entry(tag).or_insert(path);
    }
    out
}
