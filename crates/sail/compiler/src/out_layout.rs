//! Canonical output-tree layout for sail-compiler emit artifacts.
//!
//! Build artifacts (`.compiled.js`, `.custom-elements.json`, `.schema.json`,
//! `.sail.meta.json`, `.ssr.html`) land under the generate mirror at
//! `<project>/.sail/<source-rel>/` (product default; or `SAIL_OUT_DIR` / the
//! `--out` dir for intentional overrides). The `.sail.d.ts` type companion
//! is the EXCEPTION: it is written ADJACENT to its `.sail` source (see
//! `compile_one` in the bin), so a controller's
//! `import type { Scope } from './x.sail'` resolves with no `rootDirs` into a
//! build dir.
//!
//! Product law: generate is **one** emit root — `.sail/` (must match
//! `deck_config::SAIL_GENERATE_DIRNAME`). Deploy is `dist`. There is no
//! `.deck` generate name. Cargo `[workspace]` is never a generate root.
//! Optional host overrides via `SAIL_OUT_DIR` are intentional cache/temp, not a
//! parallel product convention.
//!
//! Production-distribution bundles (`crates/deck/deck-core/assets/sail-*.js`)
//! are a separate concern and are NOT routed through this module.

use std::path::{Path, PathBuf};

/// Default product generate root, relative to the Sail project root.
///
/// Must match `deck_config::SAIL_GENERATE_DIRNAME` (`.sail`). `pub` so
/// consumers (deck catalog, test harness) resolve the SAME location the
/// compiler writes to — never a second hardcoded copy that can drift.
/// Not `dist` (deploy). Not a Cargo workspace leaf.
pub const DEFAULT_OUT_DIR: &str = ".sail";

/// The generate sub-directory in effect: `SAIL_OUT_DIR` env override, else the
/// product default (`.sail`). Relative to the project root.
pub fn sail_out_dir() -> String {
    std::env::var("SAIL_OUT_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_OUT_DIR.to_string())
}

/// Resolve the canonical generate mirror directory for a given `.sail` source.
///
/// Layout: `<project>/<SAIL_OUT_DIR>/<source-dir-relative-to-project>/`,
/// where `<SAIL_OUT_DIR>` defaults to `.sail` and is host-overridable.
/// The project root is [`sail_lint::Project::discover`] (sail.config), never
/// a Cargo `[workspace]` walk.
///
/// Returns `None` when no Project can be located OR the source path is
/// outside the project (e.g. a tempdir test fixture). Callers fall back to a
/// sibling `dist/` directory in that case so tests keep working.
pub fn mirror_dir_for(sail_path: &Path) -> Option<PathBuf> {
    let abs = absolutize(sail_path)?;
    let project = sail_lint::Project::discover(&abs)?;
    let root = project.root();
    let src_dir = abs.parent()?;
    let rel = src_dir.strip_prefix(root).ok()?;
    Some(root.join(sail_out_dir()).join(rel))
}

/// Returns the generate directory for a `.sail` source path, falling back to
/// a sibling `dist/` directory when the source is outside any detectable
/// Project (only happens in unit tests that compile from a tempdir).
pub fn resolve_out_dir(sail_path: &Path) -> PathBuf {
    if let Some(p) = mirror_dir_for(sail_path) {
        return p;
    }
    let src_dir = sail_path.parent().unwrap_or(Path::new("."));
    src_dir.join("dist")
}

fn absolutize(p: &Path) -> Option<PathBuf> {
    if p.is_absolute() {
        Some(p.to_path_buf())
    } else {
        Some(std::env::current_dir().ok()?.join(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_dist_outside_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let sail = tmp.path().join("foo.sail");
        std::fs::write(&sail, "x").unwrap();
        let out = resolve_out_dir(&sail);
        assert!(out.ends_with("dist"));
        assert!(mirror_dir_for(&sail).is_none());
    }

    #[test]
    fn cargo_workspace_is_not_a_generate_root() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let src_dir = tmp.path().join("packages/foo/src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let sail = src_dir.join("bar.sail");
        std::fs::write(&sail, "x").unwrap();
        assert!(
            mirror_dir_for(&sail).is_none(),
            "Cargo [workspace] is not a generate root"
        );
        let out = resolve_out_dir(&sail);
        assert!(
            out.ends_with("dist"),
            "no Project → sibling dist fallback, not a cargo-rooted generate tree: {}",
            out.display()
        );
    }

    #[test]
    fn generate_is_project_sail_not_dist() {
        let tmp = tempfile::tempdir().unwrap();
        let app = tmp.path().join("hello");
        std::fs::create_dir_all(app.join("routes")).unwrap();
        std::fs::write(app.join("sail.config.jsonc"), "{}\n").unwrap();
        let sail = app.join("routes/page.sail");
        std::fs::write(&sail, "x").unwrap();
        let out = mirror_dir_for(&sail).expect("Project generate root");
        assert_eq!(out, app.join(".sail/routes"));
        assert!(
            !out.components().any(|c| c.as_os_str() == "dist"),
            "generate is .sail, not dist: {}",
            out.display()
        );
        assert!(
            !out.components().any(|c| c.as_os_str() == ".deck"),
            "generate is .sail, not .deck: {}",
            out.display()
        );
    }

    #[test]
    fn default_out_dir_is_sail() {
        assert_eq!(DEFAULT_OUT_DIR, ".sail");
        assert_eq!(sail_out_dir(), ".sail");
        assert_ne!(DEFAULT_OUT_DIR, "dist");
    }
}
