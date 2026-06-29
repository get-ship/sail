//! Element + attribute DOM-construction emit.

// FILE_CAP_OVERRIDE: one cohesive concern — emitting DOM construction for an
// element and its attributes. emit_attr is a single (large) static/dynamic/
// hook/modifier dispatch matrix; the modifier + kbd-shortcut helpers are its
// leaves. Splitting by attr-kind fragments one mapping table.
use super::super::util::camel_of;
use super::super::writer::{js_string, Writer};
use super::TemplateEmitter;
use crate::ir::*;

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

    pub(in crate::emit) fn emit_element(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let lower = el.tag.to_ascii_lowercase();
        // Mirror `bindings::Builder::walk_element`: skip `<data>`
        // entirely (no children walked, no keys allocated).
        if lower == "data" {
            return;
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
        // Bump svg_depth BEFORE emitting attrs so that the <svg> root's
        // own attrs (viewBox, preserveAspectRatio, etc.) get camelCase
        // canonicalization. attr_name_for_emit guards on svg_depth > 0.
        if is_svg_root {
            self.svg_depth += 1;
        }
        for a in &el.attrs {
            self.emit_attr(w, a, &v, cleanups_var);
        }
        self.emit_children(w, &el.children, &v, cleanups_var);
        if is_svg_root {
            self.svg_depth -= 1;
        }
        w.line(&format!("{parent_var}.appendChild({v});"));
        // `<use-*>` elements aren't visible UI — they're behavior
        // markers. After the DOM is built, look the hook up in the
        // template-hook registry and invoke its factory so listeners
        // (e.g. `<use-kbd>` document keydown) actually attach.
        if let Some(hook_name) = lower.strip_prefix("use-") {
            emit_hook_activation(w, &v, hook_name, el, cleanups_var);
        }
    }

    pub(super) fn emit_attr(
        &mut self,
        w: &mut Writer,
        a: &AttrInstr,
        el_var: &str,
        cleanups_var: &str,
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
                    w.line(&format!(
                        "{cleanups_var}.push(bindAttr({el_var}, {n}, () => ({fv}{rw})));",
                        n = js_string(&emit_name)
                    ));
                }
                if self.is_view && self.boundary_depth == 0 {
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
                if self.is_view && self.boundary_depth == 0 {
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
                if self.is_view && self.boundary_depth == 0 {
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
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateBindChecked(__m[{k}], ({rw})));",
                        k = js_string(&key),
                    ));
                }
            }
            AttrInstr::BindThis { var, .. } => {
                let key = self.next_key();
                // Refs are write-once-from-runtime. If the controller's
                // scope: factory pre-declared `var: ref(null)` we write
                // the freshly constructed element into that slot;
                // otherwise we install a fresh signal seeded with the
                // element. Same shape as React's `ref.current`, but
                // signal-typed.
                w.line(&format!("attachRef($scope, {v}, {el_var});", v = js_string(var)));
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateRef(__m[{k}], $scope, {v}));",
                        k = js_string(&key),
                        v = js_string(var),
                    ));
                }
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
                w.line(&format!(
                    "{cleanups_var}.push(on({el_var}, {ev}, (function () {{ const __h = ({rw}); return (typeof __h === \"function\") ? __h.bind(undefined) : (() => {{}}); }})(), {{ modifiers: {mods_js} }}));",
                    ev = js_string(event)
                ));
                // Phase 3b.4: at depth 0 in views, also emit a direct
                // hydrate call so SSR'd `on:{event}="bN"` markers wire
                // up without going through the runtime walker /
                // `bindings` manifest. The matching `on:*` manifest
                // entry is suppressed in `js.rs::emit_bindings`.
                if self.is_view && self.boundary_depth == 0 {
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
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateClass(__m[{k}], {n}, () => ({rw})));",
                        k = js_string(&key),
                        n = js_string(name),
                    ));
                }
            }
            AttrInstr::Style { name, expr, .. } => {
                let key = self.next_key();
                let rw = self.rw(&expr.source, &[]);
                let prop = js_string(name);
                if self.is_static_row_expr(&expr.source) {
                    // D5: row-scope static style — initial setProperty,
                    // matching the effect's null → remove pass. Also
                    // recorded for the for's `update()` callback so
                    // keyed-row reuse refreshes the style.
                    let v = self.fresh("sv");
                    let stmt = format!(
                        "{{ const {v} = ({rw}); if ({v} == null) {el_var}.style.removeProperty({prop}); else {el_var}.style.setProperty({prop}, String({v})); }}"
                    );
                    w.line(&stmt);
                    let use_ = self.row_use_for_source(&expr.source);
                    self.inline_update_stmts.push((stmt, use_));
                } else {
                    let fvs = self.for_version_stmt(&expr.source);
                    w.line(&format!(
                        "{cleanups_var}.push(effect(() => {{ {fvs}const __v = ({rw}); if (__v == null) {el_var}.style.removeProperty({prop}); else {el_var}.style.setProperty({prop}, String(__v)); }}));"
                    ));
                }
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateStyle(__m[{k}], {p}, () => ({rw})));",
                        k = js_string(&key),
                        p = prop,
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
                if self.is_view && self.boundary_depth == 0 {
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
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(wireFormMutation(__m[{k}], {chain}, $scope));",
                        k = js_string(&key),
                    ));
                }
            }
            AttrInstr::Transition { name, opts, dir, .. } => {
                // Built-in names (fade/slide/scale) are auto-imported,
                // so `rewrite_for_scope` leaves them bare; custom names
                // resolve through `$scope`. Transitions register on
                // element creation — no boundary key, no SSR marker,
                // no manifest entry.
                let name_rw = self.rw(name, &[]);
                let opts_rw = opts
                    .as_ref()
                    .map(|e| self.rw(&e.source, &[]))
                    .unwrap_or_else(|| "{}".to_string());
                w.line(&format!(
                    "registerTransition({el_var}, {name_rw}, ({opts_rw}), \"{d}\");",
                    d = dir.as_str(),
                ));
            }
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
