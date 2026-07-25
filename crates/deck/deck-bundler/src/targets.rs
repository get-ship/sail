//! Bundle target resolution for the Rust bundler (#163).
//!
//! `TARGETS` + `discoverRouteBundles` + `assetsDirFor` for the bundler. A
//! `BundleTarget` names what to build, where its entry is, what stays external,
//! and where the output lands. The Rolldown build (`rolldown_bundle.rs`)
//! iterates these; nothing here touches the filesystem beyond discovery reads.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use deck_config::DeckConfig;

/// Deck's OWN framework singleton — the runtime primitives deck ships with
/// itself (like Next shipping its own runtime). `spec` is the npm specifier the
/// bundler resolves by NODE RESOLUTION (`node_resolve::resolve_entry`) to find
/// the entry; `name` is the logical target/output stem; `externals` are the
/// other singletons it must NOT inline. SSOT for the singleton set: the build
/// target list, the `assets_dir_for` routing, and `is_framework_singleton` all
/// read it. Chrome components are NOT here — the per-tag tree-shaking resolver
/// (`sail-components/<name>` in `assets_dir_for` + the `@sail-tag/<tag>` virtual
/// import in `sail_plugin.rs`) is the canonical way chrome loads.
pub(crate) struct FrameworkSingleton {
    pub name: &'static str,
    pub spec: &'static str,
    pub out_file: &'static str,
    pub externals: &'static [&'static str],
    /// Extra files copied verbatim to the PROJECT ROOT (not next to the built
    /// `out_file`), sourced from the same directory as the resolved entry
    /// (`resolve_entry`'s result, before Rolldown ever sees it), under their
    /// own literal filename — never renamed, never bundled. `diagnostics.ts`
    /// (`packages/deck/lsp-browser/src/diagnostics.ts`) calls the wasm's
    /// `initWasm(wasmUrl)` with an EXPLICIT url (default `/sail_compiler_bg.wasm`,
    /// root-absolute) rather than letting wasm-bindgen fall back to its own
    /// `import.meta.url`-relative default, so the `.wasm` binary must sit at
    /// the root path callers actually request — see `bundle_owned_framework`
    /// in `app.rs`. Empty for every singleton that is pure JS/TS source (the
    /// rest).
    pub co_assets: &'static [&'static str],
}

