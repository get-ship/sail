//! `sui-compiler` CLI. Compiles a single `.sui` file or a directory of
//! them, writing the emitted JS (and CEM/schema/meta/dts/ssr.html sidecars)
//! into a project-level mirror tree at `<repo-root>/.sui-out/<source-
//! relative-path>/`. Pass `--out <dir>` to override and emit every file to
//! a single directory (legacy behaviour). Errors render with file path,
//! line, and column.
//
// FILE_CAP_OVERRIDE: single cohesive CLI driver — arg parsing (clap), the
// component-path guardrail, mirror-tree resolution, per-file emit, and the
// fatal-diagnostics exit-code gate are one binary's responsibility. Splitting
// would scatter the one entry point across modules for no clarity gain.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use sui_compiler::out_layout::resolve_out_dir;
use sui_compiler::{check_all, compile, CompileMode, ComponentMeta, CompileOpts, DiagSeverity, Policy};

/// Hard caps on virtual memory and wall-clock time. A bug in the
/// compiler (or a pathological input) must never be able to OOM the
/// host: a single uncaught `Vec::with_capacity` once allocated 19+GB
/// before the kernel killed the process and took down the surrounding
/// VM. These bounds guarantee a clean death long before that.
///
/// Generous for any real input: a single `.sui` file compiles in
/// milliseconds with <100MB RSS. A directory of hundreds compiles in
/// seconds. Hitting either cap is a bug to file, not a knob to tune.
/// To override (e.g. unusually large project, deliberate stress run),
/// set `SUI_COMPILER_NO_LIMITS=1`.
const MEM_CAP_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GB virtual memory
const WALL_CAP_SECS: u64 = 60;

fn install_resource_bounds() {
    if std::env::var_os("SUI_COMPILER_NO_LIMITS").is_some() {
        return;
    }
    // SAFETY: setrlimit with valid resource id and rlimit struct is sound.
    unsafe {
        let lim = libc::rlimit { rlim_cur: MEM_CAP_BYTES as libc::rlim_t, rlim_max: MEM_CAP_BYTES as libc::rlim_t };
        libc::setrlimit(libc::RLIMIT_AS, &lim);
    }
    thread::Builder::new()
        .name("sui-compiler-watchdog".into())
        .spawn(|| {
            thread::sleep(Duration::from_secs(WALL_CAP_SECS));
            eprintln!("sui-compiler: aborted after {WALL_CAP_SECS}s wall-clock cap");
            process::abort();
        })
        .expect("spawn watchdog thread");
}

