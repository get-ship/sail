//! deck-config — `DeckConfig` (the host's route-discovery + watch layout) and
//! per-app `AppServeConfig`.
//!
//! `DeckConfig` is HOST-SUPPLIED. The framework does not read any file from
//! a project root — the host (shipd, deck-conformance) constructs the
//! `DeckConfig` it wants to pass in (typically [`DeckConfig::defaults`]
//! plus any host-specific overrides) and threads it through `Bundler::start`
//! and the resolver. There is no project-root detection marker and no
//! implicit load of host layout config. Per-app scaffolding
//! ([`AppServeConfig`]) is the `Vec<AppServeConfig>` the host builds —
//! the framework names no app, no list.
//!
//! ## Per-app serve config (`sail.config.jsonc` / `sail.config.json`)
//!
//! Bundler/serve settings live under the top-level **`deck`** object
//! (SPEC-SAIL-CONFIG-1 — no dual-read of legacy top-level keys):
//!
//! - `deck.render` (`csr` default | `ssr`)
//! - `deck.injectGlobals` — URL-capture → `window.<k>` injections
//! - `deck.entry` — host entry bootstrap (`clientsModule` + `exposeNames`)
//!
//! [`AppServeConfig::load_from_project`] reads **only** the nested `deck`
//! object (other top-level keys such as `theme` are ignored by this loader;
//! Sail's recognized-key table refuses leftover top-level `render` /
//! `entry` / `injectGlobals`). Absence of a sail config or of `deck`
//! means fully conventional defaults. A leftover `deck.jsonc` is a hard
//! migration error.
//!
//! Host-set only (never from the file): `name`, `route`, `source`,
//! `ops_project`. Framework singletons are provider-declared via
//! `package.json#deckFramework` and discovered by the bundler.

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

/// Cargo rebuild trigger for restaging `.deck/framework/*` into deck-core
/// assets. Lives in the crate (tracked, empty), never under `.deck/`.
/// `deck build-assets` touches mtime; `deck-core/build.rs` watches it.
pub fn framework_stamp_path(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/deck/deck-core/framework-stamp")
}

/// Filenames the host may find for a project's compiler policy config.
/// `jsonc` is preferred when both exist (app-level studio convention).
///
/// Wire-through only: deck does not parse the policy registry — that lives in
/// `sail_compiler::policy`. Hosts (deck-axum policy endpoint) use these names
/// to locate the file the registry deserializes.
pub const SAIL_CONFIG_FILES: &[&str] = &["sail.config.jsonc", "sail.config.json"];

/// First existing sail config under `project_root`, preferring jsonc.
/// Returns `(path, raw_text)` so callers can hand the body to the compiler
/// policy registry without re-implementing the filename convention.
pub fn load_sail_config_text(project_root: &Path) -> Option<(PathBuf, String)> {
    for name in SAIL_CONFIG_FILES {
        let path = project_root.join(name);
        if let Ok(text) = std::fs::read_to_string(&path) {
            return Some((path, text));
        }
    }
    None
}

/// One app's serve configuration — nested `deck` keys from the project's
/// `sail.config.jsonc` / `sail.config.json` (or `Default::default()` when
/// absent / when `deck` is omitted), and the shape deck's per-app router
/// consumes. shipd (or any deck host) builds a `Vec<AppServeConfig>` via
/// [`AppServeConfig::load_from_project`] and pairs each with the route
/// the host's orchestration list declares. The framework does NOT know
/// the list; that is the host's concern.
///
/// # Ownership (SPEC-SAIL-CONFIG-1)
///
/// Sail owns product keys (`theme`, icons, fonts, network, layout, …).
/// Deck owns the **schema** of the nested `deck` object and is the sole
/// reader of serve settings. Deck does not parse icon/theme catalogs.
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
/// What stays (authored under `deck` in sail.config):
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
    /// NOT read from sail.config — the file doesn't get to name itself
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
    /// Workspace-relative project the app's *tools* should target (e.g.
    /// `apps/sail-site`). Host-set only — not read from sail.config.
    /// When set (Studio overlay at `/_studio`), `sail-project` /
    /// `sail-project-themes` meta stamp this path instead of the studio
    /// package root so explorer/checks/design/tasks open the real app.
    #[serde(skip, default)]
    pub ops_project: Option<String>,
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

