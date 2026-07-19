//! Element + attribute DOM-construction emit.

// FILE_CAP_OVERRIDE: one cohesive concern — emitting DOM construction for an
// element and its attributes. emit_attr is a single (large) static/dynamic/
// hook/modifier dispatch matrix; the modifier + kbd-shortcut helpers are its
// leaves. Splitting by attr-kind fragments one mapping table.
use super::super::util::camel_of;
use super::super::writer::{js_string, Writer};
use super::TemplateEmitter;
use crate::ir::*;

use crate::lower;

/// SVG attribute names that require camelCase preservation. The HTML
/// parser lowercases all attribute names (mirroring DOMParser), but
/// SVG's `viewBox`, `preserveAspectRatio`, `gradientUnits`, etc. are
/// case-sensitive — the browser only honors them when set in their
/// canonical camelCase form via setAttribute on an SVG-namespaced
/// element. Map the lowercase parse-time name back to the canonical
/// form at emit time when inside an SVG subtree.
fn svg_camel_case(lower: &str) -> Option<&'static str> {
    match lower {
        "viewbox" => Some("viewBox"),
        "preserveaspectratio" => Some("preserveAspectRatio"),
        "gradientunits" => Some("gradientUnits"),
        "gradienttransform" => Some("gradientTransform"),
        "patternunits" => Some("patternUnits"),
        "patterntransform" => Some("patternTransform"),
        "clippathunits" => Some("clipPathUnits"),
        "maskunits" => Some("maskUnits"),
        "maskcontentunits" => Some("maskContentUnits"),
        "markerunits" => Some("markerUnits"),
        "markerwidth" => Some("markerWidth"),
        "markerheight" => Some("markerHeight"),
        "spreadmethod" => Some("spreadMethod"),
        "tablevalues" => Some("tableValues"),
        "stddeviation" => Some("stdDeviation"),
        "stoplocations" => Some("stopLocations"),
        "stitchtiles" => Some("stitchTiles"),
        "lengthadjust" => Some("lengthAdjust"),
        "textlength" => Some("textLength"),
        "startoffset" => Some("startOffset"),
        "pathlength" => Some("pathLength"),
        "refx" => Some("refX"),
        "refy" => Some("refY"),
        "diffuseconstant" => Some("diffuseConstant"),
        "specularconstant" => Some("specularConstant"),
        "specularexponent" => Some("specularExponent"),
        "surfacescale" => Some("surfaceScale"),
        "kernelmatrix" => Some("kernelMatrix"),
        "kernelunitlength" => Some("kernelUnitLength"),
        "filterunits" => Some("filterUnits"),
        "primitiveunits" => Some("primitiveUnits"),
        "baseprofile" => Some("baseProfile"),
        "xchannelselector" => Some("xChannelSelector"),
        "ychannelselector" => Some("yChannelSelector"),
        "lightingcolor" => Some("lightingColor"),
        "edgemode" => Some("edgeMode"),
        "limitingconeangle" => Some("limitingConeAngle"),
        "pointsatx" => Some("pointsAtX"),
        "pointsaty" => Some("pointsAtY"),
        "pointsatz" => Some("pointsAtZ"),
        _ => None,
    }
}

impl TemplateEmitter {
    fn attr_name_for_emit(&self, name: &str) -> String {
        if self.svg_depth > 0
            && let Some(canonical) = svg_camel_case(name)
        {
            return canonical.to_string();
        }
        name.to_string()
    }

    /// Emit an element. Returns the variable holding it.
    ///
    /// An EMPTY `parent_var` means DETACHED: build the element but do not append it
    /// anywhere. That is what lets a single-root `{#for}` body skip the
    /// DocumentFragment (and the `Array.from(frag.childNodes)` that reads it back
    /// out) — a row is one <tr>, and allocating a fragment plus an array to carry one
    /// element, once per row, is 2 objects x 10,000 rows of pure garbage.
    pub(in crate::emit) fn emit_element(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) -> String {
        let lower = el.tag.to_ascii_lowercase();
        // ── TEMPLATE CLONE (the fast path) ──────────────────────────────────
        // If this subtree is static structure, clone it instead of building it.
        // One DOM call instead of one per node AND one per attribute — 2.1x on the
        // benchmark row, which is the whole creation gap against Svelte/Solid.
        //
        // The `root_pending` flag must be consumed BEFORE serializing, because the
        // `sail-root` class is baked into the markup rather than added at runtime.
        let is_root = self.root_pending;
        if self.tpl_enabled
            && self.svg_depth == 0
            && let Some(html) = self.static_skeleton(el, is_root)
        {
            self.root_pending = false;
            let tpl = self.hoist_template(&html);
            let v = self.fresh("e");
            w.line(&format!("const {v} = {tpl}();"));
            self.wire_template(w, el, &v, cleanups_var);
            if !parent_var.is_empty() {
                w.line(&format!("{parent_var}.appendChild({v});"));
            }
            return v;
        }

        let v = self.fresh("e");
        // SVG-namespace elements must be created via createElementNS;
        // otherwise the browser parses them as HTML and they render
        // blank (no SVG layout, no viewBox honored). The `<svg>` root
        // bumps `svg_depth` and every descendant inherits the SVG
        // namespace for the duration of its subtree walk. `<foreignObject>`
        // contents switch back to HTML — call sites can extend this
        // when that need lands.
        let is_svg_root = lower == "svg";
        let in_svg = self.svg_depth > 0 || is_svg_root;
        if in_svg {
            w.line(&format!(
                "const {v} = document.createElementNS(\"http://www.w3.org/2000/svg\", {});",
                js_string(&el.tag)
            ));
        } else {
            w.line(&format!(
                "const {v} = document.createElement({});",
                js_string(&el.tag)
            ));
        }
        // Model-A root: a component's first template element IS the styled root
        // (the host is a transparent pass-through). It claims the `sail-root`
        // class so root CSS (`.sail-root`, `:host(X)` → `.sail-root`) targets it.
        if self.root_pending {
            self.root_pending = false;
            w.line(&format!("{v}.classList.add(\"sail-root\");"));
        }
        // Bump svg_depth BEFORE emitting attrs so that the <svg> root's
        // own attrs (viewBox, preserveAspectRatio, etc.) get camelCase
        // canonicalization. attr_name_for_emit guards on svg_depth > 0.
        if is_svg_root {
            self.svg_depth += 1;
        }
        for a in &el.attrs {
            self.emit_attr(w, a, &v, cleanups_var, &el.tag);
        }
        self.emit_children(w, &el.children, &v, cleanups_var);
        if is_svg_root {
            self.svg_depth -= 1;
        }
        if !parent_var.is_empty() {
            w.line(&format!("{parent_var}.appendChild({v});"));
        }
        // `<use-*>` elements aren't visible UI — they're behavior
        // markers. After the DOM is built, look the hook up in the
        // template-hook registry and invoke its factory so listeners
        // (e.g. `<use-kbd>` document keydown) actually attach.
        if let Some(hook_name) = lower.strip_prefix("use-") {
            emit_hook_activation(w, &v, hook_name, el, cleanups_var);
        }
        v
    }

