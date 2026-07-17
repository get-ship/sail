//! Canonical output-tree layout for sail-compiler emit artifacts.
//!
//! Build artifacts (`.compiled.js`, `.custom-elements.json`, `.schema.json`,
//! `.sui.meta.json`, `.ssr.html`) land under the mirror tree at
//! `<repo-root>/.sail-out/<source-rel>/` (the neutral default — or `SAIL_OUT_DIR`
//! / the `--out` dir). The
//! `.sui.d.ts` type companion is the EXCEPTION: it is written ADJACENT to its
//! `.sui` source (see `compile_one` in the bin), so a controller's
//! `import type { Scope } from './x.sui'` resolves with no `rootDirs` into a
//! build dir. NOTE: `.ship/sail-out` is the Ship dynamic-views output and is
//! moving — it must not be load-bearing for component typing.
//!
//! Production-distribution bundles (`crates/deck/deck-core/assets/sui-*.js`)
//! are a separate concern and are NOT routed through this module.

use std::path::{Path, PathBuf};

/// Anchor file that marks the repo root: the workspace `Cargo.toml`
/// (the only one in this monorepo with a `[workspace]` table). Used by
/// the ancestor walk so the mirror layout works the same from any cwd.
const WORKSPACE_MARKER: &str = "[workspace]";

/// Default mirror sub-directory, relative to the repo root. The compiler is
/// framework-neutral and must NOT bake a `.ship/` (Ship) path: `.ship/sail-out`
/// is the Ship dynamic-views location, it is MOVING, and the host owns it. So
/// the default is a neutral, sui-owned scratch dir; a host (deck/shipd — "however
/// Ship compiles deck") points it at the real location via `SAIL_OUT_DIR`.
const DEFAULT_SAIL_OUT_DIR: &str = ".sail-out";

/// The mirror sub-directory in effect: `SAIL_OUT_DIR` env override, else the
/// neutral default. Relative to the repo root. `pub` so consumers (deck's
/// catalog) resolve the SAME location the compiler writes to — never a second
/// hardcoded copy that can drift.
pub fn sui_out_dir() -> String {
    std::env::var("SAIL_OUT_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SAIL_OUT_DIR.to_string())
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

/// Resolve the canonical mirror directory for a given `.sui` source path.
///
/// Layout: `<repo-root>/<SAIL_OUT_DIR>/<source-dir-relative-to-repo-root>/`,
/// where `<SAIL_OUT_DIR>` defaults to `.sail-out` and is host-configurable.
///
/// Returns `None` when no repo root can be located OR the source path is
/// outside the repo (e.g. a tempdir test fixture). Callers fall back to a
/// sibling `dist/` directory in that case so tests keep working.
pub fn mirror_dir_for(sui_path: &Path) -> Option<PathBuf> {
    let abs = absolutize(sui_path)?;
    let root = find_repo_root(&abs)?;
    let src_dir = abs.parent()?;
    let rel = src_dir.strip_prefix(&root).ok()?;
    Some(root.join(sui_out_dir()).join(rel))
}

/// Returns the mirror directory for a `.sui` source path, falling back to
/// a sibling `dist/` directory when the source is outside any detectable
/// repo (only happens in unit tests that compile from a tempdir).
pub fn resolve_out_dir(sui_path: &Path) -> PathBuf {
    if let Some(p) = mirror_dir_for(sui_path) {
        return p;
    }
    let src_dir = sui_path.parent().unwrap_or(Path::new("."));
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
        let sui = tmp.path().join("foo.sui");
        std::fs::write(&sui, "x").unwrap();
        let out = resolve_out_dir(&sui);
        assert!(out.ends_with("dist"));
    }

    #[test]
    fn mirrors_path_under_repo() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let src_dir = tmp.path().join("packages/foo/src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let sui = src_dir.join("bar.sui");
        std::fs::write(&sui, "x").unwrap();
        let out = mirror_dir_for(&sui).unwrap();
        assert_eq!(out, tmp.path().join(".sail-out/packages/foo/src"));
    }
}
