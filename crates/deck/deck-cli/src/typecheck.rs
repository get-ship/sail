//! Opt-in `deck build --typecheck`: an on-the-fly tsconfig + `tsc --noEmit`
//! pass over the app's `.ts` sources (authored controllers + the bundler's
//! generated `routes.gen.ts` / `entry.gen.ts` / `meta.gen.ts` /
//! `components.gen.ts`), failing the build command on TypeScript errors.
//!
//! Default build behavior is unchanged — the flag is opt-in for v1. The
//! tsconfig + subprocess invocation are modeled on the compiler's
//! `crates/sail/compiler/tests/synthetic_tsc.rs` (`write_tsconfig` / `run_tsc`):
//! full-strict, `noEmit`, `moduleResolution: Bundler`, run with `tsc -p <cfg>`.
//!
//! Unlike the test (which SKIPs when `tsc` is absent), the flag was requested
//! explicitly, so a missing `node_modules/.bin/tsc` is a hard error.

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

/// Write the on-the-fly tsconfig into `out_dir` and return its path. Modeled on
/// `synthetic_tsc.rs::write_tsconfig`: full-strict, `noEmit`, `Bundler`
/// resolution. `include`/`exclude`/`baseUrl` are ABSOLUTE so the config can
/// live in the build output dir while still covering the project tree —
/// `@ship/*` and friends resolve through the consumer's own `node_modules`
/// (no `paths` map needed; a general consumer is not the sail workspace).
fn write_tsconfig(project: &Path, out_dir: &Path) -> Result<PathBuf> {
    let proj = project.display();
    let out = out_dir.display();
    let cfg = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "noImplicitAny": true,
    "skipLibCheck": true,
    "noEmit": true,
    "allowImportingTsExtensions": true,
    "baseUrl": "{proj}"
  }},
  "include": ["{proj}/**/*.ts"],
  "exclude": ["{proj}/node_modules", "{out}"]
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
        // Surface tsc's own diagnostics, then fail the command.
        eprint!("{output}");
        bail!("deck build --typecheck: tsc reported type errors");
    }
    println!("  typecheck  ok (tsc --noEmit)");
    Ok(())
}
