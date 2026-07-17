//! Detect whether a `{#for}` body references the row variable or
//! index variable. Used by `control_emit::emit_for` to skip the
//! per-row `__v` version signal (and matching `void $scope.__v()`
//! reads plus `__v.set` increment) when no binding inside the body
//! would observe the update.
//!
//! Conservative: matches identifier-shaped substrings in raw
//! expression source. False positives (e.g. `'row'` inside a string
//! literal) over-emit `__v` — correct but slower. False negatives
//! would silently break key-reuse reactivity, so the walk visits
//! every reachable `Expr` (including nested boundaries' tests,
//! discriminants, iterables, keys, and attribute expressions).

use crate::ir::{AttrInstr, Expr, Instr};

/// True iff any expression reachable from `body` references `var_name`
/// (the row binder) or `idx_name` (the optional index binder) as an
/// identifier. Walks through nested elements and control-flow blocks.
pub fn body_references_for_vars(
    body: &[Instr],
    var_name: &str,
    idx_name: Option<&str>,
) -> bool {
    body.iter().any(|i| instr_references(i, var_name, idx_name))
}

fn instr_references(instr: &Instr, var_name: &str, idx_name: Option<&str>) -> bool {
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } => false,
        Instr::DynamicText { expr } => expr_references(expr, var_name, idx_name),
        Instr::Element(el) => {
            el.attrs
                .iter()
                .any(|a| attr_references(a, var_name, idx_name))
                || body_references_for_vars(&el.children, var_name, idx_name)
        }
        Instr::If(i) => {
            i.branches.iter().any(|b| {
                expr_references(&b.cond, var_name, idx_name)
                    || body_references_for_vars(&b.body, var_name, idx_name)
            }) || i
                .else_body
                .as_ref()
                .map(|b| body_references_for_vars(b, var_name, idx_name))
                .unwrap_or(false)
        }
        Instr::For(f) => {
            // The inner {#for} may close over the outer row via its
            // iterable, key, or body bindings.
            expr_references(&f.iterable, var_name, idx_name)
                || f.key
                    .as_ref()
                    .is_some_and(|k| expr_references(k, var_name, idx_name))
                || body_references_for_vars(&f.body, var_name, idx_name)
                || f.else_body
                    .as_ref()
                    .is_some_and(|b| body_references_for_vars(b, var_name, idx_name))
        }
        Instr::Match(m) => {
            expr_references(&m.discriminant, var_name, idx_name)
                || m.cases.iter().any(|c| {
                    expr_references(&c.test, var_name, idx_name)
                        || body_references_for_vars(&c.body, var_name, idx_name)
                })
                || m.default
                    .as_ref()
                    .is_some_and(|b| body_references_for_vars(b, var_name, idx_name))
        }
        Instr::WhileLoading(w) => body_references_for_vars(&w.body, var_name, idx_name),
        Instr::WhileError(w) => body_references_for_vars(&w.body, var_name, idx_name),
    }
}

fn attr_references(attr: &AttrInstr, var_name: &str, idx_name: Option<&str>) -> bool {
    match attr {
        AttrInstr::Static { .. }
        | AttrInstr::BindThis { .. }
        | AttrInstr::SlotReflectEmpty { .. }
        | AttrInstr::Motion { .. } => false,
        AttrInstr::Expr { expr, .. }
        | AttrInstr::BindValue { expr, .. }
        | AttrInstr::BindChecked { expr, .. }
        | AttrInstr::Class { expr, .. }
        | AttrInstr::Prop { expr, .. }
        | AttrInstr::Mutation { expr, .. } => expr_references(expr, var_name, idx_name),
        AttrInstr::On { handler, .. } => expr_references(handler, var_name, idx_name),
        AttrInstr::DynamicAttr { name_expr, expr, .. } => {
            expr_references(name_expr, var_name, idx_name)
                || expr_references(expr, var_name, idx_name)
        }
        AttrInstr::Use { param, .. } => param
            .as_ref()
            .is_some_and(|e| expr_references(e, var_name, idx_name)),
    }
}

fn expr_references(expr: &Expr, var_name: &str, idx_name: Option<&str>) -> bool {
    contains_identifier(&expr.source, var_name)
        || idx_name.is_some_and(|n| contains_identifier(&expr.source, n))
}

/// Substring search with JS identifier word boundaries on each side.
/// `$` and `_` are identifier chars; digits are too but never start an
/// identifier. We don't decode escapes inside string literals — a
/// `'row'` literal still matches, which is a safe over-emit.
fn contains_identifier(source: &str, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let bytes = source.as_bytes();
    let nbytes = name.as_bytes();
    let nlen = nbytes.len();
    if bytes.len() < nlen {
        return false;
    }
    let mut i = 0;
    while i + nlen <= bytes.len() {
        if &bytes[i..i + nlen] == nbytes {
            let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
            let after_ok = i + nlen == bytes.len() || !is_ident_byte(bytes[i + nlen]);
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_bare_identifier() {
        assert!(contains_identifier("row.label", "row"));
        assert!(contains_identifier("a + row", "row"));
        assert!(contains_identifier("fn(row)", "row"));
    }

    #[test]
    fn rejects_substring_in_other_identifier() {
        assert!(!contains_identifier("rows.length", "row"));
        assert!(!contains_identifier("borrow()", "row"));
        assert!(!contains_identifier("arrow_fn", "row"));
        assert!(!contains_identifier("$row", "row"));
    }

    #[test]
    fn matches_in_string_literal_is_conservative() {
        // Over-emit is acceptable; under-emit would break reactivity.
        assert!(contains_identifier("'row'", "row"));
    }

    #[test]
    fn idx_name_alternative_matched() {
        let src = "i + 1";
        assert!(contains_identifier(src, "i"));
        assert!(!contains_identifier(src, "row"));
    }
}
