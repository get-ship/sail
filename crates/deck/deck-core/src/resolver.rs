use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Per-instance resolution context — the mount-scoped root(s) + per-app route
/// outputs a request resolves against. This is the WHOLE of deck's resolution
/// state: there is no process-global `PROJECT_ROOT_OVERRIDE` / `VIEW_OUTPUTS`
/// backing it, so one process can serve many projects with no cross-talk
/// (`inv-deck-mount-scoped-resolution`). A host constructs one per project and
/// carries it per-request (an axum `Extension`, like the `DataPort`); the serve
/// path threads `Option<&Resolution>` end to end.
///
/// The host declares route roots in PRECEDENCE ORDER and expresses "these are
/// mine first" by ordering — deck privileges no named 'first-party' bucket
/// (`inv-deck-no-first-party`); `roots[0]` simply wins.
#[derive(Debug, Clone, Default)]
pub struct Resolution {
    roots: Vec<PathBuf>,
    route_outputs: BTreeMap<String, PathBuf>,
}

impl Resolution {
    /// An empty context — no roots, no route outputs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a route root. Earlier calls have higher precedence (first wins).
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.roots.push(root.into());
        self
    }

    /// Seed a Resolution by DETECTING the project root from `cwd` —
    /// [`sail_lint::Project::discover`] (nearest `sail.config`) plus a
    /// documented `routes/`-only zero-config Deck app (no sail.config on the
    /// walk), else `cwd` itself.
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

    /// Register a per-app route-output dir. Hosts call this for each app whose
    /// built route bundles live outside the conventional `<root>/dist`
    /// (a dir containing `routes/<name>/route.js`). Probed BEFORE the
    /// project-root `dist` fallback by [`route_bundle_roots`](Self::route_bundle_roots).
    /// `views/` is not a public Deck out layout.
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

    /// The route-output dir for `app`, if registered.
    pub fn route_output(&self, app: &str) -> Option<&Path> {
        self.route_outputs.get(app).map(|p| p.as_path())
    }

    /// Route-bundle roots to probe for `routes/<name>/route.js`, in
    /// precedence order: the per-app route-output dirs first (host-registered),
    /// then the project root's `<root>/dist` (the bundler's default app out).
    /// The join is [`deck_config::deck_out_path`].
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

/// One walk, one hop cap ([`sail_lint::project::DISCOVER_MAX_HOPS`]).
///
/// 1. Nearest `sail.config.jsonc` / `.json` — same as
///    [`sail_lint::Project::discover`].
/// 2. Else the nearest `routes/`-only directory (zero-config Deck; no
///    sail.config on the walk). `routes/` is a FileKind, not a tie-break
///    against a closer config.
///
/// Never a TOPMOST-Cargo.toml workspace resolver (that would shadow a nested
/// project's root onto the monorepo). Never `dist`, `deck.jsonc`, or `views/`.
pub(crate) fn detect_project_root_from(cwd: &Path) -> Option<PathBuf> {
    let mut cur = if cwd.is_file() {
        cwd.parent()?.to_path_buf()
    } else {
        cwd.to_path_buf()
    };
    let mut first_routes = None;
    for _ in 0..sail_lint::project::DISCOVER_MAX_HOPS {
        if sail_lint::project::config_file_in(&cur).is_some() {
            return Some(cur);
        }
        if first_routes.is_none() && cur.join("routes").is_dir() {
            first_routes = Some(cur.clone());
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => break,
        }
    }
    first_routes
}

use deck_host::resolver::RouteLayout;
pub use deck_host::resolver::{ResolveError, ResolvedRoute};
/// Build a [`RouteLayout`] from the conventional `["routes"]` bucket with a
/// single `page.sail` leaf. The leaf is unified across all buckets (fixes the
/// #254 `index.sail` vs `page.sail` divergence). The framework loads no config
/// file — a host that needs a different layout constructs a `DeckConfig` and
/// threads it into `Bundler::start`; the bundler's route-discovery walks
/// `cfg.route_dirs` directly. The runtime resolver here reports the conventional
/// defaults.
fn route_layout() -> RouteLayout {
    RouteLayout::conventional()
}

/// Resolve a route by name under an EXPLICIT project `root` (mount-scoped — the
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

/// Best-effort list of top-level route names servable under `root`, for the
/// human-facing not-found page. Scans the conventional route buckets (default
/// `["routes"]`) under the given root. Cheap (one shallow `read_dir` per bucket,
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

