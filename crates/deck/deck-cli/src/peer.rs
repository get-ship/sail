//! Peer subcommands for the unified `sail` CLI surface (C-package).
//!
//! Wraps existing crates — does not invent second protocols:
//! - `compile` / `tokens` / `check` → `sail-compiler` library
//! - `lsp` → `sail-lsp` library (stdio LSP)
//! - `mcp` → `sail-mcp` library (stdio / Streamable HTTP, MCP 2026-07-28)
//! - `layout` → `sail-layout` library
//! - `doctor` / `version` → local diagnostics

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use sail_compiler::out_layout::resolve_out_dir;
use sail_compiler::{
    CompileMode, CompileOpts, ComponentMeta, DiagSeverity, Policy, check_all,
    compile as compile_source,
};

/// `sail --version` / `sail version` surface (C6).
pub fn print_version(brand: &str) {
    let pkg = env!("CARGO_PKG_VERSION");
    println!("{brand} {pkg}");
    // Cargo package id is historical (`deck-cli`); the product binary is `sail`.
    // Deck crates (bundler/axum/core/host) are linked in-process — not shell-outs.
    println!("  binary: sail (cargo package deck-cli {pkg})");
    println!("  embeds: sail-compiler, sail-lsp, sail-mcp, sail-layout, sail-ops, sail-render,");
    println!("          deck-bundler, deck-axum, deck-core, deck-config, deck-ssr (in-process)");
    println!("  install: product install TBD — put `sail` on PATH");
    println!("  from this repo: cargo install --path crates/deck/deck-cli --bin sail");
    println!("  docs:    README.md § Build / CLI; `sail doctor` for local health");
}

/// Local environment health check (C6).
///
/// Never prints `status: ok` on an empty probe. bun and tsc/tsgo are
/// required. When a Project exists, `<project>/.sail/types.d.ts` is
/// required for a first-user typecheck.
/// MCP is probed in-process (`sail mcp --list-tools` equivalent) so doctor
/// does not require a different `sail` on PATH or a project MCP config.
pub fn doctor(brand: &str) -> Result<()> {
    println!("{brand} doctor");
    println!("  binary: {brand} {}", env!("CARGO_PKG_VERSION"));

    let cwd = std::env::current_dir().context("cwd")?;
    let cargo_root = deck_bundler::paths::topmost_cargo_root(&cwd);
    match &cargo_root {
        Some(root) => println!("  workspace: {}", root.display()),
        None => println!("  workspace: (none — standalone project)"),
    }

    let model = cargo_root
        .as_ref()
        .map(|r| r.join("packages/sail/tokens/model"));
    match model {
        Some(path) if path.is_dir() => {
            println!("  tokens model: {} (ok)", path.display());
        }
        Some(path) => {
            println!(
                "  tokens model: baked catalog (monorepo disk model missing at {})",
                path.display()
            );
        }
        None => {
            println!("  tokens model: baked catalog (ok)");
        }
    }

    // Peer libraries linked into this binary — not PATH probes.
    println!("  sail compile/tokens/check/fmt: in-process (sail-compiler)");
    println!("  sail inspect: in-process (sail-ops SailInspect/Inspect)");
    println!("  sail lsp: in-process (sail-lsp)");
    println!("  sail mcp: in-process (sail-mcp → sail-ops)");
    println!("  sail op: in-process (sail-ops Connect handlers)");
    println!("  sail layout: in-process (sail-layout)");
    println!("  sail render: in-process (sail-render)");

    let mut missing = Vec::new();
    // In-process `--list-tools` (this binary embeds sail-mcp). Do not spawn
    // a different `sail` from PATH — doctor must pass inside the monorepo
    // and must not require a project `.mcp.json`.
    match probe_mcp_list_tools() {
        Ok(n) => println!("  sail mcp --list-tools: {n} tools (ok)"),
        Err(e) => {
            println!("  sail mcp --list-tools: fail ({e})");
            missing.push("sail mcp --list-tools");
        }
    }
    if !tool_ok(&cwd, "bun", &["--version"]) {
        missing.push("bun");
    } else {
        println!("  bun: ok");
    }
    let tsc = tool_ok(&cwd, "tsc", &["--version"]);
    let tsgo = tool_ok(&cwd, "tsgo", &["--version"]);
    if !tsc && !tsgo {
        missing.push("tsc or tsgo");
    } else {
        if tsc {
            println!("  tsc: ok");
        }
        if tsgo {
            println!("  tsgo: ok");
        }
    }
    if let Some(project) = sail_lint::Project::discover(&cwd) {
        let types = project
            .root()
            .join(sail_compiler::ambients::PROJECT_TYPES_REL);
        if types.is_file() {
            println!("  types: {}", types.display());
        } else {
            println!("  types: missing at {}", types.display());
            missing.push(".sail/types.d.ts (run `sail types`)");
        }
    } else {
        println!("  types: (no project — skipped)");
    }

    if missing.is_empty() {
        println!("  status: ok");
        Ok(())
    } else {
        println!("  status: fail");
        bail!("doctor: missing {}", missing.join(", "));
    }
}

