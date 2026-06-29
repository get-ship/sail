use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, Once, RwLock};

/// One-root-per-process invariant: a single daemon serves one workspace, so a
/// process-global override is sound. If that ever breaks (multi-tenant
/// embedding), per-request threading through the resolver is the fallback.
static PROJECT_ROOT_OVERRIDE: RwLock<Option<PathBuf>> = RwLock::new(None);
/// Per-app view-output dirs (slice-2 component 1). A host (shipd) calls
/// `register_view_output(app_name, dir)` so apps with a non-conventional
/// layout (e.g. `apps/hub/.deck/views/<name>/view.js`) serve their own view
/// bundles without depending on the global project root. Read by
/// `assets::resolve_view_js` BEFORE the workspace-root fallback.
static VIEW_OUTPUTS: Mutex<BTreeMap<String, PathBuf>> = Mutex::new(BTreeMap::new());
static DETECTION_WARN_ONCE: Once = Once::new();

/// Per-instance resolution context — the mount-scoped replacement for the
/// process-global `PROJECT_ROOT_OVERRIDE` + `VIEW_OUTPUTS`. A host constructs
/// one per project and carries it per-request (like the `DataPort` Extension),
/// so one process serves many projects without cross-talk
/// (`inv-deck-single-root-scoped`).
///
/// The host declares view roots in PRECEDENCE ORDER and expresses "these are
/// mine first" by ordering — deck privileges no named 'first-party' bucket
/// (`inv-deck-no-first-party`); `roots[0]` simply wins.
///
/// (Step 1: the abstraction + its isolation contract. Step 2 threads it through
/// the resolve/serve path in place of the globals; the shipd caller migration
/// is the ship-repo follow-on.)
#[derive(Debug, Clone, Default)]
pub struct Resolution {
    roots: Vec<PathBuf>,
    view_outputs: BTreeMap<String, PathBuf>,
}

impl Resolution {
    /// An empty context — no roots, no view outputs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a view root. Earlier calls have higher precedence (first wins).
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.roots.push(root.into());
        self
    }

    /// Register a per-app view-output dir (replaces the global VIEW_OUTPUTS).
    pub fn register_view_output(&mut self, app: impl Into<String>, dir: impl Into<PathBuf>) {
        self.view_outputs.insert(app.into(), dir.into());
    }

    /// The host's declared roots, in precedence order.
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// The highest-precedence root, if any. (No 'first-party' privilege — this
    /// is simply the first declared root.)
    pub fn project_root(&self) -> Option<&std::path::Path> {
        self.roots.first().map(|p| p.as_path())
    }

    /// The view-output dir for `app`, if registered.
    pub fn view_output(&self, app: &str) -> Option<&std::path::Path> {
        self.view_outputs.get(app).map(|p| p.as_path())
    }
}

/// Authoritative project-root override set ONCE at host startup.
///
/// Long-running / embedding hosts (shipd, deck-conformance) call this with the
/// workspace root they already hold. The resolver consults this BEFORE
/// the cwd-detection floor, so per-workspace requests resolve against
/// the workspace they were launched against even if cwd drifts later.
pub fn set_project_root(root: PathBuf) {
    if let Ok(mut slot) = PROJECT_ROOT_OVERRIDE.write() {
        *slot = Some(root);
    }
}

/// Register a per-app view-bundle output dir. Hosts call this for each
/// `AppServeConfig.view_output` they own. The resolver probes these dirs
/// BEFORE the workspace-root fallback when serving `/_deck/views/{name}/view.js`.
pub fn register_view_output(app_name: impl Into<String>, dir: PathBuf) {
    if let Ok(mut map) = VIEW_OUTPUTS.lock() {
        map.insert(app_name.into(), dir);
    }
}

/// Snapshot the currently-registered view outputs (test-only inspection).
pub fn current_view_outputs() -> Vec<(String, PathBuf)> {
    VIEW_OUTPUTS
        .lock()
        .ok()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default()
}

/// Read the current override (test-only inspection).
pub fn current_project_root_override() -> Option<PathBuf> {
    PROJECT_ROOT_OVERRIDE.read().ok().and_then(|g| g.clone())
}

#[cfg(test)]
pub(crate) fn reset_project_root_override_for_test() {
    if let Ok(mut slot) = PROJECT_ROOT_OVERRIDE.write() {
        *slot = None;
    }
    if let Ok(mut map) = VIEW_OUTPUTS.lock() {
        map.clear();
    }
    // Note: `DETECTION_WARN_ONCE` is NOT reset — the per-process warn
    // is one-shot by design (per spec). Tests that need a fresh warn
    // stamp must run in their own process.
}

