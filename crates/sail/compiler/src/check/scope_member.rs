//! Type-strengthening #1: `ctx.X` / `props.X` member typo detection.
//!
//! `scope_bound` already validates that the controller's ctx-param name and
//! `props` etc. resolve as identifiers — but the byte walker treats `.X`
//! after the head as unrestricted property access. This pass parses each
//! interpolation expression with the grammar parser and flags any
//! head.member chain whose head is a known namespace but whose first
//! dotted segment is not a declared key.
//!
//! Closes a major typo class:
//! ```text
//! {ctx.fooo()}           // typo — fooo not a returned scope member
//! {props.titlee}         // typo — titlee not a declared <prop>
//! ```
//!
//! The old `setup.X` / `derived.X` namespace heads are retired along with
//! the object-literal sections they named: the controller's setup function
//! returns ONE flat scope object now, so `ctx` (the setup function's own
//! first parameter, whatever the author named it) is the only remaining
//! head worth checking — it exposes the ambient view scope + template
//! `<data>`/`<state>` signals, and its declared members are exactly the
//! returned scope member set (the same names a bare `{count}` reference
//! resolves against).

use crate::check::controller_parse::ParsedController;
use crate::check::symbol_table::SymbolTable;
use crate::diagnostics::CompileError;
use crate::ir::Program;
use crate::parse::expr::{parse_expression, parse_expression_with_arrows, Ast, AstKind};

pub fn check(
    program: &Program,
    controller: Option<&ParsedController>,
) -> Result<(), CompileError> {
    let table = SymbolTable::build(program, controller);
    let ctx_head = controller.map(|c| c.ctx_param.as_str()).unwrap_or("");
    super::visit_exprs(program, &mut |expr| check_one(expr, &table, ctx_head))
}

/// Collect EVERY invalid member access (one per expression), for the LSP /
/// agent multi-diagnostic surface. `check()` returns only the first.
pub fn collect(
    program: &Program,
    controller: Option<&ParsedController>,
) -> Vec<CompileError> {
    let table = SymbolTable::build(program, controller);
    let ctx_head = controller.map(|c| c.ctx_param.as_str()).unwrap_or("");
    let mut errs = Vec::new();
    // Closure always returns Ok so the walk visits every expression.
    let _ = super::visit_exprs(program, &mut |expr| {
        if let Err(e) = check_one(expr, &table, ctx_head) {
            errs.push(e);
        }
        Ok::<(), CompileError>(())
    });
    errs
}

fn check_one(
    expr: &crate::ir::Expr,
    table: &SymbolTable,
    ctx_head: &str,
) -> Result<(), CompileError> {
    let parsed = if expr.allow_arrow {
        parse_expression_with_arrows(&expr.source, expr.span.start)
    } else {
        parse_expression(&expr.source, expr.span.start)
    };
    let Ok(ast) = parsed else {
        // Parser couldn't model this shape (legacy permissive expr) —
        // skip; the lenient validators still police it.
        return Ok(());
    };
    walk(&ast, table, ctx_head)
}

fn walk(ast: &Ast, table: &SymbolTable, ctx_head: &str) -> Result<(), CompileError> {
    if let AstKind::Member { obj, prop, optional } = &ast.kind
        && let AstKind::Ident(head) = &obj.kind
        && let Some(err) = check_head_member(head, prop, *optional, table, ast, ctx_head) {
            return Err(err);
        }
    // Recurse via the AST walker.
    let mut error: Option<CompileError> = None;
    ast.walk(&mut |n| {
        if error.is_some() {
            return false;
        }
        if let AstKind::Member { obj, prop, optional } = &n.kind
            && let AstKind::Ident(head) = &obj.kind
            && let Some(err) = check_head_member(head, prop, *optional, table, n, ctx_head)
        {
            error = Some(err);
            return false;
        }
        true
    });
    match error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

fn check_head_member(
    head: &str,
    prop: &str,
    _optional: bool,
    table: &SymbolTable,
    node: &Ast,
    ctx_head: &str,
) -> Option<CompileError> {
    let (label, exists, available): (&str, bool, Vec<String>) = if !ctx_head.is_empty() && head == ctx_head {
        (
            ctx_head,
            table.scope.contains_key(prop),
            table.scope.keys().cloned().collect(),
        )
    } else if head == "props" {
        (
            "props",
            table.props.contains_key(prop),
            table.props.keys().cloned().collect(),
        )
    } else {
        return None;
    };
    if exists {
        return None;
    }
    if available.is_empty() {
        // No keys to compare against → can't fault the user.
        return None;
    }
    let mut sorted = available.clone();
    sorted.sort();
    Some(
        CompileError::new(format!(
            "`{label}.{prop}` is not a declared {label} key. Available: {}.",
            sorted.join(", ")
        ))
        .with_span(node.span)
        .with_category(format!("expr `{}`", truncate(prop, 60))),
    )
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max - 1).chain(std::iter::once('…')).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use std::path::PathBuf;

    fn empty_program() -> Program {
        Program {
            props: vec![],
            styles: String::new(),
            scope_types: String::new(),
            template: vec![],
            source_len: 0,
            span: crate::diagnostics::Span::point(0),
        }
    }

    fn ctrl(src: &str) -> ParsedController {
        parse(PathBuf::from("x.controller.ts"), src.to_string()).unwrap()
    }

    fn expr(src: &str) -> crate::ir::Expr {
        crate::ir::Expr::new(src, crate::diagnostics::Span::new(0, src.len()), false)
    }

    #[test]
    fn flags_ctx_dot_typo() {
        let c = ctrl("export default controller((ctx) => ({ count: signal(0) }));");
        let table = SymbolTable::build(&empty_program(), Some(&c));
        let e = expr("ctx.coun");
        let err = check_one(&e, &table, "ctx").unwrap_err();
        assert!(err.message.contains("ctx.coun"), "{}", err.message);
    }

    #[test]
    fn allows_known_scope_member() {
        let c = ctrl("export default controller((ctx) => ({ count: signal(0) }));");
        let table = SymbolTable::build(&empty_program(), Some(&c));
        let e = expr("ctx.count()");
        check_one(&e, &table, "ctx").unwrap();
    }

    #[test]
    fn flags_derived_dot_typo() {
        let c = ctrl(
            "export default controller((ctx) => ({ total: derived(() => 1) }));",
        );
        let table = SymbolTable::build(&empty_program(), Some(&c));
        let e = expr("ctx.totl");
        let err = check_one(&e, &table, "ctx").unwrap_err();
        assert!(err.message.contains("ctx.totl"), "{}", err.message);
    }
}
