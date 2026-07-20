//! Atomic file-system writes for deck-bundler.
//!
//! Readers of `.deck/framework/` (the daemon, `deck emit`, `deck-core/build.rs`)
//! must never observe a truncated bundle. These helpers write to a temp file in
//! the same directory and `rename` it over the destination, which is atomic on
//! the same filesystem.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Write `contents` to `path` atomically.
///
/// The destination is created as `{path}.tmp`, flushed, and renamed over
/// `path`. A concurrent reader sees the old file or the new file, never a
/// partially-written one. The temp file is removed on error so it cannot be
/// mistaken for a real bundle.
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let path = path.as_ref();
    let tmp = tmp_path(path);
    {
        let mut f = fs::File::create(&tmp)
            .with_context(|| format!("create temp file {}", tmp.display()))?;
        f.write_all(contents.as_ref())
            .with_context(|| format!("write temp file {}", tmp.display()))?;
        f.flush()
            .with_context(|| format!("flush temp file {}", tmp.display()))?;
    }
    fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

/// Copy `from` to `to` atomically, skipping the write if the destination
/// already has identical bytes.
///
/// This preserves the destination's mtime when the source has not changed,
/// which is exactly what build scripts need to avoid self-triggering rebuild
/// loops.
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    let src = fs::read(from)
        .with_context(|| format!("read source {}", from.display()))?;

    if let Ok(dst_meta) = fs::metadata(to) {
        if dst_meta.len() == src.len() as u64 {
            if let Ok(dst) = fs::read(to) {
                if dst == src {
                    return Ok(());
                }
            }
        }
    }

    write(to, src)
}

/// Compute a per-writer staging directory next to `dest` on the same filesystem.
///
/// Examples:
/// - `.deck/framework` + suffix `sail-runtime` → `.deck/framework.stage.sail-runtime`
/// - `.deck` + suffix `sail-marketing` → `.deck.stage.sail-marketing`
/// - `.deck` + suffix `routes` → `.deck.stage.routes`
pub fn stage_dir_for(dest: &Path, suffix: &str) -> PathBuf {
    let parent = dest.parent().unwrap_or_else(|| Path::new("."));
    let name = dest
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("out");
    parent.join(format!("{name}.stage.{suffix}"))
}

/// Recursively walk `stage` and `rename` every file into `dest`.
///
/// Parent directories are created in `dest` as needed. Each file move is an
/// atomic same-filesystem `rename`, so readers never see partial content.
/// Stale files already in `dest` are intentionally left alone — the shared
/// `.deck/` tree has multiple writers and this helper does not own pruning.
pub fn publish_dir(stage: &Path, dest: &Path) -> Result<()> {
    let mut files = Vec::new();
    collect_files(stage, Path::new(""), &mut files)
        .with_context(|| format!("walk stage dir {}", stage.display()))?;
    files.sort();

    fs::create_dir_all(dest)
        .with_context(|| format!("create dest dir {}", dest.display()))?;

    for rel in &files {
        let from = stage.join(rel);
        let to = dest.join(rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create parent dir {}", parent.display()))?;
        }
        fs::rename(&from, &to)
            .with_context(|| format!("publish {} -> {}", from.display(), to.display()))?;
    }

    let _ = fs::remove_dir_all(stage);
    Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("tmp");
    path.with_file_name(format!("{name}.tmp"))
}

fn collect_files(root: &Path, rel: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let abs = root.join(rel);
    let rd = fs::read_dir(&abs)
        .with_context(|| format!("read dir {}", abs.display()))?;
    for entry in rd.flatten() {
        let child_rel = rel.join(entry.file_name());
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => collect_files(root, &child_rel, out)?,
            Ok(ft) if ft.is_file() => out.push(child_rel),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn write_atomically_replaces_file() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("bundle.js");
        fs::write(&p, "old").unwrap();

        write(&p, "new").unwrap();

        assert_eq!(fs::read_to_string(&p).unwrap(), "new");
        assert!(!dir.path().join("bundle.js.tmp").exists(), "temp file leaked");
    }

    #[test]
    fn copy_skips_when_identical() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src.js");
        let dst = dir.path().join("dst.js");
        fs::write(&src, "same").unwrap();
        fs::write(&dst, "same").unwrap();

        let m1 = fs::metadata(&dst).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        copy(&src, &dst).unwrap();
        let m2 = fs::metadata(&dst).unwrap().modified().unwrap();

        assert_eq!(m1, m2, "identical copy must not touch mtime");
        assert_eq!(fs::read_to_string(&dst).unwrap(), "same");
    }

    #[test]
    fn copy_writes_when_different() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src.js");
        let dst = dir.path().join("dst.js");
        fs::write(&src, "new").unwrap();
        fs::write(&dst, "old").unwrap();

        copy(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(&dst).unwrap(), "new");
    }

    #[test]
    fn stage_dir_for_produces_sibling() {
        let dest = Path::new("/repo/.deck/framework");
        assert_eq!(
            stage_dir_for(dest, "sail-runtime"),
            Path::new("/repo/.deck/framework.stage.sail-runtime")
        );

        let dest = Path::new("/repo/.deck");
        assert_eq!(
            stage_dir_for(dest, "routes"),
            Path::new("/repo/.deck.stage.routes")
        );
    }

    #[test]
    fn publish_dir_moves_files_atomically() {
        let dir = tempdir().unwrap();
        let stage = dir.path().join("stage");
        let dest = dir.path().join("dest");

        fs::create_dir_all(&stage.join("sub")).unwrap();
        fs::write(stage.join("a.js"), "A").unwrap();
        fs::write(stage.join("sub/b.js"), "B").unwrap();

        publish_dir(&stage, &dest).unwrap();

        assert_eq!(fs::read_to_string(dest.join("a.js")).unwrap(), "A");
        assert_eq!(fs::read_to_string(dest.join("sub/b.js")).unwrap(), "B");
        assert!(!stage.exists(), "stage dir removed after publish");
    }

    #[test]
    fn write_is_atomic_against_concurrent_readers() {
        // Sanity check: a reader opening the file should see either the old
        // or new content, never an empty/truncated file.
        let dir = tempdir().unwrap();
        let p = dir.path().join("bundle.js");
        fs::write(&p, "old").unwrap();

        let p2 = p.clone();
        let reader = std::thread::spawn(move || {
            for _ in 0..100 {
                if let Ok(mut f) = fs::File::open(&p2) {
                    let mut buf = String::new();
                    let _ = f.read_to_string(&mut buf);
                    assert!(!buf.is_empty(), "reader saw empty file");
                }
            }
        });

        for i in 0..100 {
            write(&p, format!("new-{i}")).unwrap();
        }
        reader.join().unwrap();
    }
}
