//! Tracked local `sail dev` process control.
//!
//! Local bind only; project path confined under workspace; only the child
//! started by this session is stopped.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::ctx::Ctx;
use crate::error::OpError;

const DEFAULT_ADDR: &str = "127.0.0.1:5173";

#[derive(Debug)]
struct TrackedServer {
    child: Child,
    url: String,
    addr: String,
    project: PathBuf,
}

static TRACKED: Mutex<Option<TrackedServer>> = Mutex::new(None);

/// Start (or ensure) a local sail/deck dev server for `project`.
///
/// Args: `{ project, addr? }` → `{ ok, url, status, pid? }`
pub fn start(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project_raw = args
        .get("project")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("dev_server.start requires `project`".into()))?;

    let project = resolve_project(project_raw, ctx)?;
    let addr = args
        .get("addr")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_ADDR)
        .to_string();
    validate_local_addr(&addr)?;

    let url = format!("http://{addr}");

    // Already tracking a live process for this project+addr → reuse.
    {
        let mut guard = TRACKED
            .lock()
            .map_err(|_| OpError::Internal("dev_server lock poisoned".into()))?;
        if let Some(tracked) = guard.as_mut() {
            if tracked.project == project && tracked.addr == addr {
                if is_running(&mut tracked.child) {
                    return Ok(json!({
                        "ok": true,
                        "url": tracked.url,
                        "status": "running",
                        "pid": tracked.child.id(),
                        "project": project.display().to_string(),
                    }));
                }
                // Dead — drop and respawn.
                let _ = tracked.child.kill();
                let _ = tracked.child.wait();
                *guard = None;
            } else if is_running(&mut tracked.child) {
                return Err(OpError::FailedPrecondition(format!(
                    "MCP already tracks a dev server for {} at {} — stop it first",
                    tracked.project.display(),
                    tracked.addr
                )));
            } else {
                let _ = tracked.child.wait();
                *guard = None;
            }
        }
    }

    let bin = resolve_sail_or_deck_bin()?;
    // Both `sail` and `deck` expose `dev --project --addr`.
    let mut cmd = Command::new(&bin);
    cmd.arg("dev")
        .arg("--project")
        .arg(&project)
        .arg("--addr")
        .arg(&addr)
        .current_dir(&ctx.workspace_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = cmd.spawn().map_err(|e| {
        OpError::Internal(format!(
            "failed to spawn {} dev --project {}: {e}",
            bin.display(),
            project.display()
        ))
    })?;
    let pid = child.id();

    {
        let mut guard = TRACKED
            .lock()
            .map_err(|_| OpError::Internal("dev_server lock poisoned".into()))?;
        *guard = Some(TrackedServer {
            child,
            url: url.clone(),
            addr: addr.clone(),
            project: project.clone(),
        });
    }

    // Best-effort wait until something answers on the port (or timeout).
    let ready = wait_for_http(&url, Duration::from_secs(8));
    Ok(json!({
        "ok": true,
        "url": url,
        "status": if ready { "running" } else { "starting" },
        "pid": pid,
        "project": project.display().to_string(),
        "addr": addr,
    }))
}

/// Status of the MCP-tracked server only.
pub fn status(_args: &Value, _ctx: &Ctx) -> Result<Value, OpError> {
    let mut guard = TRACKED
        .lock()
        .map_err(|_| OpError::Internal("dev_server lock poisoned".into()))?;
    let Some(tracked) = guard.as_mut() else {
        return Ok(json!({
            "ok": true,
            "running": false,
        }));
    };
    if is_running(&mut tracked.child) {
        return Ok(json!({
            "ok": true,
            "running": true,
            "url": tracked.url,
            "pid": tracked.child.id(),
            "project": tracked.project.display().to_string(),
            "addr": tracked.addr,
        }));
    }
    let _ = tracked.child.wait();
    *guard = None;
    Ok(json!({
        "ok": true,
        "running": false,
    }))
}

/// Stop **only** the MCP-started child.
pub fn stop(_args: &Value, _ctx: &Ctx) -> Result<Value, OpError> {
    let mut guard = TRACKED
        .lock()
        .map_err(|_| OpError::Internal("dev_server lock poisoned".into()))?;
    match guard.take() {
        Some(mut tracked) => {
            let pid = tracked.child.id();
            let _ = tracked.child.kill();
            let _ = tracked.child.wait();
            Ok(json!({
                "ok": true,
                "stopped": true,
                "pid": pid,
            }))
        }
        None => Ok(json!({
            "ok": true,
            "stopped": false,
            "running": false,
        })),
    }
}