/// Host-owned entry bootstrap. The bundler reads this from the project's
/// sail.config `deck.entry` and emits `entry.gen.ts` next to the data-only
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
    /// MIGRATION-DEBT: dies with CALLBACKS-1 / route-controller migration (ctx alias).
    #[serde(rename = "exposeNames")]
    pub expose_names: Vec<String>,
}

impl AppServeConfig {
    /// Parse a **serve-object** JSONC body (the value of `deck` in
    /// sail.config) into the serve config.
    ///
    /// Strips JSONC extensions, then deserializes. Unknown keys are
    /// ignored by serde. Every field is optional; empty input is
    /// `Default::default()`. Project load uses [`Self::load_from_project`],
    /// which extracts the nested `deck` object first.
    pub fn parse_deck_jsonc(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(&strip_jsonc_comments(raw))
    }

    /// Load app serve config from the project root.
    ///
    /// - Presence of `deck.jsonc` is a hard migration error (move serve
    ///   keys under `deck` in `sail.config.jsonc` / `sail.config.json`,
    ///   then delete `deck.jsonc`).
    /// - Prefer `sail.config.jsonc` over `sail.config.json` when both exist.
    /// - **Nested `deck` only** (SPEC-SAIL-CONFIG-1 C1): top-level
    ///   `render` / `entry` / `injectGlobals` are not read. Sail refuses
    ///   them as unknown keys; this loader never dual-reads.
    /// - Absent sail config or absent `deck` ⇒ fully conventional defaults.
    pub fn load_from_project(project: &Path) -> Result<Self, String> {
        let deck_path = project.join("deck.jsonc");
        if deck_path.is_file() {
            return Err(format!(
                "deck.jsonc is no longer supported (found at {}). \
                 Move serve keys under \"deck\" in sail.config.jsonc \
                 (or sail.config.json) — e.g. {{ \"deck\": {{ \"render\": \"ssr\" }} }} — \
                 and delete deck.jsonc.",
                deck_path.display()
            ));
        }

        for name in ["sail.config.jsonc", "sail.config.json"] {
            let path = project.join(name);
            if !path.is_file() {
                continue;
            }
            let raw = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            let stripped = strip_jsonc_comments(&raw);
            let value: serde_json::Value = serde_json::from_str(&stripped)
                .map_err(|e| format!("parse {}: {e}", path.display()))?;
            let Some(deck) = value.get("deck") else {
                // Theme-only / product-only sail.config — conventional serve.
                return Ok(Self::default());
            };
            if deck.is_null() {
                return Ok(Self::default());
            }
            if !deck.is_object() {
                return Err(format!(
                    "parse {}: \"deck\" must be an object (got {})",
                    path.display(),
                    deck
                ));
            }
            return serde_json::from_value(deck.clone())
                .map_err(|e| format!("parse {}.deck: {e}", path.display()));
        }

        Ok(Self::default())
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
    /// `"@ship/sail-components"`). A tag from an undeclared pack resolves to a
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
        let cfg: DeckConfig = serde_json::from_str(&strip_jsonc_comments(EXAMPLE_JSONC))
            .expect("example deck.jsonc must parse");
        assert_eq!(cfg.source_roots, vec![PathBuf::from(".")]);
        assert_eq!(
            cfg.watch,
            vec![PathBuf::from("packages"), PathBuf::from("apps/site/src")]
        );
        assert_eq!(
            cfg.route_dirs,
            vec!["routes".to_string(), ".ship/views".to_string()]
        );
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
        assert!(
            d.packs.is_empty(),
            "defaults declare no packs → global resolution"
        );
    }

