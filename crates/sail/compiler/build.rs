//! Build script — emits `BUILD_*` env vars and extracts the TypeScript 7
//! native sidecar next to the compiler binary.
//!
//! Token-model bake lives in `spinnaker-tokens` (J1). Do not re-bake here.

use std::env;
use std::path::{Path, PathBuf};

#[path = "src/ts_native/fetch.rs"]
mod fetch;
#[path = "src/ts_native/pins.rs"]
mod pins;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ts_native/pins.toml");
    println!("cargo:rerun-if-env-changed=SAIL_TS_NATIVE_TARBALL");
    println!("cargo:rerun-if-env-changed=SAIL_TS_NATIVE_OFFLINE");
    println!("cargo:rerun-if-env-changed=SAIL_TS_NATIVE_REQUIRED");
    build_info::emit_build_env();
    extract_ts_native_sidecar();
}

fn extract_ts_native_sidecar() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if arch == "wasm32" {
        return;
    }
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let (npm_os, npm_arch) = fetch::cargo_cfg_to_npm(&os, &arch);
    if pins::pin_for(&npm_os, &npm_arch).is_none() {
        return;
    }
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let dest = sidecar_dest(&profile);
    let offline = env::var_os("SAIL_TS_NATIVE_OFFLINE").is_some();
    let tarball_override = env::var_os("SAIL_TS_NATIVE_TARBALL").map(PathBuf::from);
    let required = env::var("SAIL_TS_NATIVE_REQUIRED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
        || profile == "release";
    let opts = fetch::EnsureOpts {
        dest,
        offline,
        tarball_override,
    };
    match fetch::ensure_sidecar(&npm_os, &npm_arch, &opts) {
        Ok(_) => {}
        Err(e) if required => {
            panic!(
                "sail-ts-native required (release / SAIL_TS_NATIVE_REQUIRED): {e}\n\
                 Place a verified tarball at $SAIL_TS_NATIVE_TARBALL or \
                 $CARGO_HOME/sail-ts-native/{}/, or run online once.",
                pins::TS_NATIVE_VERSION
            );
        }
        Err(e) => {
            println!("cargo:warning=sail-ts-native extract skipped: {e}");
        }
    }
}

fn sidecar_dest(profile: &str) -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        if !target.is_empty() {
            return PathBuf::from(target).join(profile).join("sail-ts-native");
        }
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let profile_dir = fetch::profile_dir_from_out_dir(&out_dir, profile).unwrap_or_else(|| {
        out_dir
            .ancestors()
            .nth(3)
            .map(Path::to_path_buf)
            .unwrap_or(out_dir)
    });
    profile_dir.join("sail-ts-native")
}