    pub(super) fn emit_attr(
        &mut self,
        w: &mut Writer,
        a: &AttrInstr,
        el_var: &str,
        cleanups_var: &str,
        tag: &str,
    ) {
        match a {
            AttrInstr::Static { name, value, .. } => {
                let emit_name = self.attr_name_for_emit(name);
                w.line(&format!(
                    "{el_var}.setAttribute({}, {});",
                    js_string(&emit_name),
                    js_string(value)
                ));
            }
            AttrInstr::Expr { name, expr, .. } => {
                let key = self.next_key();
                let rw = self.rw(&expr.source, &[]);
                if self.is_static_row_expr(&expr.source) {
                    // D5: row-scope static attr — initial setAttribute
                    // only, matching bindAttr's null/false/"" → drop and
                    // true → "" semantics. Also recorded for the for's
                    // `update()` callback so keyed-row reuse refreshes
                    // the attribute when the row instance changes.
                    let v = self.fresh("av");
                    let emit_name = self.attr_name_for_emit(name);
                    let stmt = format!(
                        "{{ const {v} = ({rw}); if ({v} == null || {v} === false || {v} === \"\") {el_var}.removeAttribute({n}); else {el_var}.setAttribute({n}, {v} === true ? \"\" : String({v})); }}",
                        n = js_string(&emit_name)
                    );
                    w.line(&stmt);
                    let use_ = self.row_use_for_source(&expr.source);
                    self.inline_update_stmts.push((stmt, use_));
                } else {
                    let fv = self.for_version_comma(&expr.source);
                    let emit_name = self.attr_name_for_emit(name);
                    // Native elements are attribute-only (`bindAttr`). Custom
                    // elements get `bindDynamic`, which the toolchain uses to
                    // route attribute-vs-property at runtime — so the author
                    // writes one plain binding and never picks the channel.
                    let binder = if tag.contains('-') { "bindDynamic" } else { "bindAttr" };
                    w.line(&format!(
                        "{cleanups_var}.push({binder}({el_var}, {n}, () => ({fv}{rw})));",
                        n = js_string(&emit_name)
                    ));
                }
                if self.is_route && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateAttr(__m[{k}], {n}, () => ({rw})));",
                        k = js_string(&key),
                        n = js_string(name),
                    ));
                }
            }
            AttrInstr::DynamicAttr { name_expr, expr, .. } => {
                let key = self.next_key();
                self.emit_dynamic_attr(w, name_expr, expr, el_var, cleanups_var);
                if self.is_route && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.emit_dynamic_attr_hydrate(&key, name_expr, expr);
                }
            }
            AttrInstr::BindValue { expr, .. } => {
                let key = self.next_key();
                let rw = self.rw(&expr.source, &[]);
                w.line(&format!(
                    "{cleanups_var}.push(bindValue({el_var}, ({rw})));"
                ));
                if self.is_route && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateBindValue(__m[{k}], ({rw})));",
                        k = js_string(&key),
                    ));
                }
            }
            AttrInstr::BindChecked { expr, .. } => {
                let key = self.next_key();
                let rw = self.rw(&expr.source, &[]);
                w.line(&format!(
                    "{cleanups_var}.push(bindChecked({el_var}, ({rw})));"
                ));
                if self.is_route && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateBindChecked(__m[{k}], ({rw})));",
                        k = js_string(&key),
                    ));
                }
            }
            AttrInstr::BindThis { var, .. } => {
                // `bind:this={var}` is an element-as-value capture: it writes
                // the mounted element into the plain signal slot the controller
                // declared as `var`. It uses the same `runAction` lifecycle as
                // `use:` actions, so the slot is reset to `null` on unmount.
                w.line(&format!(
                    "runAction({el_var}, (__n) => {{ $scope.{var}.set(__n); return {{ destroy: () => $scope.{var}.set(null) }}; }}, undefined);",
                    el_var = el_var,
                    var = var
                ));
            }
            AttrInstr::SlotReflectEmpty { attr, .. } => {
                // `bindSlotEmpty(slot, host, attr)` reflects the attr on the
                // HOST element. In the emitted `__mount(container)` scope the
                // host is `container` (the CE adapter calls `__mount(host)`;
                // `__root = container.shadowRoot ?? container` is the render
                // root, NOT the host). Emitting bare `host` referenced an
                // unbound identifier → `ReferenceError` on mount for every
                // primitive with `<slot reflect-empty>`.
                w.line(&format!(
                    "{cleanups_var}.push(bindSlotEmpty({el_var}, container, {a}));",
                    a = js_string(attr)
                ));
            }
            AttrInstr::On { event, handler, modifiers, .. } => {
                let key = self.next_key();
                let rw = self.rw(&handler.source, &[]);
                let mods_js = serialize_modifiers(modifiers);

                // DELEGATE when we can. `addEventListener` allocates a listener record
                // per element AND a cleanup closure to remove it again — 2,000 of each
                // for a 1,000-row table with two handlers a row. A delegated handler is
                // one property assignment, and removing the row costs nothing at all
                // because the property dies with the node.
                if modifiers.is_empty() && DELEGATABLE_EVENTS.contains(&event.as_str()) {
                    self.delegated_events.insert(event.clone());
                    w.line(&format!(
                        "{el_var}.$${ev} = (function () {{ const __h = ({rw}); return (typeof __h === \"function\") ? __h.bind(undefined) : (() => {{}}); }})();",
                        ev = event
                    ));
                } else {
                    w.line(&format!(
                        "{cleanups_var}.push(on({el_var}, {ev}, (function () {{ const __h = ({rw}); return (typeof __h === \"function\") ? __h.bind(undefined) : (() => {{}}); }})(), {{ modifiers: {mods_js} }}));",
                        ev = js_string(event)
                    ));
                }
                // Phase 3b.4: at depth 0 in routes, also emit a direct
                // hydrate call so SSR'd `on:{event}="bN"` markers wire
                // up without going through the runtime walker /
                // `bindings` manifest. The matching `on:*` manifest
                // entry is suppressed in `js.rs::emit_bindings`.
                if self.is_route && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateOn(__m[{k}], {ev}, (function () {{ const __h = ({rw}); return (typeof __h === \"function\") ? __h.bind(undefined) : (() => {{}}); }})(), {mods_js}));",
                        k = js_string(&key),
                        ev = js_string(event),
                    ));
                }
            }
            AttrInstr::Class { name, expr, .. } => {
                let key = self.next_key();
                let rw = self.rw(&expr.source, &[]);
                if self.is_static_row_expr(&expr.source) {
                    // D5: row-scope static class — truthy → add. Also
                    // recorded for the for's `update()` callback so
                    // keyed-row reuse refreshes the class.
                    let stmt = format!(
                        "if ({rw}) {el_var}.classList.add({n}); else {el_var}.classList.remove({n});",
                        n = js_string(name)
                    );
                    w.line(&stmt);
                    let use_ = self.row_use_for_source(&expr.source);
                    self.inline_update_stmts.push((stmt, use_));
                } else {
                    let fv = self.for_version_comma(&expr.source);
                    w.line(&format!(
                        "{cleanups_var}.push(bindClass({el_var}, {n}, () => ({fv}{rw})));",
                        n = js_string(name)
                    ));
                }
                if self.is_route && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateClass(__m[{k}], {n}, () => ({rw})));",
                        k = js_string(&key),
                        n = js_string(name),
                    ));
                }
            }
            AttrInstr::Prop { name, expr, .. } => {
                let key = self.next_key();
                let rw = self.rw(&expr.source, &[]);
                // `prop:` targets a JS PROPERTY on the element, camelCase as it
                // appears on the object — but the template attribute name is
                // HTML-lowercased, so a multi-word target arrives kebab/lower
                // (`prop:prop-values`). camelCase it (the same `<prop>` → scope
                // identifier rule, `camel_of`) so `prop:prop-values` writes the
                // `propValues` property a component declares, not `prop-values`.
                let prop_key = camel_of(name);
                if self.is_static_row_expr(&expr.source) {
                    // D5: row-scope static prop — initial assignment.
                    // Also recorded for the for's `update()` callback
                    // so keyed-row reuse refreshes the prop.
                    let stmt = format!(
                        "{el_var}[{n}] = ({rw});",
                        n = js_string(&prop_key)
                    );
                    w.line(&stmt);
                    let use_ = self.row_use_for_source(&expr.source);
                    self.inline_update_stmts.push((stmt, use_));
                } else {
                    let fv = self.for_version_comma(&expr.source);
                    w.line(&format!(
                        "{cleanups_var}.push(bindProp({el_var}, {n}, () => ({fv}{rw})));",
                        n = js_string(&prop_key)
                    ));
                }
                if self.is_route && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateProp(__m[{k}], {n}, () => ({rw})));",
                        k = js_string(&key),
                        n = js_string(&prop_key),
                    ));
                }
            }
            AttrInstr::Mutation { expr, .. } => {
                let key = self.next_key();
                let chain = js_string(&expr.source);
                w.line(&format!(
                    "{cleanups_var}.push(wireFormMutation({el_var}, {chain}, $scope));"
                ));
                if self.is_route && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(wireFormMutation(__m[{k}], {chain}, $scope));",
                        k = js_string(&key),
                    ));
                }
            }
