//! Native TypeScript 7 sidecar resolver.
//!
//! Product discover: `SAIL_TS_NATIVE` disable → `SAIL_TSC`/`SAIL_TSGO` →
//! peer dir (`deps/` hop) → XDG → PATH last-resort ELF. Never walks
//! ancestor `node_modules`. Returned `program` is a Go ELF next to
//! `lib.d.ts`; `--lsp --help` + `--version` must accept pin `7.0.2`.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

mod pins;
pub use pins::{PINS_TOML, Pin, TS_NATIVE_VERSION, parse_pins, pin_for, pins};
pub mod fetch;
/// npm optionalDependency package prefix.
pub const TS_NATIVE_NPM_PREFIX: &str = "@typescript/typescript";
/// argv for the language server. Same flags tsgo nightly already speaks.
pub const TS_NATIVE_LSP_ARGS: &[&str] = &["--lsp", "-stdio"];

/// Env names. `SAIL_TSGO` is a documented alias of `SAIL_TSC`.
pub const ENV_TSC: &str = "SAIL_TSC";
pub const ENV_TSGO: &str = "SAIL_TSGO";
/// Explicit disable. `0` / `false` / empty → Missing (no fallthrough).
pub const ENV_DISABLE: &str = "SAIL_TS_NATIVE";

pub const SIDECAR_DIR_NAME: &str = "sail-ts-native";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsNativeTool {
    pub program: PathBuf,
    /// How we found it — for doctor, initialize payload, tests.
    pub source: TsNativeSource,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsNativeSource {
    Env,
    Peer,
    Xdg,
    Path,
}

impl TsNativeSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Peer => "peer",
            Self::Xdg => "xdg",
            Self::Path => "path",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsNativeStatus {
    Live,
    Missing,
}

/// npm `{os}` / `{arch}` used in `@typescript/typescript-{os}-{arch}`.
pub fn npm_os_arch() -> (&'static str, &'static str) {
    let os = match std::env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    (os, arch)
}

pub fn tsc_file_name() -> &'static str {
    if cfg!(windows) { "tsc.exe" } else { "tsc" }
}

pub fn tsgo_file_name() -> &'static str {
    if cfg!(windows) { "tsgo.exe" } else { "tsgo" }
}

/// `dirname(exe)/sail-ts-native` plus one `deps/` / `examples/` hop.
/// `$CARGO_TARGET_DIR/{profile}` is included only when that env is set.
pub fn peer_sidecar_dirs(exe: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(exe_dir) = exe.parent() {
        dirs.push(exe_dir.join(SIDECAR_DIR_NAME));
        if let Some(name) = exe_dir.file_name().and_then(|n| n.to_str()) {
            if name == "deps" || name == "examples" {
                if let Some(profile) = exe_dir.parent() {
                    dirs.push(profile.join(SIDECAR_DIR_NAME));
                }
            }
        }
    }
    if let Ok(target) = std::env::var("CARGO_TARGET_DIR") {
        if !target.is_empty() {
            let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
            dirs.push(PathBuf::from(target).join(profile).join(SIDECAR_DIR_NAME));
        }
    }
    dirs
}

/// Absolute expected peer `tsc` next to `exe` (or one `deps/` hop).
pub fn expected_peer_tsc(exe: &Path) -> PathBuf {
    if let Some(exe_dir) = exe.parent() {
        if let Some(name) = exe_dir.file_name().and_then(|n| n.to_str()) {
            if name == "deps" || name == "examples" {
                if let Some(profile) = exe_dir.parent() {
                    return profile.join(SIDECAR_DIR_NAME).join(tsc_file_name());
                }
            }
        }
    }
    match exe.parent() {
        Some(dir) => dir.join(SIDECAR_DIR_NAME).join(tsc_file_name()),
        None => PathBuf::from(SIDECAR_DIR_NAME).join(tsc_file_name()),
    }
}

/// Peer `tsc` derived from a test harness exe (`…/deps/<harness>-*`).
pub fn expected_peer_tsc_from_current_exe() -> PathBuf {
    match std::env::current_exe() {
        Ok(exe) => expected_peer_tsc(&exe),
        Err(_) => PathBuf::from(SIDECAR_DIR_NAME).join(tsc_file_name()),
    }
}

