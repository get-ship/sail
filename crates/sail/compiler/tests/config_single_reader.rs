//! inv-sail-one-policy-resolver: `sail.config.json` is read (walked-up + parsed
//! from disk) in exactly ONE place across the toolchain. A second independent
//! reader is the drift this test forbids — the effective config for a file must
//! come from one resolver, not be re-derived per consumer.
//!
//! This is a source-level uniqueness test (the same family as the ABI-boundary
//! drift tests): it fails loudly when a new reader appears, which is what keeps
//! the single path single.

use std::fs;
use std::path::{Path, PathBuf};

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            walk_ok(&p).then(|| rs_files(&p, out));
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn walk_ok(p: &Path) -> bool {
    !matches!(
        p.file_name().and_then(|n| n.to_str()),
        Some("target" | "tests" | "node_modules")
    )
}

/// A file is a `sail.config.json` READER if it reads a file from disk AND names
/// the config file as a path it constructs — either the `"sail.config.json"`
/// literal joined onto a dir, or via the `CONFIG_FILE` const. A prose mention
/// in a comment (deck's `cem.rs` / `compile.rs`) does not construct a path, so
/// it does not match.
fn is_config_reader(text: &str) -> bool {
    let reads = text.contains("read_to_string") || text.contains("fs::read(");
    let constructs =
        text.contains("join(\"sail.config.json\")") || text.contains("join(CONFIG_FILE)");
    reads && constructs
}

#[test]
fn sail_config_has_a_single_reader() {
    // crates/sail/compiler -> crates/sail
    let sail_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut readers = Vec::new();
    for sub in ["compiler/src", "lint/src"] {
        let mut files = Vec::new();
        rs_files(&sail_root.join(sub), &mut files);
        for f in files {
            let text = fs::read_to_string(&f).unwrap_or_default();
            // Test modules (fixtures write/read sail.config.json) are not readers.
            let real = text.split("#[cfg(test)]").next().unwrap_or(&text);
            if is_config_reader(real) {
                readers.push(f.strip_prefix(sail_root).unwrap().to_path_buf());
            }
        }
    }
    assert_eq!(
        readers.len(),
        1,
        "sail.config.json must have exactly one reader (inv-sail-one-policy-resolver); found {}: {:#?}",
        readers.len(),
        readers
    );
}
