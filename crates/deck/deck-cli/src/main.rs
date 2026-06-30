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
        /// Asset-URL base. `--base assets` makes asset URLs (`/assets/<file>`)
        /// match the on-disk `assets/` layout, so the build is directly
        /// static-host-servable (no deck server). Default: the package-name
        /// serve namespace (a host maps `/<name>/<file>` → `assets/<file>`).
        #[arg(long)]
        base: Option<String>,
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
        Command::Build { project, out, bundle_framework, no_minify, typecheck, base } => {
            build(project, out, bundle_framework, !no_minify, typecheck, base)
        }
        Command::Scaffold(Scaffold::Pack { name }) => {
            scaffold::run_pack(name)
        }
        Command::Emit { targets, out } => emit(targets, out),
        Command::BuildAssets => build_assets(),
        Command::Dev { project, addr } => dev(project, addr),
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
///
/// NOTE: for a pure-JS bun/npm workspace built from an app subdir
/// (`ship-apps/apps/helm`), `start` is the app dir, NOT the workspace root —
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
        // serves nothing — every `<sui-*>` stays unregistered and the app renders
        // unstyled with no error. shipd bundles in-process via `bundle_app`, not
        // this CLI path, so this warning targets exactly the standalone builds.
        eprintln!(
            "  warning: built WITHOUT a framework bundle (no --bundle-framework).\n  \
             This app's <sui-*> components need /_deck/runtime.js, dom.js and route.js\n  \
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

// ── `deck dev` — standalone dev server ─────────────────────────────────────
//
// Composes the existing bundler + router (the same wiring as
// `deck_conformance::deck_dev_server`) so deck can run as a framework
// without a host adapter or shipd. The watcher rebuilds on edit and
// pushes HMR; the router serves the whole `/_deck/*` surface.
//
// The data port is a `NoopDataPort` and the live-data multiplex transport
// (`/_deck/mux/*`) is NOT mounted: standalone `deck dev` serves routing,
// assets, SSR and HMR, but NOT live server `<data source>`. Live data is a
// HOST capability — a host (shipd) merges `deck::multiplex_router()` and
// layers its own `DataPort`. Wiring live data into the standalone CLI is the
// server-functions arc (#32), targeted for a later release.

/// The app's URL namespace = `package.json#name`'s last `/`-segment
/// (`@ship/hub` → `hub`). MUST equal what `bundle_app` baked the asset
/// hrefs under, so `serve_apps`' `/<name>/<file>` route resolves them.
/// Mirrors the bundler's own `read_package_name` + `rsplit_once('/')`.
fn app_url_namespace(project: &Path) -> Result<String> {
    let pkg_path = project.join("package.json");
    let body = std::fs::read_to_string(&pkg_path)
        .with_context(|| format!("read {}", pkg_path.display()))?;
    let json: serde_json::Value =
        serde_json::from_str(&body).with_context(|| format!("parse {}", pkg_path.display()))?;
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .with_context(|| format!("{} missing \"name\"", pkg_path.display()))?;
    Ok(name
        .rsplit_once('/')
        .map(|(_, tail)| tail)
        .unwrap_or(name)
        .to_string())
}

/// A `DataPort` that resolves nothing — the standalone `deck dev` default.
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
        let project_dir = project_path
            .canonicalize()
            .with_context(|| format!("canonicalize {}", project_path.display()))?;
        // Bail up front if the path isn't a directory — keeps the
        // later bundler diagnostics readable.
        if !project_dir.is_dir() {
            anyhow::bail!("{} is not a directory", project_dir.display());
        }

        // Point deck-core's resolver at this root: `serve_index` /
        // `serve_apps` / `spa_fallback` read the app's built assets
        // (`.deck/index.html`, `/<name>/<file>`) relative to it.
        deck_core::resolver::set_project_root(project_dir.clone());

        let cfg = deck_config::DeckConfig::defaults();

        // Build the app ONCE up front. `start_watching` only REBUILDS on
        // edit; without an initial build the project emits no `.deck/`
        // output and EVERY page 404s — the gap a runtime-only smoke test
        // (just `/_deck/runtime.js`) misses. `bundle_framework = false`:
        // deck dev serves the framework singletons itself at `/_deck/*`,
        // so the app graph keeps them external (no per-app copy). The real
        // workspace root (component/workspace resolution) is found by
        // walking up from the project; the project dir itself is one app.
        let workspace_root = find_repo_root(&project_dir)?;
        let out_dir = project_dir.join(".deck");
        let build = deck_bundler::app::bundle_app(
            &workspace_root,
            &cfg.packs,
            &project_dir,
            &out_dir,
            false,
            false,
        )
        .await
        .map_err(|e| anyhow::anyhow!("deck dev: initial app build failed: {e:#}"))?;
        println!("deck dev: built app ({} css asset(s))", build.css_hrefs.len());

        // Register the project as the PRIMARY app at `/`. `name` MUST equal
        // the URL namespace `bundle_app` baked the asset hrefs under
        // (`package.json#name`'s last `/`-segment) so `/<name>/<file>`
        // resolves; `source` is the build out dir relative to the project
        // root. `deck.jsonc` (optional) carries injectGlobals / render /
        // entry — name/route/source are host-bound here, never the file's.
        let app_cfg_path = project_dir.join("deck.jsonc");
        let mut app_cfg = if app_cfg_path.exists() {
            let raw = std::fs::read_to_string(&app_cfg_path)
                .with_context(|| format!("read {}", app_cfg_path.display()))?;
            deck_config::AppServeConfig::parse_deck_jsonc(&raw)
                .with_context(|| format!("parse {}", app_cfg_path.display()))?
        } else {
            deck_config::AppServeConfig::default()
        };
        // SSR engine: only spin up ship-js (V8) when this app opts into SSR
        // (`deck.jsonc { "render": "ssr" }`). A CSR app (the default) pays no
        // V8 cost. Standalone dev has no live data port, so the engine renders
        // with NoopData (None). Additive — if the engine can't start, log +
        // continue; the serve path falls back to CSR.
        if app_cfg.render == deck_config::RenderMode::Ssr {
            match deck_ssr::serve::register(None) {
                Ok(()) => println!("deck dev: ship-js SSR engine registered (render: ssr)"),
                Err(e) => eprintln!(
                    "deck dev: ship-js SSR engine unavailable, serving CSR: {e:#}"
                ),
            }
        }
        app_cfg.name = app_url_namespace(&project_dir)?;
        app_cfg.route = "/".to_string();
        app_cfg.source = ".deck".to_string();
        // The per-app asset namespace is the app's name (`/<name>/<file>`);
        // `serve_app_asset` reads the app off this per-route extension and
        // maps to `<source>/assets/<file>`.
        let asset_route = format!("/{}/{{file}}", app_cfg.name);
        let app_ext = Arc::new(app_cfg.clone());
        let apps: Arc<Vec<deck_config::AppServeConfig>> = Arc::new(vec![app_cfg]);

        // `Resolution` Step 2: carry deck dev's single root as a per-request
        // mount-scoped handle so the SSR first-paint resolves THROUGH the threaded
        // resolver (`resolve_project_root_with`) rather than reading the process
        // global directly. deck dev serves ONE project, so this root equals the
        // global — but threading it proves the serve path honors a per-request
        // Resolution end-to-end (the harness exercises it). The multi-app host
        // (shipd) is the path that carries a DISTINCT Resolution per app, killing
        // the cross-talk for real (`inv-deck-single-root-scoped`).
        let resolution = Arc::new(deck_core::Resolution::new().with_root(project_dir.clone()));

        // Bundler with the FS watcher (HMR SSE on `/_deck/dev/hmr`). The
        // up-front `bundle_app` is the one-shot SPA bundle that makes the
        // app servable; the watcher drives rebuilds + live-reload on edit.
        let bundler = deck_bundler::Bundler::start(project_dir.clone(), None, cfg.clone());
        if let Err(e) = bundler.start_watching().await {
            eprintln!("deck dev: watcher failed to start: {e}");
        }

        // ── HMR (foundational, full-document) ────────────────────────────────
        // The bundler's own watcher rebuilds TARGETS (views, framework). The
        // SPA is the one-shot `bundle_app` ENTRY — not a target — so `routes/` +
        // `src/` edits emit nothing through it. deck dev owns this loop: a notify
        // watcher on the authored source re-runs `bundle_app` and emits a
        // `module-changed` (target `app`) on the bundler's firehose, which
        // `serve_bundle_hmr` streams to the live-reload client → full-document
        // reload. (Module-granular reload is the next layer.) We watch `routes/`
        // + `src/` + `public/` ONLY — never the project root or `.deck/` — so the
        // regenerated `routes.gen.ts` / `.deck/` writes can't self-trigger a loop.
        let hmr_packs = cfg.packs.clone();
        let (hmr_tx, mut hmr_rx) = tokio::sync::mpsc::channel::<()>(8);
        let hmr_watcher = {
            use notify::Watcher;
            let tx = hmr_tx;
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
                // Ignore generated artifacts a rebuild writes (`.gen.ts`,
                // `.sui.d.ts` companions, `.compiled.js`) so they can't
                // self-trigger.
                let all_generated = !ev.paths.is_empty()
                    && ev.paths.iter().all(|p| {
                        let s = p.to_string_lossy();
                        s.ends_with(".gen.ts")
                            || s.contains(".sui.d.ts")
                            || s.contains(".compiled.js")
                    });
                if all_generated {
                    return;
                }
                // Coalesce a save's event burst into one rebuild via the debounce
                // below; dropping on a full channel is fine (the next tick rebuilds).
                let _ = tx.try_send(());
            })
            .map_err(|e| anyhow::anyhow!("deck dev: HMR watcher init: {e}"))?;
            for sub in ["routes", "src", "public"] {
                let dir = project_dir.join(sub);
                if dir.is_dir() {
                    let _ = w.watch(&dir, notify::RecursiveMode::Recursive);
                }
            }
            w
        };
        {
            let bundler_hmr = bundler.clone();
            let ws = workspace_root.clone();
            let proj = project_dir.clone();
            let od = out_dir.clone();
            tokio::spawn(async move {
                while hmr_rx.recv().await.is_some() {
                    // Debounce: settle the burst, then drain coalesced ticks.
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    while hmr_rx.try_recv().is_ok() {}
                    match deck_bundler::app::bundle_app(&ws, &hmr_packs, &proj, &od, false, false).await {
                        Ok(_) => {
                            let _ = bundler_hmr.emit_watch_event(
                                deck_bundler::BundlerEvent::ModuleChanged {
                                    target: "app".to_string(),
                                    out_path: od.join("index.html").display().to_string(),
                                    bytes: 0,
                                },
                            );
                            println!("deck dev: rebuilt app (HMR)");
                        }
                        Err(e) => {
                            eprintln!("deck dev: rebuild failed: {e:#}");
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
        // fallback — two merged fallbacks panic axum). deck dev IS the
        // host, so it adds the primary `/` index, the per-app routes
        // (`serve_apps`: asset namespace + single-segment public catch-all),
        // and the SPA navigation fallback that serves the app shell for
        // multi-segment client routes. Extension layers: `DaemonMode`, the
        // `DeckConfig` (Arc), the app list (`serve_index` / `spa_fallback`
        // read it to find the primary app), the `Bundler`, and the no-op
        // `DataPort`. The live-data multiplex (`/_deck/mux/*`) is intentionally
        // NOT merged here — standalone deck has no live `<data source>` (#32).
        use axum::routing::get;
        let app = deck_axum::router()
            // `/` → the primary app's index.html.
            .route("/", get(deck_axum::pages::serve_index))
            // `/<name>/<file>` → the app's hashed bundle assets
            // (`entry-*.js`, `tokens-*.css`) under `<source>/assets/`.
            .route(
                &asset_route,
                get(deck_axum::apps::serve_app_asset).layer(axum::Extension(app_ext)),
            )
            // `/<seg>` → single-segment: a DOTTED name is a public asset
            // (`favicon.ico`); a BARE segment is a CLIENT route (spaces'
            // `/:appId`), so serve the app shell and let client-side routing
            // resolve it. `serve_app_public` (what `serve_apps` mounts here)
            // would 404 the bare case — `spa_or_public_asset` handles both.
            .route("/{file}", get(deck_axum::pages::spa_or_public_asset))
            // Deeper unmatched GET navigations (multi-segment client routes)
            // → the shell. deck's `router()` sets no fallback by design (it
            // expects to merge alongside a host's ops router); deck dev IS
            // the host, so it owns the fallback.
            .fallback(deck_axum::pages::spa_fallback)
            .layer(axum::Extension(DaemonMode::Development))
            .layer(axum::Extension(Arc::new(cfg)))
            .layer(axum::Extension(bundler.clone()))
            .layer(axum::Extension(apps))
            .layer(axum::Extension(resolution))
            .layer(axum::Extension(Arc::new(NoopDataPort) as Arc<dyn deck_core::DataPort>));
        let bind_addr = addr.clone();
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .with_context(|| format!("bind {bind_addr}"))?;
        let bound = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| bind_addr.clone());
        println!("deck dev: serving {} on http://{bound}", project_dir.display());
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
        // the process doesn't linger on a detached tokio task. Drop the HMR
        // watcher too (it lived for the server's lifetime; dropping ends its
        // notify thread).
        drop(hmr_watcher);
        if let Err(e) = bundler.shutdown().await {
            eprintln!("deck dev: bundler shutdown: {e}");
        }
        Ok(())
    })
}

