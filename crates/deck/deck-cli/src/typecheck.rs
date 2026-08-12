//! Opt-in `deck build --typecheck`: ambient pack + on-the-fly tsconfig +
//! `tsc --noEmit` over the app's `.ts` sources (authored controllers + the
//! bundler's generated `routes.gen.ts` / `entry.gen.ts` /
//! `components.gen.ts`), failing the build command on TypeScript errors.
//!
//! Default build behavior is unchanged — the flag is opt-in for v1.
//!
//! ## Two-checker contract
//!
//! - **tsc** — controller / scope / logic types; Style builder types when the
//!   source is valid TypeScript; ambient globals from
//!   `node_modules/.sail/types.d.ts`.
//! - **sail-compiler** — template / layout / vocabulary; Style DSL forms that
//!   are not expressible as TS (hyphenated bare tokens, etc.).
//!
//! Unlike the synthetic_tsc test (which SKIPs when `tsc` is absent), the flag
//! was requested explicitly, so a missing `node_modules/.bin/tsc` is a hard
//! error.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

/// Locate the workspace `tsc`. Walks up from `project` (covering `repo_root`
/// when it is an ancestor), then falls back to `repo_root` directly for the
/// cross-repo case where the two don't share a parent.
fn find_tsc(project: &Path, repo_root: &Path) -> Option<PathBuf> {
    let candidates = ["node_modules/.bin/tsc", "node_modules/.bin/tsc.cmd"];
    let mut dir = Some(project);
    while let Some(d) = dir {
        for rel in candidates {
            let p = d.join(rel);
            if p.exists() {
                return Some(p);
            }
        }
        dir = d.parent();
    }
    for rel in candidates {
        let p = repo_root.join(rel);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Write the ambient pack + on-the-fly tsconfig into `out_dir` and return the
/// tsconfig path. `include`/`exclude`/`baseUrl` are ABSOLUTE so the config can
/// live in the build output dir while still covering the project tree —
/// `@ship/*` resolves through the consumer's own `node_modules`.
fn write_tsconfig(project: &Path, out_dir: &Path) -> Result<PathBuf> {
    // Project-local pack (`node_modules/.sail/types.d.ts`) so editors + tsc share
    // one ambient surface. Also mirror into out_dir for the ephemeral project.
    let ambient = sail_compiler::ambients::write_project_types(project)?;
    let ambient_copy = out_dir.join("sail-types.d.ts");
    std::fs::copy(&ambient, &ambient_copy)
        .with_context(|| format!("copy {} → {}", ambient.display(), ambient_copy.display()))?;

    let proj = project.display().to_string().replace('\\', "/");
    let out = out_dir.display().to_string().replace('\\', "/");
    // Prefer the out_dir copy so `exclude: node_modules` cannot drop the components package
    // (project pack lives under `node_modules/.sail/`).
    let ambient_for_tsc = ambient_copy
        .canonicalize()
        .unwrap_or(ambient_copy)
        .display()
        .to_string()
        .replace('\\', "/");
    let opts = sail_compiler::ambients::sail_tsconfig_compiler_options_json();
    // Drop outer braces from the shared options object so we can inject baseUrl.
    let opts_inner = opts
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    let cfg = format!(
        r#"{{
  "compilerOptions": {{
    {opts_inner},
    "baseUrl": "{proj}"
  }},
  "files": ["{ambient_for_tsc}"],
  "include": [
    "{proj}/**/*.ts"
  ],
  "exclude": ["{proj}/node_modules", "{out}", "{proj}/.deck"]
}}"#
    );
    let path = out_dir.join("tsconfig.typecheck.json");
    std::fs::write(&path, cfg).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

/// Run `tsc -p <cfg> --noEmit` (cwd = `repo_root`, so node-module resolution
/// and the binary's own libs resolve), returning the combined stdout+stderr.
fn run_tsc(tsc: &Path, cfg: &Path, repo_root: &Path) -> Result<(bool, String)> {
    let out = Command::new(tsc)
        .arg("-p")
        .arg(cfg)
        .arg("--noEmit")
        .current_dir(repo_root)
        .output()
        .with_context(|| format!("run {}", tsc.display()))?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Ok((out.status.success(), combined))
}

/// Typecheck the freshly-built app. Errors (a missing `tsc`, or a `tsc`
/// failure) bubble up so `deck build --typecheck` exits non-zero.
pub fn run(project: &Path, out_dir: &Path, repo_root: &Path) -> Result<()> {
    let Some(tsc) = find_tsc(project, repo_root) else {
        bail!(
            "--typecheck requires `node_modules/.bin/tsc` (searched up from {} and at {}). \
             Install dependencies first (e.g. `bun install` / `npm install`).",
            project.display(),
            repo_root.display(),
        );
    };
    let cfg = write_tsconfig(project, out_dir)?;
    let (ok, output) = run_tsc(&tsc, &cfg, repo_root)?;
    if !ok {
        eprint!("{output}");
        bail!("deck build --typecheck: tsc reported type errors");
    }
    println!(
        "  typecheck  ok (tsc --noEmit; ambients {})",
        sail_compiler::ambients::PROJECT_TYPES_REL
    );
    Ok(())
}
