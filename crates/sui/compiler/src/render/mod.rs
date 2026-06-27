//! SSR render walker — Phase D-1 (scaffold + static rendering).
//!
//! Walks the lowered IR and emits HTML with author-syntax markers per
//! the marker contract ADR (`docs/ship-adrs/.../sui-marker-contract.md`).
//!
//! Marker shapes:
//! * Dynamic attribute kinds emit the author attribute name directly,
//!   e.g. `on:click="bN"`, `bind:value="bN"`, `class:active="bN"`.
//! * Dynamic text (`{expr}`) emits `<!--sui:text:bN-->value<!--/sui-->`.
//! * Control flow (`{#if}`, `{#for}`, `{#match}`) emits a custom-element
//!   wrapper: `<sui-if key="bN">…</sui-if>` (active) or
//!   `<sui-if key="bN" hidden></sui-if>` (placeholder).
//!
//! No expression evaluation happens in this phase; every dynamic value
//! is a marker.

use crate::bindings;
use crate::diagnostics::CompileError;
use crate::ir::{AttrInstr, ElementInstr, ForInstr, IfInstr, Instr, MatchInstr, Program,
    WhileLoadingInstr, WhileErrorInstr};
use crate::keys::BindingRegistry;

mod control_flow;
pub(crate) mod eval;
mod island;
mod out;
mod render_for;
mod scope;
mod template_helpers;
mod walker;
mod walker_attrs;

#[cfg(test)]
mod tests;

pub use crate::escape::{escape_attr_value, escape_text_value, is_void_element};

/// Public entry point.
#[derive(Debug, Clone)]
pub struct RenderOpts {
    /// View name — used for diagnostics and (later) the JSON island id.
    pub view_name: String,
}

/// Output of a render pass.
#[derive(Debug, Clone)]
pub struct RenderOutput {
    /// Rendered HTML, including the `<script type="application/json"
    /// id="__set_init__">…</script>` island appended after the body.
    pub html: String,
    /// Structured access to the JSON-island contents — useful for
    /// tests and SSR consumers that don't want to re-parse the inline
    /// JSON.
    pub init: serde_json::Value,
}

/// Output of a "split" render — body HTML and initial scope returned
/// separately so callers that want to ship scope through a different
/// channel (e.g. as a field of a JSON payload) don't have to re-parse
/// the inline `<script id="__set_init__">` island out of the body.
#[derive(Debug, Clone)]
pub struct RenderSplit {
    /// Rendered HTML *without* the JSON-island script. Suitable for
    /// callers shipping scope as a separate field.
    pub html: String,
    /// Initial scope values as a JSON object. Same shape that the
    /// `<script id="__set_init__">` island carries in `RenderOutput.html`.
    /// Does NOT contain `__subscribe__` — the host (deck-core) injects that
    /// using `has_live_data` + its own route builders.
    pub init: serde_json::Value,
    /// Whether `<data live />` was declared in the program. When true the
    /// host must inject `__subscribe__` into the payload scope using its own
    /// `routes::subscribe_url` builder.
    pub has_live_data: bool,
}

/// Render a parsed program to HTML with author-syntax markers.
///
/// D-2 evaluates the renderable subset against the program's initial
/// scope and inlines the result into text/attribute output (markers
/// stay so the client runtime can re-bind). Expressions outside the
/// subset, or that touch unbound names, fall back to D-1's marker-
/// only shape.
///
/// Phase E-prep: the walker consumes a `BindingRegistry` built earlier
/// in `compile()` so its emitted `b<N>` keys index into the same
/// manifest the JS emitter writes out as `export const bindings`.
pub fn render(
    program: &Program,
    registry: &BindingRegistry,
    opts: &RenderOpts,
) -> Result<RenderOutput, CompileError> {
    // Body HTML + initial scope as separate values. `render_split` already
    // folds the `__subscribe__` URL into `init` for live-data views, so
    // there is exactly one place that decides the seed's shape.
    let split = render_split(program, registry, opts)?;
    // …then re-assemble: append the JSON-island script the SSR host page
    // and the marker contract expect after the body.
    let mut html = split.html;
    html.push_str(&island::emit(&split.init));
    Ok(RenderOutput { html, init: split.init })
}

