//! Free-identifier scan for the synthetic-controller emit: does a template
//! (or a `<data source>`) reference a given manifest name? A word-boundary
//! match so `route` hits in `route.params` but not in `myroute`, and a
//! leading `.` is treated as a member access (not a free root).

use crate::ir::*;

/// Does any template expression or `<data source>` reference `name`?
pub(super) fn references_ident(program: &Program, name: &str) -> bool {
    program.data.iter().any(|d| expr_references_ident(&d.source.source, name))
        || template_references_ident(&program.template, name)
}

fn template_references_ident(instrs: &[Instr], name: &str) -> bool {
    instrs.iter().any(|i| instr_references_ident(i, name))
}

fn instr_references_ident(instr: &Instr, name: &str) -> bool {
    match instr {
        Instr::DynamicText { expr } => expr_references_ident(&expr.source, name),
        Instr::Element(el) => {
            el.attrs.iter().any(|a| attr_references_ident(a, name))
                || template_references_ident(&el.children, name)
        }
        Instr::If(IfInstr { branches, else_body, .. }) => {
            branches.iter().any(|b| {
                expr_references_ident(&b.cond.source, name)
                    || template_references_ident(&b.body, name)
            }) || else_body.as_ref().is_some_and(|b| template_references_ident(b, name))
        }
        Instr::For(ForInstr { iterable, key, body, else_body, .. }) => {
            expr_references_ident(&iterable.source, name)
                || key.as_ref().is_some_and(|k| expr_references_ident(&k.source, name))
                || template_references_ident(body, name)
                || else_body.as_ref().is_some_and(|b| template_references_ident(b, name))
        }
        Instr::Match(MatchInstr { discriminant, cases, default, .. }) => {
            expr_references_ident(&discriminant.source, name)
                || cases.iter().any(|c| {
                    expr_references_ident(&c.test.source, name)
                        || template_references_ident(&c.body, name)
                })
                || default.as_ref().is_some_and(|d| template_references_ident(d, name))
        }
        Instr::WhileLoading(wl) => template_references_ident(&wl.body, name),
        Instr::WhileError(we) => template_references_ident(&we.body, name),
        Instr::Let(l) => {
            expr_references_ident(&l.value.source, name)
                || template_references_ident(&l.body, name)
        }
        Instr::StaticText { .. } | Instr::Comment { .. } => false,
    }
}

fn attr_references_ident(attr: &AttrInstr, name: &str) -> bool {
    match attr {
        AttrInstr::Expr { expr, .. }
        | AttrInstr::BindValue { expr, .. }
        | AttrInstr::BindChecked { expr, .. }
        | AttrInstr::Class { expr, .. }
        | AttrInstr::Style { expr, .. }
        | AttrInstr::Prop { expr, .. }
        | AttrInstr::Mutation { expr, .. } => expr_references_ident(&expr.source, name),
        AttrInstr::On { handler, .. } => expr_references_ident(&handler.source, name),
        AttrInstr::DynamicAttr { name_expr, expr, .. } => {
            expr_references_ident(&name_expr.source, name)
                || expr_references_ident(&expr.source, name)
        }
        AttrInstr::Transition { opts, .. } => {
            opts.as_ref().is_some_and(|e| expr_references_ident(&e.source, name))
        }
        AttrInstr::Use { action, param, .. } => {
            action == name
                || param.as_ref().is_some_and(|e| expr_references_ident(&e.source, name))
        }
        AttrInstr::Static { .. }
        | AttrInstr::BindThis { .. }
        | AttrInstr::SlotReflectEmpty { .. }
        | AttrInstr::Motion { .. } => false,
    }
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Word-boundary scan for `name` in `src`. A leading `.` disqualifies a
/// match (it would be a member access, not a free root identifier).
/// String literals are skipped — `class:active={x === 'ship-light'}`
/// would otherwise spuriously import the `ship` client (gap G3, slice
/// 2). Template literals (backticks) are likewise skipped: the host
/// surface never references template-literal contents as a free
/// identifier.
fn expr_references_ident(src: &str, name: &str) -> bool {
    let bytes = src.as_bytes();
    let target = name.as_bytes();
    let tlen = target.len();
    if tlen == 0 || bytes.len() < tlen {
        return false;
    }
    // Identify "in-string" / "in-template" / "in-line-comment" spans so
    // the scan skips them. The skip map is a `Vec<bool>` parallel to
    // `bytes` — `skip[i]` is true when `bytes[i]` is inside a literal
    // or comment and therefore not a candidate for a match.
    let mut skip = vec![false; bytes.len()];
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' || b == b'\'' {
            // String literal — skip until matching close (no escape
            // handling needed; a `\` before the close quote is rare in
            // SUI expressions and the false positive is bounded).
            let quote = b;
            i += 1;
            while i < bytes.len() && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    skip[i] = true;
                    skip[i + 1] = true;
                    i += 2;
                } else {
                    skip[i] = true;
                    i += 1;
                }
            }
            if i < bytes.len() {
                // The close quote itself is part of the literal span.
                skip[i] = true;
                i += 1;
            }
        } else if b == b'`' {
            // Template literal — same skip rule. `${...}` interpolations
            // are still scanned (they can reference manifest names),
            // so we toggle the skip state at every `{` / `}`.
            i += 1;
            let mut depth: i32 = 0;
            while i < bytes.len() {
                let c = bytes[i];
                if c == b'\\' && i + 1 < bytes.len() {
                    skip[i] = true;
                    skip[i + 1] = true;
                    i += 2;
                    continue;
                }
                if depth == 0 && c == b'`' {
                    skip[i] = true;
                    i += 1;
                    break;
                }
                if c == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    skip[i] = true;
                    skip[i + 1] = true;
                    depth += 1;
                    i += 2;
                    continue;
                }
                if depth > 0 && c == b'}' {
                    skip[i] = true;
                    depth -= 1;
                    i += 1;
                    continue;
                }
                if depth == 0 {
                    skip[i] = true;
                }
                i += 1;
            }
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            // Line comment — skip until newline.
            while i < bytes.len() && bytes[i] != b'\n' {
                skip[i] = true;
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    for i in 0..=bytes.len() - tlen {
        if &bytes[i..i + tlen] != target {
            continue;
        }
        // Match is rejected if any byte in the candidate span is in a
        // literal/comment.
        if skip[i..i + tlen].iter().any(|&s| s) {
            continue;
        }
        let ok_before = i == 0 || (!is_ident_char(bytes[i - 1]) && bytes[i - 1] != b'.');
        let ok_after = i + tlen >= bytes.len() || !is_ident_char(bytes[i + tlen]);
        if ok_before && ok_after {
            return true;
        }
    }
    false
}
