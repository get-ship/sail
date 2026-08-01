//! Shared build-time identity for Ship binaries.
//!
//! ## Build side
//! From a binary crate's `build.rs`, call [`emit_build_env`]. It runs git
//! once, captures the rustc version, and emits five canonical
//! `cargo:rustc-env` lines: `BUILD_GIT_SHA`, `BUILD_GIT_DIRTY`,
//! `BUILD_TIMESTAMP`, `BUILD_RUSTC_VERSION`, `BUILD_PROFILE`.
//!
//! ## Runtime side
//! Use [`build_info!`] from the binary crate to construct a typed
//! [`BuildInfo`] at compile time. The macro expands at the call site so
//! `env!()` reads the env vars set by the binary's own `build.rs` (cargo
//! scopes `rustc-env` per crate).
//!
//! ## Why this crate exists
//! Three parallel implementations existed before — `SHIPD_GIT_COMMIT`,
//! `VERGEN_GIT_SHA`, `GIT_COMMIT_SHORT`, `SHIP_MCP_GIT_SHA`, etc. — each
//! with its own naming and feature set. One source of truth keeps the
//! daemon, CLI, and MCP servers consistent so version skew detection
//! works across them.

#[doc(hidden)]
pub mod __private {
    /// Re-exports for the [`crate::build_info`] macro.
    pub use core::option_env;
}

/// Build-time identity captured at compile time.
///
/// Construct via the [`build_info!`] macro from a binary crate that calls
/// [`emit_build_env`] in its `build.rs`. Reads `BUILD_*` env vars at the
/// macro expansion site.
#[derive(Debug, Clone, Copy)]
pub struct BuildInfo {
    /// Short git SHA at build time, or `"unknown"` if git was unavailable.
    pub git_sha: &'static str,
    /// Whether the working tree had uncommitted changes (excluding untracked).
    pub git_dirty: bool,
    /// RFC 3339 UTC timestamp of the build.
    pub timestamp: &'static str,
    /// `rustc --version` output captured at build time.
    pub rustc_version: &'static str,
    /// Cargo profile: `"dev"` for debug builds, `"release"` for release.
    pub profile: &'static str,
}

impl BuildInfo {
    /// Short identity string: `<sha>` or `<sha>+dirty`.
    pub fn short(&self) -> String {
        if self.git_dirty {
            format!("{}+dirty", self.git_sha)
        } else {
            self.git_sha.to_string()
        }
    }

    /// Full build identifier suitable for HTTP headers and logs.
    /// Format: `<sha>[+dirty] (<profile>, rustc <ver>) <timestamp>`.
    pub fn full(&self) -> String {
        format!(
            "{} ({}, rustc {}) {}",
            self.short(),
            self.profile,
            self.rustc_version,
            self.timestamp,
        )
    }
}

/// Construct a [`BuildInfo`] at compile time from the calling crate's env.
///
/// Must be invoked from a crate whose `build.rs` calls [`emit_build_env`].
/// Cargo scopes `cargo:rustc-env` per crate, so `env!` reads the local
/// values — not values set by other crates' build scripts.
///
/// # Example
/// ```ignore
/// use build_info::build_info;
///
/// pub fn build_info() -> build_info::BuildInfo {
///     build_info!()
/// }
/// ```
#[macro_export]
macro_rules! build_info {
    () => {{
        $crate::BuildInfo {
            git_sha: $crate::__private::option_env!("BUILD_GIT_SHA").unwrap_or("unknown"),
            git_dirty: matches!(
                $crate::__private::option_env!("BUILD_GIT_DIRTY"),
                Some("1") | Some("true")
            ),
            timestamp: $crate::__private::option_env!("BUILD_TIMESTAMP").unwrap_or("unknown"),
            rustc_version: $crate::__private::option_env!("BUILD_RUSTC_VERSION")
                .unwrap_or("unknown"),
            profile: $crate::__private::option_env!("BUILD_PROFILE").unwrap_or("unknown"),
        }
    }};
}

// ── Build-script side ───────────────────────────────────────────────────────
//
// These functions are intentionally usable from a binary's `build.rs`
// without any build-time deps beyond std + this crate. They exit cleanly
// on git failure so source-tarball builds still produce a binary with
// `git_sha = "unknown"`.

