//! `sail` — the Sail framework CLI (user-facing brand).
//!
//! One installable binary covers dev/build/scaffold/tokens/compile/check/lsp/
//! mcp/layout/doctor. Internals still use deck crates (bundler, axum, core);
//! the product path is **Sail**.

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
    about = "Sail — the UI framework CLI (dev, build, scaffold, compile, tokens, check, lsp, mcp, op, layout, render, doctor, version)",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerate the typed route tables for one or more apps.
    ///
    ///   --root <path>  walks <path>/routes/ for file-routed pages, writes
    ///                  <path>/routes.gen.ts. Repeatable; the label is the
    ///                  path's basename. First-party app routes.
    ///
    /// At least one --root is required.
    Routes {
        /// App root holding a `routes/` dir. Walks `<root>/routes/` and
        /// writes `<root>/routes.gen.ts`. The label is the path's basename.
        /// Repeatable. e.g. `sail routes --root apps/shell --root apps/spaces`.
        #[arg(long = "root")]
        root: Vec<PathBuf>,
    },

    /// Generate boilerplate for a new route, component, pack, or theme.
    /// Aliases: `new`, `init` (C-pack scaffold surface).
    #[command(subcommand, visible_alias = "new", visible_alias = "init")]
    Scaffold(Scaffold),

    /// Build a SAIL SPA project to a static output dir.
    Build {
        /// Project root (`package.json` + `routes/` + `public/`). Default: cwd.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Output dir (default: <project>/.deck/).
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
        /// Project root (`routes/` + `package.json` + `public/`). Default: cwd.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Address to bind (host:port). Default `127.0.0.1:7878`.
        #[arg(long, default_value = "127.0.0.1:7878")]
        addr: String,
    },

    /// Emit already-baked target bundles into a directory.
    Emit {
        /// Target name(s). Repeatable.
        #[arg(long = "target", required = true)]
        targets: Vec<String>,
        /// Destination directory (created if absent).
        #[arg(long)]
        out: PathBuf,
    },

    /// Bake all framework + consumer bundles via the in-process Rolldown bundler.
    BuildAssets {
        /// Skip writing the `.deck/framework-stamp` file (tests).
        #[arg(long = "no-stamp")]
        no_stamp: bool,
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

    /// Generate design tokens from the DTCG model (delivery CSS/registry only).
    Tokens {
        /// Model directory. Default: `packages/sail/tokens/model`.
        #[arg(long)]
        model: Option<PathBuf>,
        /// Output directory. Default: `packages/sail/tokens/src`.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Project root for project-theme merge (optional).
        #[arg(long)]
        project: Option<PathBuf>,
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

    /// Language server (stdio) — same protocol as the `sail-lsp` package binary.
    Lsp,

    /// MCP server (stdio) — same protocol as the `sail-mcp` package binary.
    Mcp,

    /// Invoke a sail-ops RPC in-process (`Service/Method`), or `gen` for codegen.
    ///
    /// Same handlers as MCP tools and Connect HTTP under `sail dev`.
    /// Example: `sail op SailTheme/List --json '{"path":"apps/sail-site/tokens/themes"}'`
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
        /// Theme id (e.g. `ship`). Default `ship`.
        #[arg(long, default_value = "ship")]
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
enum Scaffold {
    /// New SAIL route under `routes/<name>/` (a `page.sail`; optional
    /// route-private widget). Deck derives the URL from the directory tree
    /// — no manifest, no host metadata.
    Route {
        /// Route dir name. Lowercase ASCII + hyphens.
        name: String,
        /// URL pattern; defaults to `/<name>`.
        #[arg(long)]
        path: Option<String>,
        /// Optional route-private widget; tag must start with `<name>-`.
        #[arg(long)]
        component: Option<String>,
    },

    /// New SAIL primitive under `packages/components/sail/primitives/src/<tag>/`.
    Component {
        /// Tag name (must start with `sail-`).
        tag: String,
        /// Also include an inline `<controller>` stub in the `.sail`.
        #[arg(long)]
        controller: bool,
    },

    /// New first-party SAIL component pack under `packages/components/<name>/`.
    /// Generates package.json, tsconfig.json, the umbrella `src/index.ts`,
    /// and a starter `<name>-example` component.
    Pack {
        /// Pack name (lowercase + hyphens). Becomes `@ship/<name>`.
        name: String,
    },

    /// Seed project theme SSOT: copy one Sail default theme into
    /// `<project>/tokens/themes/` and write a thin `sail.config.jsonc`
    /// theme section when absent (C5). Apps do not ship the full catalog.
    Theme {
        /// Project root (`routes/` + `package.json`). Default: cwd.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Catalog theme id to copy (default: ship — modes-in-file primary).
        #[arg(long)]
        id: Option<String>,
    },
}

fn main() {
    // Surface bundler diagnostics (e.g. a view referencing a component whose
    // pack isn't declared in `deck.jsonc packs`) to stderr. Honors RUST_LOG;
    // floors at `warn` so the diagnostic always shows even when unset.
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
        Command::Routes { root } => {
            let cwd = std::env::current_dir().context("cwd")?;
            let resolve = |p: PathBuf| {
                if p.is_absolute() { p } else { cwd.join(p) }
            };
            if root.is_empty() {
                anyhow::bail!("specify --root=<path> at least once");
            }
            let mut routes_specs = Vec::new();
            for s in &root {
                let r = resolve(s.clone());
                let label = r
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| r.display().to_string());
                routes_specs.push(deck_bundler::routes::RouteGenSpec {
                    dir: r.join("routes"),
                    out: r.join("routes.gen.ts"),
                    label,
                    entry_bootstrap: None,
                });
            }
            let n_routes = deck_bundler::routes::generate_routes(&routes_specs)?;
            println!("{brand} routes → {n_routes} first-party route(s)");
            Ok(())
        }
        Command::Scaffold(Scaffold::Route {
            name,
            path,
            component,
        }) => scaffold::run_route(name, path, component),
        Command::Scaffold(Scaffold::Component { tag, controller }) => {
            scaffold::run_component(tag, controller)
        }
        Command::Build {
            project,
            out,
            bundle_framework,
            no_minify,
            typecheck,
            base,
        } => build(project, out, bundle_framework, !no_minify, typecheck, base),
        Command::Scaffold(Scaffold::Pack { name }) => scaffold::run_pack(name),
        Command::Scaffold(Scaffold::Theme { project, id }) => scaffold::run_theme(project, id),
        Command::Emit { targets, out } => emit(targets, out),
        Command::BuildAssets { no_stamp } => build_assets(no_stamp),
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
        Command::Check { input, fix } => peer::check(input, fix),
        Command::Lsp => peer::lsp(),
        Command::Mcp => {
            peer::mcp();
            Ok(())
        }
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
/// closure — see `deck_bundler::emit`).
fn emit(targets: Vec<String>, out: PathBuf) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let out = if out.is_absolute() {
        out
    } else {
        cwd.join(out)
    };
    let repo_root = find_repo_root(&cwd)?;
    let cfg = deck_bundler::DeckConfig::defaults();
    let layout = deck_bundler::targets::BundleLayout::with_config(repo_root, None, cfg);
    let n = deck_bundler::emit::emit_targets(&layout, &targets, &out)?;
    println!(
        "{} emit → {} files into {}",
        binary_brand(),
        n,
        out.display()
    );
    Ok(())
}

fn build_assets(no_stamp: bool) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let repo_root = find_repo_root(&cwd)?;

    let cfg = deck_bundler::DeckConfig::defaults();
    let bundler = deck_bundler::Bundler::start(repo_root.clone(), None, cfg.clone());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    rt.block_on(bundler.build_all())
        .map_err(|e| anyhow::anyhow!("build_all failed: {e}"))?;

    // Build the layout once — used for the framework verify+stamp, the pack
    // CEM step, and the consumer guard below. assets_dir_for("sail-runtime")
    // is now .deck/framework/ (the canonical bake target per Recommendation
    // B1).
    let layout = deck_bundler::targets::BundleLayout::with_config(repo_root.clone(), None, cfg);

    // Verify + stamp the framework singletons BEFORE the pack CEM step
    // (deck-deploy-gap): `deck-core/build.rs` needs these 4 files to build at
    // all (an `include_str!` bake), but pack CEM generation can legitimately
    // fail on an UNRELATED component's lint violations — that must not take
    // down the framework bake `build.rs` depends on. Verify every framework
    // singleton that RESOLVED for this repo is non-empty.
    // build_all returns Err on per-target failure, but a swallowed empty-write
    // wouldn't surface there — keep this as defense in depth so the daemon can
    // never ship empty framework bytes. The set is derived from the bundler's
    // own target list (node-resolved): a singleton not installed in this repo is
    // skipped by the bundler, so a hardcoded list would false-fail a consumer
    // bake that doesn't depend on every singleton.
    let fw_targets: Vec<_> = layout
        .all_targets()
        .into_iter()
        .filter(|t| deck_bundler::targets::is_framework_singleton(&t.name))
        .collect();
    for t in &fw_targets {
        let path = layout.assets_dir_for(&t.name).join(&t.out_file);
        let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if len == 0 {
            bail!(
                "framework bundle missing or empty after bake: {}",
                path.display()
            );
        }
        println!("  {}: {len} bytes", t.out_file);
    }
    if fw_targets.is_empty() {
        println!("  (no framework singletons resolved from this repo — none baked here)");
    }

    // Stamp each freshly-baked framework bundle with the content hash of the
    // TS sources it was built from (deck-deploy-gap): a leading `/*! ... */`
    // comment header IN the bundle (so `deck-core`'s `include_str!` bakes it
    // along with the code — no sidecar read needed to check the RUNNING
    // binary) plus a `manifest.json` sidecar (for tooling that wants the
    // stamps without parsing JS). `sail dev` recomputes the current source
    // hash at startup and compares against the header baked into its own
    // binary; a mismatch means the binary is older than its sources.
    if !fw_targets.is_empty() {
        let stamps = deck_bundler::stamp::current_framework_stamps(&repo_root)
            .context("hashing framework source trees for the bake stamp")?;
        let mut manifest = serde_json::Map::new();
        for t in &fw_targets {
            let Some(stamp) = stamps.iter().find(|s| s.bundle == t.out_file) else {
                // No source root under this repo for this singleton (an
                // external consumer vendoring a prebuilt framework) — nothing
                // to stamp against, leave the bundle unstamped.
                continue;
            };
            let path = layout.assets_dir_for(&t.name).join(&t.out_file);
            let body = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {} to stamp it", path.display()))?;
            // Re-running build-assets must not pile up headers.
            let body = match deck_core::parse_baked_stamp(&body) {
                Some(_) => body.split_once('\n').map(|x| x.1).unwrap_or("").to_string(),
                None => body,
            };
            let stamped = format!(
                "{}{body}",
                deck_bundler::stamp::format_stamp_header(&stamp.hash)
            );
            deck_bundler::atomic::write(&path, stamped)
                .with_context(|| format!("writing stamped {}", path.display()))?;
            manifest.insert(
                t.out_file.clone(),
                serde_json::Value::String(stamp.hash.clone()),
            );
        }
        if !manifest.is_empty() {
            let manifest_dir = layout.assets_dir_for(&fw_targets[0].name);
            let manifest_path = manifest_dir.join("manifest.json");
            deck_bundler::atomic::write(
                &manifest_path,
                serde_json::to_string_pretty(&serde_json::Value::Object(manifest))?,
            )
            .with_context(|| format!("writing {}", manifest_path.display()))?;
            println!("  manifest: {}", manifest_path.display());
        }

        // Touch the framework bake stamp used by `deck-core/build.rs` to
        // decide when to re-stage bundles into `assets/`. The content is
        // irrelevant; only the mtime matters. Tests pass `--no-stamp` so they
        // can run `deck build-assets` without re-triggering this build script
        // while downstream doctests are queued.
        if !no_stamp {
            let stamp_path = deck_config::deck_out_path(&repo_root).join("framework-stamp");
            deck_bundler::atomic::write(
                &stamp_path,
                // Empty content keeps the committed stamp unmodified in git;
                // Cargo's rerun-if-changed tracks mtime, not content.
                b"",
            )
            .with_context(|| format!("writing {}", stamp_path.display()))?;
            println!("  stamp: {}", stamp_path.display());
        }
    }

    // Materialize each component pack's standard `dist/custom-elements.json`
    // (CEM) from its source. Run once here (the framework build), so per-app
    // builds aggregate the pre-written manifests into `<app>/components.gen.ts`
    // (`generate_components_gen`). Also the file every pack declares for
    // external tooling (Storybook / VS Code / lit-analyzer).
    // A pack that fails to compile fails the build. Printing and continuing is
    // how un-migrated packs stayed invisible behind an exit-0 asset bake.
    // Runs AFTER the framework verify+stamp above (deck-deploy-gap): a
    // component pack's lint failure is real and must still fail the overall
    // command, but it must never block the framework bundles `build.rs`
    // needs from being staged and stamped first.
    let n =
        deck_bundler::cem::write_all_pack_cems(&repo_root).context("pack manifest generation")?;
    println!("  cem: {n} component pack manifest(s) written");

    // Consumer guard: the discovered consumer bundles (per-package
    // `deck.bundles`) have no build.rs guard — verify each is non-empty as
    // defense in depth.
    // Use BundleLayout to derive the exact output path per target (same routing
    // logic the bundler uses) so the guard is always in sync with reality.
    //
    // Dev-loop skip: when DECK_DEV_SKIP_LAZY=1, the same lazy targets the
    // bundler skipped above are also skipped by the guard (otherwise a
    // successful dev build would always fail the guard for the skipped
    // targets). The lazy flag rides on the BundleTarget (from the package's
    // self-declaration) — no central-config lookup.
    let skip_lazy = std::env::var("DECK_DEV_SKIP_LAZY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    for t in layout.consumer_targets() {
        if skip_lazy && t.lazy {
            println!("  {}: skipped (DECK_DEV_SKIP_LAZY=1, lazy)", t.name);
            continue;
        }
        let out_dir = layout.assets_dir_for(&t.name);
        let out_path = out_dir.join(&t.out_file);
        let len = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
        if len == 0 {
            bail!(
                "consumer bundle missing or empty after bake: {} \
                 (target `{}` failed — check bundler logs)",
                out_path.display(),
                t.name
            );
        }
        println!("  {}: {len} bytes", t.name);
    }

    println!(
        "{} build-assets: bundles baked via in-process Rolldown",
        binary_brand()
    );
    println!("note: lucide-icons.json is committed source; regen via `cargo xtask gen-icons`");
    Ok(())
}

