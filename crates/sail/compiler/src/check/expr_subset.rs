//! Renderable-expression subset checker for `.sail` interpolations.
//!
//! Allowed: identifier reads; literals; property access incl. `?.`;
//! `??`, logical / arithmetic / comparison / ternary / unary `!`;
//! template literals (interpolations recurse); array + object literals;
//! zero-arg calls on idents / chains (signal reads); method calls with
//! args from `METHOD_ALLOWLIST`.
//!
//! Rejected: any other call, arrow / closure, `new`, `await`, `yield`,
//! `this`, assignment / update operators, regex literals, tagged
//! template literals, spread outside array / object literals.
//!
//! There is no JS AST — the check walks the source while tracking
//! string-literal state so keywords inside strings are ignored.

use crate::diagnostics::{CompileError, Span};
use crate::ir::expr::Expr;

/// Method names allowed when called with arguments.
///
/// Principle: side-effect-free, deterministic methods on primitives/collections
/// — no mutation, no I/O, no eval-shaped args. The list covers the natural
/// idioms agents reach for in templates (`count.toFixed(2)`, `name.startsWith(prefix)`,
/// `xs.includes(x)`, `xs.find(pred)`) so authors don't bounce through a controller
/// signal just to format a value.
///
/// Mutation-shaped methods (`push`, `pop`, `splice`, `sort`, `reverse`, `fill`,
/// `copyWithin`, …) deliberately stay out — they alias the receiver and would
/// silently mutate controller state from a render path. Forbidden keywords
/// (`new`, `await`, `yield`, `this`, …) are rejected on an independent path
/// in `check_call` / `FORBIDDEN_KEYWORDS`.
const METHOD_ALLOWLIST: &[&str] = &[
    "slice", "map", "filter", "includes", "toString", "length", "keys", "values", "entries",
    // Read-only Set / Map membership accessors — `selected.has(id)` /
    // `map.get(id)`. Referentially transparent, no I/O, same shape as the
    // already-allowlisted `includes` / `keys` / `values` / `entries`. Without
    // these, the natural canvas idiom `selectedSet().has(n.id)` rejects as a
    // bare `expr` CompileError with no rule-id (#263-B).
    "has", "get",
    // Deterministic formatters — referentially transparent, no I/O, no eval-shaped
    // arguments. Real agent friction: writing `count.toFixed(2)` in a template is
    // the natural shape and used to reject; documenting the workaround in SKILL is
    // strictly worse than admitting it.
    "toFixed",
    // String predicates / search — read-only, no I/O, no mutation.
    "startsWith", "endsWith", "indexOf", "lastIndexOf",
    // String formatters / shapers — pure, deterministic, no I/O.
    "trim", "trimStart", "trimEnd", "toLowerCase", "toUpperCase",
    "padStart", "padEnd", "repeat", "charAt", "charCodeAt", "concat",
    // Array / collection read-only access — same posture as the already-allowlisted
    // `map` / `filter`: callbacks are passed through verbatim, but the call sites
    // are restricted to method calls on identifiers / chains (not bare `expr`
    // calls), so the callback body itself is checked by the surrounding rules.
    "at", "join", "find", "findIndex", "some", "every", "flat",
];

/// Forbidden keywords that may appear bare in an expression.
const FORBIDDEN_KEYWORDS: &[&str] =
    &["new", "await", "yield", "this", "function", "delete", "void"];

/// Run the renderable subset check on a single expression. Returns a
/// span-bearing error on failure.
pub fn check(expr: &Expr) -> Result<(), CompileError> {
    let src = expr.source.trim();
    if src.is_empty() {
        return Ok(());
    }
    let bytes = src.as_bytes();
    let span = expr.span;
    let bail = |msg: String| Err(reject(span, msg, src));
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            i = skip_string(bytes, i, c);
            continue;
        }
        if c == b'`' {
            i = skip_template_literal(bytes, i, span)?;
            continue;
        }
        if is_ident_start(c) {
            let start = i;
            while i < bytes.len() && is_ident_cont(bytes[i]) {
                i += 1;
            }
            let word = &src[start..i];
            if FORBIDDEN_KEYWORDS.contains(&word) {
                return bail(format!("`{word}` is not allowed in renderable expressions"));
            }
            let mut k = i;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k < bytes.len() && bytes[k] == b'`' {
                return bail(
                    "tagged template literals are not allowed in renderable expressions".into(),
                );
            }
            if k < bytes.len() && bytes[k] == b'(' {
                check_call(src, bytes, start, k, span)?;
                i = k + 1;
                continue;
            }
            continue;
        }
        if c.is_ascii_digit() {
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.' || bytes[i] == b'_')
            {
                i += 1;
            }
            continue;
        }
        if c == b'=' {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
            if next == b'>' {
                return bail("arrow functions are not allowed in renderable expressions".into());
            }
            let is_compare = matches!(prev, b'=' | b'!' | b'<' | b'>') || next == b'=';
            if !is_compare {
                return bail("assignments are not allowed in renderable expressions".into());
            }
        }
        if (c == b'+' || c == b'-') && i + 1 < bytes.len() && bytes[i + 1] == c {
            return bail(
                "increment / decrement operators are not allowed in renderable expressions".into(),
            );
        }
        if c == b'.' && i + 2 < bytes.len() && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' {
            check_spread_position(bytes, i, span, src)?;
            i += 3;
            continue;
        }
        if c == b'/' && looks_like_regex(bytes, i) {
            return bail("regex literals are not allowed in renderable expressions".into());
        }
        i += 1;
    }
    Ok(())
}

