//! Expression validator + grammar entry points.
//!
//! Two flavours of expression analysis live here:
//!
//! 1. [`validate`] — fast byte-level cross-check of the legacy rules
//!    from `packages/sui/dom/src/template/expr.ts` (no statements, no
//!    assignments outside compares / arrows, no empty expressions).
//!    The compiler emits the source byte-for-byte and pairs this
//!    pass with the renderable-subset check in `check::expr_subset`,
//!    so the runtime sees the same shape the validator approved.
//!
//! 2. [`grammar`] — a recursive-descent parser that produces an AST
//!    for the subset documented in `crates/sui/compiler/EXPRESSIONS.md`.
//!    Downstream type-strengthening checks (`check::scope_member`,
//!    `check::key_identity`, ...) and LSP intelligence walk the AST
//!    rather than re-implementing the byte-level scan five more
//!    times.
//!
//! Both flavours share span semantics — diagnostics surface byte
//! ranges into the original `.sui` source.

pub mod ast;
pub mod grammar;
pub mod idents;

pub use ast::{Arg, Ast, AstKind, ObjEntry, ObjKey, TemplatePart};
pub use grammar::{parse_expression, parse_expression_with_arrows, ParseError};
pub use idents::collect_idents;

use crate::diagnostics::{CompileError, Span};

pub fn validate(expr: &str, allow_arrow: bool, location: &str) -> Result<(), CompileError> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err(CompileError::new("empty expression")
            .with_category(location.to_string()));
    }
    let bytes = trimmed.as_bytes();
    let mut in_string: Option<u8> = None;
    let mut template_depth = 0i32;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        let prev = if i > 0 { bytes[i - 1] } else { 0 };
        let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };

        if let Some(q) = in_string {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_string = None;
            } else if q == b'`' && c == b'$' && next == b'{' {
                template_depth += 1;
                in_string = None;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            in_string = Some(c);
            i += 1;
            continue;
        }
        if c == b'{' {
            brace_depth += 1;
            i += 1;
            continue;
        }
        if c == b'}' {
            if template_depth > 0 && brace_depth == 0 {
                template_depth -= 1;
                in_string = Some(b'`');
                i += 1;
                continue;
            }
            brace_depth -= 1;
            i += 1;
            continue;
        }
        if c == b'(' {
            paren_depth += 1;
        }
        if c == b')' {
            paren_depth -= 1;
        }
        // Allow `;` inside block bodies (brace_depth > 0) so that
        // `() => { x.set(1); y.set(2) }` survives parse and reaches
        // the handler_complexity check pass with the right message.
        if c == b';' && brace_depth == 0 {
            return Err(CompileError::new(format!(
                "statements are not allowed in expressions: `{trimmed}`"
            ))
            .with_category(location.to_string()));
        }
        if c == b'=' && next == b'>' {
            if !allow_arrow {
                return Err(CompileError::new(format!(
                    "arrow functions are not allowed here: `{trimmed}`"
                ))
                .with_category(location.to_string()));
            }
            i += 2;
            continue;
        }
        if c == b'=' {
            let is_compare = matches!(prev, b'=' | b'!' | b'<' | b'>') || next == b'=';
            if !is_compare {
                return Err(CompileError::new(format!(
                    "assignments are not allowed in expressions: `{trimmed}`"
                ))
                .with_category(location.to_string()));
            }
        }
        i += 1;
    }
    if in_string.is_some() {
        return Err(CompileError::new(format!(
            "unterminated string in expression: `{trimmed}`"
        ))
        .with_category(location.to_string()));
    }
    let _ = paren_depth;
    Ok(())
}

/// Convenience wrapper that attaches a span to validation errors.
pub fn validate_at(
    expr: &str,
    allow_arrow: bool,
    location: &str,
    span: Span,
) -> Result<(), CompileError> {
    validate(expr, allow_arrow, location).map_err(|e| e.with_span(span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_call() {
        validate("count()", false, "x").unwrap();
    }

    #[test]
    fn rejects_assignment() {
        let err = validate("x = 1", false, "x").unwrap_err();
        assert!(err.message.contains("assignments"));
    }

    #[test]
    fn allows_compare() {
        validate("a == b", false, "x").unwrap();
        validate("a >= b", false, "x").unwrap();
        validate("a !== b", false, "x").unwrap();
    }

    #[test]
    fn rejects_statements() {
        let err = validate("x; y", false, "x").unwrap_err();
        assert!(err.message.contains("statements"));
    }

    #[test]
    fn rejects_arrow_by_default() {
        let err = validate("() => 1", false, "x").unwrap_err();
        assert!(err.message.contains("arrow"));
    }

    #[test]
    fn allows_arrow_when_enabled() {
        validate("(e) => x.set(e.target.value)", true, "x").unwrap();
    }

    #[test]
    fn ignores_inside_strings() {
        validate("'a; b'", false, "x").unwrap();
        validate("`a=>b`", false, "x").unwrap();
        validate("`a=${1}`", false, "x").unwrap();
    }

    #[test]
    fn rejects_empty() {
        let err = validate("   ", false, "x").unwrap_err();
        assert!(err.message.contains("empty"));
    }
}
