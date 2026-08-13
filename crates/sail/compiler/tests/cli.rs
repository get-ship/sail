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
    let input = manifest_dir().join("tests/fixtures/hello.sail");
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
fn cli_tokens_writes_three_artifacts() {
    let bin = binary_path();
    if !bin.exists() {
        eprintln!("binary not built; run `cargo build -p sail-compiler`");
        return;
    }
    let workspace = manifest_dir()
        .ancestors()
        .find(|p| {
            p.join("Cargo.toml").exists()
                && std::fs::read_to_string(p.join("Cargo.toml"))
                    .map(|s| s.contains("[workspace]"))
                    .unwrap_or(false)
        })
        .expect("workspace root")
        .to_path_buf();
    let model = workspace.join("packages/sail/tokens/model");
    if !model.is_dir() {
        eprintln!("skip: tokens model missing at {}", model.display());
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("src");
    let output = Command::new(&bin)
        .args(["tokens"])
        .arg("--model")
        .arg(&model)
        .arg("--out")
        .arg(&out)
        .output()
        .expect("invoke sail-compiler tokens");
    assert!(
        output.status.success(),
        "tokens CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.join("tokens.css").is_file());
    assert!(out.join("themes.css").is_file());
    assert!(out.join("token-registry.json").is_file());
    let css = std::fs::read_to_string(out.join("tokens.css")).unwrap();
    assert!(css.contains("GENERATED FILE"));
    assert!(css.contains("--space-4:"));
    // Re-run is stable.
    let again = Command::new(&bin)
        .args(["tokens", "--model"])
        .arg(&model)
        .args(["--out"])
        .arg(&out)
        .status()
        .expect("re-invoke");
    assert!(again.success());
    let css2 = std::fs::read_to_string(out.join("tokens.css")).unwrap();
    assert_eq!(css, css2);
}

#[test]
fn cli_renders_diagnostic_with_path() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let bad = tmp.path().join("bad.sail");
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
        stderr.contains("bad.sail"),
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

#[test]
fn cli_directory_builds_two_fixtures() {
    let bin = binary_path();
    if !bin.exists() {
        eprintln!("binary not built; run `cargo build -p sail-compiler`");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    // Directory builds auto-tag from the stem; custom-element names need a hyphen.
    std::fs::copy(
        manifest_dir().join("tests/fixtures/hello.sail"),
        src.join("hello-world.sail"),
    )
    .unwrap();
    std::fs::copy(
        manifest_dir().join("tests/fixtures/hello.sail"),
        src.join("hello-again.sail"),
    )
    .unwrap();
    let out = tmp.path().join("out");
    let output = Command::new(&bin)
        .args(["build"])
        .arg(&src)
        .args(["--out"])
        .arg(&out)
        .output()
        .expect("invoke sail-compiler directory build");
    assert!(
        output.status.success(),
        "directory build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.join("hello-world.js").exists());
    assert!(out.join("hello-again.js").exists());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("compiled 2 file"),
        "expected two-file summary, got: {stdout}"
    );
}