/// Same path as `sail mcp --list-tools` / `print_list_tools`, in-process.
fn probe_mcp_list_tools() -> Result<usize> {
    let tools = sail_mcp::list_tools().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(tools.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn mcp_list_tools_probe_nonempty() {
        let n = super::probe_mcp_list_tools().expect("in-process list-tools");
        assert!(n > 0, "list-tools returned empty");
    }

    #[test]
    fn types_refuses_dir_without_sail_config() {
        let dir = tempfile::tempdir().unwrap();
        let err = types(Some(dir.path().to_path_buf()), None, None, false).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("inside a project") && msg.contains("--project"),
            "expected refuse message, got: {msg}"
        );
        assert!(
            !dir.path().join("tsconfig.json").exists(),
            "types must not write tsconfig.json outside a Sail project"
        );
    }

    #[test]
    fn types_refuses_explicit_project_without_sail_config() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("not-an-app")).unwrap();
        let target = dir.path().join("not-an-app");
        let err = types(
            Some(target.clone()),
            Some(dir.path().join("out.d.ts")),
            None,
            false,
        )
        .unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("inside a project"),
            "expected refuse even with --out, got: {msg}"
        );
        assert!(!target.join("tsconfig.json").exists());
        assert!(!dir.path().join("out.d.ts").exists());
    }

    #[test]
    fn require_sail_project_accepts_jsonc() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("sail.config.jsonc"), "{}\n").unwrap();
        require_sail_project(dir.path()).unwrap();
    }

    #[test]
    fn tokens_without_flags_refuses_when_no_project() {
        let dir = tempfile::tempdir().unwrap();
        let err = tokens(None, None, Some(dir.path().to_path_buf())).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("pass --model/--out (bake)") || msg.contains("not inside a Sail project"),
            "expected bake-flag refuse or missing project, got: {msg}"
        );
    }

    #[test]
    fn tokens_one_bake_flag_refuses_default_packages_path() {
        let dir = tempfile::tempdir().unwrap();
        let err = tokens(Some(dir.path().join("model")), None, None).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("pass --model/--out (bake)"),
            "must not default --out to packages/sail/tokens, got: {msg}"
        );
        assert!(
            !msg.contains("packages/sail/tokens"),
            "must not mention the monorepo default path, got: {msg}"
        );
    }
}

fn tool_ok(cwd: &Path, bin: &str, args: &[&str]) -> bool {
    let local = cwd.join("node_modules/.bin").join(bin);
    if local.is_file() && cmd_ok(&local, args) {
        return true;
    }
    cmd_ok(Path::new(bin), args)
}

fn cmd_ok(bin: &Path, args: &[&str]) -> bool {
    Command::new(bin)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// `sail types` — ambient pack for tsc/tsgo.
///
/// Product types writes `.sail` ambients (and kind tsconfigs) only.
/// Package `dist/*.d.ts` is bun prepare / bake (`scripts/emit-package-dts.ts`).
///
/// `project_dir` must itself be a Sail project (`dir_has_config`). The
/// monorepo root is not one — refuse so we never write a root `tsconfig.json`.
pub fn types(
    project: Option<PathBuf>,
    out: Option<PathBuf>,
    tsconfig_out: Option<PathBuf>,
    check: bool,
) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let project_dir = match project {
        Some(p) if p.is_absolute() => p,
        Some(p) => cwd.join(p),
        None => cwd,
    };
    require_sail_project(&project_dir)?;
    let path = match out {
        Some(p) => {
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("mkdir {}", parent.display()))?;
                }
            }
            let text = sail_compiler::ambients::emit_types_dts();
            std::fs::write(&p, &text).with_context(|| format!("write {}", p.display()))?;
            // Still regenerate solution + kind configs next to the project (G2c).
            let _ = sail_compiler::ambients::write_project_type_configs(&project_dir)?;
            p
        }
        None => {
            let (types, pure, action) =
                sail_compiler::ambients::write_project_type_configs(&project_dir)?;
            println!("sail types: tsconfig.sail → {}", pure.display());
            println!("sail types: tsconfig.action → {}", action.display());
            types
        }
    };
    println!("sail types: {}", path.display());
    if let Some(ts_out) = tsconfig_out {
        // Explicit dump path: sail kind body (IDE child under .sail/), not the
        // solution root — solution is always written as project/tsconfig.json.
        let body = sail_compiler::ambients::emit_project_tsconfig_json(
            sail_compiler::ambients::PROJECT_SAIL_INCLUDE,
        );
        if ts_out.as_os_str() == "-" {
            print!("{body}");
        } else {
            if let Some(parent) = ts_out.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("mkdir {}", parent.display()))?;
                }
            }
            std::fs::write(&ts_out, body).with_context(|| format!("write {}", ts_out.display()))?;
            println!("sail types: tsconfig → {}", ts_out.display());
        }
    }
    if check {
        types_check(&project_dir)?;
    }
    Ok(())
}

/// Refuse unless `dir` itself contains `sail.config.jsonc` / `sail.config.json`.
/// Uses [`sail_lint::config::dir_has_config`] — no second filename walk.
fn require_sail_project(dir: &Path) -> Result<()> {
    if sail_lint::config::dir_has_config(dir) {
        return Ok(());
    }
    bail!(
        "sail types: {} is not a Sail project (no sail.config.jsonc / sail.config.json). Run inside a project (`--project <app>`).",
        dir.display()
    );
}

