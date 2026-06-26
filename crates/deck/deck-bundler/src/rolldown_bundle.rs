//! Rolldown bundler — drives Rolldown with the SUI plugin to bundle each target.
//! Output goes to `assets_dir_for` (sui-out/dist) — no source-tree writes.
//!
//! The SUI resolve/load plugin lives in `sui_plugin.rs`; this module owns the
//! target-to-bundle driver and its integration tests.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
use rolldown_common::{ManualCodeSplittingOptions, PreserveEntrySignatures};

use super::proto::BundleManifest;
use super::targets::{BundleLayout, BundleTarget, TargetEntry};
use crate::sui_plugin::{ResolverIndex, SuiPlugin, component_index};

/// Build one target with Rolldown. Output written under `assets_dir_for`;
/// returns the manifest. `staging` holds generated virtual-aggregate entries.
pub async fn bundle_target(
    layout: &BundleLayout,
    target: &BundleTarget,
    staging: &Path,
    production: bool,
) -> Result<BundleManifest> {
    // Entry: real `.ts`/`.sui` directly, or a generated `.ts` for virtual
    // aggregates importing each source's `.compiled.js` (so `load` compiles it).
    let entry: PathBuf = match &target.entry {
        TargetEntry::Real(p) => p.clone(),
        TargetEntry::Virtual { sources } => {
            std::fs::create_dir_all(staging)?;
            let mut imports = String::new();
            for s in sources {
                let compiled = s.with_extension("compiled.js");
                imports.push_str(&format!("import {:?};\n", compiled.to_string_lossy()));
            }
            let entry = staging.join(format!("_entry_{}.ts", target.name.replace('/', "_")));
            std::fs::write(&entry, imports)?;
            entry
        }
    };

    let out_dir = layout.assets_dir_for(&target.name);
    std::fs::create_dir_all(&out_dir)?;
    let entry_name = target.out_file.strip_suffix(".js").unwrap_or(&target.out_file);

    let options = BundlerOptions {
        input: Some(vec![InputItem {
            name: Some(entry_name.to_string()),
            import: entry.to_string_lossy().to_string(),
        }]),
        cwd: Some(layout.repo_root.clone()),
        dir: Some(out_dir.to_string_lossy().to_string()),
        format: Some(OutputFormat::Esm),
        minify: Some(production.into()),
        // Hoist every shared module into its own chunk (mirrors the multi-view
        // build in views_bundle.rs). Without this the single-view path uses
        // rolldown's default per-entry split, which produces 1 chunk per shared
        // primitive *per consumer* — N chunks for `sui-pagination` where 1
        // would do. Each duplicate chunk's `customElements.define()` adds a no-op
        // "called twice" warning at runtime; cumulative heap retention is the
        // 1.5 GB Firefox OOM. See .artifacts/2026-06-06-spaces-isolation.md.
        manual_code_splitting: Some(ManualCodeSplittingOptions {
            min_share_count: Some(1),
            min_size: Some(0.0),
            min_module_size: Some(0.0),
            ..Default::default()
        }),
        // Framework singletons (sui-runtime/sui-dom/sui-sdk-client) are libraries
        // consumed by name via the host importmap — preserve their full public
        // export surface. The default `ExportsOnly` tree-shook a re-exported const
        // (`DECK_API_ASSETS`) with no in-bundle consumer, producing a runtime
        // "does not provide an export named …" error. `Strict` makes the entry's
        // declared exports roots. App/view targets keep the default.
        preserve_entry_signatures: crate::targets::is_framework_singleton(&target.name)
            .then_some(PreserveEntrySignatures::Strict),
        ..Default::default()
    };

    // Per-view resolver (#255/#267): view builds emit `@sui-tag/<tag>` imports
    // that resolve to component source so the bundle carries exactly what the
    // view uses. Always on for views — the in-process compiler is the linked
    // crate, so there is no compiler-version drift to gate around. Framework/
    // app builds leave component_index empty (no `@sui-tag` imports emitted).
    let emit_component_imports = target.name.starts_with("views/");
    let resolver = if emit_component_imports {
        component_index(
            &layout.repo_root,
            &layout.cfg.packs,
        )
    } else {
        ResolverIndex::empty()
    };
    let plugin = SuiPlugin {
        repo_root: layout.repo_root.clone(),
        emit_component_imports,
        component_index: resolver.index,
        shadowed_tags: resolver.shadowed,
        component_roots: crate::catalog::component_package_roots(&layout.repo_root),
    };
    // The bundler + its oxc module graph live in `bundler` and are dropped at
    // the end of this scope. Earlier this called malloc_trim(0) via
    // crate::arena::trim_arenas() to return the freed arenas to glibc
    // (cut 4: arena.rs deleted; shipd now uses jemalloc, which returns
    // freed arenas to the OS by default — no trim required).
    let module_paths = {
        let mut bundler = BundlerBuilder::default()
            .with_options(options)
            .with_plugins(vec![Arc::new(plugin)])
            .build()
            .map_err(|e| anyhow!("rolldown build({}): {e:?}", target.name))?;
        let output = bundler
            .write()
            .await
            .map_err(|e| anyhow!("rolldown write({}): {e:?}", target.name))?;
        extract_module_paths(&output)
    };

    // Post-bundle tag-collision check (#308). Walks every emitted
    // chunk and errors on a tag defined by more than one file. Runs
    // unconditionally so a regression to the chunking config (e.g.
    // `manual_code_splitting.min_share_count`) fails the build
    // rather than silently re-introducing the `customElements.define
    // called twice` runtime warning + first-write-wins hijack.
    crate::tag_check::verify_no_duplicate_tag_defines(&out_dir)
        .map_err(|e| anyhow!("{} tag-collision check failed: {e}", target.name))?;
    let out_path = out_dir.join(&target.out_file);
    let bytes = std::fs::metadata(&out_path).map(|m| m.len() as usize).unwrap_or(0);
    Ok(BundleManifest {
        target: target.name.clone(),
        out_path: out_path.to_string_lossy().to_string(),
        bytes,
        inputs: match &target.entry {
            TargetEntry::Real(p) => vec![p.to_string_lossy().to_string()],
            TargetEntry::Virtual { sources } => {
                sources.iter().map(|p| p.to_string_lossy().to_string()).collect()
            }
        },
        modules: module_paths,
    })
}