/// deck-deploy-gap: the incident this closes is a stale `sail-runtime.js`
/// baked via `include_str!` served against post-rename compiled views —
/// every setup binding came back undefined, the site rendered nothing, and
/// the only symptom was a silent console error. The manual cure was a
/// 3-step dance (`build-assets`, `cargo build -p deck-cli`, restart) that
/// nothing enforced.
///
/// Called once at `sail dev` startup: recomputes the current content hash of
/// each framework singleton's TS source tree and compares it against the
/// hash stamped into THIS binary's baked bundle (the leading `/*! ... */`
/// comment `build_assets` writes — see `deck_bundler::stamp`). Three
/// outcomes:
///   - fresh (hashes match): silent, `sail dev` proceeds.
///   - stale AND regeneration is possible (a real Cargo workspace + `cargo`
///     on PATH, and we haven't already tried once this restart cycle): log
///     loud, run `build_assets()` in-process to re-stage `.deck/framework/`
///     with a fresh stamp and update `.deck/framework-stamp`, then
///     `cargo run -p deck-cli -- dev …` — cargo picks up the changed stamp
///     via `build.rs`'s `cargo:rerun-if-changed`, rebuilds `deck-core`
///     (re-baking the fresh, now-stamped bundle) and `deck-cli`, and launches
///     the new binary with the same args. This process then exits with the
///     child's status — the restart IS the "then continue."
///   - stale AND regeneration is impossible (no Cargo workspace / no cargo /
///     already tried once and still mismatched): HARD FAIL, naming the exact
///     commands. Never serve a mismatched runtime silently.
fn ensure_framework_fresh(repo_root: &Path, project: &Path, addr: &str) -> Result<()> {
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

    const REGEN_CMDS: &str = "cargo run -p deck-cli -- build-assets && cargo build -p deck-cli";
    let repo_has_cargo = repo_root.join("Cargo.toml").is_file();
    let cargo_available = std::process::Command::new("cargo")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let already_tried = std::env::var_os("DECK_DEV_STALE_REGEN_ATTEMPTED").is_some();

    if !repo_has_cargo || !cargo_available {
        anyhow::bail!(
            "sail dev: baked framework bundle(s) are STALE and cannot be regenerated from here \
             (no Cargo workspace at {} / no `cargo` on PATH):\n  {}\n\
             Fix: from the sail monorepo, run `{REGEN_CMDS}`, then restart `sail dev`.",
            repo_root.display(),
            stale.join("\n  ")
        );
    }
    if already_tried {
        anyhow::bail!(
            "sail dev: baked framework bundle(s) are STILL stale after an automatic \
             regenerate-and-restart:\n  {}\n\
             The stamp may not be reproducible (e.g. a source tree that changes on every \
             read). Fix manually: `{REGEN_CMDS}`, then restart `sail dev`.",
            stale.join("\n  ")
        );
    }

    eprintln!(
        "sail dev: STALE framework bundle(s) detected — regenerating automatically:\n  {}",
        stale.join("\n  ")
    );
    build_assets(false).context("sail dev: automatic build-assets (staleness regen) failed")?;

    eprintln!("sail dev: rebuilding + restarting with the fresh bake...");
    let status = std::process::Command::new("cargo")
        .current_dir(repo_root)
        .env("DECK_DEV_STALE_REGEN_ATTEMPTED", "1")
        .args(["run", "--quiet", "-p", "deck-cli", "--", "dev", "--project"])
        .arg(project)
        .args(["--addr", addr])
        .status()
        .context("sail dev: spawning `cargo run -p deck-cli -- dev` for the staleness restart")?;
    std::process::exit(status.code().unwrap_or(1));
}