/// Run tsc on the generated sail + action kind configs. Fail if red.
fn types_check(project_dir: &Path) -> Result<()> {
    let sail = project_dir.join(sail_compiler::ambients::PROJECT_TSCONFIG_SAIL_REL);
    let action = project_dir.join(sail_compiler::ambients::PROJECT_TSCONFIG_ACTION_REL);
    if !sail.is_file() || !action.is_file() {
        bail!(
            "sail types --check: kind configs missing (expected {} and {})",
            sail.display(),
            action.display()
        );
    }
    let local_tsc = project_dir.join("node_modules/.bin/tsc");
    let tsc: PathBuf = if local_tsc.is_file() {
        local_tsc.clone()
    } else {
        PathBuf::from("tsc")
    };
    for cfg in [&sail, &action] {
        let status = Command::new(&tsc)
            .args(["--noEmit", "-p"])
            .arg(cfg)
            .current_dir(project_dir)
            .status()
            .with_context(|| {
                if !local_tsc.is_file() {
                    format!(
                        "sail types --check: tsc not found (no {} and no PATH tsc). \
                         typescript is a create devDep — bun install after @ship packages are published",
                        local_tsc.display()
                    )
                } else {
                    format!("run tsc -p {}", cfg.display())
                }
            })?;
        if !status.success() {
            bail!(
                "sail types --check: tsc -p {} failed (exit {:?})",
                cfg.display(),
                status.code()
            );
        }
        println!("sail types --check: {} ok", cfg.display());
    }
    Ok(())
}

/// `sail tokens` — project themes, or explicit `--model`/`--out` bake.
///
/// Without both bake flags: generate project themes if a Project exists;
/// else refuse (`pass --model/--out (bake)`). Never default to
/// `packages/sail/tokens`.
pub fn tokens(
    model: Option<PathBuf>,
    out: Option<PathBuf>,
    project: Option<PathBuf>,
) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    match (model, out) {
        (Some(model), Some(out)) => tokens_bake(&cwd, model, out, project),
        (None, None) => tokens_project(&cwd, project),
        _ => bail!("sail tokens: pass --model/--out (bake)"),
    }
}

fn tokens_bake(cwd: &Path, model: PathBuf, out: PathBuf, project: Option<PathBuf>) -> Result<()> {
    let root = deck_bundler::paths::topmost_cargo_root(cwd).unwrap_or_else(|| cwd.to_path_buf());
    let model_dir = if model.is_absolute() {
        model
    } else {
        cwd.join(model)
    };
    let out_dir = if out.is_absolute() {
        out
    } else {
        cwd.join(out)
    };

    let mut args = serde_json::json!({
        "workspace": root.display().to_string(),
        "model": model_dir.display().to_string(),
        "out": out_dir.display().to_string(),
    });
    let ctx_project = if let Some(proj) = project {
        let project_dir = if proj.is_absolute() {
            proj
        } else {
            cwd.join(proj)
        };
        args["project"] = serde_json::json!(project_dir.display().to_string());
        project_dir
    } else {
        root.clone()
    };

    let ctx = sail_ops::Ctx::new(&root, &ctx_project, cwd);
    let report = sail_ops::call_rpc("SailTheme", "Generate", &args, &ctx)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    println!(
        "sail tokens: {} tokens → {}",
        report["token_count"],
        report["tokens_css"].as_str().unwrap_or("?")
    );
    println!(
        "  themes: {} → {}",
        report["theme_count"],
        report["themes_css"].as_str().unwrap_or("?")
    );
    println!("  unions: {}", report["unions_ts"].as_str().unwrap_or("?"));
    Ok(())
}

fn tokens_project(cwd: &Path, project: Option<PathBuf>) -> Result<()> {
    let project_dir = if let Some(proj) = project {
        let project_dir = if proj.is_absolute() {
            proj
        } else {
            cwd.join(proj)
        };
        discover_app(&project_dir)?.root().to_path_buf()
    } else if let Some(p) = sail_lint::Project::discover(cwd) {
        p.root().to_path_buf()
    } else {
        bail!("sail tokens: pass --model/--out (bake)");
    };

    let workspace = deck_bundler::paths::topmost_cargo_root(cwd);
    let report = sail_compiler::tokens_generate::generate_app_delivery_opt(
        workspace.as_deref(),
        &project_dir,
    )
    .map_err(|e| anyhow::anyhow!("sail tokens: project themes: {e:#}"))?;
    println!(
        "sail tokens: {} tokens, {} themes → {}",
        report.token_count,
        report.theme_count,
        report.themes_css.display()
    );
    Ok(())
}

/// Walk up from `from` to the Sail app (`sail.config`). **Not Cargo.**
pub fn discover_app(from: &Path) -> Result<sail_lint::Project> {
    sail_lint::Project::discover(from).ok_or_else(|| {
        anyhow::anyhow!(
            "not inside a Sail project (no sail.config.jsonc / sail.config.json near {})",
            from.display()
        )
    })
}

