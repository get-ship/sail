//! Expression IR.
//!
//! We deliberately do **not** parse a full JS AST. The interpreter
//! validates expressions and then defers evaluation to `new Function`,
//! so the contract is "any JS expression except statements,
//! assignments (outside compares), and arrows (unless allowed)".
//!
//! The compiler enforces the same restrictions at build time and then
//! emits the original source byte-for-byte — wrapped in the right
//! runtime helper. That keeps semantics identical to the interpreter
//! and avoids a second JS parser.

use crate::diagnostics::Span;

/// Validated expression — the source text the user wrote, minus
/// surrounding `{` `}`, with `Span` pointing at its location.
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    /// Raw expression text. Already trimmed and brace-stripped.
    pub source: String,
    pub span: Span,
    /// Was the expression source allowed to contain an arrow?
    /// Used by `on:` handlers.
    pub allow_arrow: bool,
}

impl Expr {
    pub fn new(source: impl Into<String>, span: Span, allow_arrow: bool) -> Self {
        Self {
            source: source.into(),
            span,
            allow_arrow,
        }
    }
}
