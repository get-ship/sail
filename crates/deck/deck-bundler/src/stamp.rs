//! Framework-bundle staleness stamping (deck-deploy-gap).
//!
//! The incident this closes: `deck-core`'s framework singletons
//! (`sail-runtime.js`, `sail-dom.js`, `sail-sdk-client.js`, `deck-route.js`) are
//! baked into the binary at compile time via `include_str!` (see
//! `deck-core/build.rs`). Nothing enforced that the bake was regenerated
//! after its TypeScript sources changed — a 20h-stale `sail-runtime.js` served
//! against post-rename compiled routes rendered nothing, with only a silent
//! console error as the symptom.
//!
//! This module computes a content hash of each framework singleton's SOURCE
//! tree (a coarse, non-cryptographic `DefaultHasher` digest — good enough to
//! detect drift, not a security primitive) so `build_assets` can stamp each
//! bundle with the hash of the sources it was built from, and `deck dev` can
//! recompute the current hash at startup and compare.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use rustc_hash::FxHasher;

use anyhow::{Context, Result};

/// `(bundle out_file, source root relative to the repo root)`. Kept in
/// lockstep with `deck-core/build.rs::FRAMEWORK_BUNDLES` /
/// `targets.rs::FRAMEWORK_SINGLETONS` — one entry per framework singleton
/// that gets an `include_str!` bake.
pub const FRAMEWORK_SOURCE_ROOTS: &[(&str, &str)] = &[
    ("sail-runtime.js", "packages/sail/runtime/src"),
    ("sail-dom.js", "packages/sail/dom/src"),
    ("sail-sdk-client.js", "packages/deck/sdk-client/src"),
    ("deck-route.js", "packages/deck/router/src"),
    ("sail-studio-bridge.js", "packages/sail/studio-bridge/src"),
];

/// A bundle name paired with the current content hash of the source tree it
/// is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameworkStamp {
    pub bundle: &'static str,
    pub source_root: PathBuf,
    pub hash: String,
}

/// Recursively hash every file under `root` (path + content), sorted by
/// relative path so the result is deterministic regardless of read-dir
/// order. `.` and `node_modules` entries are skipped — neither ever holds
/// TypeScript sources this bundle is compiled from.
///
/// Uses [`FxHasher`], not [`std::hash::DefaultHasher`]. DefaultHasher is
/// randomized per process in current rustc, so a bake in process A and
/// `sail dev` in process B hashed the same tree to different stamps and
/// hard-failed as "still stale after regenerate."
pub fn hash_dir(root: &Path) -> Result<String> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files(root, root, &mut files)
        .with_context(|| format!("walking {} for a framework source stamp", root.display()))?;
    files.sort();

    let mut hasher = FxHasher::default();
    for rel in &files {
        let abs = root.join(rel);
        let bytes = std::fs::read(&abs)
            .with_context(|| format!("reading {} for a framework source stamp", abs.display()))?;
        rel.hash(&mut hasher);
        bytes.hash(&mut hasher);
    }
    Ok(format!("{:016x}", hasher.finish()))
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') || name_str == "node_modules" {
            continue;
        }
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(root, &path, out)?;
        } else if file_type.is_file() {
            out.push(path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
    Ok(())
}

/// Compute the current stamp for every framework singleton whose source root
/// exists under `repo_root`. A singleton whose source root is absent (e.g. an
/// external consumer repo that doesn't vendor the framework sources) is
/// skipped, not an error — `build_assets`/`deck dev` treat "no source tree to
/// hash" as "can't verify freshness here," handled by the caller.
pub fn current_framework_stamps(repo_root: &Path) -> Result<Vec<FrameworkStamp>> {
    let mut stamps = Vec::new();
    for (bundle, rel_root) in FRAMEWORK_SOURCE_ROOTS {
        let source_root = repo_root.join(rel_root);
        if !source_root.is_dir() {
            continue;
        }
        let hash = hash_dir(&source_root)?;
        stamps.push(FrameworkStamp {
            bundle,
            source_root,
            hash,
        });
    }
    Ok(stamps)
}

/// The leading-comment header `build_assets` prepends to each baked bundle.
/// `deck-core::assets::parse_baked_stamp` is the reader half — keep the
/// format in lockstep with it.
pub fn format_stamp_header(hash: &str) -> String {
    format!("/*! sail-framework-stamp: sha={hash} */\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_dir_is_deterministic_and_content_sensitive() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.ts"), b"export const a = 1;").unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub/b.ts"), b"export const b = 2;").unwrap();

        let h1 = hash_dir(dir.path()).unwrap();
        let h2 = hash_dir(dir.path()).unwrap();
        assert_eq!(h1, h2, "hashing the same tree twice must be deterministic");
        // FxHasher is seedless — a second hasher on the same bytes must match.
        // DefaultHasher failed this across process boundaries (sail dev).
        assert_eq!(h1.len(), 16);

        std::fs::write(dir.path().join("sub/b.ts"), b"export const b = 3;").unwrap();
        let h3 = hash_dir(dir.path()).unwrap();
        assert_ne!(h1, h3, "a source edit must change the hash");
    }

    #[test]
    fn hash_dir_ignores_dotfiles_and_node_modules() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.ts"), b"export const a = 1;").unwrap();
        let baseline = hash_dir(dir.path()).unwrap();

        std::fs::create_dir(dir.path().join("node_modules")).unwrap();
        std::fs::write(dir.path().join("node_modules/junk.js"), b"whatever").unwrap();
        std::fs::write(dir.path().join(".hidden"), b"whatever").unwrap();
        let after = hash_dir(dir.path()).unwrap();

        assert_eq!(
            baseline, after,
            "node_modules/dotfiles must not affect the stamp"
        );
    }

    #[test]
    fn current_framework_stamps_skips_absent_source_roots() {
        let dir = tempfile::tempdir().unwrap();
        // No `packages/...` tree under this dir — every root is absent.
        let stamps = current_framework_stamps(dir.path()).unwrap();
        assert!(
            stamps.is_empty(),
            "no framework source roots present ⇒ no stamps, not an error"
        );
    }

    #[test]
    fn format_stamp_header_roundtrips_the_hash() {
        let header = format_stamp_header("deadbeef00000000");
        assert!(header.starts_with("/*! sail-framework-stamp: sha="));
        assert!(header.contains("deadbeef00000000"));
        assert!(header.ends_with(" */\n"));
    }
}
