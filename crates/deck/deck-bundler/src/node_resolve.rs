//! Node-style package resolution for the framework singletons.
//!
//! deck's framework singletons (`@ship/sail-runtime`, `@ship/sail-dom`,
//! `@ship/deck-router`, `@ship/sdk-client`) are no longer pinned to a hardcoded
//! monorepo path (`packages/sail/runtime/src/index.ts`). They are resolved like
//! any npm dependency: from the project being built, find the package's
//! `package.json` and read its `exports`/`main` to locate the entry module.
//!
//! This is what lets a build run OUTSIDE the sail monorepo — a consumer repo
//! (ship-apps, ship-site) installs the framework from the registry, and the
//! bundler resolves it from wherever npm/bun put it, with no
//! `DECK_FRAMEWORK_ASSETS_DIR` crutch and no `|| true` workaround.
//!
//! Resolution order (first hit wins):
//!   1. **node_modules ancestor walk** — `<anc>/node_modules/<name>/package.json`
//!      for `start` and each of its ancestors. Covers consumer installs and
//!      direct-dependency symlinks (bun/npm hoist + workspace links).
//!   2. **workspace member** — `start`'s own `package.json` `workspaces` globs.
//!      Covers the monorepo bake, where members live in the package store, not
//!      the root `node_modules` (bun does not symlink workspace members into the
//!      root `node_modules`).
//!
//! The selected `package.json` then drives the entry: `exports[<subpath>]`
//! (honoring the `import`/`module`/`default` conditions), else `main`, else
//! `index.js`.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::catalog::discover_workspace_packages;

