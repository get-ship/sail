//! Type-strengthening #4: `on:event={X}` callable handler.
//!
//! This is the TEMPLATE `on:` directive (DOM event binding), unrelated to
//! the retired controller `on: {...}` section.
//!
//! Handlers must be either:
//!   - An arrow expression (`(e) => …`).
//!   - An identifier that resolves to a callable scope member — i.e. a
//!     returned scope member of arrow/function shape (`name: (e) => …`).
//!
//! Without this, `on:click={count}` typechecks because `count` is a
//! declared scope name; at runtime the renderer tries to invoke a
//! signal as a function and dies with `TypeError: count is not a
//! function`.

use crate::check::controller_parse::ParsedController;
use crate::check::symbol_table::SymbolTable;
use crate::diagnostics::CompileError;
use crate::ir::{AttrInstr, Instr, Program};
use crate::parse::expr::{AstKind, parse_expression_with_arrows};

const RULE_ID: &str = "sail-on-callable";
pub fn check(program: &Program, controller: Option<&ParsedController>) -> Result<(), CompileError> {
    let table = SymbolTable::build(program, controller);
    let ctx_head = controller.map(|c| c.ctx_param.as_str()).unwrap_or("");
    walk(&program.template, &table, ctx_head)
}

fn walk(instrs: &[Instr], table: &SymbolTable, ctx_head: &str) -> Result<(), CompileError> {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                for a in &el.attrs {
                    if let AttrInstr::On { event, handler, .. } = a {
                        validate(event, handler, table, ctx_head)?;
                    }
                }
                walk(&el.children, table, ctx_head)?;
            }
            Instr::If(i) => {
                for b in &i.branches {
                    walk(&b.body, table, ctx_head)?;
                }
                if let Some(eb) = &i.else_body {
                    walk(eb, table, ctx_head)?;
                }
            }
            Instr::For(f) => {
                walk(&f.body, table, ctx_head)?;
                if let Some(eb) = &f.else_body {
                    walk(eb, table, ctx_head)?;
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    walk(&c.body, table, ctx_head)?;
                }
                if let Some(d) = &m.default {
                    walk(d, table, ctx_head)?;
                }
            }
            Instr::When(w) => {
                walk(&w.body, table, ctx_head)?;
                if let Some(e) = &w.else_body {
                    walk(e, table, ctx_head)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate(
    event: &str,
    handler: &crate::ir::Expr,
    table: &SymbolTable,
    ctx_head: &str,
) -> Result<(), CompileError> {
    let Ok(ast) = parse_expression_with_arrows(&handler.source, handler.span.start) else {
        return Ok(());
    };
    match &ast.kind {
        AstKind::Arrow { .. } => Ok(()),
        // Calling an action: `on:click={scope.increment()}` — disallow,
        // because that runs at template-eval time, not at click time.
        AstKind::Call { .. } => Err(CompileError::new(format!(
            "on:{event}={{…}} cannot be a call expression — that fires the action at render time, not on the event. Use `on:{event}={{scope.handlerName}}` (no parens) or wrap in an arrow `(e) => scope.handlerName()`."
        ))
        .with_span(handler.span)
        .with_category(RULE_ID)),
        AstKind::Ident(name) => {
            if is_callable_name(name, table) {
                Ok(())
            } else {
                let kind_label = name_kind(name, table);
                Err(CompileError::new(format!(
                    "on:{event}={{{name}}} expects a callable handler — `{name}` is {kind_label}. Declare it as an `on:` action, a scope action `{name}: (e) => ...`, or wrap inline `(e) => ...`."
                ))
                .with_span(handler.span)
                .with_category(RULE_ID))
            }
        }
        AstKind::Member { obj, prop, .. } => {
            // `ctx.handlerName` — verify the head segment (the setup
            // function's own ctx param) routes to a callable scope key.
            if let AstKind::Ident(head) = &obj.kind
                && !ctx_head.is_empty()
                && head == ctx_head
            {
                match table.kind_of_scope(prop) {
                    Some(k) if k.is_callable() => Ok(()),
                    Some(k) => Err(CompileError::new(format!(
                        "on:{event}={{{ctx_head}.{prop}}} expects a callable — `{prop}` is a {} (declare it as `{prop}: (e) => ...`).",
                        k.label()
                    ))
                    .with_span(handler.span)
                    .with_category(RULE_ID)),
                    None => Ok(()),
                }
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

fn is_callable_name(name: &str, table: &SymbolTable) -> bool {
    if let Some(k) = table.kind_of_scope(name) {
        return k.is_callable();
    }
    // Common builtins.
    matches!(name, "navigate")
}

fn name_kind(name: &str, table: &SymbolTable) -> String {
    if let Some(k) = table.kind_of_scope(name) {
        return format!("a scope {} (not callable)", k.label());
    }
    "not declared".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use crate::diagnostics::Span;
    use crate::ir::{ElementInstr, Expr, Instr, Modifier};
    use std::path::PathBuf;

    fn ctrl(src: &str) -> ParsedController {
        parse(PathBuf::from("x.controller.ts"), src.to_string()).unwrap()
    }

    fn program_with_on(handler_src: &str) -> Program {
        let handler = Expr::new(handler_src, Span::new(0, handler_src.len()), true);
        let attr = AttrInstr::On {
            event: "click".into(),
            handler,
            modifiers: vec![] as Vec<Modifier>,
            span: Span::point(0),
        };
        let el = ElementInstr {
            tag: "button".into(),
            attrs: vec![attr],
            children: vec![],
            span: Span::point(0),
        };
        Program {
            props: vec![],
            styles: String::new(),
            scope_types: String::new(),
            template: vec![Instr::Element(el)],
            source_len: 0,
            span: Span::point(0),
        }
    }

    #[test]
    fn allows_arrow_handler() {
        let c = ctrl("export default controller((ctx) => ({ n: signal(0) }));");
        check(&program_with_on("(e) => n.set(1)"), Some(&c)).unwrap();
    }

    #[test]
    fn allows_scope_action_ident() {
        let c = ctrl("export default controller((ctx) => ({ inc: (e) => e }));");
        check(&program_with_on("inc"), Some(&c)).unwrap();
    }

    #[test]
    fn rejects_signal_as_handler() {
        let c = ctrl("export default controller((ctx) => ({ n: signal(0) }));");
        let err = check(&program_with_on("n"), Some(&c)).unwrap_err();
        assert!(err.message.contains("signal"), "{}", err.message);
    }

    #[test]
    fn rejects_call_expression_handler() {
        let c = ctrl("export default controller((ctx) => ({ inc: (e) => e }));");
        let err = check(&program_with_on("inc()"), Some(&c)).unwrap_err();
        assert!(err.message.contains("call expression"), "{}", err.message);
    }

    #[test]
    fn allows_scope_dot_action() {
        let c = ctrl("export default controller((ctx) => ({ inc: (e) => e }));");
        check(&program_with_on("ctx.inc"), Some(&c)).unwrap();
    }

    #[test]
    fn rejects_scope_dot_signal() {
        let c = ctrl("export default controller((ctx) => ({ n: signal(0) }));");
        let err = check(&program_with_on("ctx.n"), Some(&c)).unwrap_err();
        assert!(err.message.contains("signal"), "{}", err.message);
    }
}
