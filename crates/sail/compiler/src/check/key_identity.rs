//! Type-strengthening #2: `<for>` `key={…}` stable identity.
//!
//! The for-block's keyed reuse only works when the key has stable
//! identity per row — the same row maps to the same key every render.
//! Fresh array / object literals defeat this (each render is a fresh
//! allocation, so every row looks new), and primitive literal keys
//! (e.g. `key={0}`) collapse every row into one bucket.
//!
//! This pass parses `key={…}` and rejects:
//!   - Object / array literals as the entire key — guaranteed-fresh.
//!   - Bare literals (number / string / bool / null / undefined) —
//!     constant across rows so every row collides on the same key.
//!
//! What's allowed: identifier reads (`key={item.id}`), member access,
//! call expressions (`key={hash(item)}` would still reach this — the
//! subset blocks unallow-listed calls separately), template literals
//! that interpolate row data (`` key={`${item.id}-${i}`} ``).

use crate::diagnostics::CompileError;
use crate::ir::{ForInstr, Instr, Program};
use crate::parse::expr::{Ast, AstKind, parse_expression};

const RULE_ID: &str = "sail-for-key";
pub fn check(program: &Program) -> Result<(), CompileError> {
    walk_instrs(&program.template)
}

fn walk_instrs(instrs: &[Instr]) -> Result<(), CompileError> {
    for instr in instrs {
        match instr {
            Instr::For(f) => check_for(f)?,
            Instr::If(i) => {
                for b in &i.branches {
                    walk_instrs(&b.body)?;
                }
                if let Some(eb) = &i.else_body {
                    walk_instrs(eb)?;
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    walk_instrs(&c.body)?;
                }
                if let Some(d) = &m.default {
                    walk_instrs(d)?;
                }
            }
            Instr::Element(el) => walk_instrs(&el.children)?,
            Instr::When(w) => {
                walk_instrs(&w.body)?;
                if let Some(e) = &w.else_body {
                    walk_instrs(e)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_for(f: &ForInstr) -> Result<(), CompileError> {
    if let Some(key_expr) = &f.key {
        let Ok(ast) = parse_expression(&key_expr.source, key_expr.span.start) else {
            // Parser rejects this shape — fall through to other checks.
            return walk_instrs(&f.body);
        };
        if let Some(reason) = identity_problem(&ast) {
            return Err(CompileError::new(format!(
                "<for> key={{…}} must have stable per-row identity: {reason}"
            ))
            .with_span(key_expr.span)
            .with_category(RULE_ID));
        }
    }
    walk_instrs(&f.body)?;
    if let Some(eb) = &f.else_body {
        walk_instrs(eb)?;
    }
    Ok(())
}

fn identity_problem(ast: &Ast) -> Option<&'static str> {
    match &ast.kind {
        AstKind::Array(_) => Some(
            "array literals are always fresh — each render produces a different array, defeating row reuse",
        ),
        AstKind::Object(_) => Some(
            "object literals are always fresh — each render produces a different object, defeating row reuse",
        ),
        AstKind::Number(_)
        | AstKind::String(_)
        | AstKind::Bool(_)
        | AstKind::Null
        | AstKind::Undefined => Some(
            "literal keys collapse every row into the same bucket — use a per-row identifier (e.g. `item.id`)",
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    use crate::ir::Expr;

    fn for_with_key(key_src: &str) -> Program {
        let key = Expr::new(key_src, Span::new(0, key_src.len()), false);
        let iterable = Expr::new("xs", Span::new(0, 2), false);
        let f = ForInstr {
            var_name: "x".into(),
            index_name: None,
            iterable,
            key: Some(key),
            body: vec![],
            else_body: None,
            span: Span::point(0),
        };
        Program {
            props: vec![],
            styles: String::new(),
            scope_types: String::new(),
            template: vec![Instr::For(f)],
            source_len: 0,
            span: Span::point(0),
        }
    }

    #[test]
    fn rejects_object_literal_key() {
        let p = for_with_key("{ id: 1 }");
        let err = check(&p).unwrap_err();
        assert!(err.message.contains("object literals"), "{}", err.message);
    }

    #[test]
    fn rejects_array_literal_key() {
        let p = for_with_key("[1, 2]");
        let err = check(&p).unwrap_err();
        assert!(err.message.contains("array literals"), "{}", err.message);
    }

    #[test]
    fn rejects_bare_number_literal_key() {
        let p = for_with_key("42");
        let err = check(&p).unwrap_err();
        assert!(err.message.contains("literal keys"), "{}", err.message);
    }

    #[test]
    fn allows_member_access_key() {
        let p = for_with_key("x.id");
        check(&p).unwrap();
    }

    #[test]
    fn allows_template_literal_key() {
        let p = for_with_key("`${x.id}-${i}`");
        check(&p).unwrap();
    }
}
