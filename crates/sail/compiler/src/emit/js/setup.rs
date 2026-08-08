use super::super::template_emit::TemplateEmitter;
use super::super::util::{camel_of, js_ident};
use super::super::writer::{Writer, js_string};
use crate::CompileOpts;
use crate::ir::*;
use crate::keys::BindingRegistry;

/// Returns the MODULE-scope hoists (the `__tpl(...)` cloners). They cannot live in
/// the setup body: a template parsed there would be re-parsed for every component
/// instance, when the whole point is to parse it once per program.
pub(super) fn emit_setup_body(
    w: &mut Writer,
    program: &Program,
    bindings: &BindingRegistry,
    opts: &CompileOpts,
    sx_var_bindings: &[crate::check::binding::StyleBinding],
) -> Writer {
    // A source is a COMPONENT (custom element) iff it carries a tag
    // (`component_meta`); otherwise it's a tagless mount-form (a route/page).
    // This is the sole route/component distinction now that `CompileMode::Route`
    // is gone — keyed on the tag, not a mode flag.
    let is_component = opts.component_meta.is_some();

    // ── ctx → $scope seeding (unified) ──────────────────────────────────────
    // Every compiled `.sail` is a component module: `setup(ctx)` seeds the scope
    // and returns `{ …$scope, __mount(container), [hydrate] }`. ship/route/
    // navigate always come from ctx (deck injects them for pages; `undefined`
    // for a registered component, which never reads them). Routing is deck's —
    // SAIL vends no router or factory fallback. Props come from `ctx.props`
    // (registered components have them, pages don't).
    w.line("const { route, navigate, providers } = ctx;");
    w.line("const $scope = Object.create(null);");
    w.line("$scope.route = route;");
    w.line("$scope.navigate = navigate;");
    w.line("for (const __k of Object.keys(ctx)) {");
    w.line("  if (__k === \"route\" || __k === \"navigate\" || __k === \"scope\" || __k === \"stateKeys\" || __k === \"host\" || __k === \"providers\" || __k === \"props\") continue;");
    w.line("  $scope[__k] = ctx[__k];");
    w.line("}");
    w.line("if (providers && typeof providers === \"object\") Object.assign($scope, providers);");
    // Props (registered components) → seed from `ctx.props` (the prop signals
    // the custom-element adapter passes). Pages declare none.
    if !program.props.is_empty() {
        w.line("const __props = ctx.props ?? {};");
        for p in &program.props {
            let ident = js_ident(&camel_of(&p.name));
            w.line(&format!("$scope.{ident} = __props.{ident};"));
        }
    }
    if opts.setup_url.is_some() {
        // Expose `$scope` to the controller (`ctx.scope`) and merge its return.
        // The LIFECYCLE_KEY bundle flows through the setup return to
        // `mountComponent` — the single engine that wires mount()/unmount() —
        // so there is no inline lifecycle wiring here anymore.
        emit_setup_call(w);
    }

    // ── deferred render: `__mount(container)` + optional `hydrate(root, ctx)` ─
    // One shape for both forms. The custom-element adapter calls `__mount(host)`
    // on connect; a page / internal child calls it with its container. Always
    // route-shaped so the hydrate buffers populate (harmless for CSR-island
    // components — `mountComponent` ignores `hydrate`). The controller lifecycle
    // is wired by `mountComponent` from the LIFECYCLE_KEY that rides the return,
    // not inline here. Hoists drain into `w` before the body so the
    // `__wireBoundary_b<N>` consts are in scope.
    let mut emitter = TemplateEmitter::new(bindings, true);
    let mut mount_w = Writer::new();
    mount_w.line("const __cleanups = [];");
    // Light DOM is the product model: render into `container` (the host or page
    // mount target). Residual: `container.shadowRoot ?? container` still prefers
    // an attached shadow root for rare foreign-content containment wrappers
    // (`shadow: true`) — not taught as L1. Golden fixtures lock this form.
    mount_w.line("const __root = container.shadowRoot ?? container;");
    // The `(--x)` binding: every prop/signal a `style="…"` read matched gets a
    // CSS `@property` registration (own, unscoped install — see
    // `emit::styles::register_properties_install_js`) and is reflected onto
    // the host as an inline custom property, seeded now and kept live by the
    // same `effect()` that seeds it (an effect's body runs once
    // synchronously at creation, which IS mount time, then again on every
    // write). `__root` is the host: for a registered component `container`
    // IS the host element (light DOM); for a page/route it's the mount target,
    // which plays the same "inline custom property host" role for its
    // subtree's `var(--x)` reads.
    if !sx_var_bindings.is_empty() {
        mount_w.push_str(&crate::emit::styles::register_properties_install_js(
            sx_var_bindings,
        ));
        // ONE effect per instance writes ALL bound custom properties: N
        // instances x M dynamic style props must not cost N x M effects
        // (each with its own scheduling + DOM write). The single effect
        // reads every bound signal, so any one firing re-runs it and
        // rewrites all — the same batching fine-grained systems accept;
        // no per-var dirty tracking. Each statement is block-scoped so the
        // `__v` temporaries don't collide.
        mount_w.line("__cleanups.push(effect(() => {");
        mount_w.enter();
        for b in sx_var_bindings {
            let scope_ident = js_ident(&b.scope_name);
            match b.unit {
                // Tone / token-group: reflect var *references* for each member
                // (`--tone: var(--primary)`, `--tone-foreground: var(--primary-foreground)`).
                // Rest = unset → removeProperty so surface pack paint falls through.
                crate::check::binding::StyleBindingUnit::TokenGroup => {
                    emit_token_group_reflect(&mut mount_w, &b.scope_name, &b.css_name, scope_ident);
                }
                other => {
                    let css_prop = js_string(&format!("--{}", b.css_name));
                    // LengthPx: number sources mean px — never write a unitless number into a length slot.
                    let value_expr = match other {
                        crate::check::binding::StyleBindingUnit::LengthPx => {
                            "(typeof __v === \"number\" ? __v + \"px\" : String(__v))".to_string()
                        }
                        _ => "String(__v)".to_string(),
                    };
                    mount_w.line(&format!(
                        "{{ const __v = ($scope.{scope_ident}()); if (__v == null) __root.style.removeProperty({css_prop}); else __root.style.setProperty({css_prop}, {value_expr}); }}"
                    ));
                }
            }
        }
        mount_w.leave();
        mount_w.line("}));");
    }
    // Pages install their lifted top-level `<style>` into the container.
    // Registered components install styles via the custom-element adapter
    // (scoped by `data-component` / @scope donut), so the module skips it.
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
        is_component,
    );
    mount_w.line("return () => { for (const c of __cleanups) try { c(); } catch (e) {} };");

    let mut module_hoists = emitter.take_module_hoists();
    // One root listener per event TYPE, installed once at module scope — instead of
    // one addEventListener per element (2,000 of them for a 1,000-row table).
    let delegated = emitter.take_delegated_events();
    if !delegated.is_empty() {
        let names: Vec<String> = delegated.iter().map(|e| js_string(e)).collect();
        module_hoists.line(&format!("delegateEvents([{}]);", names.join(", ")));
    }
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
        emit_route_hydrate(w, &hydrate_calls, hydrate_uses_text, hydrate_uses_markers);
    }
    if !callbacks.is_empty() {
        w.push_str(&callbacks);
    }
    w.leave();
    w.line("});");
    module_hoists
}

