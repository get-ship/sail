//! Path-math + probe unit tests for `sail_compiler::ts_native`.
//!
//! Fixtures are temp trees (script `tsc` + sibling `lib.d.ts`). They do not
//! run `--version` (7.0.2 panics without a real lib dir).

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::ts_native::{
    PINS_TOML, Pin, TS_NATIVE_VERSION, TsNativeSource, accept_lsp_help, expected_peer_tsc,
    parse_pins, peer_sidecar_dirs, pins, resolve_ts_native_from_exe,
};

fn write_file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn plant_sidecar(dir: &Path) {
    write_file(&dir.join("tsc"), "#!/bin/sh\nexit 0\n");
    write_file(&dir.join("lib.d.ts"), "// fixture\n");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dir.join("tsc")).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dir.join("tsc"), perms).unwrap();
    }
}

fn plant_exe(path: &Path) {
    write_file(path, "");
}

#[test]
fn pins_table_has_six_hashed_desktop_tuples() {
    let parsed = parse_pins(PINS_TOML).expect("pins.toml");
    assert_eq!(parsed.len(), 6, "public-cut pin table ships all six tuples");
    for pin in &parsed {
        assert_ne!(
            pin.sha256, "fill",
            "{}-{} sha256 still fill",
            pin.os, pin.arch
        );
        assert!(
            pin.ssri.starts_with("sha512-"),
            "{}-{} ssri",
            pin.os,
            pin.arch
        );
        assert_eq!(pin.sha256.len(), 64, "{}-{} sha256 hex", pin.os, pin.arch);
    }
    let linux_x64 = parsed
        .iter()
        .find(|p| p.os == "linux" && p.arch == "x64")
        .expect("linux-x64");
    assert_eq!(
        linux_x64.sha256,
        "7ecad6f67377e831856367ab062ef394f21506a611405bf8ac0ff039348637d3"
    );
    assert_eq!(pins().len(), 6);
    let _typed: &[Pin] = pins();
    assert_eq!(TS_NATIVE_VERSION, "7.0.2");
}

#[test]
fn peer_next_to_fake_exe_is_source_peer() {
    let tmp = tempfile::tempdir().unwrap();
    let exe = tmp.path().join("sail");
    plant_exe(&exe);
    plant_sidecar(&tmp.path().join("sail-ts-native"));

    let tool = resolve_ts_native_from_exe(&exe).expect("peer sidecar");
    assert_eq!(tool.source, TsNativeSource::Peer);
    assert_eq!(tool.program, tmp.path().join("sail-ts-native").join("tsc"));
    assert!(
        !tmp.path().join("node_modules").exists(),
        "fixture must not plant node_modules"
    );
}

#[test]
fn ancestor_node_modules_tsgo_ignored_by_path_math() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_file(
        &root.join("node_modules/.bin/tsgo"),
        "#!/bin/sh\necho leaked\n",
    );
    let exe = root.join("app/bin/sail");
    plant_exe(&exe);

    assert!(
        resolve_ts_native_from_exe(&exe).is_none(),
        "path-math must ignore ancestor node_modules/.bin/tsgo"
    );
}

#[test]
fn target_debug_shaped_exe_does_not_walk_parents_for_node_modules() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let leaked = root.join("node_modules/.bin/tsgo");
    write_file(&leaked, "#!/bin/sh\necho leaked\n");
    let exe = root.join("target/debug/sail");
    plant_exe(&exe);

    let got = resolve_ts_native_from_exe(&exe);
    if let Some(tool) = got {
        assert_ne!(
            tool.program, leaked,
            "must not resolve via parent walk of current_exe"
        );
        assert_eq!(tool.source, TsNativeSource::Peer);
    }
}

#[test]
fn deps_hop_finds_profile_peer() {
    let tmp = tempfile::tempdir().unwrap();
    let profile = tmp.path().join("target/debug");
    let exe = profile.join("deps/tsgo_proxy-deadbeef");
    plant_exe(&exe);
    plant_sidecar(&profile.join("sail-ts-native"));

    let tool = resolve_ts_native_from_exe(&exe).expect("deps hop");
    assert_eq!(tool.source, TsNativeSource::Peer);
    assert_eq!(tool.program, profile.join("sail-ts-native/tsc"));

    let dirs = peer_sidecar_dirs(&exe);
    assert!(
        dirs.iter().any(|d| d == &profile.join("sail-ts-native")),
        "deps hop dirs: {dirs:?}"
    );
    assert_eq!(expected_peer_tsc(&exe), profile.join("sail-ts-native/tsc"));
}

#[test]
fn probe_accepts_exit_2_and_stdio_on_stderr() {
    assert!(accept_lsp_help(2, "", "Usage of lsp:\n  -stdio\n  -pipe\n",));
    assert!(accept_lsp_help(0, "stdio\n", ""));
    assert!(accept_lsp_help(1, "", "listen on stdio\n"));
}

#[test]
fn probe_rejects_unknown_compiler_option() {
    assert!(!accept_lsp_help(
        1,
        "error TS5023: Unknown compiler option '--lsp'.\n",
        "",
    ));
    assert!(!accept_lsp_help(2, "", "Unknown compiler option '--lsp'"));
    assert!(!accept_lsp_help(3, "", "stdio"));
    assert!(!accept_lsp_help(2, "hello", "world"));
}

#[test]
fn probe_script_fixture_exit_2_stdio() {
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("tsc");
    write_file(
        &script,
        "#!/bin/sh\necho 'Usage of lsp:' >&2\necho '  -stdio' >&2\nexit 2\n",
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).unwrap();
    }
    let probe = sail_compiler::ts_native::probe_lsp_help(&script).expect("probe");
    assert_eq!(probe.exit, Some(2));
    assert!(probe.accepted, "combined={}", probe.combined);
}

#[test]
fn expected_peer_path_is_dir_plus_tsc_not_renamed_file() {
    let exe = PathBuf::from("/opt/sail/sail");
    assert_eq!(
        expected_peer_tsc(&exe),
        PathBuf::from("/opt/sail/sail-ts-native/tsc")
    );
}
