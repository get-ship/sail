//! `deck` — the deck framework CLI (the `next` / `vite` analog).
//!
//! deck is the framework; an app (the hub `apps/shell`, a space scaffold) is
//! built *with* deck via `deck build`, just as `next build` builds a Next app.
//! This is why the command lives here and not in the `ship` app CLI. Backed by
//! `deck-bundler` (Rolldown). `.sui` sources must be compiled first
//! by the deck bundler; `deck dev` / `deck serve` are future commands.

mod scaffold;
mod typecheck;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

use deck_axum::DaemonMode;

#[derive(Parser)]
#[command(name = "deck", about = "The deck framework CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerate the typed route tables for one or more apps. Two surfaces
    /// per invocation:
    ///
    ///   --root <path>  walks <path>/routes/ for file-routed views, writes
    ///                  <path>/routes.gen.ts. Repeatable; the label is the
    ///                  path's basename. First-party app routes.
    ///   --views <path> walks <path>/<name>/manifest.jsonc, writes
    ///                  <path>/routes.gen.ts (router fields) AND
    ///                  <path>/meta.gen.ts (host fields). The split is
    ///                  the contract: the router is host-agnostic, the
    ///                  host reads the meta. Repeatable.
    ///
    /// At least one of --root or --views is required.
    Routes {
        /// App root holding a `routes/` dir. Walks `<root>/routes/` and
        /// writes `<root>/routes.gen.ts`. The label is the path's basename.
        /// Repeatable. e.g. `deck routes --root apps/shell --root apps/spaces`.
        #[arg(long = "root")]
        root: Vec<PathBuf>,
        /// Views dir holding `<name>/manifest.jsonc` entries. Walks the
        /// dir, writes `<views>/routes.gen.ts` (router) and
        /// `<views>/meta.gen.ts` (host). Repeatable. e.g.
        /// `deck routes --views views`.
        #[arg(long = "views")]
        views: Vec<PathBuf>,
    },

    /// Generate boilerplate for a new view or primitive component.
    #[command(subcommand)]
    Scaffold(Scaffold),

    /// Build a SUI SPA project (the hub, a space scaffold) to a static
    /// output dir. The project's source tree holds `package.json` (the
    /// manifest: the bundler reads `name` for the page <title>), `routes/`
    /// (the file-routed chrome), and `public/` (static assets). The
    /// bundler generates `index.html` and `routes.gen.ts` (the entry).
    Build {
        /// Project root (the dev's deck project: `package.json` + `routes/`
        /// + `public/`). Default: cwd.
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
        /// After the build, type-check the app's `.ts` sources (authored +
        /// generated) with `tsc --noEmit` over an on-the-fly tsconfig.
        /// Opt-in: off by default; fails the command on type errors.
        #[arg(long)]
        typecheck: bool,
    },

    /// Run a standalone dev server for a SUI project — the framework
    /// standing on its own, no shipd, no host adapter. Boots the
    /// bundler FS watcher (HMR) and serves the deck router on
    /// `127.0.0.1:<port>`. `Ctrl-C` triggers a graceful bundler
    /// shutdown before the process exits.
    Dev {
        /// Project root (the dev's deck project: `routes/` + `package.json`
        /// + `public/`). Default: cwd.
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Address to bind (host:port). Default `127.0.0.1:7878`.
        #[arg(long, default_value = "127.0.0.1:7878")]
        addr: String,
        /// Use the in-memory `DemoStore` with sample data instead of the
        /// no-op data port. Opt-in; off by default.
        #[arg(long)]
        demo: bool,
    },

    /// Emit already-baked target bundles (plus their lazy-chunk closure) into
    /// a directory. A consumer (e.g. a static site build) names deck targets;
    /// deck resolves their output paths and copies the closure. No consumer
    /// ever hardcodes deck's internal output layout. Run after `build-assets`.
    Emit {
        /// Target name(s): framework singletons (sui-runtime, sui-dom,
        /// sui-sdk-client) or discovered consumer bundles (sui-marketing,
        /// sui-arcade, ship-code-editor, …) — the names declared in each
        /// package's `deck.bundles`. Repeatable.
        #[arg(long = "target", required = true)]
        targets: Vec<String>,
        /// Destination directory (created if absent).
        #[arg(long)]
        out: PathBuf,
    },

    /// Bake all framework + consumer bundles via the in-process Rolldown
    /// bundler. Consumer bundles are discovered from per-package
    /// `deck.bundles`; framework singletons are hardcoded in deck-bundler.
    /// Honesty guards verify every framework bundle is non-empty after the
    /// bake — a silently-failed target cannot ship empty bytes.
    BuildAssets,
}

