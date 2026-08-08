//! Shared low-level scanning helpers for the textual `.ts` checks
//! (`controller_parse`, `pure_derived`, `scope_bound`).
//!
//! Each helper is string-, comment-, and brace-aware so the callers
//! don't accidentally match keywords inside string literals or skip
//! past the end of a parenthesised expression.

/// Advance past whitespace + comments to the next significant byte. Thin
/// unbounded adapter (`end = len`) over the canonical bounded
/// [`crate::scan::skip_trivia`]; the ~50 check-side callers scan to EOF.
pub(super) fn skip_trivia(bytes: &[u8], i: usize) -> usize {
    crate::scan::skip_trivia(bytes, i, bytes.len())
}

/// Walk to the next `,` or `}` at depth zero. Used by object-literal
/// parsers to find the end of a `key: value` entry.
pub(super) fn next_top_separator(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' {
                k += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
                k += 1;
            }
            b'/' if let Some(after) = crate::scan::try_skip_regex(bytes, k, end) => {
                k = after;
            }
            b'(' | b'{' | b'[' => {
                k = skip_balanced(bytes, k, end);
            }
            b',' => return k,
            _ => k += 1,
        }
    }
    end
}

// The identifier predicates are crate-shared; re-export so this module's
// consumers (scope_bound_expr, symbol_table) keep importing them from here
// alongside the skip_* scanners that still live in this file.
pub(super) use crate::scan::{is_ident_cont, is_ident_start, skip_balanced};
