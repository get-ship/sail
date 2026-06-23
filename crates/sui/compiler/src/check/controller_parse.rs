//! Lightweight textual extraction of the `controller(def)` call from a
//! paired `.ts` file.
//!
//! The validations don't need a full TS AST. They need:
//!
//!  * Whether the file uses the `controller(...)` shape at all (vs. the
//!    legacy `function setup` / `export default null` shapes).
//!  * The list of top-level keys in the def object literal, with spans.
//!  * The body text of each `derived: { X: () => ... }` entry.
//!
//! Everything is recovered with a single linear scan that tracks
//! string / template / comment state and brace depth — same approach
//! `expr_subset` and `handler_complexity` use elsewhere in this crate.

use std::path::{Path, PathBuf};

use super::js_scan::{
    is_ident_cont, is_ident_start, skip_balanced, skip_block_comment,
    skip_line_comment, skip_trivia,
};
use crate::diagnostics::Span;

/// A parsed `controller({...})` def. Spans point into [`Self::source`].
#[derive(Debug)]
pub struct ParsedController {
    /// Absolute or relative path to the `.ts` file. Surfaced in errors.
    pub path: PathBuf,
    /// Full source text of the controller file. Owned so callers can
    /// keep the parse around without juggling lifetimes.
    pub source: String,
    /// Top-level keys in the def object literal, in source order.
    pub sections: Vec<Section>,
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    /// Span of the key identifier (for error pointing).
    pub key_span: Span,
    /// Span of the value (the expression after `:` up to the trailing
    /// comma or closing `}`). For `derived: {...}` this brackets the
    /// inner object literal including its braces.
    pub value_span: Span,
}

impl ParsedController {
    /// Look up a section by name.
    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.name == name)
    }

    /// The slice of `source` covered by `span`.
    pub fn slice(&self, span: Span) -> &str {
        &self.source[span.start..span.end]
    }
}

/// If a paired controller file exists next to `source_path`, read and
/// parse it. Returns `None` if no paired file, the file isn't readable,
/// or it doesn't use the `controller({...})` shape (e.g.
/// `export default null`).
///
/// Naming follows the source file's stem:
///   - `page.sui`  → `controller.ts` (new view-directory convention)
///   - `<stem>.sui` → `<stem>.controller.ts` (legacy + components)
pub fn read_paired_controller(source_path: Option<&Path>) -> Option<ParsedController> {
    let p = source_path?;
    let stem = p.file_stem()?.to_str()?;
    let dir = p.parent().unwrap_or(Path::new("."));
    let ts_path = if stem == "page" {
        dir.join("controller.ts")
    } else {
        dir.join(format!("{stem}.controller.ts"))
    };
    let source = std::fs::read_to_string(&ts_path).ok()?;
    parse(ts_path, source)
}

/// Parse a controller `.ts` source. Public so tests can drive it
/// without touching the filesystem.
pub fn parse(path: PathBuf, source: String) -> Option<ParsedController> {
    let call_open = find_controller_call(&source)?;
    let arg_open = find_arg_object(&source, call_open)?;
    let arg_close = match_brace(&source, arg_open)?;
    let sections = parse_sections(&source, arg_open + 1, arg_close);
    Some(ParsedController { path, source, sections })
}

