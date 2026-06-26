//! Component catalog aggregation.
//!
//! Walk per-component CEMs from published packages (each package's
//! `customElements` manifest) AND from `.ship/sui-out/**/*.custom-elements.json`
//! (non-packaged: shell-local, view-local), merge into the canonical workspace
//! catalog document. Filesystem-only, deterministic (sort by tagName).
//! Pass-through CEM arrays are kept as raw JSON so component metadata isn't
//! lost to a typed schema.

mod workspace;

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

pub use workspace::{
    BundleDecl, WorkspacePackage, component_package_roots, composed_component_index,
    discover_bundle_declarations, discover_workspace_packages,
    discover_workspace_packages_scoped, pack_id_for_name,
};
use workspace::{read_json, source_tags_for_package};

const CEM_SUFFIX: &str = ".custom-elements.json";
const SKIP_DIRS: [&str; 4] = ["node_modules", "dist", "target", ".git"];

/// One catalog entry. Pass-through CEM arrays are raw `Value`.
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct CatalogEntry {
    pub tagName: String,
    pub description: String,
    pub firstDesc: String,
    pub attributes: Value,
    pub slots: Value,
    pub events: Value,
    pub methods: Value,
    pub cssProperties: Value,
    pub cssParts: Value,
    pub examples: Value,
    pub package: String,
    pub private: bool,
    pub sourcePath: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct CatalogDocument {
    pub schemaVersion: &'static str,
    pub count: usize,
    pub entries: Vec<CatalogEntry>,
}

pub fn stringify_catalog(doc: &CatalogDocument) -> String {
    serde_json::to_string_pretty(doc).unwrap_or_default() + "\n"
}

/// Build the workspace catalog for a repo — every component, sorted by tagName.
///
/// `` is the optional embedded host app (e.g. `apps/shell` →
/// `app:shell` from `deck.jsonc`). When `None`, the shell-local CEM projection
/// does not apply — suitable for zero-config hosts.
pub fn build_catalog(
    repo_root: &Path,
) -> Result<CatalogDocument, String> {
    build_catalog_scoped(repo_root, None)
}

/// Scoped variant of [`build_catalog`]. When `packs` is a non-empty declared
/// set, only those packs' components are aggregated; `None`/empty preserves the
/// global behavior.
pub fn build_catalog_scoped(
    repo_root: &Path,
    packs: Option<&[String]>,
) -> Result<CatalogDocument, String> {
    let packages = discover_workspace_packages_scoped(repo_root, packs)?;

    let mut all: Vec<CatalogEntry> = Vec::new();
    for pkg in &packages {
        if pkg.custom_elements.is_some() {
            all.extend(entries_from_packaged_manifest(repo_root, pkg));
        }
    }
    all.extend(entries_from_sui_out(repo_root, &packages));

    all.sort_by(|a, b| a.tagName.cmp(&b.tagName));
    // Dedup by tagName, first wins (packaged entries precede sui-out ones).
    let mut seen = std::collections::HashSet::new();
    all.retain(|e| seen.insert(e.tagName.clone()));

    Ok(CatalogDocument {
        schemaVersion: "1",
        count: all.len(),
        entries: all,
    })
}

/// Map every workspace component tag → its source `.sui` path (absolute), for
/// the per-view resolver: the bundler resolves `@sui-tag/<tag>` to this path so
/// the component compiles into the view bundle.
///
/// SOURCE-MODE (primary): for every workspace package, recursively scan
/// `<pkg>/src/` for `.sui` files whose stem contains a hyphen and is not
/// "page" or "index" (the HTML custom-element tag-name rule). This is the
/// authoritative component convention and requires no pre-built CEM. Views
/// (`page.sui`, `index.sui`) are excluded by the guard and never leak into the
/// resolver index.
///
/// CEM-MODE (secondary): for any tag NOT already found via source scan, fall
/// back to the CEM-derived path from `build_catalog` (useful for packages that
/// only ship a dist CEM with no `.sui` source in the repo). In practice,
/// the monorepo always has `.sui` source, so CEM only adds anything if a
/// package genuinely has no local source.
///
/// A tag whose source can't be located (neither source scan nor CEM) is absent;
/// the resolver then falls back to external.
pub fn tag_source_index(
    repo_root: &Path,
) -> std::collections::HashMap<String, PathBuf> {
    tag_source_index_scoped(repo_root, None)
}

/// Scoped variant of [`tag_source_index`] — per-project component access control.
///
/// When `packs` declares a non-empty set, the index is built ONLY from those
/// packs (both the source scan and the CEM fallback are scoped). A tag provided
/// solely by an undeclared pack is therefore absent → the resolver emits a
/// `@sui-tag` no-op for it (the component never bundles), and the bundler warns
/// the author (see [`shadowed_tags`]). When `packs` is `None`/empty the behavior
/// is identical to [`tag_source_index`] (global).
pub fn tag_source_index_scoped(
    repo_root: &Path,
    packs: Option<&[String]>,
) -> std::collections::HashMap<String, PathBuf> {
    let mut out = std::collections::HashMap::new();

    // ── the project's OWN components ──────────────────────────────────────
    // An app (the hub) is built with `repo_root` set to its own dir, which is
    // NOT always one of its `workspaces` entries — so the workspace scan below
    // misses the app's local components. Scan the project root's own `src/`
    // unconditionally (never scope-gated: the app always owns its own tags).
    // This is what makes the `workspaces: ["."]` self-listing hack unnecessary.
    // For a monorepo root (sui, ship-apps) there is no `<root>/src` → no-op.
    for (tag, abs_path) in source_tags_for_package(repo_root) {
        if abs_path.is_file() {
            out.entry(tag).or_insert(abs_path);
        }
    }

    // ── primary: source scan of every (declared) workspace package ────────
    if let Ok(packages) = discover_workspace_packages_scoped(repo_root, packs) {
        for pkg in &packages {
            for (tag, abs_path) in source_tags_for_package(&pkg.root) {
                if abs_path.is_file() {
                    out.entry(tag).or_insert(abs_path);
                }
            }
        }
    }

    // ── secondary: CEM-derived paths for anything source scan missed ───────
    if let Ok(workspace) = build_catalog_scoped(repo_root, packs) {
        for e in workspace.entries {
            if out.contains_key(&e.tagName) {
                continue; // source-mode already resolved this tag
            }
            if !e.sourcePath.ends_with(".sui") {
                continue;
            }
            let abs = repo_root.join(&e.sourcePath);
            if abs.is_file() {
                out.insert(e.tagName, abs);
            }
        }
    }

    out
}

/// Global map: every workspace component tag → its owning pack's `package.json`
/// `name`. First pack to provide a tag wins (discovery is name-sorted, stable).
/// Source-scan only — the authoritative component convention — so a tag's pack
/// attribution matches exactly what a declared scope filters on.
pub fn tag_pack_index(repo_root: &Path) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    if let Ok(packages) = discover_workspace_packages(repo_root) {
        for pkg in &packages {
            for (tag, _path) in source_tags_for_package(&pkg.root) {
                out.entry(tag).or_insert_with(|| pkg.name.clone());
            }
        }
    }
    out
}

