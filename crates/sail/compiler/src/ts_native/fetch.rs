//! Fetch / extract the pinned TypeScript 7 `package/lib/**` sidecar.
//!
//! Shared by `sail-compiler` `build.rs` and `sail ts-native setup`.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use base64::Engine;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256, Sha512};
use tar::Archive;

use super::pins::{Pin, TS_NATIVE_VERSION, pin_for};

pub const ENV_TARBALL: &str = "SAIL_TS_NATIVE_TARBALL";
#[allow(dead_code)] // product CLI; build.rs only needs extract
pub const ENV_OFFLINE: &str = "SAIL_TS_NATIVE_OFFLINE";
#[allow(dead_code)]
pub const ENV_REQUIRED: &str = "SAIL_TS_NATIVE_REQUIRED";

pub struct EnsureOpts {
    pub dest: PathBuf,
    pub offline: bool,
    pub tarball_override: Option<PathBuf>,
}

#[derive(Debug)]
pub struct EnsureError(pub String);

impl std::fmt::Display for EnsureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EnsureError {}

pub fn cargo_cfg_to_npm(os: &str, arch: &str) -> (String, String) {
    let os = match os {
        "macos" => "darwin",
        "windows" => "win32",
        other => other,
    };
    let arch = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    (os.to_string(), arch.to_string())
}

pub fn profile_dir_from_out_dir(out_dir: &Path, profile: &str) -> Option<PathBuf> {
    let mut cur = Some(out_dir);
    while let Some(dir) = cur {
        if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
            if name == "debug" || name == "release" || name == profile {
                return Some(dir.to_path_buf());
            }
        }
        cur = dir.parent();
    }
    None
}

pub fn sidecar_complete(dir: &Path) -> bool {
    let tsc = if cfg!(windows) {
        dir.join("tsc.exe")
    } else {
        dir.join("tsc")
    };
    tsc.is_file() && dir.join("lib.d.ts").is_file()
}

pub fn cargo_home() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|v| !v.is_empty())
                .map(|h| PathBuf::from(h).join(".cargo"))
        })
}

pub fn cached_tarball_path(pin: &Pin) -> Option<PathBuf> {
    Some(cargo_home()?.join(pin.cargo_home_rel()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex_lower(&Sha256::digest(bytes))
}

pub fn ssri_sha512(bytes: &[u8]) -> String {
    format!(
        "sha512-{}",
        base64::engine::general_purpose::STANDARD.encode(Sha512::digest(bytes))
    )
}

pub fn verify_tarball(bytes: &[u8], pin: &Pin) -> Result<(), EnsureError> {
    let sha = sha256_hex(bytes);
    if sha != pin.sha256 {
        return Err(EnsureError(format!(
            "sha256 mismatch for {}-{}: got {sha}, want {}",
            pin.os, pin.arch, pin.sha256
        )));
    }
    let ssri = ssri_sha512(bytes);
    if ssri != pin.ssri {
        return Err(EnsureError(format!(
            "ssri mismatch for {}-{}: got {ssri}, want {}",
            pin.os, pin.arch, pin.ssri
        )));
    }
    Ok(())
}

fn read_file(path: &Path) -> Result<Vec<u8>, EnsureError> {
    fs::read(path).map_err(|e| EnsureError(format!("read {}: {e}", path.display())))
}

fn download(url: &str, dest: &Path) -> Result<(), EnsureError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| EnsureError(format!("mkdir {}: {e}", parent.display())))?;
    }
    let tmp = dest.with_extension("tgz.part");
    let status = Command::new("curl")
        .args([
            "-fsS",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--max-redirs",
            "0",
            "-o",
        ])
        .arg(&tmp)
        .arg(url)
        .status()
        .map_err(|e| EnsureError(format!("spawn curl: {e}")))?;
    if !status.success() {
        let _ = fs::remove_file(&tmp);
        return Err(EnsureError(format!("curl {url} exited {status}")));
    }
    fs::rename(&tmp, dest).map_err(|e| EnsureError(format!("rename tarball: {e}")))
}

fn locate_tarball(pin: &Pin, opts: &EnsureOpts) -> Result<PathBuf, EnsureError> {
    if let Some(over) = &opts.tarball_override {
        if !over.is_file() {
            return Err(EnsureError(format!(
                "{ENV_TARBALL} is not a file: {}",
                over.display()
            )));
        }
        return Ok(over.clone());
    }
    if let Some(cached) = cached_tarball_path(pin) {
        if cached.is_file() {
            return Ok(cached);
        }
    }
    if opts.offline {
        return Err(EnsureError(format!(
            "offline: no tarball at {} or $CARGO_HOME/{}",
            ENV_TARBALL,
            pin.cargo_home_rel()
        )));
    }
    let dest = cached_tarball_path(pin)
        .ok_or_else(|| EnsureError("CARGO_HOME / HOME unset; cannot cache tarball".into()))?;
    eprintln!(
        "sail-ts-native: fetching {} → {}",
        pin.tarball_url(),
        dest.display()
    );
    download(&pin.tarball_url(), &dest)?;
    Ok(dest)
}

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(perms.mode() | 0o111);
            let _ = fs::set_permissions(path, perms);
        }
    }
    let _ = path;
}

