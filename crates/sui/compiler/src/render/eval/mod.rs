//! SSR expression evaluator — parses an expression from the renderable
//! subset and executes it against an initial scope.
//!
//! Anything outside the supported subset (arrow callbacks, identifier
//! calls with arguments, dynamic method names, …) returns
//! `EvalError::NonEvaluable` and the walker falls back to D-1's
//! marker-only output.

use std::collections::HashMap;

mod ast;
mod coerce;
mod exec;
mod lex;
mod methods;
mod parser;

#[cfg(test)]
mod tests;

pub use coerce::{strict_eq, truthy, to_string as stringify};
pub use parser::parse_expr;

pub use ast::Node;

/// Map of identifier name -> JSON value. Built once per render from
/// the program's `<data>` declarations plus the synthetic `route`
/// placeholder. Reactive state declared in the controller's `scope:`
/// factory has no SSR representation — those identifiers stay
/// unbound during static render.
pub type Scope = HashMap<String, serde_json::Value>;

/// Failure modes for the evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// Identifier was not in scope.
    Unbound(String),
    /// Operand had the wrong shape (e.g. property access on `null`).
    TypeMismatch(String),
    /// Outside the supported subset (e.g. a `.map` with a non-evaluable
    /// callback). Caller should fall back to marker-only output.
    NonEvaluable(String),
    /// Could not tokenize / parse the expression source.
    ParseError(String),
}

/// Parse and evaluate an expression source against the given scope.
pub fn eval(src: &str, scope: &Scope) -> Result<serde_json::Value, EvalError> {
    let node = parse_expr(src)?;
    exec::execute(&node, scope)
}

/// Collect the bare identifier roots an expression reads.
///
/// Used by the binding-manifest pre-pass to populate the `deps` array
/// the attach runtime needs for reactivity. We chain property accesses
/// up to the root: `agents().items.length` → `["agents"]`. Identifier
/// arguments to a call site (`add(x, y)`) count as reads of `x` and
/// `y`. Method names (the right-hand side of `obj.method(…)`) and
/// object-literal keys do not count.
///
/// On parse failure, falls back to a regex/word scan over the source —
/// the registry pre-pass must not fail compilation just because an
/// expression is outside the renderable subset (Phase E binds it
/// anyway).
pub fn collect_idents(src: &str) -> Vec<String> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    match parse_expr(trimmed) {
        Ok(node) => {
            let mut out: Vec<String> = Vec::new();
            walk_idents(&node, &mut out);
            out
        }
        Err(_) => fallback_scan(trimmed),
    }
}

fn walk_idents(node: &Node, out: &mut Vec<String>) {
    use ast::TplPart;
    match node {
        Node::Null | Node::Undefined | Node::Bool(_) | Node::Num(_) | Node::Str(_) => {}
        Node::Ident(name) => push_unique(out, name),
        Node::Template(parts) => {
            for part in parts {
                if let TplPart::Expr(e) = part {
                    walk_idents(e, out);
                }
            }
        }
        Node::Array(items) => {
            for it in items {
                walk_idents(it, out);
            }
        }
        Node::Object(props) => {
            for (_, v) in props {
                walk_idents(v, out);
            }
        }
        Node::Member { obj, .. } => walk_idents(obj, out),
        Node::Index { obj, idx, .. } => {
            walk_idents(obj, out);
            walk_idents(idx, out);
        }
        Node::Call { callee, args } => {
            walk_idents(callee, out);
            for a in args {
                walk_idents(a, out);
            }
        }
        Node::Unary { expr, .. } => walk_idents(expr, out),
        Node::Binary { l, r, .. } => {
            walk_idents(l, out);
            walk_idents(r, out);
        }
        Node::Logical { l, r, .. } => {
            walk_idents(l, out);
            walk_idents(r, out);
        }
        Node::Conditional { cond, then, els } => {
            walk_idents(cond, out);
            walk_idents(then, out);
            walk_idents(els, out);
        }
    }
}

fn push_unique(out: &mut Vec<String>, name: &str) {
    if matches!(name, "true" | "false" | "null" | "undefined") {
        return;
    }
    if !out.iter().any(|n| n == name) {
        out.push(name.to_string());
    }
}

/// Fallback word scan for non-parseable expressions. Picks ASCII
/// identifier-shaped tokens, drops keywords, drops anything that
/// follows a `.` (method names / property names).
fn fallback_scan(src: &str) -> Vec<String> {
    let bytes = src.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    let mut last_was_dot = false;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'.' {
            last_was_dot = true;
            i += 1;
            continue;
        }
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if is_ident_start(b) {
            let start = i;
            while i < bytes.len() && is_ident_continue(bytes[i]) {
                i += 1;
            }
            let name = &src[start..i];
            if !last_was_dot && !is_reserved_word(name) {
                push_unique(&mut out, name);
            }
            last_was_dot = false;
            continue;
        }
        last_was_dot = false;
        i += 1;
    }
    out
}

use crate::scan::{is_ident_cont as is_ident_continue, is_ident_start};

fn is_reserved_word(s: &str) -> bool {
    matches!(
        s,
        "true"
            | "false"
            | "null"
            | "undefined"
            | "void"
            | "typeof"
            | "instanceof"
            | "in"
            | "of"
            | "new"
            | "this"
            | "delete"
            | "return"
            | "function"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "default"
            | "break"
            | "continue"
    )
}
