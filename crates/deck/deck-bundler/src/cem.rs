//! CEM catalog integration for deck-bundler.
//!
//! B1: `write_component_package_cem` — compile all component `.sail` files in a component
//!     pack in-process and write the merged `dist/custom-elements.json`
//!     declared in `package.json`.
//! B2: `generate_components_gen` — emit `<project>/components.gen.ts` for an
//!     app from the components package module graph. Called by `bundle_app`
//!     before the Rolldown step so the generated catalog is current when the
//!     bundler reads the app graph.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::catalog::discover_workspace_packages;

/// Compile all component `.sail` files in a components package and write the merged
/// `dist/custom-elements.json` declared in its `package.json`.
///
/// - Reads `<package_root>/package.json` for `customElements` field; returns
///   `Ok(None)` if absent.
/// - Walks `src/` for `<stem>/<stem>.sail` files (component convention).
/// - Compiles each in-process via `sail_compiler::compile` with
///   `CompileMode::Component` and extracts the CEM from `CompileOutput::cem`.
/// - Calls `sail_compiler::emit::cem::merge_cem` to produce the standard
///   manifest.
/// - Writes to `<package_root>/<customElements_field_value>` (creates `dist/` if
///   needed).
/// - Returns the path written, or `None` when the components package has no `customElements`
///   field.
pub fn write_component_package_cem(package_root: &Path) -> Result<Option<PathBuf>> {
    // 1. Read package.json and get the customElements field.
    let pkg_path = package_root.join("package.json");
    let pkg_raw = std::fs::read_to_string(&pkg_path)
        .with_context(|| format!("read {}", pkg_path.display()))?;
    let pkg: Value = serde_json::from_str(&pkg_raw).context("parse package.json")?;

    let Some(ce_field) = pkg["customElements"].as_str() else {
        return Ok(None);
    };
    let ce_field = ce_field.to_string();

    // 2. Walk component .sail files: classic `src/`, plus domain `*/src/` for
    // umbrella packages (`sailComponents: true` / multi-domain layout).
    let mut sail_files: Vec<PathBuf> = Vec::new();
    let src_dir = package_root.join("src");
    if src_dir.exists() {
        walk_component_sail_files(&src_dir, &mut sail_files);
    }
    let is_umbrella = pkg.get("sailComponents").and_then(|v| v.as_bool()) == Some(true);
    if is_umbrella {
        if let Ok(entries) = std::fs::read_dir(package_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name();
                if name == "src" || name == "node_modules" || name == "dist" {
                    continue;
                }
                let domain_src = path.join("src");
                if domain_src.is_dir() {
                    walk_component_sail_files(&domain_src, &mut sail_files);
                }
            }
        }
    }
    sail_files.sort();

    // 3. Compile each component in-process and collect the CEM JSON values.
    let mut cems: Vec<Value> = Vec::new();

    for sail_file in &sail_files {
        let stem = sail_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let source = std::fs::read_to_string(sail_file)
            .with_context(|| format!("read {}", sail_file.display()))?;
        let opts = sail_compiler::CompileOpts {
            route_params: None,
            setup_url: None,
            mode: sail_compiler::CompileMode::Component,
            source_path: Some(sail_file.to_path_buf()),
            component_meta: Some(sail_compiler::ComponentMeta { name: stem.clone() }),
            emit_dts: false,
            scope_capabilities: Vec::new(),
            emit_component_imports: false,
            // Language policy comes from the component's own sail.config.json
            // walk-up — deck never sets or relaxes it.
            lint_overrides: sail_compiler::Policy::overrides_for(sail_file),
        };
        let out = sail_compiler::compile(&source, opts).map_err(|e| {
            anyhow::anyhow!("compile {}: {}", sail_file.display(), e.render(&source))
        })?;
        if let Some(cem) = out.cem {
            cems.push(cem);
        }
    }

    // 4. Merge into one standard manifest via the sail-compiler library.
    let merged = sail_compiler::emit::cem::merge_cem(&cems);

    // 5. Write to package_root/<customElements_field>, creating dist/ if needed.
    let out_path = package_root.join(&ce_field);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create dir {}", parent.display()))?;
    }
    std::fs::write(&out_path, serde_json::to_string_pretty(&merged)?)
        .with_context(|| format!("write {}", out_path.display()))?;

    Ok(Some(out_path))
}