/// Spread is only valid when the previous non-whitespace byte is
/// `[`, `{`, or `,` — i.e. inside an array / object literal.
fn check_spread_position(b: &[u8], at: usize, span: Span, src: &str) -> Result<(), CompileError> {
    let mut k = at;
    while k > 0 {
        k -= 1;
        let p = b[k];
        if p.is_ascii_whitespace() {
            continue;
        }
        if matches!(p, b'[' | b'{' | b',') {
            return Ok(());
        }
        return Err(reject(span, "spread is only allowed inside array or object literals".into(), src));
    }
    Ok(())
}

fn check_call(
    src: &str,
    bytes: &[u8],
    ident_start: usize,
    paren_at: usize,
    span: Span,
) -> Result<(), CompileError> {
    if is_empty_paren(bytes, paren_at) {
        return Ok(());
    }
    let name_end = ident_end_back(bytes, paren_at);
    let name = &src[ident_start..name_end];
    let mut k = ident_start;
    while k > 0 && bytes[k - 1].is_ascii_whitespace() {
        k -= 1;
    }
    let dotted = (k >= 1 && bytes[k - 1] == b'.')
        || (k >= 2 && bytes[k - 2] == b'?' && bytes[k - 1] == b'.');
    if dotted {
        if METHOD_ALLOWLIST.contains(&name) {
            return Ok(());
        }
        return Err(reject(
            span,
            format!(
                "method call `.{name}(…)` is not in the renderable subset (allowlist: {})",
                METHOD_ALLOWLIST.join(", ")
            ),
            src,
        ));
    }
    Err(reject(
        span,
        format!(
            "call `{name}(…)` with arguments is not allowed in renderable expressions; only zero-arg signal reads are permitted"
        ),
        src,
    ))
}

fn ident_end_back(bytes: &[u8], paren_at: usize) -> usize {
    let mut k = paren_at;
    while k > 0 && bytes[k - 1].is_ascii_whitespace() {
        k -= 1;
    }
    k
}

fn is_empty_paren(bytes: &[u8], paren_at: usize) -> bool {
    let mut k = paren_at + 1;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }
    k < bytes.len() && bytes[k] == b')'
}

fn skip_string(bytes: &[u8], from: usize, quote: u8) -> usize {
    let mut i = from + 1;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' {
            // Bounded skip: a trailing `\\` at end-of-input must not push
            // i past `bytes.len()` (callers slice with this value).
            i = (i + 2).min(bytes.len());
            continue;
        }
        if c == quote {
            return i + 1;
        }
        i += 1;
    }
    i
}

fn skip_template_literal(bytes: &[u8], from: usize, span: Span) -> Result<usize, CompileError> {
    let mut i = from + 1;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        if c == b'`' {
            return Ok(i + 1);
        }
        if c == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            let start = i + 2;
            let mut depth = 1i32;
            let mut k = start;
            while k < bytes.len() && depth > 0 {
                let cc = bytes[k];
                if cc == b'{' {
                    depth += 1;
                } else if cc == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                } else if cc == b'"' || cc == b'\'' {
                    k = skip_string(bytes, k, cc);
                    continue;
                } else if cc == b'`' {
                    k = skip_template_literal(bytes, k, span)?;
                    continue;
                }
                k += 1;
            }
            // Cap both ends — the inner loop can leave `k == bytes.len()`
            // (no closing `}`), and downstream nested skip_template_literal
            // returns can push i past the end via `i = k + 1`. Either way
            // the slice is bounded here.
            let body_end = k.min(bytes.len());
            let body_start = start.min(body_end);
            let body = std::str::from_utf8(&bytes[body_start..body_end]).map_err(|_| {
                CompileError::new("non-UTF8 expression source").with_category("expression")
            })?;
            check(&Expr::new(body, span, false))?;
            i = (k + 1).min(bytes.len());
            continue;
        }
        i += 1;
    }
    Ok(i)
}

/// `/` is a regex literal when in operand position — i.e. the previous
/// non-whitespace token isn't an identifier, number, `)`, or `]`.
fn looks_like_regex(bytes: &[u8], at: usize) -> bool {
    let mut k = at;
    while k > 0 {
        k -= 1;
        let p = bytes[k];
        if p.is_ascii_whitespace() {
            continue;
        }
        return matches!(
            p,
            b'=' | b',' | b'(' | b'?' | b':' | b'!' | b'&' | b'|' | b'<' | b'>'
                | b'+' | b'*' | b'%' | b'^' | b'~' | b'/' | b';' | b'{' | b'['
        );
    }
    true
}

use crate::scan::{is_ident_cont, is_ident_start};

fn reject(span: Span, msg: String, src: &str) -> CompileError {
    CompileError::new(msg)
        .with_span(span)
        .with_category(format!("expr `{}`", truncate(src, 60)))
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

#[cfg(test)]
#[path = "expr_subset_tests.rs"]
mod tests;
