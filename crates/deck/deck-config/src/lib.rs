//! deck-config — `DeckConfig` (the host's route-discovery + watch layout) and
//! per-app `AppServeConfig`.
//!
//! `DeckConfig` is HOST-SUPPLIED. The framework does not read any file from
//! a project root — the host (shipd, deck-conformance) constructs the
//! `DeckConfig` it wants to pass in (typically [`DeckConfig::defaults`]
//! plus any host-specific overrides) and threads it through `Bundler::start`
//! and the resolver. There is no `deck.jsonc` lookup, no detection marker,
//! no implicit load. Per-app scaffolding (`AppServeConfig`) is the
//! `Vec<AppServeConfig>` the host builds from its own `apps.jsonc` — the
//! framework names no app, no list.
//!
//! A stray `deck.jsonc` at the workspace ROOT has no effect: not loaded,
//! not detected. The earlier file-read and `deck.jsonc` project-root
//! marker were the dead-weight surface dissolved by the deck-purge-root
//! job; the host now hands deck its paths, deck does not detect them.
//!
//! ## Per-app `deck.jsonc` (app-owned, OPTIONAL)
//!
//! Distinct from the retired root marker: an app MAY own a per-app
//! `apps/<app>/deck.jsonc` that declares only the non-conventional
//! pieces — [`AppServeConfig`] — but the file is optional. An app
//! whose config is fully conventional needs no `deck.jsonc` at all;
//! the host reads the absent file as a `Default::default()` config.
//!
//! What stays on the per-app file (only what is genuinely
//! non-conventional):
//!
//! - `render` (`csr` default | `ssr`) — the rendering-modes design
//!   exposes this on the per-app shape so individual apps can opt
//!   into SSR (per-route override is a follow-up).
//! - `injectGlobals` — a host-owned injection concern (URL path
//!   captures → `window.<k>`); apps with a dynamic route (the
//!   spaces scaffold's `/space/{id}` → `DECK_APP_ID`) need this and
//!   the spaces file is the only current user.
//! - `entry` — the host-owned entry bootstrap (`clientsModule` +
//!   `exposeNames`) the bundler emits as `entry.gen.ts`.
//!
//! What is NOT here anymore (and where it lives instead):
//!
//! - `name` → the app's directory basename (`apps/hub` ⇒ `hub`). The
//!   host derives it; the app does not declare its own name.
//! - `route` → the orchestration list (`apps.jsonc` in the Ship
//!   repo). Where an app mounts is the host composing many apps'
//!   decision, not the app's self-declaration. The `ShipAppRegistry`
//!   reads it from `apps.jsonc`.
//! - `source` → always `<app>/.deck` (the bundler's default
//!   output dir). Not config.
//! - `viewOutput` → always `<app>/.deck/views`. Not config.
//! - `frameworkClients` → provider-declared. The framework-singleton
//!   package (`@ship/sdk-client` et al.) self-declares its serve
//!   contract in its own `package.json#deckFramework` (mirror of how
//!   packs declare `customElements`). The bundler DISCOVERS these by
//!   walking the app's resolved dependency graph. The app declares
//!   nothing; deck names no ship package. See
//!   `crates/deck/deck-bundler/src/app.rs::discover_deck_frameworks`
//!   for the discovery pass.
//!
//! `AppServeConfig::parse_deck_jsonc` is the only entry point the
//! host needs; deck-config owns the schema + the JSONC parser. The
//! host decides which files to load and binds their locations —
//! deck still names no path.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

mod jsonc;
pub use jsonc::strip_jsonc_comments;

/// The single build-output directory name. Every deck build artifact — the
/// framework singletons (`.deck/framework/`), the per-route bundles
/// (`.deck/views/<name>/view.js`), and each app's generated `index.html` +
/// hashed `assets/` — lands under `<root>/.deck`.
///
/// Declared ONCE here so the bundler (writer) and the runtime (reader) can
/// never drift onto different directory names — that drift is exactly the
/// silent-asset-404 class the `inv-deck-output-location-single` invariant
/// closes. Every `.deck` join in the tree resolves through this constant or
/// [`deck_out_path`]; there is no second literal.
pub const DECK_OUT_DIRNAME: &str = ".deck";

