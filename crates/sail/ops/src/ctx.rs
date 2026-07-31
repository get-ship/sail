//! Execution context for sail-ops handlers.

use std::path::{Path, PathBuf};

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
