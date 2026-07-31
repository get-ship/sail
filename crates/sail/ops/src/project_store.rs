//! Project-local store helpers under `<project>/.sail/` and app `.gitignore`.

use std::collections::HashSet;
use std::path::Path;

use crate::error::OpError;

/// Canonical Sail app `.gitignore` body (no trailing blank-line requirement).
///
/// Never default-ignore: `tokens/themes/**`, `sail.config.json(c)`.
pub const CANONICAL_GITIGNORE: &str = r#"# Dependencies & generic build
node_modules/
dist/
*.tsbuildinfo
*.log
.DS_Store

# Sail / deck build output (SPA + B1 app themes.css)
.deck/

# Codegen (sail routes / build / dev)
routes.gen.ts
entry.gen.ts
components.gen.ts

# Compiler companions next to .sail sources
*.sail.d.ts
*.controller.ts

# Env / secrets
.env
.env.*
!.env.example

# Test runners
test-results/
playwright-report/
blob-report/
.cache/
coverage/

# Ephemeral Sail local store (tasks, agent scratch — never commit)
.sail/
"#;

/// Critical lines that must exist after merge (idempotent ensure).
pub const CRITICAL_GITIGNORE_LINES: &[&str] = &[
    ".deck/",
    ".sail/",
    "routes.gen.ts",
    "entry.gen.ts",
    "components.gen.ts",
    "node_modules/",
    ".env",
];

const SAIL_README: &str = r#"# Sail local store (gitignored)

- `tasks/` — ephemeral agent/human threads for this machine
- Build output is `.deck/`, not here
"#;

/// Result of [`ensure_sail_gitignore`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnsureGitignoreResult {
    Created,
    Merged,
    Unchanged,
}

/// Reject empty / non-directory project roots before writing.
///
/// Writes are confined to fixed basenames under `project_dir` (`.gitignore`,
/// `.sail/README.md`) — never arbitrary relative paths from the caller.
fn validate_project_dir(project_dir: &Path) -> Result<(), OpError> {
    if project_dir.as_os_str().is_empty() {
        return Err(OpError::InvalidArgument(
            "project_dir must not be empty".into(),
        ));
    }
    if !project_dir.is_dir() {
        return Err(OpError::InvalidArgument(format!(
            "project dir does not exist: {}",
            project_dir.display()
        )));
    }
    Ok(())
}

/// Ensure `<project>/.gitignore` contains the Sail critical set.
///
/// - Missing file → write full canonical block + short header.
/// - Existing → append any missing critical lines (never delete user rules).
/// - Second run is a no-op when the critical set is already present.
///
/// Writes only `<project>/.gitignore` (confined fixed basename).
pub fn ensure_sail_gitignore(project_dir: &Path) -> Result<EnsureGitignoreResult, OpError> {
    validate_project_dir(project_dir)?;
    let path = project_dir.join(".gitignore");
    if !path.exists() {
        let body = format!(
            "# Sail app defaults (managed by ensure_sail_gitignore)\n{CANONICAL_GITIGNORE}"
        );
        std::fs::write(&path, body)
            .map_err(|e| OpError::Internal(format!("write {}: {e}", path.display())))?;
        return Ok(EnsureGitignoreResult::Created);
    }

    let existing = std::fs::read_to_string(&path)
        .map_err(|e| OpError::Internal(format!("read {}: {e}", path.display())))?;
    let present: HashSet<&str> = existing
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    let missing: Vec<&str> = CRITICAL_GITIGNORE_LINES
        .iter()
        .copied()
        .filter(|line| !gitignore_has(&present, line))
        .collect();

    if missing.is_empty() {
        return Ok(EnsureGitignoreResult::Unchanged);
    }

    let mut out = existing;
    if !out.ends_with('\n') && !out.is_empty() {
        out.push('\n');
    }
    out.push_str("\n# Sail defaults (merged by ensure_sail_gitignore)\n");
    for line in &missing {
        out.push_str(line);
        out.push('\n');
    }
    std::fs::write(&path, out)
        .map_err(|e| OpError::Internal(format!("write {}: {e}", path.display())))?;
    Ok(EnsureGitignoreResult::Merged)
}

