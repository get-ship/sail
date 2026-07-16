//! Directive declarations — `<data>`, `<prop>`.
//!
//! These are lifted out of the template tree by the parser. The
//! emitter expands each one into the matching runtime call.
//!
//! Reactive component state is declared in the paired controller's
//! `setup:` factory; the `<state>` directive was retired in favour
//! of typed, testable controller scope. See `.ship/skills/ship-views`.

use crate::diagnostics::Span;
use crate::ir::expr::Expr;

/// `<client name="ship" />` — declares a host-injected data/ops client that the
/// view's `<data source>` expressions and `on:` handlers call (e.g.
/// `ship.git.overview()`). The host supplies the concrete client at mount; its
/// TYPE comes from a `declare module` augmentation of the runtime's `Clients`
/// interface. Declaring it legalizes the identifier for the scope-bound check —
/// the per-view, implementor-named replacement for the hardcoded ambient `ship`
/// (multiple `<client>`s = multiple data sources).
#[derive(Debug, Clone)]
pub struct ClientDecl {
    pub name: String,
    pub span: Span,
}

/// `<data name="agents" source="ship.agents.summaries()" live />`
#[derive(Debug, Clone)]
pub struct DataDecl {
    pub name: String,
    /// Source expression — read at hydrate time, parameters are
    /// `(ship, route)`. Not braces-stripped; the body is a plain JS
    /// expression literal lifted from the `source=` attribute.
    pub source: Expr,
    pub live: bool,
    pub span: Span,
}

/// Top-level `<prop>` directive in a component file.
#[derive(Debug, Clone)]
pub struct PropDecl {
    pub name: String,
    pub kind: PropKind,
    /// String form of the `default=` attribute; the emitter coerces
    /// this to the right JS literal based on `kind`.
    pub default: Option<String>,
    pub reflect: bool,
    pub bindable: bool,
    /// `<prop ... form-value />` — when set, the component is registered
    /// as form-associated and the runtime auto-wires the prop's signal
    /// to `ElementInternals.setFormValue` plus `formResetCallback` /
    /// `formStateRestoreCallback`.
    pub form_value: bool,
    /// `<prop ... form-disabled />` — when set, the runtime wires
    /// `formDisabledCallback(disabled)` to this (bool) prop's signal.
    pub form_disabled: bool,
    /// Author-supplied JSDoc description from a `<!-- /** ... */ -->`
    /// HTML comment immediately preceding the `<prop>` declaration.
    pub description: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropKind {
    String,
    Bool,
    Number,
    /// `values="default|primary"` — kept in source order.
    Union(Vec<String>),
    /// A prop whose type is arbitrary authored TypeScript (`config: { … }`,
    /// `columns: Column[]`, `data: DataPoint`). NOT an attribute — a plain
    /// reactive property (signal + `el.prop` accessor), set via `el.prop = …`
    /// or `prop:`. The string is the verbatim TS type, carried to the `.d.ts`
    /// as `Signal<T>`. No reflection, no coercion — TypeScript stays TypeScript.
    Identity(String),
}