fn extract_lib(tarball: &Path, dest: &Path) -> Result<(), EnsureError> {
    let file =
        File::open(tarball).map_err(|e| EnsureError(format!("open {}: {e}", tarball.display())))?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    let parent = dest
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&parent)
        .map_err(|e| EnsureError(format!("mkdir {}: {e}", parent.display())))?;
    let staging = parent.join(format!(
        ".{}-extract-{}",
        dest.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sail-ts-native"),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)
        .map_err(|e| EnsureError(format!("mkdir {}: {e}", staging.display())))?;

    let entries = archive
        .entries()
        .map_err(|e| EnsureError(format!("tar entries: {e}")))?;
    let mut extracted = 0usize;
    for entry in entries {
        let mut entry = entry.map_err(|e| EnsureError(format!("tar entry: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| EnsureError(format!("tar path: {e}")))?
            .into_owned();
        let Ok(rel) = path.strip_prefix("package/lib") else {
            continue;
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        if rel
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
        {
            continue;
        }
        let out = staging.join(rel);
        if let Some(p) = out.parent() {
            fs::create_dir_all(p)
                .map_err(|e| EnsureError(format!("mkdir {}: {e}", p.display())))?;
        }
        let mut buf = Vec::new();
        entry
            .read_to_end(&mut buf)
            .map_err(|e| EnsureError(format!("read tar member: {e}")))?;
        let mut f = File::create(&out)
            .map_err(|e| EnsureError(format!("create {}: {e}", out.display())))?;
        f.write_all(&buf)
            .map_err(|e| EnsureError(format!("write {}: {e}", out.display())))?;
        extracted += 1;
    }
    if extracted == 0 {
        let _ = fs::remove_dir_all(&staging);
        return Err(EnsureError(
            "tarball contained no package/lib/** members".into(),
        ));
    }
    let tsc = if cfg!(windows) {
        staging.join("tsc.exe")
    } else {
        staging.join("tsc")
    };
    if !tsc.is_file() || !staging.join("lib.d.ts").is_file() {
        let _ = fs::remove_dir_all(&staging);
        return Err(EnsureError(
            "extracted lib/ is missing tsc or lib.d.ts".into(),
        ));
    }
    make_executable(&tsc);
    if dest.exists() {
        fs::remove_dir_all(dest)
            .map_err(|e| EnsureError(format!("remove {}: {e}", dest.display())))?;
    }
    fs::rename(&staging, dest).map_err(|e| {
        let _ = fs::remove_dir_all(&staging);
        EnsureError(format!(
            "rename {} → {}: {e}",
            staging.display(),
            dest.display()
        ))
    })?;
    Ok(())
}

/// Extract a verified `package/lib/**` into `opts.dest` when it is not already complete.
pub fn ensure_sidecar(os: &str, arch: &str, opts: &EnsureOpts) -> Result<PathBuf, EnsureError> {
    if sidecar_complete(&opts.dest) {
        return Ok(opts.dest.clone());
    }
    let pin = pin_for(os, arch).ok_or_else(|| {
        EnsureError(format!(
            "no pin for {os}-{arch} (TypeScript {TS_NATIVE_VERSION})"
        ))
    })?;
    let tarball = locate_tarball(pin, opts)?;
    let bytes = read_file(&tarball)?;
    verify_tarball(&bytes, pin)?;
    extract_lib(&tarball, &opts.dest)?;
    if !sidecar_complete(&opts.dest) {
        return Err(EnsureError(format!(
            "extract left incomplete dir {}",
            opts.dest.display()
        )));
    }
    Ok(opts.dest.clone())
}

#[allow(dead_code)] // product CLI; build.rs only needs extract
pub fn copy_sidecar_dir(src: &Path, dest: &Path) -> Result<(), EnsureError> {
    if sidecar_complete(dest) {
        return Ok(());
    }
    if !sidecar_complete(src) {
        return Err(EnsureError(format!(
            "source sidecar incomplete: {}",
            src.display()
        )));
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| EnsureError(format!("mkdir {}: {e}", parent.display())))?;
    }
    let staging = dest.with_file_name(format!(
        ".{}-copy-{}",
        dest.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sail-ts-native"),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&staging);
    copy_dir_all(src, &staging)?;
    if dest.exists() {
        fs::remove_dir_all(dest)
            .map_err(|e| EnsureError(format!("remove {}: {e}", dest.display())))?;
    }
    fs::rename(&staging, dest).map_err(|e| {
        let _ = fs::remove_dir_all(&staging);
        EnsureError(format!("rename copy → {}: {e}", dest.display()))
    })
}

fn copy_dir_all(src: &Path, dest: &Path) -> Result<(), EnsureError> {
    fs::create_dir_all(dest).map_err(|e| EnsureError(format!("mkdir {}: {e}", dest.display())))?;
    for ent in fs::read_dir(src).map_err(|e| EnsureError(format!("read {}: {e}", src.display())))? {
        let ent = ent.map_err(|e| EnsureError(format!("read_dir: {e}")))?;
        let from = ent.path();
        let to = dest.join(ent.file_name());
        let ty = ent
            .file_type()
            .map_err(|e| EnsureError(format!("file_type: {e}")))?;
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else if ty.is_file() {
            fs::copy(&from, &to).map_err(|e| {
                EnsureError(format!("copy {} → {}: {e}", from.display(), to.display()))
            })?;
        }
    }
    Ok(())
}
