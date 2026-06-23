//! Bundle target resolution for the Rust bundler (#163).
//!
//! `TARGETS` + `discoverViewBundles` + `assetsDirFor` for the bundler. A
//! `BundleTarget` names what to build, where its entry is, what stays external,
//! and where the output lands. The Rolldown build (`rolldown_bundle.rs`)
//! iterates these; nothing here touches the filesystem beyond discovery reads.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use deck_config::DeckConfig;

/// Deck's OWN runtime bundles — baked into the deck-core crate (like Next
/// shipping its own runtime). Everything else under `sui-`/`views/` is app/user
/// content and goes to `.deck/`. Chrome components are NOT here — the
/// per-tag tree-shaking resolver (the `sui-components/<name>` prefix in
/// `assets_dir_for` + the `@sui-tag/<tag>` virtual import in `sui_plugin.rs`)
/// is the canonical way chrome loads. There is no umbrella.
const DECK_OWN_BUNDLES: [&str; 4] = ["sui-runtime", "sui-dom", "sui-sdk-client", "deck-route"];

/// True for deck's own framework singletons — libraries consumed by name via the
/// host importmap (`@ship/sdk-client` → `/_deck/sdk-client.js`, …). Their full
/// public export surface (including re-exported consts like `DECK_API_ASSETS`)
/// must survive tree-shaking, so the Rolldown build sets
/// `preserve_entry_signatures: Strict` for them. The default `ExportsOnly` DCE'd
/// a re-exported const with no in-bundle consumer, breaking the client at runtime
/// with "does not provide an export named …". App/view targets keep the default.
pub fn is_framework_singleton(name: &str) -> bool {
    DECK_OWN_BUNDLES.contains(&name)
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
    /// Logical name, e.g. `sui-runtime`, `sui-components/sui-rail`, `views/git/view`.
    pub name: String,
    pub entry: TargetEntry,
    /// Externals the author declared. The build phase additionally forces
    /// `@ship/sui-runtime` external for every target except `sui-runtime`.
    pub externals: Vec<String>,
    /// Output path relative to the target's assets dir (`assets_dir_for`).
    pub out_file: String,
    /// Lazy intent (from a consumer bundle's self-declaration). Only the
    /// optional dev-loop skip (`DECK_DEV_SKIP_LAZY=1`) reads it — it omits heavy
    /// lazy packs from a dev bake. Always `false` for framework + view targets.
    /// The default bake (env unset) builds lazy and eager identically, so this
    /// flag never changes production output.
    pub lazy: bool,
}

/// Repo + workspace layout. `repo_root` is the ship monorepo (framework +
/// built-in views); `source_roots` is `[repo_root]` in contributor mode or
/// `[repo_root, workspace_root]` when the daemon serves an external project.
pub struct BundleLayout {
    pub repo_root: PathBuf,
    pub source_roots: Vec<PathBuf>,
    /// Base dir output paths are computed under (the `crates/.../assets`, `.deck`,
    /// `apps/shipd/assets` subtrees hang off this). Defaults to `repo_root`;
    /// redirect for tests / out-of-tree builds so output never clobbers
    /// committed assets.
    pub out_root: PathBuf,
    /// View-discovery + watch config (`view_dirs`, `view_leaf`, `packs`,
    /// `watch`). Consumer bundle targets are NO LONGER here — they are
    /// discovered from per-package `deck.bundles` (see `consumer_targets`).
    /// Defaults to `DeckConfig::defaults()` for zero-config hosts.
    pub cfg: DeckConfig,
}

impl BundleLayout {
    /// Construct with an explicit [`DeckConfig`] — the only construction path.
    /// `DeckConfig` is host-supplied (the daemon calls [`DeckConfig::defaults`]
    /// plus any overrides it wants); the bundler does not load any file from
    /// the workspace root. `repo_root` is the host's project root (set via
    /// `set_project_root`); `workspace_root` distinguishes an external-project
    /// workspace from the framework repo for source-root resolution.
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
    /// area — `deck build-assets` writes framework + view + chrome
    /// bundles here, and `crates/deck-core/build.rs` copies the
    /// framework singletons into `crates/deck/deck-core/assets/` for
    /// `include_str!`. The framework's RUNTIME does not read this
    /// directory; the daemon serves singletons via `include_str!`
    /// constants and views on demand (view.rs). This is build-time
    /// orchestration only — `deck.jsonc` had no opinion here, and
    /// `set_project_root` does not own the path either (the bundler
    /// always writes under `<out_root>/.deck`, the bake's contract).
    pub fn deck_out(&self) -> PathBuf {
        self.out_root.join(".deck")
    }
    fn framework_dir(&self) -> PathBuf {
        self.deck_out().join("framework")
    }