AttrInstr::Use { action, param, .. } => {
                // `use:action` / `use:action={param}`. The action resolves
                // from scope (a use-able hook handle or an imported action
                // fn); `runAction` invokes it with the element on mount,
                // calls its `update(param)` when the param thunk's deps
                // change, and `destroy()` on unmount. Registers at element
                // construction — no boundary key, no SSR marker.
                let action_rw = self.rw(action, &[]);
                let param_thunk = param
                    .as_ref()
                    .map(|e| format!("() => ({})", self.rw(&e.source, &[])))
                    .unwrap_or_else(|| "undefined".to_string());
                w.line(&format!(
                    "runAction({el_var}, {action_rw}, {param_thunk});",
                ));
            }
            AttrInstr::Motion { trigger, props, stagger_ms, timing, span: _ } => {
                // `motion:` is a thin runtime directive. The compiler resolves the
                // config — preset defaults, then config-level timing, then per-channel
                // overrides — and emits it; motion.dev creates and orchestrates every
                // animation after the node is live. This avoids inert-document WAAPI
                // animations and keeps reduced motion under one runtime authority.
                let db = lower::presets::load()
                    .expect("animation-presets.json is bundled at compile time");
                let mut motions: Vec<String> = Vec::with_capacity(props.len());
                for prop in props {
                    let spec = prop
                        .preset
                        .as_deref()
                        .map(|name| {
                            db.preset(name).expect("preset already validated in lower/bindings")
                        });
                    // No preset means a bare inline object, which is always a tween:
                    // springs are a physics config you name, not magnitudes you write.
                    let kind = match spec.map(|s| s.kind) {
                        Some(lower::presets::PresetKind::Spring) => "spring",
                        _ => "tween",
                    };
                    let mut fields: Vec<String> = vec![
                        format!("property: {}", js_string(&prop.property)),
                        format!(
                            "preset: {}",
                            js_string(prop.preset.as_deref().unwrap_or("inline"))
                        ),
                        format!("kind: {}", js_string(kind)),
                    ];
                    if let Some(spec) = spec {
                        if let Some(value) = spec.stiffness {
                            fields.push(format!("stiffness: {value}"));
                        }
                        if let Some(value) = spec.damping {
                            fields.push(format!("damping: {value}"));
                        }
                        if let Some(value) = spec.mass {
                            fields.push(format!("mass: {value}"));
                        }
                        if let Some(value) = spec.rest_speed {
                            fields.push(format!("restSpeed: {value}"));
                        }
                        if let Some(value) = spec.rest_delta {
                            fields.push(format!("restDelta: {value}"));
                        }
                    }

                    // Magnitudes: inline beats the preset's baked from/to.
                    let preset_from = spec.and_then(|s| s.from);
                    let preset_to = spec.and_then(|s| s.to);
                    if let Some(js) = self.motion_magnitude(prop.from.as_ref(), preset_from) {
                        fields.push(format!("from: {js}"));
                    }
                    if let Some(js) = self.motion_magnitude(prop.to.as_ref(), preset_to) {
                        fields.push(format!("to: {js}"));
                    }

                    // Timing: preset < config-level < channel-level.
                    let duration = prop.timing.duration.as_ref().or(timing.duration.as_ref());
                    if let Some(js) = self.motion_seconds(duration) {
                        fields.push(format!("duration: {js}"));
                    } else if let Some(token) = spec.and_then(|s| s.duration_token.as_deref()) {
                        // Motion reads duration in seconds; the compiler resolves the
                        // token to ms (one source of truth) then converts at the boundary.
                        fields.push(format!("duration: {}", duration_token_to_ms(token) / 1000.0));
                    }
                    let delay = prop.timing.delay.as_ref().or(timing.delay.as_ref());
                    if let Some(js) = self.motion_seconds(delay) {
                        fields.push(format!("delay: {js}"));
                    }
                    let easing = prop.timing.easing.as_ref().or(timing.easing.as_ref());
                    if let Some(js) = self.motion_easing(easing, &db) {
                        fields.push(format!("easing: {js}"));
                    } else if let Some(e) = spec.and_then(|s| s.easing.as_deref()) {
                        fields.push(format!("easing: {}", js_string(e)));
                    }
                    let iterations =
                        prop.timing.iterations.as_ref().or(timing.iterations.as_ref());
                    if let Some(js) = self.motion_iterations(iterations) {
                        fields.push(format!("iterations: {js}"));
                    } else if let Some(it) = spec.and_then(|s| s.iterations.as_deref()) {
                        // `infinite` (the catalogue's spell) maps to Motion's
                        // `Infinity` for `repeat`. `1` means "play once".
                        let mapped = if it == "infinite" {
                            "Infinity".to_string()
                        } else {
                            it.to_string()
                        };
                        fields.push(format!("iterations: {mapped}"));
                    }
                    let direction = prop.timing.direction.as_ref().or(timing.direction.as_ref());
                    if let Some(js) = self.motion_plain_string(direction) {
                        fields.push(format!("direction: {js}"));
                    } else if let Some(d) = spec.and_then(|s| s.direction.as_deref()) {
                        fields.push(format!("direction: {}", js_string(d)));
                    }
                    motions.push(format!("{{ {} }}", fields.join(", ")));
                }
                let stagger = stagger_ms
                    .map(|ms| format!(", stagger: {}", ms as f64 / 1000.0))
                    .unwrap_or_default();
                let config = format!(
                    "{{ trigger: {trigger}, motions: [{motions}]{stagger} }}",
                    trigger = js_string(trigger),
                    motions = motions.join(", "),
                );
                // A config with a signal in it is REACTIVE: pass a thunk so the
                // runtime can re-read it inside an effect and rebuild the animation
                // when the signal changes. A fully-literal config stays a plain
                // object — no effect, no overhead, byte-identical to before.
                let reactive = props.iter().any(|p| !p.exprs().is_empty())
                    || !timing.exprs().is_empty();
                // Either way the disposer is now registered. It was previously
                // dropped on the floor, which left an ambient loop running against a
                // node the framework had already torn down.
                if reactive {
                    w.line(&format!(
                        "{cleanups_var}.push(__sui_motion_attach({el_var}, () => ({config})));"
                    ));
                } else {
                    w.line(&format!(
                        "{cleanups_var}.push(__sui_motion_attach({el_var}, {config}));"
                    ));
                }
            }
        }
    }

    /// A magnitude (`from` / `to`). A bare number is the channel's native
    /// unit and passes straight through; a quoted string keeps whatever unit
    /// it carries (`'120px'`, `'-12%'`) and the runtime uses it verbatim; an
    /// expression becomes live JS. Falls back to the preset's baked value.
    fn motion_magnitude(
        &mut self,
        inline: Option<&MotionValue>,
        preset: Option<f64>,
    ) -> Option<String> {
        match inline {
            Some(MotionValue::Num(n)) => Some(fmt_num(*n)),
            Some(MotionValue::Str(s)) => Some(match s.trim().parse::<f64>() {
                // `'120'` and `120` should not mean different things.
                Ok(n) => fmt_num(n),
                Err(_) => js_string(s),
            }),
            Some(MotionValue::Expr(e)) => Some(format!("({})", self.rw(&e.source, &[]))),
            Some(MotionValue::Bezier(_)) => None,
            None => preset.map(fmt_num),
        }
    }

    /// A time field. The author writes MILLISECONDS; motion.dev wants
    /// seconds, so the conversion happens here, at the boundary, exactly
    /// once — including for a signal, whose value is also read as ms.
    fn motion_seconds(&mut self, v: Option<&MotionValue>) -> Option<String> {
        match v? {
            MotionValue::Num(n) => Some(fmt_num(n / 1000.0)),
            MotionValue::Str(s) => parse_time_ms(s).map(|ms| fmt_num(ms / 1000.0)),
            MotionValue::Expr(e) => Some(format!("(({}) / 1000)", self.rw(&e.source, &[]))),
            MotionValue::Bezier(_) => None,
        }
    }

    /// An easing. An inline bezier becomes the CSS function; a keyword
    /// resolves through the catalogue's easing tokens; anything else is
    /// passed through so `'linear'` and a hand-written `cubic-bezier(...)`
    /// both work.
    fn motion_easing(
        &mut self,
        v: Option<&MotionValue>,
        db: &lower::presets::PresetDb,
    ) -> Option<String> {
        match v? {
            MotionValue::Bezier(b) => Some(js_string(&format!(
                "cubic-bezier({}, {}, {}, {})",
                fmt_num(b[0]),
                fmt_num(b[1]),
                fmt_num(b[2]),
                fmt_num(b[3])
            ))),
            MotionValue::Str(s) => Some(js_string(
                db.resolver.easing_tokens.get(s.as_str()).map(String::as_str).unwrap_or(s),
            )),
            MotionValue::Expr(e) => Some(format!("({})", self.rw(&e.source, &[]))),
            MotionValue::Num(_) => None,
        }
    }

    fn motion_iterations(&mut self, v: Option<&MotionValue>) -> Option<String> {
        match v? {
            MotionValue::Num(n) => Some(fmt_num(*n)),
            MotionValue::Str(s) if s == "infinite" => Some("Infinity".to_string()),
            MotionValue::Str(s) => s.trim().parse::<f64>().ok().map(fmt_num),
            MotionValue::Expr(e) => Some(format!("({})", self.rw(&e.source, &[]))),
            MotionValue::Bezier(_) => None,
        }
    }

    fn motion_plain_string(&mut self, v: Option<&MotionValue>) -> Option<String> {
        match v? {
            MotionValue::Str(s) => Some(js_string(s)),
            MotionValue::Expr(e) => Some(format!("({})", self.rw(&e.source, &[]))),
            _ => None,
        }
    }

    fn emit_dynamic_attr(
        &mut self,
        w: &mut Writer,
        name_expr: &Expr,
        value_expr: &Expr,
        el_var: &str,
        cleanups_var: &str,
    ) {
        let n_rw = self.rw(&name_expr.source, &[]);
        let v_rw = self.rw(&value_expr.source, &[]);
        let prev = self.fresh("prevAttr");
        // Dynamic attr: subscribe to row-deps of BOTH the name and
        // value expressions (the effect re-runs when either changes).
        // Conservative — same field-set logic applies; if either
        // expression has Whole-row use, the whole effect escalates.
        let combined_source = format!("{} ; {}", name_expr.source, value_expr.source);
        let fvs = self.for_version_stmt(&combined_source);
        w.line(&format!("{{ let {prev} = null;"));
        w.line(&format!("  {cleanups_var}.push(effect(() => {{"));
        w.line(&format!("    {fvs}const __n = String({n_rw});"));
        w.line(&format!("    const __v = ({v_rw});"));
        w.line(&format!(
            "    if ({prev} && {prev} !== __n) {el_var}.removeAttribute({prev});"
        ));
        w.line(&format!("    {prev} = __n;"));
        w.line(&format!(
            "    if (__v == null || __v === false || __v === \"\") {el_var}.removeAttribute(__n);"
        ));
        w.line(&format!(
            "    else if (__v === true) {el_var}.setAttribute(__n, \"\");"
        ));
        w.line(&format!("    else {el_var}.setAttribute(__n, String(__v));"));
        w.line("  })); }");
    }

    /// Hydrate-side mirror of `emit_dynamic_attr`. The marker carrying
    /// the binding key is `attr-dynamic="bN"` on the SSR'd element, so
    /// `__m[bN]` resolves to the same element the CSR effect would
    /// observe. The body matches CSR exactly, modulo the element source.
    fn emit_dynamic_attr_hydrate(&mut self, key: &str, name_expr: &Expr, value_expr: &Expr) {
        let n_rw = self.rw(&name_expr.source, &[]);
        let v_rw = self.rw(&value_expr.source, &[]);
        let k = js_string(key);
        self.hydrate_calls.line(&format!("if (__m[{k}]) {{"));
        self.hydrate_calls.line(&format!("  const __el = __m[{k}];"));
        self.hydrate_calls.line("  let __prev = null;");
        self.hydrate_calls.line("  __cleanups.push(effect(() => {");
        self.hydrate_calls.line(&format!("    const __n = String({n_rw});"));
        self.hydrate_calls.line(&format!("    const __v = ({v_rw});"));
        self.hydrate_calls.line("    if (__prev && __prev !== __n) __el.removeAttribute(__prev);");
        self.hydrate_calls.line("    __prev = __n;");
        self.hydrate_calls.line(
            "    if (__v == null || __v === false || __v === \"\") __el.removeAttribute(__n);",
        );
        self.hydrate_calls.line("    else if (__v === true) __el.setAttribute(__n, \"\");");
        self.hydrate_calls.line("    else __el.setAttribute(__n, String(__v));");
        self.hydrate_calls.line("  }));");
        self.hydrate_calls.line("}");
    }
}