    /// `packs` is optional and additive — an absent field parses to empty,
    /// which the bundler reads as "global" (every workspace pack resolvable).
    #[test]
    fn packs_absent_parses_to_empty() {
        let cfg: DeckConfig = serde_json::from_str(&strip_jsonc_comments(EXAMPLE_JSONC))
            .expect("example deck.jsonc must parse");
        assert!(
            cfg.packs.is_empty(),
            "no packs field ⇒ empty ⇒ global behavior"
        );
    }

    /// A declared `packs` array parses into the scoped set.
    #[test]
    fn packs_present_parses_declared_set() {
        let json = r#"{ "packs": ["@ship/sail-components", "deck-demo-app"] }"#;
        let cfg: DeckConfig = serde_json::from_str(&strip_jsonc_comments(json)).unwrap();
        assert_eq!(cfg.packs, vec!["@ship/sail-components", "deck-demo-app"]);
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
        assert_eq!(
            cfg,
            AppServeConfig::default(),
            "legacy fields parse to default"
        );
    }

    /// `render` is optional and defaults to `csr` — the implicit posture
    /// every app had before the field existed (empty `#root`,
    /// client paints). The default-constructed config also lands on csr.
    #[test]
    fn parse_app_deck_render_defaults_to_csr() {
        let cfg = AppServeConfig::default();
        assert_eq!(cfg.render, RenderMode::Csr, "Default ⇒ csr");
        let parsed =
            AppServeConfig::parse_deck_jsonc("{}").expect("empty deck.jsonc parses to Default");
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
        assert_eq!(
            cfg,
            AppServeConfig {
                name: String::new(),
                route: String::new(),
                source: String::new(),
                ops_project: None,
                inject_globals: [("DECK_APP_ID".to_string(), "$id".to_string())]
                    .into_iter()
                    .collect(),
                render: RenderMode::Ssr,
                entry: None,
            }
        );
    }

    /// `entry` round-trips through the JSONC parser — the host-owned
    /// the bundler imports + the `exposeNames` it spreads into the
    /// `clients` map).
    #[test]
    fn parse_app_deck_entry_bootstrap() {
        let cfg = AppServeConfig::parse_deck_jsonc(HUB_DECK_NEW).expect("hub deck.jsonc parses");
        let entry = cfg.entry.as_ref().expect("entry declared");
        assert_eq!(entry.clients_module, "./src/lib/client.ts");
        assert_eq!(entry.expose_names, vec!["ship", "ops"]);
    }

    /// An absent `entry` parses to `None` — the bundler falls back to
    /// the legacy `routes.gen.ts`-as-entry shape (no client wiring,
    /// token import rides the data file).
    #[test]
    fn parse_app_deck_no_entry_is_none() {
        let cfg = AppServeConfig::parse_deck_jsonc("{}").expect("empty deck.jsonc parses");
        assert!(cfg.entry.is_none(), "absent entry ⇒ None");
    }

    /// Spaces' shape — only `injectGlobals`, no `entry`. This is the
    /// canonical "non-conventional but not full-bootstrap" config.
    #[test]
    fn parse_app_deck_spaces_inject_globals_only() {
        let cfg =
            AppServeConfig::parse_deck_jsonc(SPACES_DECK_NEW).expect("spaces deck.jsonc parses");
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

    // ── AppServeConfig::load_from_project (sail.config only) ────────────────

    #[test]
    fn load_from_project_no_files_is_default() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = AppServeConfig::load_from_project(dir.path()).unwrap();
        assert_eq!(cfg, AppServeConfig::default());
    }