/// `sail op Service/Method --json '{...}'` — in-process sail-ops RPC.
///
/// Special target `gen` regenerates the TS client + schema snapshots from
/// typed contracts (`sail op gen`).
pub fn op(target: &str, json_body: &str) -> Result<()> {
    if target.trim() == "gen" {
        return op_gen();
    }
    let (service, method) = parse_op_target(target)?;
    let args: serde_json::Value = serde_json::from_str(json_body)
        .with_context(|| format!("parse --json body for {target}"))?;
    let ctx = sail_ops::Ctx::from_env();
    let result =
        sail_ops::call_rpc(&service, &method, &args, &ctx).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
    );
    Ok(())
}

/// `sail op gen` — emit packages/sail/studio-client + schema snapshots.
pub fn op_gen() -> Result<()> {
    let written = sail_ops::op_gen(None).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("sail op gen: wrote {} file(s)", written.len());
    for p in &written {
        println!("  {}", p.display());
    }
    Ok(())
}

fn parse_op_target(target: &str) -> Result<(String, String)> {
    let t = target.trim().trim_start_matches('/');
    // Accept `Service/Method` or full Connect path `sail.ops.v1.Service/Method`.
    if let Some((s, m)) = sail_ops::parse_connect_path(t) {
        return Ok((s, m));
    }
    let (service, method) = t
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("expected Service/Method, got `{target}`"))?;
    if service.is_empty() || method.is_empty() {
        bail!("expected Service/Method, got `{target}`");
    }
    Ok((service.to_string(), method.to_string()))
}

/// `sail inspect <topic>` — read-only IR projections (TOOLING-1 WI5).
///
/// Same handlers as MCP `sail_inspect` / SailInspect/Inspect.
pub fn inspect(
    topic: String,
    path: String,
    viewport: Option<(u32, u32)>,
    token: Option<String>,
    theme_id: Option<String>,
    node_id: Option<String>,
) -> Result<()> {
    let ctx = sail_ops::default_ctx();
    let mut args = serde_json::json!({ "topic": topic });
    let obj = args.as_object_mut().expect("object");
    let path = path.trim();
    if !path.is_empty() {
        // Prefer workspace-relative path: strip cwd prefix when absolute under workspace.
        let rel = {
            let p = PathBuf::from(path);
            if p.is_absolute() {
                p.strip_prefix(&ctx.workspace_root)
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| path.to_string())
            } else {
                path.to_string()
            }
        };
        obj.insert("path".into(), serde_json::json!(rel));
    }
    if let Some((w, h)) = viewport {
        obj.insert(
            "viewport".into(),
            serde_json::json!({ "width": w, "height": h }),
        );
    }
    if let Some(t) = token {
        obj.insert("token".into(), serde_json::json!(t));
    }
    if let Some(t) = theme_id {
        obj.insert("themeId".into(), serde_json::json!(t));
    }
    if let Some(n) = node_id {
        obj.insert("nodeId".into(), serde_json::json!(n));
    }
    let out = sail_ops::call_rpc("SailInspect", "Inspect", &args, &ctx)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

/// `sail fmt` — pretty-print `.sail` files (CST template + oxfmt TS regions).
///
/// `--check` fails (exit via bail) when any file is not already canonical.
pub fn fmt(input: PathBuf, check: bool) -> Result<()> {
    let path = if input.is_absolute() {
        input
    } else {
        std::env::current_dir()?.join(input)
    };
    let mut files = Vec::new();
    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) != Some("sail") {
            bail!("not a .sail file: {}", path.display());
        }
        files.push(path);
    } else if path.is_dir() {
        collect_sail_files(&path, &mut files)?;
        if files.is_empty() {
            bail!("no .sail files under {}", path.display());
        }
        files.sort();
    } else {
        bail!("not a file or directory: {}", path.display());
    }

    let mut oxfmt_warned = false;
    let mut changed = 0usize;
    let mut failed = 0usize;
    for f in &files {
        match fmt_one(f, check, &mut oxfmt_warned) {
            Ok(true) => changed += 1,
            Ok(false) => {}
            Err(e) => {
                eprintln!("sail fmt: {}: {e:#}", f.display());
                failed += 1;
            }
        }
    }
    if failed > 0 {
        bail!("{failed}/{} file(s) failed to format", files.len());
    }
    if check {
        if changed > 0 {
            bail!("{changed}/{} file(s) need formatting", files.len());
        }
        println!("sail fmt --check: {} file(s) ok", files.len());
    } else {
        println!("sail fmt: {} file(s), {} rewritten", files.len(), changed);
    }
    Ok(())
}

/// Format one file. Returns `Ok(true)` if content would change / did change.
fn fmt_one(path: &Path, check: bool, oxfmt_warned: &mut bool) -> Result<bool> {
    let original =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let result = sail_compiler::format_source(&original).map_err(|e| anyhow::anyhow!("{e}"))?;
    if result.oxfmt_missing && !*oxfmt_warned {
        eprintln!(
            "sail fmt: warning: oxfmt not found (node_modules/.bin or PATH) — \
             scope/controller regions indent-only; install oxfmt for TS format"
        );
        *oxfmt_warned = true;
    }
    if result.source == original {
        return Ok(false);
    }
    if check {
        println!("would reformat {}", path.display());
        return Ok(true);
    }
    std::fs::write(path, &result.source).with_context(|| format!("write {}", path.display()))?;
    println!("formatted {}", path.display());
    Ok(true)
}

