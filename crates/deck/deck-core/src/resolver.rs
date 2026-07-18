use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Per-instance resolution context — the mount-scoped root(s) + per-app view
/// outputs a request resolves against. This is the WHOLE of deck's resolution
/// state: there is no process-global `PROJECT_ROOT_OVERRIDE` / `VIEW_OUTPUTS`
/// backing it, so one process can serve many projects with no cross-talk
/// (`inv-deck-mount-scoped-resolution`). A host constructs one per project and
/// carries it per-request (an axum `Extension`, like the `DataPort`); the serve
/// path threads `Option<&Resolution>` end to end.
///
/// The host declares view roots in PRECEDENCE ORDER and expresses "these are
/// mine first" by ordering — deck privileges no named 'first-party' bucket
/// (`inv-deck-no-first-party`); `roots[0]` simply wins.
#[derive(Debug, Clone, Default)]
pub struct Resolution {
    roots: Vec<PathBuf>,
    route_outputs: BTreeMap<String, PathBuf>,
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

    /// Seed a Resolution by DETECTING the project root from `cwd` — the nearest
    /// ancestor holding the deck-owned `views/` source marker, else `cwd` itself.
    ///
    /// This is the EXPLICIT replacement for the old cwd-detection FLOOR that
    /// used to hide inside the process-global resolver as a silent fallback. A
    /// caller with no host-supplied root (a bare CLI invocation, a test) asks
    /// for detection here; the serve path never falls back to it on its own.
    pub fn detect() -> Self {
        let mut r = Self::default();
        if let Ok(cwd) = std::env::current_dir() {
            r.roots.push(detect_project_root_from(&cwd).unwrap_or(cwd));
        }
        r
    }

    /// Register a per-app view-output dir. Hosts call this for each app whose
    /// built view bundles live outside the conventional `<root>/.deck/views`
    /// (e.g. `apps/hub/.deck/views/<name>/view.js`). Probed BEFORE the
    /// project-root `.deck` fallback by [`route_bundle_roots`](Self::route_bundle_roots).
    pub fn register_route_output(&mut self, app: impl Into<String>, dir: impl Into<PathBuf>) {
        self.route_outputs.insert(app.into(), dir.into());
    }

    /// The host's declared roots, in precedence order.
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// The highest-precedence root, if any. (No 'first-party' privilege — this
    /// is simply the first declared root.)
    pub fn project_root(&self) -> Option<&Path> {
        self.roots.first().map(|p| p.as_path())
    }

    /// The view-output dir for `app`, if registered.
    pub fn route_output(&self, app: &str) -> Option<&Path> {
        self.route_outputs.get(app).map(|p| p.as_path())
    }

    /// View-bundle roots to probe for `<root>/views/<name>/view.js`, in
    /// precedence order: the per-app view-output dirs first (host-registered),
    /// then the project root's `<root>/.deck` (the bundler's default output).
    /// The `.deck` join is the SAME single constant the bundler writes through
    /// (`deck_config::deck_out_path`), so this reader can never drift from the
    /// writer (`inv-deck-output-location-single`).
    pub(crate) fn route_bundle_roots(&self) -> Vec<PathBuf> {
        let mut roots: Vec<PathBuf> = self.route_outputs.values().cloned().collect();
        if let Some(pr) = self.project_root() {
            roots.push(deck_config::deck_out_path(pr));
        }
        roots
    }
}

/// The project root a serve-path request resolves against — the highest-
/// precedence root of the per-request [`Resolution`], or `None` when the host
/// carried no Resolution. `None` is an HONEST miss, not a hidden fallback: the
/// caller serves a 404 / empty shell rather than silently resolving against a
/// process global or cwd. A caller that wants cwd detection constructs
/// [`Resolution::detect`].
pub fn resolve_project_root_with(resolution: Option<&Resolution>) -> Option<PathBuf> {
    resolution
        .and_then(|r| r.project_root())
        .map(|p| p.to_path_buf())
}