    /// Output dir for a target by name. Every build artifact lands under
    /// `.deck/` — the bundler owns its output layout; the host / project
    /// root is never a bake target. Deck-own framework singletons go
    /// under `.deck/framework/` (the build-script staging path that
    /// `build.rs` copies into `crates/deck/deck-core/assets/` for
    /// `include_str!`); everything else — `views/*`, `sui-components/*`,
    /// and discovered consumer packs (`sui-marketing`, `sui-arcade`,
    /// `sui-pdf`, `ship-code-editor`, …) — goes directly under `.deck/`.
    pub fn assets_dir_for(&self, name: &str) -> PathBuf {
        if DECK_OWN_BUNDLES.contains(&name) {
            return self.framework_dir();
        }
        self.deck_out()
    }

    /// All targets: framework singletons + discovered consumer bundles + view bundles.
    pub fn all_targets(&self) -> Vec<BundleTarget> {
        let mut out = self.framework_singletons();
        out.extend(self.consumer_targets());
        out.extend(self.discover_view_bundles());
        out
    }

    /// Deck's OWN framework singletons — always built, hardcoded, never in
    /// deck.jsonc. These are the 4 runtime primitives deck ships with itself
    /// (like Next.js shipping its own runtime).
    fn framework_singletons(&self) -> Vec<BundleTarget> {
        let r = |rel: &str| self.repo_root.join(rel);
        vec![
            BundleTarget {
                name: "sui-runtime".into(),
                entry: TargetEntry::Real(r("packages/sui/runtime/src/index.ts")),
                externals: vec![],
                out_file: "sui-runtime.js".into(),
                lazy: false,
            },
            BundleTarget {
                name: "sui-dom".into(),
                entry: TargetEntry::Real(r("packages/sui/dom/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into()],
                out_file: "sui-dom.js".into(),
                lazy: false,
            },
            BundleTarget {
                name: "sui-sdk-client".into(),
                // TEMP: absolute path, becomes ../ship/... after the layout reshuffle.
                // @ship/sdk-client stayed in the ship repo (not carved out); the
                // sui-sdk-client framework singleton is bundled from its source there.
                entry: TargetEntry::Real(PathBuf::from(
                    "/home/madvib/dev/Ship/packages/sdk-client/src/index.ts",
                )),
                externals: vec!["@ship/sui-runtime".into()],
                out_file: "sui-sdk-client.js".into(),
                lazy: false,
            },
            // `@ship/deck/router` — the unified router singleton (defineRoutes /
            // route / navigate / mountRouter / mergeRoutes). Every per-app
            // `routes.gen.ts` imports `mountRouter` from it (ADR
            // unified-deck-router), the daemon serves it via `include_str!`
            // (assets.rs `DECK_ROUTE_JS`), and build.rs stages it from
            // `.deck/framework/`. It was dropped from the build target list in
            // 643543394 (deck.jsonc → discovery) but its source kept gaining
            // exports (mountRouter), so the served bundle went stale and lacked
            // them — breaking client hydration. Restored here as the 4th
            // singleton so the bake regenerates it. `is_framework_singleton`
            // (via DECK_OWN_BUNDLES) gives it `preserve_entry_signatures: Strict`
            // so re-exported handles can't be tree-shaken away.
            BundleTarget {
                name: "deck-route".into(),
                entry: TargetEntry::Real(r("packages/deck/router/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into()],
                out_file: "deck-route.js".into(),
                lazy: false,
            },
        ]
    }

    /// Consumer bundle targets, DISCOVERED from per-package self-declaration
    /// (`package.json` `deck.bundles[]`) — the dissolved central `deck.jsonc
    /// targets`. The package that owns the entry declares the bundle; the
    /// bundler walks the workspace to find them, the same way it discovers
    /// `views/*`. Each declaration's `entry` is package-relative, joined onto
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

    /// The `views/` bucket dirs the framework walks for view discovery —
    /// every `source_root` × `cfg.view_dirs` (defaults `["views",
    /// ".ship/views"]`). One definition, two consumers: `discover_view_bundles`
    /// scans these for view *bundles*, and the view-table codegen
    /// (`<bucket>/routes.gen.ts` + `<bucket>/meta.gen.ts`, run from
    /// `build_all`) walks the same dirs. SSOT for "where views live" — no
    /// second hardcoded path.
    pub fn view_bucket_dirs(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for root in &self.source_roots {
            for bucket in &self.cfg.view_dirs {
                out.push(root.join(bucket));
            }
        }
        out
    }

    /// One bundle per discovered view. The scan is fully config-driven
    /// (#273 — deck hardcodes no path):
    ///
    /// - For each `source_root`, walk each `cfg.view_dirs` bucket
    ///   (defaults to `["views", ".ship/views"]`) and look for a
    ///   `<bucket>/<name>/<view_leaf>` artifact. The leaf is
    ///   `cfg.view_leaf` (defaults to `page.sui`) — one convention for
    ///   all buckets, fixing the #254 `index.sui` vs `page.sui` divergence.
    /// - If `cfg.` is set, its `root` is treated as a
    ///   first-party view root too: `<root>/<name>/<view_leaf>`. The
    ///   host (e.g. ship) declares its own first-party app layout in
    ///   `deck.jsonc`; deck does not name the app.
    ///
    /// First-seen wins: earlier `view_dirs` entries (and earlier
    /// `source_roots`) shadow later matches.
    fn discover_view_bundles(&self) -> Vec<BundleTarget> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let leaf = self.cfg.view_leaf.as_deref().unwrap_or("page.sui");
        for dir in self.view_bucket_dirs() {
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
                    name: format!("views/{name}/view"),
                    entry: TargetEntry::Real(entry),
                    externals: vec![
                        "@ship/sui-runtime".into(),
                        "@ship/sui-runtime/template-hooks".into(),
                        "@ship/sui-dom".into(),
                        "@ship/sdk-client".into(),
                    ],
                    out_file: format!("views/{name}/view.js"),
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

    #[test]
    fn assets_dir_routes_by_name() {
        // Framework singletons land in `.deck/framework/` (build-script staging).
        let l = BundleLayout::with_config(PathBuf::from("/repo"), None, DeckConfig::defaults());
        assert_eq!(l.assets_dir_for("sui-runtime"), PathBuf::from("/repo/.deck/framework"));
        assert_eq!(l.assets_dir_for("sui-dom"), PathBuf::from("/repo/.deck/framework"));
        assert_eq!(l.assets_dir_for("sui-sdk-client"), PathBuf::from("/repo/.deck/framework"));
        // Per-tag chrome (`sui-components/<name>`) lands in `.deck/`.
        // (The umbrella `sui-components` target no longer exists — chrome
        // is per-tag tree-shaking only.)
        assert_eq!(l.assets_dir_for("sui-components/sui-rail"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("views/git/view"), PathBuf::from("/repo/.deck"));
        // Consumer targets (e.g. sui-marketing, sui-arcade, sui-pdf,
        // ship-code-editor) ALSO land in `.deck/` — the bake's staging
        // area, sibling to the framework singletons. No target is a host
        // asset; the bundler owns its output layout and every build
        // artifact belongs under `.deck/`.
        assert_eq!(l.assets_dir_for("sui-marketing"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("sui-arcade"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("ship-something"), PathBuf::from("/repo/.deck"));
    }

    #[test]
    fn discovers_view_bundles() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // A view.
        write(&root.join("views/git/page.sui"), "<div>git</div>\n");

        let l = BundleLayout::with_config(root, None, DeckConfig::defaults());
        let targets = l.all_targets();
        let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();

        assert!(names.contains(&"sui-runtime"), "static framework target present");
        assert!(names.contains(&"views/git/view"), "discovered view");
    }

    /// `view_bucket_dirs()` is the single definition of "where views live" —
    /// every `source_root` × `cfg.view_dirs`. The view-bundle scan AND the
    /// view-table codegen (`build_all`) both walk exactly these dirs, so a
    /// regression here would desync the bundle scan from the table gen.
    #[test]
    fn view_bucket_dirs_are_source_roots_times_view_dirs() {
        // Contributor mode: one source root.
        let l = BundleLayout::with_config(PathBuf::from("/repo"), None, DeckConfig::defaults());
        assert_eq!(
            l.view_bucket_dirs(),
            vec![PathBuf::from("/repo/views"), PathBuf::from("/repo/.ship/views")],
        );
        // External-project mode: repo_root + workspace_root, each × view_dirs.
        let l2 = BundleLayout::with_config(
            PathBuf::from("/repo"),
            Some(PathBuf::from("/ws")),
            DeckConfig::defaults(),
        );
        assert_eq!(
            l2.view_bucket_dirs(),
            vec![
                PathBuf::from("/repo/views"),
                PathBuf::from("/repo/.ship/views"),
                PathBuf::from("/ws/views"),
                PathBuf::from("/ws/.ship/views"),
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
                { "name": "sui-marketing", "entry": "src/marketing/marketing-bundle.ts",
                  "externals": ["@ship/sui-runtime", "@ship/sui-dom"], "lazy": false } ] } }"#,
        );
        write(
            &root.join("packages/arcade/package.json"),
            r#"{ "name": "@ship/arcade", "deck": { "bundles": [
                { "name": "sui-arcade", "entry": "src/index.ts",
                  "externals": ["@ship/sui-runtime", "@ship/sui-dom"], "lazy": true } ] } }"#,
        );

        let l = BundleLayout::with_config(root.clone(), None, DeckConfig::defaults());
        let consumer = l.consumer_targets();

        let marketing = consumer.iter().find(|t| t.name == "sui-marketing").expect("marketing discovered");
        assert_eq!(
            marketing.entry,
            TargetEntry::Real(root.join("apps/site/src/marketing/marketing-bundle.ts")),
            "entry is package-relative, joined onto the declaring package root"
        );
        assert_eq!(marketing.externals, vec!["@ship/sui-runtime", "@ship/sui-dom"]);
        assert_eq!(marketing.out_file, "sui-marketing.js");
        assert!(!marketing.lazy, "marketing is eager");

        let arcade = consumer.iter().find(|t| t.name == "sui-arcade").expect("arcade discovered");
        assert!(arcade.lazy, "arcade's lazy intent rides onto the BundleTarget");

        // No central registry: discovery is the only source of consumer targets.
        assert_eq!(consumer.len(), 2, "exactly the two self-declared bundles");
    }
}
