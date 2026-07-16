//! Type-strengthening #1: `scope.X` / `derived.X` / `props.X` member typo
//! detection.
//!
//! `scope_bound` already validates that `scope`, `derived`, `props` etc.
//! resolve as identifiers — but the byte walker treats `.X` after the
//! head as unrestricted property access. This pass parses each
//! interpolation expression with the grammar parser and flags any
//! head.member chain whose head is a known controller section but
//! whose first dotted segment is not a declared key.
//!
//! Closes a major typo class:
//! ```text
//! {scope.fooo()}        // typo — fooo undeclared in scope
//! {derived.totalz}       // typo — totalz undeclared in derived
//! {props.titlee}         // typo — titlee not a declared <prop>
//! ```

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
    super::visit_exprs(program, &mut |expr| check_one(expr, &table))
}

/// Collect EVERY invalid member access (one per expression), for the LSP /
/// agent multi-diagnostic surface. `check()` returns only the first.
pub fn collect(
    program: &Program,
    controller: Option<&ParsedController>,
) -> Vec<CompileError> {
    let table = SymbolTable::build(program, controller);
    let mut errs = Vec::new();
    // Closure always returns Ok so the walk visits every expression.
    let _ = super::visit_exprs(program, &mut |expr| {
        if let Err(e) = check_one(expr, &table) {
            errs.push(e);
        }
        Ok::<(), CompileError>(())
    });
    errs
}

fn check_one(
    expr: &crate::ir::Expr,
    table: &SymbolTable,
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
    walk(&ast, table)
}

fn walk(ast: &Ast, table: &SymbolTable) -> Result<(), CompileError> {
    if let AstKind::Member { obj, prop, optional } = &ast.kind
        && let AstKind::Ident(head) = &obj.kind
        && let Some(err) = check_head_member(head, prop, *optional, table, ast) {
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
            && let Some(err) = check_head_member(head, prop, *optional, table, n)
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
) -> Option<CompileError> {
    let (label, exists, available): (&str, bool, Vec<String>) = match head {
        "setup" => (
            "setup",
            table.scope.contains_key(prop),
            table.scope.keys().cloned().collect(),
        ),
        "derived" => (
            "derived",
            table.derived.contains_key(prop),
            table.derived.keys().cloned().collect(),
        ),
        "props" => (
            "props",
            table.props.contains_key(prop),
            table.props.keys().cloned().collect(),
        ),
        _ => return None,
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
    fn flags_scope_dot_typo() {
        let c = ctrl(
            "export default controller({ setup: () => ({ count: signal(0) }) });",
        );
        let table = SymbolTable::build(&empty_program(), Some(&c));
        let e = expr("setup.coun");
        let err = check_one(&e, &table).unwrap_err();
        assert!(err.message.contains("setup.coun"), "{}", err.message);
    }

    #[test]
    fn allows_known_scope_member() {
        let c = ctrl(
            "export default controller({ setup: () => ({ count: signal(0) }) });",
        );
        let table = SymbolTable::build(&empty_program(), Some(&c));
        let e = expr("setup.count()");
        check_one(&e, &table).unwrap();
    }

    #[test]
    fn flags_derived_dot_typo() {
        let c = ctrl(
            "export default controller({ derived: { total: () => 1 } });",
        );
        let table = SymbolTable::build(&empty_program(), Some(&c));
        let e = expr("derived.totl");
        let err = check_one(&e, &table).unwrap_err();
        assert!(err.message.contains("derived.totl"), "{}", err.message);
    }
}