#[derive(Parser)]
#[command(name = "sui-compiler", about = "AOT compiler for .sui files")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile one file or every `.sui` under a directory.
    Build {
        /// Path to a `.sui` file or a directory.
        input: PathBuf,
        /// Output directory. When omitted, each file's outputs land in the
        /// project mirror tree at `<repo-root>/.sui-out/<source-rel>/`
        /// and the JS module is named `<stem>.compiled.js`. When supplied,
        /// every file is emitted into this single directory and the JS
        /// module is named `<stem>.js` (legacy behaviour).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Remove each source file's mirror directory (or `--out` directory)
        /// before emitting. Idempotent rebuilds.
        #[arg(long)]
        clean: bool,
        /// Override mode (otherwise auto-detected from --component-name).
        #[arg(long)]
        mode: Option<ModeArg>,
        /// Suppress emitting the `.sui.d.ts` declaration file.
        #[arg(long)]
        no_dts: bool,
        /// Emit `import "@sui-tag/<tag>";` per used custom-element tag so the
        /// bundler can tree-shake exactly the components a view uses (the
        /// per-view resolver). The bundler resolves `@sui-tag/<tag>` to the
        /// component source; default off leaves component loading to the host.
        #[arg(long)]
        emit_component_imports: bool,
        /// Path to a CSS file whose contents are prepended as a `<style>`
        /// block. Only valid for component sources (requires --component-name).
        /// Requires `input` to be a single file (not a directory).
        #[arg(long)]
        styles: Option<PathBuf>,
        /// Custom element tag name for this component file. Setting this flag
        /// implies component mode — no `--mode component` needed. When omitted
        /// for a file in component mode, defaults to the filename stem.
        #[arg(long)]
        component_name: Option<String>,
    },
    /// Synthesize a single `.sui` into a manifest-driven, tsc-checkable
    /// pair — a **synthetic controller** (`<stem>.synthetic.controller.ts`)
    /// and a **render** (`<stem>.render.ts`). Each name in `--manifest`
    /// becomes a real ESM import; the compiler stays ignorant of it. A
    /// sibling `controller.ts` (if present) is composed, not rewritten.
    /// Additive: the ambient/`BUILTINS` path and render-eval are untouched.
    Synthesize {
        /// Path to a single `.sui` file.
        input: PathBuf,
        /// Path to the app manifest (JSON / JSONC-lite) — a name → import map.
        #[arg(long)]
        manifest: PathBuf,
        /// Directory to write `<stem>.synthetic.controller.ts` +
        /// `<stem>.render.ts` into.
        #[arg(long)]
        out_dir: PathBuf,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ModeArg {
    Component,
    Auto,
}

impl From<ModeArg> for CompileMode {
    fn from(m: ModeArg) -> Self {
        match m {
            ModeArg::Component => CompileMode::Component,
            ModeArg::Auto => CompileMode::Auto,
        }
    }
}

fn main() -> Result<()> {
    install_resource_bounds();
    let cli = Cli::parse();
    match cli.cmd {
        Command::Build {
            input, out, clean, mode, no_dts, styles, component_name,
            emit_component_imports,
        } => build(
            input, out, clean, mode, no_dts, styles, component_name,
            emit_component_imports,
        ),
        Command::Synthesize { input, manifest, out_dir } => {
            synthesize(input, manifest, out_dir)
        }
    }
}

/// Synthesize one `.sui` into a synthetic controller + render pair.
fn synthesize(
    input: PathBuf,
    manifest_path: PathBuf,
    out_dir: PathBuf,
) -> Result<()> {
    use sui_compiler::emit::synthetic::{emit_synthetic, AppManifest};
    use sui_compiler::parse;

    let source = fs::read_to_string(&input)
        .with_context(|| format!("read {}", input.display()))?;
    let manifest_src = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    let program = parse::parse(&source).map_err(|e| anyhow!("{}", e.render(&source)))?;
    let manifest =
        AppManifest::parse(&manifest_src).map_err(|e| anyhow!("{}", e.render(&manifest_src)))?;

    let file = input.file_name().and_then(|f| f.to_str()).unwrap_or("view.sui");
    let raw_stem = file.strip_suffix(".sui").unwrap_or(file);
    // Every `.sui` synthesizes to a component; the retired `page.sui` view
    // path is gone, so the mode is always `Component`.
    let mode = CompileMode::Component;

    let stem = {
        let s = raw_stem.replace(['/', '\\'], "_");
        if s.is_empty() { "view".to_string() } else { s }
    };
    // `page.sui` pairs with a sibling `controller.ts`; compose it when present.
    let authored = sibling_controller(&input, &out_dir, raw_stem);
    let controller_name = format!("{stem}.synthetic.controller.ts");
    let render_name = format!("{stem}.render.ts");
    let client_name = format!("{stem}.client.ts");

    let out = emit_synthetic(
        &program,
        &manifest,
        authored.as_ref(),
        &format!("./{controller_name}"),
        mode,
        None,
    )
    .map_err(|e| anyhow!("{}", e.render(&source)))?;

    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;
    let controller_path = out_dir.join(&controller_name);
    let render_path = out_dir.join(&render_name);
    let client_path = out_dir.join(&client_name);
    fs::write(&controller_path, &out.controller_ts)
        .with_context(|| format!("write {}", controller_path.display()))?;
    fs::write(&render_path, &out.render_ts)
        .with_context(|| format!("write {}", render_path.display()))?;
    fs::write(&client_path, &out.render_client_ts)
        .with_context(|| format!("write {}", client_path.display()))?;
    println!(
        "synthesized {} ({:?}) → {}, {}, {} (clients: {}{})",
        input.display(),
        mode,
        controller_path.display(),
        render_path.display(),
        client_path.display(),
        if out.used_clients.is_empty() { "none".to_string() } else { out.used_clients.join(", ") },
        if authored.is_some() { "; composed authored controller.ts" } else { "" },
    );
    Ok(())
}

/// A sibling `controller.ts` (for `page.sui`) or `<stem>.controller.ts`,
/// composed by the synthetic controller via a relative import from `out_dir`.
fn sibling_controller(
    input: &Path,
    out_dir: &Path,
    stem: &str,
) -> Option<sui_compiler::emit::synthetic::AuthoredController> {
    use sui_compiler::emit::synthetic::AuthoredController;
    let parent = input.parent()?;
    let candidate = if stem == "page" {
        parent.join("controller.ts")
    } else {
        parent.join(format!("{stem}.controller.ts"))
    };
    if !candidate.exists() {
        return None;
    }
    Some(AuthoredController { specifier: rel_specifier(&candidate, out_dir) })
}

/// POSIX relative-path specifier from `out_dir` to `target` (both made
/// absolute against CWD), suitable as an ESM import string.
fn rel_specifier(target: &Path, out_dir: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    let abs = |p: &Path| if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };
    let (target, out_dir) = (abs(target), abs(out_dir));
    let from: Vec<_> = out_dir.components().collect();
    let to: Vec<_> = target.components().collect();
    let common = from.iter().zip(to.iter()).take_while(|(a, b)| a == b).count();
    let mut out = String::new();
    for _ in 0..(from.len() - common) {
        out.push_str("../");
    }
    let rest: Vec<String> =
        to[common..].iter().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    out.push_str(&rest.join("/"));
    if !out.starts_with('.') {
        out = format!("./{out}");
    }
    out
}

