//! `sail mcp install` — write MCP server entries into detected agent host configs.
//!
//! Host formats (TOOLING-1 WI4):
//! - **Claude Code** — project `.mcp.json` (`mcpServers` map)
//! - **Cursor** — project `.cursor/mcp.json` (`mcpServers` map)
//!
//! Detection is best-effort (home dir markers + project files). Claude Code
//! project format is always written so at least one host is configured without
//! manual JSON editing.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// One host config write result.
#[derive(Debug, Clone)]
pub struct HostWrite {
    pub host: &'static str,
    pub path: PathBuf,
    pub action: &'static str,
}

/// Report from [`install`].
#[derive(Debug, Clone)]
pub struct InstallReport {
    pub project_root: PathBuf,
    pub writes: Vec<HostWrite>,
    pub command: String,
    pub args: Vec<String>,
}

/// Install Sail MCP into detected agent hosts under `project_root`.
///
/// Merges a `sail` entry into each host's config, preserving other servers.
pub fn install(project_root: &Path) -> Result<InstallReport, String> {
    install_with_bin(project_root, None)
}

/// Like [`install`]. `bin` overrides the command (developers only).
/// Default is always `sail mcp` — never `cargo run`, never a worktree
/// `target/debug/sail` unless `--bin` was passed.
pub fn install_with_bin(project_root: &Path, bin: Option<&Path>) -> Result<InstallReport, String> {
    fs::create_dir_all(project_root)
        .map_err(|e| format!("mkdir project root {}: {e}", project_root.display()))?;
    let project_root = project_root
        .canonicalize()
        .map_err(|e| format!("canonicalize project root {}: {e}", project_root.display()))?;

    let launch = server_launch(bin);
    let entry = server_entry(&launch, &project_root);

    let mut writes = Vec::new();

    // Claude Code — always write project `.mcp.json` (primary documented format).
    let claude_path = project_root.join(".mcp.json");
    write_mcp_servers_file(&claude_path, "sail", &entry)?;
    writes.push(HostWrite {
        host: "claude-code",
        path: claude_path,
        action: "wrote",
    });

    // Cursor — when project already uses Cursor, or Cursor is installed for the user.
    if cursor_detected(&project_root) {
        let cursor_dir = project_root.join(".cursor");
        fs::create_dir_all(&cursor_dir)
            .map_err(|e| format!("mkdir {}: {e}", cursor_dir.display()))?;
        let cursor_path = cursor_dir.join("mcp.json");
        write_mcp_servers_file(&cursor_path, "sail", &entry)?;
        writes.push(HostWrite {
            host: "cursor",
            path: cursor_path,
            action: "wrote",
        });
    }

    Ok(InstallReport {
        project_root,
        writes,
        command: launch.command,
        args: launch.args,
    })
}

/// Whether Cursor looks present for this project / user.
fn cursor_detected(project_root: &Path) -> bool {
    if project_root.join(".cursor").is_dir() {
        return true;
    }
    if let Some(home) = home_dir() {
        if home.join(".cursor").is_dir() {
            return true;
        }
    }
    // `cursor` CLI on PATH
    which_on_path("cursor")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn which_on_path(bin: &str) -> bool {
    let Ok(path) = std::env::var("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return true;
        }
        // Windows-ish
        let exe = dir.join(format!("{bin}.exe"));
        if exe.is_file() {
            return true;
        }
    }
    false
}

/// How hosts should launch the Sail MCP server.
struct Launch {
    command: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
}

/// Choose how hosts should launch the Sail MCP server.
fn server_launch(bin: Option<&Path>) -> Launch {
    if let Some(bin) = bin {
        let name = bin
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .trim_end_matches(".exe");
        let args = if name == "sail-mcp" {
            vec![]
        } else {
            vec!["mcp".into()]
        };
        return Launch {
            command: bin.display().to_string(),
            args,
            cwd: None,
        };
    }
    Launch {
        command: "sail".into(),
        args: vec!["mcp".into()],
        cwd: None,
    }
}

