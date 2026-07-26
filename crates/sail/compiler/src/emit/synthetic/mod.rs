//! Manifest-driven **synthetic controller** emit (additive).
//!
//! Thesis: the `.sail` compiler can be *ignorant of* `route` (and, by
//! extension, `ship`, data sources, components). Those are **deck/app**
//! concepts, not SAIL ones — SAIL compiles `.sail` into TS; deck owns the
//! "route" feature (routing, the route handle, the route-host). Composition
//! happens through real ESM imports declared in an **app manifest** (a
//! name → module-specifier import map) and injected here, validated
//! downstream by tsgo/tsc.
//!
//! A `.sail` lowers into **two artifacts**:
//!  * a **synthetic controller** — the logic/data half: real ESM imports
//!    for manifested names, the authored `controller.ts` **composed**
//!    (imported, never rewritten), and a single **typed** `scope` export.
//!  * a **render** (see [`render`]) — the markup half: reuses the
//!    production [`super::template_emit::TemplateEmitter`] with a `scope.`
//!    author-expression prefix so all template features wire against the
//!    controller's typed `scope`.
//!
//! Nothing here consults `check::scope_bound`'s `BUILTINS` or the ambient
//! `emit/js/setup.rs` ctx handling. EVERY free name a `.sail` uses — `route`,
//! `navigate`, `ship`, data sources — resolves through the app manifest; the
//! compiler names no provider (no `@ship/deck/router` hardcode, no
//! convention fallback). The host declares its context into the manifest seam.
mod merge;
pub mod render;
mod scan;

pub use merge::emit_synthetic_client_module;

use std::collections::BTreeMap;

use serde::Deserialize;

use super::writer::{js_string, Writer};
use crate::diagnostics::CompileError;
use crate::ir::*;
use crate::CompileMode;

/// One app-manifest entry: a free name → the real ESM import that provides
/// it. `construct` distinguishes a value bound directly from the import
/// (`import { x }`) from a handle constructed from an imported factory
/// (`const x = makeX()`). `route` is the latter — its export is the
/// `route()` factory, not a `RouteSignals` instance.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestClient {
    /// Module specifier, e.g. `@ship/deck/router`.
    pub from: String,
    /// Named export to import, e.g. `route`.
    pub import: String,
    /// When true, the import is a factory: emit `const <name> = <import>()`.
    #[serde(default)]
    pub construct: bool,
}

/// The app-manifest: a name → provider import map. Co-located with an app
/// as `app.manifest.jsonc`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppManifest {
    #[serde(default)]
    pub clients: BTreeMap<String, ManifestClient>,
}

/// Look up a manifested name. **SAIL names no host:** a name resolves ONLY
/// through the app manifest. The host (deck) declares its own context —
/// `route`, `navigate`, and anything else — into the manifest seam; the
/// compiler stays ignorant of any specific provider (no `@ship/deck/router`
/// hardcode, no convention fallback). An unmanifested free name simply isn't a
/// client and falls through to the typed scope (a typo is a hard type error
/// downstream, not an ambient passthrough).
fn resolve_client(manifest: &AppManifest, name: &str) -> Option<ManifestClient> {
    manifest.clients.get(name).cloned()
}

impl AppManifest {
    /// Parse a manifest string. Pure — no file IO (the compiler is a pure
    /// transformation; the caller reads the file and hands us the bytes).
    /// Accepts JSON with whole-line `//` comments (JSONC-lite) so the
    /// checked-in manifest can document itself.
    pub fn parse(src: &str) -> Result<Self, CompileError> {
        let stripped = strip_line_comments(src);
        serde_json::from_str(&stripped)
            .map_err(|e| CompileError::new(format!("app manifest: {e}")).with_category("manifest"))
    }
}

/// An authored `controller.ts` to **compose** (import + merge), identified
/// by the import specifier the synthetic controller should use. Convention:
/// the authored module default-exports a `() => object` scope factory.
#[derive(Debug, Clone)]
pub struct AuthoredController {
    pub specifier: String,
}