/// Reflect a Tone-typed prop onto bridge custom properties.
///
/// `prop="primary"` → `--{css}: var(--primary); --{css}-foreground: var(--primary-foreground)`.
/// Unset / empty → removeProperty so component-surface rest paint falls through (IACVT).
fn emit_token_group_reflect(
    w: &mut Writer,
    _scope_name: &str,
    css_name: &str,
    scope_ident: String,
) {
    // Full Tone table (registry-derived) as a compact JS object.
    let mut map = String::from("{");
    for (i, g) in sail_lint::token_groups::tone_groups().iter().enumerate() {
        if i > 0 {
            map.push(',');
        }
        map.push_str(&js_string(g.name));
        map.push_str(":{");
        for (j, m) in g.members.iter().enumerate() {
            if j > 0 {
                map.push(',');
            }
            let key = if m.name == "default" { "" } else { m.name };
            map.push_str(&js_string(key));
            map.push(':');
            map.push_str(&js_string(m.public_name));
        }
        map.push('}');
    }
    map.push('}');
    // All bridge prop names that any group might write (union of members).
    let mut bridges: Vec<String> = Vec::new();
    bridges.push(format!("--{css_name}"));
    for g in sail_lint::token_groups::tone_groups() {
        for m in g.members {
            if m.name == "default" {
                continue;
            }
            let p = format!("--{css_name}-{}", m.name);
            if !bridges.contains(&p) {
                bridges.push(p);
            }
        }
    }
    let bridges_js: Vec<String> = bridges.iter().map(|b| js_string(b)).collect();
    let bridges_lit = bridges_js.join(", ");
    // Build the effect body line carefully (nested braces / template literals).
    let mut line = String::new();
    line.push_str("{ const __v = ($scope.");
    line.push_str(&scope_ident);
    line.push_str("()); const __map = ");
    line.push_str(&map);
    line.push_str("; const __bridges = [");
    line.push_str(&bridges_lit);
    line.push_str("]; if (__v == null || __v === \"\") { for (const __p of __bridges) __root.style.removeProperty(__p); } else { const __g = __map[String(__v)]; for (const __p of __bridges) __root.style.removeProperty(__p); if (__g) { for (const __k of Object.keys(__g)) { const __prop = __k ? `--");
    line.push_str(css_name);
    line.push_str("-${__k}` : `--");
    line.push_str(css_name);
    line.push_str("`; __root.style.setProperty(__prop, `var(--${__g[__k]})`); } } } }");
    w.line(&line);
}

