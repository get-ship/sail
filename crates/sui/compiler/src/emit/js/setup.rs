use super::super::template_emit::TemplateEmitter;
use super::super::util::{camel_of, js_ident};
use super::super::writer::{js_string, Writer};
use crate::ir::*;
use crate::keys::BindingRegistry;
use crate::lower::Lowered;
use crate::CompileOpts;
use std::collections::BTreeSet;

/// Returns the MODULE-scope hoists (the `__tpl(...)` cloners). They cannot live in
/// the setup body: a template parsed there would be re-parsed for every component
/// instance, when the whole point is to parse it once per program.
pub(super) fn emit_setup_body(
    w: &mut Writer,
    program: &Program,
    bindings: &BindingRegistry,
    opts: &CompileOpts,
    _lowered: &Lowered,
) -> Writer {
    // A source is a COMPONENT (custom element) iff it carries a tag
    // (`component_meta`); otherwise it's a tagless mount-form ("view"/page).
    // This is the sole view/component distinction now that `CompileMode::View`
    // is gone — keyed on the tag, not a mode flag.
    let is_component = opts.component_meta.is_some();
    let is_shadow = opts.component_meta.as_ref().map(|m| m.shadow).unwrap_or(false);
    // A source that DECLARES `<client name="ship" />` resolves `ship` from
    // `ctx.clients.ship` (falling back to the `ctx.ship` alias); undeclared
    // `ship` stays the ambient ctx default.
    let ship_via_clients = program.clients.iter().any(|c| c.name == "ship");

    // ── ctx → $scope seeding (unified) ──────────────────────────────────────
    // Every compiled `.sui` is a component module: `setup(ctx)` seeds the scope
    // and returns `{ …$scope, __mount(container), [hydrate] }`. ship/route/
    // navigate always come from ctx (deck injects them for pages; `undefined`
    // for a registered component, which never reads them). Routing is deck's —
    // SUI vends no router or factory fallback. Props come from `ctx.props`
    // (registered components have them, pages don't).
    if ship_via_clients {
        w.line("const { route, navigate } = ctx;");
        w.line("const ship = (ctx.clients ?? {}).ship ?? ctx.ship;");
    } else {
        w.line("const { ship, route, navigate } = ctx;");
    }
    w.line("const $scope = Object.create(null);");
    w.line("$scope.ship = ship;");
    w.line("$scope.route = route;");
    w.line("$scope.navigate = navigate;");
    // Declared `<client>` names → seed from `ctx.clients` so `<data
    // source="X.op()">` + inline handlers resolve bare `X` (host-injected).
    for client in &program.clients {
        let name = client.name.as_str();
        if name == "ship" || name == "route" || name == "navigate" {
            continue;
        }
        w.line(&format!("const {{ {name} }} = ctx.clients ?? {{}};"));
        w.line(&format!("$scope.{name} = {name};"));
    }
    // Props (registered components) → seed from `ctx.props` (the prop signals
    // the custom-element adapter passes). Pages declare none.
    if !program.props.is_empty() {
        w.line("const __props = ctx.props ?? {};");
        for p in &program.props {
            let ident = js_ident(&camel_of(&p.name));
            w.line(&format!("$scope.{ident} = __props.{ident};"));
        }
    }
    emit_data_decls(w, program);
    // Pre-seed `$scope.<var> = signal(null)` for every `bind:this={var}`
    // target the template references — including ones inside conditional /
    // loop branches that won't render until later. The controller's
    // scope: factory may overwrite specific keys with an author-declared
    // `ref(null)`; either way, `scope.<var>()` is always callable at
    // setup time, so a controller mount() reading `scope.headerEl()`
    // before the `{#if open()}` branch first renders sees `null`
    // instead of crashing with `headerEl is not a function`.
    emit_ref_slot_seeds(w, &program.template);
    if opts.setup_url.is_some() {
        // Expose `$scope` to the controller (`ctx.scope`) and merge its return.
        // The LIFECYCLE_KEY bundle flows through the setup return to
        // `mountComponent` — the single engine that wires mount()/unmount() —
        // so there is no inline lifecycle wiring here anymore.
        emit_setup_call(w);
    }

    // ── deferred render: `__mount(container)` + optional `hydrate(root, ctx)` ─
    // One shape for both forms. The custom-element adapter calls `__mount(host)`
    // on connect (`__root = host.shadowRoot ?? host`); a page / internal child
    // calls it with its container. Always view-shaped so the hydrate buffers
    // populate (harmless for CSR-island components — `mountComponent` ignores
    // `hydrate`). The controller lifecycle is wired by `mountComponent` from the
    // LIFECYCLE_KEY that rides the return, not inline here. Hoists drain into
    // `w` before the body so the `__wireBoundary_b<N>` consts are in scope.
    let mut emitter = TemplateEmitter::new(bindings, true);
    let mut mount_w = Writer::new();
    mount_w.line("const __cleanups = [];");
    mount_w.line("const __root = container.shadowRoot ?? container;");
    if is_shadow {
        mount_w.line("adoptSuiSheets(__root);");
    }
    // Pages install their lifted top-level `<style>` into the container.
    // Registered components install styles via the custom-element adapter
    // (scoped by `data-component` / the shadow root), so the module skips it.
    if !is_component {
        let styles_install = crate::emit::styles::styles_install_js(&program.styles, false);
        if !styles_install.is_empty() {
            mount_w.push_str(&styles_install);
        }
    }
    emitter.emit_top_level(
        &mut mount_w,
        &program.template,
        "__root",
        "__cleanups",
        is_component && !is_shadow,
    );
    mount_w.line("return () => { for (const c of __cleanups) try { c(); } catch (e) {} };");

    let module_hoists = emitter.take_module_hoists();
    let hoists = emitter.take_boundary_hoists();
    let callbacks = emitter.take_render_callbacks();
    let hydrate_uses_text = emitter.hydrate_uses_text();
    let hydrate_uses_markers = emitter.hydrate_uses_markers();
    let hydrate_calls = emitter.take_hydrate_calls();

    if !hoists.is_empty() {
        w.push_str(&hoists);
    }
    w.line("return Object.assign({}, $scope, {");
    w.enter();
    w.line("__mount(container) {");
    w.enter();
    w.push_str(&mount_w.into_string());
    w.leave();
    w.line("},");
    if !hydrate_calls.is_empty() {
        emit_view_hydrate(w, &hydrate_calls, hydrate_uses_text, hydrate_uses_markers);
    }
    if !callbacks.is_empty() {
        w.push_str(&callbacks);
    }
    w.leave();
    w.line("});");
    module_hoists
}

