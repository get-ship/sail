//! `sail` — the Sail framework CLI (user-facing brand).
//!
//! One installable binary covers create/dev/build/check/types/scaffold theme/tokens/compile/lsp/
//! mcp/layout/doctor. Internals still use deck crates (bundler, axum, core);
//! the product path is **Sail**. Workspace bake (`build-assets`, `emit`) lives
//! on the sibling `deck-bake` bin — not this product surface.

mod editor;
mod peer;
mod scaffold;
mod typecheck;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

use deck_axum::DaemonMode;

/// Product binary name used in error/version prefixes.
fn binary_brand() -> &'static str {
    "sail"
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn parse_render_viewport(s: &str) -> Result<(u32, u32), String> {
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| "viewport must be `WxH`".to_string())?;
    let w: u32 = w.parse().map_err(|_| format!("bad width {w:?}"))?;
    let h: u32 = h.parse().map_err(|_| format!("bad height {h:?}"))?;
    Ok((w, h))
}

#[derive(Parser)]
#[command(
    name = "sail",
    about = "Sail — the UI framework CLI (create, dev, build, check, types, scaffold theme, compile, tokens, fmt, inspect, lsp, mcp, editor, op, layout, render, doctor, version)",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a new Sail app (`package.json`, `sail.config.jsonc`, `routes/`,
    /// `themes/`, `.gitignore` with `dist/` + `.sail/`). External-app product path (EXT-1 E8).
    Create {
        /// App directory to create (e.g. `my-app` → `./my-app`). Basename must
        /// match `/^[a-z][a-z0-9-]*$/`.
        name: PathBuf,
    },

    /// Generate boilerplate (theme). Aliases: `new`, `init`.
    #[command(subcommand, visible_alias = "new", visible_alias = "init")]
    Scaffold(Scaffold),

    /// Build a SAIL SPA project to a static output dir.
    Build {
        /// Path inside a Sail project (walks up to `sail.config.jsonc` / `.json`). Default: cwd.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Output dir (default: <project>/dist/).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Materialize the /_deck/* framework surface into the output (self-contained).
        #[arg(long)]
        bundle_framework: bool,
        /// Skip minification.
        #[arg(long)]
        no_minify: bool,
        /// After the build, type-check the app's `.ts` sources with `tsc --noEmit`.
        #[arg(long)]
        typecheck: bool,
        /// Asset-URL base. `--base assets` for static-host-servable builds.
        #[arg(long)]
        base: Option<String>,
    },

    /// Run a standalone dev server with HMR (and Sail Studio bridge overlay).
    Dev {
        /// Path inside a Sail project (walks up to `sail.config.jsonc` / `.json`). Default: cwd.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Address to bind (host:port). Default `127.0.0.1:7878`.
        #[arg(long, default_value = "127.0.0.1:7878")]
        addr: String,
    },

    /// Compile `.sail` sources (wraps sail-compiler build).
    Compile {
        /// Path to a `.sail` file or directory.
        input: PathBuf,
        /// Output directory override.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Clean mirror dir before emit.
        #[arg(long)]
        clean: bool,
        /// Custom element tag for component mode.
        #[arg(long)]
        component_name: Option<String>,
    },

    /// Generate design tokens (project themes, or explicit bake paths).
    Tokens {
        /// Model directory. Required with `--out` for workspace bake.
        #[arg(long)]
        model: Option<PathBuf>,
        /// Output directory. Required with `--model` for workspace bake.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Path inside a Sail project for theme generate (walks up to sail.config).
        #[arg(long)]
        project: Option<PathBuf>,
    },

    /// Emit the sail ambient TypeScript pack (`.sail/types.d.ts`)
    /// for tsc/tsgo. Package `dist/*.d.ts` is bun prepare / bake — not this verb.
    ///
    /// Regenerated from compiler manifests (runtime ambients + Style setters +
    /// token scales). Pair with `@ship/sail-tsconfig/base` or
    /// `deck build --typecheck`. tsc owns controller/logic types; sail-compiler
    /// owns template/layout/vocabulary.
    Types {
        /// Path inside a Sail project (walks up to `sail.config.jsonc` / `.json`).
        /// Writes `.sail/types.d.ts` at the project root. Default: cwd.
        #[arg(long)]
        project: Option<PathBuf>,
        /// Write ambient pack to this path instead of
        /// `<project>/.sail/types.d.ts`.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Also write a recommended project tsconfig (use `-` for stdout).
        #[arg(long)]
        tsconfig_out: Option<PathBuf>,
        /// After writing kind configs, run tsc on sail + action and fail if red.
        #[arg(long)]
        check: bool,
    },

    /// Run compile-time checks on `.sail` files.
    Check {
        /// File or directory of `.sail` sources.
        input: PathBuf,
        /// Auto-fix confluent allow-listed repairs (rhythm, contrast, spacing,
        /// raw-container, sx typo, participation-style, display-on-layout,
        /// prefer-primitive): off | suggest | apply.
        #[arg(long, default_value = "off")]
        fix: String,
    },

    /// Pretty-print `.sail` sources (template CST + oxfmt for scope/controller).
    ///
    /// Format is scaffold-pinned (no user knobs). Idempotent: `fmt ∘ fmt = fmt`.
    Fmt {
        /// File or directory of `.sail` sources. Default: cwd.
        #[arg(default_value = ".")]
        input: PathBuf,
        /// Check only — exit 1 if any file would change (CI).
        #[arg(long)]
        check: bool,
    },

    /// Read-only IR projections (TOOLING-1 WI5). Twin of MCP `sail_inspect`.
    ///
    /// Topics: tree | geometry | cascade | states | a11y | invalidations |
    /// uses | cost | themes. JSON on stdout.
    ///
    /// Examples:
    ///   sail inspect tree apps/demo/routes/page.sail
    ///   sail inspect geometry apps/demo/routes/page.sail --viewport 1440x900
    ///   sail inspect cascade --token primary
    ///   sail inspect themes
    Inspect {
        /// Inspect topic (tree | geometry | cascade | states | a11y |
        /// invalidations | uses | cost | themes).
        topic: String,
        /// Workspace-relative `.sail` path (required for most topics).
        #[arg(default_value = "")]
        path: String,
        /// Geometry viewport `WxH` (e.g. 1440x900).
        #[arg(long, value_parser = parse_render_viewport)]
        viewport: Option<(u32, u32)>,
        /// Cascade: design token path or cssName tail.
        #[arg(long)]
        token: Option<String>,
        /// Cascade / themes: theme id filter.
        #[arg(long = "theme-id")]
        theme_id: Option<String>,
        /// Optional tree node id filter.
        #[arg(long = "node-id")]
        node_id: Option<String>,
    },

    /// Language server (stdio) — same protocol as the `sail-lsp` package binary.
    Lsp,

    /// Write editor config so `*.sail` uses `sail lsp` and the tree-sitter grammar.
    Editor {
        #[command(subcommand)]
        cmd: EditorCmd,
    },

    /// MCP server (stdio / Streamable HTTP) or install into agent hosts.
    ///
    /// Default (no subcommand): run MCP (stdio; pass `--http [ADDR]` through
    /// for Streamable HTTP — MCP 2026-07-28 stateless). Tools are a generated
    /// projection of sail-ops METHOD_CONTRACTS.
    /// `sail mcp install` writes host config (Claude Code `.mcp.json`, Cursor
    /// when detected).
    Mcp {
        #[command(subcommand)]
        cmd: Option<McpCmd>,
        /// Print tool descriptors as JSON and exit.
        #[arg(long = "list-tools")]
        list_tools: bool,
    },

    /// Invoke a sail-ops RPC in-process (`Service/Method`), or `gen` for codegen.
    ///
    /// Same handlers as MCP tools and Connect HTTP under `sail dev`.
    /// Example: `sail op SailTheme/List --json '{"path":"apps/sail-site/themes"}'`
    /// Codegen: `sail op gen` → packages/sail/studio-client + schema snapshots.
    Op {
        /// `Service/Method` (e.g. `SailAuthoring/Surface`) or `gen`.
        target: String,
        /// JSON request body (object). Default `{}`. Ignored for `gen`.
        #[arg(long, default_value = "{}")]
        json: String,
    },

    /// Static box-tree layout diagnostics (spike).
    Layout {
        /// Path to preprocessed/source HTML, or `-` for stdin.
        input: PathBuf,
        /// Viewport `WxH` (default 1440x900).
        #[arg(long, value_parser = peer::parse_viewport, default_value = "1440x900")]
        viewport: (f32, f32),
        /// Emit diagnostics as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Static themed screenshot from a `.sail` file (no browser).
    Render {
        /// Path to a `.sail` source file.
        input: PathBuf,
        /// Theme id (e.g. `sail`). Default is the product brand theme.
        #[arg(long, default_value = sail_compiler::tokens_generate::DEFAULT_PROJECT_THEME_ID)]
        theme: String,
        /// Mode `light` or `dark` (data-mode). Default `light`.
        #[arg(long, default_value = "light")]
        mode: String,
        /// Viewport `WxH` (default 1440x900).
        #[arg(long, value_parser = parse_render_viewport, default_value = "1440x900")]
        viewport: (u32, u32),
        /// Output PNG path. Default: `<input>.png` beside the source.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Also write JSON sidecar (`regions`, `dynamics`, `ms`, …) next to PNG.
        #[arg(long)]
        json: bool,
        /// Optional fonts directory (token woff2).
        #[arg(long)]
        fonts_dir: Option<PathBuf>,
    },

    /// Local environment health (workspace, tokens model, peer surfaces).
    Doctor,

    /// Print version + install hints (also available as `--version`).
    Version,
}

#[derive(Subcommand)]
enum EditorCmd {
    /// Write Helix / Neovim config. Default command is `sail` on PATH.
    Setup {
        /// `helix` or `nvim`.
        editor: String,
        /// Override the `sail` executable (developers). Never implied.
        #[arg(long)]
        bin: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum McpCmd {
    /// Detect agent hosts and write the Sail MCP server entry (no manual JSON).
    ///
    /// Always writes Claude Code project `.mcp.json`. Also writes Cursor
    /// `.cursor/mcp.json` when Cursor is detected.
    Install {
        /// Project root for config files. Default: cwd.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Pin a specific binary (developers). Default: `sail mcp` on PATH.
        #[arg(long)]
        bin: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum Scaffold {
    /// Seed project theme SSOT: copy one Sail default theme into
    /// `<project>/themes/` and write a thin `sail.config.jsonc`
    /// theme section when absent (C5). Apps do not ship the full catalog.
    Theme {
        /// Path inside a Sail project (walks up to `sail.config.jsonc` / `.json`). Default: cwd.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Catalog theme id to copy (default: ship — modes-in-file primary).
        #[arg(long)]
        id: Option<String>,
    },
}

fn main() {
    // Surface bundler diagnostics to stderr. Honors RUST_LOG; floors at
    // `warn` so diagnostics always show even when unset.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .without_time()
        .try_init();
    let brand = binary_brand();
    if let Err(e) = run() {
        eprintln!("{brand}: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let brand = binary_brand();
    match Cli::parse().command {
        Command::Create { name } => scaffold::run_create(name),
        Command::Build {
            project,
            out,
            bundle_framework,
            no_minify,
            typecheck,
            base,
        } => build(project, out, bundle_framework, !no_minify, typecheck, base),
        Command::Scaffold(Scaffold::Theme { project, id }) => scaffold::run_theme(project, id),
        Command::Dev { project, addr } => dev(project, addr),
        Command::Compile {
            input,
            out,
            clean,
            component_name,
        } => peer::compile(input, out, clean, component_name),
        Command::Tokens {
            model,
            out,
            project,
        } => peer::tokens(model, out, project),
        Command::Types {
            project,
            out,
            tsconfig_out,
            check,
        } => peer::types(project, out, tsconfig_out, check),
        Command::Check { input, fix } => peer::check(input, fix),
        Command::Fmt { input, check } => peer::fmt(input, check),
        Command::Inspect {
            topic,
            path,
            viewport,
            token,
            theme_id,
            node_id,
        } => peer::inspect(topic, path, viewport, token, theme_id, node_id),
        Command::Lsp => peer::lsp(),
        Command::Editor { cmd } => match cmd {
            EditorCmd::Setup { editor, bin } => {
                let kind = editor::EditorKind::parse(&editor)?;
                let home = dirs_home().context("HOME / USERPROFILE required for editor setup")?;
                let report = editor::setup(kind, bin.as_deref(), &home)?;
                editor::print_report(&report);
                Ok(())
            }
        },
        Command::Mcp { cmd, list_tools } => match cmd {
            Some(McpCmd::Install { project, bin }) => peer::mcp_install(project, bin),
            None if list_tools => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&sail_mcp::tool_descriptors())
                        .unwrap_or_else(|_| "[]".into())
                );
                Ok(())
            }
            None => {
                peer::mcp();
                Ok(())
            }
        },
        Command::Op { target, json } => peer::op(&target, &json),
        Command::Layout {
            input,
            viewport,
            json,
        } => peer::layout(input, viewport, json),
        Command::Render {
            input,
            theme,
            mode,
            viewport,
            out,
            json,
            fonts_dir,
        } => peer::render(input, theme, mode, viewport, out, json, fonts_dir),
        Command::Doctor => peer::doctor(brand),
        Command::Version => {
            peer::print_version(brand);
            Ok(())
        }
    }
}

/// deck-deploy-gap: the incident this closes is a stale `sail-runtime.js`
/// baked via `include_str!` served against post-rename compiled views —
/// every setup binding came back undefined, the site rendered nothing, and
/// the only symptom was a silent console error.
///
/// Called once at `sail dev` startup: recomputes the current content hash of
/// each framework singleton's TS source tree and compares it against the
/// hash stamped into THIS binary's baked bundle (the leading `/*! ... */`
/// comment `deck-bake build-assets` writes — see `deck_bundler::stamp`).
/// Product `sail` never silent-regens via cargo. Two outcomes:
///   - fresh (hashes match): silent, `sail dev` proceeds.
///   - stale: HARD FAIL. Workspace bake checkout → run `deck-bake`. Else
///     upgrade/rebuild `sail`. Never serve a mismatched runtime silently.
fn ensure_framework_fresh(repo_root: &Path) -> Result<()> {
    let current = deck_bundler::stamp::current_framework_stamps(repo_root)
        .context("sail dev: hashing framework source trees to check bake freshness")?;
    let baked = deck_core::baked_framework_stamps();

    let mut stale: Vec<String> = Vec::new();
    for (name, baked_hash) in &baked {
        // No source tree for this singleton under this repo root — nothing
        // to verify against (an external consumer / partial checkout).
        // `current_framework_stamps` already treats this as "unverifiable,"
        // not "stale"; mirror that here rather than false-failing.
        let Some(cur) = current.iter().find(|s| s.bundle == *name) else {
            continue;
        };
        match baked_hash {
            Some(h) if *h == cur.hash => {}
            Some(h) => stale.push(format!("{name}: baked sha={h} current sha={}", cur.hash)),
            None => stale.push(format!(
                "{name}: baked bundle predates the stamp (no header) — current sha={}",
                cur.hash
            )),
        }
    }

    if stale.is_empty() {
        return Ok(());
    }

    if is_workspace_bake_checkout(repo_root) {
        anyhow::bail!(
            "sail dev: baked framework bundle(s) are STALE:\n  {}\n\
             This checkout bakes with deck-bake, not product sail.\n\
             Run `cargo run -p deck-cli --bin deck-bake -- build-assets`, \
             then rebuild `sail` and restart `sail dev`.",
            stale.join("\n  ")
        );
    }
    anyhow::bail!(
        "sail dev: baked framework bundle(s) are STALE:\n  {}\n\
         Upgrade or rebuild `sail` (this is not a workspace bake checkout).",
        stale.join("\n  ")
    );
}

/// This monorepo checkout — the one that bakes with `deck-bake`, not product `sail`.
fn is_workspace_bake_checkout(root: &Path) -> bool {
    root.join("crates/deck/deck-cli/Cargo.toml").is_file()
        && root.join("packages/sail/runtime/src").is_dir()
}

fn build(
    project: PathBuf,
    out: Option<PathBuf>,
    bundle_framework: bool,
    minify: bool,
    typecheck: bool,
    base: Option<String>,
) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let abs = |p: &Path| {
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            cwd.join(p)
        }
    };

    let from = abs(&project);
    if !from.exists() {
        bail!("project {} does not exist", from.display());
    }
    let project = peer::discover_app(&from)?
        .root()
        .canonicalize()
        .with_context(|| format!("project {}", from.display()))?;
    // The project is the directory that owns `sail.config`. The default
    // out dir lives INSIDE it as `<project>/dist/` — a sibling of
    // `routes/` and `public/`, not above them. The bundler writes the
    // generated `index.html`, the entry chunk, and the framework surface
    // there.
    let out_dir = out
        .map(|o| abs(&o))
        .unwrap_or_else(|| deck_config::deck_out_path(&project));
    let repo_root =
        deck_bundler::paths::topmost_workspace_root(&project).unwrap_or(project.clone());
    // The framework singletons are resolved by NODE RESOLUTION from the
    // project's `node_modules` (the bundler's `framework_singletons` /
    // `bundle_owned_framework`) — `--bundle-framework` builds them from source.
    // There is no prebuilt-asset staging dir and no `DECK_FRAMEWORK_ASSETS_DIR`
    // crutch: the framework comes from wherever it was installed (the registry,
    // a workspace symlink), so a build runs identically inside or outside the
    // sail monorepo.

    // Host route-discovery defaults. Consumer bundles are
    // discovered from per-package `deck.bundles`. L1 (deck) has no per-app
    // config — shipd's apps.jsonc owns the app list; the bundler is per-app
    // and reads its own project root.

    // The app graph compiles its own `.sail` in-process via the linked
    // sail-compiler crate (#255) — no binary to locate.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    let b = rt
        .block_on(deck_bundler::app::bundle_app_with_base(
            &repo_root,
            &project,
            &out_dir,
            minify,
            bundle_framework,
            base.as_deref(),
        ))
        .map_err(|e| anyhow::anyhow!("{e:#}"))?;
    if b.framework_assets > 0 {
        println!("  _deck  {} files (self-contained)", b.framework_assets);
    } else if !bundle_framework {
        // The silent trap: with no `--bundle-framework`, the build emits the app
        // but ZERO framework (`_deck/runtime.js`, `dom.js`, `route.js`) and still
        // exits 0. Fine for a shipd-served app (shipd embeds the framework via
        // `include_str!`), but a standalone host (a Tauri shell, a custom server)
        // serves nothing — every `<sail-*>` stays unregistered and the app renders
        // unstyled with no error. shipd bundles in-process via `bundle_app`, not
        // this CLI path, so this warning targets exactly the standalone builds.
        eprintln!(
            "  warning: built WITHOUT a framework bundle (no --bundle-framework).\n  \
             This app's <sail-*> components need /_deck/runtime.js, dom.js and route.js\n  \
             served by the host. A bundling host embeds them; a standalone host serves nothing,\n  \
             so the app then renders unstyled with every custom element unregistered.\n  \
             For a self-contained build, re-run with --bundle-framework."
        );
    }
    // Opt-in type-check pass (#1). Runs AFTER the bundle so the generated
    // `routes.gen.ts` / `entry.gen.ts` exist and are covered. Default build
    // (flag off) is byte-for-byte unchanged.
    if typecheck {
        typecheck::run(&project, &out_dir, &repo_root)?;
    }
    Ok(())
}

// ── `sail dev` — standalone dev server ─────────────────────────────────────
//
// Composes the existing bundler + router (the same wiring as
// `deck_conformance::deck_dev_server`) so deck can run as a framework
// without a host adapter or shipd. The watcher rebuilds on edit and
// pushes HMR; the router serves the whole `/_deck/*` surface.
//
// The data port is a `NoopDataPort` and the live-data multiplex transport
// (`/_deck/mux/*`) is NOT mounted: standalone `sail dev` serves routing,
// assets, SSR and HMR, but NOT live server query() sources. Live data is a
// HOST capability — a host (shipd) merges `deck::multiplex_router()` and
// layers its own `DataPort`. Wiring live data into the standalone CLI is the
// server-functions arc (#32), targeted for a later release.

/// A `DataPort` that resolves nothing — the standalone `sail dev` default.
/// The live-data multiplex is not mounted here, so this is never actually
/// queried; it satisfies the `Extension<Arc<dyn DataPort>>` the shared
/// handlers expect. (`is_provisioned` defaults to `false` from the trait, so
/// the framework's first-run routing serves `/setup` until a host opts in.)
struct NoopDataPort;

impl deck_core::DataPort for NoopDataPort {
    fn resolve(
        &self,
        _domain: &str,
        _op: &str,
        _args: &serde_json::Value,
    ) -> Option<serde_json::Value> {
        None
    }
}

fn dev(project: PathBuf, addr: String) -> Result<()> {
    // Resolve the project via Project::discover (cwd or --project may be
    // nested). Canonicalize so symlinks/bare relative paths collapse to a
    // single absolute path the bundler and resolver can key off.
    let cwd = std::env::current_dir().context("cwd")?;
    let project_path = if project.is_absolute() {
        project
    } else {
        cwd.join(&project)
    };
    if !project_path.exists() {
        anyhow::bail!("{} does not exist", project_path.display());
    }
    let project_dir = peer::discover_app(&project_path)?
        .root()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_path.display()))?;
    // Workspace is bake / monorepo `@ship` resolution — not the app.
    let workspace_root =
        deck_bundler::paths::topmost_workspace_root(&project_dir).unwrap_or(project_dir.clone());

    // deck-deploy-gap: never serve a framework bundle staler than its
    // sources. Product `sail` does not silent-regen via cargo — fail loud
    // (workspace bake checkout → `deck-bake`; else upgrade/rebuild `sail`).
    ensure_framework_fresh(&workspace_root)?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    runtime.block_on(async move {
        // deck-core's resolution is mount-scoped: the project root is carried
        // per-request through the `Resolution` Extension layered below (not a
        // process global), so `serve_index` / `serve_app_*` / `spa_fallback`
        // resolve the app's built assets (`dist/index.html`, `/<name>/<file>`)
        // against THIS mount's root.
        let cfg = deck_config::DeckConfig::defaults();

        // Cold-start tokens generate when the project has theme SSOT on disk
        // (themes/*.json or sail.config theme section).
        // Without this,
        // project DTCG only re-enters the emit set after a watch event — first
        // paint after restart would stay catalog-only package CSS.
        if sail_compiler::tokens_generate::project_wants_theme_generate(&project_dir) {
            match run_tokens_generate(&workspace_root, &project_dir) {
                Ok(report) => {
                    println!(
                        "sail dev: tokens generate (cold-start: {} tokens, {} themes) → {}",
                        report.token_count,
                        report.theme_count,
                        report.themes_css.display()
                    );
                }
                Err(e) => {
                    // Fail loud: theme SSOT present but generate broke — don't
                    // serve stale catalog CSS as if project paint worked.
                    return Err(anyhow::anyhow!("sail dev: cold-start tokens generate failed: {e:#}"));
                }
            }
        }

        // Build the app ONCE up front. `start_watching` only REBUILDS on
        // edit; without an initial build the project emits no `dist/`
        // output and EVERY page 404s — the gap a runtime-only smoke test
        // (just `/_deck/runtime.js`) misses. `bundle_framework = false`:
        // sail dev serves the framework singletons itself at `/_deck/*`,
        // so the app graph keeps them external (no per-app copy).
        let out_dir = deck_config::deck_out_path(&project_dir);
        let build = deck_bundler::app::bundle_app(
            &workspace_root,
            &project_dir,
            &out_dir,
            false,
            false,
        )
        .await
        .map_err(|e| anyhow::anyhow!("sail dev: initial app build failed: {e:#}"))?;
        println!("sail dev: built app ({} css asset(s))", build.css_hrefs.len());

        // Register the project as the PRIMARY app at `/`. `name` MUST equal
        // the URL namespace `bundle_app` baked the asset hrefs under, so
        // `/<name>/<file>` resolves — so we take it from the build result
        // (`AppBuild::url_namespace`) rather than recomputing it here, killing
        // the second namespace derivation that could silently drift from the
        // bake. `source` is the build out dir relative to the project root.
        // sail.config nested `deck` serve keys for injectGlobals / render /
        // entry — name/route/source are host-bound here, never the file's.
        let mut app_cfg = deck_bundler::load_app_serve(&project_dir)
            .map_err(|e| anyhow::anyhow!("app serve config: {e}"))?;
        // SSR engine: only spin up ship-js (V8) when this app opts into SSR
        // (`sail.config { "deck": { "render": "ssr" } }`). A CSR app (the
        // default) pays no V8 cost. Standalone dev has no live data port, so
        // the engine renders with NoopData (None). The engine is a MOUNT-SCOPED
        // handle layered as an Extension below — not a process-global install.
        // If it can't start, log + continue; the serve path falls back to CSR
        // (`renderer = None`).
        let renderer: Option<Arc<dyn deck_core::synthetic_ssr::SyntheticSsr>> =
            if app_cfg.render == deck_config::RenderMode::Ssr {
                match deck_ssr::serve::engine(None) {
                    Ok(r) => {
                        println!("sail dev: ship-js SSR engine ready (render: ssr)");
                        Some(r)
                    }
                    Err(e) => {
                        eprintln!("sail dev: ship-js SSR engine unavailable, serving CSR: {e:#}");
                        None
                    }
                }
            } else {
                None
            };
        app_cfg.name = build.url_namespace.clone();
        app_cfg.route = "/".to_string();
        app_cfg.source = deck_config::DECK_OUT_DIRNAME.to_string();
        // The per-app asset namespace is the app's name (`/<name>/<file>`);
        // `serve_app_asset` reads the app off this per-route extension and
        // maps to `<source>/assets/<file>`.
        let asset_route = format!("/{}/{{file}}", app_cfg.name);
        let app_ext = Arc::new(app_cfg.clone());

        // Sail Studio (apps/studio): when the monorepo ships Studio and the
        // primary project is NOT Studio itself, also serve it at `/_studio`
        // for the overlay bridge (TOOL-4). Fail-soft: missing/broken/hung
        // studio build never blocks the primary app.
        //
        // Reuse existing `dist/` by default — a full studio rebuild can hang
        // in layout (theme-shape etc.) and used to block docs from ever
        // reaching `serving`. Set DECK_STUDIO_FORCE_REBUILD=1 to force rebuild.
        let studio_dir = workspace_root.join("apps/studio");
        let primary_is_studio = project_dir
            .canonicalize()
            .ok()
            .zip(studio_dir.canonicalize().ok())
            .map(|(a, b)| a == b)
            .unwrap_or(false);
        let mut studio_mount: Option<(
            Arc<deck_config::AppServeConfig>,
            Arc<deck_core::Resolution>,
            Option<Arc<dyn deck_core::synthetic_ssr::SyntheticSsr>>,
        )> = None;
        if studio_dir.is_dir() && !primary_is_studio {
            let studio_out = deck_config::deck_out_path(&studio_dir);
            let force_rebuild = std::env::var_os("DECK_STUDIO_FORCE_REBUILD").is_some();
            let has_out = studio_out.join("index.html").is_file();
            let reuse = has_out && !force_rebuild;

            let namespace: Option<String> = if reuse {
                println!(
                    "sail dev: reusing studio out {} (DECK_STUDIO_FORCE_REBUILD=1 to rebuild)",
                    studio_out.display()
                );
                // On-disk index references /studio/* assets.
                Some("studio".to_string())
            } else {
                println!("sail dev: building studio overlay…");
                match tokio::time::timeout(
                    std::time::Duration::from_secs(180),
                    deck_bundler::app::bundle_app(
                        &workspace_root,
                        &studio_dir,
                        &studio_out,
                        false,
                        false,
                    ),
                )
                .await
                {
                    Ok(Ok(studio_build)) => Some(studio_build.url_namespace),
                    Ok(Err(e)) => {
                        eprintln!("sail dev: studio overlay build skipped: {e:#}");
                        None
                    }
                    Err(_) => {
                        eprintln!(
                            "sail dev: studio overlay rebuild timed out after 180s"
                        );
                        if has_out {
                            eprintln!(
                                "sail dev: falling back to existing studio out {}",
                                studio_out.display()
                            );
                            Some("studio".to_string())
                        } else {
                            None
                        }
                    }
                }
            };

            if let Some(url_namespace) = namespace {
                let mut studio_cfg =
                    deck_bundler::load_app_serve(&studio_dir).unwrap_or_default();
                studio_cfg.name = url_namespace;
                studio_cfg.route = "/_studio".to_string();
                studio_cfg.source = deck_config::DECK_OUT_DIRNAME.to_string();
                // Tools (explorer/checks/design/tasks) target the *primary*
                // app under `sail dev --project`, not apps/studio itself.
                if let Some(rel) = deck_core::sail_project_rel_from_root(&project_dir) {
                    studio_cfg.ops_project = Some(rel);
                }
                // Studio opts into SSR; share the primary renderer if already
                // spun up, else start one when Studio needs it.
                let studio_renderer: Option<Arc<dyn deck_core::synthetic_ssr::SyntheticSsr>> =
                    if studio_cfg.render == deck_config::RenderMode::Ssr {
                        if let Some(r) = renderer.clone() {
                            Some(r)
                        } else {
                            match deck_ssr::serve::engine(None) {
                                Ok(r) => {
                                    println!(
                                        "sail dev: ship-js SSR engine ready (studio overlay)"
                                    );
                                    Some(r)
                                }
                                Err(e) => {
                                    eprintln!(
                                        "sail dev: studio SSR unavailable, CSR overlay: {e:#}"
                                    );
                                    None
                                }
                            }
                        }
                    } else {
                        None
                    };
                let studio_resolution =
                    Arc::new(deck_core::Resolution::new().with_root(studio_dir.clone()));
                println!(
                    "sail dev: studio overlay at /_studio (namespace {})",
                    studio_cfg.name
                );
                studio_mount = Some((
                    Arc::new(studio_cfg),
                    studio_resolution,
                    studio_renderer,
                ));
            }
        }

        let apps: Arc<Vec<deck_config::AppServeConfig>> = Arc::new(vec![app_cfg]);

        // Mount-scoped resolution: sail dev carries its single project root as a
        // per-request `Resolution` handle (layered as an Extension below). The
        // ENTIRE serve path — index / assets / public / SSR first-paint —
        // resolves through it (`resolve_project_root_with`); there is no process
        // global to read. A multi-app host (shipd) carries a DISTINCT Resolution
        // per app, so two projects in one process never cross-talk
        // (`inv-deck-mount-scoped-resolution`).
        let resolution = Arc::new(deck_core::Resolution::new().with_root(project_dir.clone()));

        // Bundler with the FS watcher (HMR SSE on `/_deck/dev/hmr`). The
        // up-front `bundle_app` is the one-shot SPA bundle that makes the
        // app servable; the watcher drives rebuilds + live-reload on edit.
        let bundler = deck_bundler::Bundler::start(project_dir.clone(), None, cfg.clone());
        if let Err(e) = bundler.start_watching().await {
            eprintln!("sail dev: watcher failed to start: {e}");
        }

        // ── HMR (app-granular remount: keep signals, swap CSS — not Vite) ──
        // The bundler's own watcher rebuilds TARGETS (views, framework). The
        // SPA is the one-shot `bundle_app` ENTRY — not a target — so `routes/` +
        // `src/` + library source edits emit nothing through it. sail dev owns
        // this loop: a notify watcher on authored source re-runs `bundle_app`
        // and emits a `module-changed` (target `app`) on the bundler's firehose,
        // which `serve_bundle_hmr` streams to the live-reload client → the
        // client re-imports the hashed entry → `__deckHmr.swapModules` remounts
        // the current route (signal values kept, hashed CSS swapped). Not a
        // full document reload. Not per-route `swapModule`.
        //
        // Watch roots:
        // - project `routes/` + `src/` + `public/` + `themes/`
        //   — never project root / `dist/` / `.sail/` (regenerated
        //   `routes.gen.ts` / out dir must not self-trigger). Project theme
        //   SSOT is `themes/**`.
        // - discovered library `src/` trees (and package root if no `src/`)
        //   so edits in `@ship/sail-components` HMR like local rails.
        // - monorepo catalog themes: `packages/sail/tokens/model/**` (DTCG SSOT;
        //   generated `src/tokens.css` / `themes.css` are NOT watched — those
        //   are outputs of `sail-compiler tokens`).
        //
        // Live theme edit path (only supported path):
        //   save DTCG under themes/ or model/themes
        //   (or project sail.config theme section) → tokens generate:
        //     package: full catalog + project → packages/sail/tokens/src
        //     app:     T0 effective set → <project>/.sail/themes/themes.css (B1)
        //   → HMR swaps CSS. No client IR / :root setProperty.
        let (hmr_tx, mut hmr_rx) = tokio::sync::mpsc::channel::<Vec<std::path::PathBuf>>(8);
        // Set when a watched path is theme/model/config source; rebuild loop runs
        // dual `tokens_generate` before `bundle_app` so delivery CSS stays
        // regenerate-only (package full + app-local filtered themes).
        // `project_theme_edit` / `catalog_model_edit` distinguish log lines
        // (project vs catalog SSOT) after generate.
        let need_tokens_generate = Arc::new(AtomicBool::new(false));
        let project_theme_edit = Arc::new(AtomicBool::new(false));
        let catalog_model_edit = Arc::new(AtomicBool::new(false));
        // Notify overflow used to be safe (next tick always rebuilt). Skip now
        // keys off the path list — a dropped `.ts` + later `.sail` would skip
        // the sail typecheck. Sticky unknown forces the gate on the next tick.
        // A red type-gate tick is also sticky (`gate_was_dirty` in the rebuild
        // loop): the next tick must re-run the gate, not skip.
        let hmr_gate_unknown = Arc::new(AtomicBool::new(false));
        // Keep the notify watcher alive for the whole `sail dev` session.
        // Ownership for mid-session theme attach (#13/#14):
        // - Strong `Arc` lives only on the main path (`hmr_watcher`).
        // - Attach thread holds `Weak` so drop(Arc) can destroy the Watcher.
        // - `attach_tx` lives only in the notify callback; Watcher drop closes
        //   the channel → attach thread's `recv` ends → join returns.
        // Never put a strong Arc on the attach thread (that cycles with tx-in-Watcher).
        let (hmr_watcher, theme_attach_join) = {
            use notify::Watcher;
            use std::sync::Mutex;
            let tx = hmr_tx;
            let need_gen = Arc::clone(&need_tokens_generate);
            let proj_edit = Arc::clone(&project_theme_edit);
            let cat_edit = Arc::clone(&catalog_model_edit);
            let gate_unknown = Arc::clone(&hmr_gate_unknown);
            let project_for_cb = project_dir.clone();
            let project_tsconfig = project_dir.join("tsconfig.json");
            let workspace_for_watch = workspace_root.clone();
            // Attach commands: callback must not call `watch` re-entrantly on
            // some backends; a dedicated thread owns re-attach under the mutex.
            let (attach_tx, attach_rx) = std::sync::mpsc::sync_channel::<()>(4);
            let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                use notify::EventKind;
                use notify::event::ModifyKind;
                let Ok(ev) = res else { return };
                // CRITICAL: ignore read/stat events. `bundle_app` READS the
                // source to compile it, which fires Access/Metadata events —
                // firing a rebuild on those is the read→rebuild→read CPU-spin
                // loop. Mirror the bundler's own watch filter.
                if matches!(
                    ev.kind,
                    EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_))
                ) {
                    return;
                }
                // Fan-out to sail-ops watch channel (SSE /sail.ops.v1.watch).
                // Paths are workspace-relative (not absolute host paths).
                for p in &ev.paths {
                    let s = p.to_string_lossy().replace('\\', "/");
                    if s.ends_with(".sail") {
                        sail_ops::publish_file_under(&workspace_for_watch, p);
                    }
                }
                // Ignore generated artifacts a rebuild writes (`.gen.ts`,
                // `.sail.d.ts` companions, `.compiled.js`, library `dist/`,
                // tokens package *delivery* under packages/sail/tokens/src)
                // so they can't self-trigger. Model/themes JSON is SSOT and
                // is watched deliberately.
                let all_generated = !ev.paths.is_empty()
                    && ev.paths.iter().all(|p| {
                        let s = p.to_string_lossy().replace('\\', "/");
                        s.ends_with(".gen.ts")
                            || s.contains(".sail.d.ts")
                            || s.contains(".compiled.js")
                            || s.contains("/dist/")
                            || s.contains("/.sail/")
                            || s.contains("/node_modules/")
                            || s.ends_with("/dist")
                            || p.file_name().and_then(|n| n.to_str()) == Some("dist")
                            || s.ends_with("/.sail")
                            || p.file_name().and_then(|n| n.to_str()) == Some(".sail")
                            // Atomic-publish sibling (`dist.stage`, `dist.stage.<suffix>`).
                            || s.contains("/dist.stage")
                            || p.file_name()
                                .and_then(|n| n.to_str())
                                .is_some_and(|n| n.starts_with("dist.stage"))
                            || is_tokens_generated_delivery(&s)
                            // HMR rebuild re-emits the generated solution
                            // tsconfig at the project root. Even a same-bytes
                            // write used to bump mtime and loop; ignore it.
                            || p == project_tsconfig.as_path()
                    });
                if all_generated {
                    return;
                }
                // Mid-session scaffold: only Create/Rename of tokens tree roots
                // or sail.config (not every theme leaf modify — #15).
                let is_create_or_rename = matches!(
                    ev.kind,
                    EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(_))
                );
                if is_create_or_rename {
                    let mut want_attach = false;
                    for p in &ev.paths {
                        if path_suggests_new_theme_watch(p, &project_for_cb) {
                            want_attach = true;
                            break;
                        }
                    }
                    if want_attach {
                        let _ = attach_tx.try_send(());
                    }
                }
                // Theme/model/config SSOT change → regenerate delivery CSS before rebuild.
                for p in &ev.paths {
                    if !is_theme_source_path(p) {
                        continue;
                    }
                    need_gen.store(true, Ordering::Release);
                    if is_project_theme_path(p) || is_project_sail_config_path(p) {
                        proj_edit.store(true, Ordering::Release);
                    }
                    if is_catalog_model_path(p) {
                        cat_edit.store(true, Ordering::Release);
                    }
                }
                // dev-loop-atomic (defect 2): a STRUCTURAL change — a new
                // `src/<tag>/` component dir, a created/removed/renamed
                // `.sail` — must clear the process-global tag→source memo
                // (`COMPONENT_INDEX_CACHE`) BEFORE the rebuild below runs.
                // The bundler's internal watcher does this for TARGET builds,
                // but the app rebuild is driven by THIS watcher; without the
                // invalidation a component authored mid-session resolves to a
                // virtual no-op (its element never upgrades, slotted children
                // render bare) until full server restart.
                if is_structural_change(&ev.kind) {
                    deck_bundler::sail_plugin::invalidate_component_index();
                }
                // Coalesce a save's event burst into one rebuild via the debounce
                // below. Overflow is no longer safe to drop silently: skip keys
                // off this path list, so a lost `.ts` + later `.sail` would skip
                // the sail typecheck. Mark unknown and still try to enqueue a tick.
                if tx.try_send(ev.paths.clone()).is_err() {
                    gate_unknown.store(true, Ordering::Release);
                }
            })
            .map_err(|e| anyhow::anyhow!("sail dev: HMR watcher init: {e}"))?;
            let mut watched: Vec<PathBuf> = Vec::new();
            for sub in ["routes", "src", "public", "tokens"] {
                let dir = project_dir.join(sub);
                if dir.is_dir() {
                    watched.push(dir);
                }
            }
            // Discovered component libraries. Prefer source trees so we
            // don't pick up library `dist/` churn. Umbrella
            // `@ship/sail-components` watches `src/` and each domain
            // `*/src` (primitives/src, charts/src, …).
            let lib_roots = deck_bundler::catalog::component_package_roots(&workspace_root);
            for lib_root in lib_roots.values() {
                if lib_root == &project_dir {
                    continue; // already covered by project src/
                }
                let src_roots = deck_bundler::catalog::component_src_roots(lib_root);
                if !src_roots.is_empty() {
                    watched.extend(src_roots);
                } else if lib_root.is_dir() {
                    watched.push(lib_root.clone());
                }
            }
            // Compiler stdlib `.sail` (buttons, layout keywords, …) — rare edits
            // but without this a stdlib tweak also needed a full restart.
            let stdlib = workspace_root.join("crates/sail/compiler/stdlib");
            if stdlib.is_dir() {
                watched.push(stdlib);
            }
            // Catalog DTCG SSOT (framework themes + base/dark/density/…).
            // Live edit = save JSON here (or apps/*/themes) → generate → HMR.
            let tokens_model = workspace_root.join("packages/sail/tokens/model");
            if tokens_model.is_dir() {
                watched.push(tokens_model);
            }
            // De-dupe + watch recursive trees
            watched.sort();
            watched.dedup();
            let mut n = 0usize;
            for dir in &watched {
                if w.watch(dir, notify::RecursiveMode::Recursive).is_ok() {
                    n += 1;
                }
            }
            // Always NonRecursive-watch the project root so mid-session create of
            // `sail.config.json(c)` or `tokens/` is visible (#13). Not Recursive:
            // `dist/` / `.sail/` write storms must not self-trigger (also filtered above).
            let mut n_root = 0usize;
            if w
                .watch(&project_dir, notify::RecursiveMode::NonRecursive)
                .is_ok()
            {
                n_root = 1;
            }
            // Existing sail.config + tokens tree (tokens already in `watched` if present).
            let n_cfg = attach_project_theme_watches(&mut w, &project_dir);
            let w = Arc::new(Mutex::new(w));
            // Weak only on the attach thread — strong Arc stays on main path so
            // drop(hmr_watcher) can destroy Watcher → drop attach_tx → recv ends.
            let w_weak = Arc::downgrade(&w);
            let project_attach = project_dir.clone();
            let theme_attach_join = std::thread::Builder::new()
                .name("deck-theme-watch-attach".into())
                .spawn(move || {
                    while attach_rx.recv().is_ok() {
                        // Drain coalesced attach requests.
                        while attach_rx.try_recv().is_ok() {}
                        let Some(strong) = w_weak.upgrade() else {
                            // Watcher already dropped — exit cleanly.
                            break;
                        };
                        if let Ok(mut guard) = strong.lock() {
                            attach_project_theme_watches(&mut *guard, &project_attach);
                        }
                        drop(strong);
                    }
                })
                .map_err(|e| anyhow::anyhow!("sail dev: theme watch attach thread: {e}"))?;
            println!(
                "sail dev: HMR watching {n} tree(s) + {n_root} project-root + \
                 {n_cfg} theme-config (app routes/src/public/tokens + libraries + \
                 tokens model; theme paths attach mid-session)"
            );
            (w, theme_attach_join)
        };
        {
            let bundler_hmr = bundler.clone();
            let ws = workspace_root.clone();
            let proj = project_dir.clone();
            let od = out_dir.clone();
            let need_gen = Arc::clone(&need_tokens_generate);
            let proj_edit = Arc::clone(&project_theme_edit);
            let cat_edit = Arc::clone(&catalog_model_edit);
            let gate_unknown = Arc::clone(&hmr_gate_unknown);
            tokio::spawn(async move {
                // Last tick had type-gate errors: the next tick must re-run
                // the gate even if `changed` is `.sail`-only. Skip would
                // rebuild + emit `module-changed` and clear the overlay
                // while `.ts` is still red.
                let mut gate_was_dirty = false;
                while let Some(first) = hmr_rx.recv().await {
                    let mut changed = first;
                    // Debounce: settle the burst, then drain coalesced ticks.
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    while let Ok(more) = hmr_rx.try_recv() {
                        changed.extend(more);
                    }
                    let force_gate = force_hmr_type_gate(
                        gate_unknown.swap(false, Ordering::AcqRel),
                        gate_was_dirty,
                    );
                    // Theme/model/config edit → dual generate (T1 B1):
                    // package full catalog + app-local .sail/themes/themes.css.
                    if need_gen.swap(false, Ordering::AcqRel) {
                        let project_edit = proj_edit.swap(false, Ordering::AcqRel);
                        let catalog = cat_edit.swap(false, Ordering::AcqRel);
                        match run_tokens_generate(&ws, &proj) {
                            Ok(report) => {
                                let src = match (project_edit, catalog) {
                                    (true, true) => "project+catalog",
                                    (true, false) => "project",
                                    (false, true) => "catalog",
                                    (false, false) => "tokens",
                                };
                                println!(
                                    "sail dev: tokens generate ({src}: {} tokens, {} themes) → {}",
                                    report.token_count,
                                    report.theme_count,
                                    report.themes_css.display()
                                );
                            }
                            Err(e) => {
                                eprintln!("sail dev: tokens generate failed: {e:#}");
                                let _ = bundler_hmr.emit_watch_event(
                                    deck_bundler::BundlerEvent::CompileError {
                                        target: "tokens".to_string(),
                                        message: format!("tokens generate: {e:#}"),
                                        line: None,
                                        col: None,
                                        file: None,
                                    },
                                );
                                // Still rebuild app so non-token edits aren't stuck;
                                // the overlay surfaces the generate failure.
                            }
                        }
                    }
                    let _ = sail_compiler::ambients::write_project_type_configs(&proj);
                    let gate_diags =
                        sail_compiler::check::tscheck::check_project_type_gate_incremental(
                            &proj, &changed, force_gate,
                        );
                    let gate_errs: Vec<_> = gate_diags
                        .into_iter()
                        .filter(|d| d.severity == sail_compiler::DiagSeverity::Error)
                        .collect();
                    if !gate_errs.is_empty() {
                        gate_was_dirty = true;
                        let message =
                            sail_compiler::check::tscheck::format_project_gate_overlay(&gate_errs);
                        eprintln!("sail dev: sail typecheck failed");
                        let file = gate_errs.first().and_then(|d| {
                            d.message
                                .rsplit_once('[')
                                .and_then(|(_, rest)| rest.strip_suffix(']'))
                                .map(|s| s.to_string())
                        });
                        let _ = bundler_hmr.emit_watch_event(
                            deck_bundler::BundlerEvent::CompileError {
                                target: "app".to_string(),
                                message,
                                line: gate_errs.first().map(|d| d.line as u32),
                                col: gate_errs.first().map(|d| d.col as u32),
                                file,
                            },
                        );
                        continue;
                    }
                    gate_was_dirty = false;
                    match deck_bundler::app::bundle_app(&ws, &proj, &od, false, false).await {
                        Ok(_) => {
                            let _ = bundler_hmr.emit_watch_event(
                                deck_bundler::BundlerEvent::ModuleChanged {
                                    target: "app".to_string(),
                                    out_path: od.join("index.html").display().to_string(),
                                    bytes: 0,
                                },
                            );
                            println!("sail dev: rebuilt app (HMR)");
                        }
                        Err(e) => {
                            eprintln!("sail dev: rebuild failed: {e:#}");
                            let _ = bundler_hmr.emit_watch_event(
                                deck_bundler::BundlerEvent::CompileError {
                                    target: "app".to_string(),
                                    message: format!("{e:#}"),
                                    line: None,
                                    col: None,
                                    file: None,
                                },
                            );
                        }
                    }
                }
            });
        }

        // The FULL hosting surface. `deck_axum::router()` is the `/_deck/*`
        // framework infra; it sets NO fallback by design (it expects to
        // merge alongside a host's Connect-ops router, which owns the
        // fallback — two merged fallbacks panic axum). sail dev IS the
        // host, so it adds the primary `/` index, the per-app routes
        // (`serve_apps`: asset namespace + single-segment public catch-all),
        // and the SPA navigation fallback that serves the app shell for
        // multi-segment client routes. Extension layers: `DaemonMode`, the
        // `DeckConfig` (Arc), the app list (`serve_index` / `spa_fallback`
        // read it to find the primary app), the `Bundler`, and the no-op
        // `DataPort`. The live-data multiplex (`/_deck/mux/*`) is intentionally
        // NOT merged here — standalone deck has no live query() source (#32).
        use axum::routing::get;
        // Connect unary: sail.ops.v1.* — same handlers as MCP + `sail op`.
        // Theme list/read/write/fork/generate replace the old `/_deck/dev/*` FS.
        let ops_ctx = sail_ops::Ctx::for_dev(workspace_root.clone(), project_dir.clone());
        let mut app = deck_axum::router()
            // `/` → the primary app's index.html.
            .route("/", get(deck_axum::pages::serve_index))
            // Merge Connect ops (theme + authoring) before SPA routes.
            .merge(sail_ops::connect_router_with_ctx(ops_ctx))
            // `/<name>/<file>` → the app's hashed bundle assets
            // (`entry-*.js`, `tokens-*.css`) under `<source>/assets/`.
            .route(
                &asset_route,
                get(deck_axum::apps::serve_app_asset).layer(axum::Extension(app_ext)),
            );
        // Studio overlay routes (when monorepo apps/studio was bundled above).
        // Nested router owns Studio's AppServeConfig + Resolution (+ optional
        // SSR engine) so it never inherits the primary app's project root.
        if let Some((studio_cfg, studio_res, studio_rend)) = studio_mount {
            let studio_asset = format!("/{}/{{file}}", studio_cfg.name);
            // `/_studio` and `/_studio/*` both serve the studio shell so deep
            // links (`/edit`, `/checks`, …) work under the overlay prefix.
            // Without the wildcard, SPA fallback hands primary-app HTML.
            let mut studio_router = axum::Router::new()
                .route(
                    "/_studio",
                    get(deck_axum::apps::serve_app_index),
                )
                .route(
                    "/_studio/{*path}",
                    get(deck_axum::apps::serve_app_index),
                )
                .route(
                    &studio_asset,
                    get(deck_axum::apps::serve_app_asset),
                )
                .layer(axum::Extension(studio_cfg))
                .layer(axum::Extension(studio_res));
            if let Some(r) = studio_rend {
                studio_router = studio_router.layer(axum::Extension(r));
            }
            app = app.merge(studio_router);
        }
        app = app
            // `/<seg>` → single-segment: a DOTTED name is a public asset
            // (`favicon.ico`); a BARE segment is a CLIENT route (spaces'
            // `/:appId`), so serve the app shell and let client-side routing
            // resolve it. `serve_app_public` (what `serve_apps` mounts here)
            // would 404 the bare case — `spa_or_public_asset` handles both.
            .route("/{file}", get(deck_axum::pages::spa_or_public_asset))
            // Deeper unmatched GET navigations (multi-segment client routes)
            // → the shell. deck's `router()` sets no fallback by design (it
            // expects to merge alongside a host's ops router); sail dev IS
            // the host, so it owns the fallback.
            .fallback(deck_axum::pages::spa_fallback)
            .layer(axum::Extension(DaemonMode::Development))
            .layer(axum::Extension(Arc::new(cfg)))
            .layer(axum::Extension(bundler.clone()))
            .layer(axum::Extension(apps))
            .layer(axum::Extension(resolution))
            .layer(axum::Extension(Arc::new(NoopDataPort) as Arc<dyn deck_core::DataPort>));
        // The SSR engine, when this app opted into `render: ssr`, is layered as a
        // mount-scoped Extension the serve path threads into first-paint. Absent
        // ⇒ CSR (no process-global renderer install).
        if let Some(r) = renderer {
            app = app.layer(axum::Extension(r));
        }
        let bind_addr = addr.clone();
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .with_context(|| format!("bind {bind_addr}"))?;
        let bound = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| bind_addr.clone());
        println!("sail dev: serving {} on http://{bound}", project_dir.display());
        println!(
            "sail dev: standalone — routes/assets/HMR/SSR-opt-in; no live DataPort/query multiplex (server-functions #32)"
        );
        println!("sail dev: ctrl-c to stop");

        // Graceful shutdown: a ctrl-c finishes the in-flight request
        // (axum::serve's documented hook) and then we tear the
        // bundler's watch loop down before exit.
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                if let Err(e) = tokio::signal::ctrl_c().await {
                    eprintln!("sail dev: ctrl_c handler: {e}");
                }
            })
            .await
            .context("dev server")?;

        // Bundler watchdog: stop the watch task before returning so
        // the process doesn't linger on a detached tokio task.
        // Theme-attach shutdown order (#14):
        // 1. Drop the strong Arc → Watcher drops → notify callback drops →
        //    attach_tx drops → attach_rx.recv fails.
        // 2. Join the attach thread (holds only Weak; cannot keep Watcher alive).
        drop(hmr_watcher);
        if let Err(e) = theme_attach_join.join() {
            eprintln!("sail dev: theme watch attach thread join: {e:?}");
        }
        if let Err(e) = bundler.shutdown().await {
            eprintln!("sail dev: bundler shutdown: {e}");
        }
        Ok(())
    })
}