/// Emit the `hydrate(root, ctx)` named property on a route's setup
/// return. Wires every binding kind the compiler now direct-emits
/// (Phase 3b.3+) — runs after `wireBoundaries` so SSR'd markers inside
/// boundary subtrees are already detached and won't be matched.
fn emit_route_hydrate(w: &mut Writer, hydrate_calls: &str, uses_text: bool, uses_markers: bool) {
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

fn emit_setup_call(w: &mut Writer) {
    // Author-facing paired-controller shape is `controller(def)` (and
    // `component(setup)` for the mount-form) — both wrap into `{ setup }`. No raw
    // `function setup(ctx)` author export. Compiler emits a single call
    // path; if a paired `.ts` returns a non-object, we ignore it instead
    // of letting it shadow the runtime's $scope.
    //
    // Expose `$scope` to the controller via `ctx.scope` BEFORE invoking
    // its setup. If the controller's own `setup:` factory declares a
    // `live(...)`-backed signal (the real living-async surface) and the
    // template reads the same name, both must resolve to the SAME signal
    // object, otherwise the controller computes against a stale copy while
    // the template sees the patched one. Mount.ts hands us `ctx.scope` as a
    // snapshot of the SSR-built scope; we narrow it to `$scope` here so
    // controller-and-template agree on a single source-of-truth signal.
    // Components already did this via `emit_component_setup_call`; route
    // modules were lagging.
    w.line("if (setup && typeof setup.setup === \"function\") {");
    w.enter();
    w.line("ctx.scope = $scope;");
    w.line("const __r = setup.setup(ctx);");
    w.line("if (__r && typeof __r === \"object\") Object.assign($scope, __r);");
    w.leave();
    w.line("}");
}