/// The framework singletons, resolved from `node_modules`/workspace by `spec`.
pub(crate) const FRAMEWORK_SINGLETONS: &[FrameworkSingleton] = &[
    FrameworkSingleton {
        name: "sail-runtime",
        // Full singleton surface (front door + all `use:` actions) so the
        // importmap can map `@ship/sail-runtime` and `/actions` to one URL
        // without losing auto-imported action names. Author TS still resolves
        // bare `@ship/sail-runtime` → curated `./src/index.ts`.
        spec: "@ship/sail-runtime/framework",
        out_file: "sail-runtime.js",
        externals: &[],
        co_assets: &[],
    },
    FrameworkSingleton {
        name: "sail-dom",
        spec: "@ship/sail-dom",
        out_file: "sail-dom.js",
        externals: &["@ship/sail-runtime"],
        co_assets: &[],
    },
    FrameworkSingleton {
        name: "sail-sdk-client",
        spec: "@ship/sdk-client",
        out_file: "sail-sdk-client.js",
        externals: &["@ship/sail-runtime"],
        co_assets: &[],
    },
    // `@ship/sail-compiler-wasm` — the in-browser WebAssembly build of the
    // SAIL compiler (docs playground + LSP worker's `analyze`/`compile`).
    // Shipped by deck itself (same class as runtime/dom), so it is a
    // framework singleton, not provider-declared. Its `.` export is a
    // wasm-bindgen glue module (`pkg/sail_compiler.js`), not TS source — see
    // `co_assets` doc: the companion `.wasm` binary must ride along.
    FrameworkSingleton {
        name: "sail-compiler-wasm",
        spec: "@ship/sail-compiler-wasm",
        out_file: "sail-compiler-wasm.js",
        externals: &[],
        co_assets: &["sail_compiler_bg.wasm"],
    },
    // `@ship/sail-lsp-browser/completions` — scope-aware `.sail` completions
    // for the docs playground editor. Self-contained (no compiler-wasm
    // dependency), so it externalises nothing.
    FrameworkSingleton {
        name: "sail-lsp-completions",
        spec: "@ship/sail-lsp-browser/completions",
        out_file: "sail-lsp-completions.js",
        externals: &[],
        co_assets: &[],
    },
    // `@ship/sail-lsp-browser/diagnostics` — the full diagnostic stream, off
    // the same wasm `analyze()` diagnostics.ts calls directly
    // (`import initWasm, { analyze } from '@ship/sail-compiler-wasm'`). Must
    // externalise `@ship/sail-compiler-wasm` or this inlines a SECOND copy of
    // the (large) wasm glue + binary into this singleton.
    FrameworkSingleton {
        name: "sail-lsp-diagnostics",
        spec: "@ship/sail-lsp-browser/diagnostics",
        out_file: "sail-lsp-diagnostics.js",
        externals: &["@ship/sail-compiler-wasm"],
        co_assets: &[],
    },
    // `@ship/sdk` — the Connect SDK (generated protobuf/Connect client, the
    // `@ship/sdk/connect` surface). It is large (~1.5MB) and shared across
    // every app that talks to the daemon, so it is a framework singleton: one
    // shared bundle, referenced via the importmap, NOT inlined per-app. Before
    // this, an app like cmdk that imports `@ship/sdk/connect` inlined the whole
    // SDK into its own bundle. Standalone (no `@ship/sail-runtime` dependency),
    // so it externalises nothing. Resolved by node from the project being built
    // and skipped when absent (consumer repos that don't depend on it).
    FrameworkSingleton {
        name: "sdk",
        spec: "@ship/sdk",
        out_file: "sdk.js",
        externals: &[],
        co_assets: &[],
    },
    // `@ship/deck-router` — the unified router singleton (defineRoutes / route /
    // navigate / mountRouter / mergeRoutes). Every per-app `routes.gen.ts`
    // imports `mountRouter` from it (ADR unified-deck-router), and the daemon
    // serves it via `include_str!` (assets.rs `DECK_ROUTE_JS`).
    FrameworkSingleton {
        name: "deck-route",
        spec: "@ship/deck-router",
        out_file: "deck-route.js",
        externals: &["@ship/sail-runtime"],
        co_assets: &[],
    },
];

/// True for deck's own framework singletons — libraries consumed by name via the
/// host importmap (`@ship/sdk-client` → `/_deck/sdk-client.js`, …). Their full
/// public export surface (including re-exported consts like `DECK_API_ASSETS`)
/// must survive tree-shaking, so the Rolldown build sets
/// `preserve_entry_signatures: Strict` for them. The default `ExportsOnly` DCE'd
/// a re-exported const with no in-bundle consumer, breaking the client at runtime
/// with "does not provide an export named …". App/route targets keep the default.
pub fn is_framework_singleton(name: &str) -> bool {
    FRAMEWORK_SINGLETONS.iter().any(|s| s.name == name)
}

