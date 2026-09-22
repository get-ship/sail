//! Filesystem watcher for the in-process bundler (#163/#193/#256).
//!
//! `run` attaches NON-recursive notify watches per source directory (pruning
//! vendored/build trees), then re-bundles affected targets on change and emits
//! `BundlerEvent`s. Two #256 hardenings live here:
//!
//! - A **bounded, drop-oldest** ring (`WatchQueue`) between the sync notify
//!   callback and the async consumer, so a write storm can't backlog the queue
//!   unboundedly (the old `unbounded_channel`).
//! - **Runtime re-prune** (#193 completion): a directory created AFTER boot is
//!   membership-checked and, if not vendored, watched; otherwise skipped. The
//!   one-shot startup prune left runtime-created subtrees un-pruned.
//!
//! Watch seed is GRAPH-DRIVEN (#deck-hmr-graph-watch): the watcher watches
//! exactly the parent directories of resolved module-graph paths (non-recursive),
//! plus `cfg.route_dirs` recursively for new route-leaf discovery. No hardcoded
//! monorepo dir names.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::Notify;
use tracing::{info, warn};

use crate::{Bundler, BundlerEvent};

/// Bounded watch-ring capacity (#256). Large enough to absorb a normal editor
/// save burst; on overflow `WatchQueue::push` drops the OLDEST entry so the ring
/// can't grow without bound under a write storm.
const WATCH_CHANNEL_CAP: usize = 1024;

/// Directory names that are never watched — vendored/build trees whose churn
/// notify would otherwise retain a PathBuf per descendant for (#193).
const PRUNE: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    deck_config::DECK_OUT_DIRNAME,
    deck_config::SAIL_GENERATE_DIRNAME,
    ".sail-out",
    "sail-out",
];

/// True when `name` is a pruned tree: an exact PRUNE entry OR an atomic-publish
/// stage sibling (`<out>.stage` / `<out>.stage.<suffix>` — `stage_dir_for`).
/// Stage dirs are create/remove-churned by every rebuild; watching one (or
/// walking into it) self-triggers the rebuild loop fixed in deck-cli.
fn is_pruned_dir_name(name: &str) -> bool {
    PRUNE.contains(&name)
        || name.starts_with(".sail.stage")
        || name.starts_with(".deck.stage")
        || name.starts_with("dist.stage")
        || name.starts_with(".sail-out.stage")
        || name.starts_with("sail-out.stage")
}

