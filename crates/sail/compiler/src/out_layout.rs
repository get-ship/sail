//! Canonical output-tree layout for sail-compiler emit artifacts.
//!
//! Build artifacts (`.compiled.js`, `.custom-elements.json`, `.schema.json`,
//! `.sail.meta.json`, `.ssr.html`) land under the mirror tree at
//! `<repo-root>/.deck/<source-rel>/` (product default — SPEC-OUTPUT-1; or
//! `SAIL_OUT_DIR` / the `--out` dir for intentional overrides). The
//! `.sail.d.ts` type companion is the EXCEPTION: it is written ADJACENT to its
//! `.sail` source (see `compile_one` in the bin), so a controller's
//! `import type { Scope } from './x.sail'` resolves with no `rootDirs` into a
//! build dir.
//!
//! Product law: **one** emit root — `.deck/` (must match
//! `deck_config::DECK_OUT_DIRNAME`). No second default (`.sail-out` is gone).
//! Optional host overrides via `SAIL_OUT_DIR` are intentional cache/temp, not a
//! parallel product convention.
//!
//! Production-distribution bundles (`crates/deck/deck-core/assets/sail-*.js`)
//! are a separate concern and are NOT routed through this module.

use std::path::{Path, PathBuf};

/// Anchor file that marks the repo root: the workspace `Cargo.toml`
/// (the only one in this monorepo with a `[workspace]` table). Used by
/// the ancestor walk so the mirror layout works the same from any cwd.
const WORKSPACE_MARKER: &str = "[workspace]";

/// Default product emit root, relative to the repo root.
///
/// Must match `deck_config::DECK_OUT_DIRNAME` (`.deck`). `pub` so consumers
/// (deck catalog, test harness) resolve the SAME location the compiler writes
/// to — never a second hardcoded copy that can drift.
pub const DEFAULT_OUT_DIR: &str = ".deck";

/// The mirror sub-directory in effect: `SAIL_OUT_DIR` env override, else the
/// product default (`.deck`). Relative to the repo root.
pub fn sail_out_dir() -> String {
    std::env::var("SAIL_OUT_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_OUT_DIR.to_string())
}

/// Walk upward from `start` looking for a workspace `Cargo.toml`.
/// Returns `None` if no marker is found before reaching the filesystem root.
pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut cur = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(start)
    };
    if cur.is_file() {
        cur = cur.parent()?.to_path_buf();
    }
    loop {
        let cargo = cur.join("Cargo.toml");
        if cargo.is_file() {
            if let Ok(s) = std::fs::read_to_string(&cargo) {
                if s.contains(WORKSPACE_MARKER) {
                    return Some(cur);
                }
            }
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Resolve the canonical mirror directory for a given `.sail` source path.
///
/// Layout: `<repo-root>/<SAIL_OUT_DIR>/<source-dir-relative-to-repo-root>/`,
/// where `<SAIL_OUT_DIR>` defaults to `.deck` and is host-overridable.
///
/// Returns `None` when no repo root can be located OR the source path is
/// outside the repo (e.g. a tempdir test fixture). Callers fall back to a
/// sibling `dist/` directory in that case so tests keep working.
pub fn mirror_dir_for(sail_path: &Path) -> Option<PathBuf> {
    let abs = absolutize(sail_path)?;
    let root = find_repo_root(&abs)?;
    let src_dir = abs.parent()?;
    let rel = src_dir.strip_prefix(&root).ok()?;
    Some(root.join(sail_out_dir()).join(rel))
}

/// Returns the mirror directory for a `.sail` source path, falling back to
/// a sibling `dist/` directory when the source is outside any detectable
/// repo (only happens in unit tests that compile from a tempdir).
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
    }

    #[test]
    fn mirrors_path_under_repo() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let src_dir = tmp.path().join("packages/foo/src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let sail = src_dir.join("bar.sail");
        std::fs::write(&sail, "x").unwrap();
        let out = mirror_dir_for(&sail).unwrap();
        assert_eq!(out, tmp.path().join(".deck/packages/foo/src"));
    }

    #[test]
    fn default_matches_deck_out_dirname() {
        assert_eq!(DEFAULT_OUT_DIR, ".deck");
        assert_eq!(sail_out_dir(), ".deck");
    }
}