/// A target's entry: a real on-disk module, or a synthetic aggregate that
/// imports each `source` by absolute path (the build phase generates the entry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetEntry {
    Real(PathBuf),
    Virtual { sources: Vec<PathBuf> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleTarget {
    /// Logical name, e.g. `sail-runtime`, `sail-components/sail-rail`, `routes/git/route`.
    pub name: String,
    pub entry: TargetEntry,
    /// Externals the author declared. The build phase additionally forces
    /// `@ship/sail-runtime` external for every target except `sail-runtime`.
    pub externals: Vec<String>,
    /// Output path relative to the target's assets dir (`assets_dir_for`).
    pub out_file: String,
    /// Lazy intent (from a consumer bundle's self-declaration). Only the
    /// optional dev-loop skip (`DECK_DEV_SKIP_LAZY=1`) reads it — it omits heavy
    /// lazy packs from a dev bake. Always `false` for framework + route targets.
    /// The default bake (env unset) builds lazy and eager identically, so this
    /// flag never changes production output.
    pub lazy: bool,
}

/// Repo + workspace layout. `repo_root` is the ship monorepo (framework +
/// built-in routes); `source_roots` is `[repo_root]` in contributor mode or
/// `[repo_root, workspace_root]` when the daemon serves an external project.
pub struct BundleLayout {
    pub repo_root: PathBuf,
    pub source_roots: Vec<PathBuf>,
    /// Base dir output paths are computed under (the `crates/.../assets`, `.deck`,
    /// `apps/shipd/assets` subtrees hang off this). Defaults to `repo_root`;
    /// redirect for tests / out-of-tree builds so output never clobbers
    /// committed assets.
    pub out_root: PathBuf,
    /// Route-discovery + watch config (`route_dirs`, `route_leaf`, `packs`,
    /// `watch`). Consumer bundle targets are NO LONGER here — they are
    /// discovered from per-package `deck.bundles` (see `consumer_targets`).
    /// Defaults to `DeckConfig::defaults()` for zero-config hosts.
    pub cfg: DeckConfig,
}

impl BundleLayout {
    /// Construct with an explicit [`DeckConfig`] — the only construction path.
    /// `DeckConfig` is host-supplied (the daemon calls [`DeckConfig::defaults`]
    /// plus any overrides it wants); the bundler does not load any file from
    /// the workspace root. `repo_root` is the host's project root (the same root
    /// it carries in its runtime `Resolution`); `workspace_root` distinguishes an
    /// external-project workspace from the framework repo for source-root resolution.
    pub fn with_config(
         repo_root: PathBuf,
         workspace_root: Option<PathBuf>,
         cfg: DeckConfig,
     ) -> Self {
         let source_roots = match &workspace_root {
             Some(ws) if *ws != repo_root => vec![repo_root.clone(), ws.clone()],
             _ => vec![repo_root.clone()],
         };
         Self { out_root: repo_root.clone(), repo_root, source_roots, cfg }
     }

    /// Redirect output under `out_root` (sources still resolve from `repo_root`).
    pub fn with_out_root(mut self, out_root: PathBuf) -> Self {
        self.out_root = out_root;
        self
    }
    /// Build-time output dir: `<out_root>/.deck` is the bake's staging
    /// area — `deck build-assets` writes framework + route + chrome
    /// bundles here, and `crates/deck-core/build.rs` copies the
    /// framework singletons into `crates/deck/deck-core/assets/` for
    /// `include_str!`. The framework's RUNTIME does not read this
    /// directory; the daemon serves singletons via `include_str!`
    /// constants and routes on demand (route.rs). This is build-time
    /// orchestration only — the bundler always writes under `<out_root>/.deck`
    /// (the bake's contract), resolved through the single
    /// `deck_config::deck_out_path` so the runtime reader can't drift from it.
    pub fn deck_out(&self) -> PathBuf {
        deck_config::deck_out_path(&self.out_root)
    }
    fn framework_dir(&self) -> PathBuf {
        self.deck_out().join("framework")
    }

    /// Output dir for a target by name. Every build artifact lands under
    /// `.deck/` — the bundler owns its output layout; the host / project
    /// root is never a bake target. Deck-own framework singletons go
    /// under `.deck/framework/` (the build-script staging path that
    /// `build.rs` copies into `crates/deck/deck-core/assets/` for
    /// `include_str!`); everything else — `routes/*`, `sail-components/*`,
    /// and discovered consumer packs (`sail-marketing`, `sail-arcade`,
    /// `sail-pdf`, `ship-code-editor`, …) — goes directly under `.deck/`.
    pub fn assets_dir_for(&self, name: &str) -> PathBuf {
        if is_framework_singleton(name) {
            return self.framework_dir();
        }
        self.deck_out()
    }

    /// All targets: framework singletons + discovered consumer bundles + route bundles.
    pub fn all_targets(&self) -> Vec<BundleTarget> {
        let mut out = self.framework_singletons();
        out.extend(self.consumer_targets());
        out.extend(self.discover_route_bundles());
        out
    }

    /// Deck's OWN framework singletons — always built, never in deck.jsonc.
    /// These are the runtime primitives deck ships with itself (like Next.js
    /// shipping its own runtime).
    ///
    /// Each entry is resolved by NODE RESOLUTION from the project being built
    /// (`node_resolve::resolve_entry` against `repo_root`) — NOT a hardcoded
    /// monorepo path. The framework comes from wherever npm/bun installed it:
    /// a workspace member in the sail monorepo, or a registry install in a
    /// consumer repo (ship-apps, ship-site). A singleton that doesn't resolve
    /// from this repo (it isn't installed here) is simply skipped — the bake
    /// builds what's present and never hard-fails on a missing framework path.
    /// This is what lets `build-assets` / per-app `build` run outside the sail
    /// monorepo without the `DECK_FRAMEWORK_ASSETS_DIR` crutch.
    pub(crate) fn framework_singletons(&self) -> Vec<BundleTarget> {
        FRAMEWORK_SINGLETONS
            .iter()
            .filter_map(|s| {
                let entry = crate::node_resolve::resolve_entry(&self.repo_root, s.spec)?;
                Some(BundleTarget {
                    name: s.name.into(),
                    entry: TargetEntry::Real(entry),
                    externals: s.externals.iter().map(|e| (*e).to_string()).collect(),
                    out_file: s.out_file.into(),
                    lazy: false,
                })
            })
            .collect()
    }

    /// Consumer bundle targets, DISCOVERED from per-package self-declaration
    /// (`package.json` `deck.bundles[]`) — the dissolved central `deck.jsonc
    /// targets`. The package that owns the entry declares the bundle; the
    /// bundler walks the workspace to find them, the same way it discovers
    /// `routes/*`. Each declaration's `entry` is package-relative, joined onto
    /// the declaring package's root. The logical name drives `assets_dir_for`
    /// routing — every discovered consumer target lands in `.deck/` (the
    /// bake's staging area, sibling to the framework singletons).
    pub fn consumer_targets(&self) -> Vec<BundleTarget> {
        crate::catalog::discover_bundle_declarations(&self.repo_root)
            .into_iter()
            .map(|(pkg_root, d)| BundleTarget {
                name: d.name.clone(),
                entry: TargetEntry::Real(pkg_root.join(&d.entry)),
                externals: d.externals,
                out_file: format!("{}.js", d.name),
                lazy: d.lazy,
            })
            .collect()
    }

    /// The `routes/` bucket dirs the framework walks for route discovery —
    /// every `source_root` × `cfg.route_dirs` (defaults `["routes"]`; a host
    /// serving more buckets overrides it). One definition, two consumers:
    /// `discover_route_bundles`
    /// scans these for route *bundles*, and the route-table codegen
    /// (`<bucket>/routes.gen.ts` + `<bucket>/meta.gen.ts`, run from
    /// `build_all`) walks the same dirs. SSOT for "where routes live" — no
    /// second hardcoded path.
    pub fn route_bucket_dirs(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for root in &self.source_roots {
            for bucket in &self.cfg.route_dirs {
                out.push(root.join(bucket));
            }
        }
        out
    }

    /// One bundle per discovered route. The scan is fully config-driven
    /// (#273 — deck hardcodes no path):
    ///
    /// - For each `source_root`, walk each `cfg.route_dirs` bucket
    ///   (defaults to `["routes"]`; a host serving more buckets overrides
    ///   it) and look for a
    ///   `<bucket>/<name>/<route_leaf>` artifact. The leaf is
    ///   `cfg.route_leaf` (defaults to `page.sail`) — one convention for
    ///   all buckets, fixing the #254 `index.sail` vs `page.sail` divergence.
    /// - If `cfg.` is set, its `root` is treated as a
    ///   first-party route root too: `<root>/<name>/<route_leaf>`. The
    ///   host (e.g. ship) declares its own first-party app layout in
    ///   `deck.jsonc`; deck does not name the app.
    ///
    /// First-seen wins: earlier `route_dirs` entries (and earlier
    /// `source_roots`) shadow later matches.
    fn discover_route_bundles(&self) -> Vec<BundleTarget> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let leaf = self.cfg.route_leaf.as_deref().unwrap_or("page.sail");
        for dir in self.route_bucket_dirs() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if seen.contains(&name) || !e.path().is_dir() {
                    continue;
                }
                let entry = e.path().join(leaf);
                if !entry.exists() {
                    continue;
                }
                seen.insert(name.clone());
                out.push(BundleTarget {
                    name: format!("routes/{name}/route"),
                    entry: TargetEntry::Real(entry),
                    externals: vec![
                        "@ship/sail-runtime".into(),
                        "@ship/sail-runtime/internal".into(),
                        "@ship/sail-runtime/template-hooks".into(),
                        "@ship/sail-dom".into(),
                        "@ship/sdk-client".into(),
                    ],
                    out_file: format!("routes/{name}/route.js"),
                    lazy: false,
                });
            }
        }
        out
    }
}