/// Nearest-ancestor detection: upwalk from `cwd`, accept the first ancestor
/// that contains the deck-owned SOURCE marker (`views/`). Bounded at ~32 hops.
/// Std-only — never reaches a TOPMOST-Cargo.toml workspace resolver (which
/// would shadow a nested project's root onto the monorepo root). Never keys on
/// `.deck` (build output, absent on fresh checkout, flips post-build), never on
/// `deck.jsonc` (purged — see deck-purge-root), and never sniffs a host-private
/// dir (a host with a non-conventional layout constructs a `Resolution` with an
/// explicit root — the safe path; deck names no host dir here).
pub(crate) fn detect_project_root_from(cwd: &Path) -> Option<PathBuf> {
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

use deck_host::resolver::RouteLayout;
pub use deck_host::resolver::{ResolveError, ResolvedRoute};
/// Build a [`RouteLayout`] from the conventional `["views"]` bucket with a
/// single `page.sail` leaf. The leaf is unified across all buckets (fixes the
/// #254 `index.sail` vs `page.sail` divergence). The framework loads no config
/// file — a host that needs a different layout constructs a `DeckConfig` and
/// threads it into `Bundler::start`; the bundler's view-discovery walks
/// `cfg.route_dirs` directly. The runtime resolver here reports the conventional
/// defaults.
fn route_layout() -> RouteLayout {
    RouteLayout::conventional()
}

/// Resolve a view by name under an EXPLICIT project `root` (mount-scoped — the
/// caller passes the root from its per-request [`Resolution`], never a process
/// global). Delegates the on-disk walk to `deck_host::resolver::resolve`.
pub fn resolve(
    root: &Path,
    name: &str,
    sub_path: Option<&str>,
) -> Result<ResolvedRoute, ResolveError> {
    let layout = route_layout();
    deck_host::resolver::resolve(root, &layout, name, sub_path)
}

/// Best-effort list of top-level view names servable under `root`, for the
/// human-facing not-found page. Scans the conventional view buckets (default
/// `["views"]`) under the given root. Cheap (one shallow `read_dir` per bucket,
/// no compilation) and graceful: a missing bucket contributes nothing. Names
/// are deduped and sorted so the page renders deterministically.
pub fn list_route_names(root: &Path) -> Vec<String> {
    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let layout = route_layout();
    if let Some(fp) = layout.first_party_root.as_deref() {
        collect_route_dirs(fp, &mut names);
    }
    for bucket in &layout.route_dirs {
        collect_route_dirs(&root.join(bucket), &mut names);
    }
    names.into_iter().collect()
}

/// Add each immediate child directory of `dir` whose name is a valid view name
/// to `out`. A non-directory or unreadable `dir` contributes nothing.
fn collect_route_dirs(dir: &Path, out: &mut std::collections::BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str()
            && crate::route::is_valid_route_name(name)
        {
            out.insert(name.to_string());
        }
    }
}

#[cfg(test)]
mod project_root_tests {
    use super::{detect_project_root_from, Resolution};
    use std::path::{Path, PathBuf};

    /// Nested project: monorepo root has a `views/`, the inner project's
    /// `app/` ALSO has a `views/`. Detection must pick the NEAREST one
    /// (`app/`), not the topmost (monorepo).
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