fn build(
    input: PathBuf,
    out: Option<PathBuf>,
    clean: bool,
    mode: Option<ModeArg>,
    no_dts: bool,
    styles: Option<PathBuf>,
    component_name: Option<String>,
    emit_component_imports: bool,
) -> Result<()> {
    let mut targets = Vec::new();
    if input.is_dir() {
        if styles.is_some() {
            return Err(anyhow!(
                "--styles requires a single .sui input, not a directory"
            ));
        }
        collect_targets(&input, &mut targets)?;
    } else if input
        .file_name()
        .and_then(|f| f.to_str())
        .is_some_and(is_sui_source)
    {
        targets.push(input.clone());
    } else {
        return Err(anyhow!("not a .sui file or directory: {}", input.display()));
    }
    if targets.is_empty() {
        return Err(anyhow!("no .sui files found under {}", input.display()));
    }

    if clean {
        if let Some(out_dir) = out.as_deref() {
            // Single --out dir mode: clean it once before emit.
            if out_dir.exists() {
                fs::remove_dir_all(out_dir)
                    .with_context(|| format!("clean {}", out_dir.display()))?;
            }
        } else {
            // Mirror-tree mode: clean each unique target mirror directory.
            let mut cleaned: HashSet<PathBuf> = HashSet::new();
            for path in &targets {
                let mirror = resolve_out_dir(path);
                if cleaned.insert(mirror.clone()) && mirror.exists() {
                    fs::remove_dir_all(&mirror)
                        .with_context(|| format!("clean {}", mirror.display()))?;
                }
            }
        }
    }

    if let Some(out_dir) = out.as_deref() {
        fs::create_dir_all(out_dir)
            .with_context(|| format!("create out dir {}", out_dir.display()))?;
    }

    let styles_text = match styles.as_deref() {
        Some(p) => Some(
            fs::read_to_string(p)
                .with_context(|| format!("read --styles {}", p.display()))?,
        ),
        None => None,
    };
    // Component mode: tag name comes from --component-name when supplied,
    // otherwise defaults to the filename stem at compile-one time. Shadow
    // is always on for components — no opt-out flag.
    //
    // Guardrail (#176): when an input matches the `<tag>/<tag>.sui` component
    // convention AND no explicit mode/component-name was given, refuse rather
    // than emit component JS with no tag. Without a tag the output never calls
    // `customElements.define` — the bundle succeeds, the browser loads it, and
    // nothing registers. Surface the ambiguity at compile time instead of
    // debugging it at runtime.
    let count = targets.len();
    for path in targets {
        let explicit = mode.is_some() || component_name.is_some();
        if !explicit && let Some(tag) = looks_like_component_path(&path) {
            return Err(anyhow!(
                "refusing to compile {} without a component tag\n\
                 \n\
                 The path matches the `<tag>/<tag>.sui` component convention\n\
                 (tag = `{tag}`). Compiling it without a tag emits component\n\
                 JS that never calls `customElements.define` — the browser\n\
                 loads the bundle but nothing registers.\n\
                 \n\
                 Pass one of these to confirm what you mean:\n  \
                 --component-name {tag}    (register under this exact tag)\n  \
                 --mode component          (register under the filename stem)",
                path.display(),
            ));
        }
        let component_meta = match (mode.clone(), component_name.clone()) {
            (Some(ModeArg::Component), name) => Some(ComponentMeta {
                name: name.unwrap_or_else(|| stem_of(&path)),
                shadow: true,
            }),
            (_, Some(name)) => Some(ComponentMeta { name, shadow: true }),
            _ => None,
        };
        compile_one(&path, out.as_deref(), mode.clone(), no_dts, styles_text.as_deref(), component_meta, emit_component_imports)?;
    }
    println!("compiled {count} file(s)");
    Ok(())
}