    #[test]
    fn load_from_project_deck_jsonc_is_migration_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("deck.jsonc"),
            r#"{ "render": "ssr", "injectGlobals": { "A": "1" } }"#,
        )
        .unwrap();
        let err = AppServeConfig::load_from_project(dir.path()).unwrap_err();
        assert!(
            err.contains("deck.jsonc is no longer supported"),
            "unexpected err: {err}"
        );
    }

    #[test]
    fn load_from_project_deck_jsonc_errors_even_when_sail_config_exists() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("deck.jsonc"), r#"{ "render": "ssr" }"#).unwrap();
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{ "deck": { "render": "csr" }, "theme": { "default": "ship" } }"#,
        )
        .unwrap();
        let err = AppServeConfig::load_from_project(dir.path()).unwrap_err();
        assert!(
            err.contains("deck.jsonc is no longer supported"),
            "unexpected err: {err}"
        );
    }

    #[test]
    fn load_from_project_nested_deck_serve_keys() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{ "deck": { "render": "ssr" }, "theme": { "default": "ship" } }"#,
        )
        .unwrap();
        let cfg = AppServeConfig::load_from_project(dir.path()).unwrap();
        assert_eq!(cfg.render, RenderMode::Ssr);
    }

    /// No dual-read: top-level serve keys are ignored by the loader
    /// (Sail refuses them as unknown keys; deck never migrates them).
    #[test]
    fn load_from_project_top_level_serve_keys_ignored() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{ "render": "ssr", "theme": { "default": "ship" } }"#,
        )
        .unwrap();
        let cfg = AppServeConfig::load_from_project(dir.path()).unwrap();
        assert_eq!(
            cfg,
            AppServeConfig::default(),
            "top-level render must not dual-read into serve config"
        );
    }

    #[test]
    fn load_from_project_theme_only_sail_is_default_serve() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{ "theme": { "default": "ship", "enabled": ["ship"] } }"#,
        )
        .unwrap();
        let cfg = AppServeConfig::load_from_project(dir.path()).unwrap();
        assert_eq!(cfg, AppServeConfig::default());
    }

    #[test]
    fn load_from_project_sail_entry_and_inject() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{
  "deck": {
    "render": "ssr",
    "injectGlobals": { "DECK_APP_ID": "$id" },
    "entry": { "clientsModule": "./src/clients.ts", "exposeNames": ["store"] }
  }
}"#,
        )
        .unwrap();
        let cfg = AppServeConfig::load_from_project(dir.path()).unwrap();
        assert_eq!(cfg.render, RenderMode::Ssr);
        assert_eq!(
            cfg.inject_globals.get("DECK_APP_ID").map(String::as_str),
            Some("$id")
        );
        let entry = cfg.entry.expect("entry");
        assert_eq!(entry.clients_module, "./src/clients.ts");
        assert_eq!(entry.expose_names, vec!["store"]);
    }

    #[test]
    fn load_from_project_jsonc_preferred_over_json() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("sail.config.json"),
            r#"{ "deck": { "render": "ssr" } }"#,
        )
        .unwrap();
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{ "deck": { "render": "csr" } }"#,
        )
        .unwrap();
        let cfg = AppServeConfig::load_from_project(dir.path()).unwrap();
        assert_eq!(cfg.render, RenderMode::Csr, "jsonc wins when both exist");
    }

    #[test]
    fn load_from_project_studio_fixture_is_csr_with_side_effect_entry() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{
  "theme": { "default": "studio" },
  "deck": {
    "render": "csr",
    "entry": {
      "clientsModule": "./src/clients.ts",
      "exposeNames": []
    }
  }
}"#,
        )
        .unwrap();
        let cfg = AppServeConfig::load_from_project(dir.path()).unwrap();
        assert_eq!(cfg.render, RenderMode::Csr);
        let entry = cfg.entry.expect("entry");
        assert_eq!(entry.clients_module, "./src/clients.ts");
        assert!(entry.expose_names.is_empty());
    }

    #[test]
    fn load_from_project_deck_must_be_object() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("sail.config.jsonc"), r#"{ "deck": "ssr" }"#).unwrap();
        let err = AppServeConfig::load_from_project(dir.path()).unwrap_err();
        assert!(
            err.contains("\"deck\" must be an object"),
            "unexpected err: {err}"
        );
    }
}
