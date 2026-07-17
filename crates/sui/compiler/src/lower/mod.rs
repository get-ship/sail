//! IR → emit-ready ops.
//!
//! The IR produced by `parse` is already structured by directive +
//! template instructions. The lower step's job is to compute the
//! information the emitter needs but doesn't want to recompute:
//!
//!  - which runtime imports the emitted JS will need
//!  - the flat list of bindings to register at mount time
//!  - the final list of decoder calls per component prop
//!
//! Mostly this is a walk over the IR collecting metadata. The emit
//! step uses the IR plus this `Lowered` struct to produce JS.
use crate::diagnostics::CompileError;
use crate::ir::*;
use std::collections::BTreeSet;

pub mod bindings;
pub mod directives;
pub mod for_uses_row;
pub mod presets;
pub mod row_field_uses;
pub mod template;

#[derive(Debug, Clone, Default)]
pub struct Lowered {
    /// Symbols imported from `@ship/sail-runtime`. Sorted for stable emit.
    pub runtime_imports: BTreeSet<&'static str>,
    /// Symbols imported from `@ship/sail-dom`. Sorted for stable emit.
    /// Phase 3b.3: views with depth-0 `DynamicText` import the hydrate
    /// helpers (`hydrateText`, `collectTextOpens`) — direct-emit replaces
    /// the `text` manifest entry that the runtime walker used to wire.
    pub dom_imports: BTreeSet<&'static str>,
    /// `true` if the user's setup file is referenced anywhere.
    pub uses_setup: bool,
    /// `true` if the template contains any `<use-*>` element. The
    /// emitter drops a side-effect import of
    /// `@ship/sail-runtime/template-hooks` so the eight built-in hooks
    /// register on view module load — `@ship/sail-runtime`'s entry no
    /// longer ships them, so views that need them have to ask.
    pub uses_template_hooks: bool,
    /// `true` if any template expression references `route` as an
    /// identifier. The emitter adds `import { route } from
    /// "@ship/sail-runtime/route"` so routeless views don't pull in the
    /// route module.
    pub uses_route: bool,
    /// Runtime attacher symbols referenced by `motion:` directives.
    /// The emitter folds these into
    /// the `@ship/sail-runtime` import list so views using `motion:`
    /// pull in the spring_attach / tween_attach helpers transparently.
    /// In milestone 1 the only entries are `"spring_attach"` (when any
    /// `motion:` preset resolves to a spring) and `"tween_attach"`
    /// (when any resolves to a tween). Inserted by
    /// [`crate::lower::bindings::collect_attr_imports`] when an
    /// `AttrInstr::Motion` is lowered.
    pub motion_builtins: BTreeSet<&'static str>,
    /// Every custom-element tag the template uses (`sui-*` / `ship-*`).
    /// Emitted as side-effect `import "@sui-tag/<tag>";` when
    /// `CompileOpts::emit_component_imports` is set, so components join the
    /// view's import graph (the per-view resolver). Collected once here; the
    /// emitter decides whether to write the imports.
    pub used_component_tags: BTreeSet<String>,
}

