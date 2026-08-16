//! `sail ts-native setup|status` — fetch / inspect the TypeScript 7 sidecar.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use sail_compiler::ts_native::fetch::{
    ENV_OFFLINE, ENV_TARBALL, EnsureOpts, copy_sidecar_dir, ensure_sidecar, sidecar_complete,
};
use sail_compiler::ts_native::{
    TS_NATIVE_VERSION, expected_peer_tsc, npm_os_arch, resolve_ts_native, sidecar_dir_is_complete,
    xdg_sidecar_dir,
};

pub fn setup() -> Result<()> {
    let (os, arch) = npm_os_arch();
    let peer = peer_dir_from_current_exe()?;
    let offline = std::env::var_os(ENV_OFFLINE).is_some();
    let tarball_override = std::env::var_os(ENV_TARBALL).map(PathBuf::from);
    eprintln!(
        "sail ts-native setup: extracting TypeScript {TS_NATIVE_VERSION} → {}",
        peer.display()
    );
    let dest = ensure_sidecar(
        os,
        arch,
        &EnsureOpts {
            dest: peer.clone(),
            offline,
            tarball_override,
        },
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    if let Some(xdg) = xdg_sidecar_dir() {
        if let Err(e) = copy_sidecar_dir(&dest, &xdg) {
            eprintln!("sail ts-native setup: XDG copy failed ({e})");
        } else {
            eprintln!("sail ts-native setup: XDG {}", xdg.display());
        }
    }
    let tsc = dest.join(if cfg!(windows) { "tsc.exe" } else { "tsc" });
    if !sidecar_complete(&dest) {
        bail!("extract incomplete at {}", dest.display());
    }
    eprintln!("sail ts-native setup: ok {}", tsc.display());
    Ok(())
}

pub fn status() -> Result<()> {
    match resolve_ts_native() {
        Some(tool) => {
            println!(
                "typescript {} live ({}) {}",
                tool.version,
                tool.source.as_str(),
                tool.program.display()
            );
            Ok(())
        }
        None => {
            let peer = peer_dir_from_current_exe()
                .map(|d| d.join(if cfg!(windows) { "tsc.exe" } else { "tsc" }))
                .unwrap_or_else(|_| PathBuf::from("sail-ts-native/tsc"));
            eprintln!(
                "typescript missing — run `sail ts-native setup` (expected {})",
                peer.display()
            );
            std::process::exit(1);
        }
    }
}

/// Best-effort: copy the peer dir next to this `sail` into XDG, or extract.
/// Never fails the caller — returns `Err` only so they can log.
pub fn ensure_best_effort() -> Result<Option<PathBuf>, String> {
    let (os, arch) = npm_os_arch();
    let peer = peer_dir_from_current_exe().map_err(|e| e.to_string())?;
    if sidecar_dir_is_complete(&peer) {
        if let Some(xdg) = xdg_sidecar_dir() {
            copy_sidecar_dir(&peer, &xdg).map_err(|e| e.to_string())?;
            return Ok(Some(xdg));
        }
        return Ok(Some(peer));
    }
    let offline = std::env::var_os(ENV_OFFLINE).is_some();
    let tarball_override = std::env::var_os(ENV_TARBALL).map(PathBuf::from);
    let dest = ensure_sidecar(
        os,
        arch,
        &EnsureOpts {
            dest: peer,
            offline,
            tarball_override,
        },
    )
    .map_err(|e| e.to_string())?;
    if let Some(xdg) = xdg_sidecar_dir() {
        let _ = copy_sidecar_dir(&dest, &xdg);
        return Ok(Some(xdg));
    }
    Ok(Some(dest))
}

fn peer_dir_from_current_exe() -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    Ok(peer_dir_from_exe(&exe))
}

pub fn peer_dir_from_exe(exe: &Path) -> PathBuf {
    expected_peer_tsc(exe)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| exe.parent().unwrap_or(exe).join("sail-ts-native"))
}
