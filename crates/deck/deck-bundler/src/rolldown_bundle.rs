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
            TargetEntry::Virtual { sources } => {
                sources.iter().map(|p| p.to_string_lossy().to_string()).collect()
            }
        },
        modules: module_paths,
        dropped_side_effect_modules,
    })
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
            let resolved_canon = match canonicalize(&resolved) {
                Some(p) => p,
                None => continue,
            };
            if !kept.contains(&resolved_canon) {
                // The target file is absent from the bundle. Confirm
                // it really is side-effect-only on disk — files that
                // have an `export` (even a dead one) are reported by
                // the dead-export path, not this one.
                if !is_side_effect_only_on_disk(&resolved_canon) {
                    continue;
                }
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

/// True when `path` has no `export` statements in its source. The
/// check is intentionally lenient (substring match on `export `) —
/// false positives are fine, false negatives would let a footgun
/// through. Comments are ignored by the test setup.
fn is_side_effect_only_on_disk(path: &Path) -> bool {
    let body = match std::fs::read_to_string(path) {
        Ok(b) => b,
        Err(_) => return false,
    };
    !body.lines().any(|l| l.trim_start().starts_with("export "))
}


/// Resolve an `import '…'` specifier against an importer's directory.
/// Supports relative paths (`.`, `..`, absolute) and the
/// `@ship/<name>` workspace map (the `SuiPlugin` resolves bare
/// specifiers through this map). Anything else is left to the caller.
fn resolve_import(importer: &Path, spec: &str) -> Option<PathBuf> {
    use std::path::PathBuf;
    let p: PathBuf = if spec.starts_with("./") || spec.starts_with("../") {
        importer.parent()?.join(spec)
    } else if spec.starts_with('/') {
        PathBuf::from(spec)
    } else if let Some(rest) = spec.strip_prefix("@ship/") {
        // The SuiPlugin re-anchors `@ship/<name>` to the workspace
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
        let candidate = if ext.is_empty() { p.clone() } else { p.with_extension(&ext[1..]) };
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
/// same map `SuiPlugin::resolve_id` uses to re-anchor bare specifiers,
/// so the drop detector stays in lockstep with the live resolver.
fn resolve_ship_package(importer: &Path, name: &str) -> Option<PathBuf> {
    let repo_root = find_workspace_root(importer)?;
    let roots = crate::catalog::discover_workspace_packages(&repo_root).unwrap_or_default();
    let map: std::collections::HashMap<String, std::path::PathBuf> = roots
        .into_iter()
        .map(|p| (p.name, p.root))
        .collect();
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
/// `SuiPlugin` reads via `component_package_roots`. Returns the first
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
        let Some(marketing) = targets.iter().find(|t| t.name == "sui-marketing") else {
            eprintln!(
                "SKIP rolldown_bundles_marketing_traced: no sui-marketing target — \
                 apps/site content outside the framework boundary"
            );
            return;
        };
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
        let Some(arcade) = targets.iter().find(|t| t.name == "sui-arcade") else {
            eprintln!(
                "SKIP rolldown_bundles_arcade_pixi_inline_runtime_external: no sui-arcade target — \
                 apps/site content outside the framework boundary"
            );
            return;
        };
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
        let Some(pdf) = targets.iter().find(|t| t.name == "sui-pdf") else {
            eprintln!(
                "SKIP rolldown_bundles_pdf_with_worker_inlined: no sui-pdf target — \
                 apps/site content outside the framework boundary"
            );
            return;
        };
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

    /// A side-effect-only import — an entry that pulls in a module whose only
    /// effect is a top-level `customElements.define(...)` — must NOT silently
    /// disappear. The default Rolldown setting (`module_side_effects = true`)
    /// keeps the module's body in the bundle; the deck bundler must NOT
    /// override that. Regression for the sui-sideeffects footgun: authors
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
        let manifest = bundle_target(&layout, &target, staging.path(), false).await.unwrap();
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
    /// `sideEffects: false` IS dropped by Rolldown (per spec — that field is
    /// the tree-shaking contract). The deck bundler must NOT do this
    /// silently: it has to report the drop on the manifest so the build
    /// emits a clear warning naming the module and the importer. A build
    /// that drops a side-effect-only module without surfacing it is the
    /// sui-sideeffects footgun this regression guards against.
    #[tokio::test(flavor = "multi_thread")]
    async fn side_effect_false_drop_is_reported_not_silent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("sidefx-pkg/src")).unwrap();
        // Workspace marker so the fixture mirrors a real Sui workspace.
        // The `*` glob matches every subdir of root (each subdir is a
        // package) — the same pattern the deck-bundler test fixtures
        // use to keep the test self-contained without depending on
        // npm-style nested workspace globs.
        std::fs::write(
            root.join("package.json"),
            r#"{"workspaces":["*"]}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("sidefx-pkg/package.json"),
            r#"{"name":"@ship/sidefx-fixture","sideEffects":false}"#,
        )
        .unwrap();
        // Entry pulls the side-effect-only module in purely for its
        // `customElements.define` registration.
        std::fs::write(
            root.join("app/entry.ts"),
            "import '@ship/sidefx-fixture';\nexport const ready = true;\n",
        )
        .unwrap();
        // The package's only entry is the registration module — exports
        // nothing, side effects only.
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
        let manifest = bundle_target(&layout, &target, staging.path(), false).await.unwrap();
        // The body of the dropped module must NOT be in the bundle — the
        // `sideEffects: false` contract is honored (we drop the right
        // thing, we just don't do it silently).
        let body = std::fs::read_to_string(&manifest.out_path).unwrap();
        assert!(
            !body.contains("x-sidefx-pkg-fixture"),
            "sideEffects:false module should be dropped, but its body was kept: {body}"
        );
        // ...and the drop must be reported on the manifest, NOT silent.
        assert!(
            !manifest.dropped_side_effect_modules.is_empty(),
            "side-effect-only drop was silent — sui-sideeffects footgun. \
             Manifest must report every dropped side-effect-only module so \
             the build emits a warning. modules={:?}, dropped={:?}",
            manifest.modules,
            manifest.dropped_side_effect_modules
        );
        // The reported drop must name the dropped module and the importer
        // (so the warning authors see is actionable).
        let report = &manifest.dropped_side_effect_modules[0];
        assert!(
            report.module_path.to_string_lossy().contains("sidefx-pkg"),
            "dropped module path missing, got: {:?}",
            report
        );
        assert!(
            !report.importer.is_empty(),
            "dropped module importer missing, got: {:?}",
            report
        );
    }

    /// Sanity check that the new behavior doesn't over-correct: a barrel
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
        let manifest = bundle_target(&layout, &target, staging.path(), false).await.unwrap();
        let body = std::fs::read_to_string(&manifest.out_path).unwrap();
        assert!(
            !body.contains("sentinel-please-shake"),
            "dead export was not shaken — tree-shaking regressed: {body}"
        );
    }
}