/// Nearest-ancestor detection: upwalk from `cwd`, accept the first ancestor
/// that contains the deck-owned SOURCE marker (`views/`). Bounded at ~32 hops.
/// Std-only — never reaches a TOPMOST-Cargo.toml workspace resolver (which
/// would shadow a nested project's root onto the monorepo root). Never keys on
/// `.deck` (build output, absent on fresh checkout, flips post-build), never on
/// `deck.jsonc` (purged — see deck-purge-root), and never sniffs a host-private
/// dir (a host with a non-conventional layout, e.g. a different view bucket,
/// calls `set_project_root` — the safe path; deck names no host dir here).
pub(crate) fn detect_project_root_from(cwd: &std::path::Path) -> Option<PathBuf> {
    const MAX_DEPTH: usize = 32;
    let mut cur = cwd;
    for _ in 0..MAX_DEPTH {
        if cur.join("views").is_dir() {
            return Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    None
}

use deck_host::resolver::ViewLayout;
pub use deck_host::resolver::{ResolvedView, ResolveError};
/// Build a [`ViewLayout`] from the conventional `["views"]` bucket with a
/// single `page.sui` leaf. The leaf is unified across all
/// buckets (fixes the #254 `index.sui` vs `page.sui` divergence). The
/// framework does not load any config file — a host that needs a different
/// layout (custom view buckets, custom leaf) constructs a `DeckConfig`
/// (clone `defaults()` + override) and threads it into `Bundler::start`;
/// the bundler's view-discovery walks `cfg.view_dirs` directly. The
/// runtime resolver here just reports the conventional defaults; the
/// SAFE PATH is `set_project_root` so long-running hosts never ride the
/// cwd-detection floor.
fn view_layout() -> ViewLayout {
    ViewLayout::conventional()
}
pub fn resolve(name: &str, sub_path: Option<&str>) -> Result<ResolvedView, ResolveError> {
    let layout = view_layout();
    // `resolve_project_root` resolves override → nearest-ancestor marker
    // detection → cwd fallback. The override is the safe path; long-running
    // hosts (shipd, deck-conformance) call `set_project_root` at startup so they
    // never ride the cwd-detection floor. The detection floor is the fragile
    // path: it keys on deck-owned SOURCE markers (`views/` > `.ship/views/`),
    // and a nested project's `app/` is not shadowed onto the monorepo root.
    let Some(dir) = resolve_project_root() else {
        return Err(ResolveError {
            message: format!(
                "view not found: {} — no project root (host should call deck_core::resolver::set_project_root at startup)",
                join_request(name, sub_path),
            ),
        });
    };
    deck_host::resolver::resolve(&dir, &layout, name, sub_path)
}
/// Project root for locating the build-output tree. deck reads app +
/// user view/chrome bundles from where the host passes them (per-app
/// `scaffold.source`, asset-graph entries) — not from a hardcoded
/// `<project_root>/.deck/` reach (that was the deck-purge-root cut).
/// The host is the authority on output location; the framework follows.
// (formerly `pub(crate) fn deck_out_root() -> Option<PathBuf>` — `<root>/.deck`) removed.

/// Resolution order (HIGH → LOW):
/// 1. **HOST OVERRIDE** — `set_project_root(root)` set at startup. Authoritative
///    for long-running hosts (shipd, deck-conformance) that already hold the workspace
///    root in typed state. Sound because deck serves one root per process.
/// 2. **DETECTION FLOOR** — std-only nearest-ancestor upwalk from
///    `std::env::current_dir()`, looking for a deck-owned SOURCE marker
///    (`views/` > `.ship/views/`) at any of the up to 32 ancestors.
///    Nearest-ancestor-wins: deck-conformance's `app/` does NOT resolve
///    to the monorepo root, and a fresh checkout resolves before `.deck` is
///    built (markers are source-side, all git-tracked).
/// 3. **ZERO-CONFIG FALLBACK** — `current_dir()`. Matches the
///    `source_roots: ["."]` default in [`DeckConfig::defaults`].
///
/// The host-override step is the SAFE path; the cwd-detection floor is
/// fragile. A long-running host that never calls `set_project_root` is
/// warned once on the detection / fallback path so the coupling is
/// surfaced instead of silently riding the floor.
pub(crate) fn resolve_project_root() -> Option<PathBuf> {
    if let Some(root) = PROJECT_ROOT_OVERRIDE.read().ok().and_then(|g| g.clone()) {
        return Some(root);
    }
    let cwd = std::env::current_dir().ok();
    if let Some(ref dir) = cwd
        && let Some(root) = detect_project_root_from(dir)
    {
        warn_server_host_uses_detection();
        return Some(root);
    }
    if let Some(dir) = cwd {
        warn_server_host_uses_detection();
        return Some(dir);
    }
    None
}

/// One-shot warn behind `std::sync::Once`. Catches a future long-running host
/// that mounts the resolver without calling `set_project_root` — a smell,
/// because the cwd-detection floor is the fragile path.
fn warn_server_host_uses_detection() {
    if std::env::var_os("DECK_SERVER_HOST").is_some() {
        DETECTION_WARN_ONCE.call_once(|| {
            tracing::warn!(
                "deck: project root resolved by cwd-detection; a long-running host should call set_project_root()"
            );
        });
    }
}

fn join_request(name: &str, sub_path: Option<&str>) -> String {
    match sub_path.filter(|s| !s.is_empty()) {
        Some(sub) => format!("{name}/{sub}"),
        None => name.to_string(),
    }
}

/// Best-effort list of top-level view names servable on this daemon, for the
/// human-facing not-found page. Scans the same on-disk roots the resolver
/// consults — the first-party bucket (config-driven via
/// `first_party_app.root` or the legacy env-var fallback) plus every
/// bucket in the host's `view_dirs` (default `["views"]`).
/// Cheap (one shallow `read_dir` per root, no compilation) and graceful:
/// any missing root contributes nothing, and with no project root at
/// all the list is simply empty. Names are deduped and sorted so the
/// page renders deterministically.
pub fn list_view_names() -> Vec<String> {
    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let layout = view_layout();
    if let Some(fp) = layout.first_party_root.as_deref() {
        collect_view_dirs(fp, &mut names);
    }
    if let Some(root) = resolve_project_root() {
        for bucket in &layout.view_dirs {
            collect_view_dirs(&root.join(bucket), &mut names);
        }
    }
    names.into_iter().collect()
}

/// Add each immediate child directory of `dir` whose name is a valid view name
/// to `out`. A non-directory or unreadable `dir` contributes nothing.
fn collect_view_dirs(dir: &std::path::Path, out: &mut std::collections::BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str()
            && crate::view::is_valid_view_name(name)
        {
            out.insert(name.to_string());
        }
    }
}

#[cfg(test)]
mod project_root_tests {
    use super::{detect_project_root_from, reset_project_root_override_for_test, set_project_root};
    use std::path::PathBuf;

    /// Nested project: monorepo root has a `views/`, the inner project's
    /// `app/` ALSO has a `views/`. Detection must pick the NEAREST one
    /// (`app/`), not the topmost (monorepo). This is the deck-conformance shape:
    /// a demo app nested inside a wider monorepo, where reaching the
    /// monorepo root would shadow the project's own views.
    #[test]
    fn detection_picks_nearest_ancestor_marker_not_topmost() {
        let tmp = tempfile::tempdir().unwrap();
        let monorepo = tmp.path().to_path_buf();
        std::fs::create_dir(monorepo.join("views")).unwrap();
        let app = monorepo.join("app");
        std::fs::create_dir(&app).unwrap();
        std::fs::create_dir(app.join("views")).unwrap();
        let deep = app.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();

        // Run from a deeply-nested leaf of `app/`; the nearest `views/`
        // is `app/views`, not the monorepo's.
        let detected = detect_project_root_from(&deep);
        assert_eq!(
            detected,
            Some(app.clone()),
            "nearest-ancestor-wins: expected the inner app root, got {detected:?}",
        );
        // The monorepo's `views/` is NOT picked up, even though it exists.
        let monorepo_with_views = monorepo.join("views");
        assert!(monorepo_with_views.is_dir());
        assert_ne!(detected, Some(monorepo));
    }

    /// `deck.jsonc` is NOT a detection marker anymore (purged). The
    /// `views/` directory is the strongest source marker — even nested
    /// inside a tree that also has a `deck.jsonc`, `views/` wins because
    /// `deck.jsonc` is inert (no effect on detection or config load).
    #[test]
    fn stray_deck_jsonc_does_not_outrank_views_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let outer = tmp.path().to_path_buf();
        std::fs::write(outer.join("deck.jsonc"), "{}\n").unwrap();
        let inner = outer.join("nested");
        std::fs::create_dir(&inner).unwrap();
        std::fs::create_dir(inner.join("views")).unwrap();
        // From `inner`, the nearest marker is `views/`. The outer's
        // `deck.jsonc` is one level up AND is ignored.
        let detected = detect_project_root_from(&inner);
        assert_eq!(detected, Some(inner));
    }

    /// deck detects via its OWN marker (`views/`) only. A `.ship/views`-only
    /// layout is a HOST convention, not deck's — detection must NOT key on it
    /// (a host with that layout calls `set_project_root`, the safe path).
    /// inv-deck-standalone. (This test was inverted from one that REQUIRED the
    /// `.ship/views` marker — it had codified the very ship coupling we removed.)
    #[test]
    fn detection_ignores_dot_ship_views_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".ship").join("views")).unwrap();
        // No deck-owned `views/` marker present → not auto-detected.
        assert_eq!(detect_project_root_from(&root), None);
    }
    /// No marker anywhere up the tree → `None`. (Caller falls back to
    /// cwd.)
    #[test]
    fn detection_returns_none_with_no_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let deep = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        assert!(detect_project_root_from(&deep).is_none());
    }

    /// The global override ALWAYS wins — even from a directory that
    /// would otherwise be detected as a project root.
    #[test]
    fn override_wins_over_detection() {
        reset_project_root_override_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let detected_root = tmp.path().to_path_buf();
        std::fs::create_dir(detected_root.join("views")).unwrap();

        // The detected-root is "the project" from disk's perspective.
        let detected = detect_project_root_from(&detected_root);
        assert_eq!(detected, Some(detected_root.clone()));

        // But the override sits at a different, unrelated path, and
        // `resolve_project_root` consults the override FIRST. We do not
        // call `resolve_project_root` here (it would `current_dir` and
        // confuse the test); we only assert the contract documented on
        // `set_project_root`.
        set_project_root(PathBuf::from("/some/explicit/host/root"));
        assert_eq!(
            super::current_project_root_override(),
            Some(PathBuf::from("/some/explicit/host/root")),
        );
        reset_project_root_override_for_test();
    }

    /// Acceptance #1 of the deck-purge-root job: a stray `deck.jsonc` at the
    /// workspace root has NO effect — not loaded (no `DeckConfig::load`), not
    /// a detection marker. Drop a `deck.jsonc` into a project root and assert
    /// `detect_project_root_from` ignores it entirely — only `views/` and
    /// `.ship/views/` are accepted as source-side markers.
    #[test]
    fn stray_deck_jsonc_is_not_a_detection_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // A stray deck.jsonc — the file the framework used to load and key
        // detection on. Drop it at the root and a nested level; detection
        // must ignore it.
        std::fs::write(root.join("deck.jsonc"), r#"{ "viewDirs": ["evil"] }"#).unwrap();
        let inner = root.join("nested");
        std::fs::create_dir(&inner).unwrap();
        std::fs::write(inner.join("deck.jsonc"), "{}\n").unwrap();

        // No `views/` or `.ship/views/` anywhere → returns None.
        assert_eq!(
            detect_project_root_from(&root),
            None,
            "a deck.jsonc alone is NOT a detection marker; framework ignores it"
        );
        assert_eq!(
            detect_project_root_from(&inner),
            None,
            "a nested deck.jsonc is also ignored"
        );

        // Add a `views/` at the nested level — detection finds the inner,
        // not the deck.jsonc-bearing root.
        std::fs::create_dir(inner.join("views")).unwrap();
        assert_eq!(
            detect_project_root_from(&inner),
            Some(inner.clone()),
            "views/ wins at the nearest ancestor; deck.jsonc is irrelevant"
        );
        // And from the root (which has BOTH deck.jsonc AND views/ via inner),
        // the nearest marker path is still views/-driven — root has deck.jsonc
        // only, so detection walks up and returns None.
        assert_eq!(
            detect_project_root_from(&root),
            None,
            "a deck.jsonc at root with no views/ nearby is still inert"
        );
    }

    /// inv-deck-single-root-scoped + inv-deck-no-first-party (BDD — now GREEN at
    /// the type level). Two projects in one process each carry their own
    /// `Resolution`, so host B registering its root does NOT clobber host A:
    /// resolution is mount-scoped, not a shared process-global. Precedence is
    /// the ORDER of declared roots (`roots[0]` wins) — deck blesses no named
    /// 'first-party' bucket.
    ///
    /// (Step 1 proves the abstraction isolates. The remaining gap — the
    /// resolve/serve PATH still reads the global override, and shipd is its
    /// caller — stays tracked on the blueprint; this test is the contract that
    /// step 2's threading must keep green.)
    #[test]
    fn resolution_is_mount_scoped_not_a_shared_global() {
        // Host A boots against project A; host B against project B — SAME process.
        let a = super::Resolution::new().with_root("/tmp/projA");
        let b = super::Resolution::new().with_root("/tmp/projB");
        // B registering its root does not clobber A — each handle keeps its own.
        assert_eq!(a.project_root(), Some(std::path::Path::new("/tmp/projA")));
        assert_eq!(
            b.project_root(),
            Some(std::path::Path::new("/tmp/projB")),
            "mount-scoped: B is isolated from A — there is no shared global to clobber"
        );
        // Precedence is the ORDER of roots, not a privileged 'first-party' bucket.
        let host = super::Resolution::new()
            .with_root("/srv/app")
            .with_root("/srv/shared");
        assert_eq!(host.roots().len(), 2);
        assert_eq!(
            host.project_root(),
            Some(std::path::Path::new("/srv/app")),
            "first declared root wins by ordering — deck names no first-party bucket"
        );
    }
}
