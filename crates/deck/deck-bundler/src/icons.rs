//! Compile-collected lucide icons → static app assets under `_deck/icons/`.
//!
//! Walks project routes **and** resolved component package sources (app
//! dependency packs with `.sail` files) for static `<icon name="…">` usage,
//! then writes only those glyphs as standalone SVGs so runtime
//! `fetch('/_deck/icons/{name}.svg')` works offline / without the daemon.
//! No CDN.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sail_compiler::collect_used_icons_from_source;

/// Relative output dir under the app stage / out root.
pub const ICONS_OUT_REL: &str = "_deck/icons";

/// Collect static icon names from every `.sail` under each root.
pub fn collect_project_icons(roots: &[&Path]) -> Result<BTreeSet<String>> {
    let mut icons = BTreeSet::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        walk_sail(root, &mut icons)?;
    }
    Ok(icons)
}

fn walk_sail(dir: &Path, icons: &mut BTreeSet<String>) -> Result<()> {
    let entries = std::fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            // Skip heavy/non-source trees.
            let name = entry.file_name();
            let n = name.to_string_lossy();
            if n == "node_modules" || n == "dist" || n == ".deck" || n.starts_with('.') {
                continue;
            }
            walk_sail(&path, icons)?;
        } else if ft.is_file() {
            let is_sail = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("sail"));
            if !is_sail {
                continue;
            }
            let src = std::fs::read_to_string(&path)
                .with_context(|| format!("read {}", path.display()))?;
            icons.extend(collect_used_icons_from_source(&src));
        }
    }
    Ok(())
}

/// Package roots the app depends on that contain component sources (`.sail`).
/// Transitive deps via package.json, resolved through the workspace package map.
pub fn app_component_source_roots(project: &Path, repo_root: &Path) -> Result<Vec<PathBuf>> {
    let package_roots = crate::catalog::component_package_roots(repo_root);
    let pkg_path = project.join("package.json");
    if !pkg_path.is_file() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&pkg_path)
        .with_context(|| format!("read {}", pkg_path.display()))?;
    let pkg: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", pkg_path.display()))?;

    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    walk_dep_roots(&pkg, &package_roots, &mut out, &mut seen);
    out.sort();
    out.dedup();
    Ok(out)
}

fn walk_dep_roots(
    pkg: &serde_json::Value,
    package_roots: &std::collections::HashMap<String, PathBuf>,
    out: &mut Vec<PathBuf>,
    seen: &mut HashSet<String>,
) {
    for deps_key in ["dependencies", "devDependencies", "peerDependencies"] {
        let Some(deps) = pkg.get(deps_key).and_then(|d| d.as_object()) else {
            continue;
        };
        for (dep_name, _) in deps {
            if !seen.insert(dep_name.clone()) {
                continue;
            }
            let Some(root) = package_roots.get(dep_name) else {
                continue;
            };
            // Only packs that author .sail (components / app chrome).
            if package_has_sail(root) {
                out.push(root.clone());
            }
            // Recurse into workspace deps so transitive component packs count.
            let dep_pkg_path = root.join("package.json");
            if let Ok(raw) = std::fs::read_to_string(&dep_pkg_path) {
                if let Ok(dep_pkg) = serde_json::from_str::<serde_json::Value>(&raw) {
                    walk_dep_roots(&dep_pkg, package_roots, out, seen);
                }
            }
        }
    }
}

fn package_has_sail(root: &Path) -> bool {
    // Fast path: customElements pack or any .sail under src/.
    let pkg = root.join("package.json");
    if let Ok(raw) = std::fs::read_to_string(&pkg) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if v.get("customElements").is_some() {
                return true;
            }
        }
    }
    for candidate in ["src", "routes", "."] {
        let dir = root.join(candidate);
        if dir.is_dir() && dir_contains_sail(&dir, 0) {
            return true;
        }
    }
    false
}

fn dir_contains_sail(dir: &Path, depth: u8) -> bool {
    if depth > 6 {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_file() {
            if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("sail"))
            {
                return true;
            }
        } else if ft.is_dir() {
            let n = entry.file_name();
            let name = n.to_string_lossy();
            if name == "node_modules" || name == "dist" || name == "test" || name.starts_with('.') {
                continue;
            }
            if dir_contains_sail(&path, depth + 1) {
                return true;
            }
        }
    }
    false
}