/// Resolve a bare package specifier to an absolute entry module, starting from
/// `start` (the project / repo root being built).
///
/// `spec` may carry a subpath: `@ship/sail-runtime` resolves the package's `.`
/// export; `@ship/sail-runtime/route` resolves its `./route` export. Returns
/// `None` when the package is not installed/discoverable from `start`, or when
/// no entry file actually exists on disk — callers treat that as "this
/// singleton isn't present in this repo" and skip it rather than fail the whole
/// build.
///
/// Resolution is EXISTENCE-AWARE: the `exports`/`main` target is preferred, but
/// when it points at a file that wasn't shipped (e.g. a package whose `exports`
/// name an unbuilt `dist/` but ship only `src/`), it falls back to the common
/// source entry points. deck bundles from source, so a real `src/index.ts` is a
/// valid entry even when `dist/` is absent.
pub fn resolve_entry(start: &Path, spec: &str) -> Option<PathBuf> {
    let (name, subpath) = split_spec(spec);
    let root = resolve_package_root(start, &name)?;

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(rel) = entry_rel_from_manifest(&root, &subpath) {
        candidates.push(root.join(rel));
    }
    // Source fallbacks for the package root only — a missing subpath export has
    // no generic stand-in.
    if subpath == "." {
        for rel in ["src/index.ts", "src/index.tsx", "src/index.js", "index.ts", "index.js"] {
            candidates.push(root.join(rel));
        }
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// Split `@scope/pkg/sub/path` into (`@scope/pkg`, `./sub/path`); a bare
/// `pkg/sub` into (`pkg`, `./sub`). The subpath is the `exports` key form
/// (`.` for the package root).
fn split_spec(spec: &str) -> (String, String) {
    let parts: Vec<&str> = spec.split('/').collect();
    // Scoped packages (`@scope/name`) consume two segments for the name.
    let name_segs = if spec.starts_with('@') { 2 } else { 1 };
    if parts.len() <= name_segs {
        return (spec.to_string(), ".".to_string());
    }
    let name = parts[..name_segs].join("/");
    let sub = format!("./{}", parts[name_segs..].join("/"));
    (name, sub)
}

/// Find a package's root directory by name, starting from `start`.
fn resolve_package_root(start: &Path, name: &str) -> Option<PathBuf> {
    // 1. node_modules ancestor walk.
    let mut cur = Some(start);
    while let Some(dir) = cur {
        let candidate = dir.join("node_modules").join(name);
        if candidate.join("package.json").is_file() {
            return Some(candidate);
        }
        cur = dir.parent();
    }
    // 2. workspace member of `start` (the monorepo bake path).
    if let Ok(pkgs) = discover_workspace_packages(start) {
        if let Some(pkg) = pkgs.into_iter().find(|p| p.name == name) {
            return Some(pkg.root);
        }
    }
    None
}

/// Read `<root>/package.json` and resolve the entry path (relative to `root`)
/// for `subpath` (`.` or `./sub`). Honors `exports` conditions, falls back to
/// `main`, then `index.js`.
fn entry_rel_from_manifest(root: &Path, subpath: &str) -> Option<PathBuf> {
    let raw = std::fs::read_to_string(root.join("package.json")).ok()?;
    let pkg: Value = serde_json::from_str(&raw).ok()?;

    if let Some(exports) = pkg.get("exports") {
        if let Some(rel) = resolve_exports(exports, subpath) {
            return Some(PathBuf::from(rel.trim_start_matches("./")));
        }
    }
    // No `exports` match. `main` only answers the package root (`.`).
    if subpath == "." {
        if let Some(main) = pkg.get("main").and_then(|m| m.as_str()) {
            return Some(PathBuf::from(main.trim_start_matches("./")));
        }
        return Some(PathBuf::from("index.js"));
    }
    None
}

/// Resolve a target string from an `exports` value for `subpath`.
///
/// `exports` may be:
///   * a bare string (`"./src/index.ts"`) — only valid for the `.` subpath;
///   * a conditions object (`{ "import": "./x.js", "default": "./y.js" }`);
///   * a subpath map (`{ ".": …, "./route": … }`).
/// Conditions are resolved by preference: `import`, `module`, `browser`,
/// `default`, then `require` (deck bundles ESM source, so import-like wins).
fn resolve_exports(exports: &Value, subpath: &str) -> Option<String> {
    match exports {
        Value::String(s) => (subpath == ".").then(|| s.clone()),
        Value::Object(map) => {
            // Subpath map iff every key starts with "." (a key set of
            // conditions otherwise).
            let is_subpath_map = map.keys().all(|k| k.starts_with('.'));
            if is_subpath_map {
                let target = map.get(subpath)?;
                resolve_conditions(target)
            } else {
                resolve_conditions(exports)
            }
        }
        _ => None,
    }
}

/// Resolve a target that is either a string or a conditions object.
fn resolve_conditions(target: &Value) -> Option<String> {
    match target {
        Value::String(s) => Some(s.clone()),
        Value::Object(map) => {
            for cond in ["import", "module", "browser", "default", "require"] {
                if let Some(v) = map.get(cond) {
                    if let Some(found) = resolve_conditions(v) {
                        return Some(found);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write(p: &Path, s: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, s).unwrap();
    }

    #[test]
    fn split_spec_scoped_and_subpath() {
        assert_eq!(split_spec("@ship/sail-runtime"), ("@ship/sail-runtime".into(), ".".into()));
        assert_eq!(
            split_spec("@ship/sail-runtime/route"),
            ("@ship/sail-runtime".into(), "./route".into())
        );
        assert_eq!(split_spec("lodash"), ("lodash".into(), ".".into()));
        assert_eq!(split_spec("lodash/fp"), ("lodash".into(), "./fp".into()));
    }

    #[test]
    fn resolves_via_node_modules_ancestor_walk() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        // Project nested two levels under the repo; the framework is installed
        // at the repo root's node_modules.
        let project = root.join("apps/hub");
        fs::create_dir_all(&project).unwrap();
        write(
            &root.join("node_modules/@ship/sail-runtime/package.json"),
            r#"{ "name": "@ship/sail-runtime", "exports": { ".": "./src/index.ts" } }"#,
        );
        write(&root.join("node_modules/@ship/sail-runtime/src/index.ts"), "export const x=1;\n");

        let entry = resolve_entry(&project, "@ship/sail-runtime").expect("resolved via ancestor walk");
        assert_eq!(entry, root.join("node_modules/@ship/sail-runtime/src/index.ts"));
    }

    #[test]
    fn resolves_subpath_export() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(
            &root.join("node_modules/@ship/sail-runtime/package.json"),
            r#"{ "name": "@ship/sail-runtime",
                "exports": { ".": "./src/index.ts", "./route": "./src/route/index.ts" } }"#,
        );
        write(&root.join("node_modules/@ship/sail-runtime/src/route/index.ts"), "export const r=1;\n");
        let entry = resolve_entry(root, "@ship/sail-runtime/route").unwrap();
        assert_eq!(entry, root.join("node_modules/@ship/sail-runtime/src/route/index.ts"));
    }

    #[test]
    fn resolves_conditional_import_export() {
        // The `@ship/deck-router` shape: `exports["."]` is a conditions object
        // pointing at the built `dist/index.js`. `import` must win over `types`.
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(
            &root.join("node_modules/@ship/deck-router/package.json"),
            r#"{ "name": "@ship/deck-router",
                "main": "dist/index.js",
                "exports": { ".": { "types": "./dist/index.d.ts", "import": "./dist/index.js" } } }"#,
        );
        write(&root.join("node_modules/@ship/deck-router/dist/index.js"), "export const x=1;\n");
        let entry = resolve_entry(root, "@ship/deck-router").unwrap();
        assert_eq!(entry, root.join("node_modules/@ship/deck-router/dist/index.js"));
    }

    #[test]
    fn falls_back_to_source_when_exports_target_missing() {
        // The real `@ship/deck-router@0.1.0` shape: `exports` name an unbuilt
        // `dist/index.js` but the tarball ships only `src/`. Resolution must
        // fall through to the existing `src/index.ts` (deck bundles source).
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(
            &root.join("node_modules/@ship/deck-router/package.json"),
            r#"{ "name": "@ship/deck-router",
                "main": "dist/index.js",
                "exports": { ".": { "import": "./dist/index.js" } } }"#,
        );
        // No dist/ shipped — only src/.
        write(&root.join("node_modules/@ship/deck-router/src/index.ts"), "export const x=1;\n");
        let entry = resolve_entry(root, "@ship/deck-router").unwrap();
        assert_eq!(entry, root.join("node_modules/@ship/deck-router/src/index.ts"));
    }

    #[test]
    fn falls_back_to_main_when_no_exports() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(
            &root.join("node_modules/@ship/sdk-client/package.json"),
            r#"{ "name": "@ship/sdk-client", "main": "src/index.ts" }"#,
        );
        write(&root.join("node_modules/@ship/sdk-client/src/index.ts"), "export const c=1;\n");
        let entry = resolve_entry(root, "@ship/sdk-client").unwrap();
        assert_eq!(entry, root.join("node_modules/@ship/sdk-client/src/index.ts"));
    }

    #[test]
    fn resolves_workspace_member_when_not_in_node_modules() {
        // The monorepo bake: the package is a workspace member (not symlinked
        // into the root node_modules), discovered via `workspaces` globs.
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{ "workspaces": ["packages/sail/*"] }"#);
        write(
            &root.join("packages/sail/runtime/package.json"),
            r#"{ "name": "@ship/sail-runtime", "exports": { ".": "./src/index.ts" } }"#,
        );
        write(&root.join("packages/sail/runtime/src/index.ts"), "export const x=1;\n");

        let entry = resolve_entry(root, "@ship/sail-runtime").expect("resolved via workspace member");
        assert_eq!(entry, root.join("packages/sail/runtime/src/index.ts"));
    }

    #[test]
    fn unresolvable_returns_none() {
        let tmp = tempdir().unwrap();
        write(&tmp.path().join("package.json"), r#"{ "name": "lonely" }"#);
        assert!(resolve_entry(tmp.path(), "@ship/not-installed").is_none());
    }
}
