//! Filesystem walk + classification for SUI authoring sources.
//!
//! Two file shapes are recognised:
//! - `*.sui` — template (parsed by `sui-compiler`)
//! - `*.controller.ts` — paired controller (parsed by `oxc_parser`)
//!
//! Plain `*.ts` files next to a `.sui` are not controllers — they're
//! helpers, types, or tests. The suffix carries the intent.

use std::path::{Path, PathBuf};

/// Recursively collect every `*.sui` and `*.controller.ts` file under `dir`.
pub fn collect(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
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
            && (name.ends_with(".sui") || name.ends_with(".controller.ts")) {
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
        Some("node_modules" | "dist" | "target" | ".git" | ".deck" | ".sui-out")
    )
}