pub fn sidecar_tsc(dir: &Path) -> PathBuf {
    dir.join(tsc_file_name())
}

pub fn sidecar_lib_dts(dir: &Path) -> PathBuf {
    dir.join("lib.d.ts")
}

pub fn sidecar_dir_is_complete(dir: &Path) -> bool {
    sidecar_tsc(dir).is_file() && sidecar_lib_dts(dir).is_file()
}

/// Path-math / future-order helper: peer next to `exe` only.
/// Never walks parents looking for `node_modules`.
pub fn resolve_ts_native_from_exe(exe: &Path) -> Option<TsNativeTool> {
    for dir in peer_sidecar_dirs(exe) {
        let program = sidecar_tsc(&dir);
        if program.is_file() {
            return Some(TsNativeTool {
                program,
                source: TsNativeSource::Peer,
                version: TS_NATIVE_VERSION.to_string(),
            });
        }
    }
    None
}

pub fn xdg_data_home() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg));
        }
    }
    if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    } else {
        std::env::var_os("HOME")
            .filter(|v| !v.is_empty())
            .map(|h| PathBuf::from(h).join(".local/share"))
    }
}

pub fn xdg_sidecar_dir() -> Option<PathBuf> {
    let (os, arch) = npm_os_arch();
    Some(
        xdg_data_home()?
            .join("sail")
            .join("ts-native")
            .join(TS_NATIVE_VERSION)
            .join(format!("{os}-{arch}")),
    )
}

pub fn disable_value(v: &str) -> bool {
    v.is_empty() || v == "0" || v.eq_ignore_ascii_case("false")
}

pub fn disable_is_set() -> bool {
    matches!(std::env::var(ENV_DISABLE), Ok(v) if disable_value(&v))
}

/// Absolute existing file. Empty / relative / missing → not usable (Missing).
pub fn env_override_is_usable(raw: &str) -> bool {
    if raw.is_empty() {
        return false;
    }
    let path = Path::new(raw);
    path.is_absolute() && path.is_file()
}

/// `Some(path)` = env is set (may be empty). `None` = unset.
pub fn env_override() -> Option<String> {
    match std::env::var(ENV_TSC) {
        Ok(v) => Some(v),
        Err(std::env::VarError::NotPresent) => match std::env::var(ENV_TSGO) {
            Ok(v) => Some(v),
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => Some(String::new()),
        },
        Err(_) => Some(String::new()),
    }
}

pub fn is_shebang(path: &Path) -> bool {
    let Ok(mut f) = File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 2];
    matches!(f.read_exact(&mut buf), Ok(())) && buf == *b"#!"
}

