//! Generic byte-level scanning primitives used by the scope-factory
//! extractor and its type-inference heuristic. These are string-, comment-,
//! and bracket-aware so callers don't match inside string literals or walk
//! past the end of a balanced group. No scope-factory domain knowledge lives
//! here — they're lexical leaves.
//!
//! `skip_trivia`/`skip_balanced` (+ the identifier predicates) are crate-shared
//! and re-exported below; this module keeps the scope-factory-specific scanners
//! (`next_top_separator` with `<…>` generic handling, `skip_angle`,
//! `advance_past_entry`).

pub(super) fn next_top_separator(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' { k = (k + 2).min(end); continue; }
            if c == q { in_str = None; }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); k += 1; }
            b'(' | b'{' | b'[' => { k = skip_balanced(bytes, k, end); }
            // A `<` in a generic position (`signal<Record<A, B>>(…)`) opens a
            // type-argument block whose commas are NOT entry separators. Without
            // skipping it, `signal<Record<Section, boolean>>(…)` truncated at the
            // inner comma and the whole entry degraded to bare `unknown`. Only
            // treat `<` as a generic when it follows an identifier char (`Foo<`)
            // so we don't misread a stray comparison.
            b'<' if k > start && is_ident_cont(bytes[k - 1]) => {
                k = skip_angle(bytes, k, end);
            }
            b',' | b'}' => return k,
            _ => k += 1,
        }
    }
    end
}

/// Skip a balanced `<…>` generic block starting at `bytes[i] == b'<'`.
/// Returns the index just past the matching `>`, or `i + 1` if it can't
/// balance (so the caller still makes progress). Tracks `(`/`{`/`[` and
/// string literals inside the type args, and ignores `=>` / `<=` / `>=`
/// so an arrow or comparison inside the block doesn't unbalance the count.
pub(super) fn skip_angle(bytes: &[u8], i: usize, end: usize) -> usize {
    if i >= end || bytes[i] != b'<' {
        return (i + 1).min(end);
    }
    let mut depth = 0i32;
    let mut k = i;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' { k = (k + 2).min(end); continue; }
            if c == q { in_str = None; }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); k += 1; }
            b'(' | b'{' | b'[' => { k = skip_balanced(bytes, k, end); }
            // `=>` is an arrow, not a closing angle.
            b'=' if k + 1 < end && bytes[k + 1] == b'>' => { k += 2; }
            // `<=` / `>=` are comparisons, not bracket motion.
            b'<' if k + 1 < end && bytes[k + 1] == b'=' => { k += 2; }
            b'>' if k + 1 < end && bytes[k + 1] == b'=' => { k += 2; }
            b'<' => { depth += 1; k += 1; }
            b'>' => {
                depth -= 1;
                k += 1;
                if depth == 0 {
                    return k;
                }
            }
            _ => k += 1,
        }
    }
    (i + 1).min(end)
}

pub(super) fn advance_past_entry(bytes: &[u8], start: usize, end: usize) -> usize {
    let stop = next_top_separator(bytes, start, end);
    if stop < end && bytes[stop] == b',' {
        stop + 1
    } else {
        stop
    }
}

// Identifier predicates are crate-shared; re-export so this module's siblings
// (scope_factory::mod, ::infer) keep importing them from `super::scan`
// alongside the bounded skip_* scanners that still live here.
pub(super) use crate::scan::{is_ident_cont, is_ident_start, skip_balanced, skip_trivia};
