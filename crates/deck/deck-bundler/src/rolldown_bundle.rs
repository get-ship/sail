//! Rolldown bundler — drives Rolldown with the SAIL plugin to bundle each target.
//! Output goes to `assets_dir_for` (under `.deck/` / dist) — no source-tree writes.
//!
//! The SAIL resolve/load plugin lives in `sail_plugin.rs`; this module owns the
//! target-to-bundle driver and its integration tests.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
use rolldown_common::{ManualCodeSplittingOptions, PreserveEntrySignatures};

use super::proto::BundleManifest;
use super::targets::{BundleLayout, BundleTarget, TargetEntry};
use crate::sail_plugin::{ResolverIndex, SailPlugin, component_index};

/// Build one target with Rolldown. Output written under `assets_dir_for`;
/// returns the manifest. `staging` holds generated virtual-aggregate entries.
pub async fn bundle_target(
    layout: &BundleLayout,
    target: &BundleTarget,
    staging: &Path,
    production: bool,
) -> Result<BundleManifest> {
    // Entry: real `.ts`/`.sail` directly, or a generated `.ts` for virtual
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
    let stage_dir = crate::atomic::stage_dir_for(&out_dir, &target.name.replace('/', "_"));
    std::fs::create_dir_all(&out_dir)?;
    std::fs::create_dir_all(&stage_dir)?;
    let entry_name = target
        .out_file
        .strip_suffix(".js")
        .unwrap_or(&target.out_file);

    let options = BundlerOptions {
        input: Some(vec![InputItem {
            name: Some(entry_name.to_string()),
            import: entry.to_string_lossy().to_string(),
        }]),
        cwd: Some(layout.repo_root.clone()),
        dir: Some(stage_dir.to_string_lossy().to_string()),
        format: Some(OutputFormat::Esm),
        minify: Some(production.into()),
        // The target's DECLARED externals (e.g. `@ship/sail-compiler-wasm` for
        // sail-lsp-diagnostics — importmap-covered framework singletons) are
        // applied here. Before this they were recorded on the target but never
        // handed to Rolldown, so a declared external externalized ONLY by
        // accident — through rolldown's unresolved-import fallback ("Module
        // not found, treating it as an external dependency") — which the
        // `assert_no_unresolved_imports` guard below now treats as a build
        // failure. Declaration, not resolution luck, decides what stays bare.
        external: (!target.externals.is_empty()).then(|| target.externals.clone().into()),
        // Hoist every shared module into its own chunk (mirrors the multi-route
        // build in routes_bundle.rs). Without this the single-route path uses
        // rolldown's default per-entry split, which produces 1 chunk per shared
        // primitive *per consumer* — N chunks for `sail-pagination` where 1
        // would do. Each duplicate chunk's `customElements.define()` adds a no-op
        // "called twice" warning at runtime; cumulative heap retention is the
        // 1.5 GB Firefox OOM. See .artifacts/2026-06-06-spaces-isolation.md.
        manual_code_splitting: Some(ManualCodeSplittingOptions {
            min_share_count: Some(1),
            min_size: Some(0.0),
            min_module_size: Some(0.0),
            ..Default::default()
        }),
        // Framework singletons (sail-runtime/sail-dom/sail-sdk-client) are libraries
        // consumed by name via the host importmap — preserve their full public
        // export surface. The default `ExportsOnly` tree-shook a re-exported const
        // (`DECK_API_ASSETS`) with no in-bundle consumer, producing a runtime
        // "does not provide an export named …" error. `Strict` makes the entry's
        // declared exports roots. App/route targets keep the default.
        preserve_entry_signatures: crate::targets::is_framework_singleton(&target.name)
            .then_some(PreserveEntrySignatures::Strict),
        ..Default::default()
    };

    // Per-route resolver (#255/#267): route builds emit `@sail-tag/<tag>` imports
    // that resolve to component source so the bundle carries exactly what the
    // route uses. Always on for routes — the in-process compiler is the linked
    // crate, so there is no compiler-version drift to gate around. Framework/
    // app builds leave component_index empty (no `@sail-tag` imports emitted).
    let emit_component_imports = target.name.starts_with("routes/");
    let resolver = if emit_component_imports {
        component_index(&layout.repo_root, &layout.cfg.packs)
    } else {
        ResolverIndex::empty()
    };
    let plugin = SailPlugin {
        repo_root: layout.repo_root.clone(),
        emit_component_imports,
        component_index: resolver.index,
        shadowed_tags: resolver.shadowed,
        component_roots: crate::catalog::component_package_roots(&layout.repo_root),
        project_dir: None,
    };
    // The bundler + its oxc module graph live in `bundler` and are dropped at
    // the end of this scope. Earlier this called malloc_trim(0) via
    // crate::arena::trim_arenas() to return the freed arenas to glibc
    // (cut 4: arena.rs deleted; shipd now uses jemalloc, which returns
    // freed arenas to the OS by default — no trim required).
    let module_paths = {
        let mut bundler = BundlerBuilder::default()
            .with_options(options)
            .with_plugins(vec![
                Arc::new(plugin),
                Arc::new(if production {
                    crate::ship_asset_css::ShipAssetCssPlugin::production()
                } else {
                    crate::ship_asset_css::ShipAssetCssPlugin::development()
                }),
            ])
            .build()
            .map_err(|e| anyhow!("rolldown build({}): {e:?}", target.name))?;
        let output = bundler
            .write()
            .await
            .map_err(|e| anyhow!("rolldown write({}): {e:?}", target.name))?;
        assert_no_unresolved_imports(&output, &target.name)?;
        extract_module_paths(&output)
    };

    // Publish atomically from the staging dir to the live dir. A reader of
    // .deck/framework/ never sees a partially-written bundle.
    crate::atomic::publish_dir(&stage_dir, &out_dir)
        .with_context(|| format!("publish bundle for {}", target.name))?;

    // Post-bundle tag-collision check (#308). Walks every emitted
    // chunk and errors on a tag defined by more than one file. Runs
    // unconditionally so a regression to the chunking config (e.g.
    // `manual_code_splitting.min_share_count`) fails the build
    // rather than silently re-introducing the `customElements.define
    // called twice` runtime warning + first-write-wins hijack.
    crate::tag_check::verify_no_duplicate_tag_defines(&out_dir)
        .map_err(|e| anyhow!("{} tag-collision check failed: {e}", target.name))?;
    // Post-bundle duplicate-package-version WARNING (#89 item 3, diagnostic
    // only — never changes what gets bundled). See `dup_version_check` for
    // the CodeMirror 6.7.0/6.7.1 case this catches.
    crate::dup_version_check::warn_duplicate_versions(&module_paths);
    let out_path = out_dir.join(&target.out_file);
    let bytes = std::fs::metadata(&out_path)
        .map(|m| m.len() as usize)
        .unwrap_or(0);
    let dropped_side_effect_modules = detect_dropped_side_effect_modules(&module_paths, &entry);
    // Emit a warning per drop. The manifest already records the drops
    // (so callers and the watcher can surface them), but the spec also
    // requires a "no silent drop" rule: a CSS or `customElements.define`
    // import that vanished from the bundle must show up in the build
    // log with the dropped module + the importer that pulled it in.
    // The reason is the `package.json#sideEffects` value (false or
    // array form) so the author can fix the contract on either side.
    for d in &dropped_side_effect_modules {
        tracing::warn!(
            target: "deck-bundler",
            bundle = %target.name,
            module = %d.module_path.display(),
            importer = %d.importer,
            reason = %d.reason,
            "deck-bundler: dropped side-effect-only module — its body was \
             tree-shaken per the importing package's `sideEffects: false` \
             contract. CSS imports and `customElements.define` calls live \
             here; if a registration is missing at runtime, this is why."
        );
    }
    Ok(BundleManifest {
        target: target.name.clone(),
        out_path: out_path.to_string_lossy().to_string(),
        bytes,
        inputs: match &target.entry {
            TargetEntry::Real(p) => vec![p.to_string_lossy().to_string()],
            TargetEntry::Virtual { sources } => sources
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
        },
        modules: module_paths,
        dropped_side_effect_modules,
    })
}