/// Tags a declared-`packs` scope EXCLUDES → `tag → owning-pack name`.
///
/// Every workspace component tag whose owning pack is not in the resolved scope.
/// Empty when `packs` is empty (global scope excludes nothing). This powers the
/// author diagnostic the bundler emits when a built view references a scoped-out
/// tag: "`<tag>` is provided by `<pack>`, which you didn't declare."
pub fn shadowed_tags(
    repo_root: &Path,
    packs: &[String],
) -> std::collections::HashMap<String, String> {
    if packs.is_empty() {
        return std::collections::HashMap::new();
    }
    let scoped = tag_source_index_scoped(repo_root, Some(packs));
    tag_pack_index(repo_root)
        .into_iter()
        .filter(|(tag, _)| !scoped.contains_key(tag))
        .collect()
}

fn arr_or_empty(v: &Value) -> Value {
    if v.is_array() { v.clone() } else { json!([]) }
}

/// Build an entry from a CEM declaration. `None` unless it's a named custom
/// element (`tagName` + `customElement: true`).
fn entry_from_decl(
    decl: &Value,
    package: String,
    private: bool,
    source_path: String,
) -> Option<CatalogEntry> {
    let tag = decl["tagName"].as_str().filter(|s| !s.is_empty())?;
    if decl["customElement"].as_bool() != Some(true) {
        return None;
    }
    let description = decl["description"].as_str().unwrap_or("").trim().to_string();
    let first_desc = description.split('\n').next().unwrap_or("").to_string();
    Some(CatalogEntry {
        tagName: tag.to_string(),
        description,
        firstDesc: first_desc,
        attributes: arr_or_empty(&decl["attributes"]),
        slots: arr_or_empty(&decl["slots"]),
        events: arr_or_empty(&decl["events"]),
        methods: arr_or_empty(&decl["members"]),
        cssProperties: arr_or_empty(&decl["cssProperties"]),
        cssParts: arr_or_empty(&decl["cssParts"]),
        examples: arr_or_empty(&decl["examples"]),
        package,
        private,
        sourcePath: source_path,
    })
}