/// Emit the `hydrate(root, ctx)` named property on a view's setup
/// return. Wires every binding kind the compiler now direct-emits
/// (Phase 3b.3+) — runs after `wireBoundaries` so SSR'd markers inside
/// boundary subtrees are already detached and won't be matched.
fn emit_view_hydrate(
    w: &mut Writer,
    hydrate_calls: &str,
    uses_text: bool,
    uses_markers: bool,
) {
    w.line("hydrate(root, ctx) {");
    w.enter();
    w.line("const __cleanups = [];");
    if uses_text {
        w.line("const __t = collectTextOpens(root);");
    }
    if uses_markers {
        w.line("const __m = collectElementMarkers(root);");
    }
    w.push_str(hydrate_calls);
    w.line("return () => { for (const c of __cleanups) try { c(); } catch (e) {} };");
    w.leave();
    w.line("},");
}

fn emit_data_decls(w: &mut Writer, program: &Program) {
    // The view's setup closure already destructures `ship`, `route`,
    // `navigate` from `ctx` into its lexical scope, so the data thunk
    // closes over them instead of taking them as args. The earlier
    // form `({ ship, route }) => …` looked like it would receive a
    // context arg, but `live()` / `data().source()` don't pass one —
    // every `<data live>` resolved its source against `undefined` and
    // got stuck on `loading` because `ship.git.overview()` threw a
    // synchronous TypeError that sat inside the `signal` reset on
    // the next effect tick.
    for d in &program.data {
        let src = &d.source.source;
        if d.live {
            w.line(&format!(
                "$scope.{name} = live(() => {src});",
                name = js_ident(&d.name),
                src = src
            ));
        } else {
            w.line(&format!(
                "$scope.{name} = data({{ initial: undefined, source: () => {src} }});",
                name = js_ident(&d.name),
                src = src
            ));
        }
    }
}

fn emit_ref_slot_seeds(w: &mut Writer, template: &[Instr]) {
    let mut names: BTreeSet<String> = BTreeSet::new();
    collect_bind_this_targets(template, &mut names);
    for name in &names {
        w.line(&format!(
            "$scope[{n}] = signal(null);",
            n = js_string(name),
        ));
    }
}

fn collect_bind_this_targets(instrs: &[Instr], out: &mut BTreeSet<String>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                for a in &el.attrs {
                    if let AttrInstr::BindThis { var, .. } = a {
                        out.insert(var.clone());
                    }
                }
                collect_bind_this_targets(&el.children, out);
            }
            Instr::If(if_instr) => {
                for b in &if_instr.branches {
                    collect_bind_this_targets(&b.body, out);
                }
                if let Some(eb) = &if_instr.else_body {
                    collect_bind_this_targets(eb, out);
                }
            }
            Instr::For(f) => {
                collect_bind_this_targets(&f.body, out);
                if let Some(eb) = &f.else_body {
                    collect_bind_this_targets(eb, out);
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    collect_bind_this_targets(&c.body, out);
                }
                if let Some(d) = &m.default {
                    collect_bind_this_targets(d, out);
                }
            }
            Instr::WhileLoading(wl) => collect_bind_this_targets(&wl.body, out),
            Instr::WhileError(we) => collect_bind_this_targets(&we.body, out),
            _ => {}
        }
    }
}

fn emit_setup_call(w: &mut Writer) {
    // Author-facing paired-controller shape is `controller(def)` (and
    // `component(setup)` for the mount-form) — both wrap into `{ setup }`. No raw
    // `function setup(ctx)` author export. Compiler emits a single call
    // path; if a paired `.ts` returns a non-object, we ignore it instead
    // of letting it shadow the runtime's $scope.
    //
    // Expose `$scope` to the controller via `ctx.scope` BEFORE invoking
    // its setup. The template's `$scope.<data>` is the live signal
    // emitted by `emit_data_decls` (e.g. `$scope.agents = live(() =>
    // ship.agents.summaries())`) — that's the signal the template's
    // bindings (`bindWhileLoading`, `bindIf` over `$scope.agents()`)
    // actually read. If the controller's `derived` section destructures
    // the same name out of `ctx.scope`, it must resolve to that same
    // signal, otherwise the controller computes against the inert
    // mount-built copy (stuck at `{kind:'loading'}`) while the template
    // sees the patched live signal. Mount.ts hands us `ctx.scope` as a
    // snapshot of the SSR-built scope; we narrow it to `$scope` here so
    // controller-and-template agree on a single source-of-truth signal
    // per `<data>` declaration. Components already did this via
    // `emit_component_setup_call`; views were lagging.
    w.line("if (setup && typeof setup.setup === \"function\") {");
    w.enter();
    w.line("ctx.scope = $scope;");
    w.line("const __r = setup.setup(ctx);");
    w.line("if (__r && typeof __r === \"object\") Object.assign($scope, __r);");
    w.leave();
    w.line("}");
}

