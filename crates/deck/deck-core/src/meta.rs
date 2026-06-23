//! `GET /_deck/meta` — drift-detection endpoint.
//!
//! Returns the compiler identity stamp baked into every compiled
//! artifact alongside the daemon's own build identity. The shell
//! fetches this at startup and compares it against the
//! `_compilerVersion` marker in the bundle it loaded. Any mismatch
//! is surfaced as a banner / console warning rather than the silent
//! breakage we hit with the `fade` export regression.
//!
//! The daemon supplies its own build info as a `DaemonBuildInfo`
//! because `build_info::build_info!()` is a macro that must expand
//! inside the binary crate (cargo scopes `rustc-env` per crate).
//! Compiler info comes from `sui_compiler` directly — it bakes the
//! same env vars via its own `build.rs`.

use serde::Serialize;
use serde_json::{json, Value};

/// Daemon-side build identity. Mirrors the relevant subset of
/// `build_info::BuildInfo` — `&'static str` fields stay static across
/// the daemon lifetime.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct DaemonBuildInfo {
    pub cargo: &'static str,
    pub git: &'static str,
    pub dirty: bool,
    pub built_at: &'static str,
}

impl DaemonBuildInfo {
    pub fn to_json(self) -> Value {
        json!({
            "cargo": self.cargo,
            "git": self.git,
            "dirty": self.dirty,
            "builtAt": self.built_at,
        })
    }
}

/// `GET /_deck/meta` logic — returns the meta JSON payload. The daemon's
/// own build info is passed in (None falls back to `"unknown"` markers);
/// the adapter binds it from its request-extension carrier.
pub fn serve_meta(daemon: Option<DaemonBuildInfo>) -> Value {
    let daemon = daemon.unwrap_or(DaemonBuildInfo {
        cargo: "unknown",
        git: "unknown",
        dirty: false,
        built_at: "unknown",
    });
    let compiler = sui_compiler::compiler_version();
    json!({
        "compilerVersion": compiler.to_json(),
        "daemonVersion": daemon.to_json(),
        "builtAt": daemon.built_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_payload_carries_compiler_and_daemon_versions() {
        let dbi = DaemonBuildInfo {
            cargo: "0.2.0",
            git: "deadbeef",
            dirty: false,
            built_at: "2026-05-14T00:00:00Z",
        };
        let payload = serve_meta(Some(dbi));

        let compiler = payload
            .get("compilerVersion")
            .expect("compilerVersion field present");
        assert!(compiler.get("cargo").and_then(Value::as_str).is_some());
        assert!(compiler.get("git").and_then(Value::as_str).is_some());
        assert!(compiler.get("builtAt").and_then(Value::as_str).is_some());

        let daemon = payload
            .get("daemonVersion")
            .expect("daemonVersion field present");
        assert_eq!(daemon.get("cargo").and_then(Value::as_str), Some("0.2.0"));
        assert_eq!(daemon.get("git").and_then(Value::as_str), Some("deadbeef"));
        assert_eq!(daemon.get("dirty").and_then(Value::as_bool), Some(false));
        assert_eq!(
            daemon.get("builtAt").and_then(Value::as_str),
            Some("2026-05-14T00:00:00Z"),
        );

        assert_eq!(
            payload.get("builtAt").and_then(Value::as_str),
            Some("2026-05-14T00:00:00Z"),
            "top-level builtAt mirrors daemon timestamp for at-a-glance freshness",
        );
    }

    #[test]
    fn meta_payload_handles_missing_extension() {
        let payload = serve_meta(None);
        let daemon = payload.get("daemonVersion").expect("daemonVersion present");
        assert_eq!(daemon.get("git").and_then(Value::as_str), Some("unknown"));
        assert_eq!(daemon.get("cargo").and_then(Value::as_str), Some("unknown"));
    }
}