fn entries_from_packaged_manifest(repo_root: &Path, pkg: &WorkspacePackage) -> Vec<CatalogEntry> {
    let Some(ce) = &pkg.custom_elements else { return vec![] };
    let manifest_abs = pkg.root.join(ce);
    if !manifest_abs.exists() {
        return vec![];
    }
    let Some(cem) = read_json(&manifest_abs) else { return vec![] };
    let Some(modules) = cem["modules"].as_array() else { return vec![] };
    let pkg_rel_root = rel_unix(repo_root, &pkg.root);

    let mut out = Vec::new();
    for m in modules {
        let mod_path = m["path"].as_str().unwrap_or("");
        let stem = mod_path.strip_suffix(".sui").unwrap_or(mod_path);
        let source_path = if !stem.is_empty() {
            format!("{pkg_rel_root}/src/{stem}/{stem}.sui")
        } else {
            format!("{pkg_rel_root}/src/{mod_path}")
        };
        if let Some(decls) = m["declarations"].as_array() {
            for decl in decls {
                if let Some(e) = entry_from_decl(
                    decl,
                    pkg.name.clone(),
                    pkg.private,
                    source_path.clone(),
                ) {
                    out.push(e);
                }
            }
        }
    }
    out
}

fn entries_from_sui_out(
    repo_root: &Path,
    packages: &[WorkspacePackage],
) -> Vec<CatalogEntry> {
    let root = repo_root.join(sui_compiler::out_layout::sui_out_dir());
    if !root.exists() {
        return vec![];
    }
    let mut cem_files = Vec::new();
    walk_cem_files(&root, &mut cem_files);
    cem_files.sort();

    // Owning-package lookup, longest rel path first so nested packages win.
    let mut packaged_roots: Vec<(String, &WorkspacePackage)> =
        packages.iter().map(|p| (rel_unix(repo_root, &p.root), p)).collect();
    packaged_roots.sort_by_key(|(rel, _)| std::cmp::Reverse(rel.len()));

    // The first-party app projection was an apps/shell-specific
    // affordance (shell-local CEM); apps/hub and apps/spaces are each
    // their own per-app project and use the per-tag resolver's workspace
    // catalog instead. The projection is gone — every pack on disk is
    // discovered via the workspace walker below.
    let mut out = Vec::new();
    for cem_abs in &cem_files {
        let source_path = source_path_for_cem(repo_root, cem_abs);
        let owning = find_owning_package(&source_path, &packaged_roots);
        // Owning pack with a resolvable aggregated manifest → served from there;
        // skip. When the manifest path is declared in `package.json` but the
        // file is absent (the pack hasn't been built), fall through to the
        // `.ship/sui-out/` path — that's what we just compiled.
        if let Some((_, p)) = owning {
            if let Some(ce) = &p.custom_elements {
                if p.root.join(ce).exists() {
                    continue;
                }
            }
        }
        let Some(cem) = read_json(cem_abs) else { continue };
        let Some(modules) = cem["modules"].as_array() else { continue };

        // Each component's origin is the package that owns its source —
        // the per-tag resolver walks `packages/<name>` and tags every
        // discovered tag with the owning package. No first-party
        // override (the apps/shell-style projection is gone).
        let origin = owning
            .map(|(_, p)| p.name.clone())
            .unwrap_or_else(|| origin_for_non_packaged_source(&source_path));
        let is_private = owning.map(|(_, p)| p.private).unwrap_or(true);
        for m in modules {
            if let Some(decls) = m["declarations"].as_array() {
                for decl in decls {
                    if let Some(e) = entry_from_decl(
                        decl,
                        origin.clone(),
                        is_private,
                        source_path.clone(),
                    ) {
                        out.push(e);
                    }
                }
            }
        }
    }
    out
}