impl BundleLayout {
    /// Targets a changed file affects (watch rebuild). Virtual aggregates match
    /// their explicit `sources` (or their dirs); real entries match the entry
    /// file or its package dir. Mirrors the worker's `targetsForFile`.
    pub fn targets_for_file(&self, abs: &Path) -> Vec<BundleTarget> {
        self.all_targets()
            .into_iter()
            .filter(|t| match &t.entry {
                TargetEntry::Virtual { sources } => sources
                    .iter()
                    .any(|s| abs == s || s.parent().is_some_and(|d| abs.starts_with(d))),
                TargetEntry::Real(entry) => {
                    abs == entry || entry.parent().is_some_and(|d| abs.starts_with(d))
                }
            })
            .collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    /// `inv-deck-output-location-single`: every `.deck` output path resolves
    /// through the ONE `deck_config::deck_out_path` join (the single declared
    /// constant), so the bundler WRITER and the runtime READER can never drift
    /// onto different directory names. This asserts the writer side equals the
    /// shared constant; the reader side is pinned by
    /// `deck_core::assets`'s route-bundle roundtrip test.
    #[test]
    fn deck_out_resolves_through_the_single_shared_constant() {
        let root = PathBuf::from("/proj");
        let writer = BundleLayout::with_config(root.clone(), None, DeckConfig::defaults()).deck_out();
        let shared = deck_config::deck_out_path(&root);
        assert_eq!(writer, shared, "the bundler writer resolves .deck via the shared join");
        assert_eq!(shared, root.join(deck_config::DECK_OUT_DIRNAME), "the shared join is root/<const>");
        assert_eq!(deck_config::DECK_OUT_DIRNAME, ".deck", "the one declared output-dir name");
        // The framework sub-dir hangs off the SAME join — never a second `.deck` root.
        let l = BundleLayout::with_config(root.clone(), None, DeckConfig::defaults());
        assert_eq!(l.assets_dir_for("sail-runtime"), shared.join("framework"));
    }

    #[test]
    fn assets_dir_routes_by_name() {
        // Framework singletons land in `.deck/framework/` (build-script staging).
        let l = BundleLayout::with_config(PathBuf::from("/repo"), None, DeckConfig::defaults());
        assert_eq!(l.assets_dir_for("sail-runtime"), PathBuf::from("/repo/.deck/framework"));
        assert_eq!(l.assets_dir_for("sail-dom"), PathBuf::from("/repo/.deck/framework"));
        assert_eq!(l.assets_dir_for("sail-sdk-client"), PathBuf::from("/repo/.deck/framework"));
        // Per-tag chrome (`sail-components/<name>`) lands in `.deck/`.
        // (The umbrella `sail-components` target no longer exists — chrome
        // is per-tag tree-shaking only.)
        assert_eq!(l.assets_dir_for("sail-components/sail-rail"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("routes/git/route"), PathBuf::from("/repo/.deck"));
        // Consumer targets (e.g. sail-marketing, sail-arcade, sail-pdf,
        // ship-code-editor) ALSO land in `.deck/` — the bake's staging
        // area, sibling to the framework singletons. No target is a host
        // asset; the bundler owns its output layout and every build
        // artifact belongs under `.deck/`.
        assert_eq!(l.assets_dir_for("sail-marketing"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("sail-arcade"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("ship-something"), PathBuf::from("/repo/.deck"));
    }

    #[test]
    fn discovers_route_bundles() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // A route.
        write(&root.join("routes/git/page.sail"), "<div>git</div>\n");
        // The framework singleton must be node-resolvable from the repo root for
        // its target to appear — install a stub `@ship/sail-runtime`.
        write(
            &root.join("node_modules/@ship/sail-runtime/package.json"),
            r#"{ "name": "@ship/sail-runtime", "exports": { ".": "./src/index.ts" } }"#,
        );
        write(&root.join("node_modules/@ship/sail-runtime/src/index.ts"), "export const x=1;\n");

        let l = BundleLayout::with_config(root, None, DeckConfig::defaults());
        let targets = l.all_targets();
        let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();

        assert!(names.contains(&"sail-runtime"), "node-resolved framework target present");
        assert!(names.contains(&"routes/git/route"), "discovered view");
    }

    /// A framework singleton that doesn't resolve from this repo (not installed)
    /// is skipped, not hard-errored — the bake builds what's present. This is
    /// the consumer-repo posture for a singleton it doesn't depend on.
    #[test]
    fn unresolvable_framework_singleton_is_skipped() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        write(&root.join("package.json"), r#"{ "name": "consumer" }"#);
        let l = BundleLayout::with_config(root, None, DeckConfig::defaults());
        let names: Vec<String> =
            l.all_targets().into_iter().map(|t| t.name).collect();
        assert!(
            !names.iter().any(|n| n == "sail-runtime"),
            "no framework target when the package isn't installed"
        );
    }

    /// `route_bucket_dirs()` is the single definition of "where routes live" —
    /// every `source_root` × `cfg.route_dirs`. The route-bundle scan AND the
    /// route-table codegen (`build_all`) both walk exactly these dirs, so a
    /// regression here would desync the bundle scan from the table gen.
    #[test]
    fn route_bucket_dirs_are_source_roots_times_route_dirs() {
        // Contributor mode: one source root.
        let l = BundleLayout::with_config(PathBuf::from("/repo"), None, DeckConfig::defaults());
        assert_eq!(
            l.route_bucket_dirs(),
            vec![PathBuf::from("/repo/routes")],
        );
        // External-project mode: repo_root + workspace_root, each × route_dirs.
        let l2 = BundleLayout::with_config(
            PathBuf::from("/repo"),
            Some(PathBuf::from("/ws")),
            DeckConfig::defaults(),
        );
        assert_eq!(
            l2.route_bucket_dirs(),
            vec![
                PathBuf::from("/repo/routes"),
                PathBuf::from("/ws/routes"),
            ],
        );
    }

    /// Consumer bundles are DISCOVERED from per-package `deck.bundles` — no
    /// central `targets` list. The declaration's package-relative `entry` is
    /// joined onto the declaring package's root, and `lazy` rides through onto
    /// the `BundleTarget`. This is the dissolution's core invariant.
    #[test]
    fn consumer_targets_discovered_from_package_self_declaration() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        write(&root.join("package.json"), r#"{ "workspaces": ["packages/*", "apps/*"] }"#);
        // An eager marketing pack (apps/site) + a lazy component pack.
        write(
            &root.join("apps/site/package.json"),
            r#"{ "name": "@ship/site", "deck": { "bundles": [
                { "name": "sail-marketing", "entry": "src/marketing/marketing-bundle.ts",
                  "externals": ["@ship/sail-runtime", "@ship/sail-dom"], "lazy": false } ] } }"#,
        );
        write(
            &root.join("packages/arcade/package.json"),
            r#"{ "name": "@ship/arcade", "deck": { "bundles": [
                { "name": "sail-arcade", "entry": "src/index.ts",
                  "externals": ["@ship/sail-runtime", "@ship/sail-dom"], "lazy": true } ] } }"#,
        );

        let l = BundleLayout::with_config(root.clone(), None, DeckConfig::defaults());
        let consumer = l.consumer_targets();

        let marketing = consumer.iter().find(|t| t.name == "sail-marketing").expect("marketing discovered");
        assert_eq!(
            marketing.entry,
            TargetEntry::Real(root.join("apps/site/src/marketing/marketing-bundle.ts")),
            "entry is package-relative, joined onto the declaring package root"
        );
        assert_eq!(marketing.externals, vec!["@ship/sail-runtime", "@ship/sail-dom"]);
        assert_eq!(marketing.out_file, "sail-marketing.js");
        assert!(!marketing.lazy, "marketing is eager");

        let arcade = consumer.iter().find(|t| t.name == "sail-arcade").expect("arcade discovered");
        assert!(arcade.lazy, "arcade's lazy intent rides onto the BundleTarget");

        // No central registry: discovery is the only source of consumer targets.
        assert_eq!(consumer.len(), 2, "exactly the two self-declared bundles");
    }
}