/// Resolve the workspace root for a build from `start` (the project dir).
///
/// In the ship monorepo the root is the topmost `Cargo.toml` (the Cargo and
/// bun workspaces coincide there). But deck is a GENERAL framework: a consumer
/// can be a pure-JS app with no Cargo workspace at all (e.g. ship-ops), and
/// deck must build it too. So when there is no topmost `Cargo.toml` we fall
/// back to the project dir itself as the root. The root is used for `@ship/*`
/// workspace-source discovery and as the DEFAULT framework-assets origin; an
/// external consumer overrides the latter with `DECK_FRAMEWORK_ASSETS_DIR` and
/// resolves the framework specifiers as importmap externals (they never need
/// source). A Cargo workspace is therefore NOT required.
///
/// NOTE: for a pure-JS bun/npm workspace built from an app subdir
/// (`apps/demo`), `start` is the app dir, NOT the workspace root —
/// framework + node_modules resolution must key off the app dir (its per-app
/// `node_modules`), but `@ship/<pkg>` SIBLING-pack discovery must reach the
/// enclosing workspace. That broadening lives in
/// `catalog::component_package_roots` (it unions the enclosing JS workspace via
/// [`deck_bundler::paths::topmost_workspace_root`]), NOT here — so this root
/// stays the project dir and framework resolution is unchanged.
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    Ok(deck_bundler::paths::topmost_cargo_root(start).unwrap_or_else(|| start.to_path_buf()))
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

    let project = abs(&project)
        .canonicalize()
        .with_context(|| format!("project {}", project.display()))?;
    // The project is the dev's authored source tree (`package.json`,
    // `routes/`, `public/`). The default out dir lives INSIDE the
    // project as `<project>/.deck/` — a sibling of `routes/` and
    // `public/`, not above them. The bundler writes the generated
    // `index.html`, the entry chunk, and the framework surface there.
    let out_dir = out
        .map(|o| abs(&o))
        .unwrap_or_else(|| deck_config::deck_out_path(&project));
    let repo_root = find_repo_root(&project)?;
    // The framework singletons are resolved by NODE RESOLUTION from the
    // project's `node_modules` (the bundler's `framework_singletons` /
    // `bundle_owned_framework`) — `--bundle-framework` builds them from source.
    // There is no prebuilt-asset staging dir and no `DECK_FRAMEWORK_ASSETS_DIR`
    // crutch: the framework comes from wherever it was installed (the registry,
    // a workspace symlink), so a build runs identically inside or outside the
    // sail monorepo.

    // Load the host's view-discovery config (packs scope). Consumer bundles
    // are discovered from per-package `deck.bundles`, not read here. L1 (deck)
    // has no per-app config — shipd's apps.jsonc owns the app list; the
    // bundler is per-app and reads its own project root.
    let cfg = deck_bundler::DeckConfig::defaults();

    // The app graph compiles its own `.sail` in-process via the linked
    // sail-compiler crate (#255) — no binary to locate.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    let b = rt
        .block_on(deck_bundler::app::bundle_app_with_base(
            &repo_root,
            &cfg.packs,
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
    // Resolve the project root. Canonicalize so symlinks/bare relative paths
    // collapse to a single absolute path the bundler and resolver can key
    // off.
    let cwd = std::env::current_dir().context("cwd")?;
    let project_path = if project.is_absolute() {
        project.clone()
    } else {
        cwd.join(&project)
    };
    let project_dir = project_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_path.display()))?;
    // Bail up front if the path isn't a directory — keeps the later bundler
    // diagnostics readable.
    if !project_dir.is_dir() {
        anyhow::bail!("{} is not a directory", project_dir.display());
    }
    // The real workspace root (component/workspace resolution) is found by
    // walking up from the project; the project dir itself is one app.
    let workspace_root = find_repo_root(&project_dir)?;

    // deck-deploy-gap: never serve a framework bundle staler than its
    // sources. Regenerates + restarts (see `ensure_framework_fresh`) or
    // hard-fails naming the exact fix — checked BEFORE the server (and
    // BEFORE the tokio runtime below exists): `ensure_framework_fresh`'s
    // regen path calls `build_assets()`, which spins up its OWN tokio
    // runtime — `block_on` from inside an already-running runtime panics
    // ("Cannot start a runtime from within a runtime"), so this must run on
    // a plain sync thread, not inside `runtime.block_on(async move { .. })`.
    ensure_framework_fresh(&workspace_root, &project_dir, &addr)?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    runtime.block_on(async move {
        // deck-core's resolution is mount-scoped: the project root is carried
        // per-request through the `Resolution` Extension layered below (not a
        // process global), so `serve_index` / `serve_app_*` / `spa_fallback`
        // resolve the app's built assets (`.deck/index.html`, `/<name>/<file>`)
        // against THIS mount's root.
        let cfg = deck_config::DeckConfig::defaults();

        // Cold-start tokens generate when the project has theme SSOT on disk
        // (tokens/themes/*.json or sail.config theme section). Without this,
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
        // edit; without an initial build the project emits no `.deck/`
        // output and EVERY page 404s — the gap a runtime-only smoke test
        // (just `/_deck/runtime.js`) misses. `bundle_framework = false`:
        // sail dev serves the framework singletons itself at `/_deck/*`,
        // so the app graph keeps them external (no per-app copy).
        let out_dir = deck_config::deck_out_path(&project_dir);
        let build = deck_bundler::app::bundle_app(
            &workspace_root,
            &cfg.packs,
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
        // sail.config serve keys for injectGlobals / render / entry —
        // name/route/source are host-bound here, never the file's.
        let mut app_cfg = deck_config::AppServeConfig::load_from_project(&project_dir)
            .map_err(|e| anyhow::anyhow!("app serve config: {e}"))?;
        // SSR engine: only spin up ship-js (V8) when this app opts into SSR
        // (`sail.config { "render": "ssr" }`). A CSR app (the default) pays no
        // V8 cost. Standalone dev has no live data port, so the engine renders
        // with NoopData (None). The engine is a MOUNT-SCOPED handle layered as
        // an Extension below — not a process-global install. If it can't start,
        // log + continue; the serve path falls back to CSR (`renderer = None`).
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
        // primary project is NOT Studio itself, also bundle + serve it at
        // `/_studio` for the overlay bridge (TOOL-4). Fail-soft: missing or
        // broken studio build never blocks the primary app.
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
            match deck_bundler::app::bundle_app(
                &workspace_root,
                &cfg.packs,
                &studio_dir,
                &studio_out,
                false,
                false,
            )
            .await
            {
                Ok(studio_build) => {
                    let mut studio_cfg =
                        deck_config::AppServeConfig::load_from_project(&studio_dir)
                            .unwrap_or_default();
                    studio_cfg.name = studio_build.url_namespace.clone();
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
                Err(e) => {
                    eprintln!("sail dev: studio overlay build skipped: {e:#}");
                }
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

        // ── HMR (foundational, full-document) ────────────────────────────────
        // The bundler's own watcher rebuilds TARGETS (views, framework). The
        // SPA is the one-shot `bundle_app` ENTRY — not a target — so `routes/` +
        // `src/` + workspace pack edits emit nothing through it. sail dev owns
        // this loop: a notify watcher on authored source re-runs `bundle_app`
        // and emits a `module-changed` (target `app`) on the bundler's firehose,
        // which `serve_bundle_hmr` streams to the live-reload client → full-
        // document reload. (Module-granular reload is the next layer.)
        //
        // Watch roots:
        // - project `routes/` + `src/` + `public/` + `tokens/` (never project
        //   root / `.deck/` — regenerated `routes.gen.ts` / out dir must not
        //   self-trigger). `tokens/themes/**` is the project theme SSOT.
        // - workspace component pack `src/` trees (and package root if no `src/`)
        //   so edits under `packages/components/...` HMR like local rails —
        //   previously only the app tree was watched and pack edits needed a
        //   full `sail dev` restart.
        // - monorepo catalog themes: `packages/sail/tokens/model/**` (DTCG SSOT;
        //   generated `src/tokens.css` / `themes.css` are NOT watched — those
        //   are outputs of `sail-compiler tokens`).
        //
        // Live theme edit path (only supported path):
        //   save DTCG under tokens/themes or model/themes (or project sail.config
        //   theme section) → dual tokens generate:
        //     package: full catalog + project → packages/sail/tokens/src
        //     app:     T0 effective set → <project>/.deck/themes/themes.css (B1)
        //   → HMR swaps CSS. No client IR / :root setProperty.
        let hmr_packs = cfg.packs.clone();
        let (hmr_tx, mut hmr_rx) = tokio::sync::mpsc::channel::<()>(8);
        // Set when a watched path is theme/model/config source; rebuild loop runs
        // dual `tokens_generate` before `bundle_app` so delivery CSS stays
        // regenerate-only (package full + app-local filtered themes).
        // `project_theme_edit` / `catalog_model_edit` distinguish log lines
        // (project vs catalog SSOT) after generate.
        let need_tokens_generate = Arc::new(AtomicBool::new(false));
        let project_theme_edit = Arc::new(AtomicBool::new(false));
        let catalog_model_edit = Arc::new(AtomicBool::new(false));
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
            let project_for_cb = project_dir.clone();
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
                // `.sail.d.ts` companions, `.compiled.js`, pack `dist/`,
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
                            || s.contains("/node_modules/")
                            // Project-root NonRecursive watch sees `.deck` create;
                            // never rebuild on deck out dir (self-trigger).
                            || s.contains("/.deck/")
                            || s.ends_with("/.deck")
                            || p.file_name().and_then(|n| n.to_str()) == Some(".deck")
                            // The atomic-publish STAGE sibling (`.deck.stage`,
                            // `.deck.stage.<suffix>`) is created+removed by every
                            // rebuild at project-root level — unfiltered, each
                            // rebuild's own stage create/remove fired the next
                            // rebuild: a self-sustaining Create/Remove loop at
                            // full CPU after the first edit (the shipd-class spin).
                            || s.contains("/.deck.stage")
                            || p.file_name()
                                .and_then(|n| n.to_str())
                                .is_some_and(|n| n.starts_with(".deck.stage"))
                            || is_tokens_generated_delivery(&s)
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
                // below; dropping on a full channel is fine (the next tick rebuilds).
                let _ = tx.try_send(());
            })
            .map_err(|e| anyhow::anyhow!("sail dev: HMR watcher init: {e}"))?;
            let mut watched: Vec<PathBuf> = Vec::new();
            for sub in ["routes", "src", "public", "tokens"] {
                let dir = project_dir.join(sub);
                if dir.is_dir() {
                    watched.push(dir);
                }
            }
            // Workspace component packs used by this monorepo (and enclosing
            // workspace). Prefer `src/` so we don't pick up pack `dist/` churn.
            let pack_roots = deck_bundler::catalog::component_package_roots(&workspace_root);
            for pack_root in pack_roots.values() {
                if pack_root == &project_dir {
                    continue; // already covered by project src/
                }
                let src = pack_root.join("src");
                if src.is_dir() {
                    watched.push(src);
                } else if pack_root.is_dir() {
                    watched.push(pack_root.clone());
                }
            }
            // Compiler stdlib `.sail` (buttons, layout keywords, …) — rare edits
            // but without this a stdlib tweak also needed a full restart.
            let stdlib = workspace_root.join("crates/sail/compiler/stdlib");
            if stdlib.is_dir() {
                watched.push(stdlib);
            }
            // Catalog DTCG SSOT (framework themes + base/dark/density/…).
            // Live edit = save JSON here (or apps/*/tokens/themes) → generate → HMR.
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
            // `.deck/` write storms must not self-trigger (also filtered above).
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
                 {n_cfg} theme-config (app routes/src/public/tokens + packs + \
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
            tokio::spawn(async move {
                while hmr_rx.recv().await.is_some() {
                    // Debounce: settle the burst, then drain coalesced ticks.
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    while hmr_rx.try_recv().is_ok() {}
                    // Theme/model/config edit → dual generate (T1 B1):
                    // package full catalog + app-local .deck/themes/themes.css.
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
                    match deck_bundler::app::bundle_app(&ws, &hmr_packs, &proj, &od, false, false).await {
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
        // Theme list/read/write/fork/generate replace legacy `/_deck/dev/*` FS.
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

/// Project theme SSOT: `**/tokens/themes/**` (and the `tokens` / `tokens/themes`
/// dirs themselves so Create events from mid-session scaffold fire generate).
fn is_project_theme_path(path: &Path) -> bool {
    let norm = format!("/{}", path.to_string_lossy().replace('\\', "/"));
    norm.contains("/tokens/themes/")
        || norm.ends_with("/tokens/themes")
        || norm.ends_with("/tokens")
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
/// `deck scaffold theme` creates `tokens/` and/or `sail.config` after startup.
///
/// Only **tree roots / config files**, not routine leaf edits under
/// `tokens/themes/*.json` (#15). Callers should also gate on Create/Rename.
fn path_suggests_new_theme_watch(path: &Path, project_dir: &Path) -> bool {
    if is_project_sail_config_path(path) {
        return true;
    }
    // tokens or tokens/themes directory only — not themes/<id>.json leaves.
    let norm = format!("/{}", path.to_string_lossy().replace('\\', "/"));
    let is_tokens_root = norm.ends_with("/tokens") || norm.ends_with("/tokens/themes");
    if !is_tokens_root {
        return false;
    }
    // Prefer paths under this project when we can resolve parents.
    let Some(parent) = path.parent() else {
        return false;
    };
    // `…/tokens` → parent is project; `…/tokens/themes` → parent is tokens,
    // grandparent is project.
    let project_candidate = if norm.ends_with("/tokens/themes") {
        parent.parent()
    } else {
        Some(parent)
    };
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
/// - project: `**/tokens/themes/**`
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
    const GENERATED_BASENAMES: &[&str] = &["tokens.css", "themes.css", "token-registry.json"];
    if let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) {
        if GENERATED_BASENAMES.contains(&name) {
            return true;
        }
    }
    norm.contains("/packages/sail/tokens/src/generated/")
}

/// Dual generate for `sail dev` paint path (T1 B1):
///
/// 1. **Package** — full catalog + project merge → `packages/sail/tokens/src`
///    (shared library; thin `theme.enabled` must not rewrite workspace CSS).
/// 2. **App** — T0 effective set only → `<project>/.deck/themes/themes.css`
///    (foreign dir survives SPA prune) so the bundler aliases
///    `@ship/tokens/themes.css` to the filtered app sheet.
///
/// Returns the **app** report when available (theme_count reflects emit set);
/// falls back to package report fields for logging.
fn run_tokens_generate(
    workspace_root: &Path,
    project_dir: &Path,
) -> Result<sail_compiler::tokens_generate::GenerateReport> {
    let ctx = sail_ops::Ctx::for_dev(workspace_root, project_dir);
    let model_dir = workspace_root.join("packages/sail/tokens/model");
    let package_out = workspace_root.join("packages/sail/tokens/src");
    let app_out = sail_compiler::tokens_generate::app_themes_out_dir(project_dir);

    // 1) Package (full catalog + project merge).
    let package_args = serde_json::json!({
        "mode": "package",
        "workspace": workspace_root.display().to_string(),
        "project": project_dir.display().to_string(),
        "model": model_dir.display().to_string(),
        "out": package_out.display().to_string(),
    });
    let package_v = sail_ops::call_rpc("SailTheme", "Generate", &package_args, &ctx)
        .map_err(|e| anyhow::anyhow!("tokens generate (package): {e}"))?;

    // 2) App-local filtered themes (B1).
    let app_args = serde_json::json!({
        "mode": "app",
        "workspace": workspace_root.display().to_string(),
        "project": project_dir.display().to_string(),
        "model": model_dir.display().to_string(),
        "out": app_out.display().to_string(),
    });
    let app_v = sail_ops::call_rpc("SailTheme", "Generate", &app_args, &ctx)
        .map_err(|e| anyhow::anyhow!("tokens generate (app): {e}"))?;

    // Prefer app report for paint-path logging (emit set story).
    Ok(sail_compiler::tokens_generate::GenerateReport {
        token_count: app_v["token_count"]
            .as_u64()
            .or_else(|| package_v["token_count"].as_u64())
            .unwrap_or(0) as usize,
        theme_count: app_v["theme_count"].as_u64().unwrap_or(0) as usize,
        color_count: 0,
        path_count: 0,
        layers: Vec::new(),
        tokens_css: PathBuf::from(package_v["tokens_css"].as_str().unwrap_or_default()),
        themes_css: PathBuf::from(app_v["themes_css"].as_str().unwrap_or_default()),
        registry_json: package_out.join("token-registry.json"),
        unions_ts: PathBuf::from(package_v["unions_ts"].as_str().unwrap_or_default()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::EventKind;
    use notify::event::{CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode};

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
        assert!(is_theme_source_path(Path::new(
            "apps/sail-site/tokens/themes/custom.json"
        )));
        assert!(is_project_theme_path(Path::new("apps/sail-site/tokens")));
        assert!(is_project_theme_path(Path::new(
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
        assert!(path_suggests_new_theme_watch(
            Path::new("apps/demo/tokens"),
            Path::new("apps/demo")
        ));
        assert!(path_suggests_new_theme_watch(
            Path::new("apps/demo/tokens/themes"),
            Path::new("apps/demo")
        ));
        // Leaf theme edits must not re-queue attach (#15).
        assert!(!path_suggests_new_theme_watch(
            Path::new("apps/demo/tokens/themes/ship.json"),
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
        assert!(is_write_theme_path(
            "apps/sail-site/tokens/themes/custom.json"
        ));
        assert!(is_list_theme_dir("apps/sail-site/tokens/themes"));
        assert!(is_list_theme_dir("apps/sail-site/tokens/themes/"));
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
        assert!(is_project_theme_path(Path::new(
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