fn walk_cem_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            walk_cem_files(&p, out);
        } else if name.ends_with(CEM_SUFFIX) {
            out.push(p);
        }
    }
}

fn find_owning_package<'a>(
    rel: &str,
    packaged_roots: &'a [(String, &'a WorkspacePackage)],
) -> Option<&'a (String, &'a WorkspacePackage)> {
    packaged_roots
        .iter()
        .find(|(r, _)| rel == r || rel.starts_with(&format!("{r}/")))
}

/// Repo-relative source `.sui` path for a per-component CEM under
/// `.ship/sui-out/<mirror>/<stem>.custom-elements.json`.
fn source_path_for_cem(repo_root: &Path, cem_abs: &Path) -> String {
    let repo_rel = rel_unix(repo_root, cem_abs);
    let mirrored = repo_rel
        .strip_prefix(&format!("{}/", sui_compiler::out_layout::sui_out_dir()))
        .unwrap_or(&repo_rel);
    let file = mirrored.rsplit('/').next().unwrap_or(mirrored);
    let stem = file.strip_suffix(CEM_SUFFIX).unwrap_or(file);
    let dir = &mirrored[..mirrored.len() - file.len()];
    format!("{dir}{stem}.sui")
}

/// Origin tag for a non-packaged source. `views/<name>/` → `view:<name>`;
/// else `workspace:<first-seg>`. Per-app labelling is resolved by the
/// caller from the bundler's own project root, not from a global config.
fn origin_for_non_packaged_source(norm: &str) -> String {
    if let Some(rest) = norm.strip_prefix("views/") {
        if let Some(name) = rest.split('/').next().filter(|s| !s.is_empty()) {
            return format!("view:{name}");
        }
    }
    if let Some(idx) = norm.find("assets/views/") {
        let rest = &norm[idx + "assets/views/".len()..];
        if let Some(name) = rest.split('/').next().filter(|s| !s.is_empty()) {
            return format!("view:{name}");
        }
    }
    let first = norm.split('/').next().unwrap_or("unknown");
    format!("workspace:{first}")
}

