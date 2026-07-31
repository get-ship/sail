//! Codegen: emit TS client + schema snapshots from typed contracts.
//!
//! Invoked by `sail op gen` (deck-cli) or tests.
//! Named `codegen` because `gen` is a Rust 2024 reserved keyword.

use std::fs;
use std::path::{Path, PathBuf};

use crate::contracts::METHOD_CONTRACTS;
use crate::typed::{
    generate_ts_client, mcp_descriptors_from, method_schema_snapshot, schema_catalog,
};

/// Default relative output for the committed TS client package.
pub const DEFAULT_TS_OUT: &str = "packages/sail/studio-client/src/generated/client.ts";

/// Schema snapshot directory (committed, drift-checked).
pub const DEFAULT_SCHEMA_DIR: &str = "crates/sail/ops/schemas";

#[derive(Debug, Clone)]
pub struct GenPaths {
    pub workspace_root: PathBuf,
    pub ts_out: PathBuf,
    pub schema_dir: PathBuf,
}

impl GenPaths {
    pub fn from_workspace(root: impl Into<PathBuf>) -> Self {
        let workspace_root = root.into();
        Self {
            ts_out: workspace_root.join(DEFAULT_TS_OUT),
            schema_dir: workspace_root.join(DEFAULT_SCHEMA_DIR),
            workspace_root,
        }
    }
}

/// Write generated artifacts. Returns paths written.
pub fn generate_all(paths: &GenPaths) -> Result<Vec<PathBuf>, String> {
    let mut written = Vec::new();

    // TS client
    let ts = generate_ts_client(METHOD_CONTRACTS);
    if let Some(parent) = paths.ts_out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    write_if_changed(&paths.ts_out, &ts)?;
    written.push(paths.ts_out.clone());

    // package index re-export (stable entry)
    let index = paths
        .ts_out
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("index.ts"))
        .ok_or_else(|| "ts_out has no package parent".to_string())?;
    let index_body = "/** Public entry — re-exports generated sail-ops client. */\nexport * from './generated/client'\n";
    write_if_changed(&index, index_body)?;
    written.push(index);

    // Per-method schema snapshots
    fs::create_dir_all(&paths.schema_dir)
        .map_err(|e| format!("mkdir {}: {e}", paths.schema_dir.display()))?;
    for c in METHOD_CONTRACTS {
        let name = format!("{}_{}.json", c.service, c.method);
        let path = paths.schema_dir.join(&name);
        let body = method_schema_snapshot(c);
        write_if_changed(&path, &format!("{body}\n"))?;
        written.push(path);
    }

    // Full catalog
    let catalog_path = paths.schema_dir.join("catalog.json");
    let catalog = serde_json::to_string_pretty(&schema_catalog(METHOD_CONTRACTS))
        .map_err(|e| e.to_string())?;
    write_if_changed(&catalog_path, &format!("{catalog}\n"))?;
    written.push(catalog_path);

    // MCP tools/list artifact (stdio parity lock)
    let mcp_path = paths.schema_dir.join("mcp-tools.json");
    let mcp = serde_json::to_string_pretty(&mcp_descriptors_from(METHOD_CONTRACTS))
        .map_err(|e| e.to_string())?;
    write_if_changed(&mcp_path, &format!("{mcp}\n"))?;
    written.push(mcp_path);

    Ok(written)
}

fn write_if_changed(path: &Path, contents: &str) -> Result<(), String> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == contents => Ok(()),
        _ => fs::write(path, contents).map_err(|e| format!("write {}: {e}", path.display())),
    }
}

fn expected_catalog() -> Result<String, String> {
    let catalog = serde_json::to_string_pretty(&schema_catalog(METHOD_CONTRACTS))
        .map_err(|e| e.to_string())?;
    Ok(format!("{catalog}\n"))
}

fn expected_mcp_tools() -> Result<String, String> {
    let mcp = serde_json::to_string_pretty(&mcp_descriptors_from(METHOD_CONTRACTS))
        .map_err(|e| e.to_string())?;
    Ok(format!("{mcp}\n"))
}

/// Drift check: regenerate in memory and compare to committed TS + schemas + catalog + mcp-tools.
pub fn assert_no_drift(paths: &GenPaths) -> Result<(), String> {
    let expected_ts = generate_ts_client(METHOD_CONTRACTS);
    let actual_ts = fs::read_to_string(&paths.ts_out).map_err(|e| {
        format!(
            "missing generated TS client at {} ({e}); run `sail op gen`",
            paths.ts_out.display()
        )
    })?;
    if actual_ts != expected_ts {
        return Err(format!(
            "drift: {} is stale; run `sail op gen`",
            paths.ts_out.display()
        ));
    }

    for c in METHOD_CONTRACTS {
        let name = format!("{}_{}.json", c.service, c.method);
        let path = paths.schema_dir.join(&name);
        let expected = format!("{}\n", method_schema_snapshot(c));
        let actual = fs::read_to_string(&path)
            .map_err(|e| format!("missing schema snapshot {} ({e})", path.display()))?;
        if actual != expected {
            return Err(format!(
                "drift: schema snapshot {} is stale; run `sail op gen`",
                path.display()
            ));
        }
    }

    let catalog_path = paths.schema_dir.join("catalog.json");
    let expected_cat = expected_catalog()?;
    let actual_cat = fs::read_to_string(&catalog_path).map_err(|e| {
        format!(
            "missing catalog.json at {} ({e}); run `sail op gen`",
            catalog_path.display()
        )
    })?;
    if actual_cat != expected_cat {
        return Err(format!(
            "drift: {} is stale; run `sail op gen`",
            catalog_path.display()
        ));
    }

    let mcp_path = paths.schema_dir.join("mcp-tools.json");
    let expected_mcp = expected_mcp_tools()?;
    let actual_mcp = fs::read_to_string(&mcp_path).map_err(|e| {
        format!(
            "missing mcp-tools.json at {} ({e}); run `sail op gen`",
            mcp_path.display()
        )
    })?;
    if actual_mcp != expected_mcp {
        return Err(format!(
            "drift: {} is stale; run `sail op gen`",
            mcp_path.display()
        ));
    }

    Ok(())
}

/// Resolve workspace root: CARGO_MANIFEST_DIR/../../.. for sail-ops, or cwd walk.
pub fn find_workspace_root() -> PathBuf {
    if let Ok(p) = std::env::var("CARGO_MANIFEST_DIR") {
        // crates/sail/ops → repo root
        let manifest = PathBuf::from(p);
        if let Some(root) = manifest
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
        {
            if root.join("Cargo.toml").is_file() {
                return root.to_path_buf();
            }
        }
    }
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..8 {
        if dir.join("Cargo.toml").is_file() && dir.join("packages/sail").is_dir() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