fn stem_of(path: &Path) -> String {
    path.file_name()
        .and_then(|f| f.to_str())
        .map(|s| {
            s.strip_suffix(".sui")
                .or_else(|| s.strip_suffix(".html.set"))
                .unwrap_or(s)
                .to_string()
        })
        .unwrap_or_default()
}

fn is_sui_source(name: &str) -> bool {
    name.ends_with(".sui") || name.ends_with(".html.set")
}

/// `<tag>/<tag>.sui` — the convention every SUI component follows. The
/// directory name IS the component contract; if it matches the file stem,
/// this is a component source. Returns the tag on match.
///
/// Pre-existing footgun (#176): running `sui-compiler build` directly on a
/// component path without `--component-name` (or `--mode component`) emits
/// component JS with no tag. The bundle succeeds, the browser loads it,
/// nothing happens — no custom element registers, because no
/// `customElements.define()` call was emitted.
fn looks_like_component_path(path: &Path) -> Option<String> {
    let stem = path.file_stem().and_then(|s| s.to_str())?;
    let parent = path.parent()?.file_name().and_then(|s| s.to_str())?;
    if stem == parent { Some(stem.to_string()) } else { None }
}

fn collect_targets(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_targets(&p, out)?;
        } else if p
            .file_name()
            .and_then(|f| f.to_str())
            .is_some_and(is_sui_source)
        {
            out.push(p);
        }
    }
    Ok(())
}

