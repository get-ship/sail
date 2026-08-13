//! Compile-collected icon **roles** → used glyph bytes under `_deck/icons/`.
//!
//! Sail resolves roles. This module only writes the already-resolved SVGs
//! the runtime fetches at `/_deck/icons/{role}.svg`. No pack choice, no
//! full-pack embed, no CDN.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sail_compiler::{
    RoleRegistry, UsedGlyph, collect_used_icons_from_source, load_glyph_svg, resolve_used_glyphs,
};

/// Relative output dir under the app stage / out root.
pub const ICONS_OUT_REL: &str = "_deck/icons";

/// Collect static icon **roles** from every `.sail` under each root.
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
            if package_has_sail(root) {
                out.push(root.clone());
            }
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

/// Write each used role's resolved glyph as `_deck/icons/{role}.svg`.
pub fn emit_icon_assets(
    out_dir: &Path,
    used: &[UsedGlyph],
    search_roots: &[&Path],
) -> Result<BTreeSet<String>> {
    if used.is_empty() {
        return Ok(BTreeSet::new());
    }
    let icons_dir = out_dir.join(ICONS_OUT_REL);
    std::fs::create_dir_all(&icons_dir)
        .with_context(|| format!("mkdir {}", icons_dir.display()))?;

    let mut written = BTreeSet::new();
    let mut missing = Vec::new();
    for item in used {
        match load_glyph_svg(&item.pack, &item.glyph, search_roots) {
            Some(svg) => {
                let path = icons_dir.join(format!("{}.svg", item.role));
                std::fs::write(&path, svg).with_context(|| format!("write {}", path.display()))?;
                written.insert(item.role.clone());
            }
            None => missing.push(item.clone()),
        }
    }
    if !missing.is_empty() {
        let detail = missing
            .iter()
            .map(|m| format!("{} → {}:{}", m.role, m.pack, m.glyph))
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "icon role(s) missing offline glyph data (install the pinned pack, no CDN): {detail}"
        );
    }

    let manifest_path = icons_dir.join("manifest.json");
    let list: Vec<serde_json::Value> = used
        .iter()
        .map(|u| {
            serde_json::json!({
                "role": u.role,
                "pack": u.pack,
                "glyph": u.glyph,
            })
        })
        .collect();
    let body = serde_json::to_string_pretty(&list).context("serialize icon manifest")?;
    std::fs::write(&manifest_path, format!("{body}\n"))
        .with_context(|| format!("write {}", manifest_path.display()))?;

    Ok(written)
}

/// Collect from project routes + app components package sources, emit into stage.
pub fn collect_and_emit_project_icons(
    project: &Path,
    out_dir: &Path,
    repo_root: &Path,
) -> Result<BTreeSet<String>> {
    let routes = project.join("routes");
    let mut roots: Vec<PathBuf> = vec![routes];
    roots.extend(app_component_source_roots(project, repo_root)?);
    let root_refs: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
    let roles = collect_project_icons(&root_refs)?;
    let registry = RoleRegistry::load_from_project(project);
    let used = resolve_used_glyphs(roles, &registry);
    emit_icon_assets(out_dir, &used, &[project, repo_root])
}

