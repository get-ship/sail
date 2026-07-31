//! Filesystem walk + classification for SAIL authoring sources.
//!
//! Two file shapes are recognised:
//! - `*.sail` — template (parsed by `sail-compiler`)
//! - `*.controller.ts` — paired controller (parsed by `oxc_parser`)
//!
//! Plain `*.ts` files next to a `.sail` are not controllers — they're
//! helpers, types, or tests. The suffix carries the intent.

use std::path::{Path, PathBuf};

/// Recursively collect every `*.sail` and `*.controller.ts` file under `dir`.
pub fn collect(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        // file_type() does NOT follow symlinks: a symlinked directory (a
        // `node_modules` isolated-linker link back into the workspace) reads
        // as a symlink, not a dir, so the walk never descends the dependency
        // graph and cannot cycle. Authoring sources are real files.
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            if is_ignored_dir(&p) {
                continue;
            }
            walk(&p, out);
            continue;
        }
        if ft.is_file()
            && let Some(name) = p.file_name().and_then(|n| n.to_str())
            && (name.ends_with(".sail") || name.ends_with(".controller.ts"))
        {
            out.push(p);
        }
    }
}

/// Directories that never hold authoring sources and whose contents can be
/// symlink graphs (`node_modules`) or large build trees. Skipping them keeps
/// the walk on real source and off the dependency graph.
fn is_ignored_dir(p: &Path) -> bool {
    matches!(
        p.file_name().and_then(|n| n.to_str()),
        Some("node_modules" | "dist" | "target" | ".git" | ".deck" | ".sail-out")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    #[cfg(unix)]
    fn walk_skips_node_modules_and_survives_a_symlink_cycle() {
        // inv-sail-nodemodules-not-source: a source walk must not descend into
        // node_modules and must terminate even against a symlink cycle back into
        // the tree (the bun isolated-linker shape that hung compiles). If the
        // fix regressed, this test either hangs (cycle followed) or fails (a
        // node_modules path collected).
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let base = root.path();

        // a real authoring source the walk MUST find
        fs::create_dir_all(base.join("src/sail-x")).unwrap();
        fs::write(base.join("src/sail-x/sail-x.sail"), "<column></column>").unwrap();

        // node_modules with a source-shaped file inside — must be skipped whole
        fs::create_dir_all(base.join("node_modules/@ship")).unwrap();
        fs::write(
            base.join("node_modules/@ship/pkg.sail"),
            "<column></column>",
        )
        .unwrap();
        // a symlink cycle NOT under node_modules — proves file_type() no-follow
        symlink(base, base.join("src/loop")).unwrap();

        let found = collect(base);

        assert!(
            found.iter().any(|p| p.ends_with("sail-x.sail")),
            "the real source is collected: {found:?}"
        );
        assert!(
            found
                .iter()
                .all(|p| !p.components().any(|c| c.as_os_str() == "node_modules")),
            "no node_modules path is collected: {found:?}"
        );
    }
}