fn server_entry(launch: &Launch, project_root: &Path) -> Value {
    let mut obj = json!({
        "command": launch.command,
        "args": launch.args,
    });
    if let Some(cwd) = &launch.cwd {
        obj["cwd"] = json!(cwd.display().to_string());
    }
    // Scope filesystem tools to the project (apply_root / SAIL_MCP_ROOT).
    obj["env"] = json!({
        "SAIL_MCP_ROOT": project_root.display().to_string(),
    });
    obj
}

/// Merge `name` → `entry` into a host config file with top-level `mcpServers`.
fn write_mcp_servers_file(path: &Path, name: &str, entry: &Value) -> Result<(), String> {
    let mut root = if path.is_file() {
        let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let v: Value =
            serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
        if !v.is_object() {
            return Err(format!(
                "{}: expected JSON object with mcpServers",
                path.display()
            ));
        }
        v
    } else {
        json!({ "mcpServers": {} })
    };

    let servers = root
        .as_object_mut()
        .unwrap()
        .entry("mcpServers")
        .or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    servers
        .as_object_mut()
        .unwrap()
        .insert(name.to_string(), entry.clone());

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    fs::write(path, format!("{body}\n")).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(())
}

/// Human-readable summary for CLI stdout.
pub fn format_report(report: &InstallReport) -> String {
    let mut out = String::new();
    out.push_str("sail mcp install — MCP server entries written\n");
    out.push_str(&format!("  project: {}\n", report.project_root.display()));
    out.push_str(&format!(
        "  launch:  {} {}\n",
        report.command,
        report.args.join(" ")
    ));
    for w in &report.writes {
        out.push_str(&format!(
            "  [{}] {} → {}\n",
            w.host,
            w.action,
            w.path.display()
        ));
    }
    if report.writes.iter().any(|w| w.host == "claude-code") {
        out.push_str("  note: Claude Code reads project `.mcp.json` (mcpServers.sail).\n");
    }
    if report.writes.iter().any(|w| w.host == "cursor") {
        out.push_str("  note: Cursor reads project `.cursor/mcp.json`.\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn install_writes_claude_code_mcp_json() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let report = install(root).expect("install");
        assert!(
            report.writes.iter().any(|w| w.host == "claude-code"),
            "claude-code must always be written: {:?}",
            report.writes
        );
        let path = root.join(".mcp.json");
        assert!(path.is_file(), "expected {}", path.display());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let sail = &v["mcpServers"]["sail"];
        assert_eq!(sail["command"], "sail", "{sail}");
        assert_eq!(sail["args"], json!(["mcp"]), "{sail}");
        assert!(
            sail["command"]
                .as_str()
                .is_some_and(|c| !c.contains("cargo") && !c.contains("worktrees")),
            "must not pin cargo run or a worktree binary: {sail}"
        );
        assert_eq!(
            sail["env"]["SAIL_MCP_ROOT"].as_str(),
            Some(root.canonicalize().unwrap().to_str().unwrap())
        );
    }

    #[test]
    fn install_merges_existing_servers() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(
            root.join(".mcp.json"),
            r#"{ "mcpServers": { "other": { "command": "echo" } } }
"#,
        )
        .unwrap();
        let report = install(root).expect("install");
        assert!(!report.writes.is_empty());
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(".mcp.json")).unwrap())
                .unwrap();
        assert!(v["mcpServers"]["other"].is_object());
        assert!(v["mcpServers"]["sail"].is_object());
    }

    #[test]
    fn install_writes_cursor_when_project_has_cursor_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".cursor")).unwrap();
        let report = install(root).expect("install");
        assert!(
            report.writes.iter().any(|w| w.host == "cursor"),
            "cursor: {:?}",
            report.writes
        );
        let path = root.join(".cursor/mcp.json");
        assert!(path.is_file(), "{}", path.display());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["sail"]["args"], json!(["mcp"]));
    }
}