/// `sail check` — compile-time diagnostics for one `.sail` file or directory.
///
/// Directories also lint plain app TypeScript under `apps/*/src/**/*.ts`
/// (floor + ambient DOM) and author action modules under `src/actions/**`
/// (ACTIONS-1 regime). That closes the escape hatch where a side module
/// under studio carried `cssText` / `createElement` because only `.sail`
/// templates were walked.
///
/// EXT-1 E8: when checking a directory (or a file inside a Sail project),
/// warn if any path under `dist/` is git-tracked — deploy output must never
/// be committed. Warning only; does not fail the check.
pub fn check(input: PathBuf, fix: String) -> Result<()> {
    let path = if input.is_absolute() {
        input
    } else {
        std::env::current_dir()?.join(input)
    };
    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("ts")
            && (sail_lint::is_app_source_ts(&path) || sail_lint::is_action_source_path(&path))
        {
            check_app_ts(&path)?;
        } else {
            check_one(&path, &fix)?;
        }
        if let Some(project) = find_project_root_for_hygiene(&path) {
            warn_if_dist_tracked(&project);
        }
    } else if path.is_dir() {
        let mut files = Vec::new();
        collect_sail_files(&path, &mut files)?;
        let mut app_ts = Vec::new();
        collect_app_source_ts(&path, &mut app_ts)?;
        if files.is_empty() && app_ts.is_empty() {
            bail!("no .sail or app source .ts files under {}", path.display());
        }
        let mut failed = 0usize;
        let mut fixed = 0usize;
        let total = files.len() + app_ts.len();
        for f in &files {
            match check_one(f, &fix) {
                Ok(n) => fixed += n,
                Err(e) => {
                    eprintln!("sail check: {}: {e:#}", f.display());
                    failed += 1;
                }
            }
        }
        for f in &app_ts {
            if let Err(e) = check_app_ts(f) {
                eprintln!("sail check: {}: {e:#}", f.display());
                failed += 1;
            }
        }
        // Repo hygiene (EXT-1 E8): warn if `dist/` is git-tracked.
        warn_if_dist_tracked(&path);
        if failed > 0 {
            bail!("{failed}/{total} file(s) failed check");
        }
        println!(
            "sail check: {} file(s) ok, {} fix(es) applied",
            total, fixed
        );
    } else {
        bail!("not a file or directory: {}", path.display());
    }
    Ok(())
}

fn find_project_root_for_hygiene(start: &Path) -> Option<PathBuf> {
    sail_lint::Project::discover(start).map(|p| p.root().to_path_buf())
}

/// If `project` is inside a git work tree and any path under `dist/` is
/// tracked, print a warning to stderr. Never fails; missing git is a no-op.
fn warn_if_dist_tracked(project: &Path) {
    let tracked = git_tracked_under_dist(project);
    if tracked.is_empty() {
        return;
    }
    let sample = tracked
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let more = if tracked.len() > 3 {
        format!(" (+{} more)", tracked.len() - 3)
    } else {
        String::new()
    };
    eprintln!(
        "sail check: warning: dist/ is tracked by git ({sample}{more}) — \
         deploy output must not be committed. Add `dist/` to .gitignore \
         and `git rm -r --cached dist`."
    );
}

/// Return paths under `<project>/dist/` that git currently tracks.
/// Empty when not a git work tree, git is missing, or nothing is tracked.
///
/// Pathspecs are repo-root relative so monorepo sub-apps (`apps/foo/dist`)
/// and external app roots (`dist` at the work-tree root) both work.
fn git_tracked_under_dist(project: &Path) -> Vec<String> {
    let top = match std::process::Command::new("git")
        .args(["-C"])
        .arg(project)
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                return Vec::new();
            }
            PathBuf::from(s)
        }
        _ => return Vec::new(),
    };

    // Repo-relative prefix of the project ("" when project is the work-tree root).
    let project_abs = project
        .canonicalize()
        .unwrap_or_else(|_| project.to_path_buf());
    let top_abs = top.canonicalize().unwrap_or(top);
    let rel = project_abs
        .strip_prefix(&top_abs)
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let dist_prefix = if rel.as_os_str().is_empty() {
        PathBuf::from("dist")
    } else {
        rel.join("dist")
    };
    let dist_str = dist_prefix.to_string_lossy().replace('\\', "/");
    let glob = format!("{dist_str}/**");

    let output = std::process::Command::new("git")
        .args(["-C"])
        .arg(&top_abs)
        .args(["ls-files", "--", dist_str.as_str(), glob.as_str()])
        .output();
    let Ok(out) = output else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

/// Lint a plain `apps/*/src/**/*.ts` module via sail-lint dispatch.
fn check_app_ts(path: &Path) -> Result<()> {
    let findings = sail_lint::lint_path(path);
    let errors: Vec<_> = findings
        .iter()
        .filter(|f| matches!(f.severity, sail_lint::Severity::Error))
        .collect();
    if errors.is_empty() {
        println!("sail check: {} ok", path.display());
        return Ok(());
    }
    for f in &errors {
        eprintln!(
            "{}:{}:{}: {} ({})",
            path.display(),
            f.line,
            f.col,
            f.message,
            f.rule
        );
    }
    bail!("{} error(s)", errors.len())
}

