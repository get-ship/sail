//! Manifest-driven **synthetic controller** emit (additive).
//!
//! Thesis: the `.sui` compiler can be *ignorant of* `route` (and, by
//! extension, `ship`, data sources, components). Those are **deck/app**
//! concepts, not SUI ones — SUI compiles `.sui` into TS; deck owns the
//! "view" feature (routing, the route handle, the view-host). Composition
//! happens through real ESM imports declared in an **app manifest** (a
//! name → module-specifier import map) and injected here, validated
//! downstream by tsgo/tsc.
//!
//! A `.sui` lowers into **two artifacts**:
//!  * a **synthetic controller** — the logic/data half: real ESM imports
//!    for manifested names, `<data>` directives lowered to controller
//!    statements, the authored `controller.ts` **composed** (imported,
//!    never rewritten), and a single **typed** `scope` export.
//!  * a **render** (see [`render`]) — the markup half: reuses the
//!    production [`super::template_emit::TemplateEmitter`] with a `scope.`
//!    author-expression prefix so all template features wire against the
//!    controller's typed `scope`.
//!
//! Nothing here consults `check::scope_bound`'s `BUILTINS` or the ambient
//! `emit/js/setup.rs` ctx handling. EVERY free name a `.sui` uses — `route`,
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

/// Look up a manifested name. **SUI names no host:** a name resolves ONLY
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
// AUTO-GENERATED by sui-compiler `synthesize` (manifest-driven synthetic
// controller). Do NOT edit. Regenerate:
//   sui-compiler synthesize <file> --manifest <manifest> --out-dir <dir>
//
// Each manifested name is a REAL ESM import injected by the app manifest —
// the compiler never reserved it (no BUILTINS, no ambient ctx). tsgo/tsc
// resolves member access against the imported type, so a typo like
// `route.parms.id` is a hard type error, not an ambient passthrough.";

/// Compute the set of manifest client names this `program` actually
/// references (template or `<data source>`). Per-view resolver: import only
/// what's used. Names are taken solely from the app manifest — there is no
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

