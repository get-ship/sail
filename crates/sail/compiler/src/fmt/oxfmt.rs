//! Discover and invoke pinned oxfmt for `<scope>` / `<controller>` TS bodies.
//!
//! Discovery order (SPEC-TOOLING-1 §WI1):
//! 1. `node_modules/.bin/oxfmt` walking up from cwd / start path
//! 2. `PATH` (`which oxfmt`)
//! 3. Missing → caller logs a warning and keeps the region as-is
//!
//! Options are scaffold-pinned: nested config is disabled so project
//! prettier/biome knobs cannot fork the sail format surface.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Result of locating the oxfmt binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OxfmtStatus {
    Found(PathBuf),
    Missing,
}

/// Locate oxfmt: `node_modules/.bin` ancestors → PATH.
pub fn discover_oxfmt(start: Option<&Path>) -> OxfmtStatus {
    if let Some(p) = find_in_node_modules(start) {
        return OxfmtStatus::Found(p);
    }
    if let Some(p) = find_on_path() {
        return OxfmtStatus::Found(p);
    }
    OxfmtStatus::Missing
}

fn find_in_node_modules(start: Option<&Path>) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok();
    let mut cur = start
        .map(|p| {
            if p.is_file() {
                p.parent().unwrap_or(p).to_path_buf()
            } else {
                p.to_path_buf()
            }
        })
        .or(cwd);
    for _ in 0..24 {
        let Some(dir) = cur else { break };
        let candidate = dir.join("node_modules/.bin/oxfmt");
        if candidate.is_file() {
            return Some(candidate);
        }
        // Bun / pnpm may nest under .bun — still expose .bin at root.
        cur = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

fn find_on_path() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join("oxfmt");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Format a TypeScript/scope body via oxfmt stdin. Returns `None` on failure
/// (caller keeps original). Uses `--stdin-filepath` so the TS parser is selected
/// and `--disable-nested-config` so options stay scaffold-pinned.
pub fn format_ts_region(oxfmt: &Path, body: &str, filename_hint: &str) -> Option<String> {
    let mut child = Command::new(oxfmt)
        .arg(format!("--stdin-filepath={filename_hint}"))
        .arg("--disable-nested-config")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    use std::io::Write;
    {
        let mut stdin = child.stdin.take()?;
        // Ensure trailing newline — oxfmt is happier with complete lines.
        if body.ends_with('\n') {
            stdin.write_all(body.as_bytes()).ok()?;
        } else {
            stdin.write_all(body.as_bytes()).ok()?;
            stdin.write_all(b"\n").ok()?;
        }
    }
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    Some(text)
}