fn resolve_project(raw: &str, ctx: &Ctx) -> Result<PathBuf, OpError> {
    if raw.contains("..") {
        return Err(OpError::PermissionDenied(
            "project path must not contain ..".into(),
        ));
    }
    let pb = PathBuf::from(raw);
    let abs = if pb.is_absolute() {
        pb
    } else {
        ctx.workspace_root.join(pb)
    };
    let canon_root = ctx.workspace_root.canonicalize().map_err(|e| {
        OpError::Internal(format!(
            "workspace root {}: {e}",
            ctx.workspace_root.display()
        ))
    })?;
    // Parent must exist for confinement; project may be missing → InvalidArgument.
    if !abs.is_dir() {
        return Err(OpError::InvalidArgument(format!(
            "project dir does not exist: {}",
            abs.display()
        )));
    }
    let canon = abs.canonicalize().map_err(|e| {
        OpError::InvalidArgument(format!("project path invalid {}: {e}", abs.display()))
    })?;
    if !canon.starts_with(&canon_root) {
        return Err(OpError::PermissionDenied(
            "project path escapes workspace".into(),
        ));
    }
    Ok(canon)
}

fn validate_local_addr(addr: &str) -> Result<(), OpError> {
    // Local IPv4 / hostname only. IPv6 is intentionally unsupported: host/port
    // splitting is colon-based (`host:port`), which is ambiguous for bare `::1`.
    let Some((host, port)) = addr.rsplit_once(':') else {
        return Err(OpError::InvalidArgument(
            "addr must be host:port (e.g. 127.0.0.1:5173); IPv6 is not supported".into(),
        ));
    };
    if port.is_empty() || !port.chars().all(|c| c.is_ascii_digit()) {
        return Err(OpError::InvalidArgument(format!(
            "addr port must be numeric (got {port:?})"
        )));
    }
    let ok = host == "127.0.0.1" || host == "localhost";
    if !ok {
        return Err(OpError::PermissionDenied(format!(
            "dev_server may only bind 127.0.0.1 or localhost (got host {host:?}; IPv6 not supported)"
        )));
    }
    Ok(())
}

fn is_running(child: &mut Child) -> bool {
    match child.try_wait() {
        Ok(None) => true,
        Ok(Some(_)) => false,
        Err(_) => false,
    }
}

fn wait_for_http(url: &str, timeout: Duration) -> bool {
    use std::net::ToSocketAddrs;
    let Some(hostport) = url.strip_prefix("http://") else {
        return false;
    };
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(mut addrs) = hostport.to_socket_addrs() {
            if let Some(a) = addrs.next() {
                if std::net::TcpStream::connect_timeout(&a, Duration::from_millis(200)).is_ok() {
                    return true;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    false
}

fn resolve_sail_or_deck_bin() -> Result<PathBuf, OpError> {
    // Prefer current_exe sibling / PATH: `sail` then `deck`.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["sail", "deck"] {
                let cand = dir.join(name);
                if cand.is_file() {
                    return Ok(cand);
                }
            }
        }
    }
    // Fall back to PATH lookup.
    for name in ["sail", "deck"] {
        if let Ok(path) = which(name) {
            return Ok(path);
        }
    }
    Err(OpError::FailedPrecondition(
        "neither `sail` nor `deck` binary found on PATH / next to current_exe".into(),
    ))
}

fn which(name: &str) -> Result<PathBuf, ()> {
    let path = std::env::var_os("PATH").ok_or(())?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(name);
        if cand.is_file() {
            return Ok(cand);
        }
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn rejects_non_local_addr() {
        assert!(validate_local_addr("0.0.0.0:5173").is_err());
        assert!(validate_local_addr("192.168.1.1:80").is_err());
        assert!(validate_local_addr("127.0.0.1:5173").is_ok());
        assert!(validate_local_addr("localhost:3000").is_ok());
        // IPv6 not supported (colon-split host:port).
        assert!(validate_local_addr("[::1]:5173").is_err());
        assert!(validate_local_addr("::1:5173").is_err());
        assert!(validate_local_addr("127.0.0.1").is_err());
    }

    #[test]
    fn project_must_be_under_workspace() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        std::fs::create_dir_all(ws.join("apps/demo")).unwrap();
        let ctx = Ctx::new(ws, ws.join("apps/demo"), ws);
        let ok = resolve_project("apps/demo", &ctx).unwrap();
        assert!(ok.ends_with("apps/demo"));
        let outside = tempdir().unwrap();
        assert!(matches!(
            resolve_project(outside.path().to_str().unwrap(), &ctx),
            Err(OpError::PermissionDenied(_))
        ));
    }

    #[test]
    fn status_when_idle() {
        // Ensure clean slate for this process (stop any prior).
        let _ = stop(&json!({}), &Ctx::from_env());
        let s = status(&json!({}), &Ctx::from_env()).unwrap();
        assert_eq!(s["running"], false);
    }
}