/// Emit the synthetic controller + render for `program`. Pure
/// transformation. `controller_specifier` is the render's import path back
/// to the emitted controller (e.g. `"./page.synthetic.controller.ts"`).
/// `mode` selects view vs component lowering.
pub fn emit_synthetic(
    program: &Program,
    manifest: &AppManifest,
    authored: Option<&AuthoredController>,
    controller_specifier: &str,
    mode: CompileMode,
    component_tag: Option<&str>,
) -> Result<SyntheticOutput, CompileError> {
    // Per-view resolver: import only the manifest clients this template (or
    // a `<data source>`) actually references — mirrors how the production
    // emitter imports only the tags a view uses. Names come solely from the
    // manifest (see [`used_clients`]); the compiler names no host provider.
    let used = used_clients(program, manifest);


    let controller_ts = emit_controller(program, manifest, &used, authored, mode)?;
    let render_ts = render::emit_render(program, controller_specifier, component_tag)?;
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
    // through the same path as views.
    if mode == CompileMode::Component && !program.props.is_empty() {
        return Err(CompileError::new(
            "synthesize: component `<prop>` declarations are not supported yet — \
             props flow in at mount, which the standalone synthetic controller \
             can't model. (Phase 2: a props-parameterized scope factory.)",
        )
        .with_category("synthesize"));
    }

    let mut w = Writer::new();
    w.line(HEADER);

    // Real ESM imports for the manifested names. Convention-backed names
    // (`route`/`navigate`) resolve through the same path when the
    // manifest is silent — see [`resolve_client`].
    for name in used {
        let Some(c) = resolve_client(manifest, name) else {
            // An unknown used name shouldn't reach here — `used_clients`
            // only emits names that are either in the manifest or in the
            // convention table. Defensive: skip rather than panic so a
            // future caller doesn't break the emit.
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
    // `<data>` directives lower to runtime statements: `live()` for the
    // reactive form, `data({initial, source})` for the non-live form —
    // matching the production `emit_data_decls` shapes exactly.
    let uses_live = program.data.iter().any(|d| d.live);
    let uses_data = program.data.iter().any(|d| !d.live);
    let mut runtime_imports: Vec<&str> = Vec::new();
    if uses_live {
        runtime_imports.push("live");
    }
    if uses_data {
        runtime_imports.push("data");
    }
    if !runtime_imports.is_empty() {
        w.line(&format!(
            "import {{ {} }} from \"@ship/sui-runtime/internal\";",
            runtime_imports.join(", ")
        ));
    }
    // Compose the authored controller — import it, never rewrite it. The
    // authored module default-exports a real `controller({ scope, derived,
    // on })` (`ControllerExport`); the synthetic controller drives its
    // `setup(ctx)` (below). The type imports recover its contributed scope
    // type and shape the resolver context.
    if let Some(a) = authored {
        w.line(&format!("import __authored from {};", js_string(&a.specifier)));
        w.line("import type { ControllerExport, ResolverContext } from \"@ship/sui-runtime/internal\";");
    }
    w.nl();

    if used.iter().any(|n| resolve_client(manifest, n).is_some_and(|c| c.construct)) {
        w.line("// Manifest value-clients, constructed once per view.");
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
    if !program.data.is_empty() {
        w.line("// `<data>` directives, lowered to controller statements.");
        for d in &program.data {
            // The source references bare manifest clients (the consts
            // above), exactly as authored — no `scope.` qualification in
            // the controller.
            if d.live {
                w.line(&format!(
                    "const {name} = live(() => ({src}));",
                    name = d.name,
                    src = d.source.source
                ));
            } else {
                w.line(&format!(
                    "const {name} = data({{ initial: undefined, source: () => ({src}) }});",
                    name = d.name,
                    src = d.source.source
                ));
            }
        }
        w.nl();
    }

    // The single typed scope. Member order: manifest clients, then
    // `<data>`, then the spread of the composed authored scope. `tsgo`/
    // `tsc` type-checks the render against THIS type.
    //
    // G5 — `{#async … as v}` lifts the `as v` binding into the typed
    // scope so the render can reference `scope.v` after the Let
    // snapshot. The actual value's TS type is unreachable from the
    // controller (it lives in the template, not the controller DSL),
    // so each Let name is typed `any` — the runtime snapshot is the
    // source of truth, the static type-check is a best-effort
    // declaration. The render's `emit_let_block` writes `$scope.v`
    // at mount, so the runtime lookup matches.
    w.line("// The view's typed scope: every name a template expression resolves,");
    w.line("// merged from manifest imports, `<data>` directives, and the composed");
    w.line("// controller. `tsgo`/`tsc` type-checks the template against THIS type.");
    let mut members: Vec<String> = used.to_vec();
    for d in &program.data {
        members.push(d.name.clone());
    }
    if authored.is_some() {
        members.push("...__authoredScope".to_string());
    }
    let let_names = collect_let_names(&program.template);
    for name in &let_names {
        members.push(format!("{name}: any"));
    }
    w.line(&format!("export const scope = {{ {} }};", members.join(", ")));
    w.line("export type Scope = typeof scope;");
    Ok(w.into_string())
}

/// Walk the template's `Instr::Let` instructions and collect the `as v`
/// binding names. G5 — each becomes a typed scope member so the
/// render can read `scope.<name>` after the Let snapshot.
fn collect_let_names(instrs: &[Instr]) -> Vec<String> {
    let mut out = Vec::new();
    for instr in instrs {
        match instr {
            Instr::Let(l) => {
                out.push(l.name.clone());
                out.extend(collect_let_names(&l.body));
            }
            Instr::Element(el) => out.extend(collect_let_names(&el.children)),
            Instr::If(i) => {
                for b in &i.branches {
                    out.extend(collect_let_names(&b.body));
                }
                if let Some(eb) = &i.else_body {
                    out.extend(collect_let_names(eb));
                }
            }
            Instr::For(f) => {
                out.extend(collect_let_names(&f.body));
                if let Some(eb) = &f.else_body {
                    out.extend(collect_let_names(eb));
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    out.extend(collect_let_names(&c.body));
                }
                if let Some(d) = &m.default {
                    out.extend(collect_let_names(d));
                }
            }
            Instr::WhileLoading(wl) => out.extend(collect_let_names(&wl.body)),
            Instr::WhileError(we) => out.extend(collect_let_names(&we.body)),
            _ => {}
        }
    }
    out
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
    w.line("// Authored controller — COMPOSED via its real `controller({...})` DSL:");
    w.line("// `setup(ctx)` runs `resolveController` (the same resolver the browser");
    w.line("// hydrate path runs) and returns the flat bindings. The scope's");
    w.line("// contributed type is recovered from the controller's `__def.setup`");
    w.line("// factory return (not the legacy `ControllerExport<infer S>` generic");
    w.line("// — the generic is often pinned to the empty view `Scope` and would");
    w.line("// erase factory-added members from the recovered type).");
    // Use `infer ReturnType<...>` so a non-function `setup` (a
    // `controller<Scope>({...})` with no `setup:` factory) falls back to
    // `object` — matches the controller's contribution of nothing.
    w.line(
        "type __AuthoredScope = typeof __authored.__def extends { setup?: infer F }",
    );
    // Match the factory regardless of its parameter list. `ScopedControllerDef`
    // types `setup` as `(init: Ctx, host?) => SC` — a REQUIRED `init` param — so
    // the zero-arg pattern `() => infer R` failed to match (a fn requiring args
    // isn't assignable to `() => R`), collapsing the recovered scope to `object`
    // and false-erroring EVERY `setup.X` read. `(...a: never[]) => infer R`
    // (the `ReturnType` idiom) matches any factory and recovers SC precisely.
    w.line("  ? F extends (...a: never[]) => infer R ? R : object");
    w.line("  : object;");
    // Map the constructed manifest clients onto the resolver context: the
    // well-known ambient names become `ResolverContext` fields; the rest go
    // into `clients` (spread into scope by the resolver, as in production).
    let mut ship = "undefined as never".to_string();
    let mut route: Option<&str> = None;
    let mut navigate: Option<&str> = None;
    let mut client_entries: Vec<&str> = Vec::new();
    for name in used {
        // `navigate` is a plain value-import (construct:false); ship/route are
        // constructed handles. Either way the const is named `<name>`.
        match name.as_str() {
            "ship" => ship = "ship".to_string(),
            "route" => route = Some("route"),
            "navigate" => navigate = Some("navigate"),
            other => client_entries.push(other),
        }
    }
    let _ = manifest; // naming map is the manifest's contract; values are the consts above.
    w.line("const __ctx = {");
    w.line(&format!("  ship: {ship},"));
    w.line(&format!("  clients: {{ {} }},", client_entries.join(", ")));
    if let Some(r) = route {
        w.line(&format!("  route: {r},"));
    }
    if let Some(n) = navigate {
        w.line(&format!("  navigate: {n},"));
    }
    w.line("} as unknown as ResolverContext;");
    w.line("const __authoredScope = __authored.setup(__ctx) as __AuthoredScope;");
    w.nl();
}