/// Absolute path helper for tests.
pub fn icon_asset_path(out_dir: &Path, name: &str) -> PathBuf {
    out_dir.join(ICONS_OUT_REL).join(format!("{name}.svg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_iconify_pack(root: &Path, pack: &str, glyphs: &[(&str, &str)]) {
        let dir = root.join("node_modules/@iconify-json").join(pack);
        fs::create_dir_all(&dir).unwrap();
        let mut icons = serde_json::Map::new();
        for (name, body) in glyphs {
            icons.insert((*name).into(), serde_json::json!({ "body": body }));
        }
        let doc = serde_json::json!({
            "prefix": pack,
            "width": 24,
            "height": 24,
            "icons": icons,
        });
        fs::write(dir.join("icons.json"), serde_json::to_string(&doc).unwrap()).unwrap();
    }

    #[test]
    fn emit_three_icons_only() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("app");
        let routes = project.join("routes");
        fs::create_dir_all(&routes).unwrap();
        write_iconify_pack(
            tmp.path(),
            "lucide",
            &[
                ("search", "<circle cx=\"11\" cy=\"11\" r=\"8\"/>"),
                ("check", "<path d=\"M20 6 9 17l-5-5\"/>"),
                ("rocket", "<path d=\"m4.5 16.5 2.5-2.5\"/>"),
            ],
        );
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
        let manifest: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(out.join("_deck/icons/manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest.as_array().unwrap().len(), 3);
    }

    #[test]
    fn emit_includes_component_pack_icons() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        write_iconify_pack(
            repo,
            "lucide",
            &[
                ("search", "<circle cx=\"11\" cy=\"11\" r=\"8\"/>"),
                ("chevron-right", "<path d=\"m9 18 6-6-6-6\"/>"),
                ("alert-triangle", "<path d=\"m10.3 3.9-8 14\"/>"),
            ],
        );
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
            "components package icon missing: {written:?}"
        );
        assert!(
            written.contains("alert-triangle"),
            "components package icon missing: {written:?}"
        );
        assert_eq!(written.len(), 3, "{written:?}");
    }

    #[test]
    fn two_packs_via_role_map() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("app");
        let routes = project.join("routes");
        fs::create_dir_all(&routes).unwrap();
        write_iconify_pack(
            tmp.path(),
            "lucide",
            &[("house", "<path d=\"M3 9l9-7 9 7\"/>")],
        );
        write_iconify_pack(
            tmp.path(),
            "mdi",
            &[("cog", "<path d=\"M12 8a4 4 0 1 0 0 8\"/>")],
        );
        fs::write(
            project.join("sail.config.jsonc"),
            r#"{
              "icons": {
                "pack": "lucide",
                "roles": {
                  "home": "house",
                  "settings": "mdi:cog"
                }
              }
            }"#,
        )
        .unwrap();
        fs::write(
            routes.join("page.sail"),
            r#"<main>
              <icon name="home"></icon>
              <icon name="settings"></icon>
            </main>"#,
        )
        .unwrap();
        let out = tmp.path().join("out");
        fs::create_dir_all(&out).unwrap();
        let written = collect_and_emit_project_icons(&project, &out, tmp.path()).unwrap();
        assert_eq!(written, BTreeSet::from(["home".into(), "settings".into()]));
        let home = fs::read_to_string(icon_asset_path(&out, "home")).unwrap();
        let settings = fs::read_to_string(icon_asset_path(&out, "settings")).unwrap();
        assert!(home.contains("M3 9l9-7 9 7"), "{home}");
        assert!(settings.contains("M12 8a4 4 0 1 0 0 8"), "{settings}");
        assert!(!icon_asset_path(&out, "house").exists());
        assert!(!icon_asset_path(&out, "cog").exists());
        let manifest: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(out.join("_deck/icons/manifest.json")).unwrap(),
        )
        .unwrap();
        let packs: BTreeSet<&str> = manifest
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|r| r.get("pack").and_then(|p| p.as_str()))
            .collect();
        assert_eq!(packs, BTreeSet::from(["lucide", "mdi"]));
    }

    #[test]
    fn missing_offline_glyph_is_build_error() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("app");
        let routes = project.join("routes");
        fs::create_dir_all(&routes).unwrap();
        // Pack present but the used glyph is not.
        write_iconify_pack(tmp.path(), "lucide", &[("house", "<path d=\"M1\"/>")]);
        fs::write(
            routes.join("page.sail"),
            r#"<main><icon name="search"></icon></main>"#,
        )
        .unwrap();
        let out = tmp.path().join("out");
        fs::create_dir_all(&out).unwrap();
        let err = collect_and_emit_project_icons(&project, &out, tmp.path()).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("search"), "{msg}");
        assert!(msg.contains("offline"), "{msg}");
    }
}