fn compile_one(
    path: &Path,
    out_override: Option<&Path>,
    mode: Option<ModeArg>,
    no_dts: bool,
    styles: Option<&str>,
    component_meta: Option<ComponentMeta>,
    emit_component_imports: bool,
) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    // SFC: an inline `<controller>` block is extracted and written to the
    // generated `<stem>.controller.ts` module that `paired_setup` + the emit
    // already consume — so authoring is one file, with no emit change. (A
    // paired `.controller.ts` on disk stays valid; the inline block wins.)
    let (source, inline_controller) =
        sui_compiler::parse::preprocess::extract_controller_block(&source);
    if let Some(body) = &inline_controller {
        if let (Some(parent), Some(file_name)) =
            (path.parent(), path.file_name().and_then(|s| s.to_str()))
        {
            let stem = file_name.strip_suffix(".sui").unwrap_or(file_name);
            let ctl_file = if stem == "page" {
                "controller.ts".to_string()
            } else {
                format!("{stem}.controller.ts")
            };
            fs::write(parent.join(&ctl_file), body)
                .with_context(|| format!("write generated controller {ctl_file}"))?;
        }
    }
    let mode = mode.map(|m| m.into()).unwrap_or(CompileMode::Auto);
    // Resolve the emit dir for this file. Without --out, outputs land in
    // the project mirror tree at `<repo-root>/.sui-out/<src-rel>/`;
    // with --out, all files share that directory (legacy single-dir mode).
    let mirror_mode = out_override.is_none();
    let out_dir: PathBuf = match out_override {
        Some(p) => p.to_path_buf(),
        None => resolve_out_dir(path),
    };
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;
    // Paired controller is auto-detected: if `<stem>.controller.ts`
    // exists next to the source, the compiler emits
    // `import setup from "<rel>"` where `<rel>` is the path from the
    // emit dir back to the controller. In mirror mode that's an absolute
    // path back to the source dir; in legacy --out=source-dir mode it's
    // `./<stem>.controller.ts`. Authors who want template-only just
    // don't write the .controller.ts file.
    let setup_url = paired_setup(path, &out_dir, mirror_mode);
    // Resolve the styling policy from the nearest `sui.config.json` walking up
    // from this file. OPT-IN (identical to the deck bundler): a package with no
    // config gets no overrides and compiles exactly as before, so the floor rolls
    // out per-package instead of failing every un-migrated file — critically, the
    // primitives (which legitimately author `<style>` blocks) still compile via
    // the CLI. A package opts in with `{"policy":"floor|stripped|strictest"}`; the
    // eval lanes do exactly that. `floor` bans raw `<style>`/`style=` and makes sx
    // the styling surface; `stripped` also bans raw containers; `strictest` also
    // bans sx.
    let opts = CompileOpts {
        setup_url,
        mode,
        source_path: Some(path.to_path_buf()),
        styles: styles.map(|s| s.to_string()),
        component_meta,
        emit_dts: !no_dts,
        emit_component_imports,
        lint_overrides: Policy::resolve_opt(path).map(|p| p.overrides()).unwrap_or_default(),
    };
    // Optional real-TypeScript validation of the `<component>` variants against
    // the declared props. Env-gated so the fast inner-loop build stays fast; the
    // LSP and CI set the toolchain env and get honest type errors the hand-parse
    // cannot produce (e.g. a variant value outside a prop's union → dead CSS).
    if let (Ok(node), Ok(tsgo_js)) =
        (std::env::var("SUI_TSGO_NODE"), std::env::var("SUI_TSGO_JS"))
    {
        let tdir = std::env::temp_dir().join("sui-tsgo-check");
        let diags = sui_compiler::check::tscheck::check_component_variants(
            &source, &tdir, &node, &tsgo_js,
        )
        .unwrap_or_default();
        if let Some(d) = diags.first() {
            anyhow::bail!(
                "{}: <component> variants type error [{}]: {}",
                path.display(),
                d.code,
                d.message
            );
        }
    }
    let result = compile(&source, opts).map_err(|e| {
        let rendered = e.render(&source);
        anyhow!("{rendered}")
    })?;

    fatal_diagnostics_gate(&source, path)?;

    let stem = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("bad path"))?;
    let stem_no_ext = stem
        .strip_suffix(".html.set")
        .or_else(|| stem.strip_suffix(".sui"))
        .unwrap_or(stem);
    // In mirror mode the emitted JS is named `<stem>.compiled.js` so
    // imports can name it directly without a separate rename step in
    // the build pipeline. With --out we keep the legacy `<stem>.js`
    // for callers that expect that name.
    let js_name = if mirror_mode {
        format!("{stem_no_ext}.compiled.js")
    } else {
        format!("{stem_no_ext}.js")
    };
    let js_path = out_dir.join(&js_name);
    fs::write(&js_path, &result.js)
        .with_context(|| format!("write {}", js_path.display()))?;
    if let Some(cem) = result.cem {
        let cem_path = out_dir.join(format!("{stem_no_ext}.custom-elements.json"));
        let pretty = serde_json::to_string_pretty(&cem)?;
        fs::write(&cem_path, pretty)
            .with_context(|| format!("write {}", cem_path.display()))?;
    }
    if let Some(schema) = result.schema {
        let schema_path = out_dir.join(format!("{stem_no_ext}.schema.json"));
        let pretty = serde_json::to_string_pretty(&schema)?;
        fs::write(&schema_path, pretty)
            .with_context(|| format!("write {}", schema_path.display()))?;
    }
    // `.sui.meta.json` sidecar — binding info for LSP/IDE tooling. The
    // runtime never loads this; it's purely developer-facing fuel.
    if let Some(meta) = result.meta {
        let meta_path = out_dir.join(format!("{stem_no_ext}.sui.meta.json"));
        let pretty = serde_json::to_string_pretty(&meta)?;
        fs::write(&meta_path, pretty)
            .with_context(|| format!("write {}", meta_path.display()))?;
    }
    // `.sui.d.ts` is a TYPE companion to its source, so it is written
    // ADJACENT to the `.sui` (not into the build mirror like the JS/CEM/SSR
    // artifacts). That lets a controller's `import type { Scope } from
    // './<name>.sui'` resolve natively to the sibling `.d.ts` — no `rootDirs`
    // round-trip into `.ship/sui-out` in every consumer's tsconfig (which was
    // fragile and, when mispathed, silently masked type errors as `any`).
    // Gitignored via `*.sui.d.ts`.
    if let Some(dts) = result.dts {
        let dts_dir = path.parent().unwrap_or(out_dir.as_path());
        let dts_path = dts_dir.join(format!("{stem}.d.ts"));
        fs::write(&dts_path, dts)
            .with_context(|| format!("write {}", dts_path.display()))?;
    }
    Ok(())
}

