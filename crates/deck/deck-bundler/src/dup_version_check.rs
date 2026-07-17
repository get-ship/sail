//! Post-bundle duplicate-package-version WARNING (#89 item 3).
//!
//! When the module graph pulls in two versions of the same package name
//! (e.g. `@codemirror/state` 6.7.0 + 6.7.1 via nested store links — a real
//! case that cost a day of debugging as a mysterious CodeMirror
//! `Unrecognized extension value` runtime error), deck-bundler bundles
//! both copies silently: nothing in the build log hints that the app now
//! carries two live copies of a singleton-shaped module. This check walks
//! the resolved module paths after bundling (the same `module_paths` the
//! tag-collision check and dropped-side-effect detector already consume —
//! no new plumbing) and warns, naming the package, both versions, and a
//! resolved path for each.
//!
//! DIAGNOSTIC ONLY: this never changes what gets bundled, only surfaces
//! the ambiguity. It is a build-time `tracing::warn!`, not a hard error —
//! unlike `tag_check`, a duplicate version is not always a bug (some
//! packages are safe to duplicate), so the author decides.
//!
//! Limitation (the seam for the rest, if this needs to get sharper): this
//! reports the resolved module path nearest each duplicated package root,
//! not the actual importing module. Rolldown's `resolve_id` hook sees the
//! real `(specifier, importer)` pair at resolve time, but `SailPlugin`
//! returns `Ok(None)` for node_modules specifiers (falls through to
//! Rolldown's native resolver) so it never observes the resolved id or its
//! importer. Wiring a real importer trail would mean either intercepting
//! that fallthrough (calling Rolldown's resolver ourselves to see the
//! result before returning `None`) or adding a Rolldown plugin hook that
//! observes post-resolution `(id, importer)` pairs, if one exists in this
//! Rolldown version. The current, cheaper post-bundle scan already
//! catches the case that mattered (two versions BOTH entering the bundled
//! output) without that plumbing.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A duplicate-version finding: one package name resolved to more than one
/// version across the module graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DupVersion {
    pub package: String,
    /// `(version, a resolved module path under that version's package root)`,
    /// sorted by version for a deterministic message.
    pub versions: Vec<(String, PathBuf)>,
}

/// Scan `module_paths` (the bundle's resolved module ids) for a package name
/// that resolves to more than one distinct version. Returns findings sorted
/// by package name.
pub fn detect_duplicate_versions(module_paths: &[PathBuf]) -> Vec<DupVersion> {
    // package name -> version -> first module path seen under that version.
    let mut by_package: BTreeMap<String, BTreeMap<String, PathBuf>> = BTreeMap::new();

    for path in module_paths {
        let Some((name, root)) = nearest_node_modules_package(path) else {
            continue;
        };
        let Some(version) = read_package_version(&root) else {
            continue;
        };
        by_package
            .entry(name)
            .or_default()
            .entry(version)
            .or_insert_with(|| path.clone());
    }

    by_package
        .into_iter()
        .filter(|(_, versions)| versions.len() > 1)
        .map(|(package, versions)| DupVersion {
            package,
            versions: versions.into_iter().collect(),
        })
        .collect()
}

/// Emit a `tracing::warn!` per duplicate-version finding, naming the
/// package, both (or more) versions, and a resolved path for each.
pub fn warn_duplicate_versions(module_paths: &[PathBuf]) {
    for dup in detect_duplicate_versions(module_paths) {
        let versions: Vec<String> = dup
            .versions
            .iter()
            .map(|(v, p)| format!("{v} ({})", p.display()))
            .collect();
        tracing::warn!(
            target: "deck-bundler",
            package = %dup.package,
            versions = %versions.join(", "),
            "deck-bundler: `{}` enters the bundle at {} distinct versions ({}) — both \
             copies are bundled silently. Nested/hoisted install layouts (bun/npm/pnpm) \
             can pull in two versions of the same package via different importers; if \
             the package is a singleton (shares module-level state, a registry, or a \
             custom-element define), this duplicates that state at runtime. Dedupe the \
             version in your lockfile/overrides or confirm duplication is safe.",
            dup.package,
            dup.versions.len(),
            versions.join(", "),
        );
    }
}

