//! Execution context for sail-ops handlers.

use std::path::{Path, PathBuf};

use crate::error::OpError;

/// Roots and policy for a single ops invocation / server lifetime.
#[derive(Debug, Clone)]
pub struct Ctx {
    /// Workspace root (theme paths resolve here).
    pub workspace_root: PathBuf,
    /// Active Sail project (`sail.config.jsonc` / `.json`).
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

    /// Bind to the Sail project that owns `from`.
    ///
    /// Workspace is this checkout's Cargo root **only when `from` sits
    /// inside it**. A standalone app (tempdir, user machine) gets
    /// workspace = project. Never cargo-as-project.
    pub fn for_source(from: &Path) -> Self {
        let abs = if from.is_absolute() {
            from.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(from)
        };
        let project_root = sail_lint::Project::discover(&abs)
            .map(|p| p.root().to_path_buf())
            .unwrap_or_else(|| {
                if abs.is_dir() {
                    abs.clone()
                } else {
                    abs.parent().unwrap_or(abs.as_path()).to_path_buf()
                }
            });
        let workspace_root = topmost_cargo_root(&abs)
            .filter(|w| {
                let w = w.canonicalize().unwrap_or_else(|_| w.clone());
                let p = project_root
                    .canonicalize()
                    .unwrap_or_else(|_| project_root.clone());
                p.starts_with(&w)
            })
            .unwrap_or_else(|| project_root.clone());
        Self {
            apply_root: project_root.clone(),
            workspace_root,
            project_root,
        }
    }

    /// MCP / CLI defaults from cwd. Same map as [`Self::for_source`];
    /// `SAIL_MCP_ROOT` may redirect apply confinement.
    pub fn from_env() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut ctx = Self::for_source(&cwd);
        ctx.apply_root = apply_root_from_env(&cwd);
        ctx
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn for_source_standalone_app_is_its_own_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let app = tmp.path().join("hello");
        fs::create_dir_all(app.join("routes")).unwrap();
        fs::write(app.join("sail.config.jsonc"), "{}\n").unwrap();
        fs::write(app.join("routes/page.sail"), "<p>x</p>\n").unwrap();
        let ctx = Ctx::for_source(&app.join("routes/page.sail"));
        assert_eq!(ctx.project_root, app);
        assert_eq!(ctx.workspace_root, app);
        assert_eq!(ctx.apply_root, app);
        let rel = ctx
            .resolve_workspace_read_path("routes/page.sail")
            .expect("project-relative path");
        assert_eq!(rel, app.join("routes/page.sail").canonicalize().unwrap());
    }

    #[test]
    fn for_source_ignores_a_sibling_cargo_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("Cargo.toml"), "[workspace]\n").unwrap();
        let app = tmp.path().join("hello");
        fs::create_dir_all(app.join("routes")).unwrap();
        fs::write(app.join("sail.config.jsonc"), "{}\n").unwrap();
        fs::write(app.join("routes/page.sail"), "<p>x</p>\n").unwrap();
        let ctx = Ctx::for_source(&app.join("routes/page.sail"));
        assert_eq!(ctx.project_root, app);
        assert_eq!(
            ctx.workspace_root, app,
            "a Cargo tree that is not an ancestor of the app is not the workspace"
        );
    }

    #[test]
    fn topmost_cargo_root_agrees_with_deck_bundler() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers=[]").unwrap();
        let inner = tmp.path().join("crates/inner");
        fs::create_dir_all(&inner).unwrap();
        fs::write(inner.join("Cargo.toml"), "[package]\nname=\"inner\"\n").unwrap();
        assert_eq!(
            topmost_cargo_root(&inner),
            deck_bundler::paths::topmost_cargo_root(&inner)
        );
        let isolated = tempfile::tempdir().unwrap();
        assert_eq!(
            topmost_cargo_root(isolated.path()),
            deck_bundler::paths::topmost_cargo_root(isolated.path())
        );
    }
}

/// Copy of `deck_bundler::paths::topmost_cargo_root` (16-hop, topmost
/// `Cargo.toml`). Workspace / `@ship` bake lives in deck-bundler; lint
/// must not grow a Cargo walker; this crate does not depend on deck-bundler.
fn topmost_cargo_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start;
    let mut topmost: Option<PathBuf> = None;
    for _ in 0..16 {
        if cur.join("Cargo.toml").exists() {
            topmost = Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    topmost
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
