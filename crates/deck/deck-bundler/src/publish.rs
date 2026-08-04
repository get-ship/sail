//! Atomic publication of a staged app build (dev-loop-atomic).
//!
//! `bundle_app` used to delete the live output dir up front and write
//! `index.html` last, so every request that landed during a watch rebuild saw
//! a missing scaffold (`scaffold '<app>' not built (missing .deck/index.html)`)
//! for the whole rebuild window. The build now writes into a sibling staging
//! dir ([`stage_path_for`]) and [`publish_build`] promotes it: every staged
//! file is renamed over its destination — a same-filesystem `rename` is
//! atomic, so a reader sees the old bytes or the new bytes, never a missing or
//! partial file — with `index.html` renamed LAST so the scaffold never
//! references hashed assets that aren't on disk yet. Stale files from the
//! previous build are pruned AFTER the swap, and only inside top-level entries
//! this build actually produced — foreign trees sharing the out dir (e.g. the
//! bundler's on-demand `routes/<name>/route.js` route bundles) are never
//! touched.

use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Sibling staging dir for `dest`: `<dest>.stage`. Same parent directory ⇒
/// same filesystem, so the per-file `rename` into the destination is atomic.
pub fn stage_path_for(dest: &Path) -> PathBuf {
    let leaf = dest.file_name().and_then(|s| s.to_str()).unwrap_or("out");
    dest.with_file_name(format!("{leaf}.stage"))
}

/// Every file under `stage` as a sorted, stage-relative path list — with
/// `index.html` moved to the END. The scaffold references the hashed assets,
/// so it must be the last file swapped in: any reader mid-publish sees either
/// the complete old build or (old scaffold + superset of assets), never a
/// scaffold pointing at files that don't exist yet.
fn ordered_rel_files(stage: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(stage, Path::new(""), &mut files);
    files.sort();
    if let Some(pos) = files.iter().position(|p| p == Path::new("index.html")) {
        let index = files.remove(pos);
        files.push(index);
    }
    files
}

fn collect_files(root: &Path, rel: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(root.join(rel)) else {
        return;
    };
    for entry in rd.flatten() {
        let child = rel.join(entry.file_name());
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => collect_files(root, &child, out),
            Ok(ft) if ft.is_file() => out.push(child),
            _ => {}
        }
    }
}

/// Promote the staged build at `stage` into the live `dest` tree.
///
/// Each staged file is renamed over its destination (atomic replace on the
/// same filesystem), `index.html` last. When `prune` is set, files under
/// `dest` that this build did not produce are removed afterward — but ONLY
/// top-level files and files inside top-level directories the stage itself
/// wrote (`assets/`, `_deck/`, …). A top-level directory the stage did not
/// produce is foreign (another writer owns it — e.g. the bundler's
/// `routes/` route bundles) and is left untouched. The staging dir is removed
/// on success.
pub fn publish_build(stage: &Path, dest: &Path, prune: bool) -> Result<()> {
    let files = ordered_rel_files(stage);
    std::fs::create_dir_all(dest)
        .with_context(|| format!("create publish dir {}", dest.display()))?;
    for rel in &files {
        let to = dest.join(rel);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        std::fs::rename(stage.join(rel), &to)
            .with_context(|| format!("publish {}", to.display()))?;
    }
    if prune {
        let manifest: HashSet<&Path> = files.iter().map(PathBuf::as_path).collect();
        let owned_dirs: HashSet<OsString> = files
            .iter()
            .filter_map(|p| {
                let mut comps = p.components();
                let first = comps.next()?;
                // Only paths with a SECOND component have a top-level dir.
                comps.next()?;
                Some(first.as_os_str().to_os_string())
            })
            .collect();
        prune_stale(dest, &manifest, &owned_dirs)?;
    }
    let _ = std::fs::remove_dir_all(stage);
    Ok(())
}

