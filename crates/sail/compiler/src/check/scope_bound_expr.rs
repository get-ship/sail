//! Free-identifier extraction from a single template expression.
//!
//! Extracted from `scope_bound` to keep that module under the file
//! cap. The single public entry point — [`check_expr`] — walks an
//! `Expr`'s source text, tracks string / template / arrow-param
//! state, and reports the first identifier that doesn't resolve in
//! the visitor's scope.

use super::js_scan::{is_ident_cont, is_ident_start, skip_balanced};
use super::scope_bound::Visitor;
use crate::diagnostics::{CompileError, Span};
use crate::ir::Expr;

/// Walk `expr.source` and flag the first identifier that doesn't
/// resolve. `extra_locals` are additional bound names from a parent
/// scope (e.g. a `{#for}` item).
pub(super) fn check_expr(expr: &Expr, v: &mut Visitor, extra_locals: &[&str]) {
    check_expr_inner(expr, v, extra_locals, None)
}

/// Same walk as [`check_expr`], but `use_action` names the `use:` action
/// this expression is the VALUE of (`use:worldTransform={expr}` passes
/// `Some("worldTransform")`). Inside that value expression a bare
/// reference to the action's own name is always the canvas-detail-crash
/// shape — see the `use_action` handling below — so it gets a dedicated
/// diagnostic before the general scope/collision checks run.
pub(super) fn check_use_value_expr(action: &str, expr: &Expr, v: &mut Visitor) {
    check_expr_inner(expr, v, &[], Some(action))
}