#[derive(Subcommand)]
enum Scaffold {
    /// New SUI view under `views/<name>/` (page.sui + controller.ts +
    /// style.css + manifest.jsonc; optional view-private widget).
    View {
        /// View dir name + manifest `name` field. Lowercase ASCII + hyphens.
        name: String,
        /// URL pattern; defaults to `/<name>`.
        #[arg(long)]
        path: Option<String>,
        /// Optional view-private widget; tag must start with `<name>-`.
        #[arg(long)]
        component: Option<String>,
    },

    /// New SUI primitive under `packages/components/sui/primitives/src/<tag>/`.
    Component {
        /// Tag name (must start with `sui-`).
        tag: String,
        /// Also write a paired `<tag>.controller.ts` stub.
        #[arg(long)]
        controller: bool,
    },

    /// New first-party SUI component pack under `packages/components/<name>/`.
    /// Generates package.json, tsconfig.json, the umbrella `src/index.ts`,
    /// and a starter `<name>-example` component.
    Pack {
        /// Pack name (lowercase + hyphens). Becomes `@ship/<name>`.
        name: String,
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
    if let Err(e) = run() {
        eprintln!("deck: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Routes { root, views } => {
            let cwd = std::env::current_dir().context("cwd")?;
            let resolve = |p: PathBuf| {
                if p.is_absolute() { p } else { cwd.join(p) }
            };
            if root.is_empty() && views.is_empty() {
                anyhow::bail!("specify --root=<path> and/or --views=<path> at least once");
            }
            // First-party app routes: walk <root>/routes/ for mountable
            // views (each is a dir with a page.sui). One routes.gen.ts per
            // --root; the label is the path's basename. The codegen has
            // no app-name coupling — generic over the workspace.
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
                    // `deck routes` is a standalone CLI command — it
                    // doesn't read the host's `deck.jsonc` (the bundler
                    // owns that). No host bootstrap, so the codegen
                    // writes `routes.gen.ts` in legacy shape (entry IS
                    // `routes.gen.ts`; the token import rides there).
                    entry_bootstrap: None,
                });
            }
            let n_routes = deck_bundler::routes::generate_routes(&routes_specs)?;
            // View discovery: walk <views>/<name>/manifest.jsonc, emit
            // TWO files per --views: routes.gen.ts (router) and
            // meta.gen.ts (host). Same RouteGenSpec shape — the codegen
            // writes meta.gen.ts as a side-effect when the spec's dir is
            // a views tree (the discover_views() walker kicks in).
            let mut views_specs = Vec::new();
            for s in &views {
                let r = resolve(s.clone());
                let label = r
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| r.display().to_string());
                views_specs.push(deck_bundler::routes::RouteGenSpec {
                    dir: r.clone(),
                    out: r.join("routes.gen.ts"),
                    label,
                    entry_bootstrap: None,
                });
            }
            let n_views = deck_bundler::routes::generate_routes(&views_specs)?;
            println!(
                "deck routes → {n_routes} first-party route(s), {n_views} view route(s)"
            );
            Ok(())
        }
        Command::Scaffold(Scaffold::View { name, path, component }) => {
            scaffold::run_view(name, path, component)
        }
        Command::Scaffold(Scaffold::Component { tag, controller }) => {
            scaffold::run_component(tag, controller)
        }
        Command::Build { project, out, bundle_framework, no_minify, typecheck } => {
            build(project, out, bundle_framework, !no_minify, typecheck)
        }
        Command::Scaffold(Scaffold::Pack { name }) => {
            scaffold::run_pack(name)
        }
        Command::Emit { targets, out } => emit(targets, out),
        Command::BuildAssets => build_assets(),
        Command::Dev { project, addr, demo } => dev(project, addr, demo),
    }
}
/// closure — see `deck_bundler::emit`).
fn emit(targets: Vec<String>, out: PathBuf) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let out = if out.is_absolute() { out } else { cwd.join(out) };
    let repo_root = find_repo_root(&cwd)?;
    let cfg = deck_bundler::DeckConfig::defaults();
    let layout = deck_bundler::targets::BundleLayout::with_config(repo_root, None, cfg);
    let n = deck_bundler::emit::emit_targets(&layout, &targets, &out)?;
    println!("deck emit → {} files into {}", n, out.display());
    Ok(())
}

