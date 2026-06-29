//! Bundler — the daemon bundles assets IN-PROCESS via Rolldown (#163).
//!
// FILE_CAP_OVERRIDE: accessor methods for graph-driven HMR watcher (#deck-hmr-graph-watch); extracting them would require pub-leaking Bundler internals.
//! Replaces the bun worker: no child process, no JSON-line protocol. `Bundler`
//! locates sui-compiler once, then `build`/`build_all` drive Rolldown
//! (`rolldown_bundle`) directly, and `start_watching` re-bundles affected
//! targets on filesystem change (via `notify`), emitting `BundlerEvent`s on a
//! broadcast channel for HMR. Output lands in `assets_dir_for` (sui-out/dist).

pub mod paths;
pub mod tag_check;
pub mod templates;

pub mod app;
// (formerly `mod asset_graph;`) — purged with the runtime `.deck/asset-graph.json` read.
pub mod catalog;
pub mod cem;
pub mod emit;
pub mod node_resolve;
pub mod routes;
pub mod compile;
pub mod proto;
pub mod rolldown_bundle;
pub(crate) mod sui_plugin;
pub mod targets;
pub mod views_bundle;
mod watch;

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{broadcast, Notify};
use tracing::error;


pub use proto::BundleManifest;
// (formerly `use asset_graph::write_asset_graph;`) — purged.
use targets::{BundleLayout, BundleTarget};
pub use deck_config::DeckConfig;

/// Unsolicited events emitted while watching.
#[derive(Debug, Clone)]
pub enum BundlerEvent {
    ModuleChanged { target: String, out_path: String, bytes: usize },
    CompileError { target: String, message: String, line: Option<u32>, col: Option<u32>, file: Option<String> },
}

#[derive(Debug, thiserror::Error)]
pub enum BundlerError {
    #[error("sui-compiler not found (runtime dir or target/)")]
    NoCompiler,
    #[error("unknown target: {0}")]
    UnknownTarget(String),
    #[error("bundle failed for {target}: {message}")]
    Bundle { target: String, message: String },
}

/// In-process bundler handle. Clone-safe.
#[derive(Clone)]
pub struct Bundler {
    repo_root: PathBuf,
    workspace: Option<PathBuf>,
    cfg: DeckConfig,
    staging: PathBuf,
    event_tx: broadcast::Sender<BundlerEvent>,
    watching: Arc<AtomicBool>,
    /// Single-flight registry (#256). Maps a target name to a `Notify` that
    /// fires when the in-flight build of that target finishes. Concurrent
    /// cold-misses for the same target wait on the same `Notify` instead of
    /// each spinning a fresh `BundlerBuilder` + oxc module graph (the primary
    /// 470MB/min driver). Without this, N concurrent fanout lanes requesting
    /// the same view → N full module graphs live simultaneously.
    in_flight: Arc<tokio::sync::Mutex<HashMap<String, Arc<Notify>>>>,
    /// Cached `all_targets()` snapshot (#256). `all_targets` re-reads every
    /// `.sui` under the source roots; the watcher hit it on every FS event and
    /// the build path on every request. Reuse one long-lived snapshot,
    /// invalidated only when a directory is created/removed (the watcher calls
    /// `invalidate_targets`).
    targets_cache: Arc<Mutex<Option<Arc<Vec<BundleTarget>>>>>,
    /// Module graph: maps target name → set of on-disk module paths that
    /// contributed to its last build. Drives the graph-based HMR watcher —
    /// watch exactly those paths' parent dirs instead of hardcoded monorepo dirs.
    module_graph: Arc<Mutex<HashMap<String, HashSet<PathBuf>>>>,
}

