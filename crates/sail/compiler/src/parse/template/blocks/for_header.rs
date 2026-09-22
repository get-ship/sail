//! Parse the header of a `{#for X of items[ key={…}]}` block.

use crate::diagnostics::{CompileError, Span};
use crate::ir::Expr;
use crate::parse::expr;

const RULE_ID: &str = "sail-for-header";
pub(super) struct ForHeader {
    pub var_name: String,
    pub index_name: Option<String>,
    pub iterable: Expr,
    pub key: Option<Expr>,
}

pub(super) fn parse_for_header(src: &str, span: Span) -> Result<ForHeader, CompileError> {
    let of_idx = find_keyword_index(src, " of ").ok_or_else(|| {
        CompileError::new("{#for} expects \"X of items\"")
            .with_span(span)
            .with_category(RULE_ID)
    })?;
    let left = src[..of_idx].trim();
    let mut right = src[of_idx + 4..].trim().to_string();

    let (var_name, index_name) = parse_for_binding(left, span)?;

    let mut key: Option<Expr> = None;
    if let Some(start) = find_key_clause_start(&right) {
        // `start` points at the `k` of `key={…}`. Skip past `key={`.
        let key_src_start = start + "key={".len();
        let key_src_end = right.len() - 1; // trailing `}`
        let key_src = right[key_src_start..key_src_end].to_string();
        expr::validate_at(&key_src, false, "{#for} key", span)?;
        key = Some(Expr::new(key_src, span, false));
        right = right[..start].trim_end().to_string();
    }
    expr::validate_at(&right, false, "{#for} iterable", span)?;
    let iterable = Expr::new(right, span, false);
    Ok(ForHeader {
        var_name,
        index_name,
        iterable,
        key,
    })
}

fn parse_for_binding(left: &str, span: Span) -> Result<(String, Option<String>), CompileError> {
    if let Some(rest) = left.strip_prefix('(') {
        let inner = rest.strip_suffix(')').ok_or_else(|| {
            CompileError::new(format!("{{#for}} bad variable: {left}"))
                .with_span(span)
                .with_category(RULE_ID)
        })?;
        let mut parts = inner.split(',');
        let v = parts
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| is_ident(s))
            .ok_or_else(|| {
                CompileError::new(format!("{{#for}} bad variable: {left}"))
                    .with_span(span)
                    .with_category(RULE_ID)
            })?;
        let idx = parts
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| is_ident(s))
            .ok_or_else(|| {
                CompileError::new(format!("{{#for}} bad index: {left}"))
                    .with_span(span)
                    .with_category(RULE_ID)
            })?;
        return Ok((v, Some(idx)));
    }
    if !is_ident(left) {
        return Err(CompileError::new(format!("{{#for}} bad variable: {left}"))
            .with_span(span)
            .with_category(RULE_ID));
    }
    Ok((left.to_string(), None))
}

fn is_ident(s: &str) -> bool {
    let b = s.as_bytes();
    if b.is_empty() {
        return false;
    }
    if !(b[0].is_ascii_alphabetic() || b[0] == b'_' || b[0] == b'$') {
        return false;
    }
    b.iter()
        .skip(1)
        .all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'$')
}

/// Locate ` key={` whose matching `}` is the last byte of `right`.
/// Returns the byte index of the `k` in `key=`, not the leading space.
fn find_key_clause_start(right: &str) -> Option<usize> {
    if !right.ends_with('}') {
        return None;
    }
    let bytes = right.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        let c = bytes[i];
        if let Some(q) = str_quote {
            if c == q && (i == 0 || bytes[i - 1] != b'\\') {
                str_quote = None;
            }
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            str_quote = Some(c);
            continue;
        }
        if c == b'}' {
            depth += 1;
            continue;
        }
        if c == b'{' {
            depth -= 1;
            if depth == 0 {
                let prefix = " key=";
                if i >= prefix.len() {
                    let pstart = i - prefix.len();
                    // Indices come from byte-walking; if `right` contains a
                    // multi-byte UTF-8 char (e.g. `↔` from `&harr;`), the
                    // slice would panic. The prefix is pure ASCII so a
                    // boundary-safe equality check via `ends_with` on a
                    // sub-slice works just as well — we only care that the
                    // 5 bytes immediately before `i` match " key=".
                    if right.is_char_boundary(pstart)
                        && right.is_char_boundary(i)
                        && &right[pstart..i] == prefix
                    {
                        return Some(pstart + 1);
                    }
                }
                return None;
            }
        }
    }
    None
}

fn find_keyword_index(s: &str, keyword: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let kw = keyword.as_bytes();
    let mut str_quote: Option<u8> = None;
    let mut depth = 0i32;
    let mut i = 0usize;
    while i + kw.len() <= bytes.len() {
        let c = bytes[i];
        if let Some(q) = str_quote {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                str_quote = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            str_quote = Some(c);
            i += 1;
            continue;
        }
        if matches!(c, b'(' | b'[' | b'{') {
            depth += 1;
        } else if matches!(c, b')' | b']' | b'}') {
            depth -= 1;
        }
        if depth == 0 && bytes[i..].starts_with(kw) {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ident_check() {
        assert!(is_ident("foo"));
        assert!(is_ident("_x1"));
        assert!(!is_ident("1foo"));
        assert!(!is_ident(""));
    }
}