fn workspace_rel(path: &Path, ctx: &sail_ops::Ctx) -> String {
    if let Ok(r) = path.strip_prefix(&ctx.workspace_root) {
        return r.to_string_lossy().replace('\\', "/");
    }
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let root = ctx
        .workspace_root
        .canonicalize()
        .unwrap_or_else(|_| ctx.workspace_root.clone());
    match canon.strip_prefix(&root) {
        Ok(r) => r.to_string_lossy().replace('\\', "/"),
        Err(_) => path.display().to_string(),
    }
}

fn check_one(path: &Path, fix: &str) -> Result<usize> {
    let _source =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;

    // Use the SailAuthoring/Check handler so fixes flow through the same
    // validate-before-write op path as the MCP/Connect surface.
    // Authoring refuses absolute paths — pass a workspace-relative one.
    let ctx = sail_ops::default_ctx();
    let args = serde_json::json!({
        "path": workspace_rel(path, &ctx),
        "fix": fix,
        "actor": "sail-check-cli",
    });
    let result = sail_ops::authoring::check(&args, &ctx)?;

    let diagnostics = result
        .get("diagnostics")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .clone();
    let auto_fixed = result
        .get("autoFixed")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .clone();
    // `suggest` surfaces plans under `fixes`; `apply` under `autoFixed`.
    let fixes = result
        .get("fixes")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .clone();
    let plans = if !auto_fixed.is_empty() {
        auto_fixed
    } else {
        fixes
    };

    // Frozen [sail:fix] contract — always print rewrites before bailing on
    // residual Errors so CLI suggest matches JSON/MCP (sx typo, off-scale, …).
    let mut fixed = 0usize;
    for f in &plans {
        fixed += 1;
        let rule = f.get("rule").and_then(|r| r.as_str()).unwrap_or("fix");
        let msg = f.get("message").and_then(|m| m.as_str()).unwrap_or("");
        let line = f.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
        let col = f.get("col").and_then(|v| v.as_u64()).unwrap_or(0);
        println!(
            "[sail:fix] {}:{}:{}: \"{msg}\" ({rule})",
            path.display(),
            line,
            col
        );
        if let Some(patch) = f.get("textPatch") {
            let repl = patch["replacement"].as_str().unwrap_or("");
            if !repl.is_empty() {
                println!("    diff: + {repl}");
            }
        }
        if let Some(ops) = f.get("ops").and_then(|o| o.as_array()) {
            if !ops.is_empty() {
                println!(
                    "    rewrite: {}",
                    serde_json::to_string(ops).unwrap_or_default()
                );
            }
        }
    }

    if fixed > 0 {
        let verb = if fix == "apply" { "fixed" } else { "suggested" };
        println!("sail check: {} {verb} {fixed} repair(s)", path.display());
    }

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.get("severity").and_then(|s| s.as_str()) == Some("error"))
        .collect();
    if !errors.is_empty() {
        for d in &errors {
            eprintln!(
                "{}:{}:{}: {}",
                path.display(),
                d["line"],
                d["col"],
                d["message"]
            );
        }
        bail!("{} error(s)", errors.len());
    }

    if fixed == 0 {
        println!("sail check: {} ok", path.display());
    }
    Ok(fixed)
}

/// Walk `dir` for plain app TypeScript that `is_app_source_ts` admits, plus
/// author action modules under `src/actions/**` (`is_action_source_path`).
///
/// When `dir` is `apps/studio` (or any apps/* tree), modules under `src/` are
/// the floor surface. When `dir` is a monorepo root, every `apps/*/src/**/*.ts`
/// under it is included. Controllers are excluded by both predicates.
fn collect_app_source_ts(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "node_modules" || name == "target" || name == "dist" || name.starts_with('.')
            {
                continue;
            }
            collect_app_source_ts(&p, out)?;
        } else if sail_lint::is_app_source_ts(&p) || sail_lint::is_action_source_path(&p) {
            out.push(p);
        }
    }
    Ok(())
}

fn collect_sail_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "node_modules" || name == "target" || name.starts_with('.') {
                continue;
            }
            collect_sail_files(&p, out)?;
        } else if p.extension().and_then(|e| e.to_str()) == Some("sail") {
            out.push(p);
        }
    }
    Ok(())
}

/// `sail compile` — in-process compile via the sail-compiler library (C7).
///
/// Same public `compile()` API as the `sail-compiler` bin (no cargo shell-out).
/// Resource caps of the dedicated binary are not applied here.
pub fn compile(
    input: PathBuf,
    out: Option<PathBuf>,
    clean: bool,
    component_name: Option<String>,
) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let input = if input.is_absolute() {
        input
    } else {
        cwd.join(input)
    };

    let mut targets = Vec::new();
    if input.is_dir() {
        collect_sail_files(&input, &mut targets)?;
    } else if input.extension().and_then(|e| e.to_str()) == Some("sail") {
        targets.push(input.clone());
    } else {
        bail!("not a .sail file or directory: {}", input.display());
    }
    if targets.is_empty() {
        bail!("no .sail files found under {}", input.display());
    }

    if clean {
        if let Some(out_dir) = out.as_deref() {
            if out_dir.exists() {
                std::fs::remove_dir_all(out_dir)
                    .with_context(|| format!("clean {}", out_dir.display()))?;
            }
        } else {
            let mut cleaned = std::collections::HashSet::new();
            for path in &targets {
                let mirror = resolve_out_dir(path);
                if cleaned.insert(mirror.clone()) && mirror.exists() {
                    std::fs::remove_dir_all(&mirror)
                        .with_context(|| format!("clean {}", mirror.display()))?;
                }
            }
        }
    }

    if let Some(out_dir) = out.as_deref() {
        std::fs::create_dir_all(out_dir)
            .with_context(|| format!("create out dir {}", out_dir.display()))?;
    }

    let count = targets.len();
    for path in &targets {
        // Component convention guard (#176): refuse bare component paths without a tag.
        if component_name.is_none() {
            if let Some(tag) = looks_like_component_path(path) {
                bail!(
                    "refusing to compile {} without a component tag\n\
                     Pass --component-name {tag} (path matches `<tag>/<tag>.sail`).",
                    path.display()
                );
            }
        }
        compile_one_file(path, out.as_deref(), component_name.clone())?;
    }
    println!("sail compile: {count} file(s)");
    Ok(())
}