/// Write each used glyph as `_deck/icons/{name}.svg` under `out_dir`.
pub fn emit_icon_assets(out_dir: &Path, names: &BTreeSet<String>) -> Result<BTreeSet<String>> {
    if names.is_empty() {
        return Ok(BTreeSet::new());
    }
    let icons_dir = out_dir.join(ICONS_OUT_REL);
    std::fs::create_dir_all(&icons_dir)
        .with_context(|| format!("mkdir {}", icons_dir.display()))?;

    let mut written = BTreeSet::new();
    for name in names {
        match deck_core::lucide_icon_svg(name) {
            Some(svg) => {
                let path = icons_dir.join(format!("{name}.svg"));
                std::fs::write(&path, svg).with_context(|| format!("write {}", path.display()))?;
                written.insert(name.clone());
            }
            None => {
                eprintln!(
                    "deck warn: icon `{name}` not in lucide pack — skipped asset emit \
                     (runtime will show slotted fallback)"
                );
            }
        }
    }

    let manifest_path = icons_dir.join("manifest.json");
    let list: Vec<&String> = written.iter().collect();
    let body = serde_json::to_string_pretty(&list).context("serialize icon manifest")?;
    std::fs::write(&manifest_path, format!("{body}\n"))
        .with_context(|| format!("write {}", manifest_path.display()))?;

    Ok(written)
}

/// Collect from project routes + app component package sources, emit into stage.
pub fn collect_and_emit_project_icons(
    project: &Path,
    out_dir: &Path,
    repo_root: &Path,
) -> Result<BTreeSet<String>> {
    let routes = project.join("routes");
    let mut roots: Vec<PathBuf> = vec![routes];
    roots.extend(app_component_source_roots(project, repo_root)?);
    let root_refs: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
    let names = collect_project_icons(&root_refs)?;
    emit_icon_assets(out_dir, &names)
}

/// Absolute path helper for tests.
pub fn icon_asset_path(out_dir: &Path, name: &str) -> PathBuf {
    out_dir.join(ICONS_OUT_REL).join(format!("{name}.svg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn emit_three_icons_only() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("app");
        let routes = project.join("routes");
        fs::create_dir_all(&routes).unwrap();
        fs::write(
            routes.join("page.sail"),
            r#"<main>
              <icon name="search"></icon>
              <icon name="check"></icon>
              <icon name="rocket"></icon>
              <icon name={dyn()}></icon>
            </main>"#,
        )
        .unwrap();
        let out = tmp.path().join("out");
        fs::create_dir_all(&out).unwrap();
        // No workspace packs needed — routes alone.
        let written = collect_and_emit_project_icons(&project, &out, tmp.path()).unwrap();
        assert_eq!(
            written,
            BTreeSet::from(["search".into(), "check".into(), "rocket".into()])
        );
        for name in ["search", "check", "rocket"] {
            let p = icon_asset_path(&out, name);
            assert!(p.is_file(), "missing {}", p.display());
        }
        assert!(!icon_asset_path(&out, "dyn").exists());
    }

    #[test]
    fn emit_includes_component_pack_icons() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        // Minimal workspace: app depends on @ship/pack-a with a .sail icon.
        let pack = repo.join("packages/pack-a");
        fs::create_dir_all(pack.join("src")).unwrap();
        fs::write(
            pack.join("package.json"),
            r#"{
              "name": "@ship/pack-a",
              "customElements": "dist/custom-elements.json"
            }"#,
        )
        .unwrap();
        fs::write(
            pack.join("src/widget.sail"),
            r#"<div><icon name="chevron-right"></icon><icon name="alert-triangle"></icon></div>"#,
        )
        .unwrap();

        let project = repo.join("apps/demo");
        fs::create_dir_all(project.join("routes")).unwrap();
        fs::write(
            project.join("package.json"),
            r#"{
              "name": "@ship/demo",
              "dependencies": { "@ship/pack-a": "workspace:*" }
            }"#,
        )
        .unwrap();
        fs::write(
            project.join("routes/page.sail"),
            r#"<main><icon name="search"></icon></main>"#,
        )
        .unwrap();

        // Workspace package map is built from package.json workspaces — seed it.
        fs::write(
            repo.join("package.json"),
            r#"{
              "name": "ws",
              "workspaces": { "packages": ["packages/*", "apps/*"] }
            }"#,
        )
        .unwrap();

        let out = repo.join("out");
        fs::create_dir_all(&out).unwrap();
        let written = collect_and_emit_project_icons(&project, &out, repo).unwrap();
        assert!(written.contains("search"), "{written:?}");
        assert!(
            written.contains("chevron-right"),
            "component pack icon missing: {written:?}"
        );
        assert!(
            written.contains("alert-triangle"),
            "component pack icon missing: {written:?}"
        );
        assert_eq!(written.len(), 3, "{written:?}");
    }
}