/// Drop whole-line `//` comments (after optional leading whitespace). Keeps
/// the parser dependency-free and string-safe — only full comment lines are
/// removed, never `//` inside a JSON string value.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The emitted artifacts plus the manifest clients actually used.
#[derive(Debug, Clone)]
pub struct SyntheticOutput {
    /// The synthetic controller module (logic/data half).
    pub controller_ts: String,
    /// The SSR render module (markup half, mount-only), importing the
    /// controller's `scope`. This is what ship-js SSR runs.
    pub render_ts: String,
    /// The CLIENT render module: `export default component(() => …)` whose setup
    /// returns the `__mount` + `hydrate` contract `mount.ts` drives. Same
    /// mount body as `render_ts`, plus the production hydrate buffers. Empty
    /// of nothing — always emitted (additive; the SSR path ignores it).
    pub render_client_ts: String,
    /// Manifest clients the template referenced (the subset imported).
    pub used_clients: Vec<String>,
}

/// Stable header — deliberately omits the volatile `_compilerVersion`
/// stamp so output is deterministic and golden-testable.
const HEADER: &str = "\
// AUTO-GENERATED by sail-compiler `synthesize` (manifest-driven synthetic
// controller). Do NOT edit. Regenerate:
//   sail-compiler synthesize <file> --manifest <manifest> --out-dir <dir>
//
// Each manifested name is a REAL ESM import injected by the app manifest —
// the compiler never reserved it (no BUILTINS, no ambient ctx). tsgo/tsc
// resolves member access against the imported type, so a typo like
// `route.parms.id` is a hard type error, not an ambient passthrough.";

/// Compute the set of manifest client names this `program`'s template
/// actually references. Per-route resolver: import only what's used. Names
/// are taken solely from the app manifest — there is no
/// host convention; the host declares `route`/`navigate`/etc. into the
/// manifest itself. Manifest key order (a `BTreeMap`) keeps the import block
/// byte-stable for the same inputs.
pub(crate) fn used_clients(program: &Program, manifest: &AppManifest) -> Vec<String> {
    let mut used: Vec<String> = Vec::new();
    for name in manifest.clients.keys() {
        if scan::references_ident(program, name) {
            used.push(name.clone());
        }
    }
    used
}

/// The manifest's clients as SCOPE CAPABILITIES — the typed members
/// `emit_dts` writes into the generated `Scope`. Every declared client is typed
/// through the SAIL-local `Host` port (`Host['api']`), which is the only
/// type SAIL owns here; a host whose injected context carries a different type
/// (deck's `route`, say) overrides its own entries after this call. The
/// compiler still names no provider.
pub fn scope_capabilities(manifest: &AppManifest) -> Vec<crate::emit::dts::ScopeCapability> {
    manifest.clients.keys().map(crate::emit::dts::ScopeCapability::client).collect()
}

/// Emit the synthetic controller + render for `program`. Pure
/// transformation. `controller_specifier` is the render's import path back
/// to the emitted controller (e.g. `"./page.synthetic.controller.ts"`).
/// `mode` selects route vs component lowering.
pub fn emit_synthetic(
    program: &Program,
    manifest: &AppManifest,
    authored: Option<&AuthoredController>,
    controller_specifier: &str,
    mode: CompileMode,
    component_tag: Option<&str>,
) -> Result<SyntheticOutput, CompileError> {
    // Per-route resolver: import only the manifest clients this template
    // actually references — mirrors how the production emitter imports only
    // the tags a route uses. Names come solely from the manifest (see
    // [`used_clients`]); the compiler names no host provider.
    let used = used_clients(program, manifest);


    let controller_ts = emit_controller(program, manifest, &used, authored, mode)?;
    let render_ts =
        render::emit_render(program, controller_specifier, component_tag, authored.is_some())?;
    let render_client_ts =
        render::emit_render_client(program, controller_specifier, component_tag)?;
    Ok(SyntheticOutput { controller_ts, render_ts, render_client_ts, used_clients: used })
}