fn rel_unix(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn cem(tag: &str) -> String {
        json!({
            "schemaVersion": "1",
            "modules": [{
                "path": format!("{tag}.sui"),
                "declarations": [{
                    "kind": "class", "name": tag, "tagName": tag,
                    "customElement": true, "description": "demo"
                }]
            }]
        })
        .to_string()
    }

    #[test]
    fn aggregates_packaged_and_sui_out() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);
        // A public pack with a CEM manifest.
        write(
            &root.join("packages/charts/package.json"),
            r#"{"name":"@ship/charts","customElements":"dist/custom-elements.json"}"#,
        );
        write(&root.join("packages/charts/dist/custom-elements.json"), &cem("sui-bar"));
        // A shell-local component in the sui-out mirror.
        write(
            &root.join(".sui-out/apps/shell/src/sui-x/sui-x.custom-elements.json"),
            &cem("sui-x"),
        );

        // Build the catalog — the per-app resolver uses the bundler's
        // own project root; the L1 framework has no per-app config.
        let workspace = build_catalog(root).unwrap();

        let ws_tags: Vec<&str> = workspace.entries.iter().map(|e| e.tagName.as_str()).collect();
        assert_eq!(ws_tags, vec!["sui-bar", "sui-x"], "sorted, both present");

        let bar = workspace.entries.iter().find(|e| e.tagName == "sui-bar").unwrap();
        assert_eq!(bar.package, "@ship/charts");
        assert!(!bar.private);
        // A sui-out CEM under apps/shell falls through to the workspace
        // origin: `workspace:apps`. The catalog has no per-app projection.
        let x = workspace.entries.iter().find(|e| e.tagName == "sui-x").unwrap();
        assert_eq!(x.package, "workspace:apps");
        assert!(x.private);
    }

    #[test]
    fn aggregates_sui_out_zero_config() {
        // A sui-out CEM with no matching package falls through to
        // origin_for_non_packaged_source → workspace:apps.
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":[]}"#);
        write(
            &root.join(".sui-out/apps/shell/src/sui-x/sui-x.custom-elements.json"),
            &cem("sui-x"),
        );
        let workspace = build_catalog(root).unwrap();
        let x = workspace.entries.iter().find(|e| e.tagName == "sui-x").unwrap();
        // No matching package — origin falls back to workspace:<first-seg>.
        assert_eq!(x.package, "workspace:apps");
    }


    #[test]
    fn index_maps_tag_to_existing_source_and_omits_missing() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);
        write(
            &root.join("packages/charts/package.json"),
            r#"{"name":"@ship/charts","customElements":"dist/custom-elements.json"}"#,
        );
        write(&root.join("packages/charts/dist/custom-elements.json"), &cem("sui-bar"));
        // The CEM implies source packages/charts/src/sui-bar/sui-bar.sui — write it.
        write(&root.join("packages/charts/src/sui-bar/sui-bar.sui"), "<div>bar</div>\n");

        let idx = tag_source_index(root);
        // Invariant: an indexed tag points at an EXISTING `.sui` source.
        let src = idx.get("sui-bar").expect("a tag with a real source is indexed");
        assert!(src.is_file());
        assert_eq!(src.extension().and_then(|e| e.to_str()), Some("sui"));
        assert!(src.ends_with("packages/charts/src/sui-bar/sui-bar.sui"));
        // Invariant: an unknown tag is absent → the resolver falls back to external.
        assert!(!idx.contains_key("sui-nonexistent"));
    }

    /// Self-discovery: when the project root is built as its own app (the hub)
    /// and does NOT list itself in `workspaces`, its OWN `src/` components must
    /// still resolve. This is the fix that removes the `workspaces: ["."]` hack:
    /// the project root's `src/` is scanned unconditionally.
    #[test]
    fn tag_source_index_includes_project_own_components_without_workspace_self_listing() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        // The app does NOT list "." in workspaces (the hack is gone).
        write(&root.join("package.json"), r#"{"name":"@ship/hub","workspaces":["packages/*"]}"#);
        // A local component in the app's OWN src.
        write(&root.join("src/hub-widget/hub-widget.sui"), "<div>widget</div>\n");

        let idx = tag_source_index(root);
        let src = idx.get("hub-widget").expect("app's own local component is discovered");
        assert!(src.is_file());
        assert!(src.ends_with("src/hub-widget/hub-widget.sui"));
    }

    /// SOURCE-MODE gate: a workspace package that has a `<X>/<X>.sui` component
    /// source but NO `customElements` CEM at all must still yield tag X in the
    /// resolver index. This proves that the resolver does not require a pre-built
    /// CEM artifact to function — the `.sui` source IS the truth.
    #[test]
    fn source_mode_resolves_tag_without_cem() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);
        // Package has NO customElements field — no CEM at all.
        write(
            &root.join("packages/mycomp/package.json"),
            r#"{"name":"@ship/mycomp","private":true}"#,
        );
        // Component source: stem "sui-foo" contains a hyphen → detected as Component.
        write(
            &root.join("packages/mycomp/src/sui-foo/sui-foo.sui"),
            "<span class=\"sui-foo-marker\">foo</span>\n",
        );
        // A view alongside it must NOT appear (stem "page" has no hyphen).
        write(
            &root.join("packages/mycomp/src/myview/page.sui"),
            "<div>view</div>\n",
        );

        let idx = tag_source_index(root);

        // The component resolves from source.
        let src = idx
            .get("sui-foo")
            .expect("source-mode: tag must resolve even without a CEM");
        assert!(src.is_file(), "resolved path must be an existing file");
        assert!(
            src.ends_with("packages/mycomp/src/sui-foo/sui-foo.sui"),
            "resolved path must be the component .sui source; got: {src:?}"
        );

        // The view (stem=page, no hyphen) must NOT be indexed.
        assert!(
            !idx.contains_key("page"),
            "page.sui (view) must not leak into the resolver index"
        );
    }

    /// GROUPED-DIRECTORY gate: a package with multiple components under one
    /// parent dir (e.g. `sui-rail/sui-rail.sui`, `sui-rail/sui-rail-item.sui`)
    /// must register ALL components, not just the stem==parent file.
    #[test]
    fn source_mode_grouped_dir_registers_all_components() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);
        write(
            &root.join("packages/chrome/package.json"),
            r#"{"name":"@ship/chrome","private":true}"#,
        );
        // Three components grouped under one parent dir.
        write(
            &root.join("packages/chrome/src/sui-rail/sui-rail.sui"),
            "<nav class=\"rail\"></nav>\n",
        );
        write(
            &root.join("packages/chrome/src/sui-rail/sui-rail-item.sui"),
            "<div class=\"rail-item\"></div>\n",
        );
        write(
            &root.join("packages/chrome/src/sui-rail/sui-rail-section.sui"),
            "<div class=\"rail-section\"></div>\n",
        );
        // A view in the same package must not appear.
        write(
            &root.join("packages/chrome/src/views/home/page.sui"),
            "<div>home</div>\n",
        );

        let idx = tag_source_index(root);

        assert!(idx.contains_key("sui-rail"), "sui-rail must be indexed");
        assert!(idx.contains_key("sui-rail-item"), "sui-rail-item must be indexed");
        assert!(idx.contains_key("sui-rail-section"), "sui-rail-section must be indexed");
        assert!(!idx.contains_key("page"), "page.sui must not be indexed");
    }

    /// Three-pack fixture mirroring a real project shape:
    ///   - `@ship/sui-primitives` exports `sui-button`
    ///   - `deck-demo-app` exports `demo-chip` (the project's own component)
    ///   - `@ship/agents` exports `ship-agent-card`
    fn three_pack_fixture() -> tempfile::TempDir {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);

        write(
            &root.join("packages/primitives/package.json"),
            r#"{"name":"@ship/sui-primitives","private":false}"#,
        );
        write(
            &root.join("packages/primitives/src/sui-button/sui-button.sui"),
            "<button class=\"sui-button-marker\"><slot></slot></button>\n",
        );

        write(
            &root.join("packages/demo/package.json"),
            r#"{"name":"deck-demo-app","private":true}"#,
        );
        write(
            &root.join("packages/demo/src/demo-chip/demo-chip.sui"),
            "<span class=\"demo-chip-marker\"><slot></slot></span>\n",
        );

        write(
            &root.join("packages/agents/package.json"),
            r#"{"name":"@ship/agents","private":false}"#,
        );
        write(
            &root.join("packages/agents/src/ship-agent-card/ship-agent-card.sui"),
            "<article class=\"ship-agent-card-marker\"><slot></slot></article>\n",
        );

        tmp
    }

    /// Per-project component access control: a project that declares only the
    /// primitives pack and its own components (NOT `@ship/agents`) gets:
    ///   (a) declared-pack + own tags still resolve;
    ///   (b) the undeclared-pack tag does NOT resolve → the resolver no-ops it;
    ///   (c) global (no scope) resolves all three — before/after evidence.
    #[test]
    fn scoped_resolution_enforces_declared_pack_access() {
        let tmp = three_pack_fixture();
        let root = tmp.path();

        // (c) BEFORE — global resolution: every workspace pack's tags resolve.
        let global = tag_source_index(root);
        let mut global_tags: Vec<&str> = global.keys().map(String::as_str).collect();
        global_tags.sort_unstable();
        assert_eq!(global_tags, vec!["demo-chip", "ship-agent-card", "sui-button"]);

        // The project declares a PARTIAL pack set (deck.jsonc `packs`).
        let declared: Vec<String> = vec!["@ship/sui-primitives".into(), "deck-demo-app".into()];
        let scoped = tag_source_index_scoped(root, Some(&declared));

        // (a) declared-pack tag resolves to a real .sui source; own tag resolves.
        let btn = scoped
            .get("sui-button")
            .expect("declared primitives tag must resolve under scope");
        assert!(btn.is_file());
        assert!(btn.ends_with("packages/primitives/src/sui-button/sui-button.sui"));
        assert!(scoped.contains_key("demo-chip"), "the project's own tag resolves");

        // (b) undeclared-pack tag is ABSENT → resolver no-ops it (access denied).
        assert!(
            !scoped.contains_key("ship-agent-card"),
            "undeclared @ship/agents tag must NOT resolve → @sui-tag no-op"
        );

        // (c) AFTER — scoped index is exactly the declared two.
        let mut scoped_tags: Vec<&str> = scoped.keys().map(String::as_str).collect();
        scoped_tags.sort_unstable();
        assert_eq!(scoped_tags, vec!["demo-chip", "sui-button"]);
    }

    /// Additive guarantee: an empty declared set is treated as "no scope" →
    /// identical to global, so existing zero-`packs` projects are unaffected.
    #[test]
    fn empty_packs_is_global() {
        let tmp = three_pack_fixture();
        let root = tmp.path();
        let empty: Vec<String> = Vec::new();
        let scoped = tag_source_index_scoped(root, Some(&empty));
        let global = tag_source_index(root);
        assert_eq!(scoped.len(), global.len(), "empty packs ⇒ identical to global");
        assert!(scoped.contains_key("ship-agent-card"), "empty scope resolves all");
    }

    /// The author diagnostic's data: a scoped-out tag is attributed to the pack
    /// that provides it, so the bundler can say "`<tag>` is from `<pack>`".
    #[test]
    fn shadowed_tags_attributes_excluded_tag_to_its_pack() {
        let tmp = three_pack_fixture();
        let root = tmp.path();
        let declared: Vec<String> = vec!["@ship/sui-primitives".into(), "deck-demo-app".into()];

        let shadowed = shadowed_tags(root, &declared);
        // Only the undeclared pack's tag is shadowed — named with its pack.
        assert_eq!(
            shadowed.get("ship-agent-card").map(String::as_str),
            Some("@ship/agents"),
            "excluded tag must carry its providing pack for the diagnostic"
        );
        assert!(!shadowed.contains_key("sui-button"), "declared tag is not shadowed");
        assert!(!shadowed.contains_key("demo-chip"), "own tag is not shadowed");

        // Global scope shadows nothing (no diagnostic noise when unscoped).
        assert!(shadowed_tags(root, &[]).is_empty(), "empty packs ⇒ nothing shadowed");
    }

    /// Monorepo-decoupled discovery: scan a registry-composed root's
    /// `components/` tree (`components/<pkg.path>/<tag>/<tag>.sui`) — the shape
    /// `compose::materialize_space` produces. The dep'd component resolves; a
    /// view alongside it does not leak; a non-present tag is absent.
    #[test]
    fn composed_root_components_resolve_from_dep_namespace() {
        let tmp = tempdir().unwrap();
        let composed = tmp.path();
        // Mirror compose output: components/<pkg.path>/<tag>/<tag>.sui
        write(
            &composed.join("components/github.com/acme/agents/ship-agent-card/ship-agent-card.sui"),
            "<article><slot></slot></article>\n",
        );
        // A view that happens to sit in the tree must NOT be indexed (no hyphen).
        write(&composed.join("components/github.com/acme/agents/page.sui"), "<div>v</div>\n");

        let idx = composed_component_index(composed);
        let src = idx
            .get("ship-agent-card")
            .expect("dep'd component resolves from the composed root");
        assert!(src.is_file());
        assert!(src.ends_with("ship-agent-card/ship-agent-card.sui"));
        assert!(!idx.contains_key("page"), "view must not leak into the index");
        assert!(!idx.contains_key("sui-nope"), "non-present tag is absent");
    }

    /// A composed root with no `components/` layer (depless space) → empty index,
    /// no panic.
    #[test]
    fn composed_root_without_components_is_empty() {
        let tmp = tempdir().unwrap();
        assert!(composed_component_index(tmp.path()).is_empty());
    }

    /// Declaring a pack that doesn't exist in the workspace contributes no tags
    /// (silently absent at the discovery layer). A view that then references a
    /// real tag from an *undeclared* pack is what surfaces the author diagnostic.
    #[test]
    fn declaring_unknown_pack_yields_no_tags() {
        let tmp = three_pack_fixture();
        let root = tmp.path();
        let declared: Vec<String> = vec!["@ship/does-not-exist".into()];
        let scoped = tag_source_index_scoped(root, Some(&declared));
        assert!(scoped.is_empty(), "unknown declared pack contributes nothing");
    }
}