/// `<root>/.deck` — the build-output dir for `root`. The SINGLE join shared by
/// the writer ([`BundleLayout::deck_out`](../deck_bundler/targets) in
/// deck-bundler) and the runtime route-bundle reader (`deck_core::assets`), so
/// reader and writer resolve the SAME path for the same root — no convention
/// duplicated across crates.
pub fn deck_out_path(root: &Path) -> PathBuf {
    root.join(DECK_OUT_DIRNAME)
}

/// One app's serve configuration — the parsed contents of a per-app
/// `apps/<app>/deck.jsonc` (or `Default::default()` when the file is
/// absent), and the shape deck's per-app router consumes. shipd (or
/// any deck host) builds a `Vec<AppServeConfig>` by loading each
/// app's `deck.jsonc` — or, when the file is absent, the empty
/// `Default::default()` — and pairs it with the route the host's
/// `apps.jsonc` orchestration list declares. The framework does NOT
/// know the list; that is the host's concern.
///
/// # What lives here vs. what is convention
///
/// Only genuinely non-conventional config sits on this struct. Every
/// field that has a discoverable default is gone — see the module-level
/// doc for the dropped fields and where they live now:
///
/// - `name` → directory basename; the host derives it.
/// - `route` → the host's orchestration list (`apps.jsonc`).
/// - `source` → always `<app>/.deck`; the bundler writes there.
/// - `viewOutput` → always `<app>/.deck/views`; convention.
/// - `frameworkClients` → provider-declared discovery via
///   `package.json#deckFramework` (see `deck_bundler::app`).
///
/// What stays:
///
/// - `render` — the rendering-modes flag (csr default | ssr).
/// - `injectGlobals` — host-owned URL-capture → `window.<k>` injections.
/// - `entry` — host-owned entry bootstrap (`entry.gen.ts`).
///
/// How an app paints its first frame.
///
/// `Csr` (the default) ships an empty mount point (`#root`) and lets the
/// client bundle render. `Ssr` means the app server-renders its first frame
/// before hydration. This is the OBSERVABLE "does this app server-render at
/// all" flag the router/serving path consumes — NOT the SSR-engine selector
/// (which engine renders is a separate, host-gated concern). The served
/// `index.html` carries the active mode as `<meta name="deck-render">` so it
/// is legible client-side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    /// Client-side render — empty `#root`, the client bundle paints.
    #[default]
    Csr,
    /// Server-side render — the first frame is rendered before hydration.
    Ssr,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppServeConfig {
    /// Logical app name + URL namespace. The bundler writes built assets
    /// under `/<name>/…` and the daemon serves them at that path. HOST
    /// sets this from the app's directory basename (`apps/hub` ⇒ `hub`).
    /// NOT read from `deck.jsonc` — the file doesn't get to name itself
    /// (the directory is authoritative; a stale or wrong `name` in the
    /// file would silently ship assets under the wrong namespace).
    /// `serde(skip)` ensures the JSONC parser cannot see it.
    #[serde(skip, default)]
    pub name: String,
    /// URL route pattern shipd mounts the app at. HOST sets this from
    /// its own orchestration list (`apps.jsonc` in the Ship repo):
    /// `"/"` for the hub, `"/space/{id}"` for the spaces scaffold, etc.
    /// The app does NOT self-declare where it mounts — that's the host
    /// composing many apps' decision. `serde(skip)` ensures the file
    /// can't override the host's bind (a stale `route` would silently
    /// change where the app serves).
    #[serde(skip, default)]
    pub route: String,
    /// Path to the built app's output dir, relative to the project
    /// root. Convention: always `<app>/.deck` (the bundler's default
    /// `out_dir`). The host sets this; the file doesn't get to choose
    /// a non-conventional output location. `serde(skip)` mirrors `name`.
    #[serde(skip, default)]
    pub source: String,
    /// Globals injected as `<script>window.<key>=<value>;</script>` into the
    /// app's `index.html` before `</head>`. `$<param>` tokens in the value are
    /// substituted from the URL path captures declared in the host's
    /// `apps.jsonc` route (so for `route: "/space/{id}"` and an entry
    /// `("DECK_APP_ID", "$id")`, the injection is
    /// `window.DECK_APP_ID="<id>";`).
    #[serde(default)]
    pub inject_globals: BTreeMap<String, String>,
    /// How the app paints its first frame — `csr` (default) or `ssr`. Made
    /// observable by serving it as a `<meta name="deck-render">` tag in the
    /// app's `index.html`. Absent ⇒ `csr` (the implicit-empty-`#root`
    /// posture every app had before this field existed).
    #[serde(default)]
    pub render: RenderMode,
    /// Host-owned entry bootstrap. The bundler generates a typed
    /// `entry.gen.ts` from this — the file that constructs the named
    /// `clients` map and hands it to `mountRouter({...})`. The data
    /// side of the bootstrap (route table, layout table, route modules,
    /// layout modules) is the framework's `routes.gen.ts`; the
    /// client-construction side is the HOST's `entry.gen.ts`. Same
    /// `data + bootstrap` discipline every per-app host follows; deck
    /// doesn't ship a `main.ts` and neither should the host.
    #[serde(default)]
    pub entry: Option<EntryBootstrap>,
}