/// Walk up from `path` to the nearest `node_modules/<name>` (or
/// `node_modules/@scope/name`) ancestor, returning the package name and its
/// root directory. `None` when `path` carries no `node_modules` segment
/// (workspace-local source, not an npm dependency).
fn nearest_node_modules_package(path: &Path) -> Option<(String, PathBuf)> {
    let comps: Vec<&std::ffi::OsStr> = path.components().map(|c| c.as_os_str()).collect();
    // Find the LAST `node_modules` segment (nested installs put the
    // resolved copy under the deepest one).
    let idx = comps.iter().rposition(|c| *c == "node_modules")?;
    let after = &comps[idx + 1..];
    if after.is_empty() {
        return None;
    }
    let first = after[0].to_str()?;
    let (name, root_len) = if let Some(scope) = first.strip_prefix('@') {
        let _ = scope;
        if after.len() < 2 {
            return None;
        }
        let second = after[1].to_str()?;
        (format!("{first}/{second}"), 2)
    } else {
        (first.to_string(), 1)
    };
    let mut root = PathBuf::new();
    for c in &comps[..=idx + root_len] {
        root.push(c);
    }
    Some((name, root))
}

/// Read `<root>/package.json`'s `version` field.
fn read_package_version(root: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(root.join("package.json")).ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&raw).ok()?;
    pkg.get("version")?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    fn scratch() -> PathBuf {
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join(format!("deck-bundler-dup-version-{}-{n}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_pkg(root: &Path, name: &str, version: &str) {
        fs::create_dir_all(root).unwrap();
        fs::write(
            root.join("package.json"),
            format!(r#"{{"name":"{name}","version":"{version}"}}"#),
        )
        .unwrap();
    }

    #[test]
    fn detects_two_versions_of_same_package() {
        let dir = scratch();
        // Synthetic two-version graph: the exact `@codemirror/state`
        // 6.7.0 + 6.7.1 shape (hoisted at root, nested via a different
        // importer's own node_modules).
        let root_pkg = dir.join("node_modules/@codemirror/state");
        write_pkg(&root_pkg, "@codemirror/state", "6.7.0");
        fs::write(root_pkg.join("index.js"), "export const x = 1;\n").unwrap();

        let nested_pkg =
            dir.join("node_modules/some-store/node_modules/@codemirror/state");
        write_pkg(&nested_pkg, "@codemirror/state", "6.7.1");
        fs::write(nested_pkg.join("index.js"), "export const x = 2;\n").unwrap();

        let module_paths = vec![root_pkg.join("index.js"), nested_pkg.join("index.js")];
        let dups = detect_duplicate_versions(&module_paths);
        assert_eq!(dups.len(), 1, "exactly one duplicated package: {dups:?}");
        assert_eq!(dups[0].package, "@codemirror/state");
        let mut versions: Vec<&str> = dups[0].versions.iter().map(|(v, _)| v.as_str()).collect();
        versions.sort();
        assert_eq!(versions, vec!["6.7.0", "6.7.1"]);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn single_version_of_a_package_is_not_flagged() {
        let dir = scratch();
        let pkg = dir.join("node_modules/lodash");
        write_pkg(&pkg, "lodash", "4.17.21");
        fs::write(pkg.join("index.js"), "module.exports = {};\n").unwrap();

        let module_paths = vec![pkg.join("index.js")];
        assert!(detect_duplicate_versions(&module_paths).is_empty());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn workspace_local_source_outside_node_modules_is_ignored() {
        let dir = scratch();
        let src = dir.join("apps/shell/src/index.ts");
        fs::create_dir_all(src.parent().unwrap()).unwrap();
        fs::write(&src, "export const x = 1;\n").unwrap();

        assert!(detect_duplicate_versions(&[src]).is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn scoped_package_name_parsed_correctly() {
        let dir = scratch();
        let pkg = dir.join("node_modules/@ship/sail-runtime");
        write_pkg(&pkg, "@ship/sail-runtime", "0.1.0");
        let (name, root) = nearest_node_modules_package(&pkg.join("src/index.ts")).unwrap();
        assert_eq!(name, "@ship/sail-runtime");
        assert_eq!(root, pkg);
        fs::remove_dir_all(&dir).ok();
    }
}
