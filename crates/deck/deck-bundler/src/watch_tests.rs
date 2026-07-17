//! Tests for watch.rs — split out to keep watch.rs under the 400-line cap.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use super::*;

// ── WatchQueue ────────────────────────────────────────────────────────────────

#[test]
fn ring_drops_oldest_when_full() {
    let q = WatchQueue::default();
    // Fill past capacity; the oldest entries must be evicted, length capped.
    for i in 0..(WATCH_CHANNEL_CAP + 50) {
        q.push(WatchSignal { path: PathBuf::from(format!("/p/{i}")), is_create: false });
    }
    let drained = q.drain();
    assert_eq!(drained.len(), WATCH_CHANNEL_CAP, "ring is bounded at capacity");
    // The first surviving entry is the 50th push (0..49 evicted).
    assert_eq!(drained.first().unwrap().path, PathBuf::from("/p/50"));
    assert_eq!(
        drained.last().unwrap().path,
        PathBuf::from(format!("/p/{}", WATCH_CHANNEL_CAP + 49))
    );
}

/// FAIL LOUD: an overflow eviction is a SKIPPED rebuild, so it must be COUNTED
/// (and surfaced by the consumer), not silently lost. `take_dropped` reports the
/// eviction count and resets — without it a dropped edit serves stale output with
/// nothing in the logs (the silent-eviction hazard).
#[test]
fn overflow_drops_are_counted_not_silent() {
    let q = WatchQueue::default();
    for i in 0..(WATCH_CHANNEL_CAP + 5) {
        q.push(WatchSignal { path: PathBuf::from(format!("/p/{i}")), is_create: false });
    }
    assert_eq!(q.take_dropped(), 5, "the 5 evictions are recorded, not silent");
    assert_eq!(q.take_dropped(), 0, "take_dropped resets the counter");
    // No overflow ⇒ no phantom drops.
    let q2 = WatchQueue::default();
    q2.push(WatchSignal { path: PathBuf::from("/a"), is_create: false });
    assert_eq!(q2.take_dropped(), 0, "under capacity ⇒ zero dropped");
}

#[test]
fn drain_empties_the_ring() {
    let q = WatchQueue::default();
    q.push(WatchSignal { path: PathBuf::from("/a"), is_create: true });
    assert_eq!(q.drain().len(), 1);
    assert!(q.drain().is_empty(), "second drain yields nothing");
}

#[tokio::test(flavor = "multi_thread")]
async fn wait_nonempty_returns_false_when_watch_cleared() {
    let q = WatchQueue::default();
    let flag = AtomicBool::new(false);
    // Flag already cleared → must return false immediately (shutdown path).
    assert!(!q.wait_nonempty(&flag).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn wait_nonempty_returns_true_when_signal_present() {
    let q = WatchQueue::default();
    let flag = AtomicBool::new(true);
    q.push(WatchSignal { path: PathBuf::from("/a"), is_create: false });
    assert!(q.wait_nonempty(&flag).await);
}

// ── classify ─────────────────────────────────────────────────────────────────

/// A `.sail` CREATE must invalidate the snapshot (P0-1); a `.sail` EDIT must
/// not (the target set is unchanged). Before the fix only dir-create
/// invalidated, leaving a leaf written into a watched bucket invisible.
#[test]
fn classify_sui_create_invalidates_snapshot() {
    let tmp = tempfile::tempdir().unwrap();
    let sail = tmp.path().join("views/bar/page.sail");
    std::fs::create_dir_all(sail.parent().unwrap()).unwrap();
    std::fs::write(&sail, "<div/>").unwrap();
    let create = classify(&WatchSignal { path: sail.clone(), is_create: true });
    assert_eq!(create, SignalAction::Rebuild { invalidate: true }, "create re-walks");
    let edit = classify(&WatchSignal { path: sail, is_create: false });
    assert_eq!(edit, SignalAction::Rebuild { invalidate: false }, "edit does not");
}

/// A newly-created directory is `NewDir` (#193/#256 behaviour, preserved).
#[test]
fn classify_new_dir_is_newdir() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("views/baz");
    std::fs::create_dir_all(&dir).unwrap();
    assert_eq!(classify(&WatchSignal { path: dir, is_create: true }), SignalAction::NewDir);
}

/// Generated outputs (`.gen.ts`/`generated/`, reacting self-triggers a
/// rebuild loop) and non-source extensions are ignored.
#[test]
fn classify_generated_and_unrelated_are_ignored() {
    assert_eq!(
        classify(&WatchSignal { path: PathBuf::from("/x/routes.gen.ts"), is_create: true }),
        SignalAction::Ignore,
    );
    assert_eq!(
        classify(&WatchSignal { path: PathBuf::from("/x/generated/foo.ts"), is_create: false }),
        SignalAction::Ignore,
    );
    assert_eq!(
        classify(&WatchSignal { path: PathBuf::from("/x/image.png"), is_create: true }),
        SignalAction::Ignore,
    );
}

// ── graph-driven helpers ──────────────────────────────────────────────────────

#[test]
fn reverse_map_finds_owning_target() {
    let mut graph: std::collections::HashMap<String, std::collections::HashSet<std::path::PathBuf>> =
        std::collections::HashMap::new();
    let entry = std::path::PathBuf::from("/repo/app/main.ts");
    let dep = std::path::PathBuf::from("/repo/lib/util.ts");
    graph.insert("test/main".to_string(), [entry.clone(), dep.clone()].into());
    let targets = targets_for_module(&graph, &dep);
    assert!(targets.contains(&"test/main".to_string()),
        "deep dep reverse-maps to its target");
    let unrelated = targets_for_module(&graph, &std::path::PathBuf::from("/repo/other/file.ts"));
    assert!(unrelated.is_empty(), "unrelated file maps to no target");
}

#[test]
fn no_hardcoded_packages_or_ship_views_in_watch_seeds() {
    let mut graph: std::collections::HashMap<String, std::collections::HashSet<std::path::PathBuf>> =
        std::collections::HashMap::new();
    let file_a = std::path::PathBuf::from("/myapp/app/routes/page.ts");
    let file_b = std::path::PathBuf::from("/myapp/app/src/lib.ts");
    graph.insert("app/page".to_string(), [file_a.clone(), file_b.clone()].into());
    let dirs = module_parent_dirs(&graph);
    assert!(dirs.contains(&std::path::PathBuf::from("/myapp/app/routes")),
        "own routes dir is watched");
    assert!(dirs.contains(&std::path::PathBuf::from("/myapp/app/src")),
        "own src dir is watched");
    assert!(!dirs.iter().any(|d| d.to_string_lossy().contains("packages")),
        "no hardcoded packages/ in watch seeds");
    assert!(!dirs.iter().any(|d| d.to_string_lossy().contains(".ship/views")),
        "no hardcoded .ship/views in watch seeds");
}

#[test]
fn new_view_discovery_via_entry_match() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    let view_dir = root.join("views/newview");
    std::fs::create_dir_all(&view_dir).unwrap();
    let leaf = view_dir.join("page.sail");
    std::fs::write(&leaf, "<div>new</div>").unwrap();

    use crate::targets::{BundleLayout, TargetEntry};
    use deck_config::DeckConfig;
    let layout = BundleLayout::with_config(root.clone(), None, DeckConfig::defaults());
    let targets = layout.all_targets();
    let found = targets.iter().find(|t| {
        matches!(&t.entry, TargetEntry::Real(p) if *p == leaf)
    });
    assert!(found.is_some(), "new view leaf is discoverable via all_targets() after invalidation");
    assert_eq!(found.unwrap().name, "views/newview/view");
}