fn build_assets() -> Result<()> {
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

    // Materialize each component pack's standard `dist/custom-elements.json`
    // (CEM) from its source. Run once here (the framework build), so per-app
    // builds aggregate the pre-written manifests into `<app>/components.gen.ts`
    // (`generate_components_gen`). Also the file every pack declares for
    // external tooling (Storybook / VS Code / lit-analyzer).
    match deck_bundler::cem::write_all_pack_cems(&repo_root) {
        Ok(n) => println!("  cem: {n} component pack manifest(s) written"),
        Err(e) => eprintln!("  cem: pack manifest generation failed: {e:#}"),
    }

    // Build the layout once — used for both the framework verify and the
    // consumer guard below. assets_dir_for("sui-runtime") is now
    // .deck/framework/ (the canonical bake target per Recommendation B1).
    let layout = deck_bundler::targets::BundleLayout::with_config(
        repo_root.clone(),
        None,
        cfg,
    );

    // Verify every framework singleton that RESOLVED for this repo is non-empty.
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
            bail!("framework bundle missing or empty after bake: {}", path.display());
        }
        println!("  {}: {len} bytes", t.out_file);
    }
    if fw_targets.is_empty() {
        println!("  (no framework singletons resolved from this repo — none baked here)");
    }

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

    println!("deck build-assets: bundles baked via in-process Rolldown");
    println!("note: lucide-icons.json is committed source; regen via `cargo xtask gen-icons`");
    Ok(())
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
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    Ok(deck_bundler::paths::topmost_cargo_root(start).unwrap_or_else(|| start.to_path_buf()))
}