/// Next HMR tick must run the project type gate (no `.sail`-only skip)
/// when notify overflowed (`unknown`) or the previous tick had type-gate
/// errors (`last_dirty`). Overflow loses path identity; a dirty previous
/// tick must not skip while still red (overlay would clear on rebuild).
fn force_hmr_type_gate(unknown: bool, last_dirty: bool) -> bool {
    unknown || last_dirty
}

/// Does this watch event change the COMPONENT STRUCTURE of the project —
/// i.e. may a `src/<tag>/` dir or a `.sail` component have appeared,
/// vanished, or moved? Create/Remove/Rename events qualify; content edits
/// (`Modify(Data)`) don't (the tag→source map is unchanged), and
/// Access/Metadata never reach here (filtered upstream). Structural events
/// clear the memoized component index so the NEXT rebuild re-walks the
/// workspace and registers the new tag — event-driven, no polling
/// (dev-loop-atomic, defect 2).
fn is_structural_change(kind: &notify::EventKind) -> bool {
    use notify::event::ModifyKind;
    matches!(
        kind,
        notify::EventKind::Create(_)
            | notify::EventKind::Remove(_)
            | notify::EventKind::Modify(ModifyKind::Name(_))
    )
}

/// Project theme SSOT: `**/themes/**`
/// (and the parent `themes` dir so Create events from mid-session
/// scaffold fire generate).
fn is_project_theme_path(path: &Path) -> bool {
    let norm = format!("/{}", path.to_string_lossy().replace('\\', "/"));
    // Never treat monorepo catalog model as project themes.
    if norm.contains("/packages/sail/tokens/model/") {
        return false;
    }
    // Project layout: <project>/themes/** — require a project segment before
    // /themes (not bare catalog model paths).
    if let Some(i) = norm.rfind("/themes/") {
        let before = &norm[..i];
        // Retired `<project>/tokens/themes` is not a project theme source.
        if before.ends_with("/tokens") {
            return false;
        }
        // Reject packages/sail/tokens/model/themes (already gated above) and
        // bare "/themes/" at root-ish paths without a project parent.
        return !before.is_empty() && before != "/";
    }
    if let Some(stripped) = norm.strip_suffix("/themes") {
        if stripped.ends_with("/tokens") {
            return false;
        }
        return !stripped.is_empty() && stripped != "/";
    }
    false
}