pub(super) fn emit_controller(
    program: &Program,
    manifest: &AppManifest,
    used: &[String],
    authored: Option<&AuthoredController>,
    mode: CompileMode,
) -> Result<String, CompileError> {
    // Component props would need to flow into the typed scope at mount; the
    // synthetic controller is a standalone `export const scope`, so propful
    // components are a Phase-2 concern. Propless components (state-only,
    // composed from an authored controller, or pure markup) synthesize
    // through the same path as routes.
    if mode == CompileMode::Component && !program.props.is_empty() {
        return Err(CompileError::new(
            "synthesize: component `<scope>` prop declarations are not supported yet — \
             props flow in at mount, which the standalone synthetic controller \
             can't model. (Phase 2: a props-parameterized scope factory.)",
        )
        .with_category("synthesize"));
    }

    let mut w = Writer::new();
    w.line(HEADER);

    // Real ESM imports for the manifested names. EVERY name resolves through
    // the manifest and nothing else — `route`/`navigate` included; a host that
    // wants them injects them into the manifest itself (see [`resolve_client`]).
    for name in used {
        let Some(c) = resolve_client(manifest, name) else {
            // An unknown used name shouldn't reach here — `used_clients`
            // only emits names it read out of the manifest. Defensive: skip
            // rather than panic so a future caller doesn't break the emit.
            continue;
        };
        if c.construct {
            w.line(&format!(
                "import {{ {imp} as __make_{name} }} from {from};",
                imp = c.import,
                from = js_string(&c.from),
            ));
        } else if c.import == *name {
            w.line(&format!(
                "import {{ {imp} }} from {from};",
                imp = c.import,
                from = js_string(&c.from)
            ));
        } else {
            w.line(&format!(
                "import {{ {imp} as {name} }} from {from};",
                imp = c.import,
                from = js_string(&c.from),
            ));
        }
    }
    // Compose the authored controller — import it, never rewrite it. The
    // authored module default-exports a real `controller({ scope, derived,
    // on })` (`ControllerExport`); the synthetic controller drives its
    // `setup(ctx)` (below). The type imports recover its contributed scope
    // type and shape the resolver context.
    if let Some(a) = authored {
        w.line(&format!("import __authored from {};", js_string(&a.specifier)));
        w.line("import type { ControllerExport, ResolverContext } from \"@ship/sail-runtime/internal\";");
        // `createScope`/`withScope` give `__authored.setup(...)` a real setup
        // window — a controller that calls the ambient `mount(fn)` (see
        // `packages/sail/runtime/src/effect-scope.ts`) throws
        // `SuiMountOutsideSetupError` with no current scope, and the browser
        // hydrate path always runs setup inside one (`mount.ts`, `compose.ts`).
        // The synthetic render is a one-shot SSR pass with no "connect" event,
        // so `mount()` callbacks registered here are simply never fired —
        // scope.dispose() drops them along with anything an authored
        // controller registered via `onCleanup` — but the CALL must not throw.
        w.line("import { createScope, withScope } from \"@ship/sail-runtime/internal\";");
    }
    w.nl();

    if used.iter().any(|n| resolve_client(manifest, n).is_some_and(|c| c.construct)) {
        w.line("// Manifest value-clients, constructed once per route.");
        for name in used {
            if resolve_client(manifest, name).is_some_and(|c| c.construct) {
                w.line(&format!("const {name} = __make_{name}();"));
            }
        }
        w.nl();
    }
    if authored.is_some() {
        emit_authored_compose(&mut w, used, manifest);
    }

    // The single typed scope. Member order: manifest clients, then the
    // spread of the composed authored scope. `tsgo`/`tsc` type-checks the
    // render against THIS type.
    w.line("// The module's typed scope: every name a template expression resolves,");
    w.line("// merged from manifest imports and the composed controller.");
    w.line("// `tsgo`/`tsc` type-checks the template against THIS type.");
    let mut members: Vec<String> = used.to_vec();
    if authored.is_some() {
        members.push("...__authoredScope".to_string());
    }
    w.line(&format!("export const scope = {{ {} }};", members.join(", ")));
    w.line("export type Scope = typeof scope;");
    Ok(w.into_string())
}