fn check_expr_inner(expr: &Expr, v: &mut Visitor, extra_locals: &[&str], use_action: Option<&str>) {
    // Collect across the whole template (the outer visitor no longer
    // short-circuits), but at most one diagnostic per expression: once THIS
    // call has pushed an error, stop scanning the rest of this expression.
    let start = v.errors.len();
    let bytes = expr.source.as_bytes();
    let mut i = 0;
    let mut prev_dot = false;
    // Last non-whitespace, non-comment byte. Drives the object-key
    // heuristic: `{ ident:` and `, ident:` mean `ident` is a property
    // name, not a free identifier.
    let mut prev_meaningful: Option<u8> = None;
    let mut in_str: Option<u8> = None;
    let mut local_stack: Vec<String> = extra_locals.iter().map(|s| s.to_string()).collect();
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            in_str = Some(c);
            i += 1;
            prev_dot = false;
            continue;
        }
        if c == b'`' {
            i = scan_template_literal(&expr.source, i, expr.span, v, &local_stack, use_action);
            if v.errors.len() > start {
                return;
            }
            prev_dot = false;
            continue;
        }
        if c == b'.' {
            prev_dot = true;
            prev_meaningful = Some(b'.');
            i += 1;
            continue;
        }
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if !is_ident_start(c) {
            if c == b'('
                && let Some(after_arrow) = peek_arrow_params(&expr.source, i, &mut local_stack)
            {
                i = after_arrow;
                prev_dot = false;
                prev_meaningful = Some(b')');
                continue;
            }
            prev_dot = false;
            prev_meaningful = Some(c);
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let word = &expr.source[start..i];
        let was_after_brace_or_comma = matches!(prev_meaningful, Some(b'{') | Some(b','));
        prev_meaningful = Some(b'A'); // identifier: treat as alpha-token
        if is_keyword(word) {
            prev_dot = false;
            continue;
        }
        // After this identifier, look ahead — if `=>` follows, this
        // is a single-ident arrow (`x => x + 1`); bind `x` as a local.
        if peek_after(&expr.source, i, b'=', b'>') {
            local_stack.push(word.to_string());
            prev_dot = false;
            continue;
        }
        if prev_dot {
            // Property access: `foo.bar` — `bar` is unrestricted.
            prev_dot = false;
            continue;
        }
        // Object-literal key: `{ key:` or `, key:` (single `:`, not
        // `::` which is reserved). Property names don't reference an
        // outer binding — skip the scope check.
        if was_after_brace_or_comma && peek_colon(&expr.source, i) {
            prev_dot = false;
            continue;
        }
        let shadowed_by_local = local_stack.iter().any(|s| s == word);
        // The exact canvas-detail-crash shape: inside `use:X={expr}`, a bare
        // `X` in `expr` NEVER means the scope value the author probably
        // intended — the emitter's expr-rewrite leaves built-in action
        // names bare (see `emit::expr_rewrite::BUILTINS`), so this compiles
        // to the ACTION IMPORT called with zero arguments, not a `$scope.X`
        // read. Refuse unconditionally rather than let it through when
        // there happens to be no colliding scope member today — renaming
        // the action's own bare name into a scope value is never valid.
        if let Some(action) = use_action
            && word == action
            && !shadowed_by_local
        {
            v.errors.push(
                CompileError::new(format!(
                    "`use:{action}={{{src}}}` calls '{action}' bare inside its own value \
                     expression — that name always resolves to the `use:{action}` action \
                     import itself (invoked with zero arguments), never to a scope value, \
                     even when scope declares a same-named binding. This is the \
                     canvas-detail-crash shape: an argless self-call where the action \
                     expects (element, param) and gets neither, then throws reading off an \
                     undefined node. If you meant to read a scope value here, give it a \
                     name other than '{action}' so it no longer collides with the action \
                     import.",
                    src = expr.source,
                ))
                .with_span(expr.span)
                .with_category(format!("use:{action} value `{}`", expr.source)),
            );
            return;
        }
        // General precedence rule for scope-vs-import name lookups: a
        // built-in `use:` action import always wins over a same-named
        // scope member in a template expression (see
        // `emit::expr_rewrite::BUILTINS` — it leaves action names bare
        // rather than qualifying them with `$scope.`, since a `use:`
        // directive needs the real ESM binding to pass to `runAction`).
        // A same-named scope member is therefore permanently unreachable
        // from any template expression while the collision exists — never
        // a case where "whichever the author meant" is a safe guess.
        // Refuse instead of silently shadowing.
        if !shadowed_by_local
            && v.scope.names.contains(word)
            && crate::lower::bindings::BUILTIN_ACTIONS.contains(&word)
        {
            v.errors.push(
                CompileError::new(format!(
                    "Identifier '{word}' is ambiguous: this component's scope declares a \
                     value named '{word}' (from setup/props), and '{word}' is also a \
                     built-in `use:` action auto-imported from @ship/sail-runtime. \
                     Template expressions resolve the action import first, so this \
                     reference reads the ACTION, not your scope value — the compiler \
                     refuses rather than silently pick one. Rename the scope binding (or \
                     stop using `use:{word}` under this name) so the two no longer share \
                     an identifier.",
                ))
                .with_span(expr.span)
                .with_category(format!("expression `{}`", expr.source)),
            );
            return;
        }
        if !v.scope.contains(word) && !shadowed_by_local {
            // `route` is host-granted, never ambient — the likeliest reach for
            // it outside a route file deserves the real explanation, not a
            // generic unknown-name error.
            let hint = if word == "route" {
                " `route` is provided by the host to route files only — this file is \
                 not registered as a route (deck routes by file location)."
            } else {
                ""
            };
            v.errors.push(
                CompileError::new(format!(
                    "Identifier '{}' is not declared in scope. Available: {}.{hint}",
                    word,
                    v.scope.list_for_diagnostic()
                ))
                .with_span(expr.span)
                .with_category(format!("expression `{}`", expr.source)),
            );
            return;
        }
        prev_dot = false;
    }
}

fn peek_after(src: &str, i: usize, c1: u8, c2: u8) -> bool {
    let bytes = src.as_bytes();
    let mut k = i;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }
    k + 1 < bytes.len() && bytes[k] == c1 && bytes[k + 1] == c2
}

/// True iff the next non-whitespace byte after `i` is a single `:`
/// (and not the start of `::`).
fn peek_colon(src: &str, i: usize) -> bool {
    let bytes = src.as_bytes();
    let mut k = i;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }
    k < bytes.len() && bytes[k] == b':' && (k + 1 >= bytes.len() || bytes[k + 1] != b':')
}