/// Honest exit codes. `compile()` short-circuits on the first diagnostic its
/// `check::run` pass gates (expr-subset, scope-bound, css-token,
/// controller-purity, …) and returns `Err` — that path already exits non-zero.
/// But the full `sui-lint` stream carries Error-severity rules `check::run`
/// does NOT invoke (e.g. `sui-attr-bool-prop`, `sui-prop-value`,
/// `sui-css-literal`, `sui-css-content-text`, `controller-*`). Without this
/// gate those compile to `Ok`, the CLI prints "compiled N file(s)" and returns
/// 0 while the LSP/agent show a hard error — the dishonest-exit-code bug
/// (#255). Re-run the full diagnostic pass and treat any Error-severity finding
/// as fatal so batch OK-checks (`sui-compiler build <dir> && …`) mean what they
/// say.
fn fatal_diagnostics_gate(source: &str, path: &Path) -> Result<()> {
    let fatal: Vec<_> = check_all(source, Some(path), &[])
        .into_iter()
        .filter(|d| d.severity == DiagSeverity::Error)
        .collect();
    if fatal.is_empty() {
        return Ok(());
    }
    // `check_all` diagnostics are already self-describing — lint findings that
    // flow through this path carry their `[rule]` tag inside the message, and
    // structural check-pass errors carry a clear category. Render the message
    // verbatim with just the file/line/col prefix; adding a separate `[rule]`
    // tag here would double the one the message already embeds.
    let rendered = fatal
        .iter()
        .map(|d| format!("{}: line {} col {}: {}", path.display(), d.line, d.col, d.message))
        .collect::<Vec<_>>()
        .join("\n");
    Err(anyhow!("{rendered}"))
}

fn paired_setup(path: &Path, out_dir: &Path, mirror_mode: bool) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let stem = file_name
        .strip_suffix(".html.set")
        .or_else(|| file_name.strip_suffix(".sui"))?;
    let parent = path.parent()?;
    // page.sui pairs with controller.ts (no stem prefix); every other
    // stem pairs with <stem>.controller.ts.
    let filename = if stem == "page" {
        "controller.ts".to_string()
    } else {
        format!("{stem}.controller.ts")
    };
    let candidate = parent.join(&filename);
    if !candidate.exists() {
        return None;
    }
    if mirror_mode {
        // Mirror tree may be far from the source. Walk up from out_dir
        // until we hit the source dir, then reuse that as the import path.
        let rel = pathdiff(&candidate, out_dir);
        Some(rel)
    } else {
        Some(format!("./{filename}"))
    }
}

/// Compute a relative path from `from_dir` to `to_file` using `..`
/// segments. Falls back to the absolute `to_file` path when the two have
/// no common ancestor.
fn pathdiff(to_file: &Path, from_dir: &Path) -> String {
    use std::path::Component;
    let to = match to_file.canonicalize() {
        Ok(p) => p,
        Err(_) => to_file.to_path_buf(),
    };
    let from = match from_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => from_dir.to_path_buf(),
    };
    let to_parts: Vec<Component> = to.components().collect();
    let from_parts: Vec<Component> = from.components().collect();
    let common = to_parts
        .iter()
        .zip(from_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();
    if common == 0 {
        return to.to_string_lossy().into_owned();
    }
    let ups = from_parts.len() - common;
    let mut out = String::new();
    for _ in 0..ups {
        out.push_str("../");
    }
    if ups == 0 {
        out.push_str("./");
    }
    let remainder: PathBuf = to_parts[common..].iter().collect();
    out.push_str(&remainder.to_string_lossy());
    out
}