/// Emit the canonical `BUILD_*` env vars from a binary's `build.rs`.
///
/// Idempotent. Intentionally does NOT declare `cargo:rerun-if-changed`, so the
/// build script only re-runs when the calling crate's source changes. Git
/// HEAD/worktree churn therefore cannot force a rebuild of every crate that
/// depends on sail-compiler.
pub fn emit_build_env() {
    use std::process::Command;

    let sha = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let dirty = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    // Deterministic timestamp. `SystemTime::now()` changed on every build-script
    // run, so `cargo:rustc-env=BUILD_TIMESTAMP` differed between runs of the same
    // cargo invocation. That changed sail-compiler's `DependencyInfo` fingerprint
    // and forced a mid-invocation rebuild, which invalidated deck-core/deck-axum/
    // deck while rustdoc was compiling the doctest harness (intermittent E0463).
    // We use SOURCE_DATE_EPOCH (reproducible-builds standard) if present, else the
    // committer date of HEAD, so the same checkout always emits the same value.
    let timestamp = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(format_rfc3339_utc)
        .or_else(|| {
            Command::new("git")
                .args(["log", "-1", "--format=%ct", "HEAD"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(format_rfc3339_utc)
        })
        .unwrap_or_else(|| "unknown".to_string());

    let rustc = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=BUILD_GIT_SHA={sha}");
    println!(
        "cargo:rustc-env=BUILD_GIT_DIRTY={}",
        if dirty { "1" } else { "0" }
    );
    println!("cargo:rustc-env=BUILD_TIMESTAMP={timestamp}");
    println!("cargo:rustc-env=BUILD_RUSTC_VERSION={rustc}");
    println!("cargo:rustc-env=BUILD_PROFILE={profile}");
    // NOTE: we intentionally do NOT emit `cargo:rerun-if-changed` here.
    // The old hardcoded `../../.git/HEAD` resolved incorrectly for callers
    // outside crates/build-info and produced missing-path stale fingerprints;
    // fixing it to the real gitdir still left worktrees vulnerable to HEAD
    // mtime churn. Re-running on every build would force a full rebuild of
    // every crate that depends on sail-compiler (including deck-core/deck-axum/
    // deck), which races with rustdoc and produces intermittent E0463. Build
    // info (sha/dirty/timestamp) is advisory telemetry; stale values are
    // acceptable until the next source-driven rebuild. See claim
    // c-2026-07-19T18-12-45-685Z-f499ce4b.
}

/// Format `secs` (UNIX epoch) as `YYYY-MM-DDTHH:MM:SSZ`. Pure stdlib —
/// no chrono dep in build.rs. Algorithm is the standard civil-from-days
/// approach (Howard Hinnant); accurate for all years cargo will encounter.
fn format_rfc3339_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let time_of_day = secs % 86_400;
    let hour = (time_of_day / 3_600) as u32;
    let minute = ((time_of_day % 3_600) / 60) as u32;
    let second = (time_of_day % 60) as u32;

    // Civil-from-days. Reference: http://howardhinnant.github.io/date_algorithms.html#civil_from_days
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_format_known_dates() {
        // 2024-01-01T00:00:00Z
        assert_eq!(format_rfc3339_utc(1_704_067_200), "2024-01-01T00:00:00Z");
        // 2025-01-01T00:00:00Z (post-epoch leap-year boundary)
        assert_eq!(format_rfc3339_utc(1_735_689_600), "2025-01-01T00:00:00Z");
        // Mid-day, mid-month sanity
        assert_eq!(format_rfc3339_utc(1_751_385_600), "2025-07-01T16:00:00Z");
        // Epoch
        assert_eq!(format_rfc3339_utc(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn build_info_short_clean() {
        let bi = BuildInfo {
            git_sha: "abc12345",
            git_dirty: false,
            timestamp: "2026-04-26T14:32:11Z",
            rustc_version: "rustc 1.85.0",
            profile: "dev",
        };
        assert_eq!(bi.short(), "abc12345");
    }

    #[test]
    fn build_info_short_dirty() {
        let bi = BuildInfo {
            git_sha: "abc12345",
            git_dirty: true,
            timestamp: "2026-04-26T14:32:11Z",
            rustc_version: "rustc 1.85.0",
            profile: "dev",
        };
        assert_eq!(bi.short(), "abc12345+dirty");
    }
}