        let detected = detect_project_root_from(&deep);
        assert_eq!(
            detected,
            Some(app.clone()),
            "nearest-ancestor-wins: expected the inner app root, got {detected:?}",
        );
        assert!(monorepo.join("views").is_dir());
        assert_ne!(detected, Some(monorepo));
    }

    /// `deck.jsonc` is NOT a detection marker (purged). `views/` is the source
    /// marker — even nested inside a tree that also has a `deck.jsonc`.
    #[test]
    fn stray_deck_jsonc_does_not_outrank_views_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let outer = tmp.path().to_path_buf();
        std::fs::write(outer.join("deck.jsonc"), "{}\n").unwrap();
        let inner = outer.join("nested");
        std::fs::create_dir(&inner).unwrap();
        std::fs::create_dir(inner.join("views")).unwrap();
        let detected = detect_project_root_from(&inner);
        assert_eq!(detected, Some(inner));
    }

    /// deck detects via its OWN marker (`views/`) only. A `.ship/views`-only
    /// layout is a HOST convention — detection must NOT key on it (a host with
    /// that layout constructs a `Resolution` with an explicit root).
    #[test]
    fn detection_ignores_dot_ship_views_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".ship").join("views")).unwrap();
        assert_eq!(detect_project_root_from(&root), None);
    }

    /// No marker anywhere up the tree → `None`.
    #[test]
    fn detection_returns_none_with_no_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let deep = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        assert!(detect_project_root_from(&deep).is_none());
    }

    /// A stray `deck.jsonc` is inert for detection — only `views/` counts.
    #[test]
    fn stray_deck_jsonc_is_not_a_detection_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::write(root.join("deck.jsonc"), r#"{ "viewDirs": ["evil"] }"#).unwrap();
        let inner = root.join("nested");
        std::fs::create_dir(&inner).unwrap();
        std::fs::write(inner.join("deck.jsonc"), "{}\n").unwrap();

        assert_eq!(detect_project_root_from(&root), None);
        assert_eq!(detect_project_root_from(&inner), None);

        std::fs::create_dir(inner.join("views")).unwrap();
        assert_eq!(detect_project_root_from(&inner), Some(inner.clone()));
        assert_eq!(detect_project_root_from(&root), None);
    }

    /// `inv-deck-mount-scoped-resolution` + `inv-deck-no-first-party`. Two
    /// projects in ONE process each carry their own `Resolution`; there is NO
    /// shared global for one to clobber the other through. Precedence is the
    /// ORDER of declared roots (`roots[0]` wins) — deck blesses no named
    /// 'first-party' bucket.
    #[test]
    fn resolution_is_mount_scoped_not_a_shared_global() {
        let a = Resolution::new().with_root("/tmp/projA");
        let b = Resolution::new().with_root("/tmp/projB");
        assert_eq!(a.project_root(), Some(Path::new("/tmp/projA")));
        assert_eq!(
            b.project_root(),
            Some(Path::new("/tmp/projB")),
            "mount-scoped: B is isolated from A — there is no shared global to clobber"
        );

        let host = Resolution::new()
            .with_root("/srv/app")
            .with_root("/srv/shared");
        assert_eq!(host.roots().len(), 2);
        assert_eq!(
            host.project_root(),
            Some(Path::new("/srv/app")),
            "first declared root wins by ordering — deck names no first-party bucket"
        );
    }

    /// The RESOLVE PATH the serve path threads (`resolve_project_root_with`)
    /// returns the per-request root and NOTHING when no Resolution is carried —
    /// there is no process global or cwd fallback to leak another project's root
    /// (`inv-deck-mount-scoped-resolution`, on the real path).
    #[test]
    fn resolve_path_uses_only_the_per_request_resolution() {
        let b = Resolution::new().with_root("/tmp/projB");
        assert_eq!(
            super::resolve_project_root_with(Some(&b)),
            Some(PathBuf::from("/tmp/projB")),
            "the resolve path returns the per-request Resolution's root"
        );
        // No Resolution ⇒ an honest miss (no global, no cwd floor).
        assert_eq!(
            super::resolve_project_root_with(None),
            None,
            "no Resolution ⇒ no root — never a shared global or cwd fallback"
        );
        // An empty Resolution carries no root ⇒ also a miss.
        let empty = Resolution::new();
        assert_eq!(super::resolve_project_root_with(Some(&empty)), None);
    }

    /// `inv-deck-output-location-single`, cross-crate: the runtime READER
    /// (`Resolution::route_bundle_roots`) and the bundler WRITER
    /// (`deck_bundler::targets::BundleLayout::deck_out`) resolve the SAME `.deck`
    /// path for the same project root — because both go through the one
    /// `deck_config::deck_out_path` join. This is the invariant in its strongest
    /// form: reader reads exactly where writer writes.
    #[test]
    fn reader_and_writer_agree_on_deck_output_path() {
        use deck_bundler::targets::BundleLayout;
        use deck_bundler::DeckConfig;
        let root = PathBuf::from("/proj");
        let writer_deck =
            BundleLayout::with_config(root.clone(), None, DeckConfig::defaults()).deck_out();
        let reader_deck = Resolution::new()
            .with_root(root.clone())
            .route_bundle_roots()
            .pop()
            .expect("project-root .deck fallback present");
        assert_eq!(
            writer_deck, reader_deck,
            "runtime reader must resolve the SAME .deck the bundler writer writes"
        );
    }

    /// The per-app view-output dirs are probed BEFORE the project-root `.deck`,
    /// and the project-root fallback resolves through the single output-dir
    /// constant — so the reader can't drift from the bundler writer.
    #[test]
    fn view_bundle_roots_precedence_and_single_output_join() {
        let mut r = Resolution::new().with_root("/proj");
        r.register_route_output("hub", "/apps/hub/.deck/views");
        let roots = r.route_bundle_roots();
        assert_eq!(roots[0], PathBuf::from("/apps/hub/.deck/views"), "per-app output first");
        assert_eq!(
            *roots.last().unwrap(),
            deck_config::deck_out_path(Path::new("/proj")),
            "project-root fallback is <root>/.deck via the single shared constant"
        );
    }
}
