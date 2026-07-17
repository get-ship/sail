//! Template IR — the instruction tree the emitter walks.
//!
//! Mirrors `packages/sail/dom/src/template/compile.ts`'s `Instr`
//! type. The emitter's job is to translate each instruction into the
//! matching runtime call.

use crate::diagnostics::Span;
use crate::ir::expr::Expr;

#[derive(Debug, Clone)]
pub enum Instr {
    StaticText { text: String, span: Span },
    DynamicText { expr: Expr },
    Comment { data: String, span: Span },
    Element(ElementInstr),
    If(IfInstr),
    For(ForInstr),
    Match(MatchInstr),
    WhileLoading(WhileLoadingInstr),
    WhileError(WhileErrorInstr),
}

#[derive(Debug, Clone)]
pub struct ElementInstr {
    /// Original tag name (preserved-case, may be a custom element).
    pub tag: String,
    pub attrs: Vec<AttrInstr>,
    pub children: Vec<Instr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileLoadingInstr {
    /// Signal name bound to (e.g. `"agents"`).
    pub bind: String,
    pub body: Vec<Instr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileErrorInstr {
    /// Signal name bound to (e.g. `"agents"`).
    pub bind: String,
    /// Local name for the error value inside the block body.
    pub error_var: String,
    pub body: Vec<Instr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AttrInstr {
    /// `attr="value"` — no expression markers.
    Static { name: String, value: String, span: Span },
    /// `attr={expr}` — single-expression attribute.
    Expr { name: String, expr: Expr, span: Span },
    /// `attr:{nameExpr}={valueExpr}` — dynamic attribute name.
    DynamicAttr { name_expr: Expr, expr: Expr, span: Span },
    /// `bind:value={signal}`.
    BindValue { expr: Expr, span: Span },
    /// `bind:checked={signal}`.
    BindChecked { expr: Expr, span: Span },
    /// `bind:this={varName}` — write a typed handle to the rendered
    /// element into the named scope variable. The expression must be a
    /// bare identifier; refs are write-once-from-runtime, not derivable.
    BindThis { var: String, span: Span },
    /// `<slot reflect-empty="data-attr">` — declarative slot-emptiness
    /// reactivity. Wires a `slotchange` listener that toggles the named
    /// attribute on the host whenever the slot's assigned content
    /// gains or loses visible text. Slot-only; component-only.
    SlotReflectEmpty { attr: String, span: Span },
    /// `on:event|mod[:arg]…={handler}`.
    On { event: String, handler: Expr, modifiers: Vec<Modifier>, span: Span },
    /// `class:name={cond}`.
    Class { name: String, expr: Expr, span: Span },
    /// `prop:foo={expr}` — assign the value to the element's JS
    /// property `foo` (camelCase). Use when passing complex objects
    /// (signals, arrays, structured data) to a child custom element.
    /// Compiles to `bindProp(el, "foo", () => expr())`.
    Prop { name: String, expr: Expr, span: Span },
    /// `mutation="ship.X.Y"` on a `<form>` — the SDK call chain.
    Mutation { expr: Expr, span: Span },
    /// `motion:trigger={...}` — declarative state-triggered animation.
    /// The config is a static object literal mapping CSS-style property
    /// names to animation preset names. Compiles to a pre-baked
    /// `element.animate(...)` call wired to the trigger's DOM events.
    Motion {
        trigger: String,
        props: Vec<MotionProp>,
        /// `stagger: '40ms'` — the per-sibling delay STEP, in milliseconds.
        /// Orchestration is what separates a list that appears from a list that
        /// arrives. It is a reserved key in the motion config, not a CSS property,
        /// because it describes the relationship BETWEEN elements rather than
        /// anything about one of them.
        stagger_ms: Option<u32>,
        span: Span,
    },
    /// `use:action` or `use:action={param}` — attach a reusable behavior
    /// (an "action") to the rendered element. `action` resolves to a value
    /// in scope (a hook handle that is itself use-able, or an imported
    /// action fn); `param` is the optional reactive parameter. Compiles to
    /// a runtime `runAction(el, action, () => param)` call that invokes the
    /// action with the element on mount, calls its `update(param)` when the
    /// param dependencies change, and `destroy()` on unmount. This is the
    /// first-class element→behavior primitive: it removes author-facing
    /// `ref` + `effect` for the whole element-behavior category.
    Use {
        action: String,
        param: Option<Expr>,
        span: Span,
    },
 }

/// One animated property within a `motion:` config. The preset name
/// resolves against `crates/sail/compiler/animation-presets.json` at
/// emit time. Multiple `MotionProp`s in the same `motion:` directive
/// share a trigger and fire from the same DOM event pair, but each
/// becomes its own `element.animate(...)` call.
#[derive(Debug, Clone, PartialEq)]
pub struct MotionProp {
    pub property: String,
    pub preset: String,
}

#[derive(Debug, Clone)]
pub enum Modifier {
    PreventDefault,
    StopPropagation,
    StopImmediatePropagation,
    SelfMod,
    Trusted,
    Key(Option<String>),
    Debounce(Option<String>),
    Throttle(Option<String>),
}

#[derive(Debug, Clone)]
pub struct IfInstr {
    pub branches: Vec<IfBranch>,
    pub else_body: Option<Vec<Instr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub cond: Expr,
    pub body: Vec<Instr>,
}

#[derive(Debug, Clone)]
pub struct ForInstr {
    pub var_name: String,
    pub index_name: Option<String>,
    pub iterable: Expr,
    pub key: Option<Expr>,
    pub body: Vec<Instr>,
    pub else_body: Option<Vec<Instr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchInstr {
    pub discriminant: Expr,
    pub cases: Vec<MatchCase>,
    pub default: Option<Vec<Instr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub test: Expr,
    pub body: Vec<Instr>,
}
