use super::super::template_emit::TemplateEmitter;
use super::super::util::{camel_of, js_ident};
use super::super::writer::{js_string, Writer};
use crate::ir::*;
use crate::keys::BindingRegistry;
use crate::lower::Lowered;
use crate::{CompileMode, CompileOpts};
use std::collections::BTreeSet;

pub(super) fn emit_setup_body(
    w: &mut Writer,
    program: &Program,
    bindings: &BindingRegistry,
    mode: CompileMode,
    opts: &CompileOpts,
    lowered: &Lowered,
) {
    let is_component = matches!(mode, CompileMode::Component);
    // A view that DECLARES `<client name="ship" />` opts into the keyed
    // injection surface: `ship` resolves from `ctx.clients.ship`, falling
    // back to the legacy `ctx.ship` alias so hosts that only pass
    // `opts.ship` keep working through the migration. Undeclared `ship`
    // stays the grandfathered ambient default (plain ctx destructure).
    let ship_via_clients =
        !is_component && program.clients.iter().any(|c| c.name == "ship");
    if is_component {
        w.line("const { props, host } = ctx;");
        w.line("if (!ctx.cleanups) ctx.cleanups = [];");
        w.line("const __cleanups = ctx.cleanups;");
    } else if lowered.uses_route {
        // Don't destructure `route` from ctx — the module-level
        // `__routeFactory` import (from `@ship/sui-runtime/route`) uses
        // the same name. Access `ctx.route` directly so there's no clash.
        if ship_via_clients {
            w.line("const { navigate } = ctx;");
        } else {
            w.line("const { ship, navigate } = ctx;");
        }
    } else if ship_via_clients {
        w.line("const { route, navigate } = ctx;");
    } else {
        w.line("const { ship, route, navigate } = ctx;");
    }
    if ship_via_clients {
        w.line("const ship = (ctx.clients ?? {}).ship ?? ctx.ship;");
    }
    w.line("const $scope = Object.create(null);");
    if is_component {
        emit_component_scope_seed(w, program);
    } else {
        w.line("$scope.ship = ship;");
        if lowered.uses_route {
            // Use host-provided RouteSignals or fall back to constructing
            // one via the view's own import of `@ship/sui-runtime/route`.
            w.line("$scope.route = ctx.route ?? __routeFactory();");
        } else {
            w.line("$scope.route = route;");
        }
        w.line("$scope.navigate = navigate;");
    }
    // Declared `<client>` names: destructure each from `ctx.clients` and seed
    // `$scope` so a view's `<data source="X.op()">` + inline handlers resolve
    // bare `X` (the per-view, host-injected data sources — the generalization
    // of the single `ship`). `ship`/`route`/`navigate` are already seeded above;
    // components inject clients via their own seam, not this path.
    if !is_component {
        for client in &program.clients {
            let name = client.name.as_str();
            if name == "ship" || name == "route" || name == "navigate" {
                continue;
            }
            w.line(&format!("const {{ {name} }} = ctx.clients ?? {{}};"));
            w.line(&format!("$scope.{name} = {name};"));
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
        if is_component {
            // Component path: pass $scope as ctx.scope so the controller's
            // on-handlers can mutate state signals. Store __r outside the
            // if-block so we can read LIFECYCLE_KEY after the template renders.
            emit_component_setup_call(w);
        } else {
            emit_setup_call(w);
        }
    }

    // Single-pass walk: emit the template body into a side writer so
    // the emitter's boundary-hoist buffer fills up. Drain hoists into
    // `w` before writing the body — the `__wireBoundary_b<N>` consts
    // must be in scope at every bindIf/bindFor call site they wrap.
    let mut emitter = TemplateEmitter::new(bindings, !is_component);
    if is_component {
        let mut tpl_w = Writer::new();
        tpl_w.line("const __root = host.shadowRoot ?? host;");
        let is_shadow = opts.component_meta.as_ref().map(|m| m.shadow).unwrap_or(false);
        if is_shadow {
            tpl_w.line("adoptSuiSheets(__root);");
        }
        emitter.emit_top_level(&mut tpl_w, &program.template, "__root", "__cleanups");
        if opts.setup_url.is_some() {
            // bind:this refs are now in $scope; call mount (if any) with a
            // proxy that reads them lazily — giving the controller access to
            // e.g. `state.termHost()` even though termHost was not yet in
            // $scope when setup.setup(ctx) ran above.
            emit_component_lifecycle_wiring(&mut tpl_w);
        }
        tpl_w.line("return $scope;");

        let hoists = emitter.take_boundary_hoists();
        if !hoists.is_empty() {
            w.push_str(&hoists);
        }
        w.push_str(&tpl_w.into_string());
    } else {
        let mut mount_w = Writer::new();
        mount_w.line("const __cleanups = [];");
        emitter.emit_top_level(
            &mut mount_w,
            &program.template,
            "container",
            "__cleanups",
        );
        mount_w.line("return () => { for (const c of __cleanups) try { c(); } catch (e) {} };");

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
    }
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

fn emit_component_scope_seed(w: &mut Writer, program: &Program) {
    for p in &program.props {
        w.line(&format!(
            "$scope.{ident} = props.{ident};",
            ident = js_ident(&camel_of(&p.name))
        ));
    }
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
    // `view(setup)` for views) — both wrap into `{ setup }`. No raw
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

fn emit_component_setup_call(w: &mut Writer) {
    // Component variant: expose $scope to the controller so its on-handlers
    // can mutate <state> signals via the writable proxy. Store __r in a
    // let-binding so emit_component_lifecycle_wiring can read LIFECYCLE_KEY
    // from it after the template has rendered (bind:this refs land in $scope
    // during emit_top_level, which runs after this call).
    w.line("let __r = null;");
    w.line("if (setup && typeof setup.setup === \"function\") {");
    w.enter();
    w.line("ctx.scope = $scope;");
    w.line("__r = setup.setup(ctx);");
    w.line("if (__r && typeof __r === \"object\") Object.assign($scope, __r);");
    w.leave();
    w.line("}");
}

fn emit_component_lifecycle_wiring(w: &mut Writer) {
    // Called after emit_top_level so bind:this refs are in $scope. Reads
    // the LIFECYCLE_KEY bundle from the controller's setup result and wires
    // mount + unmount into __cleanups.
    //
    // `__sp` is the component-mount state: raw-signal reads (so refs/prop
    // signals stay callable, `state.termHost()`) plus `state.$signals.x`. It's
    // the SAME runtime builder the view-mount + on-handler paths use — the
    // compiler emits a CALL to it, never a hand-rolled proxy, so the proxy
    // contract can't drift between compiled output and the runtime (the #121
    // `$signals`-undefined regression class).
    w.line("if (__r != null) {");
    w.enter();
    w.line("const __lc = Object.getOwnPropertyDescriptor(__r, \"__shipSetLifecycle\")?.value;");
    w.line("if (__lc) {");
    w.enter();
    w.line("const __sp = componentMountState($scope);");
    w.line("if (__lc.mount) { const __mc = __lc.mount(__sp, host); if (typeof __mc === \"function\") __cleanups.push(__mc); }");
    w.line("if (__lc.unmount) __cleanups.push(() => __lc.unmount(__sp));");
    w.leave();
    w.line("}");
    w.leave();
    w.line("}");
}