/// Fail the build when Rolldown could not resolve an import and silently
/// externalized it. Rolldown reports an unresolvable BARE specifier as an
/// `UNRESOLVED_IMPORT` *warning* ("Module not found, treating it as an
/// external dependency") and ships the specifier verbatim in the emitted
/// chunk. The document importmap only covers deck's framework singletons, so
/// the browser dies at module-instantiation time — `Failed to resolve module
/// specifier "<spec>"` — on EVERY route: a blank page from a build that
/// reported success. The class case is dependencies missing at bake time —
/// a fresh clone that ran the gen/dev step before `bun install`, or an
/// install predating a newly-added dependency (`motion/mini` in
/// `@ship/sail-runtime` shipped exactly this way). Deliberate externals never
/// reach this guard: deck's resolve hooks mark them `external: true`, so
/// Rolldown never warns on them. (An unresolvable PATH-LIKE specifier is
/// already a hard error inside Rolldown; this guard covers the bare-import
/// half.)
pub(crate) fn assert_no_unresolved_imports(
    output: &rolldown::BundleOutput,
    target: &str,
) -> Result<()> {
    let unresolved: Vec<String> = output
        .warnings
        .iter()
        // `EventKind` has no `PartialEq` in rolldown_error 1.0; its `Display`
        // IS the stable diagnostic code rolldown prints ("UNRESOLVED_IMPORT").
        .filter(|w| w.kind().to_string() == "UNRESOLVED_IMPORT")
        .map(|w| w.to_string())
        .collect();
    if unresolved.is_empty() {
        return Ok(());
    }
    bail!(
        "bundle({target}): {} import(s) did not resolve and would ship as bare \
         specifiers no importmap covers — the browser would fail with \
         `Failed to resolve module specifier` on every route:\n  {}\nA \
         dependency the bundle inlines is missing from node_modules — run \
         `bun install` at the workspace root and rebuild.",
        unresolved.len(),
        unresolved.join("\n  "),
    );
}