/// Walk the program, build a `Lowered` summary.
///
/// Returns `Err` for hard validation failures discovered during the
/// walk (currently: `motion:` directives that reference an unknown
/// preset). Soft signals (`uses_setup`, `uses_route`, …) stay on the
/// [`Lowered`] value.
pub fn lower(program: &Program, is_view: bool) -> Result<Lowered, CompileError> {
    let mut out = Lowered::default();
    // Every compiled `.sui` exports `component(...)` — the tagless mount-form
    // (`component(setup)`) or the registered custom element (`component(tag,
    // def)`). `prop` only rides along when a registered component declares props.
    out.runtime_imports.insert("component");
    // A tagged source registers itself via `component(tag, def, __default)` —
    // the same `component` symbol, 3-arg overload — so no extra import. `prop`
    // rides along only when a registered component declares props.
    if !is_view && !program.props.is_empty() {
        out.runtime_imports.insert("prop");
    }
    let mut depth = 0;
    walk_instrs(&program.template, &mut out, is_view, &mut depth)?;
    out.used_component_tags = crate::used_tags::collect_used_tags(&program.template);
    Ok(out)
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

fn expr_references_route(source: &str) -> bool {
    let target = b"route";
    let bytes = source.as_bytes();
    let tlen = target.len();
    if bytes.len() < tlen {
        return false;
    }
    for i in 0..=bytes.len() - tlen {
        if &bytes[i..i + tlen] == target {
            let ok_before = i == 0 || !is_ident_char(bytes[i - 1]);
            let ok_after = i + tlen >= bytes.len() || !is_ident_char(bytes[i + tlen]);
            if ok_before && ok_after {
                return true;
            }
        }
    }
    false
}

fn check_attr_for_route(a: &AttrInstr, out: &mut Lowered) {
    if out.uses_route {
        return;
    }
    match a {
        AttrInstr::Expr { expr, .. }
        | AttrInstr::BindValue { expr, .. }
        | AttrInstr::BindChecked { expr, .. }
        | AttrInstr::Class { expr, .. }
        | AttrInstr::Prop { expr, .. }
        | AttrInstr::Mutation { expr, .. } => {
            if expr_references_route(&expr.source) {
                out.uses_route = true;
            }
        }
        AttrInstr::On { handler, .. } => {
            if expr_references_route(&handler.source) {
                out.uses_route = true;
            }
        }
        AttrInstr::DynamicAttr { name_expr, expr, .. } => {
            if expr_references_route(&name_expr.source) || expr_references_route(&expr.source) {
                out.uses_route = true;
            }
        }
        AttrInstr::Static { .. }
        | AttrInstr::BindThis { .. }
        | AttrInstr::SlotReflectEmpty { .. }
        | AttrInstr::Motion { .. } => {}
AttrInstr::Use { param, .. } => {
            if let Some(e) = param
                && expr_references_route(&e.source)
            {
                out.uses_route = true;
            }
        }
    }
}

fn walk_instrs(instrs: &[Instr], out: &mut Lowered, is_view: bool, depth: &mut usize) -> Result<(), CompileError> {
    for instr in instrs {
        walk_instr(instr, out, is_view, depth)?;
    }
    Ok(())
}
fn walk_instr(instr: &Instr, out: &mut Lowered, is_view: bool, depth: &mut usize) -> Result<(), CompileError> {
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } => {}
        Instr::DynamicText { expr } => {
            out.runtime_imports.insert("bindText");
            // Slice 2 — emit `markHydrateText` alongside every dynamic
            // text position so the ship-js SSR path serializes the
            // marker comments the client `hydrate()` reads. Browser
            // DOM ignores (no `__hydrateKey` field on a real Text);
            // server-DOM wraps the node at toHTML time.
            out.runtime_imports.insert("markHydrateText");
            // Phase 3b.3: depth-0 text bindings in views are direct-emit.
            // The compiler still calls `bindText` from `__mount` (legacy
            // CSR path), but `hydrate(root, ctx)` wires via these helpers.
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectTextOpens");
                out.dom_imports.insert("hydrateText");
            }
            if is_view && !out.uses_route && expr_references_route(&expr.source) {
                out.uses_route = true;
            }
        }
        Instr::Element(el) => {
            if el.tag.starts_with("use-") {
                out.uses_template_hooks = true;
                // Hook activation needs `getTemplateHook` to look the
                // factory up in the registry at mount time.
                out.runtime_imports.insert("getTemplateHook");
            }
            let is_custom_element = el.tag.contains('-');
            for a in &el.attrs {
                if is_view {
                    check_attr_for_route(a, out);
                }
                bindings::collect_attr_imports(a, out, is_view, *depth, is_custom_element)?;
            }
            walk_instrs(&el.children, out, is_view, depth)?;
        }
        Instr::WhileLoading(wl) => {
            out.runtime_imports.insert("bindWhileLoading");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateWhileLoading");
            }
            *depth += 1;
            walk_instrs(&wl.body, out, is_view, depth)?;
            *depth -= 1;
        }
        Instr::WhileError(we) => {
            out.runtime_imports.insert("bindWhileError");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateWhileError");
            }
            *depth += 1;
            walk_instrs(&we.body, out, is_view, depth)?;
            *depth -= 1;
        }
        Instr::If(IfInstr { branches, else_body, .. }) => {
            out.runtime_imports.insert("bindIf");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
            }
            *depth += 1;
            for b in branches {
                if is_view && !out.uses_route && expr_references_route(&b.cond.source) {
                    out.uses_route = true;
                }
                walk_instrs(&b.body, out, is_view, depth)?;
            }
            if let Some(eb) = else_body {
                walk_instrs(eb, out, is_view, depth)?;
            }
            *depth -= 1;
        }
        Instr::For(ForInstr { var_name, index_name, iterable, key, body, else_body, .. }) => {
            out.runtime_imports.insert("bindFor");
            // the per-row update (i.e. emit_for will keep the `__v`
            // version sentinel). Static row bodies skip it.
            if for_uses_row::body_references_for_vars(
                body,
                var_name,
                index_name.as_deref(),
            ) {
                out.runtime_imports.insert("signal");
            }
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
            }
            if is_view && !out.uses_route {
                if expr_references_route(&iterable.source) {
                    out.uses_route = true;
                } else if let Some(k) = key
                    && expr_references_route(&k.source) {
                        out.uses_route = true;
                    }
            }
            *depth += 1;
            walk_instrs(body, out, is_view, depth)?;
            if let Some(eb) = else_body {
                walk_instrs(eb, out, is_view, depth)?;
            }
            *depth -= 1;
        }
        Instr::Match(MatchInstr { discriminant, cases, default, .. }) => {
            out.runtime_imports.insert("bindIf");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
            }
            if is_view && !out.uses_route && expr_references_route(&discriminant.source) {
                out.uses_route = true;
            }
            *depth += 1;
            for c in cases {
                if is_view && !out.uses_route && expr_references_route(&c.test.source) {
                    out.uses_route = true;
                }
                walk_instrs(&c.body, out, is_view, depth)?;
            }
            if let Some(d) = default {
                walk_instrs(d, out, is_view, depth)?;
            }
            *depth -= 1;
        }
    }
    Ok(())
}