/// Materialize the standard `dist/custom-elements.json` for EVERY workspace
/// components package that declares a `customElements` field. Run ONCE during the
/// framework build (`deck build-assets`), so apps then aggregate the
/// pre-written manifests via `generate_components_gen` (B2 stays a pure read).
/// Returns the count written. A pack that fails to compile is skipped (logged),
/// A pack that fails to compile is a BUILD FAILURE, not a warning. Skipping the
/// bad pack and exiting 0 is how five packs sat un-migrated and invisible: the
/// components silently had no manifest, and every consumer saw an empty catalog
/// instead of an error. Every failure is collected (so one run names them all)
/// and the build stops.
pub fn write_all_component_cems(repo_root: &Path) -> Result<usize> {
    let packages = discover_workspace_packages(repo_root)
        .map_err(|e| anyhow::anyhow!("discover workspace packages: {e}"))?;
    let mut written = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for pkg in &packages {
        if pkg.custom_elements.is_none() {
            continue;
        }
        match write_component_package_cem(&pkg.root) {
            Ok(Some(_)) => written += 1,
            Ok(None) => {}
            Err(e) => failures.push(format!("{}:\n  {e:#}", pkg.root.display())),
        }
    }
    if !failures.is_empty() {
        anyhow::bail!(
            "{} pack(s) failed to compile:\n\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }
    Ok(written)
}

/// Walk `dir` recursively and collect every component `.sail` file. Per the SAIL
/// authoring convention the filename stem IS the custom-element tag — a `.sail`
/// file always declares a component (dynamic markup lives inside it via control
/// flow, never in sibling partials). Companion tags (`sail-tab` in `sail-tabs/`,
/// `sail-accordion-item` in `sail-accordion/`, the `sail-annotation-*` family) are
/// co-located under the PRIMARY tag's directory, so their stem does NOT equal
/// the parent dir — an earlier `stem == parent` filter silently dropped them
/// from every components package's manifest. The only requirement is that the stem be a valid
/// custom-element name (contains a hyphen); anything without one can't register.
fn walk_component_sail_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_component_sail_files(&p, out);
        } else if p.extension().map_or(false, |e| e == "sail") {
            // The stem is the tag; a custom element must contain a hyphen.
            let is_component = p
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|stem| stem.contains('-'));
            if is_component {
                out.push(p);
            }
        }
    }
}