const MODULE_PRUNE: &[&str] = &["node_modules", ".git", "target", "dist", ".deck", ".sui-out", "sui-out"];

fn extract_module_paths(output: &rolldown::BundleOutput) -> Vec<PathBuf> {
    use rolldown_common::Output;
    let mut paths = std::collections::HashSet::new();
    for asset in &output.assets {
        if let Output::Chunk(chunk) = asset {
            for module_id in &chunk.module_ids {
                let s: &str = module_id;
                if !s.starts_with('/') {
                    continue;
                }
                let p = PathBuf::from(s);
                if p.components().any(|c| {
                    matches!(c, std::path::Component::Normal(n)
                        if MODULE_PRUNE.contains(&n.to_string_lossy().as_ref()))
                }) {
                    continue;
                }
                paths.insert(p);
            }
        }
    }
    paths.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeckConfig;

    fn repo_root() -> PathBuf {
        let mut d = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        loop {
            if d.join("package.json").exists() && d.join("packages").is_dir() {
                return d;
            }
            if !d.pop() {
                panic!("repo root not found");
            }
        }
    }

    /// The `sui-marketing` bundle is now TRACED from the site-owned entry
    /// `apps/site/src/marketing/marketing-bundle.ts` (no hardcoded source list).
    /// Rolldown must follow that import graph and register exactly the marketing
    /// tag set — no tag dropped, and the full umbrella must NOT leak in.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_marketing_traced() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout =
            BundleLayout::with_config(repo, None, DeckConfig::defaults()).with_out_root(scratch.path().to_path_buf());

        let targets = layout.all_targets();
        let marketing = targets.iter().find(|t| t.name == "sui-marketing").unwrap();
        // Traced entry: a real on-disk site module, not a synthetic aggregate.
        assert!(
            matches!(&marketing.entry, TargetEntry::Real(p)
                if p.ends_with("apps/site/src/marketing/marketing-bundle.ts")),
            "sui-marketing must be a Real (traced) entry, got {:?}",
            marketing.entry
        );

        let m = bundle_target(&layout, marketing, staging.path(), true)
            .await
            .expect("marketing should bundle");
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        assert!(body.contains("@ship/sui-runtime"), "runtime kept external");

        // The traceable marketing components (sui-stat / sui-toolbar /
        // sui-progress from the retired const have no `.sui` source and are not
        // members — see marketing-bundle.ts). Each registers via its
        // `component(\`<tag>\`, …)` call; the tag literal survives minification.
        for tag in [
            "sui-icon", "sui-badge", "sui-button", "sui-separator",
            "sui-status-dot", "sui-table", "sui-bar-chart",
            "sui-sparkline", "sui-brand-mark", "sui-theme-picker",
        ] {
            assert!(
                body.contains(&format!("`{tag}`")),
                "marketing bundle must register {tag}"
            );
        }
        // Other-pack tags must NOT leak in (entry imports specific components,
        // not a package index). `ship-git-diff` lives in the @ship/code pack, not
        // anything the focused marketing entry imports.
        assert!(
            !body.contains("`ship-git-diff`"),
            "out-of-entry packs must not leak into the focused marketing bundle"
        );
    }

    /// The `sui-arcade` bundle is the heavy lazy chunk for the marketing site's
    /// games. Its `index.ts` registers every arcade tag and pulls the
    /// pixi + arcade-engine graph. pixi + arcade-engine must be BUNDLED (it's the
    /// big chunk), while the runtime singleton stays EXTERNAL — exactly one
    /// `@ship/sui-runtime` instance across the page.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_arcade_pixi_inline_runtime_external() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout =
            BundleLayout::with_config(repo, None, DeckConfig::defaults()).with_out_root(scratch.path().to_path_buf());

        let targets = layout.all_targets();
        let arcade = targets.iter().find(|t| t.name == "sui-arcade").unwrap();
        let m = bundle_target(&layout, arcade, staging.path(), true)
            .await
            .expect("arcade should bundle");
        // The heavy lazy chunk: pixi (~800 KB) is bundled in.
        assert!(m.bytes > 200_000, "arcade bundles pixi, got {} bytes", m.bytes);
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        // Runtime stays external (one instance) — an import of the bare
        // specifier survives, not an inlined second copy.
        assert!(
            body.contains("@ship/sui-runtime"),
            "arcade keeps the runtime singleton external"
        );
        // Each arcade tag registers via its `component(\`<tag>\`, …)` call.
        for tag in ["sui-stage", "sui-sprite", "sui-emitter", "sui-stage-input", "sui-hud"] {
            assert!(
                body.contains(&format!("`{tag}`")),
                "arcade bundle must register {tag}"
            );
        }
    }

    /// The `sui-pdf` bundle wraps the pdf.js (`pdfjs-dist`) engine. The pack's
    /// `index.ts` side-effect-imports the worker module
    /// (`pdfjs-dist/build/pdf.worker.mjs`) instead of a `?url` asset — the deck
    /// bundler has no `?url` support, so a `?url` import ABORTS the build (the bug
    /// this guards). Bundling the worker module straight in (it registers
    /// `globalThis.pdfjsWorker`) keeps the worker same-origin + offline with no
    /// served-asset URL. This test asserts the pack BUILDS through Rolldown, the
    /// engine + worker are inlined (heavy chunk), the runtime stays external, and
    /// `<ship-pdf>` registers.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_pdf_with_worker_inlined() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout =
            BundleLayout::with_config(repo, None, DeckConfig::defaults()).with_out_root(scratch.path().to_path_buf());

        let targets = layout.all_targets();
        let pdf = targets.iter().find(|t| t.name == "sui-pdf").unwrap();
        let m = bundle_target(&layout, pdf, staging.path(), true)
            .await
            .expect("pdf pack must bundle through Rolldown (no `?url` abort)");
        // The heavy lazy chunk: pdf.js engine + worker are inlined.
        assert!(m.bytes > 200_000, "pdf bundles the engine, got {} bytes", m.bytes);
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        // Runtime stays external (one instance).
        assert!(
            body.contains("@ship/sui-runtime"),
            "pdf keeps the runtime singleton external"
        );
        // The worker module is bundled, not fetched — it side-effect-registers the
        // main-thread handler on `globalThis.pdfjsWorker`. Its presence proves the
        // worker resolves SAME-ORIGIN with no served-asset URL and no CDN.
        assert!(
            body.contains("pdfjsWorker"),
            "pdf must bundle the worker module's main-thread handler registration"
        );
        // No CDN / `?url` leaked into the bundle.
        assert!(
            !body.contains("pdf.worker.mjs?url"),
            "the `?url` asset suffix must be gone (it aborts the deck bundler)"
        );
        for needle in ["cdnjs", "unpkg", "jsdelivr"] {
            assert!(!body.contains(needle), "worker must not load from a CDN ({needle})");
        }
        // `<ship-pdf>` registers via its `component(\`ship-pdf\`, …)` call.
        assert!(
            body.contains("`ship-pdf`"),
            "pdf bundle must register the ship-pdf tag"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn graph_capture_includes_transitive_dep_excludes_node_modules() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("lib")).unwrap();
        std::fs::write(root.join("app/main.ts"),
            "import { getX } from '../lib/util.ts'; export default getX;").unwrap();
        std::fs::write(root.join("lib/util.ts"), "export function getX() { return Math.random(); }").unwrap();

        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());
        let target = BundleTarget {
            name: "test/main".to_string(),
            entry: TargetEntry::Real(root.join("app/main.ts")),
            externals: vec![],
            out_file: "main.js".to_string(),
            lazy: false,
        };
        let manifest = bundle_target(&layout, &target, staging.path(), false).await.unwrap();
        assert!(manifest.modules.iter().any(|m| m.ends_with("app/main.ts")),
            "entry in module set, got: {:?}", manifest.modules);
        assert!(manifest.modules.iter().any(|m| m.ends_with("lib/util.ts")),
            "imported dep from different dir in module set, got: {:?}", manifest.modules);
        assert!(!manifest.modules.iter().any(|m| m.components().any(|c| {
            matches!(c, std::path::Component::Normal(n) if n == "node_modules")
        })), "node_modules must not appear in module set");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn app_source_coverage_outside_conventional_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app/routes")).unwrap();
        std::fs::create_dir_all(root.join("app/src")).unwrap();
        std::fs::write(root.join("app/routes/page.ts"),
            "import { getHelper } from '../src/helper.ts'; export default getHelper;").unwrap();
        std::fs::write(root.join("app/src/helper.ts"), "export function getHelper() { return Math.random().toString(); }").unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());
        let target = BundleTarget {
            name: "app/page".to_string(),
            entry: TargetEntry::Real(root.join("app/routes/page.ts")),
            externals: vec![],
            out_file: "page.js".to_string(),
            lazy: false,
        };
        let manifest = bundle_target(&layout, &target, staging.path(), false).await.unwrap();
        assert!(manifest.modules.iter().any(|m| m.ends_with("app/routes/page.ts")),
            "entry in module set");
        assert!(manifest.modules.iter().any(|m| m.ends_with("app/src/helper.ts")),
            "imported src file in module set — this is the apps/hub regression");
    }
}
