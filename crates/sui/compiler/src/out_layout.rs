//! Canonical output-tree layout for sui-compiler emit artifacts.
//!
//! Tooling outputs (`.compiled.js`, `.sui.d.ts`, `.custom-elements.json`,
//! `.schema.json`, `.sui.meta.json`, `.ssr.html`) land under one project-
//! level mirror tree at `<repo-root>/.ship/sui-out/<source-rel>/` instead
//! of a per-component sibling `dist/` directory.
//!
//! Production-distribution bundles (`crates/deck/deck-core/assets/sui-*.js`)
//! are a separate concern and are NOT routed through this module.

use std::path::{Path, PathBuf};

/// Anchor file that marks the repo root: the workspace `Cargo.toml`
/// (the only one in this monorepo with a `[workspace]` table). Used by
/// the ancestor walk so the mirror layout works the same from any cwd.
const WORKSPACE_MARKER: &str = "[workspace]";

/// Default mirror sub-directory, relative to the repo root. SUI is a mechanical
/// part of Ship and this monorepo is a Ship project, so the output lives where
/// it lives on customer machines: under `.ship/` (gitignored). It is NOT a fixed
/// constant — a host configures it via `SUI_OUT_DIR` (a non-Ship SUI consumer
/// would point it elsewhere). We just don't dump at the repo root by default.
const DEFAULT_SUI_OUT_DIR: &str = ".ship/sui-out";

/// The mirror sub-directory in effect: `SUI_OUT_DIR` env override, else the
/// default. Relative to the repo root. Kept a function (not a const) so the
/// host can configure where compiler artifacts land.
fn sui_out_dir() -> String {
    std::env::var("SUI_OUT_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SUI_OUT_DIR.to_string())
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
/// Layout: `<repo-root>/<SUI_OUT_DIR>/<source-dir-relative-to-repo-root>/`,
/// where `<SUI_OUT_DIR>` defaults to `.ship/sui-out` and is host-configurable.
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
        assert_eq!(out, tmp.path().join(".ship/sui-out/packages/foo/src"));
    }
}
