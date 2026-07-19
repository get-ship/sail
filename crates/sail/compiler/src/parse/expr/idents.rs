//! Root-identifier extraction for binding dep-tracking.
//!
//! Walks the expression AST collecting the root identifiers a binding
//! expression reads. Method / property names (`obj.PROP`), object-literal
//! keys, and arrow params do not count; a shorthand object entry (`{ foo }`)
//! does (it references `foo`). The parser chosen depends on the source: an
//! `on:` handler is parsed with arrows enabled; every other binding source
//! is parsed with the renderable subset. If neither parse succeeds, the dep
//! set is empty — the registry pre-pass must not guess. Analysis only; there
//! is no expression evaluator.

use super::ast::{Arg, ArrowBody, Ast, AstKind, ObjKey, TemplatePart};
use super::grammar::{parse_expression, parse_expression_with_arrows};

/// Root identifiers a binding expression reads, in first-seen order.
pub fn collect_idents(src: &str, allow_arrow: bool) -> Vec<String> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let parser = if allow_arrow {
        parse_expression_with_arrows
    } else {
        parse_expression
    };
    match parser(trimmed, 0) {
        Ok(ast) => {
            let mut out = Vec::new();
            walk(&ast, &mut out);
            out
        }
        // Non-parseable source — the registry pre-pass must not guess. Downstream
        // checks (expr_subset, prop-value validation) are the places that diagnose
        // a bad expression; the manifest simply records no dependencies.
        Err(_) => Vec::new(),
    }
}

fn walk(ast: &Ast, out: &mut Vec<String>) {
    match &ast.kind {
        AstKind::Number(_)
        | AstKind::String(_)
        | AstKind::Bool(_)
        | AstKind::Null
        | AstKind::Undefined => {}
        AstKind::Ident(name) => push_unique(out, name),
        AstKind::Template(parts) => {
            for p in parts {
                if let TemplatePart::Expr(e) = p {
                    walk(e, out);
                }
            }
        }
        // `obj.prop` — read `obj`, never the property name.
        AstKind::Member { obj, .. } => walk(obj, out),
        AstKind::Index { obj, idx, .. } => {
            walk(obj, out);
            walk(idx, out);
        }
        AstKind::Call { callee, args, .. } => {
            walk(callee, out);
            for a in args {
                walk_arg(a, out);
            }
        }
        AstKind::Unary { arg, .. } => walk(arg, out),
        AstKind::Binary { lhs, rhs, .. } | AstKind::Logical { lhs, rhs, .. } => {
            walk(lhs, out);
            walk(rhs, out);
        }
        AstKind::Ternary { cond, then, otherwise } => {
            walk(cond, out);
            walk(then, out);
            walk(otherwise, out);
        }
        AstKind::Array(items) => {
            for a in items {
                walk_arg(a, out);
            }
        }
        AstKind::Object(entries) => {
            for e in entries {
                match (&e.key, &e.value) {
                    // Shorthand `{ foo }` references `foo`.
                    (ObjKey::Ident(name), None) => push_unique(out, name),
                    (ObjKey::Spread(s), _) => walk(s, out),
                    _ => {}
                }
                if let Some(v) = &e.value {
                    walk(v, out);
                }
            }
        }
        // Reachable only when `allow_arrow: true`. Read the body, never the
        // params — a handler's deps are the scope variables it READS, not
        // the names it binds. (`on:click={(count) => …}` doesn't read
        // `count` from scope.)
        AstKind::Arrow { body, .. } => {
            if let ArrowBody::Expr(e) = body {
                walk(e, out);
            }
        }
    }
}

fn walk_arg(a: &Arg, out: &mut Vec<String>) {
    match a {
        Arg::Expr(e) | Arg::Spread(e) => walk(e, out),
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

#[cfg(test)]
mod tests {
    use super::collect_idents;

    #[test]
    fn root_idents_only_through_member_and_call() {
        assert_eq!(collect_idents("agents().items.length", false), vec!["agents".to_string()]);
        assert_eq!(collect_idents("count.set(1)", false), vec!["count".to_string()]);
    }

    #[test]
    fn skips_literals_and_object_keys() {
        assert_eq!(collect_idents("true", false), Vec::<String>::new());
        // `{ a: b }` reads `b`, not the key `a`.
        assert_eq!(collect_idents("{ a: b }", false), vec!["b".to_string()]);
    }

    #[test]
    fn arrow_handler_uses_arrow_parser() {
        // Handlers are parsed with arrows enabled. The body references `count`
        // (twice — dedup'd); `set` is a property name and must not appear.
        let deps = collect_idents("() => count.set(count() + 1)", true);
        assert!(deps.contains(&"count".to_string()));
        assert!(!deps.contains(&"set".to_string()));
    }

    #[test]
    fn non_parseable_source_yields_empty_deps() {
        // When a source cannot be parsed in its allowed subset, the dep set is
        // empty rather than a guessed set. (A bare `=>` is not a valid expression.)
        assert_eq!(collect_idents("=> foo", false), Vec::<String>::new());
    }
}
