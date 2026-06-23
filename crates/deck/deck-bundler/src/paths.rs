//! Dev-time path resolution for the deck framework tooling.
//!
//! This is a deck-local copy of the std-only ancestor walk the framework
//! tooling needs (#273) — it lets `crates/deck/` stay ship-free.
//! Semantics: walk ancestors from `start` and return the **topmost** directory
//! that contains a `Cargo.toml`. A nested crate's `Cargo.toml` is a member
//! manifest, not the workspace root — the workspace root is whichever
//! ancestor has the most-generality `Cargo.toml` (the topmost match).
//!
//! Returns `None` when no `Cargo.toml` is found within 16 ancestor levels;
//! callers should treat that as "not in a Cargo workspace."
//!
//! The 16-level cap is a paranoia guard against symlink loops / pathological
//! FS layouts, not a documented limit.

use std::path::{Path, PathBuf};

/// Walk up from `start` to the topmost ancestor that holds a `Cargo.toml`.
///
/// The topmost (not the nearest) Cargo.toml is the workspace root in a
/// nested-crate layout — the inner Cargo.toml's are member manifests.
pub fn topmost_cargo_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start;
    let mut topmost: Option<PathBuf> = None;
    for _ in 0..16 {
        if cur.join("Cargo.toml").exists() {
            topmost = Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    topmost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topmost_cargo_root_finds_topmost() {
        // Build a fake nested-workspace layout under tempdir.
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\nmembers=[]").unwrap();
        let inner = root.path().join("crates/inner");
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(inner.join("Cargo.toml"), "[package]\nname=\"inner\"").unwrap();

        // From the inner crate, we want the OUTER workspace root, not the inner.
        let found = topmost_cargo_root(&inner).unwrap();
        assert_eq!(found, root.path());
    }

    #[test]
    fn topmost_cargo_root_returns_none_outside_workspace() {
        let root = tempfile::tempdir().unwrap();
        assert!(topmost_cargo_root(root.path()).is_none());
    }

    #[test]
    fn topmost_cargo_root_walks_three_levels() {
        // The deck-cli/build path calls this on a deeply-nested shell root
        // (e.g. `apps/shell/` inside a space scaffold). The walk must skip
        // the inner `apps/shell/Cargo.toml` (member manifest) and return the
        // monorepo root that holds `[workspace]`.
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\nmembers=[]").unwrap();
        let deep = root.path().join("a/b/c");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("Cargo.toml"), "[package]\nname=\"deep\"").unwrap();

        let found = topmost_cargo_root(&deep).unwrap();
        assert_eq!(found, root.path());
    }
}
