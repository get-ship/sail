//! Type-strengthening #5: `<for as="x" of={EXPR}>` iterable shape.
//!
//! `<for>` walks an iterable. Number, boolean, null, and undefined
//! literals can never satisfy that contract; the runtime currently
//! either renders zero items (silent) or throws on first iteration.
//! Object literals at the top of the iterable expression don't iterate
//! sensibly either (`for…of` would need entries / values explicitly).
//!
//! Rule: reject literal shapes that can't produce a sequence. Allow
//! everything else (identifiers, calls, member access, ternaries —
//! anything where the type might be array-shaped at runtime).

use crate::diagnostics::CompileError;
use crate::ir::{ForInstr, Instr, Program};
use crate::parse::expr::{AstKind, parse_expression};

pub fn check(program: &Program) -> Result<(), CompileError> {
    walk(&program.template)
}

fn walk(instrs: &[Instr]) -> Result<(), CompileError> {
    for instr in instrs {
        match instr {
            Instr::For(f) => check_for(f)?,
            Instr::If(i) => {
                for b in &i.branches {
                    walk(&b.body)?;
                }
                if let Some(eb) = &i.else_body {
                    walk(eb)?;
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    walk(&c.body)?;
                }
                if let Some(d) = &m.default {
                    walk(d)?;
                }
            }
            Instr::Element(el) => walk(&el.children)?,
            Instr::When(w) => {
                walk(&w.body)?;
                if let Some(e) = &w.else_body {
                    walk(e)?;
                }
            }
                        Instr::WhileLoading(wl) => walk(&wl.body)?,
            Instr::WhileError(we) => walk(&we.body)?,
            _ => {}
        }
    }
    Ok(())
}

fn check_for(f: &ForInstr) -> Result<(), CompileError> {
    let it = &f.iterable;
    if let Ok(ast) = parse_expression(&it.source, it.span.start)
        && let Some(reason) = non_iterable_reason(&ast.kind)
    {
        return Err(CompileError::new(format!("<for> iterable: {reason}"))
            .with_span(it.span)
            .with_category("for-iterable".to_string()));
    }
    walk(&f.body)?;
    if let Some(eb) = &f.else_body {
        walk(eb)?;
    }
    Ok(())
}

fn non_iterable_reason(kind: &AstKind) -> Option<&'static str> {
    match kind {
        AstKind::Number(_) => Some("a number is not iterable"),
        AstKind::Bool(_) => Some("a boolean is not iterable"),
        AstKind::Null => Some("`null` is not iterable"),
        AstKind::Undefined => Some("`undefined` is not iterable"),
        AstKind::Object(_) => Some(
            "object literals are not iterable — use `Object.keys(obj)` / `Object.entries(obj)` if you mean to iterate the keys",
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    use crate::ir::{Expr, Instr};

    fn for_with_iterable(src: &str) -> Program {
        let it = Expr::new(src, Span::new(0, src.len()), false);
        let f = ForInstr {
            var_name: "x".into(),
            index_name: None,
            iterable: it,
            key: None,
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
    fn rejects_number_iterable() {
        let err = check(&for_with_iterable("3")).unwrap_err();
        assert!(err.message.contains("number"), "{}", err.message);
    }

    #[test]
    fn rejects_object_iterable() {
        let err = check(&for_with_iterable("{ a: 1 }")).unwrap_err();
        assert!(err.message.contains("object literals"), "{}", err.message);
    }

    #[test]
    fn rejects_null_iterable() {
        let err = check(&for_with_iterable("null")).unwrap_err();
        assert!(err.message.contains("null"), "{}", err.message);
    }

    #[test]
    fn allows_identifier_iterable() {
        check(&for_with_iterable("xs")).unwrap();
    }

    #[test]
    fn allows_call_iterable() {
        check(&for_with_iterable("agents()")).unwrap();
    }
}
