//! `deck` — the deck framework CLI (the `next` / `vite` analog).
//!
//! deck is the framework; an app (the hub `apps/shell`, a space scaffold) is
//! built *with* deck via `deck build`, just as `next build` builds a Next app.
//! This is why the command lives here and not in the `ship` app CLI. Backed by
//! `deck-bundler` (Rolldown). `.sui` sources must be compiled first
//! by the deck bundler; `deck dev` / `deck serve` are future commands.

mod scaffold;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};

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
        Command::Build { project, out, bundle_framework, no_minify } => {
            build(project, out, bundle_framework, !no_minify)
        }
        Command::Scaffold(Scaffold::Pack { name }) => {
            scaffold::run_pack(name)
        }
        Command::Emit { targets, out } => emit(targets, out),
        Command::BuildAssets => build_assets(),
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

/// The framework bundles deck-core/build.rs bakes via `include_str!`. Verified
/// non-empty after the bake so a silently-failed target cannot ship empty bytes
/// (the failure mode build.rs guards — top nav vanishing with no warning).
const FRAMEWORK_BUNDLES: &[&str] = &[
    "sui-runtime.js",
    "sui-dom.js",
    "sui-sdk-client.js",
];

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

    // Verify framework bundles are non-empty after the bake. build_all returns
    // Err on per-target failure, but a swallowed empty-write wouldn't surface
    // there — keep this as defense in depth so the daemon can never ship empty
    // framework bytes. Use assets_dir_for("sui-runtime") so the verify path
    // stays in sync with the bundler's output path (.deck/framework/).
    let fw_dir = layout.assets_dir_for("sui-runtime");
    for name in FRAMEWORK_BUNDLES {
        let path = fw_dir.join(name);
        let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if len == 0 {
            bail!("framework bundle missing or empty after bake: {}", path.display());
        }
        println!("  {name}: {len} bytes");
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

/// Resolve the Cargo workspace root from `start`. It's the cwd for module
/// resolution and the source of the framework bundles. Walking the topmost
/// `Cargo.toml` is the one canonical fact; any monorepo-specific marker
/// surfaces later when bundle_app reads the framework assets dir.
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    deck_bundler::paths::topmost_cargo_root(start).ok_or_else(|| {
        anyhow!("deck build: no Cargo workspace root above {}", start.display())
    })
}

fn build(project: PathBuf, out: Option<PathBuf>, bundle_framework: bool, minify: bool) -> Result<()> {
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
    // Override the framework-asset staging path with an env var. Default:
    // `<repo_root>/crates/deck/deck-core/assets`. After the carve, the
    // host app (ship) needs the framework bundles to live in the
    // external sui repo's `deck-core/assets/` — but its `find_repo_root`
    // only walks the cwd's Cargo workspace, which in cross-repo
    // invocations lands at ship (not sui). Allow the host to point at
    // sui's tree directly. TEMP — re-evaluate after the layout
    // reshuffle capstone (when the two repos share a parent again).
    let framework_assets_dir = std::env::var("DECK_FRAMEWORK_ASSETS_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("crates/deck/deck-core/assets"));
    // The framework surface (.deck/framework/<bundle>) is shared across
    // every project. Per-project outputs go under `<project>/.deck/`.
    let deck_out_dir = repo_root.join(".deck");

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
            &framework_assets_dir,
            &deck_out_dir,
        ))
        .map_err(|e| anyhow::anyhow!("{e:#}"))?;
    if b.framework_assets > 0 {
        println!("  _deck  {} files (self-contained)", b.framework_assets);
    }
    Ok(())
}