/// Project-root (or any) `sail.config.json` / `sail.config.jsonc` — theme
/// emit config. Basename match: load only uses the project-root file, but
/// any config save still re-runs generate (cheap; options re-read disk).
fn is_project_sail_config_path(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|n| n.to_str()),
        Some("sail.config.json") | Some("sail.config.jsonc")
    )
}

/// Attach recursive `tokens/` + NonRecursive `sail.config.json(c)` watches when
/// those paths exist. Safe to call repeatedly (`watch` on an already-watched
/// path is a no-op / benign error). Returns how many sail.config files attached.
fn attach_project_theme_watches(w: &mut impl notify::Watcher, project_dir: &Path) -> usize {
    let tokens = project_dir.join("tokens");
    if tokens.is_dir() {
        let _ = w.watch(&tokens, notify::RecursiveMode::Recursive);
    }
    let themes = project_dir.join("themes");
    if themes.is_dir() {
        let _ = w.watch(&themes, notify::RecursiveMode::Recursive);
    }
    let mut n_cfg = 0usize;
    for name in ["sail.config.jsonc", "sail.config.json"] {
        let cfg_path = project_dir.join(name);
        if cfg_path.is_file()
            && w.watch(&cfg_path, notify::RecursiveMode::NonRecursive)
                .is_ok()
        {
            n_cfg += 1;
        }
    }
    n_cfg
}

