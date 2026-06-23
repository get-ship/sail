//! Tokenizer for the renderable-subset expression grammar. Hand-rolled
//! to avoid pulling in a full JS lexer; mirrors what
//! `check/expr_subset.rs` walks for validation.

use super::EvalError;
use super::ast::{Node, TplPart};

#[derive(Debug, Clone)]
pub(super) enum Tok {
    Punct(&'static str),
    Num(f64),
    Str(String),
    Ident(String),
    Template(Vec<TplPart>),
}

pub(super) fn tokenize(src: &str) -> Result<Vec<Tok>, EvalError> {
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'\'' || c == b'"' {
            let (s, n) = read_string(src, i, c)?;
            out.push(Tok::Str(s));
            i = n;
            continue;
        }
        if c == b'`' {
            let (parts, n) = read_template(src, i)?;
            out.push(Tok::Template(parts));
            i = n;
            continue;
        }
        if c.is_ascii_digit()
            || (c == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit())
        {
            let (v, n) = read_number(src, i)?;
            out.push(Tok::Num(v));
            i = n;
            continue;
        }
        if is_ident_start(c) {
            let start = i;
            while i < bytes.len() && is_ident_cont(bytes[i]) {
                i += 1;
            }
            out.push(Tok::Ident(src[start..i].to_string()));
            continue;
        }
        // Multi-byte punctuation peeks compare bytes directly. Slicing
        // `src` here would panic when `i+N` lands inside a multi-byte
        // UTF-8 char (e.g. `°` from a decoded `&deg;`). All recognized
        // puncts are pure ASCII, so a byte comparison is safe and
        // sufficient.
        let three_bytes: &[u8] =
            if i + 3 <= bytes.len() { &bytes[i..i + 3] } else { &[] };
        if three_bytes == b"===" {
            out.push(Tok::Punct("==="));
            i += 3;
            continue;
        }
        if three_bytes == b"!==" {
            out.push(Tok::Punct("!=="));
            i += 3;
            continue;
        }
        let two_bytes: &[u8] =
            if i + 2 <= bytes.len() { &bytes[i..i + 2] } else { &[] };
        if two_bytes == b"=>" {
            return Err(EvalError::NonEvaluable(
                "arrow functions are not evaluable at render time".into(),
            ));
        }
        if two_bytes == b"++" || two_bytes == b"--" {
            return Err(EvalError::NonEvaluable(
                "increment / decrement operators are not evaluable".into(),
            ));
        }
        let two_punct: Option<&'static str> = match two_bytes {
            b"==" => Some("=="),
            b"!=" => Some("!="),
            b"<=" => Some("<="),
            b">=" => Some(">="),
            b"&&" => Some("&&"),
            b"||" => Some("||"),
            b"??" => Some("??"),
            b"?." => Some("?."),
            _ => None,
        };
        if let Some(p) = two_punct {
            out.push(Tok::Punct(p));
            i += 2;
            continue;
        }
        if c == b'=' {
            return Err(EvalError::NonEvaluable(
                "assignment is not evaluable at render time".into(),
            ));
        }
        let one: &'static str = match c {
            b'+' => "+", b'-' => "-", b'*' => "*", b'/' => "/", b'%' => "%",
            b'<' => "<", b'>' => ">", b'!' => "!", b'?' => "?", b':' => ":",
            b'.' => ".", b',' => ",", b'(' => "(", b')' => ")",
            b'[' => "[", b']' => "]", b'{' => "{", b'}' => "}",
            _ => {
                return Err(EvalError::ParseError(format!(
                    "unexpected character at offset {i}"
                )));
            }
        };
        out.push(Tok::Punct(one));
        i += 1;
    }
    Ok(out)
}

use crate::scan::{is_ident_cont, is_ident_start};