/// Detect side-effect-only modules that the bundler dropped (a bare
/// `import '…'` whose target is not in `module_paths`). The recursion
/// walks the entry's import graph on disk and reports every bare import
/// whose resolved file is absent from the bundle — provided the target
/// itself is side-effect-only on disk (no `export` statements). This is
/// the spec's "no silent drop" rule: a CSS or `customElements.define`
/// import that vanished from the bundle must show up in
/// `BundleManifest.dropped_side_effect_modules` so the build can
/// `tracing::warn!` with module + importer + reason.
///
/// **Path canonicalization.** Rolldown emits `module_ids` as
/// `std::fs::canonicalize`d absolute paths (no `.` or `..` segments).
/// A naive `Path::join("..")` produces a non-canonical path that never
/// matches. We canonicalize every resolved path before comparing.
fn detect_dropped_side_effect_modules(
    module_paths: &[PathBuf],
    entry: &Path,
) -> Vec<super::proto::DroppedSideEffectModule> {
    use std::collections::HashSet;
    use std::fs;
    let kept: HashSet<PathBuf> = module_paths
        .iter()
        .filter_map(|p| canonicalize(p))
        .collect();
    let entry_canon = canonicalize(entry).unwrap_or_else(|| entry.to_path_buf());
    let mut dropped: Vec<super::proto::DroppedSideEffectModule> = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();
    // The entry is the root of the walk. Its "importer" is itself
    // (so the emitted warning names the file that pulled in the
    // dropped side-effect-only module — empty would be a footgun in
    // the opposite direction, a warning with no caller to attribute
    // the drop to).
    let mut stack: Vec<(PathBuf, PathBuf)> = vec![(entry_canon.clone(), entry_canon.clone())];
    while let Some((path, importer)) = stack.pop() {
        if !visited.insert(path.clone()) {
            continue;
        }
        let body = match fs::read_to_string(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        for line in body.lines() {
            // A bare side-effect import: `import '…';` — no default,
            // no namespace, no destructuring. Anything else (`import x from`,
            // `import * as`, `import {a}`) keeps a binding the importer
            // uses, so the drop is intentional and not a footgun.
            let trimmed = line.trim_start();
            let Some(after_kw) = trimmed
                .strip_prefix("import ")
                .or_else(|| trimmed.strip_prefix("import\t"))
            else {
                continue;
            };
            let spec = match parse_bare_import_spec(after_kw) {
                Some(s) => s,
                None => continue,
            };
            let Some(resolved) = resolve_import(&path, &spec) else {
                continue;
            };
            // Skip asset extensions (CSS / JSON / SVG / image /
            // font). These don't appear in `module_paths` because
            // they are emitted by the built-in AssetModulePlugin
            // as hashed `assets/<name>-<hash>.<ext>` files, NOT
            // tracked as JS module_ids in any chunk. Without
            // this filter, a bare `import './x.css'` would
            // resolve to the source file, find it missing from
            // `kept`, and emit a spurious "dropped side-effect
            // module" warning — even though the CSS is correctly
            // emitted as an asset and the import is a deliberate
            // side-effect request for styles, not JS.
            //
            // The list mirrors what the deck bundler hands to
            // the AssetModulePlugin via `module_types` (the
            // `.css` → Asset mapping in `app.rs` / the per-route
            // build). The detector's contract is "JS side-effect
            // imports" — assets are out of scope; their absence
            // from the JS chunk graph is expected, not a bug.
            if is_asset_extension(&resolved) {
                continue;
            }
            let resolved_canon = match canonicalize(&resolved) {
                Some(p) => p,
                None => continue,
            };
            if !kept.contains(&resolved_canon) {
                // The target file is absent from the bundle. A bare
                // `import '…'` whose target is missing means the
                // author asked for a side effect and the bundler
                // didn't deliver. Report it regardless of whether
                // import form (should be a named import) AND in the
                // bundler's analysis (the export should have been
                // shaken; the body is gone; the side effect is
                // gone). Either way, the author needs the signal.
                // The reason is the nearest `package.json#sideEffects`
                // (or a generic "no package.json" message) so the
                // author can act on it.
                let reason = side_effects_reason(&resolved_canon);
                dropped.push(super::proto::DroppedSideEffectModule {
                    module_path: resolved_canon,
                    importer: importer.to_string_lossy().to_string(),
                    reason,
                });
                continue;
            }
            if !visited.contains(&resolved_canon) {
                stack.push((resolved_canon, path.clone()));
            }
        }
    }
    // Dedup by (module_path, importer) so the warning emission sees one
    // row per dropped module even when the same target is reached via
    // two paths.
    let mut seen: HashSet<(PathBuf, String)> = HashSet::new();
    dropped.retain(|d| seen.insert((d.module_path.clone(), d.importer.clone())));
    dropped
}

/// Canonicalize a path; on failure (nonexistent file), clean `.` /
/// `..` segments without touching the filesystem so the comparison
/// against `module_paths` (which are also canonical) works.
fn canonicalize(p: &Path) -> Option<PathBuf> {
    if let Ok(c) = std::fs::canonicalize(p) {
        return Some(c);
    }
    // Best-effort lexical normalization — `path-clean`-equivalent
    // for the filesystem-lookup fallback in `resolve_import`.
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    Some(out)
}

/// Parse the specifier from a bare `import '…';` statement. The
/// input is the source text after the `import` keyword (already
/// trimmed). Returns `None` for any non-bare form (default, namespace,
/// destructured) so the caller can skip it.
fn parse_bare_import_spec(after_kw: &str) -> Option<String> {
    let rest = after_kw.trim_start();
    // Bare: starts with a quote. Anything else (`x from`, `* as`,
    // `{a, b}`) is a binding import — not what we want.
    if rest.starts_with('\'') || rest.starts_with('"') {
        let quote = rest.chars().next().unwrap();
        let end = rest[1..].find(quote)?;
        Some(rest[1..1 + end].to_string())
    } else {
        None
    }
}

/// True when `path`'s extension is one the deck bundler hands to
/// the built-in `AssetModulePlugin` via the `module_types` map
/// (currently `.css`; `.json` is reserved for the same pipeline
/// in case a future mapping adds it). The detector skips these
/// so a bare `import './x.css'` doesn't get reported as a
/// dropped side-effect module — the asset pipeline emits it
/// separately as `assets/<name>-<hash>.css` and the
/// `module_paths` it doesn't appear in is expected, not a bug.
/// Image / font / SVG are pre-emptively listed (they aren't
/// mapped today, but if/when they are, the detector stays
/// correct without a per-extension patch).
fn is_asset_extension(path: &Path) -> bool {
    const ASSET_EXTS: &[&str] = &[
        "css", "json", "svg", "png", "jpg", "jpeg", "gif", "webp", "avif", "ico", "woff", "woff2",
        "ttf", "otf", "eot",
    ];
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| ASSET_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Supports relative paths (`.`, `..`, absolute) and the
/// `@ship/<name>` workspace map (the `SailPlugin` resolves bare
/// specifiers through this map). Anything else is left to the caller.
fn resolve_import(importer: &Path, spec: &str) -> Option<PathBuf> {
    use std::path::PathBuf;
    let p: PathBuf = if spec.starts_with("./") || spec.starts_with("../") {
        importer.parent()?.join(spec)
    } else if spec.starts_with('/') {
        PathBuf::from(spec)
    } else if let Some(rest) = spec.strip_prefix("@ship/") {
        // The SailPlugin re-anchors `@ship/<name>` to the workspace
        // package's `src/index.ts`. The detector uses the same map
        // (via `discover_workspace_packages`) so the warning names
        // the exact file the plugin resolved.
        let name = rest.split('/').next()?.to_string();
        return resolve_ship_package(importer, &format!("@ship/{name}"));
    } else {
        return None;
    };
    // Try the obvious extensions the bundler accepts.
    let candidates: &[&str] = &[
        "",
        ".ts",
        ".tsx",
        ".js",
        ".mjs",
        ".cjs",
        "/index.ts",
        "/index.js",
    ];
    for ext in candidates {
        let candidate = if ext.is_empty() {
            p.clone()
        } else {
            p.with_extension(&ext[1..])
        };
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    // The path may not exist on disk in this fixture (rare); return
    // the guessed `.ts` form so the warning can still name it.
    Some(p.with_extension("ts"))
}

/// Resolve `@ship/<name>` against the workspace by walking up from the
/// importer looking for a `package.json` that declares a `workspaces`
/// field (the repo root), then asking the catalog for the package
/// map (`@ship/<name>` → dir) and reading `src/index.ts`. This is the
/// same map `SailPlugin::resolve_id` uses to re-anchor bare specifiers,
/// so the drop detector stays in lockstep with the live resolver.
fn resolve_ship_package(importer: &Path, name: &str) -> Option<PathBuf> {
    let repo_root = find_workspace_root(importer)?;
    let roots = crate::catalog::discover_workspace_packages(&repo_root).unwrap_or_default();
    let map: std::collections::HashMap<String, std::path::PathBuf> =
        roots.into_iter().map(|p| (p.name, p.root)).collect();
    let dir = map.get(name)?;
    let idx = dir.join("src/index.ts");
    if idx.is_file() {
        Some(idx)
    } else {
        Some(dir.join("index.ts"))
    }
}

/// Walk up from `start` looking for a directory whose `package.json`
/// declares a `workspaces` field. That directory is the workspace root
/// `SailPlugin` reads via `component_package_roots`. Returns the first
/// such ancestor; the test fixture's own package.json sets
/// `workspaces: [...]` so this resolves to the temp root.
fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut dir: Option<&Path> = start.parent();
    while let Some(d) = dir {
        let pj = d.join("package.json");
        if pj.is_file() {
            if let Ok(body) = std::fs::read_to_string(&pj) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    if v.get("workspaces").is_some() {
                        return Some(d.to_path_buf());
                    }
                }
            }
        }
        dir = d.parent();
    }
    None
}

