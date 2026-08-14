//! Compile-check owned tutorial lesson fixtures as components.
//!
//! Lessons the compiler test tree owns live under
//! `tests/fixtures/tutorial/`. Each `.sail` is built the same way a
//! reader is told to build a lesson:
//!
//! ```text
//! sail-compiler build --mode component <lesson>.sail
//! ```
//!
//! Discovery is from disk rather than a name list, so a new fixture is
//! covered the moment it lands. A non-empty assertion keeps the suite
//! from passing vacuously if the path moves.

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, Policy, compile};

fn lessons_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tutorial")
}

/// Every owned lesson `.sail`, as (relative id, absolute path), sorted.
fn lesson_files() -> Vec<(String, PathBuf)> {
    let root = lessons_root();
    let mut out = Vec::new();
    collect_sail(&root, &root, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn collect_sail(dir: &Path, root: &Path, out: &mut Vec<(String, PathBuf)>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("tutorial fixtures dir {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("dirent").path();
        if path.is_dir() {
            collect_sail(&path, root, out);
        } else if path.extension().is_some_and(|x| x == "sail") {
            let id = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");
            out.push((id, path));
        }
    }
}

/// Build one lesson the way the tutorial tells a reader to:
/// `sail-compiler build --mode component <lesson>.sail`.
///
/// `component_meta` is the load-bearing field — `compile()` treats a source
/// as a component only when it is set, so passing `CompileMode::Component`
/// alone would leave lessons compiled as ROUTES, and the ones that
/// declare `props:` would fail on a rule that does not apply to them. The
/// CLI derives the name from the file stem; so do we.
fn build(path: &Path) -> Result<(), String> {
    let src = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .ok_or_else(|| format!("bad path {}", path.display()))?;
    let opts = CompileOpts {
        mode: CompileMode::Component,
        source_path: Some(path.to_path_buf()),
        component_meta: Some(ComponentMeta { name: stem }),
        lint_overrides: Policy::overrides_for(path),
        ..Default::default()
    };
    compile(&src, opts).map(|_| ()).map_err(|e| e.to_string())
}

#[test]
fn every_owned_lesson_compiles() {
    let files = lesson_files();
    assert!(
        !files.is_empty(),
        "expected owned tutorial fixtures under {} — discovery broke, and this \
         suite would otherwise pass vacuously",
        lessons_root().display(),
    );

    let mut failures = Vec::new();
    for (id, path) in &files {
        if let Err(e) = build(path) {
            failures.push(format!(
                "  {id} ({}):\n    {e}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} owned tutorial fixture(s) do not compile:\n{}",
        failures.len(),
        failures.join("\n"),
    );
}