fn read_string(src: &str, from: usize, quote: u8) -> Result<(String, usize), EvalError> {
    let bytes = src.as_bytes();
    let mut i = from + 1;
    let mut out = String::new();
    let mut chunk_start = i;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' && i + 1 < bytes.len() {
            out.push_str(&src[chunk_start..i]);
            let (escaped, advance) = unescape_at(src, i + 1);
            out.push(escaped);
            i += 1 + advance;
            chunk_start = i;
            continue;
        }
        if c == quote {
            out.push_str(&src[chunk_start..i]);
            return Ok((out, i + 1));
        }
        i += 1;
    }
    Err(EvalError::ParseError("unterminated string literal".into()))
}

/// Decode the escape sequence starting at `src[pos]` (the byte after the
/// backslash). Returns the resulting char and the number of source bytes
/// consumed. Single-byte ASCII escapes consume 1; if the byte after the
/// backslash is the lead of a multi-byte UTF-8 char (e.g. `\<emoji>`)
/// the whole char is consumed so `chunk_start` stays on a boundary.
fn unescape_at(src: &str, pos: usize) -> (char, usize) {
    // First char starting at `pos` — its length determines `advance`.
    let ch = src[pos..].chars().next().unwrap_or('\0');
    let advance = ch.len_utf8();
    let mapped = match ch {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '\\' => '\\',
        '\'' => '\'',
        '"' => '"',
        '`' => '`',
        '0' => '\0',
        other => other,
    };
    (mapped, advance)
}

fn read_template(src: &str, from: usize) -> Result<(Vec<TplPart>, usize), EvalError> {
    let bytes = src.as_bytes();
    let mut i = from + 1;
    let mut parts: Vec<TplPart> = Vec::new();
    let mut cur = String::new();
    let mut chunk_start = i;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' && i + 1 < bytes.len() {
            cur.push_str(&src[chunk_start..i]);
            let (escaped, advance) = unescape_at(src, i + 1);
            cur.push(escaped);
            i += 1 + advance;
            chunk_start = i;
            continue;
        }
        if c == b'`' {
            cur.push_str(&src[chunk_start..i]);
            if !cur.is_empty() {
                parts.push(TplPart::Str(std::mem::take(&mut cur)));
            }
            return Ok((parts, i + 1));
        }
        if c == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            cur.push_str(&src[chunk_start..i]);
            if !cur.is_empty() {
                parts.push(TplPart::Str(std::mem::take(&mut cur)));
            }
            let (inner, end) = read_template_expr(src, i + 2)?;
            parts.push(TplPart::Expr(Box::new(inner)));
            i = end;
            chunk_start = i;
            continue;
        }
        i += 1;
    }
    Err(EvalError::ParseError("unterminated template literal".into()))
}

fn read_template_expr(src: &str, start: usize) -> Result<(Node, usize), EvalError> {
    let bytes = src.as_bytes();
    let mut depth = 1i32;
    let mut k = start;
    while k < bytes.len() && depth > 0 {
        let c = bytes[k];
        if c == b'{' {
            depth += 1;
            k += 1;
            continue;
        }
        if c == b'}' {
            depth -= 1;
            if depth == 0 {
                break;
            }
            k += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            k = read_string(src, k, c)?.1;
            continue;
        }
        if c == b'`' {
            k = read_template(src, k)?.1;
            continue;
        }
        k += 1;
    }
    if k >= bytes.len() {
        return Err(EvalError::ParseError(
            "unterminated `${...}` in template".into(),
        ));
    }
    let inner = super::parser::parse_expr(&src[start..k])?;
    Ok((inner, k + 1))
}

fn read_number(src: &str, start: usize) -> Result<(f64, usize), EvalError> {
    let bytes = src.as_bytes();
    let mut i = start;
    if bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len()
            && bytes[i] == b'.'
            && i + 1 < bytes.len()
            && bytes[i + 1].is_ascii_digit()
        {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
        }
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    let s = &src[start..i];
    s.parse::<f64>()
        .map(|v| (v, i))
        .map_err(|_| EvalError::ParseError(format!("invalid number: {s}")))
}
