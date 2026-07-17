//! Golden-fixture parity suite for `sail-compiler`.
//!
//! Each fixture under `tests/golden/<name>.sui` is compiled with
//! `CompileOpts::default()` and the full `CompileOutput` is captured
//! as JSON in `<name>.expected.json`. The expected JSON locks the
//! compiler's wire format so future drift surfaces as a test failure
//! with a clear text diff.
//!
//! Set `SAIL_GOLDEN_UPDATE=1` to rewrite the expected files. See
//! `tests/golden/README.md` for the workflow.

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::{compile, CompileOpts, CompileOutput};

mod common;
use common::strip_version_stamp;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn output_to_json(out: &CompileOutput) -> serde_json::Value {
    // `js` and `dts` are raw multi-line strings — the `_compilerVersion`
    // stamp lives on real newlines there, so normalize them with the
    // shared stripper before they're folded into the JSON (inside the
    // JSON they'd become escaped `\n` content, invisible to a
    // line-based pass). The `cem` / `schema` / `meta` JSON values still
    // carry the stamp as a real `"_compilerVersion"` field; that form
    // is handled by running `strip_version_stamp` over the final
    // pretty-printed text in `run_golden`.
    serde_json::json!({
        "js": strip_version_stamp(&out.js),
        "css": out.css,
        "cem": out.cem,
        "schema": out.schema,
        "dts": out.dts.as_deref().map(strip_version_stamp),
        "meta": out.meta,
    })
}

fn run_golden(name: &str) {
    let dir = golden_dir();
    let src_path = dir.join(format!("{name}.sui"));
    let expected_path = dir.join(format!("{name}.expected.json"));

    let source = fs::read_to_string(&src_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", src_path.display()));

    let out = compile(&source, CompileOpts::default())
        .unwrap_or_else(|e| panic!("compile {name}: {e:?}"));

    let actual = output_to_json(&out);
    // Route through the shared `_compilerVersion` normalizer — the
    // identity stamp carries a git sha + timestamp + dirty flag, all
    // non-deterministic. Stripping it (here and on the compare path)
    // keeps the golden fixtures deterministic, exactly as `snapshot.rs`
    // does for its snapshots.
    let actual_text = format!(
        "{}\n",
        strip_version_stamp(&serde_json::to_string_pretty(&actual).unwrap()),
    );

    let updating = std::env::var("SAIL_GOLDEN_UPDATE").is_ok();

    if updating {
        fs::write(&expected_path, &actual_text)
            .unwrap_or_else(|e| panic!("write {}: {e}", expected_path.display()));
        return;
    }

    let expected_text = match fs::read_to_string(&expected_path) {
        Ok(s) => s,
        Err(_) => {
            fs::write(&expected_path, &actual_text)
                .unwrap_or_else(|e| panic!("bootstrap-write {}: {e}", expected_path.display()));
            panic!(
                "golden {} did not exist; wrote it now — re-run to confirm",
                expected_path.display()
            );
        }
    };

    // Strip the expected side too — mirrors `snapshot.rs`, which
    // normalizes both sides so a stale stamped fixture still compares
    // clean against freshly-normalized output.
    let expected_text = strip_version_stamp(&expected_text);

    assert_eq_text(name, &expected_text, &actual_text, &expected_path);
}

fn assert_eq_text(name: &str, expected: &str, actual: &str, expected_path: &Path) {
    if expected == actual {
        return;
    }
    let diff = unified_diff(expected, actual);
    panic!(
        "golden mismatch for `{name}`\n  expected: {}\n  Set SAIL_GOLDEN_UPDATE=1 to refresh.\n\n{}",
        expected_path.display(),
        diff,
    );
}

/// Minimal line-level diff. Avoids pulling a diff crate into the
/// workspace just to print mismatches — this is only invoked on
/// failure paths.
fn unified_diff(expected: &str, actual: &str) -> String {
    let mut out = String::new();
    let exp_lines: Vec<&str> = expected.lines().collect();
    let act_lines: Vec<&str> = actual.lines().collect();
    let max = exp_lines.len().max(act_lines.len());
    for i in 0..max {
        let e = exp_lines.get(i).copied().unwrap_or("");
        let a = act_lines.get(i).copied().unwrap_or("");
        if e == a {
            continue;
        }
        out.push_str(&format!("L{:>4} -{}\n", i + 1, e));
        out.push_str(&format!("L{:>4} +{}\n", i + 1, a));
    }
    if out.is_empty() {
        out.push_str("(text differs but line-by-line scan found no diffs — check trailing newline / whitespace)\n");
    }
    out
}

#[test]
fn golden_simple() {
    run_golden("simple");
}

#[test]
fn golden_signal_text() {
    run_golden("signal-text");
}

#[test]
fn golden_event_handler() {
    run_golden("event-handler");
}
