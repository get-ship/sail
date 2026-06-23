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
        if p.is_dir() {
            walk(&p, out);
            continue;
        }
        if let Some(name) = p.file_name().and_then(|n| n.to_str())
            && (name.ends_with(".sui") || name.ends_with(".controller.ts")) {
                out.push(p);
            }
    }
}
