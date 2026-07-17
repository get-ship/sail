//! Shared byte-scan primitives for the compiler's textual passes.
//!
//! Several passes (parse, render, check, emit, lower) walk raw `.ts`/expr
//! source byte-by-byte. The lexical leaves — "is this byte an identifier
//! char?" — were re-defined in 8 places; this is their one home. They are
//! pure, stable character-class predicates with no pass-specific knowledge.
//!
//! The bounded, string- and comment-aware `skip_trivia` / `skip_balanced` are
//! the canonical scanners. Bounded `(bytes, i, end)` is the general form;
//! unbounded callers pass `end = bytes.len()` (js_scan keeps a thin wrapper).

/// A JavaScript identifier start byte: ASCII letter, `_`, or `$`.
#[inline]
pub(crate) fn is_ident_start(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphabetic()
}

/// A JavaScript identifier continuation byte: an [`is_ident_start`] byte or a digit.
#[inline]
pub(crate) fn is_ident_cont(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

/// Advance past whitespace, `//` line comments, and `/* … */` block comments
/// within `[i, end)`. Idempotent at non-trivia bytes.
pub(crate) fn skip_trivia(bytes: &[u8], mut i: usize, end: usize) -> usize {
    loop {
        while i < end && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(end);
            continue;
        }
        break;
    }
    i
}

/// At a `(`, `{`, or `[`, return the offset just past the matching closer,
/// within `[i, end)`. String/template-literal contents and `//`+`/* */`
/// comments are skipped. Returns `i + 1` when `bytes[i]` isn't an opener so
/// callers can defensively step forward.
pub(crate) fn skip_balanced(bytes: &[u8], i: usize, end: usize) -> usize {
    if i >= end {
        return end;
    }
    let (open, close) = match bytes[i] {
        b'(' => (b'(', b')'),
        b'{' => (b'{', b'}'),
        b'[' => (b'[', b']'),
        _ => return i + 1,
    };
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
            c if c == open => { depth += 1; k += 1; }
            c if c == close => { depth -= 1; k += 1; if depth == 0 { return k; } }
            b'/' if k + 1 < end && bytes[k + 1] == b'/' => {
                while k < end && bytes[k] != b'\n' { k += 1; }
            }
            b'/' if k + 1 < end && bytes[k + 1] == b'*' => {
                k += 2;
                while k + 1 < end && !(bytes[k] == b'*' && bytes[k + 1] == b'/') { k += 1; }
                k = (k + 2).min(end);
            }
            _ => k += 1,
        }
    }
    end
}
