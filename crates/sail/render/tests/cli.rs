//! Smoke test for the `sail-render` CLI binary.

use std::process::Command;

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sail-render"));
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

#[test]
fn cli_renders_png() {
    let out = tempfile::NamedTempFile::with_suffix(".png").unwrap();
    let output = bin()
        .args([
            "--source",
            "tests/fixtures/golden.sail",
            "--viewport",
            "800x600",
            "--out",
            out.path().to_str().unwrap(),
            "--fonts-dir",
            "tests/fixtures/fonts",
        ])
        .output()
        .expect("spawning sail-render");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "sail-render exited with {}:\n{stderr}",
        output.status
    );

    let meta = std::fs::metadata(out.path()).unwrap();
    assert!(meta.len() > 0, "PNG output must be non-empty");
}