pub(super) fn serialize_modifiers(mods: &[Modifier]) -> String {
    if mods.is_empty() {
        return "[]".into();
    }
    let parts: Vec<String> = mods.iter().map(modifier_js).collect();
    format!("[{}]", parts.join(", "))
}

fn modifier_js(m: &Modifier) -> String {
    match m {
        Modifier::PreventDefault => "{ kind: \"preventDefault\" }".into(),
        Modifier::StopPropagation => "{ kind: \"stopPropagation\" }".into(),
        Modifier::StopImmediatePropagation => {
            "{ kind: \"stopImmediatePropagation\" }".into()
        }
        Modifier::SelfMod => "{ kind: \"self\" }".into(),
        Modifier::Trusted => "{ kind: \"trusted\" }".into(),
        Modifier::Key(arg) => with_arg("key", arg.as_deref()),
        Modifier::Debounce(arg) => with_arg("debounce", arg.as_deref()),
        Modifier::Throttle(arg) => with_arg("throttle", arg.as_deref()),
    }
}

fn with_arg(kind: &str, arg: Option<&str>) -> String {
    match arg {
        Some(a) => format!("{{ kind: \"{kind}\", arg: {} }}", js_string(a)),
        None => format!("{{ kind: \"{kind}\" }}"),
    }
}

