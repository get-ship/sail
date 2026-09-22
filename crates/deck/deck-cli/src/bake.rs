//! Workspace bake (`deck-bake`): `build-assets`, `emit`, and `gen`.
//!
//! Not product `sail`. A stranger never runs these; this checkout bakes
//! with the `deck-bake` binary.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};

const BRAND: &str = "deck-bake";

/// Copy already-baked target bundles (and their lazy-chunk closure) into `out`.
pub fn emit(targets: Vec<String>, out: PathBuf) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let out = if out.is_absolute() {
        out
    } else {
        cwd.join(out)
    };
    let repo_root = deck_bundler::paths::topmost_workspace_root(&cwd).unwrap_or(cwd);
    let cfg = deck_bundler::DeckConfig::defaults();
    let layout = deck_bundler::targets::BundleLayout::with_config(repo_root, None, cfg);
    let n = deck_bundler::emit::emit_targets(&layout, &targets, &out)?;
    println!("{BRAND} emit → {n} files into {}", out.display());
    Ok(())
}

/// Bake all framework + consumer bundles via the in-process Rolldown bundler.
pub fn build_assets(no_stamp: bool) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let repo_root = deck_bundler::paths::topmost_workspace_root(&cwd).unwrap_or(cwd);

    let cfg = deck_bundler::DeckConfig::defaults();
    let bundler = deck_bundler::Bundler::start(repo_root.clone(), None, cfg.clone());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    rt.block_on(bundler.build_all())
        .map_err(|e| anyhow::anyhow!("build_all failed: {e}"))?;

    // Build the layout once — used for the framework verify+stamp, the library
    // CEM step, and the consumer guard below. assets_dir_for("sail-runtime")
    // is now .sail/framework/ (the canonical workspace bake target).
    let layout = deck_bundler::targets::BundleLayout::with_config(repo_root.clone(), None, cfg);

    // Verify + stamp the framework singletons BEFORE the library CEM step
    // (deck-deploy-gap): `deck-core/build.rs` needs these 4 files to build at
    // all (an `include_str!` bake), but library CEM generation can legitimately
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
        // can run `deck-bake build-assets` without re-triggering this build
        // script while downstream doctests are queued.
        if !no_stamp {
            let stamp_path = deck_config::framework_stamp_path(&repo_root);
            deck_bundler::atomic::write(
                &stamp_path,
                // Empty content keeps the tracked crate stamp unmodified in git;
                // Cargo's rerun-if-changed tracks mtime, not content.
                b"",
            )
            .with_context(|| format!("writing {}", stamp_path.display()))?;
            println!("  stamp: {}", stamp_path.display());
        }
    }

    // Materialize `@ship/sail-components` `dist/custom-elements.json` (CEM)
    // from source. Run once here (the framework build). Studio aggregates
    // the pre-written manifest into `components.gen.ts`. Also the file the
    // published package declares for external tooling.
    // A compile failure fails the build. Printing and continuing is how
    // un-migrated sources stayed invisible behind an exit-0 asset bake.
    // Runs AFTER the framework verify+stamp above (deck-deploy-gap): a
    // library lint failure is real and must still fail the overall
    // command, but it must never block the framework bundles `build.rs`
    // needs from being staged and stamped first.
    let n = deck_bundler::cem::write_all_component_cems(&repo_root)
        .context("component package CEM generation")?;
    println!("  cem: {n} component library manifest(s) written");

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

    println!("{BRAND} build-assets: bundles baked via in-process Rolldown");
    Ok(())
}

/// Emit studio-client + sail-ops schema snapshots. Refuses outside this checkout.
pub fn generate_ops() -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let root = deck_bundler::paths::topmost_cargo_root(&cwd).unwrap_or(cwd);
    if !root.join("crates/sail/ops").is_dir() || !root.join("packages/sail/studio-client").is_dir()
    {
        bail!("deck-bake gen: this is not the Sail checkout");
    }
    let written = sail_ops::op_gen(Some(&root)).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("{BRAND} gen: wrote {} file(s)", written.len());
    for p in &written {
        println!("  {}", p.display());
    }
    Ok(())
}