/// True when an event path means we should (re)attach theme watches — mid-session
/// `deck scaffold theme` creates `themes/` and/or `sail.config` after startup.
///
/// Only **tree roots / config files**, not routine leaf edits under
/// `themes/*.json`. Callers should also gate on Create/Rename.
fn path_suggests_new_theme_watch(path: &Path, project_dir: &Path) -> bool {
    if is_project_sail_config_path(path) {
        return true;
    }
    // Theme tree roots only — not themes/<id>.json leaves.
    let norm = format!("/{}", path.to_string_lossy().replace('\\', "/"));
    if !norm.ends_with("/themes") {
        return false;
    }
    // Prefer paths under this project when we can resolve parents.
    let Some(parent) = path.parent() else {
        return false;
    };
    // `…/themes` → parent is project.
    let project_candidate = Some(parent);
    let Some(project_candidate) = project_candidate else {
        return false;
    };
    if let (Ok(cand), Ok(proj)) = (project_candidate.canonicalize(), project_dir.canonicalize()) {
        return cand == proj;
    }
    project_candidate == project_dir
}

/// Catalog model SSOT: `packages/sail/tokens/model/**`.
fn is_catalog_model_path(path: &Path) -> bool {
    let norm = format!("/{}", path.to_string_lossy().replace('\\', "/"));
    norm.contains("/packages/sail/tokens/model/")
}