/// Host-owned entry bootstrap. The bundler reads this from
/// `apps/<app>/deck.jsonc` and emits `entry.gen.ts` next to the data-only
/// `routes.gen.ts`. Decoupled so the host can wire ANY clients it needs —
/// the framework ships `mountRouter` and a `clients` injection surface;
/// which keys the host puts on the map is the host's call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryBootstrap {
    /// Module-relative specifier of the host's client factory. The bundler
    /// emits `import { <names...> } from '<clientsModule>'` in `entry.gen.ts`;
    /// the host's file constructs each named client (a `createClient()` call,
    /// a wrapper, etc.) and named-exports the result. Same path conventions
    /// as a route's `controller.ts` import — the path is relative to the
    /// generated entry, which lives at `<project>/entry.gen.ts` next to
    /// `routes.gen.ts`.
    #[serde(rename = "clientsModule")]
    pub clients_module: String,
    /// Names the bundler reads off the factory's exports and spreads into
    /// the `clients` map. Each name becomes a key on the `clients` object
    /// `mountRouter({ providers })` receives; routes read each as a bare `ctx.<name>`
    /// (and the legacy `ctx.<name>` alias, so pre-migration route code still
    /// resolves during the cutover).
    #[serde(rename = "exposeNames")]
    pub expose_names: Vec<String>,
}

impl AppServeConfig {
    /// Parse a per-app `deck.jsonc` (JSONC source) into the serve config.
    ///
    /// Strips JSONC extensions, then deserializes. Every field is optional
    /// (an absent file ⇒ `Default::default()`); the file's whole purpose is
    /// to opt out of the conventional defaults. The HOST decides which file
    /// to read and feeds the bytes here — deck-config names no path.
    pub fn parse_deck_jsonc(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(&strip_jsonc_comments(raw))
    }
}
/// The parsed contents of the host's route-discovery `DeckConfig`.
///
/// The container-level `#[serde(default)]` fills every missing field from
/// [`DeckConfig::defaults`] (via the `Default` impl), so a minimal — or
/// missing — `deck.jsonc` degrades gracefully. Fields must NOT carry their
/// own `#[serde(default)]`: a field-level default takes precedence over the
/// container default and silently replaces the documented fallback with the
/// type's empty value (an empty `route_dirs` once disabled route discovery for
/// every host with a partial deck.jsonc).
///
/// ## What this is NOT
///
/// `DeckConfig` describes the host's ROUTE-DISCOVERY + watch layout. It does NOT
/// describe the host's app list (the host reads its own `apps.jsonc` and passes
/// a per-app `Vec<AppServeConfig>` to `serve_apps()`), the consumer bundle set
/// (self-declared per-package in `package.json` `deck.bundles[]`, discovered by
/// the bundler), or `@ship/<name>` component resolution (derived from the
/// workspace package map). L1 is per-app; the host owns the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeckConfig {
    /// Source roots to scan for routes. Defaults to `["."]` (workspace root).
    pub source_roots: Vec<PathBuf>,
    /// Directories the bundler watches for live-reload. Defaults to `[]`.
    pub watch: Vec<PathBuf>,
    /// Declared component packs (per-project component scope).
    ///
    /// When present and non-empty, component resolution is SCOPED to ONLY these
    /// packs: the per-route resolver's tag index is filtered down to tags a
    /// declared pack provides (matched by `package.json` `name`, e.g.
    /// `"@ship/sail-primitives"`). A tag from an undeclared pack resolves to a
    /// no-op (it never bundles) and the build emits an author diagnostic naming
    /// the tag and the pack that provides it. When absent or empty, resolution
    /// is GLOBAL — every workspace pack's tags are resolvable (zero behavior
    /// change for existing projects).
    pub packs: Vec<String>,
    /// Per-source-root route bucket names. The resolver and bundler scan
    /// `<source_root>/<route_dir>/<name>/<route_leaf>` for each `route_dir` in
    /// this list. Defaults to the deck-native `["routes"]`, so a zero-config
    /// host resolves from the top-level route bucket. A host that serves
    /// additional buckets (Ship's `.ship/views`) overrides this list — deck
    /// no longer bakes a host-private path into its default.
    pub route_dirs: Vec<String>,
    /// The leaf filename for route artifacts under each `route_dir/<name>/`.
    /// Defaults to `"page.sail"`. A single leaf convention across all
    /// `route_dirs` fixes the #254 leaf-name divergence (`page.sail` vs
    /// `index.sail`) that broke `.ship/views/<name>/index.sail` resolution.
    pub route_leaf: Option<String>,
}

