//! Multi-entry route bundling with cross-route shared-chunk dedup (#267 wave 0,
//! capability gap #2). The per-route resolver (gap #1) makes each route import its
//! components; building all routes in ONE Rolldown pass with shared-chunk
//! splitting means a component used by N routes lands in ONE shared chunk — not
//! inlined N times. That is the dedup the global umbrella does today, so it must
//! exist before the umbrella can be deleted (wave 1).
//!
//! The daemon's single-route cold-miss (`rolldown_bundle::bundle_target`) stays
//! single-entry: a dev fallback with no cross-route dedup. This is the bake/
//! production path.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
use rolldown_common::ManualCodeSplittingOptions;

use crate::sail_plugin::{SailPlugin, component_index};
use crate::targets::{BundleLayout, BundleTarget, TargetEntry};

/// Bundle every route in `views` together so a component shared by ≥2 routes
/// dedups into one shared chunk. Output lands under the routes' `assets_dir_for`
/// (`.deck`). Always runs the per-route resolver (it is the whole point — the
/// routes must import their components for there to be anything to dedup). The
/// oxc graph is dropped + arenas trimmed before returning (#256), though the
/// multi-entry graph is necessarily larger than a single route's.
pub async fn bundle_routes(
    layout: &BundleLayout,
    views: &[BundleTarget],
    staging: &Path,
    production: bool,
) -> Result<()> {
    if views.is_empty() {
        return Ok(());
    }
    let out_dir = layout.assets_dir_for(&views[0].name);
    let stage_dir = crate::atomic::stage_dir_for(&out_dir, "routes");
    std::fs::create_dir_all(&out_dir)?;
    std::fs::create_dir_all(&stage_dir)?;
    std::fs::create_dir_all(staging)?;

    let mut input = Vec::with_capacity(views.len());
    for t in views {
        let import = match &t.entry {
            TargetEntry::Real(p) => p.to_string_lossy().to_string(),
            TargetEntry::Virtual { sources } => {
                // Routes aren't virtual aggregates; synthesize a side-effect entry.
                let entry = staging.join(format!("_view_{}.ts", t.name.replace('/', "_")));
                let body: String = sources
                    .iter()
                    .map(|s| {
                        format!(
                            "import {:?};\n",
                            s.with_extension("compiled.js").to_string_lossy()
                        )
                    })
                    .collect();
                std::fs::write(&entry, body)?;
                entry.to_string_lossy().to_string()
            }
        };
        input.push(InputItem {
            name: Some(
                t.out_file
                    .strip_suffix(".js")
                    .unwrap_or(&t.out_file)
                    .to_string(),
            ),
            import,
        });
    }

    let options = BundlerOptions {
        input: Some(input),
        cwd: Some(layout.repo_root.clone()),
        dir: Some(stage_dir.to_string_lossy().to_string()),
        format: Some(OutputFormat::Esm),
        minify: Some(production.into()),
        // `min_share_count: 1` (was 2): hoist every shared module into its
        // own chunk. With 2, a primitive used by exactly one route got inlined
        // into that route, and used-by-many primitives could still produce
        // content-different chunks (rolldown's per-usage import-order
        // content hash differs from the source's standalone hash). With 1,
        // every shared dep is hoisted to a single content-addressed chunk
        // regardless of how many routes use it. See
        // .artifacts/2026-06-06-spaces-isolation.md for the duplication
        // evidence (35 sail-pagination-*.js chunks pre-fix).
        manual_code_splitting: Some(ManualCodeSplittingOptions {
            min_share_count: Some(1),
            min_size: Some(0.0),
            min_module_size: Some(0.0),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = component_index(&layout.repo_root, &layout.cfg.packs);
    let plugin = SailPlugin {
        repo_root: layout.repo_root.clone(),
        emit_component_imports: true,
        component_index: resolver.index,
        shadowed_tags: resolver.shadowed,
        component_roots: crate::catalog::component_package_roots(&layout.repo_root),
        project_dir: None,
    };
    {
        let mut bundler = BundlerBuilder::default()
            .with_options(options)
            .with_plugins(vec![Arc::new(plugin)])
            .build()
            .map_err(|e| anyhow!("rolldown build(views multi-entry): {e:?}"))?;
        let output = bundler
            .write()
            .await
            .map_err(|e| anyhow!("rolldown write(views multi-entry): {e:?}"))?;
        crate::rolldown_bundle::assert_no_unresolved_imports(&output, "routes")?;
    }

    // Publish atomically from the staging dir to the live dir.
    crate::atomic::publish_dir(&stage_dir, &out_dir).with_context(|| "publish route bundles")?;
    // (cut 4: arena.rs deleted; the malloc_trim(0) call here was the
    // glibc band-aid. shipd now uses jemalloc, which returns freed
    // arenas to the OS by default.)
    Ok(())
}
