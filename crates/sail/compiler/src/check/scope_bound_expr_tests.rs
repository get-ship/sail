//! Unit tests for `scope_bound_expr::check_expr`. Sibling module so
//! the implementation file stays under the line cap.

use std::collections::BTreeSet;

use super::scope_bound::{Scope, Visitor};
use super::scope_bound_expr::check_expr;
use crate::diagnostics::Span;
use crate::ir::Expr;

fn empty_visitor() -> Visitor {
    Visitor {
        scope: Scope { names: BTreeSet::new() },
        errors: Vec::new(),
    }
}

fn run(src: &str) -> Visitor {
    let expr = Expr::new(src, Span::new(0, src.len()), false);
    let mut v = empty_visitor();
    check_expr(&expr, &mut v, &[]);
    v
}

#[test]
fn object_literal_key_is_not_a_free_identifier() {
    // Regression: the keys of a directive opts object literal
    // (e.g. `fx:exit={{ duration: 180 }}`) must not be treated
    // as references to outer bindings. With an empty scope, the
    // check must pass.
    let v = run("{ duration: 180 }");
    assert!(v.errors.is_empty(), "expected no error, got: {:?}", v.errors);
}

#[test]
fn object_literal_multiple_keys_pass() {
    let v = run("{ duration: 100, axis: 'x', delay: 0 }");
    assert!(v.errors.is_empty(), "expected no error, got: {:?}", v.errors);
}

#[test]
fn nested_object_literal_keys_pass() {
    let v = run("{ duration: 120, easing: { in: 'ease-in', out: 'ease-out' } }");
    assert!(v.errors.is_empty(), "expected no error, got: {:?}", v.errors);
}

#[test]
fn unbound_value_still_flagged() {
    // Sanity: the heuristic only skips the *key* — values that
    // reference an unbound identifier are still rejected.
    let v = run("{ duration: missing }");
    assert!(!v.errors.is_empty(), "expected error for unbound `missing`");
    let msg = v.errors.into_iter().next().unwrap().message;
    assert!(msg.contains("missing"), "diag should name the missing ident: {msg}");
}