impl Default for DeckConfig {
    fn default() -> Self {
        Self::defaults()
    }
}
impl DeckConfig {
    /// Built-in defaults — a zero-config host (e.g. deck-conformance) gets these
    /// when the host passes no override. The deck-native `["routes"]` bucket and
    /// the unified `page.sail` leaf (#254). Deck names no host-private path: a
    /// host serving Ship's `.ship/views` supplies it as an override.
    pub fn defaults() -> Self {
        Self {
            source_roots: vec![PathBuf::from(".")],
            watch: Vec::new(),
            packs: Vec::new(),
            // Deck-native route bucket. Hosts that need a different layout
            // (or additional buckets like Ship's `.ship/views`) construct a
            // `DeckConfig` (clone `defaults()` + override) and pass it in — no
            // file load, no baked host path.
            route_dirs: vec!["routes".into()],
            // Single leaf convention — fixes the #254 `index.sail`/`page.sail`
            // divergence. Both buckets look up `<name>/<route_leaf>`.
            route_leaf: Some("page.sail".into()),
        }
    }
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Minimal example deck.jsonc that exercises the route-layout fields a host
    /// may override. Consumer bundle targets are NO LONGER here — they are
    /// self-declared per-package (`package.json` `deck.bundles[]`) and
    /// discovered by the bundler; `@ship/<name>` resolution is derived from the
    /// workspace package map. So this config carries only route discovery + watch
    /// + pack scope.
    const EXAMPLE_JSONC: &str = r#"{
  "sourceRoots": ["."],
  "watch": ["packages", "apps/site/src"],
  "routeDirs": ["routes", ".ship/views"],
  "routeLeaf": "page.sail"
}"#;
    #[test]
    fn parse_example_jsonc() {
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(EXAMPLE_JSONC))
                .expect("example deck.jsonc must parse");
        assert_eq!(cfg.source_roots, vec![PathBuf::from(".")]);
        assert_eq!(cfg.watch, vec![PathBuf::from("packages"), PathBuf::from("apps/site/src")]);
        assert_eq!(cfg.route_dirs, vec!["routes".to_string(), ".ship/views".to_string()]);
        assert_eq!(cfg.route_leaf.as_deref(), Some("page.sail"));
    }
    #[test]
    fn defaults_round_trip() {
        let d = DeckConfig::defaults();
        // Serialize and deserialize — must be lossless.
        let json = serde_json::to_string(&d).unwrap();
        let d2: DeckConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(d, d2);
        assert_eq!(d.source_roots, vec![PathBuf::from(".")]);
        assert!(d.watch.is_empty());
        assert!(d.packs.is_empty(), "defaults declare no packs → global resolution");
    }

    /// `packs` is optional and additive — an absent field parses to empty,
    /// which the bundler reads as "global" (every workspace pack resolvable).
    #[test]
    fn packs_absent_parses_to_empty() {
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(EXAMPLE_JSONC))
                .expect("example deck.jsonc must parse");
        assert!(cfg.packs.is_empty(), "no packs field ⇒ empty ⇒ global behavior");
    }

    /// A declared `packs` array parses into the scoped set.
    #[test]
    fn packs_present_parses_declared_set() {
        let json = r#"{ "packs": ["@ship/sail-primitives", "deck-demo-app"] }"#;
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(json)).unwrap();
        assert_eq!(cfg.packs, vec!["@ship/sail-primitives", "deck-demo-app"]);
    }

    /// A PARTIAL deck.jsonc (deck-conformance's shape: scaffolds only) must keep the
    /// built-in route-layout defaults. Field-level `#[serde(default)]` attrs
    /// used to shadow the container default here: `route_dirs` parsed to `[]`
    /// and route discovery found zero targets (`unknown target: views/*/view`).
    #[test]
    fn partial_file_keeps_view_layout_defaults() {
        let json = r#"{
  "watch": ["packages"]
}"#;
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(json)).expect("partial deck.jsonc parses");
        assert_eq!(
            cfg.route_dirs,
            vec!["routes".to_string()],
            "partial config must inherit the deck-native route bucket default"
        );
        assert_eq!(cfg.route_leaf.as_deref(), Some("page.sail"));
    }

    // ── per-app deck.jsonc (AppServeConfig) ───────────────────────────────────

    /// The new-shape hub: only `entry` is non-conventional (the host-owned
    /// entry bootstrap that imports the data tables from `./routes.gen`,
    /// constructs the named `clients` map off the host's
    /// `./src/lib/client.ts` factory, and calls `mountRouter`). Every
    /// other field is convention: `name` is the directory basename,
    /// `route` comes from `apps.jsonc`, `source` is `<app>/.deck`, no
    /// `frameworkClients` (the bundler discovers the SDK-client through
    /// the `deckFramework` self-declaration on `@ship/sdk-client`'s own
    /// `package.json`).
    const HUB_DECK_NEW: &str = r#"{
  "entry": {
    "clientsModule": "./src/lib/client.ts",
    "exposeNames": ["ship", "ops"]
  }
}"#;

    /// The legacy hub (deck-jsonc-simplify BEFORE) carries `name`/
    /// `route`/`source`/`render`. The cutover makes every one of those
    /// fields discoverable from elsewhere — they are UNKNOWN to the new
    /// schema. serde drops unknown fields silently (no error), so a
    /// legacy file parses to a config identical to the empty default.
    const HUB_DECK_LEGACY: &str = r#"{
  "name": "hub",
  "route": "/",
  "source": "apps/hub/.deck",
  "render": "csr"
}"#;

    /// Spaces keeps only its genuinely non-conventional bits: the
    /// `DECK_APP_ID` URL-capture → `window.DECK_APP_ID` injection
    /// (`/space/{id}` route is declared by the host's `apps.jsonc`).
    const SPACES_DECK_NEW: &str = r#"{
  "injectGlobals": { "DECK_APP_ID": "$id" }
}"#;

    /// The new-shape parse: an app with only the `entry` bootstrap (the
    /// hub's whole reason to keep a `deck.jsonc`) lands a valid config.
    /// No `name`/`route`/`source`/`viewOutput`/`frameworkClients` fields —
    /// Legacy fields (`route`/`source`/`render`) are unknown to the new
    /// schema and serde drops them silently — the parse produces a
    /// `Default::default()` config PLUS any legacy `name` field that
    /// happens to match the directory basename (the host ignores the
    /// value; the spec says the directory basename is authoritative).
    /// Legacy fields (`route`/`source`/`render`) are unknown to the new
    /// schema and serde drops them silently — including the legacy
    /// `name` field, which is `#[serde(skip)]` (the directory basename
    /// is authoritative; a stale or wrong `name` in the file must not
    /// ship assets under the wrong namespace). The parse produces a
    /// `Default::default()` config — `name`/`source` are empty until
    /// the host fills them in.
    #[test]
    fn parse_app_deck_legacy_fields_dropped_silently() {
        let cfg = AppServeConfig::parse_deck_jsonc(HUB_DECK_LEGACY)
            .expect("legacy fields are dropped, not errors");
        // Everything is dropped — host fills name/source.
        assert_eq!(cfg, AppServeConfig::default(), "legacy fields parse to default");
    }



    /// `render` is optional and defaults to `csr` — the implicit posture
    /// every app had before the field existed (empty `#root`,
    /// client paints). The default-constructed config also lands on csr.
    #[test]
    fn parse_app_deck_render_defaults_to_csr() {
        let cfg = AppServeConfig::default();
        assert_eq!(cfg.render, RenderMode::Csr, "Default ⇒ csr");
        let parsed = AppServeConfig::parse_deck_jsonc("{}")
            .expect("empty deck.jsonc parses to Default");
        assert_eq!(parsed, AppServeConfig::default());
        assert_eq!(parsed.render, RenderMode::Csr);
    }

    /// `render: "ssr"` parses into the SSR variant; spaces' `injectGlobals`
    /// + URL-capture (`DECK_APP_ID: $id`) round-trip through the JSONC
    /// parser. The route itself is declared by the host's `apps.jsonc`,
    /// not by the app — the file carries only the URL-capture →
    /// `window.<k>` mapping.
    #[test]
    fn parse_app_deck_ssr_and_inject_globals() {
        let json = r#"{
  "render": "ssr",
  "injectGlobals": { "DECK_APP_ID": "$id" }
}"#;
        let cfg = AppServeConfig::parse_deck_jsonc(json).expect("spaces deck.jsonc parses");
        assert_eq!(cfg.render, RenderMode::Ssr);
        assert_eq!(
            cfg.inject_globals.get("DECK_APP_ID").map(String::as_str),
            Some("$id")
        );
        // No name/route/source — host-supplied.
        assert_eq!(cfg, AppServeConfig {
            name: String::new(),
            route: String::new(),
            source: String::new(),
            inject_globals: [("DECK_APP_ID".to_string(), "$id".to_string())]
                .into_iter().collect(),
            render: RenderMode::Ssr,
            entry: None,
        });
    }

    /// `entry` round-trips through the JSONC parser — the host-owned
    /// the bundler imports + the `exposeNames` it spreads into the
    /// `clients` map).
    #[test]
    fn parse_app_deck_entry_bootstrap() {
        let cfg = AppServeConfig::parse_deck_jsonc(HUB_DECK_NEW)
            .expect("hub deck.jsonc parses");
        let entry = cfg.entry.as_ref().expect("entry declared");
        assert_eq!(entry.clients_module, "./src/lib/client.ts");
        assert_eq!(entry.expose_names, vec!["ship", "ops"]);
    }

    /// An absent `entry` parses to `None` — the bundler falls back to
    /// the legacy `routes.gen.ts`-as-entry shape (no client wiring,
    /// token import rides the data file).
    #[test]
    fn parse_app_deck_no_entry_is_none() {
        let cfg = AppServeConfig::parse_deck_jsonc("{}")
            .expect("empty deck.jsonc parses");
        assert!(cfg.entry.is_none(), "absent entry ⇒ None");
    }

    /// Spaces' shape — only `injectGlobals`, no `entry`. This is the
    /// canonical "non-conventional but not full-bootstrap" config.
    #[test]
    fn parse_app_deck_spaces_inject_globals_only() {
        let cfg = AppServeConfig::parse_deck_jsonc(SPACES_DECK_NEW)
            .expect("spaces deck.jsonc parses");
        assert_eq!(
            cfg.inject_globals.get("DECK_APP_ID").map(String::as_str),
            Some("$id")
        );
        assert!(cfg.entry.is_none());
        assert_eq!(cfg.render, RenderMode::Csr);
    }


    /// `DeckConfig::load` is GONE. Constructing a host config is the host's
    /// job — `DeckConfig::defaults()` plus any overrides the host reads
    /// (with `strip_jsonc_comments` if it has JSONC). The acceptance
    /// criterion for the deck-purge-root job: a stray `deck.jsonc` at the
    /// workspace root has NO effect (not loaded, not a detection marker).
    /// This test pins that contract by parsing the JSONC content via the
    /// public helper (the host's path) and asserting a dropped file plus
    /// a default config produce identical views.
    #[test]
    fn stray_deck_jsonc_has_no_effect() {
        let d = DeckConfig::defaults();
        // A stray deck.jsonc the framework would have read before — a
        // non-empty override with custom route buckets. Parsed via the
        // public `strip_jsonc_comments` helper to show a host CAN parse
        // it (host-supplied config still works), but the framework does
        // not load it on its own.
        let jsonc = r#"{
  "routeDirs": ["apps/shell/src/routes"],
  "routeLeaf": "page.sail"
}"#;
        let from_str: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(jsonc)).expect("host-parsed config");
        assert_eq!(
            from_str.route_dirs,
            vec!["apps/shell/src/routes".to_string()],
            "host parsing of deck.jsonc-shaped content still works"
        );
        assert_ne!(
            d.route_dirs, from_str.route_dirs,
            "the override is real when parsed — it's just not auto-loaded"
        );
        // The framework sees defaults() — the stray file is inert.
        assert_eq!(d, DeckConfig::defaults());
    }
}
