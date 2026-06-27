//! Root-identifier extraction for binding dep-tracking.
//!
//! Walks the (one, real) expression AST collecting the root identifiers a
//! binding expression reads. Method / property names (`obj.PROP`), object-literal
//! keys, and arrow params do not count; a shorthand object entry (`{ foo }`)
//! does (it references `foo`). Expressions outside the renderable subset — e.g.
//! arrow handlers, which `parse_expression` rejects — fall back to a
//! keyword-aware word scan. Analysis only; there is no expression evaluator.

use crate::scan::{is_ident_cont, is_ident_start};

use super::ast::{Arg, ArrowBody, Ast, AstKind, ObjKey, TemplatePart};
use super::grammar::parse_expression;

/// Root identifiers a binding expression reads, in first-seen order.
pub fn collect_idents(src: &str) -> Vec<String> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    match parse_expression(trimmed, 0) {
        Ok(ast) => {
            let mut out = Vec::new();
            walk(&ast, &mut out);
            out
        }
        // Non-parseable (e.g. an arrow handler) — the registry pre-pass must not
        // fail compilation, so degrade to a word scan (Phase E binds it anyway).
        Err(_) => fallback_scan(trimmed),
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
        // Arrows are rejected by `parse_expression` (they reach `fallback_scan`),
        // so this arm is defensive: read the body, never the params.
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

/// Fallback word scan for non-parseable expressions. Picks ASCII
/// identifier-shaped tokens, drops keywords, drops anything following a `.`
/// (method / property names).
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
            while i < bytes.len() && is_ident_cont(bytes[i]) {
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

#[cfg(test)]
mod tests {
    use super::collect_idents;

    #[test]
    fn root_idents_only_through_member_and_call() {
        assert_eq!(collect_idents("agents().items.length"), vec!["agents".to_string()]);
        assert_eq!(collect_idents("count.set(1)"), vec!["count".to_string()]);
    }

    #[test]
    fn skips_literals_and_object_keys() {
        assert_eq!(collect_idents("true"), Vec::<String>::new());
        // `{ a: b }` reads `b`, not the key `a`.
        assert_eq!(collect_idents("{ a: b }"), vec!["b".to_string()]);
    }

    #[test]
    fn arrow_falls_back_to_word_scan() {
        let deps = collect_idents("() => count.set(count() + 1)");
        assert!(deps.contains(&"count".to_string()));
        assert!(!deps.contains(&"set".to_string()));
    }
}