/// Compose an authored `controller({ scope, derived, on })` (the production
/// DSL) into the synthetic controller's scope.
///
/// The authored module default-exports a `ControllerExport` (what
/// `controller(def)` returns). We run its `setup(ctx)` — the SAME
/// `resolveController` the browser hydrate path runs — so the authored
/// logic is COMPOSED, never rewritten: signals from the `setup:` factory,
/// memoized `derived`, and pre-bound `on` handlers all resolve exactly as in
/// production. `setup` returns flat `ResolvedBindings` (erased to
/// `Record<string, unknown>`), so we recover the controller's contributed
/// scope TYPE from its `ControllerExport<Scope>` parameter and cast — that
/// recovered type is what the render type-checks its `scope.X` reads against.
///
/// `ctx` mirrors what production's hydrate flow hands the resolver: the
/// well-known ambient names (`ship`, `route`, `navigate`) map to the
/// `ResolverContext` fields; every other manifest client is spread into
/// `clients`. The values are the real constructed clients (see the
/// value-client block above); only the *shape* is cast across the
/// app-manifest-naming → platform-`ResolverContext` seam.
fn emit_authored_compose(w: &mut Writer, used: &[String], manifest: &AppManifest) {
    w.line("// Authored controller — COMPOSED via its real `controller(fn)` DSL:");
    w.line("// `setup(ctx)` runs the SAME resolver the browser hydrate path runs");
    w.line("// (calls `fn` with the ambient scope) and returns fn's result —");
    w.line("// the flat bindings — verbatim. The scope's contributed type is");
    w.line("// recovered from the controller's `__def` function return (not the");
    w.line("// legacy `ControllerExport<infer S>` generic — the generic is often");
    w.line("// pinned to the empty `Scope` and would erase setup-added");
    w.line("// members from the recovered type).");
    // `__def` IS the setup function (the object-form's `{ setup?: F }` wrapper
    // is gone with the sections it wrapped) — `ReturnType`-idiom infer over
    // `(...a: never[]) => infer R` matches any function regardless of its
    // param list (a fn requiring args isn't assignable to a zero-arg `() => R`,
    // which is why this isn't a plain `ReturnType<typeof __authored.__def>`).
    w.line("type __AuthoredScope = typeof __authored.__def extends (...a: never[]) => infer R ? R : object;");
    // Map the constructed manifest clients onto the resolver context: the
    // well-known ambient names become `ResolverContext` fields; the rest go
    // into `clients` (spread into scope by the resolver, as in production).
    let mut route: Option<&str> = None;
    let mut navigate: Option<&str> = None;
    let mut provider_entries: Vec<&str> = Vec::new();
    for name in used {
        // `navigate` is a plain value-import (construct:false); route is a
        // constructed handle. Either way the const is named `<name>`.
        //
        // #105 — `ship` is NOT special-cased any more. It used to be lifted
        // onto its own `ResolverContext.ship` field, which is what made a Ship
        // host a framework concept. It is now an ORDINARY manifest client: it
        // rides `clients` like `api` does, and the resolver spreads the map
        // into scope, so a template reading `ship.demo.todos()` resolves
        // exactly as before. Nothing about an app's manifest changes.
        match name.as_str() {
            "route" => route = Some("route"),
            "navigate" => navigate = Some("navigate"),
            other => provider_entries.push(other),
        }
    }
    let _ = manifest; // naming map is the manifest's contract; values are the consts above.
    w.line("const __ctx = {");
    for p in &provider_entries {
        w.line(&format!("  {p},"));
    }
    if let Some(r) = route {
        w.line(&format!("  route: {r},"));
    }
    if let Some(n) = navigate {
        w.line(&format!("  navigate: {n},"));
    }
    w.line("} as unknown as ResolverContext;");
    // Run inside a scope: the controller's setup may call the ambient
    // `mount(fn)`/`onCleanup(fn)` (see the import comment above) — without a
    // current scope `mount()` throws `SuiMountOutsideSetupError`. The scope
    // is NOT disposed here: a `query()`/`nativeQuery()` source homes its
    // cache on this scope as owner-tree root (`useNativeCache()`), and its
    // in-flight PULL fetch must survive past `setup()` returning — disposing
    // synchronously here would mark the cache entry `disposed` before the
    // host-dispatch promise settles, discarding the resolved value and
    // leaving first paint stuck on `loading` regardless of how long the SSR
    // entry waits (see native-cache.ts's `disposed` guard in `fetch()`).
    // Disposal is exported instead — `render.ts`'s `mount()` disposer calls
    // it only after SSR has flushed pending queries and serialized first
    // paint (see `render.rs`'s `emit_render`), so this scope's lifetime
    // spans the whole one-shot render, not just `setup()`.
    w.line("const __authoredSetupScope = createScope();");
    w.line(
        "const __authoredScope = withScope(__authoredSetupScope, () => __authored.setup(__ctx)) as __AuthoredScope;",
    );
    w.line("export const __disposeAuthoredSetup = (): void => { __authoredSetupScope.dispose(); };");
    w.nl();
}