/// Emit a `getTemplateHook(name)`-backed activation for a `<use-*>`
/// element. The hook factory expects `attrs` shaped per its declared
/// `attrs` schema; for `<use-kbd>` the children are `<on key=… action=…>`
/// records that get parsed at runtime into the `shortcuts` array.
///
/// Other hooks (`use-title`, `use-click-outside`, …) don't have
/// element children — their inputs come through the parent element's
/// attributes, which are already emitted as `setAttribute` calls on
/// `__el`. The factory reads them back via `el.getAttribute(…)` or via
/// a wrapping hydrate flow. Today only `<use-kbd>` and `<use-title>`
/// have authoring use in the repo, so this emit only special-cases
/// the kbd shortcut shape.
fn emit_hook_activation(
    w: &mut Writer,
    el_var: &str,
    hook_name: &str,
    el: &ElementInstr,
    cleanups_var: &str,
) {
    let hook_var = format!("__hook_{hook_name}");
    w.line(&format!(
        "const {hook_var} = getTemplateHook({});",
        js_string(hook_name)
    ));
    w.line(&format!("if ({hook_var}) {{"));
    w.enter();
    w.line("const __hookAttrs = Object.create(null);");
    if hook_name == "kbd" {
        // Pre-compute shortcuts from `<on key=… action=…>` static children.
        let shortcuts = collect_kbd_shortcuts(&el.children);
        w.line(&format!("__hookAttrs.shortcuts = {shortcuts};"));
        w.line("__hookAttrs.resolve = (__n) => $scope[__n];");
    } else {
        // Generic: surface el attributes by name as literals so the
        // factory can read them without re-traversing the DOM.
        for a in &el.attrs {
            if let AttrInstr::Static { name, value, .. } = a {
                w.line(&format!(
                    "__hookAttrs[{}] = {};",
                    js_string(name),
                    js_string(value)
                ));
            }
        }
    }
    let cleanup_fn = format!("(__fn) => {cleanups_var}.push(__fn)");
    w.line(&format!(
        "{hook_var}.factory({{ attrs: __hookAttrs, el: {el_var}, cleanup: {cleanup_fn} }});"
    ));
    w.leave();
    w.line("}");
}