fn looks_like_component_path(path: &Path) -> Option<String> {
    let stem = path.file_stem().and_then(|s| s.to_str())?;
    let parent = path.parent()?.file_name().and_then(|s| s.to_str())?;
    if stem == parent {
        Some(stem.to_string())
    } else {
        None
    }
}

fn compile_one_file(
    path: &Path,
    out_override: Option<&Path>,
    component_name: Option<String>,
) -> Result<()> {
    let orig_source =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let inline_controller =
        sail_compiler::parse::preprocess::extract_controller_block(&orig_source).1;
    let mirror_mode = out_override.is_none();
    let out_dir: PathBuf = match out_override {
        Some(p) => p.to_path_buf(),
        None => resolve_out_dir(path),
    };
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;

    let setup_url = inline_controller.as_ref().map(|_| {
        let file = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("component.sail");
        if mirror_mode {
            format!("{}.controller.ts", path.display())
        } else {
            format!("./{file}.controller.ts")
        }
    });

    let component_meta = component_name.map(|name| ComponentMeta { name });
    let opts = CompileOpts {
        route_params: None,
        setup_url,
        mode: if component_meta.is_some() {
            CompileMode::Component
        } else {
            CompileMode::Auto
        },
        source_path: Some(path.to_path_buf()),
        component_meta,
        emit_dts: true,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Policy::overrides_for(path),
    };
    let result = compile_source(&orig_source, opts).map_err(|e| {
        let rendered = e.render(&orig_source);
        anyhow::anyhow!("{rendered}")
    })?;

    // Fatal lint gate (same honesty as sail-compiler build).
    let fatal: Vec<_> = check_all(&orig_source, Some(path), &[], None)
        .into_iter()
        .filter(|d| matches!(d.severity, DiagSeverity::Error))
        .collect();
    if !fatal.is_empty() {
        for d in &fatal {
            eprintln!("{}:{}:{}: {}", path.display(), d.line, d.col, d.message);
        }
        bail!("{} error(s) from check_all", fatal.len());
    }

    let stem = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow::anyhow!("bad path"))?;
    let stem_no_ext = stem.strip_suffix(".sail").unwrap_or(stem);
    let js_name = if mirror_mode {
        format!("{stem_no_ext}.compiled.js")
    } else {
        format!("{stem_no_ext}.js")
    };
    let js_path = out_dir.join(&js_name);
    std::fs::write(&js_path, &result.js).with_context(|| format!("write {}", js_path.display()))?;
    if let Some(cem) = result.cem {
        let cem_path = out_dir.join(format!("{stem_no_ext}.custom-elements.json"));
        std::fs::write(&cem_path, serde_json::to_string_pretty(&cem)?)
            .with_context(|| format!("write {}", cem_path.display()))?;
    }
    if let Some(schema) = result.schema {
        let schema_path = out_dir.join(format!("{stem_no_ext}.schema.json"));
        std::fs::write(&schema_path, serde_json::to_string_pretty(&schema)?)
            .with_context(|| format!("write {}", schema_path.display()))?;
    }
    if let Some(meta) = result.meta {
        let meta_path = out_dir.join(format!("{stem_no_ext}.sail.meta.json"));
        std::fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)
            .with_context(|| format!("write {}", meta_path.display()))?;
    }
    if let Some(dts) = result.dts {
        let dts_dir = path.parent().unwrap_or(out_dir.as_path());
        let dts_path = dts_dir.join(format!("{stem}.d.ts"));
        std::fs::write(&dts_path, dts).with_context(|| format!("write {}", dts_path.display()))?;
    }
    Ok(())
}

/// `sail lsp` — in-process language server (same protocol as `sail-lsp`).
pub fn lsp() -> Result<()> {
    sail_lsp::run().context("sail lsp")
}

/// `sail mcp` — in-process MCP server (same protocol as `sail-mcp`).
///
/// Stdio by default; `--http [ADDR]` for Streamable HTTP (MCP 2026-07-28).
pub fn mcp() {
    sail_mcp::run();
}