fn build(
    project: PathBuf,
    out: Option<PathBuf>,
    bundle_framework: bool,
    minify: bool,
    typecheck: bool,
) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let abs = |p: &Path| if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };

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
        .unwrap_or_else(|| project.join(".deck"));
    let repo_root = find_repo_root(&project)?;
    // The framework singletons are resolved by NODE RESOLUTION from the
    // project's `node_modules` (the bundler's `framework_singletons` /
    // `bundle_owned_framework`) — `--bundle-framework` builds them from source.
    // There is no prebuilt-asset staging dir and no `DECK_FRAMEWORK_ASSETS_DIR`
    // crutch: the framework comes from wherever it was installed (the registry,
    // a workspace symlink), so a build runs identically inside or outside the
    // sui monorepo.

    // Load the host's view-discovery config (packs scope). Consumer bundles
    // are discovered from per-package `deck.bundles`, not read here. L1 (deck)
    // has no per-app config — shipd's apps.jsonc owns the app list; the
    // bundler is per-app and reads its own project root.
    let cfg = deck_bundler::DeckConfig::defaults();

    // The app graph compiles its own `.sui` in-process via the linked
    // sui-compiler crate (#255) — no binary to locate.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    let b = rt
        .block_on(deck_bundler::app::bundle_app(
            &repo_root,
            &cfg.packs,
            &project,
            &out_dir,
            minify,
            bundle_framework,
        ))
        .map_err(|e| anyhow::anyhow!("{e:#}"))?;
    if b.framework_assets > 0 {
        println!("  _deck  {} files (self-contained)", b.framework_assets);
    } else if !bundle_framework {
        // The silent trap: with no `--bundle-framework`, the build emits the app
        // but ZERO framework (`_deck/runtime.js`, `dom.js`, `route.js`) and still
        // exits 0. Fine for a shipd-served app (shipd embeds the framework via
        // `include_str!`), but a standalone host (a Tauri shell, a custom server)
        // serves nothing — every `<sui-*>` stays unregistered and the app renders
        // unstyled with no error. shipd bundles in-process via `bundle_app`, not
        // this CLI path, so this warning targets exactly the standalone builds.
        eprintln!(
            "  warning: built WITHOUT a framework bundle (no --bundle-framework).\n  \
             This app's <sui-*> components need /_deck/runtime.js, dom.js and route.js\n  \
             served by the host. shipd embeds them; a standalone host serves nothing,\n  \
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

// ── `deck dev` — standalone dev server ─────────────────────────────────────
//
// Composes the existing bundler + router (the same wiring as
// `deck_conformance::deck_dev_server`) so deck can run as a framework
// without a host adapter or shipd. The watcher rebuilds on edit and
// pushes HMR; the router serves the whole `/_deck/*` surface.
//
// The default data port is a `NoopDataPort`: there is no per-view live
// data wired here. `--demo` is the documented escape hatch for hosts
// that want the conformance `DemoStore` — it lives in
// `deck-conformance`, which is test-only by design; for now the
// standalone CLI defers the wired-demo shape to `deck dev --demo`
// (rejected at runtime until the host wiring is upstreamed).

/// A `DataPort` that resolves nothing. Default in `deck dev` —
/// live-data is opt-in via `--demo`. (`is_provisioned` defaults to
/// `false` from the trait, so the framework's first-run routing serves
/// `/setup` until a host opts in.)
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

fn dev(project: PathBuf, addr: String, demo: bool) -> Result<()> {
    if demo {
        anyhow::bail!(
            "deck dev --demo is reserved for the conformance harness; \
             use `cargo run -p deck-conformance` for the wired-demo shape"
        );
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    runtime.block_on(async move {
        // Resolve the project root. Canonicalize so symlinks/bare
        // relative paths collapse to a single absolute path the bundler
        // and resolver can key off.
        let cwd = std::env::current_dir().context("cwd")?;
        let project_path = if project.is_absolute() {
            project.clone()
        } else {
            cwd.join(&project)
        };
        let repo_root = project_path
            .canonicalize()
            .with_context(|| format!("canonicalize {}", project_path.display()))?;
        // Bail up front if the path isn't a directory — keeps the
        // later bundler diagnostics readable.
        if !repo_root.is_dir() {
            anyhow::bail!("{} is not a directory", repo_root.display());
        }

        // Point deck-core's resolver at this root (set once per
        // process; deck serves one project per daemon).
        deck_core::resolver::set_project_root(repo_root.clone());

        // Bundler construction mirrors `deck_conformance::make_bundler`:
        // built-in DeckConfig defaults, single-project root, FS watcher
        // running in-process.
        let cfg = deck_config::DeckConfig::defaults();
        let bundler = deck_bundler::Bundler::start(repo_root.clone(), None, cfg.clone());
        if let Err(e) = bundler.start_watching().await {
            eprintln!("deck dev: watcher failed to start: {e}");
        }
        // The hosting surface. Extension layers match what deck-axum's
        // handlers read: `DaemonMode` for mode-aware hand-offs, the
        // parsed `DeckConfig` (Arc) for scaffold lookups, the
        // `Bundler` itself for `/_deck/views/{name}/setup` on-demand
        // rebuilds, and a `DataPort` extension so a future multiplex
        // mount can reach it with no shape change here. The default
        // data port is the no-op impl — `deck dev --demo` is the
        // documented escape hatch for the conformance `DemoStore`
        // (rejected today; the conformance harness is where the
        // wired-demo shape lives).
        let app = deck_axum::router()
            .layer(axum::Extension(DaemonMode::Development))
            .layer(axum::Extension(Arc::new(cfg)))
            .layer(axum::Extension(bundler.clone()))
            .layer(axum::Extension(Arc::new(NoopDataPort) as Arc<dyn deck_core::DataPort>));
        let bind_addr = addr.clone();
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .with_context(|| format!("bind {bind_addr}"))?;
        let bound = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| bind_addr.clone());
        println!("deck dev: serving {} on http://{bound}", repo_root.display());
        println!("deck dev: ctrl-c to stop");

        // Graceful shutdown: a ctrl-c finishes the in-flight request
        // (axum::serve's documented hook) and then we tear the
        // bundler's watch loop down before exit.
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                if let Err(e) = tokio::signal::ctrl_c().await {
                    eprintln!("deck dev: ctrl_c handler: {e}");
                }
            })
            .await
            .context("dev server")?;

        // Bundler watchdog: stop the watch task before returning so
        // the process doesn't linger on a detached tokio task.
        if let Err(e) = bundler.shutdown().await {
            eprintln!("deck dev: bundler shutdown: {e}");
        }
        Ok(())
    })
}

