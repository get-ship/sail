//! Directive declarations — `<prop>`.
//!
//! These are lifted out of the template tree by the parser. The
//! emitter expands each one into the matching runtime call.
//!
//! Reactive component state is declared in the paired controller's
//! `setup:` factory; the `<state>` directive was retired in favour
//! of typed, testable controller scope. See `.ship/skills/ship-views`.

use crate::diagnostics::Span;

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
    /// reactive property (signal + `el.prop` accessor), set with plain
    /// `name={expr}`. The string is the verbatim TS type, carried to the `.d.ts`
    /// as `Signal<T>`. No reflection, no coercion — TypeScript stays TypeScript.
    Identity(String),
}