/// `sail mcp install` — write MCP server entries into agent host configs.
pub fn mcp_install(project: PathBuf, bin: Option<PathBuf>) -> Result<()> {
    let report = match bin {
        Some(p) => sail_mcp::install_with_bin(&project, Some(&p)),
        None => sail_mcp::install(&project),
    }
    .map_err(|e| anyhow::anyhow!(e))?;
    print!("{}", sail_mcp::format_report(&report));
    Ok(())
}

/// `sail layout` — static box-tree diagnostics.
///
/// Non-zero exit when any `Severity::Error` is present (CI-friendly).
pub fn layout(input: PathBuf, viewport: (f32, f32), json: bool) -> Result<()> {
    let source = if input.as_os_str() == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("read stdin")?;
        buf
    } else {
        std::fs::read_to_string(&input).with_context(|| format!("read {}", input.display()))?
    };

    let result = sail_layout::check_preprocessed(&source, viewport).context("layout check")?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result.diagnostics).context("serialize diagnostics")?
        );
        let errors = result
            .diagnostics
            .iter()
            .filter(|d| matches!(d.severity, sail_layout::Severity::Error))
            .count();
        if errors > 0 {
            bail!("{errors} layout error(s)");
        }
        return Ok(());
    }

    if result.diagnostics.is_empty() {
        eprintln!(
            "ok — no layout diagnostics at viewport {}x{}",
            viewport.0 as u32, viewport.1 as u32
        );
        return Ok(());
    }

    let mut errors = 0usize;
    for d in &result.diagnostics {
        let sev = match d.severity {
            sail_layout::Severity::Error => {
                errors += 1;
                "error"
            }
            sail_layout::Severity::Warning => "warning",
        };
        eprintln!(
            "{sev}: [{}] at byte {}..{} (tag={}, class=\"{}\") — {}",
            d.kind, d.span.0, d.span.1, d.element_tag, d.element_class, d.message
        );
    }
    eprintln!(
        "\n{}/{} layout diagnostic(s) at viewport {}x{}",
        result.diagnostics.len(),
        result.diagnostics.len(),
        viewport.0 as u32,
        viewport.1 as u32
    );
    if errors > 0 {
        bail!("{errors} layout error(s)");
    }
    Ok(())
}

/// Parse `WxH` viewport strings (`1440x900`).
pub fn parse_viewport(s: &str) -> Result<(f32, f32), String> {
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| "viewport must be `WxH`".to_string())?;
    let w: f32 = w.parse().map_err(|_| format!("bad width {w:?}"))?;
    let h: f32 = h.parse().map_err(|_| format!("bad height {h:?}"))?;
    Ok((w, h))
}

/// `sail render` — static themed PNG from a `.sail` file (no browser).
pub fn render(
    input: PathBuf,
    theme: String,
    mode: String,
    viewport: (u32, u32),
    out: Option<PathBuf>,
    json: bool,
    fonts_dir: Option<PathBuf>,
) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let path = if input.is_absolute() {
        input
    } else {
        cwd.join(&input)
    };
    if !path.is_file() {
        bail!("not a file: {}", path.display());
    }
    let out_png = out.unwrap_or_else(|| path.with_extension("png"));
    let out_png = if out_png.is_absolute() {
        out_png
    } else {
        cwd.join(out_png)
    };

    let workspace = deck_bundler::paths::topmost_cargo_root(&cwd).unwrap_or_else(|| cwd.clone());
    let rel = path
        .strip_prefix(&workspace)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string());

    let app_fonts_dir = {
        // Prefer project fonts next to the sail file (apps/<app>/fonts).
        let mut cur = path.parent();
        let mut found = None;
        while let Some(dir) = cur {
            let candidate = dir.join("fonts");
            if candidate.is_dir() {
                found = Some(candidate);
                break;
            }
            if dir.join("package.json").is_file() || dir.join("routes").is_dir() {
                break;
            }
            cur = dir.parent();
        }
        found
    };
    let opts = sail_render::RenderOptions {
        viewport,
        fonts_dir,
        app_fonts_dir,
        theme: Some(theme),
        mode: Some(mode),
        workspace: Some(workspace),
        file: Some(rel),
        drop_bg_token: None,
    };
    let result = sail_render::render_file(&path, opts)
        .with_context(|| format!("render {}", path.display()))?;

    if let Some(parent) = out_png.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    sail_render::write_png(&result.image, &out_png)
        .with_context(|| format!("write {}", out_png.display()))?;

    eprintln!(
        "sail render: {} → {} ({}x{}, theme {}:{}, {}ms, {} regions, {} dynamics)",
        path.display(),
        out_png.display(),
        result.viewport.0,
        result.viewport.1,
        result.theme.data_theme(),
        result.theme.data_mode(),
        result.ms,
        result.regions.len(),
        result.dynamics.len()
    );

    if json {
        let sidecar = out_png.with_extension("json");
        let body = serde_json::json!({
            "viewport": { "width": result.viewport.0, "height": result.viewport.1 },
            "theme": format!("{}:{}", result.theme.data_theme(), result.theme.data_mode()),
            "ms": result.ms,
            "regions": result.regions,
            "dynamics": result.dynamics,
            "png": out_png.display().to_string(),
        });
        std::fs::write(&sidecar, serde_json::to_string_pretty(&body)?)
            .with_context(|| format!("write {}", sidecar.display()))?;
        eprintln!("  sidecar → {}", sidecar.display());
    }
    Ok(())
}