/// Add each immediate child directory of `dir` whose name is a valid route name
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
    use super::{Resolution, detect_project_root_from};
    use std::path::{Path, PathBuf};

    /// Nested project: monorepo root has a `routes/`, the inner project's
    /// `app/` ALSO has a `routes/`. Detection must pick the NEAREST one
    /// (`app/`), not the topmost (monorepo).
    #[test]
    fn detection_picks_nearest_ancestor_marker_not_topmost() {
        let tmp = tempfile::tempdir().unwrap();
        let monorepo = tmp.path().to_path_buf();
        std::fs::create_dir(monorepo.join("routes")).unwrap();
        let app = monorepo.join("app");
        std::fs::create_dir(&app).unwrap();
        std::fs::create_dir(app.join("routes")).unwrap();
        let deep = app.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();

        let detected = detect_project_root_from(&deep);
        assert_eq!(
            detected,
            Some(app.clone()),
            "nearest-ancestor-wins: expected the inner app root, got {detected:?}",
        );
        assert!(monorepo.join("routes").is_dir());
        assert_ne!(detected, Some(monorepo));
    }

    /// `deck.jsonc` is NOT a detection marker (purged). `routes/` is a source
    /// marker — even nested inside a tree that also has a `deck.jsonc`.
    #[test]
    fn stray_deck_jsonc_does_not_outrank_routes_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let outer = tmp.path().to_path_buf();
        std::fs::write(outer.join("deck.jsonc"), "{}\n").unwrap();
        let inner = outer.join("nested");
        std::fs::create_dir(&inner).unwrap();
        std::fs::create_dir(inner.join("routes")).unwrap();
        let detected = detect_project_root_from(&inner);
        assert_eq!(detected, Some(inner));
    }

    /// deck detects via its OWN markers (`routes/` / `sail.config`) only. A
    /// `.ship/views`-only layout is a HOST convention — detection must NOT
    /// key on it (a host with that layout constructs a `Resolution` with an
    /// explicit root).
    #[test]
    fn detection_ignores_dot_ship_views_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".ship").join("views")).unwrap();
        assert_eq!(detect_project_root_from(&root), None);
    }

    /// `views/` is a retired Ship-host source bucket, not a public Deck
    /// marker. A views-only tree is not a Sail/Deck project.
    #[test]
    fn detection_ignores_views_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir(root.join("views")).unwrap();
        assert_eq!(detect_project_root_from(&root), None);
        let deep = root.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();
        assert_eq!(detect_project_root_from(&deep), None);
    }

    /// A `sail.config.jsonc` (or `.json`) is a detection marker — same as
    /// `Project::discover`. Config alone identifies the project; `routes/`
    /// is not required.
    #[test]
    fn detection_accepts_sail_config_without_routes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::write(root.join("sail.config.jsonc"), "{}\n").unwrap();
        let deep = root.join("src");
        std::fs::create_dir(&deep).unwrap();
        assert_eq!(detect_project_root_from(&deep), Some(root.clone()));

        let json_only = tempfile::tempdir().unwrap();
        let json_root = json_only.path().to_path_buf();
        std::fs::write(json_root.join("sail.config.json"), "{}\n").unwrap();
        assert_eq!(detect_project_root_from(&json_root), Some(json_root));
    }

    /// Zero-config Deck: `routes/` and no sail.config still resolves.
    #[test]
    fn detection_accepts_routes_only_zero_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir(root.join("routes")).unwrap();
        let deep = root.join("src");
        std::fs::create_dir(&deep).unwrap();
        assert_eq!(detect_project_root_from(&deep), Some(root.clone()));
        assert_eq!(detect_project_root_from(&root), Some(root));
    }

    /// Nested package config (no routes) wins over an ancestor app that
    /// has routes+config — same root as `Project::discover`.
    #[test]
    fn detection_nearest_config_wins_over_ancestor_routes() {
        let tmp = tempfile::tempdir().unwrap();
        let app = tmp.path().join("hello");
        std::fs::create_dir_all(app.join("routes")).unwrap();
        std::fs::write(app.join("sail.config.jsonc"), "{}\n").unwrap();
        let foo = app.join("packages/foo");
        std::fs::create_dir_all(&foo).unwrap();
        std::fs::write(foo.join("sail.config.json"), "{}\n").unwrap();
        assert_eq!(detect_project_root_from(&foo), Some(foo));
    }

    /// No marker anywhere up the tree → `None`.
    #[test]
    fn detection_returns_none_with_no_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let deep = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        assert!(detect_project_root_from(&deep).is_none());
    }

    /// A stray `deck.jsonc` is inert for detection — only `routes/` /
    /// `sail.config` count.
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

        std::fs::create_dir(inner.join("routes")).unwrap();
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

    /// App serve reads `dist/`. Workspace bake writes `.sail/`.
    /// Those are different homes.
    #[test]
    fn app_serve_is_dist_workspace_bake_is_sail() {
        use deck_bundler::DeckConfig;
        use deck_bundler::targets::BundleLayout;
        let root = PathBuf::from("/proj");
        let bake = BundleLayout::with_config(root.clone(), None, DeckConfig::defaults()).deck_out();
        let serve = Resolution::new()
            .with_root(root.clone())
            .route_bundle_roots()
            .pop()
            .expect("project-root dist fallback present");
        assert_eq!(bake, deck_config::sail_generate_path(&root));
        assert_eq!(serve, deck_config::deck_out_path(&root));
        assert_ne!(bake, serve);
    }

    /// The per-app route-output dirs are probed BEFORE the project-root `dist`.
    #[test]
    fn view_bundle_roots_precedence_and_single_output_join() {
        let mut r = Resolution::new().with_root("/proj");
        r.register_route_output("hub", "/apps/hub/dist");
        let roots = r.route_bundle_roots();
        assert_eq!(
            roots[0],
            PathBuf::from("/apps/hub/dist"),
            "per-app output first"
        );
        assert_eq!(
            *roots.last().unwrap(),
            deck_config::deck_out_path(Path::new("/proj")),
            "project-root fallback is <root>/dist"
        );
    }
}
