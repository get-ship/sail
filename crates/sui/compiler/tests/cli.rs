//! CLI smoke tests. The binary must compile a fixture file (or
//! directory) into the requested output directory and surface the
//! diagnostic on bad input.

use std::path::PathBuf;
use std::process::Command;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn binary_path() -> PathBuf {
    let mut p = manifest_dir();
    while !p.join("target").exists() {
        let parent = p.parent().expect("workspace root");
        p = parent.to_path_buf();
    }
    let exe = if cfg!(windows) {
        "sail-compiler.exe"
    } else {
        "sail-compiler"
    };
    p.join("target").join("debug").join(exe)
}

#[test]
fn cli_builds_a_single_fixture() {
    let bin = binary_path();
    if !bin.exists() {
        eprintln!("binary not built; run `cargo build -p sail-compiler`");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let input = manifest_dir()
        .join("tests/fixtures/hello.sui");
    let out = tmp.path().to_path_buf();
    let status = Command::new(&bin)
        .args(["build"])
        .arg(&input)
        .args(["--out"])
        .arg(&out)
        .status()
        .expect("invoke sail-compiler");
    assert!(status.success(), "CLI failed");
    let js = out.join("hello.js");
    assert!(js.exists(), "expected {} to exist", js.display());
    let body = std::fs::read_to_string(js).unwrap();
    // The compiler wraps `component(...)` in `Object.assign(...)` so the
    // default export carries `_compilerVersion`. Match either shape.
    assert!(
        body.contains("export default component")
            || body.contains("export default Object.assign(component"),
        "expected component default export, got:\n{body}"
    );
}

#[test]
fn cli_renders_diagnostic_with_path() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let bad = tmp.path().join("bad.sui");
    std::fs::write(&bad, r#"<p>{a = 1}</p>"#).unwrap();
    let out = tmp.path().join("out");
    let output = Command::new(&bin)
        .args(["build"])
        .arg(&bad)
        .args(["--out"])
        .arg(&out)
        .output()
        .expect("invoke");
    assert!(!output.status.success(), "expected failure");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("bad.sui"),
        "stderr should mention the source path: {stderr}"
    );
    assert!(
        stderr.contains("line 1"),
        "stderr should mention a line number: {stderr}"
    );
    assert!(
        stderr.contains("assignments"),
        "stderr should describe the error: {stderr}"
    );
}