/// Remove `dest` files this build did not produce: top-level files not in the
/// manifest, and unmanifested files (plus emptied subdirs) inside the
/// top-level dirs the build owns. Foreign top-level dirs are skipped.
fn prune_stale(
    dest: &Path,
    manifest: &HashSet<&Path>,
    owned_dirs: &HashSet<OsString>,
) -> Result<()> {
    let Ok(rd) = std::fs::read_dir(dest) else {
        return Ok(());
    };
    for entry in rd.flatten() {
        let name = entry.file_name();
        let path = entry.path();
        match entry.file_type() {
            Ok(ft) if ft.is_file() => {
                if !manifest.contains(Path::new(&name)) {
                    std::fs::remove_file(&path)
                        .with_context(|| format!("prune stale {}", path.display()))?;
                }
            }
            Ok(ft) if ft.is_dir() => {
                if owned_dirs.contains(&name) {
                    prune_dir(dest, Path::new(&name), manifest)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Recursively remove unmanifested files under `dest/<rel>`; remove a subdir
/// once it is empty (the top-level owned dir itself is kept).
fn prune_dir(dest: &Path, rel: &Path, manifest: &HashSet<&Path>) -> Result<()> {
    let abs = dest.join(rel);
    let Ok(rd) = std::fs::read_dir(&abs) else {
        return Ok(());
    };
    for entry in rd.flatten() {
        let child_rel = rel.join(entry.file_name());
        let child_abs = entry.path();
        match entry.file_type() {
            Ok(ft) if ft.is_file() => {
                if !manifest.contains(child_rel.as_path()) {
                    std::fs::remove_file(&child_abs)
                        .with_context(|| format!("prune stale {}", child_abs.display()))?;
                }
            }
            Ok(ft) if ft.is_dir() => {
                prune_dir(dest, &child_rel, manifest)?;
                // Drop the subdir if the prune emptied it (fails when
                // non-empty, which is exactly the keep case — ignore).
                let _ = std::fs::remove_dir(&child_abs);
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn write(p: &Path, s: &str) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, s).unwrap();
    }

    /// The publish swaps new content in, prunes the previous build's stale
    /// hashed assets, and NEVER touches a foreign top-level tree (the
    /// bundler's `routes/` route bundles share the out dir).
    #[test]
    fn publish_swaps_prunes_and_preserves_foreign_trees() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join(".deck");
        // Previous build + a bundler-owned route bundle.
        write(&dest.join("index.html"), "old");
        write(&dest.join("assets/entry-OLD.js"), "old-entry");
        write(&dest.join("stale-public.txt"), "removed-from-public");
        write(&dest.join("routes/git/route.js"), "bundler-owned");
        // T1 B1 app themes (foreign top-level dir — must survive prune).
        write(
            &dest.join("themes/themes.css"),
            "/* thin app themes */ :root[theme='ship']{}",
        );
        // Bare top-level themes.css would be pruned (regression guard for old path).
        write(&dest.join("themes.css"), "/* doomed top-level */");
        // Staged new build.
        let stage = stage_path_for(&dest);
        write(&stage.join("index.html"), "new");
        write(&stage.join("assets/entry-NEW.js"), "new-entry");
        write(&stage.join("favicon.ico"), "icon");

        publish_build(&stage, &dest, true).unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.join("index.html")).unwrap(),
            "new"
        );
        assert!(
            dest.join("assets/entry-NEW.js").exists(),
            "new asset published"
        );
        assert!(
            dest.join("themes/themes.css").is_file(),
            "B1 app themes under .deck/themes/ must survive publish prune"
        );
        assert!(
            !dest.join("themes.css").exists(),
            "bare top-level themes.css must still be pruned (foreign file)"
        );
        assert!(
            !dest.join("assets/entry-OLD.js").exists(),
            "stale asset pruned"
        );
        assert!(
            !dest.join("stale-public.txt").exists(),
            "stale top-level file pruned"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("routes/git/route.js")).unwrap(),
            "bundler-owned",
            "foreign top-level tree is never pruned"
        );
        assert!(!stage.exists(), "staging dir removed after publish");
    }

    /// `prune: false` (a non-`.deck`/`dist` out dir) overlays without deleting
    /// anything — the pre-fix no-clean behavior for custom out dirs.
    #[test]
    fn publish_without_prune_overlays_only() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("custom-out");
        write(&dest.join("assets/entry-OLD.js"), "old");
        let stage = stage_path_for(&dest);
        write(&stage.join("index.html"), "new");
        publish_build(&stage, &dest, false).unwrap();
        assert!(
            dest.join("assets/entry-OLD.js").exists(),
            "no prune ⇒ nothing deleted"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("index.html")).unwrap(),
            "new"
        );
    }

    /// `index.html` is the LAST file published: the scaffold must never point
    /// at hashed assets that aren't on disk yet.
    #[test]
    fn index_html_publishes_last() {
        let tmp = tempfile::tempdir().unwrap();
        let stage = tmp.path().join("s");
        write(&stage.join("index.html"), "x");
        write(&stage.join("assets/z-later-alphabetically.js"), "x");
        write(&stage.join("assets/a.css"), "x");
        let files = ordered_rel_files(&stage);
        assert_eq!(files.last().unwrap(), Path::new("index.html"));
        assert_eq!(files.len(), 3);
    }

    /// The scaffold-availability invariant the whole module exists for: across
    /// MANY repeated publishes over a live destination, `index.html` is never
    /// absent for even one observation. The pre-fix `remove_dir_all` +
    /// write-last flow fails this immediately.
    #[test]
    fn index_html_never_absent_across_repeated_publishes() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join(".deck");
        write(&dest.join("index.html"), "v0");

        let stop = Arc::new(AtomicBool::new(false));
        let saw_missing = Arc::new(AtomicBool::new(false));
        let checker = {
            let dest = dest.clone();
            let stop = stop.clone();
            let saw_missing = saw_missing.clone();
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    if !dest.join("index.html").exists() {
                        saw_missing.store(true, Ordering::Relaxed);
                    }
                    std::hint::spin_loop();
                }
            })
        };
        for i in 0..50 {
            let stage = stage_path_for(&dest);
            write(&stage.join("index.html"), &format!("v{i}"));
            write(&stage.join(format!("assets/entry-{i}.js")), "e");
            publish_build(&stage, &dest, true).unwrap();
        }
        stop.store(true, Ordering::Relaxed);
        checker.join().unwrap();
        assert!(
            !saw_missing.load(Ordering::Relaxed),
            "index.html must be observable at every instant across rebuild publishes"
        );
        // And the steady state is exactly the last build (stale assets pruned).
        assert_eq!(
            std::fs::read_to_string(dest.join("index.html")).unwrap(),
            "v49"
        );
        assert!(dest.join("assets/entry-49.js").exists());
        assert!(
            !dest.join("assets/entry-0.js").exists(),
            "old hashed assets pruned"
        );
    }
}