/// Find the byte offset of the `(` that opens the
/// `export default controller(...)` (or `controller<Generic>(...)`)
/// call, skipping comments and strings.
fn find_controller_call(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        i = skip_trivia(bytes, i);
        if !is_ident_start(bytes.get(i).copied()?) {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let word = &src[start..i];
        if word != "controller" {
            continue;
        }
        let mut k = skip_trivia(bytes, i);
        // Optional generic: `<...>` — match angle brackets allowing
        // nested generics. This is lossy (TS allows `<` as a less-than
        // in expressions, but not in this position right after the
        // identifier `controller`), so it's fine to handle naively.
        if k < bytes.len() && bytes[k] == b'<' {
            let mut depth = 0i32;
            while k < bytes.len() {
                match bytes[k] {
                    b'<' => depth += 1,
                    b'>' => {
                        depth -= 1;
                        if depth == 0 {
                            k += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                k += 1;
            }
            k = skip_trivia(bytes, k);
        }
        if k < bytes.len() && bytes[k] == b'(' {
            return Some(k);
        }
    }
    None
}

/// Given the offset of `(`, skip whitespace/comments and return the
/// offset of the first `{` (the opening of the def object literal).
fn find_arg_object(src: &str, paren_open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = paren_open + 1;
    i = skip_trivia(bytes, i);
    if i < bytes.len() && bytes[i] == b'{' {
        Some(i)
    } else {
        None
    }
}

/// Walk top-level entries of an object literal between `start..end`
/// (start points at the byte AFTER the opening `{`; end points AT the
/// closing `}`). Each entry is `key: value,` — collect (key, key_span,
/// value_span). Skips comments / strings / nested braces.
fn parse_sections(src: &str, start: usize, end: usize) -> Vec<Section> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = start;
    while i < end {
        i = skip_trivia(bytes, i);
        if i >= end {
            break;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        // Object key: identifier, or quoted string. Quoted-string
        // sections are unusual but legal TS; treat them as opaque
        // (caller's `unknown section` rule will reject if applicable).
        let key_start = i;
        let key;
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            let s = i + 1;
            i += 1;
            while i < end && bytes[i] != q {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            key = src[s..i].to_string();
            if i < end {
                i += 1;
            }
        } else if is_ident_start(bytes[i]) {
            while i < end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            key = src[key_start..i].to_string();
        } else {
            // Spread (`...x`), computed (`[x]:`), or other shape we
            // don't recognise — bail on this entry.
            i = advance_past_entry(bytes, i, end);
            continue;
        }
        let key_span = Span::new(key_start, i);
        i = skip_trivia(bytes, i);
        if i >= end || bytes[i] != b':' {
            // Shorthand property (`{ foo, bar }`) or method
            // (`{ foo() {...} }`). Method form: next non-trivia is `(`.
            if i < end && bytes[i] == b'(' {
                let value_start = i;
                i = skip_balanced(bytes, i, end);
                i = skip_trivia(bytes, i);
                if i < end && bytes[i] == b'{' {
                    i = skip_balanced(bytes, i, end);
                }
                out.push(Section { name: key, key_span, value_span: Span::new(value_start, i) });
            } else {
                out.push(Section { name: key, key_span, value_span: Span::new(key_span.end, key_span.end) });
            }
            i = advance_past_entry(bytes, i, end);
            continue;
        }
        // Skip the colon.
        i += 1;
        i = skip_trivia(bytes, i);
        let value_start = i;
        let raw_end = entry_value_end(bytes, i, end);
        let mut value_end = raw_end;
        while value_end > value_start && bytes[value_end - 1].is_ascii_whitespace() {
            value_end -= 1;
        }
        out.push(Section { name: key, key_span, value_span: Span::new(value_start, value_end) });
        i = raw_end;
        i = skip_trivia(bytes, i);
        if i < end && bytes[i] == b',' {
            i += 1;
        }
    }
    out
}

/// Find the end offset of an object-literal entry's value: a position
/// at the next top-level `,` or `}`.
fn entry_value_end(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' { k += 2; continue; }
            if c == q { in_str = None; }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); k += 1; }
            b'(' | b'{' | b'[' => { k = skip_balanced(bytes, k, end); }
            b'/' if k + 1 < end && bytes[k + 1] == b'/' => {
                k = skip_line_comment(bytes, k);
            }
            b'/' if k + 1 < end && bytes[k + 1] == b'*' => {
                k = skip_block_comment(bytes, k);
            }
            b',' => return k,
            _ => k += 1,
        }
    }
    end
}

/// Recovery: skip to next top-level `,` or `}` after a malformed entry.
fn advance_past_entry(bytes: &[u8], start: usize, end: usize) -> usize {
    let stop = entry_value_end(bytes, start, end);
    if stop < end && bytes[stop] == b',' {
        stop + 1
    } else {
        stop
    }
}

/// Match a `{` at `pos` to its closing `}`. Returns the offset of `}`.
fn match_brace(src: &str, pos: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if pos >= bytes.len() || bytes[pos] != b'{' {
        return None;
    }
    let stop = skip_balanced(bytes, pos, bytes.len());
    if stop > pos { Some(stop - 1) } else { None }
}

