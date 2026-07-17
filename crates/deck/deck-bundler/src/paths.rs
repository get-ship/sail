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

/// True when `dir/package.json` declares a `workspaces` field (the bun/npm
/// workspace marker — array form or `{ packages: [...] }` form). The actual
/// glob parsing lives in `catalog::workspace`; here we only need to know
/// whether this dir IS a workspace root.
fn declares_js_workspaces(dir: &Path) -> bool {
    let Ok(body) = std::fs::read_to_string(dir.join("package.json")) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) else {
        return false;
    };
    matches!(json.get("workspaces"), Some(ws) if ws.is_array() || ws.is_object())
}

/// Walk up from `start` to the WORKSPACE ROOT for a build — the directory the
/// bundler resolves `@ship/<pkg>` workspace sources and node_modules against.
///
/// A Cargo workspace wins (the sail monorepo): when an ancestor holds a
/// `Cargo.toml` the topmost one is the root, identical to [`topmost_cargo_root`].
/// Only when there is NO Cargo workspace (a pure-JS consumer like `ship-apps`)
/// do we fall back to the topmost ancestor whose `package.json` declares a
/// `workspaces` field — the bun/npm workspace root. Without this, a build run
/// from an app subdir (`ship-apps/apps/helm`) would resolve `@ship/<pkg>`
/// against the app dir and MISS its sibling workspace packs (`@ship/agents`,
/// …), externalizing them as bare specifiers the importmap never covers — a
/// blank page at runtime.
///
/// `None` when neither marker is found within 16 ancestor levels; callers treat
/// that as "not in a workspace" (a truly standalone app) and fall back to the
/// project dir.
pub fn topmost_workspace_root(start: &Path) -> Option<PathBuf> {
    // A Cargo workspace takes precedence — preserves the sail-monorepo path
    // byte-for-byte (the JS fallback only fires when there is no Cargo.toml).
    if let Some(cargo) = topmost_cargo_root(start) {
        return Some(cargo);
    }
    let mut cur = start;
    let mut topmost: Option<PathBuf> = None;
    for _ in 0..16 {
        if declares_js_workspaces(cur) {
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

    /// A pure-JS bun/npm workspace (no `Cargo.toml`): the workspace root is the
    /// topmost ancestor whose `package.json` declares `workspaces`. Built from an
    /// app subdir (`apps/helm`), the resolver must climb past the app's own
    /// `package.json` (no `workspaces`) to the root — the fix that lets sibling
    /// pack discovery (`@ship/agents`) reach the enclosing workspace.
    #[test]
    fn topmost_workspace_root_finds_js_workspace_from_app_subdir() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        std::fs::write(
            r.join("package.json"),
            r#"{"name":"ws","workspaces":["apps/*","packages/*"]}"#,
        )
        .unwrap();
        let app = r.join("apps/helm");
        std::fs::create_dir_all(&app).unwrap();
        // The app's OWN package.json declares no `workspaces` — must be skipped.
        std::fs::write(app.join("package.json"), r#"{"name":"@ship/helm"}"#).unwrap();

        let found = topmost_workspace_root(&app).unwrap();
        assert_eq!(found, r, "resolves the enclosing JS workspace root, not the app dir");
    }

    /// The `{ packages: [...] }` object form of `workspaces` also counts.
    #[test]
    fn topmost_workspace_root_accepts_object_workspaces_form() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        std::fs::write(
            r.join("package.json"),
            r#"{"name":"ws","workspaces":{"packages":["apps/*"]}}"#,
        )
        .unwrap();
        let app = r.join("apps/hub");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::write(app.join("package.json"), r#"{"name":"@ship/hub"}"#).unwrap();

        assert_eq!(topmost_workspace_root(&app).unwrap(), r);
    }

    /// A Cargo workspace WINS — the JS fallback only fires when there is no
    /// `Cargo.toml`, so the sail-monorepo path is byte-for-byte unchanged.
    #[test]
    fn topmost_workspace_root_prefers_cargo() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        std::fs::write(r.join("Cargo.toml"), "[workspace]\nmembers=[]").unwrap();
        std::fs::write(r.join("package.json"), r#"{"workspaces":["x/*"]}"#).unwrap();
        let inner = r.join("crates/inner");
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(inner.join("Cargo.toml"), "[package]\nname=\"inner\"").unwrap();

        assert_eq!(topmost_workspace_root(&inner).unwrap(), r);
    }

    /// A truly standalone app (no Cargo, no enclosing `workspaces`) yields
    /// `None` — the caller falls back to the project dir.
    #[test]
    fn topmost_workspace_root_none_when_standalone() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("package.json"), r#"{"name":"solo"}"#).unwrap();
        assert!(topmost_workspace_root(root.path()).is_none());
    }
}