/// Follow a Node wrapper (explicit path / PATH hit) to the sibling platform ELF.
/// Does not walk ancestor `node_modules`.
pub fn sibling_platform_elf(wrapper: &Path) -> Option<PathBuf> {
    let (os, arch) = npm_os_arch();
    let nm = wrapper.parent()?.parent()?;
    let tsc_pkg = format!("{TS_NATIVE_NPM_PREFIX}-{os}-{arch}");
    let preview_pkg = format!("@typescript/native-preview-{os}-{arch}");
    let candidates = [
        nm.join(&tsc_pkg).join("lib").join(tsc_file_name()),
        nm.join(&preview_pkg).join("lib").join(tsgo_file_name()),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn accept_sidecar_elf(program: &Path) -> bool {
    program.is_file() && sidecar_lib_dts(&program.parent().unwrap_or(program)).is_file()
}

fn tool(program: PathBuf, source: TsNativeSource) -> TsNativeTool {
    TsNativeTool {
        program,
        source,
        version: TS_NATIVE_VERSION.to_string(),
    }
}

/// Probe `--lsp --help` then `--version`. Never return a shebang.
fn finalize_tool(mut tool: TsNativeTool) -> Option<TsNativeTool> {
    if !tool.program.is_file() {
        return None;
    }
    if is_shebang(&tool.program) {
        tool.program = sibling_platform_elf(&tool.program)?;
    }
    if is_shebang(&tool.program) || !accept_sidecar_elf(&tool.program) {
        return None;
    }
    let probe = probe_lsp_help(&tool.program).ok()?;
    if !probe.accepted {
        return None;
    }
    let out = Command::new(&tool.program).arg("--version").output().ok()?;
    let v = parse_version_stdout(&String::from_utf8_lossy(&out.stdout))?;
    if !accept_probed_version(&v) {
        return None;
    }
    tool.version = v;
    Some(tool)
}

fn resolve_env_override() -> Option<Option<TsNativeTool>> {
    let raw = env_override()?;
    if !env_override_is_usable(&raw) {
        return Some(None);
    }
    let path = PathBuf::from(&raw);
    let program = if is_shebang(&path) {
        sibling_platform_elf(&path)
    } else {
        Some(path)
    };
    match program {
        Some(program) => Some(finalize_tool(tool(program, TsNativeSource::Env))),
        None => Some(None),
    }
}

fn resolve_xdg() -> Option<TsNativeTool> {
    let dir = xdg_sidecar_dir()?;
    finalize_tool(tool(sidecar_tsc(&dir), TsNativeSource::Xdg))
}

pub fn which_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let mut candidate = dir.join(name);
        if cfg!(windows) && candidate.extension().is_none() {
            candidate.set_extension("exe");
        }
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn resolve_path_last_resort() -> Option<TsNativeTool> {
    for name in ["tsc", "tsgo"] {
        let Some(hit) = which_on_path(name) else {
            continue;
        };
        let program = if is_shebang(&hit) {
            sibling_platform_elf(&hit)
        } else {
            Some(hit)
        };
        if let Some(program) = program {
            if let Some(tool) = finalize_tool(tool(program, TsNativeSource::Path)) {
                return Some(tool);
            }
        }
    }
    None
}

/// Env → peer (plus `deps/` hop) → XDG → PATH. Never walks `node_modules`.
pub fn resolve_ts_native() -> Option<TsNativeTool> {
    resolve_ts_native_at(std::env::current_exe().ok().as_deref())
}

pub fn resolve_ts_native_at(exe: Option<&Path>) -> Option<TsNativeTool> {
    if disable_is_set() {
        return None;
    }
    if let Some(env_result) = resolve_env_override() {
        return env_result;
    }
    if let Some(exe) = exe {
        if let Some(tool) = resolve_ts_native_from_exe(exe) {
            if let Some(tool) = finalize_tool(tool) {
                return Some(tool);
            }
        }
    }
    if let Some(tool) = resolve_xdg() {
        return Some(tool);
    }
    resolve_path_last_resort()
}

/// Panic text when a typed test cannot see the profile-dir peer.
pub fn missing_sidecar_hint() -> String {
    format!(
        "TypeScript sidecar missing at {}",
        expected_peer_tsc_from_current_exe().display()
    )
}

pub fn ts_native_status() -> TsNativeStatus {
    if resolve_ts_native().is_some() {
        TsNativeStatus::Live
    } else {
        TsNativeStatus::Missing
    }
}

/// `--lsp --help` accept rule. Combined stdout+stderr; exit ∈ {0,1,2}.
pub fn accept_lsp_help(exit: i32, stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}{stderr}");
    if combined.contains("Unknown compiler option") {
        return false;
    }
    matches!(exit, 0..=2) && combined.contains("stdio")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspHelpProbe {
    pub accepted: bool,
    pub exit: Option<i32>,
    pub combined: String,
}

/// Run `{bin} --lsp --help`. Caller must already have a sibling `lib.d.ts`
/// before using this on a real 7.0.2 ELF (it panics without one).
pub fn probe_lsp_help(bin: &Path) -> std::io::Result<LspHelpProbe> {
    let out = Command::new(bin).args(["--lsp", "--help"]).output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let exit = out.status.code().unwrap_or(1);
    Ok(LspHelpProbe {
        accepted: accept_lsp_help(exit, &stdout, &stderr),
        exit: out.status.code(),
        combined: format!("{stdout}{stderr}"),
    })
}

pub fn parse_version_stdout(stdout: &str) -> Option<String> {
    let line = stdout.lines().next()?.trim();
    let ver = line.strip_prefix("Version ").unwrap_or(line).trim();
    if ver.is_empty() {
        None
    } else {
        Some(ver.to_string())
    }
}

/// Product pin is exact `7.0.2`.
pub fn accept_probed_version(version: &str) -> bool {
    version == TS_NATIVE_VERSION
}