/// Re-bundle affected targets on filesystem change. Returns when the bundler's
/// `watching` flag is cleared by `Bundler::shutdown` (#256) or the watcher dies.
/// Watch seed: module graph parent-dirs (non-recursive) + route_dirs (recursive
/// for new-leaf discovery). No hardcoded monorepo dirs.
pub(crate) async fn run(b: &Bundler) {
    let route_dirs = b.route_bucket_dirs_for_watch();
    let mut seed_dirs: Vec<PathBuf> = b.module_dirs_for_watch();
    seed_dirs.extend(route_dirs.iter().cloned());
    seed_dirs.retain(|p| p.exists());
    if seed_dirs.is_empty() {
        return;
    }

    // #256: a write storm under a fanout outran the single-threaded
    // `bundle_target` consumer; an UNBOUNDED channel let the queue backlog
    // without bound. Use a BOUNDED, DROP-OLDEST ring (a capped `VecDeque`
    // behind a `std::sync::Mutex`, woken via `Notify`): the sync notify callback
    // pushes, and on overflow pops the oldest entry so the buffer can't exceed
    // `WATCH_CHANNEL_CAP`. The 150ms debounce drain coalesces whatever survives.
    // The signal carries a create-dir flag so the consumer can re-prune
    // runtime-created vendored subtrees.
    let queue: Arc<WatchQueue> = Arc::new(WatchQueue::default());
    let producer = queue.clone();
    let mut watcher =
        match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(ev) = res {
                use notify::EventKind;
                use notify::event::ModifyKind;
                // SELF-TRIGGER GUARD: the bundler READS source files to compile them.
                // notify's inotify mask surfaces those reads as `Access` events and
                // the resulting atime bump as `Modify(Metadata)` (IN_ATTRIB) — neither
                // is a source EDIT. Reacting to them is a read → rebuild → read loop:
                // every target re-bundles ~1/s with no edits, emitting `module-changed`,
                // and the live-reload client remounts the SPA (or full-reloads a
                // named-view host) on each → the page resets many times per second.
                // Only real content/structural changes (Create / Remove /
                // Modify::Data / Modify::Name) drive a rebuild.
                if matches!(
                    ev.kind,
                    EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_))
                ) {
                    return;
                }
                let is_create = matches!(ev.kind, EventKind::Create(_));
                for p in ev.paths {
                    producer.push(WatchSignal { path: p, is_create });
                }
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                warn!(error = %e, "bundler: watcher init failed");
                return;
            }
        };

    // Boot: route_dirs watched recursively (new-leaf discovery); module-graph
    // dirs watched non-recursively (exact source coverage, no vendored subtrees).
    // #193: NON-recursive watches only — `watch_subtree` walks explicitly, pruning
    // vendored/build trees so notify never retains a PathBuf per descendant.
    let mut watched_dirs: HashSet<PathBuf> = HashSet::new();
    let route_dirs_existing: Vec<PathBuf> = route_dirs.into_iter().filter(|d| d.exists()).collect();
    let graph_dirs_existing: Vec<PathBuf> = b
        .module_dirs_for_watch()
        .into_iter()
        .filter(|d| d.exists() && !route_dirs_existing.contains(d))
        .collect();
    let mut watched = watch_subtree(&mut watcher, &route_dirs_existing, &mut watched_dirs);
    {
        use notify::{RecursiveMode, Watcher};
        for dir in &graph_dirs_existing {
            if !watched_dirs.contains(dir) {
                if watcher.watch(dir, RecursiveMode::NonRecursive).is_ok() {
                    watched_dirs.insert(dir.clone());
                    watched += 1;
                }
            }
        }
    }
    info!(
        dirs = watched,
        "bundler: watching (graph-driven, view dirs pruned-recursive)"
    );

    loop {
        // Block until at least one signal is queued (or shutdown). The ring's
        // `Notify` wakes us; `wait_nonempty` also returns if the watch flag was
        // cleared by `Bundler::shutdown`.
        if !queue.wait_nonempty(b.watching_flag()).await {
            break;
        }
        // Debounce: let the burst accumulate over 150ms, then drain it all.
        tokio::time::sleep(Duration::from_millis(150)).await;
        let changed = queue.drain();
        // FAIL LOUD on overflow: a drop-oldest eviction is a SKIPPED rebuild, so
        // a silent drop would serve stale output with no signal. Surface it (and
        // invalidate the targets snapshot so the next build re-walks structure —
        // a partial recovery; the dropped leaf still needs its own re-save).
        let dropped = queue.take_dropped();
        if dropped > 0 {
            warn!(
                dropped,
                cap = WATCH_CHANNEL_CAP,
                "bundler: watch ring overflowed under load — change events dropped; \
                 some rebuilds were skipped and outputs may be stale until the next edit or restart"
            );
            b.invalidate_targets_for_watch();
        }
        if changed.is_empty() {
            if !b.watching_flag().load(Ordering::SeqCst) {
                break;
            }
            continue;
        }
        let mut seen = std::collections::HashSet::new();
        for sig in changed {
            match classify(&sig) {
                // #193/#256: a dir created after boot is membership-checked
                // (skip pruned trees, watch the survivor) and invalidates the
                // snapshot (a route may appear under it).
                SignalAction::NewDir => {
                    let name = sig.path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if !is_pruned_dir_name(name) {
                        watch_subtree(
                            &mut watcher,
                            std::slice::from_ref(&sig.path),
                            &mut watched_dirs,
                        );
                    }
                    b.invalidate_targets_for_watch();
                }
                SignalAction::Ignore => {}
                SignalAction::Rebuild { invalidate } => {
                    info!(path = %sig.path.display(), "watch: rebuild trigger");
                    // P0-1: a `.sail` written into an already-watched bucket
                    // creates no directory, so dir-create invalidation misses it
                    // — refresh the snapshot the moment the leaf lands.
                    if invalidate {
                        b.invalidate_targets_for_watch();
                    }
                    // Graph-driven reverse-map: which targets own this changed file?
                    let mut graph_targets = b.targets_for_module_for_watch(&sig.path);
                    // New-route-leaf bootstrap: not in graph yet (first time seen),
                    // but the entry path matches after snapshot invalidation.
                    if graph_targets.is_empty() && invalidate {
                        if let Some(t) = b.target_for_entry_path_for_watch(&sig.path) {
                            graph_targets.push(t);
                        }
                    }
                    for t in graph_targets {
                        if !seen.insert(t.name.clone()) {
                            continue;
                        }
                        // Single-flight: coalesce this rebuild with any on-demand
                        // build of the SAME target (cross-build collision). #256.
                        match b.rebuild_for_watch(&t).await {
                            Ok(m) => {
                                // Update graph + expand watch set with newly-revealed dirs.
                                b.update_module_graph(&t.name, m.modules.iter().cloned().collect());
                                let new_dirs: Vec<PathBuf> = b
                                    .module_dirs_for_watch()
                                    .into_iter()
                                    .filter(|d| !watched_dirs.contains(d) && d.exists())
                                    .collect();
                                {
                                    use notify::{RecursiveMode, Watcher};
                                    for dir in new_dirs {
                                        if watcher.watch(&dir, RecursiveMode::NonRecursive).is_ok()
                                        {
                                            watched_dirs.insert(dir);
                                        }
                                    }
                                }
                                let _ = b.emit_watch_event(BundlerEvent::ModuleChanged {
                                    target: m.target,
                                    out_path: m.out_path,
                                    bytes: m.bytes,
                                });
                            }
                            Err(e) => {
                                let _ = b.emit_watch_event(BundlerEvent::CompileError {
                                    target: t.name.clone(),
                                    message: e.to_string(),
                                    line: None,
                                    col: None,
                                    file: Some(sig.path.display().to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
        // Honor shutdown: `Bundler::shutdown` flips the flag so the watch loop
        // exits rather than running detached for the daemon's life.
        if !b.watching_flag().load(Ordering::SeqCst) {
            break;
        }
    }
}

/// Reverse-map: which target names own `changed` in the module graph?
pub(crate) fn targets_for_module(
    graph: &HashMap<String, HashSet<PathBuf>>,
    changed: &Path,
) -> Vec<String> {
    graph
        .iter()
        .filter(|(_, modules)| modules.contains(changed))
        .map(|(name, _)| name.clone())
        .collect()
}

/// Collect all unique parent dirs of every module in the graph (non-recursive
/// watch seeds — each dir covers the files in it, not its subdirectories).
pub(crate) fn module_parent_dirs(graph: &HashMap<String, HashSet<PathBuf>>) -> HashSet<PathBuf> {
    graph
        .values()
        .flat_map(|modules| modules.iter())
        .filter_map(|p| p.parent().map(|d| d.to_path_buf()))
        .collect()
}

/// One FS signal from the notify callback: the changed path plus whether the
/// originating event was a CREATE (a new directory to re-prune, #193 completion,
/// or a new `.sail` route leaf to discover, P0-1). Modify/remove events carry
/// `is_create = false`.
struct WatchSignal {
    path: PathBuf,
    is_create: bool,
}

/// What the consumer loop does with one drained signal. Split out of `run` so
/// the create→invalidate policy is unit-testable (P0-1).
#[derive(Debug, PartialEq, Eq)]
enum SignalAction {
    /// A directory created after boot: watch its subtree (unless pruned) and
    /// invalidate the target snapshot.
    NewDir,
    /// A source file that may rebuild targets. `invalidate` is true only when the
    /// event CREATED a `.sail` leaf — a new route target the snapshot predates, so
    /// it must be re-walked (P0-1). A `.sail` edit / `.ts` change leaves the
    /// target set unchanged.
    Rebuild { invalidate: bool },
    /// Generated build output or an unrelated extension — nothing to do.
    Ignore,
}

/// Classify one drained `WatchSignal` (pure aside from `is_dir`/extension reads).
fn classify(sig: &WatchSignal) -> SignalAction {
    if sig.is_create && sig.path.is_dir() {
        return SignalAction::NewDir;
    }
    let ext = sig.path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !matches!(ext, "ts" | "tsx" | "js" | "sail") {
        return SignalAction::Ignore;
    }
    // Generated build OUTPUTS (routes.gen.ts, packages/.../generated/*.ts) are a
    // rebuild RESULT, not a source change — reacting self-triggers an endless
    // rebuild loop (shipd CPU spin). A real route change still fires via the
    // `[param]` dir / `.sail`.
    let s = sig.path.to_string_lossy();
    if s.ends_with(".gen.ts") || s.contains("/generated/") {
        return SignalAction::Ignore;
    }
    SignalAction::Rebuild {
        invalidate: sig.is_create && ext == "sail",
    }
}

/// Bounded, drop-oldest, coalescing watch ring shared between the sync notify
/// callback (producer) and the async `run` loop (consumer) (#256).
///
/// The notify callback is synchronous, so it can't `await` a bounded
/// `mpsc::Sender::send` for backpressure. A `VecDeque` behind a `std::sync`
/// `Mutex`, capped at `WATCH_CHANNEL_CAP` and woken via `Notify`, gives a true
/// drop-oldest bound: `push` evicts the front when full, so a write storm can't
/// backlog the queue unboundedly. The consumer drains the whole ring after a
/// debounce, coalescing the burst.
#[derive(Default)]
struct WatchQueue {
    buf: Mutex<VecDeque<WatchSignal>>,
    notify: Notify,
    /// Count of signals evicted to overflow since the last drain. Non-zero means
    /// the ring overran the consumer and some rebuilds were skipped — surfaced
    /// LOUDLY so a dropped edit doesn't silently serve stale output.
    dropped: AtomicU64,
}

impl WatchQueue {
    /// Enqueue a signal, evicting the OLDEST entry if the ring is full, then
    /// wake the consumer. Never blocks — safe to call from the sync callback.
    fn push(&self, sig: WatchSignal) {
        {
            let mut buf = self.buf.lock().unwrap_or_else(|e| e.into_inner());
            if buf.len() >= WATCH_CHANNEL_CAP {
                // Overflow: a write storm outran the consumer. Drop-oldest keeps
                // the ring bounded, but a dropped signal is a SKIPPED rebuild —
                // count it so the consumer warns loudly instead of silently
                // serving stale output (a lost edit would otherwise vanish).
                buf.pop_front();
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            buf.push_back(sig);
        }
        self.notify.notify_one();
    }

    /// Take + reset the overflow-drop count since the last call. Non-zero means
    /// the ring overflowed and some rebuilds were skipped — the consumer surfaces
    /// it rather than letting a lost edit go unnoticed.
    fn take_dropped(&self) -> u64 {
        self.dropped.swap(0, Ordering::Relaxed)
    }

    /// Drain every queued signal in arrival order.
    fn drain(&self) -> Vec<WatchSignal> {
        let mut buf = self.buf.lock().unwrap_or_else(|e| e.into_inner());
        buf.drain(..).collect()
    }

    /// Await until the ring is non-empty (returns `true`) or the watch flag is
    /// cleared by `Bundler::shutdown` (returns `false`). Re-checks the flag on
    /// every wake so shutdown can't be missed.
    async fn wait_nonempty(&self, watching: &AtomicBool) -> bool {
        loop {
            if !watching.load(Ordering::SeqCst) {
                return false;
            }
            if !self
                .buf
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_empty()
            {
                return true;
            }
            // Wake on the next push OR after a short timeout so a cleared watch
            // flag (shutdown) is observed even with no pending FS events.
            let _ = tokio::time::timeout(Duration::from_millis(200), self.notify.notified()).await;
        }
    }
}

/// Walk `dirs` with an explicit stack, PRUNE vendored/build trees, and attach
/// NON-recursive notify watches per surviving directory (#193). Tracks watched
/// dirs in `watched_dirs` to avoid double-watching. Returns the count of newly
/// watched directories. Reused for the boot walk AND the runtime re-prune of
/// directories created after boot (#256).
fn watch_subtree(
    watcher: &mut notify::RecommendedWatcher,
    dirs: &[PathBuf],
    watched_dirs: &mut HashSet<PathBuf>,
) -> usize {
    use notify::{RecursiveMode, Watcher};
    let mut stack: Vec<PathBuf> = dirs.to_vec();
    let mut watched = 0usize;
    while let Some(dir) = stack.pop() {
        if watched_dirs.contains(&dir) {
            continue;
        }
        if watcher.watch(&dir, RecursiveMode::NonRecursive).is_err() {
            warn!(dir = %dir.display(), "bundler: watch failed");
        } else {
            watched_dirs.insert(dir.clone());
            watched += 1;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in rd.flatten() {
            // `file_type()` does not follow symlinks, so symlinked dirs are
            // skipped here (no cycles) and only real subdirectories recurse.
            if matches!(entry.file_type(), Ok(ft) if ft.is_dir())
                && !is_pruned_dir_name(entry.file_name().to_string_lossy().as_ref())
            {
                stack.push(entry.path());
            }
        }
    }
    watched
}

#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;