/// Match a critical line against present entries, accepting leading-slash variants
/// (`/.deck/` counts as `.deck/`).
fn gitignore_has(present: &HashSet<&str>, line: &str) -> bool {
    let want = strip_leading_slashes(line);
    present
        .iter()
        .any(|entry| strip_leading_slashes(entry) == want)
}

fn strip_leading_slashes(s: &str) -> &str {
    s.trim_start_matches('/')
}

/// Create `.sail/README.md` if missing (does not create empty `tasks/`).
///
/// Writes only under `<project>/.sail/` (confined).
pub fn ensure_sail_readme(project_dir: &Path) -> Result<(), OpError> {
    validate_project_dir(project_dir)?;
    let sail_dir = project_dir.join(".sail");
    if !sail_dir.exists() {
        std::fs::create_dir_all(&sail_dir)
            .map_err(|e| OpError::Internal(format!("mkdir {}: {e}", sail_dir.display())))?;
    }
    let readme = sail_dir.join("README.md");
    if !readme.exists() {
        std::fs::write(&readme, SAIL_README)
            .map_err(|e| OpError::Internal(format!("write {}: {e}", readme.display())))?;
    }
    Ok(())
}

/// Best-effort project store init: gitignore + `.sail/README.md`.
///
/// Call from theme scaffold and (U5) first task create.
pub fn ensure_project_store(project_dir: &Path) -> Result<(), OpError> {
    let _ = ensure_sail_gitignore(project_dir)?;
    ensure_sail_readme(project_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn creates_full_canonical_when_missing() {
        let dir = tempdir().unwrap();
        let r = ensure_sail_gitignore(dir.path()).unwrap();
        assert_eq!(r, EnsureGitignoreResult::Created);
        let body = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        for line in CRITICAL_GITIGNORE_LINES {
            assert!(
                body.lines().any(|l| l.trim() == *line),
                "missing critical {line:?} in:\n{body}"
            );
        }
        assert!(body.contains(".sail/"));
        assert!(body.contains(".deck/"));
        // Themes / config must never be in the default ignore set.
        assert!(!body.contains("tokens/themes"));
        assert!(!body.contains("sail.config"));
    }

    #[test]
    fn merge_preserves_user_lines_and_adds_critical() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join(".gitignore"), "my-custom/\nnode_modules/\n").unwrap();
        let r = ensure_sail_gitignore(dir.path()).unwrap();
        assert_eq!(r, EnsureGitignoreResult::Merged);
        let body = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(body.contains("my-custom/"));
        assert!(body.contains("node_modules/"));
        for line in CRITICAL_GITIGNORE_LINES {
            assert!(
                body.lines().any(|l| {
                    let t = l.trim();
                    strip_leading_slashes(t) == strip_leading_slashes(line)
                }),
                "missing critical {line:?}"
            );
        }
        // Idempotent
        let r2 = ensure_sail_gitignore(dir.path()).unwrap();
        assert_eq!(r2, EnsureGitignoreResult::Unchanged);
    }

    #[test]
    fn accepts_leading_slash_variants() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join(".gitignore"),
            "/.deck/\n/.sail/\n/routes.gen.ts\n/entry.gen.ts\n/components.gen.ts\nnode_modules/\n.env\n",
        )
        .unwrap();
        let r = ensure_sail_gitignore(dir.path()).unwrap();
        assert_eq!(r, EnsureGitignoreResult::Unchanged);
    }

    #[test]
    fn sail_readme_written_once() {
        let dir = tempdir().unwrap();
        ensure_sail_readme(dir.path()).unwrap();
        let p = dir.path().join(".sail/README.md");
        assert!(p.is_file());
        let body = std::fs::read_to_string(&p).unwrap();
        assert!(body.contains("tasks/"));
        // second call leaves content
        std::fs::write(&p, "custom").unwrap();
        ensure_sail_readme(dir.path()).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "custom");
    }

    #[test]
    fn ensure_project_store_writes_both() {
        let dir = tempdir().unwrap();
        ensure_project_store(dir.path()).unwrap();
        assert!(dir.path().join(".gitignore").is_file());
        assert!(dir.path().join(".sail/README.md").is_file());
    }

    #[test]
    fn rejects_missing_project_dir() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("nope");
        let err = ensure_sail_gitignore(&missing).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }
}