/// Best-effort reason lookup: read the nearest `package.json` and
/// report its `sideEffects` value. Falls back to a generic message
/// when the `package.json` is absent or unreadable.
fn side_effects_reason(module_path: &Path) -> String {
    let mut dir = module_path.parent();
    while let Some(d) = dir {
        let pj = d.join("package.json");
        if pj.is_file() {
            if let Ok(body) = std::fs::read_to_string(&pj) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(se) = v.get("sideEffects") {
                        return format!("package.json#sideEffects = {se}");
                    }
                }
            }
            return "package.json without sideEffects field".to_string();
        }
        dir = d.parent();
    }
    "no package.json found".to_string()
}

const MODULE_PRUNE: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    deck_config::DECK_OUT_DIRNAME,
    ".sail-out",
    "sail-out",
];

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

    /// The `sail-marketing` bundle is now TRACED from the site-owned entry
    /// `apps/site/src/marketing/marketing-bundle.ts` (no hardcoded source list).
    /// Rolldown must follow that import graph and register exactly the marketing
    /// tag set — no tag dropped, and the full umbrella must NOT leak in.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_marketing_traced() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(repo, None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());

        let targets = layout.all_targets();
        let Some(marketing) = targets.iter().find(|t| t.name == "sail-marketing") else {
            eprintln!(
                "SKIP rolldown_bundles_marketing_traced: no sail-marketing target — \
                 apps/site content outside the framework boundary"
            );
            return;
        };
        // Traced entry: a real on-disk site module, not a synthetic aggregate.
        assert!(
            matches!(&marketing.entry, TargetEntry::Real(p)
                if p.ends_with("apps/site/src/marketing/marketing-bundle.ts")),
            "sail-marketing must be a Real (traced) entry, got {:?}",
            marketing.entry
        );

        let m = bundle_target(&layout, marketing, staging.path(), true)
            .await
            .expect("marketing should bundle");
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        assert!(body.contains("@ship/sail-runtime"), "runtime kept external");

        // The traceable marketing components (sail-stat / sail-toolbar /
        // sail-progress from the retired const have no `.sail` source and are not
        // members — see marketing-bundle.ts). Each registers via its
        // `component(\`<tag>\`, …)` call; the tag literal survives minification.
        for tag in [
            "sail-icon",
            "sail-badge",
            "sail-button",
            "sail-separator",
            "sail-table",
            "sail-bar-chart",
            "sail-sparkline",
            "sail-brand-mark",
            "sail-theme-picker",
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

    /// The `sail-arcade` bundle is the heavy lazy chunk for the marketing site's
    /// games. Its `index.ts` registers every arcade tag and pulls the
    /// pixi + arcade-engine graph. pixi + arcade-engine must be BUNDLED (it's the
    /// big chunk), while the runtime singleton stays EXTERNAL — exactly one
    /// `@ship/sail-runtime` instance across the page.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_arcade_pixi_inline_runtime_external() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(repo, None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());

        let targets = layout.all_targets();
        let Some(arcade) = targets.iter().find(|t| t.name == "sail-arcade") else {
            eprintln!(
                "SKIP rolldown_bundles_arcade_pixi_inline_runtime_external: no sail-arcade target — \
                 apps/site content outside the framework boundary"
            );
            return;
        };
        let m = bundle_target(&layout, arcade, staging.path(), true)
            .await
            .expect("arcade should bundle");
        // The heavy lazy chunk: pixi (~800 KB) is bundled in.
        assert!(
            m.bytes > 200_000,
            "arcade bundles pixi, got {} bytes",
            m.bytes
        );
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        // Runtime stays external (one instance) — an import of the bare
        // specifier survives, not an inlined second copy.
        assert!(
            body.contains("@ship/sail-runtime"),
            "arcade keeps the runtime singleton external"
        );
        // Each arcade tag registers via its `component(\`<tag>\`, …)` call.
        for tag in [
            "sail-stage",
            "sail-sprite",
            "sail-emitter",
            "sail-stage-input",
            "sail-hud",
        ] {
            assert!(
                body.contains(&format!("`{tag}`")),
                "arcade bundle must register {tag}"
            );
        }
    }

    /// The `sail-pdf` bundle wraps the pdf.js (`pdfjs-dist`) engine. The pack's
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
        let layout = BundleLayout::with_config(repo, None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());

        let targets = layout.all_targets();
        let Some(pdf) = targets.iter().find(|t| t.name == "sail-pdf") else {
            eprintln!(
                "SKIP rolldown_bundles_pdf_with_worker_inlined: no sail-pdf target — \
                 apps/site content outside the framework boundary"
            );
            return;
        };
        let m = bundle_target(&layout, pdf, staging.path(), true)
            .await
            .expect("pdf pack must bundle through Rolldown (no `?url` abort)");
        // The heavy lazy chunk: pdf.js engine + worker are inlined.
        assert!(
            m.bytes > 200_000,
            "pdf bundles the engine, got {} bytes",
            m.bytes
        );
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        // Runtime stays external (one instance).
        assert!(
            body.contains("@ship/sail-runtime"),
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
            assert!(
                !body.contains(needle),
                "worker must not load from a CDN ({needle})"
            );
        }
        // `<ship-pdf>` registers via its `component(\`ship-pdf\`, …)` call.
        assert!(
            body.contains("`ship-pdf`"),
            "pdf bundle must register the ship-pdf tag"
        );
    }

    /// STUDIO-BOOT-1: deck-route must keep `@ship/sail-runtime/internal` external
    /// so host `mount` is not inlined with a second CURRENT_SCOPE. The importmap
    /// maps bare + `/internal` to one `/_deck/runtime.js`; inlining forks it.
    #[tokio::test(flavor = "multi_thread")]
    async fn deck_route_keeps_runtime_internal_external() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(repo, None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());

        let targets = layout.all_targets();
        let Some(route) = targets.iter().find(|t| t.name == "deck-route") else {
            eprintln!(
                "SKIP deck_route_keeps_runtime_internal_external: no deck-route target \
                 (package not resolvable from this root)"
            );
            return;
        };
        let m = bundle_target(&layout, route, staging.path(), true)
            .await
            .expect("deck-route should bundle");
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        assert!(
            body.contains("@ship/sail-runtime"),
            "deck-route keeps bare @ship/sail-runtime external"
        );
        assert!(
            body.contains("@ship/sail-runtime/internal"),
            "deck-route keeps @ship/sail-runtime/internal external (host mount)"
        );
        // Host mount.ts lives on /internal; if it is inlined, this sentinel from
        // packages/sail/runtime/src/mount.ts rides into deck-route.js.
        assert!(
            !body.contains("missing default.setup()"),
            "host mount must not be inlined into deck-route (dual CURRENT_SCOPE)"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn graph_capture_includes_transitive_dep_excludes_node_modules() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("lib")).unwrap();
        std::fs::write(
            root.join("app/main.ts"),
            "import { getX } from '../lib/util.ts'; export default getX;",
        )
        .unwrap();
        std::fs::write(
            root.join("lib/util.ts"),
            "export function getX() { return Math.random(); }",
        )
        .unwrap();

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
        let manifest = bundle_target(&layout, &target, staging.path(), false)
            .await
            .unwrap();
        assert!(
            manifest.modules.iter().any(|m| m.ends_with("app/main.ts")),
            "entry in module set, got: {:?}",
            manifest.modules
        );
        assert!(
            manifest.modules.iter().any(|m| m.ends_with("lib/util.ts")),
            "imported dep from different dir in module set, got: {:?}",
            manifest.modules
        );
        assert!(
            !manifest.modules.iter().any(|m| m
                .components()
                .any(|c| { matches!(c, std::path::Component::Normal(n) if n == "node_modules") })),
            "node_modules must not appear in module set"
        );
    }

    /// The unresolved-import guard's failure contract: a bare import rolldown
    /// cannot resolve must FAIL the build naming the specifier — never ship
    /// as a silent external the importmap doesn't cover (the `motion/mini`
    /// blank-page class).
    #[tokio::test(flavor = "multi_thread")]
    async fn unresolved_bare_import_fails_the_build() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::write(
            root.join("app/main.ts"),
            "import { ghost } from 'this-package-does-not-exist'; export default ghost;",
        )
        .unwrap();

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
        let err = bundle_target(&layout, &target, staging.path(), false)
            .await
            .expect_err("an unresolvable bare import must fail the build");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("this-package-does-not-exist"),
            "error must name the unresolved specifier, got: {msg}"
        );
        assert!(
            msg.contains("bun install"),
            "error must carry the install hint, got: {msg}"
        );
    }

    /// The other half of the contract: a DECLARED external stays external by
    /// declaration, not by resolution luck — it builds fine with the package
    /// absent from disk, and the bare specifier rides the output (the
    /// importmap's job to cover it, e.g. `@ship/sail-compiler-wasm`).
    #[tokio::test(flavor = "multi_thread")]
    async fn declared_external_stays_external_without_resolving() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::write(
            root.join("app/main.ts"),
            "import { ghost } from 'ghost-pkg'; export default ghost;",
        )
        .unwrap();

        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());
        let target = BundleTarget {
            name: "test/main".to_string(),
            entry: TargetEntry::Real(root.join("app/main.ts")),
            externals: vec!["ghost-pkg".to_string()],
            out_file: "main.js".to_string(),
            lazy: false,
        };
        let manifest = bundle_target(&layout, &target, staging.path(), false)
            .await
            .expect("a declared external builds with the package absent");
        let body = std::fs::read_to_string(&manifest.out_path).unwrap();
        assert!(
            body.contains("ghost-pkg"),
            "declared external must stay a bare specifier in the output, got: {body}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn app_source_coverage_outside_conventional_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app/routes")).unwrap();
        std::fs::create_dir_all(root.join("app/src")).unwrap();
        std::fs::write(
            root.join("app/routes/page.ts"),
            "import { getHelper } from '../src/helper.ts'; export default getHelper;",
        )
        .unwrap();
        std::fs::write(
            root.join("app/src/helper.ts"),
            "export function getHelper() { return Math.random().toString(); }",
        )
        .unwrap();
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
        let manifest = bundle_target(&layout, &target, staging.path(), false)
            .await
            .unwrap();
        assert!(
            manifest
                .modules
                .iter()
                .any(|m| m.ends_with("app/routes/page.ts")),
            "entry in module set"
        );
        assert!(
            manifest
                .modules
                .iter()
                .any(|m| m.ends_with("app/src/helper.ts")),
            "imported src file in module set — this is the apps/hub regression"
        );
    }

    /// Unit test for the detector's asset-extension filter. A
    /// bare `import './x.css'` (and similarly for `.json`,
    /// `.svg`, ...) is a deliberate side-effect request for an
    /// asset, NOT a JS module. The asset pipeline emits it
    /// separately (the AssetModulePlugin in `app.rs` adds
    /// `.css` → Asset, and future mappings extend the same
    /// path for `.json`/`.svg`/...). The detector's job is
    /// to track JS side-effect imports; assets are out of
    /// scope. A spurious "dropped side-effect module" warning
    /// for a correctly-emitted CSS would erode trust in the
    /// detector's signal — every true drop gets harder to
    /// spot against the noise.
    ///
    /// We can't test this through `bundle_target` because
    /// Rolldown 1.0.3 has REMOVED experimental CSS bundling
    /// (issue rolldown/rolldown#4271); the bundler errors
    /// before the detector runs. The test is therefore a
    /// direct unit test of `detect_dropped_side_effect_modules`
    /// with synthetic input: an entry that bare-imports a
    /// CSS file, an empty `module_paths` for non-CSS modules
    /// (emulating the "CSS was emitted as an asset, not a
    /// chunk" reality).
    #[test]
    fn detector_skips_css_bare_import() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("styles")).unwrap();
        std::fs::write(
            root.join("app/entry.ts"),
            "import '../styles/global.css';\nexport const ready = true;\n",
        )
        .unwrap();
        std::fs::write(root.join("styles/global.css"), ":root { --x: 1; }\n").unwrap();
        let entry = root.join("app/entry.ts");
        // No JS modules in the bundle beyond the entry —
        // emulating "the CSS was emitted as an asset, not a
        // JS chunk".
        let module_paths = vec![entry.clone()];
        let dropped = detect_dropped_side_effect_modules(&module_paths, &entry);
        assert!(
            dropped.is_empty(),
            "detector must skip the CSS bare import — the \
             asset pipeline handles it, so it must NOT appear \
             in the dropped list. dropped={dropped:?}"
        );
    }

    #[test]
    fn detector_skips_json_bare_import() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("data")).unwrap();
        std::fs::write(
            root.join("app/entry.ts"),
            "import '../data/manifest.json';\nexport const ready = true;\n",
        )
        .unwrap();
        std::fs::write(root.join("data/manifest.json"), r#"{"name":"fixture"}"#).unwrap();
        let entry = root.join("app/entry.ts");
        let module_paths = vec![entry.clone()];
        let dropped = detect_dropped_side_effect_modules(&module_paths, &entry);
        assert!(
            dropped.is_empty(),
            "detector must skip the JSON bare import. dropped={dropped:?}"
        );
    }

    #[test]
    fn detector_skips_svg_bare_import() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("icons")).unwrap();
        std::fs::write(
            root.join("app/entry.ts"),
            "import '../icons/logo.svg';\nexport const ready = true;\n",
        )
        .unwrap();
        std::fs::write(
            root.join("icons/logo.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
        )
        .unwrap();
        let entry = root.join("app/entry.ts");
        let module_paths = vec![entry.clone()];
        let dropped = detect_dropped_side_effect_modules(&module_paths, &entry);
        assert!(
            dropped.is_empty(),
            "detector must skip the SVG bare import. dropped={dropped:?}"
        );
    }

    /// Negative-control test: a bare `import './missing.ts'`
    /// of a JS file (NOT an asset) that exists on disk but
    /// doesn't appear in `module_paths` IS a dropped
    /// side-effect module and the detector must report it.
    /// Guards against the asset filter becoming too broad
    /// (e.g. matching `.ts` by accident).
    #[test]
    fn detector_reports_missing_ts_bare_import() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("orphan")).unwrap();
        std::fs::write(
            root.join("app/entry.ts"),
            "import '../orphan/dead.ts';\nexport const ready = true;\n",
        )
        .unwrap();
        std::fs::write(
            root.join("orphan/dead.ts"),
            "// a .ts file with no observable side effect\n",
        )
        .unwrap();
        let entry = root.join("app/entry.ts");
        // The .ts file isn't in the bundle (the analyzer
        // finds no side effect, so the body is dropped).
        let module_paths = vec![entry.clone()];
        let dropped = detect_dropped_side_effect_modules(&module_paths, &entry);
        let ts_drops: Vec<_> = dropped
            .iter()
            .filter(|d| d.module_path.extension().is_some_and(|e| e == "ts"))
            .collect();
        assert!(
            !ts_drops.is_empty(),
            "detector must still report a .ts bare import as a drop — \
             the asset filter must not over-match. dropped={dropped:?}"
        );
    }

    /// effect is a top-level `customElements.define(...)` — must NOT silently
    /// disappear. The default Rolldown setting (`module_side_effects = true`)
    /// keeps the module's body in the bundle; the deck bundler must NOT
    /// override that. Regression for the sail-sideeffects footgun: authors
    /// were forced to route such imports through a graph-anchored module
    /// (the shell) because the bundler dropped their CSS/registration
    /// import without warning.
    #[tokio::test(flavor = "multi_thread")]
    async fn side_effect_only_module_is_preserved_in_bundle() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("sidefx")).unwrap();
        // Entry imports the registration module purely for its side effect.
        std::fs::write(
            root.join("app/entry.ts"),
            "import '../sidefx/register.ts';\nexport const ready = true;\n",
        )
        .unwrap();
        // The registration module exports nothing — its only effect is
        // registering a custom element. Rolldown must keep the body
        // (default `module_side_effects = true`).
        std::fs::write(
            root.join("sidefx/register.ts"),
            "if (typeof customElements !== 'undefined') {\n  customElements.define('x-sidefx-fixture', class extends HTMLElement {});\n}\n",
        )
        .unwrap();

        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());
        let target = BundleTarget {
            name: "app/sidefx".to_string(),
            entry: TargetEntry::Real(root.join("app/entry.ts")),
            externals: vec![],
            out_file: "sidefx.js".to_string(),
            lazy: false,
        };
        let manifest = bundle_target(&layout, &target, staging.path(), false)
            .await
            .unwrap();
        let body = std::fs::read_to_string(&manifest.out_path).unwrap();
        // The side effect must survive — the registration call is the
        // observable proof the module's body ran.
        assert!(
            body.contains("x-sidefx-fixture"),
            "side-effect-only module was tree-shaken: registration marker missing in bundle"
        );
        // And it should NOT appear in the dropped list (it wasn't dropped).
        assert!(
            manifest.dropped_side_effect_modules.is_empty(),
            "default-sideEffects module must not be reported as dropped, got: {:?}",
            manifest.dropped_side_effect_modules
        );
    }

    /// A side-effect-only module whose owning `package.json` declares
    /// `sideEffects: false` MUST still survive in the deck bundle.
    /// The deck bundler forces `HookSideEffects::True` from the
    /// `load` hook so the importer's intent (a bare `import
    /// '@ship/<pkg>'`) wins over `package.json`'s
    /// `sideEffects: false` / array form — otherwise the body
    /// vanishes and a CSS or `customElements.define` import
    /// silently disappears (the sail-sideeffects footgun).
    ///
    /// This is the deck bundler's "respect" contract: the
    /// import statement is a deliberate side-effect request from
    /// the author; the package's `sideEffects: false` is a
    /// tree-shaking hint for tooling that can't tell the
    /// difference between a binding import and a side-effect
    /// import. The deck bundler can (no specifier = side-effect
    /// only), so the importer's intent wins.
    #[tokio::test(flavor = "multi_thread")]
    async fn side_effect_false_body_still_survives_via_load_hook() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("sidefx-pkg/src")).unwrap();
        // Workspace marker so the fixture mirrors a real Sail
        // workspace. The `*` glob matches every subdir of root
        // (each subdir is a package) — the same pattern the
        // deck-bundler test fixtures use to keep the test
        // self-contained without depending on npm-style nested
        // workspace globs.
        std::fs::write(root.join("package.json"), r#"{"workspaces":["*"]}"#).unwrap();
        // `sideEffects: false` is the bait: without the
        // `load`-hook override, Rolldown would honor this field
        // and drop the registration body's `customElements.define`
        // call. With the override, the importer's intent wins
        // and the body is kept.
        std::fs::write(
            root.join("sidefx-pkg/package.json"),
            r#"{"name":"@ship/sidefx-fixture","sideEffects":false}"#,
        )
        .unwrap();
        // Entry pulls the side-effect-only module in purely
        // for its `customElements.define` registration. The
        // bare form (no `from`, no destructuring) is the
        // side-effect marker the detector keys on.
        std::fs::write(
            root.join("app/entry.ts"),
            "import '@ship/sidefx-fixture';\nexport const ready = true;\n",
        )
        .unwrap();
        // The package's only entry is the registration
        // module — exports nothing, side effects only.
        std::fs::write(
            root.join("sidefx-pkg/src/index.ts"),
            "if (typeof customElements !== 'undefined') {\n  customElements.define('x-sidefx-pkg-fixture', class extends HTMLElement {});\n}\n",
        )
        .unwrap();

        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());
        let target = BundleTarget {
            name: "app/sidefx-pkg".to_string(),
            entry: TargetEntry::Real(root.join("app/entry.ts")),
            externals: vec![],
            out_file: "sidefx-pkg.js".to_string(),
            lazy: false,
        };
        let manifest = bundle_target(&layout, &target, staging.path(), false)
            .await
            .unwrap();
        let body = std::fs::read_to_string(&manifest.out_path).unwrap();
        // The body MUST survive — the `load`-hook override
        // forces `side_effects = true` regardless of
        // `package.json#sideEffects`. Without this, the
        // registration never runs and authors have no
        // signal why.
        assert!(
            body.contains("x-sidefx-pkg-fixture"),
            "side-effect-only module body was tree-shaken despite the \
             `load`-hook `side_effects: true` override. The package \
             contract (`sideEffects: false`) is being honored instead \
             of the importer's intent (a bare `import` statement). \
             This is the sail-sideeffects footgun. bundle: {body}"
        );
        // And the dropped list MUST be empty — the module
        // wasn't dropped, so nothing to warn about.
        assert!(
            manifest.dropped_side_effect_modules.is_empty(),
            "kept module must not appear in dropped list, got: {:?}",
            manifest.dropped_side_effect_modules
        );
    }

    /// A bare `import '…'` of a module that has NO observable
    /// side effects in its source — e.g. a barrel that
    /// declares one export and no `customElements.define`,
    /// top-level `addEventListener`, or similar — is dropped
    /// by Rolldown's analyzer (the file has no
    /// `SideEffectDetail::Unknown` statements, so the body
    /// can't justify itself). The detector flags this as
    /// "genuinely orphaned" and reports it on the manifest
    /// so the build emits a loud warning. Authors see the
    /// drop with the file path + the importer that pulled
    /// it in.
    #[tokio::test(flavor = "multi_thread")]
    async fn side_effect_false_with_dead_export_still_warns() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("sidefx-pkg/src")).unwrap();
        std::fs::write(root.join("package.json"), r#"{"workspaces":["*"]}"#).unwrap();
        std::fs::write(
            root.join("sidefx-pkg/package.json"),
            r#"{"name":"@ship/sidefx-orphan","sideEffects":false}"#,
        )
        .unwrap();
        // Entry: a BARE import of the package, with no
        // bindings used. The module exports a name that
        // the entry never imports, so the package's
        // `sideEffects: false` plus the lack of any
        // referenced export leaves the body orphaned.
        std::fs::write(
            root.join("app/entry.ts"),
            "import '@ship/sidefx-orphan';\nexport const ready = true;\n",
        )
        .unwrap();
        // The package exports an UNUSED name AND has no
        // observable side effect in its body. The analyzer
        // finds no `SideEffectDetail::Unknown` statements, so
        // the body can't justify itself. The module is
        // dropped entirely (its only export was never
        // consumed either). The file must EXIST: an
        // unresolvable bare import is a build failure now
        // (`assert_no_unresolved_imports`), not a silent
        // external — this test exercises the resolved-then-
        // tree-shaken drop path.
        std::fs::write(
            root.join("sidefx-pkg/src/index.ts"),
            "export const sentinelOrphan = 'sentinel-orphan';\n",
        )
        .unwrap();

        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());
        let target = BundleTarget {
            name: "app/sidefx-orphan".to_string(),
            entry: TargetEntry::Real(root.join("app/entry.ts")),
            externals: vec![],
            out_file: "sidefx-orphan.js".to_string(),
            lazy: false,
        };
        let manifest = bundle_target(&layout, &target, staging.path(), false)
            .await
            .unwrap();
        let body = std::fs::read_to_string(&manifest.out_path).unwrap();
        // The dead export is gone (tree-shaken), and the
        // package's body is also gone (nothing references
        // it).
        assert!(
            !body.contains("sentinel-orphan"),
            "dead export was kept — tree-shaking regressed: {body}"
        );
        // ...and the drop surfaces on the manifest as the
        // warning fallback for genuinely-orphaned modules.
        assert!(
            !manifest.dropped_side_effect_modules.is_empty(),
            "genuinely-orphaned side-effect-only import was silent. \
             This is the warn-only fallback path: the module had no \
             live exports and the import was a bare side-effect form, \
             so the bundler reports it for the author to inspect. \
             modules={:?}, dropped={:?}",
            manifest.modules,
            manifest.dropped_side_effect_modules
        );
    }

    /// Direct unit test for `SailPlugin::load`'s read-error
    /// path. A failed read must fail-loud (the "no silent
    /// errors" rule) — silently emitting an empty `code`
    /// would surface downstream as a confusing
    /// `MissingExport` on a named-import consumer, masking
    /// the real cause (a missing or unreadable file).
    ///
    /// This is a unit test, not a `bundle_target`
    /// integration test: Rolldown's resolve step catches
    /// a missing relative import with a "Module not found"
    /// error BEFORE the load hook runs, so the
    /// integration path can't reach the load hook's
    /// read-failure branch. Calling the hook directly
    /// with a path that does not exist on disk exercises
    /// the read-error propagation specifically.
    #[tokio::test(flavor = "multi_thread")]
    async fn load_hook_propagates_read_error_for_missing_module() {
        use crate::sail_plugin::SailPlugin;
        use rolldown::plugin::{
            HookLoadArgs, LoadPluginContext, Plugin, PluginContext, SharedLoadPluginContext,
        };
        use rolldown_common::ModuleIdx;
        // The SailPlugin's `load` hook doesn't read the
        // context (it only uses `args.id`), so a
        // `PluginContext::new_napi_context()` is a valid
        // stand-in for the test — it's the only
        // public constructor the type exposes.
        let ctx: SharedLoadPluginContext = std::sync::Arc::new(LoadPluginContext::new(
            PluginContext::new_napi_context(),
            ModuleIdx::new(0),
        ));
        let plugin = SailPlugin {
            repo_root: PathBuf::from("/nonexistent"),
            emit_component_imports: false,
            component_index: Default::default(),
            shadowed_tags: Default::default(),
            component_roots: Default::default(),
            project_dir: None,
        };
        // A path that does not exist on disk. The
        // `read_to_string` call in `load` will fail; the
        // hook must propagate the error, NOT return an
        // `Ok(HookLoadOutput { code: "", .. })` that
        // would silently empty the module.
        let missing_str: &str = "/tmp/this-file-definitely-does-not-exist-deck-bundler-1234567.ts";
        let args = HookLoadArgs {
            id: missing_str,
            asserted_module_type: None,
            module_idx: ModuleIdx::new(0),
        };
        let result = plugin.load(ctx, &args).await;
        let err = result.expect_err(
            "load hook must fail-loud on a missing file — \
             silent empty bodies mask missing files as \
             MissingExport errors downstream",
        );
        let err_str = format!("{err}");
        assert!(
            err_str.contains("failed to read module"),
            "load hook error must come from the read \
             failure ('failed to read module ...'); a \
             silent empty body would mask the cause. got: {err_str}"
        );
        assert!(
            err_str.contains(missing_str),
            "load hook error must name the missing file path; got: {err_str}"
        );
    }

    /// entry that exports a name with no consumer in the entry must still
    /// be tree-shaken (the constant must NOT appear in the bundle). This
    /// is the contract tree-shaking MUST keep: only side-effect-only
    /// modules are protected from silent drop — genuinely dead exports
    /// still shake.
    #[tokio::test(flavor = "multi_thread")]
    async fn dead_exports_still_shake() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("lib")).unwrap();
        std::fs::write(
            root.join("app/entry.ts"),
            "import { used } from '../lib/barrel.ts';\nexport const v = used;\n",
        )
        .unwrap();
        // Barrel re-exports `used` (consumed) and `UNUSED` (dead). The
        // dead export must be shaken; the side-effect-only protection is
        // scoped to modules with no exports at all, not "modules that
        // happen to have a dead export".
        std::fs::write(
            root.join("lib/barrel.ts"),
            "export const used = 1;\nexport const UNUSED_DEAD_MARKER = 'sentinel-please-shake';\n",
        )
        .unwrap();

        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults())
            .with_out_root(scratch.path().to_path_buf());
        let target = BundleTarget {
            name: "app/dead".to_string(),
            entry: TargetEntry::Real(root.join("app/entry.ts")),
            externals: vec![],
            out_file: "dead.js".to_string(),
            lazy: false,
        };
        let manifest = bundle_target(&layout, &target, staging.path(), false)
            .await
            .unwrap();
        let body = std::fs::read_to_string(&manifest.out_path).unwrap();
        assert!(
            !body.contains("sentinel-please-shake"),
            "dead export was not shaken — tree-shaking regressed: {body}"
        );
    }
}
