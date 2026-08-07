//! Execution context for sail-ops handlers.

use std::path::{Path, PathBuf};

use crate::error::OpError;

/// Roots and policy for a single ops invocation / server lifetime.
#[derive(Debug, Clone)]
pub struct Ctx {
    /// Workspace root (theme paths resolve here).
    pub workspace_root: PathBuf,
    /// Active project root (`routes/` + `package.json`).
    pub project_root: PathBuf,
    /// Root confinement for `SailAuthoring.Edit` writes.
    pub apply_root: PathBuf,
}

impl Ctx {
    /// Build from explicit roots.
    pub fn new(
        workspace_root: impl Into<PathBuf>,
        project_root: impl Into<PathBuf>,
        apply_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            project_root: project_root.into(),
            apply_root: apply_root.into(),
        }
    }

    /// MCP / CLI defaults: cwd for project + apply; topmost Cargo.toml as workspace.
    pub fn from_env() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let workspace_root = topmost_cargo_root(&cwd).unwrap_or_else(|| cwd.clone());
        let apply_root = apply_root_from_env(&cwd);
        Self {
            workspace_root,
            project_root: cwd,
            apply_root,
        }
    }

    /// Dev server: known workspace + project; apply root = workspace.
    pub fn for_dev(workspace_root: impl Into<PathBuf>, project_root: impl Into<PathBuf>) -> Self {
        let workspace_root = workspace_root.into();
        let project_root = project_root.into();
        Self {
            apply_root: workspace_root.clone(),
            workspace_root,
            project_root,
        }
    }

    pub fn workspace(&self) -> &Path {
        &self.workspace_root
    }

    pub fn project(&self) -> &Path {
        &self.project_root
    }

    /// Resolve a **workspace-scoped** read path (TOOLING-1 WI4 / sail_check path mode).
    ///
    /// Rules:
    /// - relative only (absolute paths refused)
    /// - no `..` components
    /// - resolved file (or parent chain) must sit under [`Self::workspace_root`]
    /// - tries `workspace_root/raw` first, then `project_root/raw`
    ///
    /// Returns the canonical absolute path on success.
    pub fn resolve_workspace_read_path(&self, raw: &str) -> Result<PathBuf, OpError> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(OpError::InvalidArgument("path is required".into()));
        }
        if raw.contains("..") {
            return Err(OpError::PermissionDenied(
                "path must not contain `..`".into(),
            ));
        }
        if Path::new(raw).is_absolute() {
            return Err(OpError::PermissionDenied(
                "path must be workspace-relative (absolute paths refused)".into(),
            ));
        }
        let under_ws = self.workspace_root.join(raw);
        let candidate = if under_ws.exists() {
            under_ws
        } else {
            self.project_root.join(raw)
        };
        let canon_root = self
            .workspace_root
            .canonicalize()
            .map_err(|e| OpError::Internal(format!("workspace root: {e}")))?;
        if let Ok(partial) = candidate.canonicalize() {
            if !partial.starts_with(&canon_root) {
                return Err(OpError::PermissionDenied("path escapes workspace".into()));
            }
            return Ok(partial);
        }
        // Missing file: still require the parent chain sits under workspace.
        let parent = candidate.parent().unwrap_or(candidate.as_path());
        if let Ok(cp) = parent.canonicalize() {
            if !cp.starts_with(&canon_root) {
                return Err(OpError::PermissionDenied("path escapes workspace".into()));
            }
        }
        Err(OpError::NotFound(format!("read {raw}: not found")))
    }
}

fn apply_root_from_env(cwd: &Path) -> PathBuf {
    if let Ok(root) = std::env::var("SAIL_MCP_ROOT") {
        let p = PathBuf::from(root);
        if p.is_absolute() {
            return p;
        }
        return cwd.join(p);
    }
    cwd.to_path_buf()
}

fn topmost_cargo_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start;
    let mut topmost = None;
    for _ in 0..24 {
        if cur.join("Cargo.toml").is_file() {
            topmost = Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    topmost
}