/// DTCG theme / model / theme-config source path that must run tokens generate
/// before rebuild.
///
/// Live theme edit = save DTCG + HMR only:
/// - project: `**/themes/**`
/// - project theme config: `sail.config.json(c)`
/// - catalog: `packages/sail/tokens/model/**` (themes + base/dark/density/…)
fn is_theme_source_path(path: &Path) -> bool {
    is_project_theme_path(path) || is_project_sail_config_path(path) || is_catalog_model_path(path)
}

/// Generated delivery under the tokens package — never treat as watch triggers
/// (generate writes these; watching them would loop).
///
/// Keep basenames aligned with `sail_compiler::tokens_generate::GenerateReport`
/// outputs. Hand-authored siblings under `src/` (themes.ts, sail-base.css, …)
/// still HMR. Future generate outputs under `src/generated/` are covered by
/// the directory prefix.
fn is_tokens_generated_delivery(path: &str) -> bool {
    let norm = format!("/{}", path.replace('\\', "/"));
    if !norm.contains("/packages/sail/tokens/src/") {
        return false;
    }
    // Must stay in sync with tokens_generate delivery artifacts.
    const GENERATED_BASENAMES: &[&str] = &[
        "tokens.css",
        "themes.css",
        "products.css",
        "token-registry.json",
    ];
    if let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) {
        if GENERATED_BASENAMES.contains(&name) {
            return true;
        }
    }
    norm.contains("/packages/sail/tokens/src/generated/")
}