impl Bundler {
    /// Start the in-process bundler. `repo_root` is the monorepo/source root;
    /// `workspace` the user-project (if distinct); `cfg` is the parsed
    /// `deck.jsonc` (caller calls `DeckConfig::defaults()` first).
    /// The compiler is the linked `sui-compiler` crate, compiled in-process
    /// (#255) — no binary to locate, no installed-vs-local drift.
    pub fn start(repo_root: PathBuf, workspace: Option<PathBuf>, cfg: DeckConfig) -> Self {
        let (event_tx, _) = broadcast::channel(128);
        let staging = std::env::temp_dir().join(format!("ship-bundle-{}", std::process::id()));
        Self {
            repo_root,
            workspace,
            cfg,
            staging,
            event_tx,
            watching: Arc::new(AtomicBool::new(false)),
            in_flight: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            targets_cache: Arc::new(Mutex::new(None)),
            module_graph: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn layout(&self) -> BundleLayout {
        BundleLayout::with_config(self.repo_root.clone(), self.workspace.clone(), self.cfg.clone())
    }

    /// One long-lived `all_targets()` snapshot, computed on first use and
    /// reused until `invalidate_targets` clears it (#256). Cheap clone of the
    /// shared `Arc<Vec<_>>` — the expensive per-`.sui` FS walk runs once per
    /// invalidation, not once per request/FS-event.
    fn targets_snapshot(&self) -> Arc<Vec<BundleTarget>> {
        {
            let guard = self.targets_cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(snap) = guard.as_ref() {
                return snap.clone();
            }
        }
        let snap = Arc::new(self.layout().all_targets());
        *self.targets_cache.lock().unwrap_or_else(|e| e.into_inner()) = Some(snap.clone());
        snap
    }

    /// Drop the cached target snapshot so the next `targets_snapshot` re-walks.
    /// Called by the watcher when a directory is created/removed (a new view
    /// may have appeared/disappeared).
    fn invalidate_targets(&self) {
        *self.targets_cache.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// Single-flight a build of `target` (#256). If another task is already
    /// building this exact target, await its completion instead of starting a
    /// second `BundlerBuilder`. The first caller registers a `Notify`, runs the
    /// build, then removes the entry and wakes every waiter. Waiters re-resolve
    /// the on-disk artifact themselves (this returns `Ok(None)` for them) — the
    /// build wrote `view.js`, which is what they actually need.
    ///
    /// Returns `Some(manifest)` for the task that performed the build, `None`
    /// for a coalesced waiter (the artifact is on disk either way).
    async fn build_single_flight(
        &self,
        target_name: &str,
        target: &BundleTarget,
    ) -> Result<Option<BundleManifest>, BundlerError> {
        // Either claim the build, or grab the in-flight notifier to wait on.
        let (notify, is_leader) = {
            let mut map = self.in_flight.lock().await;
            match map.get(target_name) {
                Some(existing) => (existing.clone(), false),
                None => {
                    let n = Arc::new(Notify::new());
                    map.insert(target_name.to_string(), n.clone());
                    (n, true)
                }
            }
        };

        if !is_leader {
            // Coalesced: wait for the leader's build to finish, then let the
            // caller read the freshly-written artifact from disk. The presence
            // of the map entry is the source of truth — `notify_waiters` only
            // wakes tasks already parked, so a wake that fires between us
            // grabbing the `Notify` and parking on it would be lost. Guard that
            // race by re-checking the map on a short timeout: once the leader
            // removed the entry, the build is done regardless of the wake.
            loop {
                if !self.in_flight.lock().await.contains_key(target_name) {
                    break;
                }
                let _ = tokio::time::timeout(Duration::from_millis(100), notify.notified()).await;
            }
            return Ok(None);
        }

        // Leader: run the one build, then clear the entry and wake all waiters.
        let layout = self.layout();
        let result = rolldown_bundle::bundle_target(
            &layout,
            target,
            &self.staging,
            false,
        )
        .await
        .map_err(|e| BundlerError::Bundle {
            target: target_name.to_string(),
            message: e.to_string(),
        });
        if let Ok(ref manifest) = result {
            self.update_module_graph(&target_name, manifest.modules.iter().cloned().collect());
        }
        {
            let mut map = self.in_flight.lock().await;
            map.remove(target_name);
        }
        notify.notify_waiters();
        result.map(Some)
    }

    /// Build one named target.
    ///
    /// Cold-misses from `serve_sui_view_setup` funnel through here; concurrent
    /// requests for the same target coalesce onto one build via
    /// `build_single_flight` (#256). A coalesced waiter doesn't get the
    /// leader's manifest, so we synthesize one pointing at the on-disk artifact
    /// — the handler only needs the bundle on disk, which the leader wrote.
    pub async fn build(&self, target: &str) -> Result<BundleManifest, BundlerError> {
        // Resolve the target from the cached snapshot. On a miss, refresh once
        // (a just-created view won't be in a stale snapshot) and retry before
        // giving up.
        let t = match self.find_target(target) {
            Some(t) => t,
            None => {
                self.invalidate_targets();
                self.find_target(target)
                    .ok_or_else(|| BundlerError::UnknownTarget(target.to_string()))?
            }
        };

        match self.build_single_flight(target, &t).await? {
            Some(m) => Ok(m),
            // Coalesced waiter: the leader already wrote the artifact. Report a
            // manifest derived from the layout so the caller has a value; the
            // handler reads the bytes off disk regardless.
            None => Ok(self.manifest_for_built(&t)),
        }
    }

    /// Look up a target by name in the cached snapshot.
    fn find_target(&self, name: &str) -> Option<BundleTarget> {
        self.targets_snapshot().iter().find(|t| t.name == name).cloned()
    }

    /// Synthesize a manifest for a target whose artifact a concurrent leader
    /// already wrote (single-flight waiter path). Reads the on-disk size.
    fn manifest_for_built(&self, t: &BundleTarget) -> BundleManifest {
        use targets::TargetEntry;
        let out_dir = self.layout().assets_dir_for(&t.name);
        let out_path = out_dir.join(&t.out_file);
        let bytes = std::fs::metadata(&out_path).map(|m| m.len() as usize).unwrap_or(0);
        BundleManifest {
            target: t.name.clone(),
            out_path: out_path.to_string_lossy().to_string(),
            bytes,
            inputs: match &t.entry {
                TargetEntry::Real(p) => vec![p.to_string_lossy().to_string()],
                TargetEntry::Virtual { sources } => {
                    sources.iter().map(|p| p.to_string_lossy().to_string()).collect()
                }
            },
            modules: vec![],
        }
    }

    /// Build every target, then aggregate the catalog into `.deck/catalogs/`.
    ///
    /// Honest exit (#262): every target is attempted (one failure must not
    /// abort the others — the catalog still needs refreshing), but per-target
    /// failures are collected and surfaced as `Err` so a caller (the asset
    /// bake, CI) cannot mistake a partial bake for a clean one.
    ///
    /// Dev loop skip-lazy gate: when the env var `DECK_DEV_SKIP_LAZY=1` is
    /// set, every consumer target whose self-declaration marks it `lazy` is
    /// skipped during the build. The dev loop page never imports the arcade
    /// or PDF packs (Pixi.js + pdfjs-dist together are ~4 MB of JS that the
    /// browser heap retains even though no view references them; this is the
    /// 400 MB dev-loop reading measured 2026-06-06). Production builds
    /// (`cargo xtask build-assets` for the dist, or any consumer that
    /// explicitly wants the full asset graph) leave the env var unset and
    /// every target emits as before — the gate is a manual dev lever, so the
    /// default bake is identical whether a target is eager or lazy.
    pub async fn build_all(&self) -> Result<(), BundlerError> {
        let layout = self.layout();
        let mut failed: Vec<String> = Vec::new();
        let mut built: Vec<BundleTarget> = Vec::new();
        // Step 0: regenerate the view route/meta tables
        // (`<bucket>/routes.gen.ts` + `<bucket>/meta.gen.ts`) for every
        // discovered view bucket. This is the framework-owned home for the
        // view tables — folded out of the dev-only Justfile `gen-routes`
        // recipe so a fresh clone + `build-assets` produces them with no
        // manual `deck routes --views` step. `view_bucket_dirs()` is the SAME
        // bucket list the view-target scan walks (`discover_view_bundles`) —
        // one source of truth for "where views live", no second hardcoded path.
        // The watch path ignores `.gen.ts`, so these writes can't self-trigger
        // a rebuild loop. Failures are collected (honest exit), not aborted.
        for views_dir in layout.view_bucket_dirs() {
            let label = views_dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "views".to_string());
            if let Err(e) = crate::routes::generate_view_artifacts(&views_dir, &label) {
                error!(bucket = %views_dir.display(), error = %e, "bundler: view route/meta gen failed");
                failed.push(format!("view-tables {} ({e})", views_dir.display()));
            }
        }
        // Per-view resolver (#255/#267): bundle the view targets TOGETHER
        // (multi-entry) so components shared across views dedup into shared
        // chunks — the dedup that replaces the umbrella. Everything else stays
        // per-target. `build_all` is the bake path (xtask build-assets), so the
        // larger multi-entry graph is a one-shot peak, not a daemon-resident cost.
        let mut views: Vec<BundleTarget> = Vec::new();
        let skip_lazy = std::env::var("DECK_DEV_SKIP_LAZY")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        // Reuse the cached snapshot instead of re-walking every `.sui` (#256).
        for t in self.targets_snapshot().iter() {
            if t.name.starts_with("views/") {
                views.push(t.clone());
                continue;
            }
            // Dev-loop skip: if the env gate is on AND this target's
            // self-declaration marks it lazy, skip it. The flag rides on the
            // `BundleTarget` itself now (the discovery carries it through) — no
            // by-name lookup into a central config.
            if skip_lazy && t.lazy {
                tracing::info!(
                    target = %t.name,
                    "build_all: skipping lazy target (DECK_DEV_SKIP_LAZY=1)"
                );
                continue;
            }
            // `production: true` — build_all IS the bake path (deck-cli /
            // xtask build-assets), so emit minified bundles (#297: the
            // framework singletons shipped 188KB raw because the bake left
            // rolldown's `minify` off). Dev rebuilds (`build` single-flight,
            // watch) stay unminified for debuggability. No source maps are
            // emitted on either path (BundlerOptions leaves `sourcemap`
            // unset), so minifying drops nothing that existed before.
            match rolldown_bundle::bundle_target(&layout, t, &self.staging, true).await {
                Err(e) => {
                    error!(target = %t.name, error = %e, "bundler: target failed");
                    failed.push(format!("{} ({e})", t.name));
                }
                Ok(manifest) => {
                    self.update_module_graph(&t.name, manifest.modules.iter().cloned().collect());
                    built.push(t.clone());
                }
            }
        }
        // View targets together (shared-chunk dedup). Atomic: one bad view fails
        // the whole view bundle — the bake must not ship a broken view.
        if !views.is_empty() {
            match crate::views_bundle::bundle_views(&layout, &views, &self.staging, true).await {
                Ok(()) => built.extend(views),
                Err(e) => {
                    error!(error = %e, "bundler: multi-entry view bundle failed");
                    failed.push(format!("views ({e})"));
                }
            }
        }
        self.refresh_catalog(&layout);
        // (formerly `write_asset_graph(&layout, &built);`) — purged:
        // the framework no longer writes a monorepo-root
        // `.deck/asset-graph.json` (deck-purge-root). The asset-graph SSOT
        // was additive-only (nothing read it yet) and the runtime
        // `.deck/asset-graph.json` read was purged in lockstep.
        if failed.is_empty() {
            Ok(())
        } else {
            Err(BundlerError::Bundle {
                target: format!("{} target(s)", failed.len()),
                message: failed.join(", "),
            })
        }
    }
    fn refresh_catalog(&self, layout: &BundleLayout) {
        // (formerly wrote `<repo_root>/.deck/catalogs/workspace.json`)
        // — purged: the framework no longer writes a monorepo-root
        // `.deck/catalogs/` (deck-purge-root). The workspace catalog was
        // served via `packs::serve_catalog_workspace`, which was also
        // removed. The `catalog::build_catalog` walker stays available
        // for hosts that want to compute the catalog in-memory for
        // their own devtools; it just does not get written to disk.
        let _ = layout;
    }

    /// Subscribe to unsolicited build events (HMR).
    pub fn subscribe_events(&self) -> broadcast::Receiver<BundlerEvent> {
        self.event_tx.subscribe()
    }

    /// Start filesystem watching: re-bundle affected targets on change and emit
    /// `BundlerEvent`s. Idempotent.
    pub async fn start_watching(&self) -> Result<(), BundlerError> {
        if self.watching.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let this = self.clone();
        tokio::spawn(async move { this.run_watch().await });
        Ok(())
    }

    async fn run_watch(&self) {
        // The watch machinery (bounded drop-oldest ring, runtime re-prune)
        // lives in `watch.rs` to keep this file under the line cap (#256).
        watch::run(self).await;
    }

    /// Graceful shutdown (#256): clear the watch flag so the watch loop exits on
    /// its next iteration instead of staying detached for the daemon's life. No
    /// child process to stop.
    pub async fn shutdown(&self) -> Result<(), BundlerError> {
        self.watching.store(false, Ordering::SeqCst);
        Ok(())
    }

    // ── `watch.rs` accessors ──────────────────────────────────────────────
    // The watcher lives in a sibling module, so it reaches the private build
    // state through these `pub(crate)` views rather than touching fields.
    pub(crate) fn repo_root(&self) -> &std::path::Path {
        &self.repo_root
    }
    pub(crate) fn workspace(&self) -> Option<&std::path::Path> {
        self.workspace.as_deref()
    }
    pub(crate) fn cfg_watch(&self) -> &[std::path::PathBuf] {
        &self.cfg.watch
    }
    pub(crate) fn watching_flag(&self) -> &AtomicBool {
        &self.watching
    }
    pub(crate) fn invalidate_targets_for_watch(&self) {
        self.invalidate_targets();
    }
    /// Rebuild one target from the watcher, COALESCED through the same
    /// single-flight gate the on-demand `build` path uses. A watcher rebuild of
    /// `views/git/view` must not run a second `BundlerBuilder` concurrently with
    /// an on-demand build of the same target — that's the cross-build collision
    /// class. Route through `build_single_flight`; a coalesced waiter gets the
    /// on-disk manifest the leader wrote (same fallback as `build`).
    pub(crate) async fn rebuild_for_watch(
        &self,
        target: &BundleTarget,
    ) -> Result<BundleManifest, BundlerError> {
        match self.build_single_flight(&target.name, target).await? {
            Some(m) => Ok(m),
            None => Ok(self.manifest_for_built(target)),
        }
    }
    /// Emit an HMR event on the bundler's broadcast channel (the firehose
    /// `serve_bundle_hmr` streams to the live-reload client). The watcher uses
    /// this on a target rebuild; a standalone host (deck dev) uses it to push a
    /// reload after rebuilding its SPA entry, which is NOT a bundler target.
    pub fn emit_watch_event(
        &self,
        ev: BundlerEvent,
    ) -> Result<usize, broadcast::error::SendError<BundlerEvent>> {
        self.event_tx.send(ev)
    }

    pub(crate) fn update_module_graph(&self, target: &str, modules: HashSet<PathBuf>) {
        let mut g = self.module_graph.lock().unwrap_or_else(|e| e.into_inner());
        g.insert(target.to_string(), modules);
    }

    pub(crate) fn module_dirs_for_watch(&self) -> Vec<PathBuf> {
        let g = self.module_graph.lock().unwrap_or_else(|e| e.into_inner());
        watch::module_parent_dirs(&g).into_iter().collect()
    }

    pub(crate) fn targets_for_module_for_watch(&self, path: &Path) -> Vec<targets::BundleTarget> {
        let g = self.module_graph.lock().unwrap_or_else(|e| e.into_inner());
        let names = watch::targets_for_module(&g, path);
        let snap = self.targets_snapshot();
        names.into_iter()
            .filter_map(|name| snap.iter().find(|t| t.name == name).cloned())
            .collect()
    }

    pub(crate) fn view_bucket_dirs_for_watch(&self) -> Vec<PathBuf> {
        self.layout().view_bucket_dirs()
    }

    pub(crate) fn target_for_entry_path_for_watch(&self, path: &Path) -> Option<targets::BundleTarget> {
        use targets::TargetEntry;
        self.targets_snapshot()
            .iter()
            .find(|t| matches!(&t.entry, TargetEntry::Real(p) if p.as_path() == path))
            .cloned()
    }
}