/// Walk `<on key=… action=…>` children of a `<use-kbd>` element and
/// emit a JS array literal of `{ key, action }` records.
fn collect_kbd_shortcuts(children: &[Instr]) -> String {
    let mut items: Vec<String> = Vec::new();
    for child in children {
        let Instr::Element(el) = child else { continue };
        if !el.tag.eq_ignore_ascii_case("on") {
            continue;
        }
        let mut key: Option<String> = None;
        let mut action: Option<String> = None;
        for a in &el.attrs {
            if let AttrInstr::Static { name, value, .. } = a {
                match name.as_str() {
                    "key" => key = Some(value.clone()),
                    "action" => action = Some(value.clone()),
                    _ => {}
                }
            }
        }
        if let (Some(k), Some(a)) = (key, action) {
            items.push(format!(
                "{{ key: {}, action: {} }}",
                js_string(&k),
                js_string(&a)
            ));
        }
    }
    format!("[{}]", items.join(", "))
}

/// Emit a float as a JS number literal without Rust's trailing `.0`
/// (`3.6`, not `3.6000000000000005`; `120`, not `120.0`). Emitted JS is
/// read by humans in diffs and in the browser's sources panel.
fn fmt_num(n: f64) -> String {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    let mut s = format!("{n}");
    if s.contains('e') || s.contains('E') {
        s = format!("{n:.6}");
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

/// `'3600ms'` / `'3.6s'` → milliseconds. A quoted time is allowed to carry
/// its unit; a bare number already means ms.
fn parse_time_ms(raw: &str) -> Option<f64> {
    let t = raw.trim();
    if let Some(n) = t.strip_suffix("ms") {
        return n.trim().parse::<f64>().ok();
    }
    if let Some(n) = t.strip_suffix('s') {
        return n.trim().parse::<f64>().ok().map(|v| v * 1000.0);
    }
    t.parse::<f64>().ok()
}

/// Resolve a duration token name (e.g. `duration-medium`) to its
/// numeric ms value. Mirrors the values in
/// `packages/sail/tokens/src/tokens.css`. The compiler is the source of
/// truth for these values because it owns the bake; the runtime reads the
/// resolved number from the emitted JS and does not look up tokens.
///
/// This table covers the UI-scale durations the presets are built from.
/// It is deliberately NOT the way an author spells a bespoke timing —
/// `duration: 3600` says that directly, and minting a global token for one
/// decorative loop is the mistake the ambient tokens made.
fn duration_token_to_ms(token: &str) -> f64 {
    match token.strip_prefix("duration-").unwrap_or(token) {
        "instant" => 60.0,
        "fast" => 120.0,
        "medium" => 180.0,
        "slow" => 280.0,
        "deliberate" => 420.0,
        "glacial" => 14000.0,
        _ => 180.0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TEMPLATE CLONING
//
// The compiler's biggest lever on DOM creation. A `createElement` chain costs one
// JS->DOM round-trip per node AND per attribute; a `cloneNode` costs one, total,
// with the browser copying an already-parsed structure in C++. Measured at 2.1x on
// the js-framework-benchmark row — which is exactly the gap between us and
// Svelte/Solid on every CREATION benchmark, both of whom do this.
//
// So: serialize each element's STATIC SKELETON to HTML at compile time, hoist it to
// module scope as a lazily-parsed `<template>`, clone it per instance, and NAVIGATE
// to the nodes that need dynamic bindings.
//
// TWO HAZARDS, both respected below.
//
//  1. BINDING KEYS are allocated in source order and must match the bindings pass.
//     The navigation walk therefore visits instructions in exactly the order
//     `emit_children` does, calling `next_key()` at the same points.
//
//  2. ADJACENT TEXT MERGES. `<td>Hello {name}</td>` serializes to `<td>Hello  </td>`,
//     which the HTML parser collapses into ONE text node — so the binding would have
//     no node of its own and every index after it would shift. We refuse to template
//     an element whose children contain two adjacent text-producing instructions,
//     and fall back to the createElement path for it. (Svelte solves this with `<!>`
//     comment separators; that is a later refinement, not a correctness gap.)

/// Events that BUBBLE, and can therefore be caught by a single root listener.
///
/// `focus`/`blur` are deliberately absent — they do not bubble, so there would be
/// nothing at the root to catch (`focusin`/`focusout` are their bubbling twins and
/// are here). Same for `mouseenter`/`mouseleave`, `load`, `scroll`.
///
/// An event off this list, or any handler with modifiers (`once`/`capture` are
/// properties of a real listener registration and mean nothing to a delegated
/// dispatch), falls back to `addEventListener`. The author writes `on:click` either
/// way and never sees the seam.
const DELEGATABLE_EVENTS: &[&str] = &[
    "beforeinput", "click", "dblclick", "contextmenu", "focusin", "focusout", "input",
    "change", "keydown", "keypress", "keyup", "mousedown", "mousemove", "mouseout",
    "mouseover", "mouseup", "pointerdown", "pointermove", "pointerout", "pointerover",
    "pointerup", "touchend", "touchmove", "touchstart", "submit", "reset",
];

/// The delegatable-event list exposed to tooling (browser LSP) so completion
/// mirrors the events the compiler will route through root delegation.
pub fn delegatable_events() -> &'static [&'static str] {
    DELEGATABLE_EVENTS
}

/// HTML void elements — serialized without a closing tag.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;")
}

/// Does this instruction produce a TEXT node in the clone?
fn is_text_node(i: &Instr) -> bool {
    matches!(i, Instr::StaticText { .. } | Instr::DynamicText { .. })
}

impl super::TemplateEmitter {
    /// Serialize an element's static skeleton to HTML, or `None` if the subtree
    /// cannot be templated (control flow inside, or adjacent text that would merge).
    pub(super) fn static_skeleton(&self, el: &ElementInstr, is_root: bool) -> Option<String> {
        let mut out = String::new();
        self.write_skeleton(el, is_root, &mut out)?;
        Some(out)
    }

    fn write_skeleton(&self, el: &ElementInstr, is_root: bool, out: &mut String) -> Option<()> {
        let lower = el.tag.to_ascii_lowercase();
        // `<use-*>` needs a runtime activation hook — it doesn't belong in a
        // cloned skeleton.
        if lower.starts_with("use-") {
            return None;
        }

        out.push('<');
        out.push_str(&el.tag);

        // Static attributes bake into the markup — which is half the win, because it
        // also deletes every `setAttribute` call from the emitted JS.
        let mut classes: Vec<String> = Vec::new();
        if is_root {
            // Model-A: the component's first template element IS the styled root.
            classes.push("sail-root".into());
        }
        for a in &el.attrs {
            match a {
                AttrInstr::Static { name, value, .. } => {
                    if name.eq_ignore_ascii_case("class") {
                        classes.push(value.clone());
                    } else {
                        out.push(' ');
                        out.push_str(name);
                        out.push_str("=\"");
                        out.push_str(&escape_attr(value));
                        out.push('"');
                    }
                }
                // Everything else is applied at runtime after the clone.
                _ => {}
            }
        }
        if !classes.is_empty() {
            out.push_str(" class=\"");
            out.push_str(&escape_attr(&classes.join(" ")));
            out.push('"');
        }
        out.push('>');

        if VOID_ELEMENTS.contains(&lower.as_str()) {
            return Some(());
        }

        // Children. Refuse adjacent text (it would merge on parse — see hazard 2).
        let mut prev_was_text = false;
        for c in &el.children {
            let this_is_text = is_text_node(c);
            if this_is_text && prev_was_text {
                return None;
            }
            prev_was_text = this_is_text;

            match c {
                Instr::StaticText { text, .. } => out.push_str(&escape_text(text)),
                // A single space, exactly as Svelte does: it forces the parser to
                // create a text node for the binding to own. `bindText` overwrites it
                // synchronously on mount, so it is never seen.
                Instr::DynamicText { .. } => out.push(' '),
                // Comments are dropped by the createElement path too — keep parity or
                // the node indices diverge.
                Instr::Comment { .. } => {}
                Instr::Element(child) => self.write_skeleton(child, false, out)?,
                // Control flow needs a runtime anchor. Not templated (yet).
                _ => return None,
            }
        }

        out.push_str("</");
        out.push_str(&el.tag);
        out.push('>');
        Some(())
    }
}

impl super::TemplateEmitter {
    /// Does this subtree need any runtime work after cloning? (A fully-static
    /// subtree gets no variable at all — we clone it and never look at it again,
    /// which is the whole point.)
    fn needs_wiring(&self, el: &ElementInstr) -> bool {
        if el.attrs.iter().any(|a| !matches!(a, AttrInstr::Static { .. })) {
            return true;
        }
        el.children.iter().any(|c| match c {
            Instr::DynamicText { .. } => true,
            Instr::Element(child) => self.needs_wiring(child),
            _ => false,
        })
    }

    /// Walk the cloned tree and attach every dynamic binding.
    ///
    /// Order matters: `next_key()` is consumed here in exactly the sequence
    /// `emit_children` would consume it, so the binding registry and the hydrate
    /// keys line up whichever path an element took.
    pub(super) fn wire_template(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        el_var: &str,
        cleanups_var: &str,
    ) {
        for a in &el.attrs {
            if !matches!(a, AttrInstr::Static { .. }) {
                self.emit_attr(w, a, el_var, cleanups_var, &el.tag);
            }
        }

        // Navigate with firstChild / nextSibling — NOT `childNodes[i]`.
        //
        // `childNodes` is a LIVE NodeList: every access goes through the collection
        // machinery, and at ~6 lookups per row x 10,000 rows that overhead ate the
        // entire cloning win (create10k script went UP, 165ms -> 194ms, with the
        // clone in place). `firstChild` and `nextSibling` are direct pointers on the
        // node. Svelte navigates this way for exactly this reason.
        //
        // Indices are exact: the skeleton emits one node per StaticText /
        // DynamicText / Element and nothing for a Comment — precisely what the
        // createElement path produces.
        let mut cursor: Option<String> = None; // last materialized node var
        let mut cursor_idx = 0usize;
        let mut idx = 0usize;

        // Emit the chain that walks from the cursor (or the parent's first child) to
        // `target`, and bind it to a fresh var.
        macro_rules! nav {
            ($self:expr, $w:expr, $target:expr, $prefix:expr) => {{
                let v = $self.fresh($prefix);
                let expr = match &cursor {
                    None => {
                        let mut e = format!("{el_var}.firstChild");
                        for _ in 0..$target {
                            e.push_str(".nextSibling");
                        }
                        e
                    }
                    Some(c) => {
                        let mut e = c.clone();
                        for _ in 0..($target - cursor_idx) {
                            e.push_str(".nextSibling");
                        }
                        e
                    }
                };
                $w.line(&format!("const {v} = {expr};"));
                cursor = Some(v.clone());
                cursor_idx = $target;
                v
            }};
        }

        for c in &el.children {
            match c {
                Instr::Comment { .. } => continue,
                Instr::StaticText { .. } => {
                    idx += 1;
                }
                Instr::DynamicText { expr } => {
                    let key = self.next_key();
                    let v = nav!(self, w, idx, "t");
                    w.line(&format!("markHydrateText({v}, {k});", k = js_string(&key)));
                    self.emit_text_binding(w, &v, expr, cleanups_var, &key);
                    idx += 1;
                }
                Instr::Element(child) => {
                    if self.needs_wiring(child) {
                        let v = nav!(self, w, idx, "e");
                        self.wire_template(w, child, &v, cleanups_var);
                    }
                    idx += 1;
                }
                _ => unreachable!("static_skeleton() rejects control flow"),
            }
        }
    }
}