/// Dev paint path tokens generate:
///
/// 1. **App** (always) — baked catalog (or monorepo disk model) + project
///    themes → `<project>/.sail/themes/themes.css` (bundler aliases
///    `@ship/tokens/themes.css` here when present).
/// 2. **Package** (monorepo only) — full catalog + project merge →
///    `packages/sail/tokens/src`, gated by
///    [`sail_compiler::tokens_generate::should_generate_package_delivery`]
///    (opt out: `SAIL_TOKENS_PACKAGE_GENERATE=0`). External apps never write
///    package delivery.
///
/// Returns the **app** report (theme_count reflects emit set).
fn run_tokens_generate(
    workspace_root: &Path,
    project_dir: &Path,
) -> Result<sail_compiler::tokens_generate::GenerateReport> {
    // Resolve model in-process (baked catalog when monorepo model is absent).
    // Avoids RPC path confinement rejecting the temp baked-model directory.
    let app_report =
        sail_compiler::tokens_generate::generate_app_delivery(workspace_root, project_dir)
            .map_err(|e| anyhow::anyhow!("tokens generate (app): {e:#}"))?;

    let package_out = workspace_root.join(sail_compiler::tokens_generate::MONOREPO_TOKENS_SRC_REL);
    let mut tokens_css = PathBuf::new();
    let mut products_css = PathBuf::new();
    let mut unions_ts = PathBuf::new();
    let mut token_count = app_report.token_count;

    if sail_compiler::tokens_generate::should_generate_package_delivery(workspace_root) {
        let package_report = sail_compiler::tokens_generate::generate_package_delivery(
            workspace_root,
            Some(project_dir),
        )
        .map_err(|e| anyhow::anyhow!("tokens generate (package): {e:#}"))?;
        tokens_css = package_report.tokens_css;
        products_css = package_report.products_css;
        unions_ts = package_report.unions_ts;
        if token_count == 0 {
            token_count = package_report.token_count;
        }
    }

    Ok(sail_compiler::tokens_generate::GenerateReport {
        token_count,
        theme_count: app_report.theme_count,
        color_count: app_report.color_count,
        path_count: app_report.path_count,
        layers: app_report.layers,
        tokens_css,
        themes_css: app_report.themes_css,
        products_css,
        registry_json: package_out.join("token-registry.json"),
        unions_ts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::EventKind;
    use notify::event::{CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode};

    #[test]
    fn type_gate_force_when_last_tick_was_red_or_overflow() {
        assert!(!force_hmr_type_gate(false, false));
        assert!(force_hmr_type_gate(true, false));
        assert!(force_hmr_type_gate(false, true));
        assert!(force_hmr_type_gate(true, true));
    }

    #[test]
    fn workspace_bake_checkout_detects_this_repo_shape() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!is_workspace_bake_checkout(tmp.path()));
        std::fs::create_dir_all(tmp.path().join("crates/deck/deck-cli")).unwrap();
        std::fs::write(
            tmp.path().join("crates/deck/deck-cli/Cargo.toml"),
            "[package]\nname = \"deck-cli\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("packages/sail/runtime/src")).unwrap();
        assert!(is_workspace_bake_checkout(tmp.path()));
    }

    /// A new component dir (Create), a removed one (Remove), and a rename all
    /// invalidate the component-index memo; a plain content edit does not —
    /// that's the exact split that keeps invalidation event-driven without
    /// re-walking the workspace on every keystroke.
    #[test]
    fn structural_events_invalidate_content_edits_do_not() {
        assert!(is_structural_change(&EventKind::Create(CreateKind::Folder)));
        assert!(is_structural_change(&EventKind::Create(CreateKind::File)));
        assert!(is_structural_change(&EventKind::Remove(RemoveKind::Folder)));
        assert!(is_structural_change(&EventKind::Modify(ModifyKind::Name(
            RenameMode::Any
        ))));
        assert!(!is_structural_change(&EventKind::Modify(ModifyKind::Data(
            DataChange::Any
        ))));
    }

    #[test]
    fn theme_source_paths_include_project_and_catalog() {
        // Modern monorepo + external layout (EXT-1 E4 primary).
        assert!(is_theme_source_path(Path::new(
            "apps/sail-site/themes/custom.json"
        )));
        assert!(is_project_theme_path(Path::new("apps/sail-site/themes")));
        assert!(is_project_theme_path(Path::new("my-app/themes/ship.json")));
        assert!(is_project_theme_path(Path::new("my-app/themes")));
        // Retired tokens/themes layout is not a project theme source.
        assert!(!is_theme_source_path(Path::new(
            "apps/sail-site/tokens/themes/custom.json"
        )));
        assert!(!is_project_theme_path(Path::new("apps/sail-site/tokens")));
        assert!(!is_project_theme_path(Path::new(
            "apps/sail-site/tokens/themes"
        )));
        assert!(is_theme_source_path(Path::new(
            "apps/sail-site/sail.config.jsonc"
        )));
        assert!(is_theme_source_path(Path::new("sail.config.json")));
        assert!(is_project_sail_config_path(Path::new(
            "apps/sail-site/sail.config.json"
        )));
        assert!(!is_project_sail_config_path(Path::new(
            "apps/sail-site/deck.jsonc"
        )));
        assert!(path_suggests_new_theme_watch(
            Path::new("apps/demo/sail.config.jsonc"),
            Path::new("apps/demo")
        ));
        assert!(!path_suggests_new_theme_watch(
            Path::new("apps/demo/tokens"),
            Path::new("apps/demo")
        ));
        assert!(!path_suggests_new_theme_watch(
            Path::new("apps/demo/tokens/themes"),
            Path::new("apps/demo")
        ));
        assert!(path_suggests_new_theme_watch(
            Path::new("apps/demo/themes"),
            Path::new("apps/demo")
        ));
        // Leaf theme edits must not re-queue attach (#15).
        assert!(!path_suggests_new_theme_watch(
            Path::new("apps/demo/tokens/themes/ship.json"),
            Path::new("apps/demo")
        ));
        assert!(!path_suggests_new_theme_watch(
            Path::new("apps/demo/themes/ship.json"),
            Path::new("apps/demo")
        ));
        assert!(!path_suggests_new_theme_watch(
            Path::new("apps/demo/package.json"),
            Path::new("apps/demo")
        ));
        assert!(is_theme_source_path(Path::new(
            "packages/sail/tokens/model/themes/paper.json"
        )));
        assert!(is_theme_source_path(Path::new(
            "packages/sail/tokens/model/base.json"
        )));
        assert!(!is_theme_source_path(Path::new(
            "packages/sail/tokens/src/themes.css"
        )));
        assert!(!is_theme_source_path(Path::new(
            "apps/sail-site/src/sail-docs-rail/sail-docs-rail.sail"
        )));
    }

    #[test]
    fn tokens_delivery_artifacts_are_ignored_as_watch_triggers() {
        assert!(is_tokens_generated_delivery(
            "packages/sail/tokens/src/themes.css"
        ));
        assert!(is_tokens_generated_delivery(
            "packages/sail/tokens/src/tokens.css"
        ));
        assert!(is_tokens_generated_delivery(
            "packages/sail/tokens/src/generated/unions.ts"
        ));
        assert!(!is_tokens_generated_delivery(
            "packages/sail/tokens/model/themes/paper.json"
        ));
        // Hand sources under src/ must still HMR (not treated as delivery).
        assert!(!is_tokens_generated_delivery(
            "packages/sail/tokens/src/themes.ts"
        ));
        assert!(!is_tokens_generated_delivery(
            "packages/sail/tokens/src/sail-base.css"
        ));
    }

    #[test]
    fn theme_write_policy_lives_in_sail_ops() {
        use sail_ops::theme::{is_list_theme_dir, is_read_theme_path, is_write_theme_path};
        // E4 themes/ (project-relative; no apps/ required).
        assert!(is_write_theme_path("themes/custom.json"));
        assert!(!is_write_theme_path("tokens/themes/custom.json"));
        assert!(!is_write_theme_path(
            "apps/sail-site/tokens/themes/custom.json"
        ));
        assert!(is_list_theme_dir("themes"));
        assert!(!is_list_theme_dir("tokens/themes"));
        assert!(!is_list_theme_dir("apps/sail-site/tokens/themes"));
        assert!(!is_list_theme_dir("apps/sail-site/tokens/themes/"));
        assert!(!is_write_theme_path("apps/sail-site/tokens/base.json"));
        assert!(!is_write_theme_path(
            "packages/sail/tokens/model/themes/paper.json"
        ));
        assert!(is_read_theme_path(
            "packages/sail/tokens/model/themes/paper.json"
        ));
        assert!(!is_write_theme_path(
            "apps/sail-site/tokens/themes/../secret.json"
        ));
        assert!(!is_write_theme_path(
            "apps/sail-site/tokens/themes/notes.txt"
        ));
        assert!(!is_write_theme_path("tmp/tokens/themes/x.json"));
    }

    #[test]
    fn project_vs_catalog_theme_paths() {
        assert!(!is_project_theme_path(Path::new(
            "apps/sail-site/tokens/themes/x.json"
        )));
        assert!(!is_catalog_model_path(Path::new(
            "apps/sail-site/tokens/themes/x.json"
        )));
        assert!(is_catalog_model_path(Path::new(
            "packages/sail/tokens/model/themes/paper.json"
        )));
        assert!(!is_project_theme_path(Path::new(
            "packages/sail/tokens/model/themes/paper.json"
        )));
    }
}