/// Emit `<project>/components.gen.ts` for an app.
///
/// Discovers components packages the app depends on via:
/// 1. workspace package map at `repo_root` (monorepo members + that root's deps)
/// 2. the app's own `package.json` deps resolved through `node_modules` / `file:`
///    (external apps; monorepo apps whose packs hang off the app dir)
///
/// Filters to packs listed in the app's `dependencies` / `devDependencies`,
/// reads each components package's CEM, annotates every module with a `package` field (npm
/// name), merges modules (path-sorted, deterministic), and writes
/// `components.gen.ts`. Returns count of components packages included.
///
/// No-op (returns `Ok(0)`) when the app has no `package.json`.
pub fn generate_components_gen(project: &Path, repo_root: &Path) -> Result<usize> {
    // 1. Load the app's dependency list.
    let app_pkg_path = project.join("package.json");
    if !app_pkg_path.exists() {
        return Ok(0);
    }
    let app_pkg_raw = std::fs::read_to_string(&app_pkg_path)
        .with_context(|| format!("read {}", app_pkg_path.display()))?;
    let app_pkg: Value = serde_json::from_str(&app_pkg_raw).context("parse app package.json")?;

    let mut app_deps: HashSet<String> = HashSet::new();
    for key in ["dependencies", "devDependencies"] {
        if let Some(obj) = app_pkg[key].as_object() {
            for name in obj.keys() {
                app_deps.insert(name.clone());
            }
        }
    }

    // 2. Discover packs: workspace map at repo_root, union project-local
    //    dependency resolution (node_modules / file:). External apps have no
    //    workspaces field — only (2) contributes. Monorepo apps get (1) and
    //    also (2) when project ≠ repo_root so app-dir installs are visible.
    let mut packages = discover_workspace_packages(repo_root)
        .map_err(|e| anyhow::anyhow!("discover workspace packages: {e}"))?;
    if project != repo_root {
        let mut seen: HashSet<String> = packages.iter().map(|p| p.name.clone()).collect();
        for pkg in crate::catalog::discover_dependency_packages(project) {
            if seen.insert(pkg.name.clone()) {
                packages.push(pkg);
            }
        }
    }

    let mut all_modules: Vec<Value> = Vec::new();
    let mut count = 0usize;

    for pkg in &packages {
        let Some(ce_field) = &pkg.custom_elements else {
            continue;
        };
        if !app_deps.contains(&pkg.name) {
            continue;
        }
        let cem_path = pkg.root.join(ce_field);
        if !cem_path.exists() {
            continue;
        }

        let cem_raw = std::fs::read_to_string(&cem_path)
            .with_context(|| format!("read {}", cem_path.display()))?;
        let cem: Value = serde_json::from_str(&cem_raw)
            .with_context(|| format!("parse CEM {}", cem_path.display()))?;

        let Some(modules) = cem["modules"].as_array() else {
            continue;
        };
        for module in modules {
            let mut m = module.clone();
            if let Some(obj) = m.as_object_mut() {
                obj.insert("package".into(), Value::String(pkg.name.clone()));
            }
            all_modules.push(m);
        }
        count += 1;
    }

    // 3. Sort all modules by path (deterministic, path-stable output).
    all_modules.sort_by(|a, b| {
        let ap = a["path"].as_str().unwrap_or("");
        let bp = b["path"].as_str().unwrap_or("");
        ap.cmp(bp)
    });

    // 4. Serialize the catalog as pretty JSON and wrap in a TypeScript const.
    let catalog = serde_json::json!({
        "schemaVersion": "1.0.0",
        "modules": all_modules,
    });
    let catalog_json = serde_json::to_string_pretty(&catalog)?;

    let body = format!(
        "// @generated by `deck build` (deck-bundler) — DO NOT EDIT.\n\
         // Component catalog for this app, aggregated from its components package deps.\n\
         // Regenerate with `deck build`. The `package` field on each module names its\n\
         // source package — use it to group or filter. CEM extension, not standard.\n\
         \n\
         export const COMPONENTS_CATALOG = {catalog_json} as const\n"
    );

    let out_path = project.join("components.gen.ts");
    std::fs::write(&out_path, &body).with_context(|| format!("write {}", out_path.display()))?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn cem_fixture(tag: &str) -> String {
        serde_json::json!({
            "schemaVersion": "1.0.0",
            "modules": [{
                "kind": "javascript-module",
                "path": format!("{tag}.sail"),
                "declarations": [{
                    "kind": "class",
                    "name": tag,
                    "tagName": tag,
                    "customElement": true,
                }]
            }]
        })
        .to_string()
    }

    /// PACK-IMPORTS-1 / L2-2: the umbrella `@ship/sail-components` package
    /// compiles under the components-package import envelope so CEM generation
    /// (and `sail dev`) can proceed. Domain trees are folders, not packages.
    #[test]
    fn first_party_packs_compile_under_pack_import_envelope() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root");
        let root = repo.join("packages/components/sail");
        match write_component_package_cem(&root) {
            Ok(Some(path)) => {
                assert!(
                    path.exists(),
                    "umbrella CEM path missing after write: {}",
                    path.display()
                );
            }
            Ok(None) => panic!("@ship/sail-components missing customElements field"),
            Err(e) => panic!("umbrella CEM generation failed: {e:#}"),
        }
    }

    /// Compile a component .sail in-process and verify the merged CEM is written.
    #[test]
    fn write_component_package_cem_writes_standard_manifest() {
        let tmp = tempdir().unwrap();
        let package_root = tmp.path();

        write(
            &package_root.join("package.json"),
            r#"{"name":"@ship/fixture-pack","customElements":"dist/custom-elements.json"}"#,
        );
        // Minimal component file — no props, tag comes from ComponentMeta.
        write(
            &package_root.join("src/sail-foo/sail-foo.sail"),
            "<div>hello</div>\n",
        );

        let result =
            write_component_package_cem(package_root).expect("write_component_package_cem");
        let out_path = result.expect("expected Some(path)");

        assert!(out_path.exists(), "dist/custom-elements.json written");

        let raw = std::fs::read_to_string(&out_path).unwrap();
        let json: Value = serde_json::from_str(&raw).unwrap();

        assert_eq!(json["schemaVersion"], "1.0.0");
        let modules = json["modules"].as_array().unwrap();
        assert!(!modules.is_empty(), "at least one module");

        let has_foo = modules.iter().any(|m| {
            m["declarations"].as_array().map_or(false, |decls| {
                decls.iter().any(|d| d["tagName"] == "sail-foo")
            })
        });
        assert!(has_foo, "sail-foo tag in merged manifest");
    }

    #[test]
    fn generate_components_gen_includes_only_graph_packs() {
        let tmp = tempdir().unwrap();
        let repo_root = tmp.path();

        // Workspace root.
        write(
            &repo_root.join("package.json"),
            r#"{"workspaces":["packages/*"]}"#,
        );

        // @ship/charts — in the app's deps.
        write(
            &repo_root.join("packages/charts/package.json"),
            r#"{"name":"@ship/charts","customElements":"dist/custom-elements.json"}"#,
        );
        write(
            &repo_root.join("packages/charts/dist/custom-elements.json"),
            &cem_fixture("sail-bar-chart"),
        );

        // @ship/agents — NOT in the app's deps.
        write(
            &repo_root.join("packages/agents/package.json"),
            r#"{"name":"@ship/agents","customElements":"dist/custom-elements.json"}"#,
        );
        write(
            &repo_root.join("packages/agents/dist/custom-elements.json"),
            &cem_fixture("sail-agent"),
        );

        // App project — depends on @ship/charts only.
        let project = repo_root.join("apps/hub");
        write(
            &project.join("package.json"),
            r#"{"name":"@ship/hub","dependencies":{"@ship/charts":"^1.0.0"}}"#,
        );

        let count = generate_components_gen(&project, repo_root).expect("generate_components_gen");

        assert_eq!(count, 1, "one components package included");

        let out = std::fs::read_to_string(project.join("components.gen.ts")).unwrap();

        assert!(out.contains("sail-bar-chart"), "charts component in output");
        assert!(!out.contains("sail-agent"), "agents component excluded");
        assert!(
            out.contains("@ship/charts"),
            "package field present for charts"
        );
        assert!(
            !out.contains("@ship/agents"),
            "agents package absent from output"
        );
    }

    /// EXT-1 E3: external app — no `workspaces` map — depends on a components package only
    /// via `node_modules` (the `bun add @ship/sail-components` shape). Tag must
    /// resolve and `components.gen.ts` must include it.
    #[test]
    fn external_components_package_via_node_modules_without_workspace_map() {
        let tmp = tempdir().unwrap();
        let app = tmp.path();

        // Pure external app: package.json deps only — no workspaces field.
        write(
            &app.join("package.json"),
            r#"{"name":"external-hello","dependencies":{"@ship/sail-components":"^0.3.0"}}"#,
        );

        // Installed components package under node_modules (registry / bun add layout).
        let pack = app.join("node_modules/@ship/sail-components");
        write(
            &pack.join("package.json"),
            r#"{"name":"@ship/sail-components","customElements":"dist/custom-elements.json"}"#,
        );
        write(
            &pack.join("src/sail-bar-chart/sail-bar-chart.sail"),
            "<div>sail-bar-chart-marker</div>\n",
        );
        write(
            &pack.join("dist/custom-elements.json"),
            &cem_fixture("sail-bar-chart"),
        );

        // components.gen aggregates the package CEM without a workspace map.
        let count = generate_components_gen(app, app).expect("generate_components_gen external");
        assert_eq!(count, 1, "node_modules components package must be included");
        let catalog = std::fs::read_to_string(app.join("components.gen.ts")).unwrap();
        assert!(
            catalog.contains("sail-bar-chart"),
            "components.gen must list sail-bar-chart; got:\n{catalog}"
        );
        assert!(
            catalog.contains("@ship/sail-components"),
            "components.gen must stamp package name; got:\n{catalog}"
        );

        // Tag source index resolves from node_modules (compile path input).
        let idx = crate::catalog::tag_source_index(app);
        let src = idx
            .get("sail-bar-chart")
            .expect("sail-bar-chart must resolve from node_modules components package");
        assert!(src.is_file());
        assert!(
            src.ends_with(
                "node_modules/@ship/sail-components/src/sail-bar-chart/sail-bar-chart.sail"
            ),
            "resolved path must be under node_modules; got: {src:?}"
        );
    }

    /// EXT-1 E3: same external shape, components package linked via `file:` instead of a
    /// registry install. Still no workspaces map.
    #[test]
    fn external_components_package_via_file_dep_without_workspace_map() {
        let tmp = tempdir().unwrap();
        let app = tmp.path();
        let vendor = app.join("vendor/sail-charts");

        write(
            &vendor.join("package.json"),
            r#"{"name":"@ship/sail-components","customElements":"dist/custom-elements.json"}"#,
        );
        write(
            &vendor.join("src/sail-bar-chart/sail-bar-chart.sail"),
            "<div>bar</div>\n",
        );
        write(
            &vendor.join("dist/custom-elements.json"),
            &cem_fixture("sail-bar-chart"),
        );
        write(
            &app.join("package.json"),
            r#"{"name":"external-hello","dependencies":{"@ship/sail-components":"file:./vendor/sail-charts"}}"#,
        );

        let count = generate_components_gen(app, app).expect("generate_components_gen file:");
        assert_eq!(count, 1);
        let catalog = std::fs::read_to_string(app.join("components.gen.ts")).unwrap();
        assert!(catalog.contains("sail-bar-chart"));
        assert!(catalog.contains("@ship/sail-components"));

        let idx = crate::catalog::tag_source_index(app);
        assert!(
            idx.contains_key("sail-bar-chart"),
            "file: pack tag must resolve"
        );
    }
}