/// Render to body HTML + initial scope, returned separately. Used by the
/// payload endpoint that ships scope as a JSON field rather than as an
/// embedded script island.
pub fn render_split(
    program: &Program,
    registry: &BindingRegistry,
    opts: &RenderOpts,
) -> Result<RenderSplit, CompileError> {
    let initial = scope::build(program);
    let mut state = walker::Walker::new_named(initial.scope, registry, opts.view_name.clone());
    state.walk_block(&program.template);
    let html = state.into_html();
    let has_live_data = program.data.iter().any(|d| d.live);
    // `__subscribe__` is NOT injected here. The host (deck-core) reads
    // `has_live_data` and injects the correct `/_deck/views/<name>/sse` URL
    // using its own route builders — the compiler must not bake a served URL.
    let init = initial.init;
    Ok(RenderSplit { html, init, has_live_data })
}

/// Convenience wrapper that builds the registry inline. Used by tests
/// and external callers that don't already have a registry handy.
pub fn render_default(
    program: &Program,
    opts: &RenderOpts,
) -> Result<RenderOutput, CompileError> {
    let registry = bindings::build(program);
    render(program, &registry, opts)
}

/// Re-export the shape of a single emit decision so the walker can
/// keep its match arms readable. Control-flow doesn't appear here —
/// it's classified by `control_flow_kind` on the `Instr` level, before
/// the walker reaches an element.
pub(crate) enum EmitDecision {
    /// Skip outright (e.g. `<data>`).
    Skip,
    /// Self-closing void element — emit `<tag …>` with no body / close.
    Void,
    /// Normal element — emit open, recurse, close.
    Normal,
}

/// Decide what to do with a top-level instruction kind. Pure helper —
/// keeps the walker's match readable.
pub(crate) fn decide_element(el: &ElementInstr) -> EmitDecision {
    let lower = el.tag.to_ascii_lowercase();
    if lower == "data" {
        return EmitDecision::Skip;
    }
    if is_void_element(&lower) {
        return EmitDecision::Void;
    }
    EmitDecision::Normal
}

/// Map an attribute-instruction variant onto the marker attribute name to
/// emit. Returns `None` for `Static`. The walker no longer routes through
/// this helper directly — each variant has its own emit branch — but it's
/// kept as the single source of truth exercised by the render-helper unit
/// tests.
#[allow(dead_code)]
pub(crate) fn marker_attr_name(attr: &AttrInstr) -> Option<String> {
    match attr {
        AttrInstr::Static { .. } => None,
        AttrInstr::Expr { name, .. } => Some(format!("attr:{name}")),
        AttrInstr::DynamicAttr { .. } => Some("attr-dynamic".to_string()),
        AttrInstr::BindValue { .. } => Some("bind:value".to_string()),
        AttrInstr::BindChecked { .. } => Some("bind:checked".to_string()),
        AttrInstr::BindThis { .. } => Some("bind:this".to_string()),
        AttrInstr::SlotReflectEmpty { .. } => None,
        AttrInstr::On { event, .. } => Some(format!("on:{event}")),
        AttrInstr::Class { name, .. } => Some(format!("class:{name}")),
        AttrInstr::Style { name, .. } => Some(format!("style:{name}")),
        AttrInstr::Prop { name, .. } => Some(format!("prop:{name}")),
        AttrInstr::Mutation { .. } => Some("mutation".to_string()),
        AttrInstr::Transition { .. } => None,
    }
}

/// Marker kind for control-flow instructions — matches the `<sui-{kind}>`
/// custom-element tag suffix.
#[allow(dead_code)]
pub(crate) fn control_flow_kind(instr: &Instr) -> Option<&'static str> {
    match instr {
        Instr::If(IfInstr { .. }) => Some("if"),
        Instr::For(ForInstr { .. }) => Some("for"),
        Instr::Match(MatchInstr { .. }) => Some("match"),
        Instr::WhileLoading(WhileLoadingInstr { .. }) => Some("while-loading"),
        Instr::WhileError(WhileErrorInstr { .. }) => Some("while-error"),
        _ => None,
    }
}
