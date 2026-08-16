//! `sail editor setup` writes user config. HOME / XDG are the temp dir.
//! Default command is `sail` on PATH — never this worktree's debug bin
//! unless `--bin` is passed.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

fn sail_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sail"))
}

fn run_setup(home: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(sail_bin())
        .args(args)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env_remove("HELIX_CONFIG")
        .env_remove("HELIX_RUNTIME")
        .output()
        .expect("spawn sail")
}

#[test]
fn help_lists_editor() {
    let out = Command::new(sail_bin())
        .arg("--help")
        .output()
        .expect("help");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.lines()
            .any(|l| l.split_whitespace().next() == Some("editor")),
        "{text}"
    );
}

#[test]
fn helix_setup_writes_path_sail_and_extracted_grammar() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let out = run_setup(home, &["editor", "setup", "helix"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "status={} stderr={stderr} stdout={stdout}",
        out.status
    );
    let langs = home.join(".config/helix/languages.toml");
    let text = fs::read_to_string(&langs).unwrap();
    assert!(text.contains("# sail-editor"), "{text}");
    assert!(text.contains("command = \"sail\""), "{text}");
    assert!(!text.contains("target/debug/sail"), "{text}");
    let scm = home.join(".local/share/sail/grammar/queries/highlights.scm");
    assert!(
        scm.is_file(),
        "extracted grammar missing: {}",
        scm.display()
    );
    let repo_scm = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../packages/sail/grammar/queries/highlights.scm");
    assert_eq!(
        fs::read_to_string(&scm).unwrap(),
        fs::read_to_string(repo_scm).unwrap(),
        "embedded highlights.scm must match the repo query"
    );
}

#[test]
fn helix_setup_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    assert!(run_setup(home, &["editor", "setup", "helix"])
        .status
        .success());
    assert!(run_setup(home, &["editor", "setup", "helix"])
        .status
        .success());
    let text = fs::read_to_string(home.join(".config/helix/languages.toml")).unwrap();
    assert_eq!(text.matches("# sail-editor").count(), 1, "{text}");
}

#[test]
fn nvim_setup_writes_plugin_with_path_sail() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let out = run_setup(home, &["editor", "setup", "nvim"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lua = fs::read_to_string(home.join(".config/nvim/plugin/sail.lua")).unwrap();
    assert!(lua.contains("sail"), "{lua}");
    assert!(lua.contains("lsp"), "{lua}");
    assert!(!lua.contains("target/debug"));
}

#[test]
fn helix_setup_bin_override_is_explicit() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let custom = tmp.path().join("custom-sail");
    fs::write(&custom, "#!/bin/sh\n").unwrap();
    let out = run_setup(
        home,
        &[
            "editor",
            "setup",
            "helix",
            "--bin",
            custom.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = fs::read_to_string(home.join(".config/helix/languages.toml")).unwrap();
    assert!(text.contains(&custom.display().to_string()), "{text}");
}