/// We're at a `(`; if this looks like an arrow param list (`(a, b) =>
/// …`), push the param idents into `locals` and return the offset
/// just past the `)`. Otherwise return `None`.
fn peek_arrow_params(src: &str, i: usize, locals: &mut Vec<String>) -> Option<usize> {
    let bytes = src.as_bytes();
    let close = skip_balanced(bytes, i, bytes.len());
    if close <= i || close > bytes.len() {
        return None;
    }
    let mut k = close;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }
    let mut is_arrow = k + 1 < bytes.len() && bytes[k] == b'=' && bytes[k + 1] == b'>';
    if !is_arrow && k < bytes.len() && bytes[k] == b':' {
        // Skip TS return type — find the next `=>` at depth zero.
        let mut depth = 0i32;
        let mut j = k + 1;
        while j + 1 < bytes.len() {
            match bytes[j] {
                b'(' | b'<' | b'[' | b'{' => depth += 1,
                b')' | b'>' | b']' | b'}' => depth -= 1,
                b'=' if depth == 0 && bytes[j + 1] == b'>' => {
                    is_arrow = true;
                    break;
                }
                _ => {}
            }
            j += 1;
        }
    }
    if !is_arrow {
        return None;
    }
    let inner_start = i + 1;
    let inner_end = close.saturating_sub(1);
    harvest_param_idents(src, inner_start, inner_end, locals);
    Some(close)
}

fn harvest_param_idents(src: &str, start: usize, end: usize, into: &mut Vec<String>) {
    let bytes = src.as_bytes();
    let mut i = start;
    let mut paren = 0i32;
    let mut in_str: Option<u8> = None;
    let mut skip_until_separator = false;
    while i < end {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
                i += 1;
            }
            b'(' | b'<' => {
                paren += 1;
                i += 1;
            }
            b')' | b'>' => {
                paren -= 1;
                i += 1;
            }
            b':' | b'=' if paren == 0 => {
                skip_until_separator = true;
                i += 1;
            }
            b',' if paren == 0 => {
                skip_until_separator = false;
                i += 1;
            }
            b'{' | b'[' if !skip_until_separator => {
                let close = skip_balanced(bytes, i, end);
                harvest_destructure_idents(src, i + 1, close.saturating_sub(1), into);
                i = close;
            }
            _ if !skip_until_separator && is_ident_start(c) => {
                let s = i;
                while i < end && is_ident_cont(bytes[i]) {
                    i += 1;
                }
                let w = &src[s..i];
                if !is_keyword(w) {
                    into.push(w.to_string());
                }
                skip_until_separator = true;
            }
            _ => i += 1,
        }
    }
}

fn harvest_destructure_idents(src: &str, start: usize, end: usize, into: &mut Vec<String>) {
    let bytes = src.as_bytes();
    let mut i = start;
    let mut last_ident: Option<String> = None;
    let mut after_colon = false;
    while i < end {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b':' {
            after_colon = true;
            i += 1;
            continue;
        }
        if c == b',' {
            if !after_colon && let Some(w) = last_ident.take() {
                into.push(w);
            }
            after_colon = false;
            last_ident = None;
            i += 1;
            continue;
        }
        if c == b'{' || c == b'[' {
            let close = skip_balanced(bytes, i, end);
            harvest_destructure_idents(src, i + 1, close.saturating_sub(1), into);
            i = close;
            after_colon = false;
            last_ident = None;
            continue;
        }
        if is_ident_start(c) {
            let s = i;
            while i < end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            let w = src[s..i].to_string();
            if after_colon {
                into.push(w);
                last_ident = None;
                after_colon = false;
            } else {
                last_ident = Some(w);
            }
            continue;
        }
        i += 1;
    }
    if !after_colon && let Some(w) = last_ident {
        into.push(w);
    }
}

fn scan_template_literal(
    src: &str,
    start: usize,
    span: Span,
    v: &mut Visitor,
    locals: &[String],
    use_action: Option<&str>,
) -> usize {
    let bytes = src.as_bytes();
    debug_assert_eq!(bytes[start], b'`');
    let mut i = start + 1;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' {
            i += 2;
            continue;
        }
        if c == b'`' {
            return i + 1;
        }
        if c == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            let close = skip_balanced(bytes, i + 1, bytes.len());
            let inner = &src[i + 2..close.saturating_sub(1)];
            let sub = Expr::new(inner, span, true);
            check_expr_inner(
                &sub,
                v,
                locals
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .as_slice(),
                use_action,
            );
            i = close;
            continue;
        }
        i += 1;
    }
    bytes.len()
}

fn is_keyword(w: &str) -> bool {
    matches!(
        w,
        "if" | "else"
            | "return"
            | "let"
            | "const"
            | "var"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "typeof"
            | "instanceof"
            | "in"
            | "of"
            | "new"
            | "true"
            | "false"
            | "null"
            | "undefined"
            | "async"
            | "await"
            | "yield"
            | "function"
            | "class"
            | "this"
            | "super"
            | "void"
            | "delete"
    )
}
