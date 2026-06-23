//! Template IR — the instruction tree the emitter walks.
//!
//! Mirrors `packages/sui/dom/src/template/compile.ts`'s `Instr`
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
    /// `{let <name> = <value> in ...body}` — internal-only instruction
    /// emitted by `{#async X as v}` to bind a snapshot of the signal's
    /// `.value` inside the `{:ok}` body. Not author-visible syntax.
    Let(LetInstr),
}

#[derive(Debug, Clone)]
pub struct LetInstr {
    pub name: String,
    pub value: Expr,
    pub body: Vec<Instr>,
    pub span: Span,
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
    /// `style:--var={value}` (or `style:color`).
    Style { name: String, expr: Expr, span: Span },
    /// `prop:foo={expr}` — assign the value to the element's JS
    /// property `foo` (camelCase). Use when passing complex objects
    /// (signals, arrays, structured data) to a child custom element.
    /// Compiles to `bindProp(el, "foo", () => expr())`.
    Prop { name: String, expr: Expr, span: Span },
    /// `mutation="ship.X.Y"` on a `<form>` — the SDK call chain.
    Mutation { expr: Expr, span: Span },
    /// `transition:NAME={cfg}`, `in:NAME={cfg}`, `out:NAME={cfg}`.
    /// `name` is the directive identifier (resolved via the runtime
    /// import set or `$scope`); `opts` is the optional `{cfg}`
    /// expression; `dir` selects the enter / exit / both phases.
    Transition {
        name: String,
        opts: Option<Expr>,
        dir: TransitionDir,
        span: Span,
    },
}

/// Which phase(s) a `transition:` directive applies to. `transition:`
/// is both; `in:` is enter-only; `out:` is exit-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionDir {
    Both,
    In,
    Out,
}

impl TransitionDir {
    pub fn as_str(self) -> &'static str {
        match self {
            TransitionDir::Both => "both",
            TransitionDir::In => "in",
            TransitionDir::Out => "out",
        }
    }
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
