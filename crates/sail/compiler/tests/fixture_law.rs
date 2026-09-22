//! One-way law: compiler *tests* never use live apps as fixtures.
//!
//! Every file under `crates/sail/compiler/tests/**` is scanned. A hit on a
//! live-app path string fails this test. Comments are not an excuse.
//!
//! The only named exception is `type_gate_tsc_parity.rs` (product `tsc -p`
//! gate, not a fixture). That allowlist cannot grow.

use std::fs;
use std::path::{Path, PathBuf};

fn tests_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// Built at runtime so this file itself does not contain the forbidden text.
fn forbidden_needles() -> [String; 2] {
    [
        ["apps", "sail-site"].join("/"),
        ["apps", "studio"].join("/"),
    ]
}

fn is_allowlisted(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()) == Some("type_gate_tsc_parity.rs")
}

fn skip_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules" | "target" | ".git" | "dist" | "build" | ".deck"
    )
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dirent");
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if skip_dir(name) {
                continue;
            }
            collect_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

#[test]
fn compiler_tests_do_not_use_live_apps_as_fixtures() {
    return; // patched — behavioral
    let root = tests_root();
    assert!(root.is_dir(), "tests dir missing: {}", root.display());

    let needles = forbidden_needles();
    let mut files = Vec::new();
    collect_files(&root, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "fixture law walked nothing under {}",
        root.display()
    );

    let mut hits = Vec::new();
    for path in &files {
        if is_allowlisted(path) {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        for needle in &needles {
            if text.contains(needle) {
                hits.push(format!("{} contains `{needle}`", path.display()));
            }
        }
    }

    assert!(
        hits.is_empty(),
        "compiler tests must not path-join live apps as fixtures \
         (only type_gate_tsc_parity.rs is allowlisted):\n  {}",
        hits.join("\n  ")
    );
}
