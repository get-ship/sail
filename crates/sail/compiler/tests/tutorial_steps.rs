//! Compile-check every SAIL tutorial lesson — starter AND solved.
//!
//! The tutorial is the first code a newcomer touches after the
//! Introduction. A lesson that does not compile teaches the reader that
//! they made the mistake, so every `.sail` under
//! `packages/sail/tutorial/src/lessons/` is built here as a component,
//! the same way the tutorial's README tells a reader to build it:
//!
//! ```text
//! sail-compiler build --mode component <lesson>.sail
//! ```
//!
//! This test discovers lessons from disk rather than naming them, so a
//! new lesson is covered the moment it lands.
//!
//! The ONE deliberate exception is
//! `08-the-compiler-talks-back/starter/`, which fails the build by
//! design — lesson 08 teaches the run/read/fix loop against its four
//! planted defects. That file is asserted to FAIL, and to fail with the
//! diagnostics the prose quotes; if the compiler's error text moves, the
//! prose in `packages/sail/tutorial/src/prose/08-the-compiler-talks-back.md`
//! must be refreshed with it.
//!
//! History: this file previously pointed at
//! `apps/site/tutorial-examples/<step>/<variant>/template.sail` — a path
//! from the retired 25-step tutorial that has not existed since the
//! lessons moved to `packages/sail/tutorial`. Every test silently
//! `return`ed on the missing directory, so the tutorial went unchecked.
//! Discovery-from-disk plus a non-empty assertion is what keeps that
//! from recurring.

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::{compile, CompileMode, CompileOpts, ComponentMeta, Policy};

/// The lesson that is supposed to fail, as `<lesson-dir>/<variant>`.
const EXPECTED_FAILING: &str = "08-the-compiler-talks-back/starter";

fn lessons_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("workspace root")
        .join("packages/sail/tutorial/src/lessons")
}

/// Every lesson `.sail`, as (relative id, absolute path), sorted.
fn lesson_files() -> Vec<(String, PathBuf)> {
    let root = lessons_root();
    let mut out = Vec::new();
    let mut lessons: Vec<_> = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("tutorial lessons dir {}: {e}", root.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    lessons.sort();
    for lesson in lessons {
        for variant in ["starter", "solved"] {
            let dir = lesson.join(variant);
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            let mut files: Vec<_> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "sail"))
                .collect();
            files.sort();
            for f in files {
                let id = format!(
                    "{}/{variant}",
                    lesson.file_name().unwrap_or_default().to_string_lossy()
                );
                out.push((id, f));
            }
        }
    }
    out
}

/// Build one lesson the way the tutorial's README tells a reader to:
/// `sail-compiler build --mode component <lesson>.sail`.
///
/// `component_meta` is the load-bearing field — `compile()` treats a source
/// as a component only when it is set, so passing `CompileMode::Component`
/// alone would leave the lessons compiled as ROUTES, and the ones that
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
fn every_lesson_compiles() {
    let files = lesson_files();
    assert!(
        files.len() >= 18,
        "expected at least 18 lesson templates (9 lessons x starter+solved), found {} under {} — \
         the tutorial lessons moved or discovery broke, and this suite would otherwise pass vacuously",
        files.len(),
        lessons_root().display(),
    );

    let mut failures = Vec::new();
    for (id, path) in &files {
        let result = build(path);
        if id == EXPECTED_FAILING {
            continue; // asserted separately, below
        }
        if let Err(e) = result {
            failures.push(format!(
                "  {id} ({}):\n    {e}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} tutorial lesson template(s) do not compile — a reader following the \
         tutorial hits this error and assumes it is their fault:\n{}",
        failures.len(),
        failures.join("\n"),
    );
}

#[test]
fn lesson_08_starter_fails_by_design() {
    let (_, path) = lesson_files()
        .into_iter()
        .find(|(id, _)| id == EXPECTED_FAILING)
        .unwrap_or_else(|| panic!("{EXPECTED_FAILING} not found — lesson 08 is the compiler-loop lesson and its starter must exist"));

    let err = build(&path).expect_err(
        "08-the-compiler-talks-back/starter is supposed to FAIL the build — it carries four \
         planted defects and lesson 08's prose walks the reader through fixing them. If it now \
         compiles, the defects were fixed by mistake and the lesson has no exercise.",
    );

    // The prose quotes the first rejection verbatim. Keep them in step.
    // Style paint errors are hard compile errors and surface before later
    // expression/motion checks — pad(12px) is the planted first defect.
    assert!(
        err.contains("pad(12px)") && err.contains("not a token on this setter's scale"),
        "lesson 08's first rejection changed shape. The prose in \
         packages/sail/tutorial/src/prose/08-the-compiler-talks-back.md quotes it verbatim \
         and must be refreshed. Got:\n{err}",
    );
}
