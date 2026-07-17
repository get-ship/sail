//! Detect which fields of a `{#for}` row binder an expression reads.
//!
//! Companion to `for_uses_row` — that one tells us whether the row is
//! used AT ALL (which gates the per-row `__v` version signal); this
//! one tells us WHICH fields are used, which gates per-field diff
//! checks in the `update()` callback. The point is to avoid re-firing
//! a binding when the row instance changed but the specific field
//! that binding reads did not.
//!
//! Conservative on purpose: any whole-row reference (`row` outside a
//! `row.<ident>` pattern, `row[…]`, spread, etc.) bumps the result to
//! `Whole`, which the emitter treats as "don't gate — re-fire always."
//! False positives (reading `row` inside a string literal) over-emit
//! the gate but stay correct.

use crate::ir::Expr;

/// How an expression references the row binder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowUse {
    /// Doesn't read the row binder.
    None,
    /// Reads only via `<var>.<field>` patterns — listed below.
    /// Empty list with `FieldsOnly` is treated as `None`.
    FieldsOnly(Vec<String>),
    /// Reads the row binder in a way we can't decompose into fields
    /// (bare reference, computed access, spread, etc.). The emitter
    /// must fall back to firing the binding on every row swap.
    Whole,
}

impl RowUse {
    /// Union two row-usage shapes. Whole is absorbing; FieldsOnly
    /// unions field lists; None is identity.
    pub fn union(self, other: RowUse) -> RowUse {
        match (self, other) {
            (RowUse::Whole, _) | (_, RowUse::Whole) => RowUse::Whole,
            (RowUse::None, x) | (x, RowUse::None) => x,
            (RowUse::FieldsOnly(a), RowUse::FieldsOnly(b)) => {
                let mut out = a;
                for f in b {
                    if !out.iter().any(|x| x == &f) {
                        out.push(f);
                    }
                }
                RowUse::FieldsOnly(out)
            }
        }
    }
}

/// Walk the raw expression source for `<var>` references and
/// classify each as field-access or whole-reference.
pub fn analyze_expr(expr: &Expr, var_name: &str) -> RowUse {
    analyze_source(&expr.source, var_name)
}

pub fn analyze_source(source: &str, var_name: &str) -> RowUse {
    if var_name.is_empty() {
        return RowUse::None;
    }
    let bytes = source.as_bytes();
    let nbytes = var_name.as_bytes();
    let nlen = nbytes.len();
    if bytes.len() < nlen {
        return RowUse::None;
    }
    let mut fields: Vec<String> = Vec::new();
    let mut whole = false;
    let mut none = true;
    let mut i = 0;
    while i + nlen <= bytes.len() {
        if &bytes[i..i + nlen] != nbytes {
            i += 1;
            continue;
        }
        // Word boundary on both sides — the matched substring must
        // BE the identifier, not be embedded inside another one
        // (`rows.length`, `borrow()`, `arrow_fn`, `$row`).
        let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
        let after_ok = i + nlen == bytes.len() || !is_ident_byte(bytes[i + nlen]);
        if !(before_ok && after_ok) {
            i += 1;
            continue;
        }
        none = false;
        let after = i + nlen;
        // Followed by `.<ident>` — record the field. Anything else
        // (bare `row` reference, `row[…]`, `row()`, `row,`, `?.`,
        // spread `...row`) means we can't decompose into a single
        // field; bump to Whole and keep walking (so we still detect
        // None vs Whole correctly).
        if after < bytes.len() && bytes[after] == b'.' && after + 1 < bytes.len() {
            let nxt = bytes[after + 1];
            if is_ident_start(nxt) {
                let mut j = after + 1;
                while j < bytes.len() && is_ident_byte(bytes[j]) {
                    j += 1;
                }
                let field = std::str::from_utf8(&bytes[after + 1..j])
                    .unwrap_or("")
                    .to_string();
                if !field.is_empty() && !fields.iter().any(|f| f == &field) {
                    fields.push(field);
                }
                i = j;
                continue;
            }
        }
        whole = true;
        i = after;
    }
    if none {
        RowUse::None
    } else if whole {
        RowUse::Whole
    } else if fields.is_empty() {
        // Reached via match-then-no-field-record path (shouldn't
        // happen with the loop above, but be defensive).
        RowUse::Whole
    } else {
        RowUse::FieldsOnly(fields)
    }
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

use crate::scan::is_ident_start;

#[cfg(test)]
mod tests {
    use super::*;

    fn an(src: &str, var: &str) -> RowUse {
        analyze_source(src, var)
    }

    #[test]
    fn none_when_var_absent() {
        assert_eq!(an("a + b", "row"), RowUse::None);
        assert_eq!(an("", "row"), RowUse::None);
        assert_eq!(an("rows.length", "row"), RowUse::None);
    }

    #[test]
    fn fields_only_collects_dot_access() {
        assert_eq!(an("row.id", "row"), RowUse::FieldsOnly(vec!["id".into()]));
        assert_eq!(
            an("row.id === row.label", "row"),
            RowUse::FieldsOnly(vec!["id".into(), "label".into()]),
        );
        // Duplicates collapse.
        assert_eq!(
            an("row.id + row.id", "row"),
            RowUse::FieldsOnly(vec!["id".into()]),
        );
    }

    #[test]
    fn whole_when_bare_or_computed_use() {
        // Bare reference (passed as arg).
        assert_eq!(an("fn(row)", "row"), RowUse::Whole);
        // Computed access.
        assert_eq!(an("row['id']", "row"), RowUse::Whole);
        // Spread.
        assert_eq!(an("...row", "row"), RowUse::Whole);
        // Optional chain — could be supported later, conservative for now.
        assert_eq!(an("row?.id", "row"), RowUse::Whole);
    }

    #[test]
    fn mixed_field_and_bare_is_whole() {
        // If a binding both reads row.id AND passes row to a fn, we
        // can't gate — the fn might depend on any field.
        assert_eq!(an("fn(row) + row.id", "row"), RowUse::Whole);
    }

    #[test]
    fn union_semantics() {
        let a = RowUse::FieldsOnly(vec!["id".into()]);
        let b = RowUse::FieldsOnly(vec!["label".into()]);
        assert_eq!(a.union(b), RowUse::FieldsOnly(vec!["id".into(), "label".into()]));

        assert_eq!(RowUse::None.union(RowUse::Whole), RowUse::Whole);
        let c = RowUse::FieldsOnly(vec!["id".into()]);
        assert_eq!(c.union(RowUse::Whole), RowUse::Whole);
    }

    #[test]
    fn comparison_with_outer_signal_is_fields_only() {
        // The bench's hot path: `row.id === selected()`. Only row.id
        // is a row-field read; `selected()` is an outer-signal call
        // that doesn't involve the row binder at all.
        assert_eq!(
            an("row.id === selected()", "row"),
            RowUse::FieldsOnly(vec!["id".into()]),
        );
    }

    #[test]
    fn index_only_expr_is_none_for_var_analysis() {
        // The analyzer itself only knows about one var at a time; an
        // `__i % 2` expression doesn't touch `row` so it returns None.
        // The TemplateEmitter::row_use_for_source caller is responsible
        // for additionally checking the index name and escalating to
        // Whole — without that escalation, `{__i}` text bindings would
        // be silently dropped from `update()` and go stale on reorder.
        assert_eq!(an("__i % 2 === 0", "row"), RowUse::None);
        assert_eq!(an("__i", "__i"), RowUse::Whole);
    }
}
