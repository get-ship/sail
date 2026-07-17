//! Lightweight textual extraction of the `controller(fn)` call from a
//! paired `.ts` file or inline `<controller>` block.
//!
//! Ratified shape (one function form — the old `controller({ setup, derived,
//! on, mount, unmount })` object-literal sections are retired):
//!
//! ```ts
//! export default controller((ctx) => {
//!   const items = signal<Item[]>([])
//!   mount((el) => { ... })
//!   onCleanup(() => { ... })
//!   return { items, add }
//! })
//! ```
//!
//! `fn` is a single arrow function. Its own returned object literal IS the
//! whole contributed scope — state signals, `derived(...)` values, plain
//! helpers, and event handlers all live in the same flat namespace,
//! classified by their VALUE's shape (signal call / derived call / function
//! / other), not by which section they used to be nested under.
//!
//! The validations don't need a full TS AST. They need:
//!
//!  * The setup function's parameter list (the `ctx` binding — a bare ident
//!    or a destructuring pattern — and the optional `host` param).
//!  * The set of members the function's `return { ... }` (or concise-body
//!    `=> ({...})`) object contributes, with spans and a shape
//!    classification (ref / signal / derived / action / other).
//!
//! Everything is recovered with a single linear scan that tracks
//! string / template / comment state and brace depth — same approach
//! `expr_subset` and `handler_complexity` use elsewhere in this crate.

use std::path::{Path, PathBuf};

use super::js_scan::{is_ident_cont, is_ident_start, skip_balanced, skip_trivia};
use crate::diagnostics::Span;

/// Author-declared shape of a returned scope member's value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// `key: ref(initial)` — DOM-handle slot; written by the runtime,
    /// read like a signal.
    Ref,
    /// `key: signal(initial)` — readable + writable cell.
    Signal,
    /// `key: derived(() => ...)` — read-only computed getter.
    Derived,
    /// `key: () => ...` or `key: (e) => ...` — bound action / handler.
    Action,
    /// Anything else (plain values, expressions we can't classify).
    Other,
}

impl ScopeKind {
    pub fn label(self) -> &'static str {
        match self {
            ScopeKind::Ref => "ref",
            ScopeKind::Signal => "signal",
            ScopeKind::Derived => "derived",
            ScopeKind::Action => "action",
            ScopeKind::Other => "value",
        }
    }

    pub fn is_callable(self) -> bool {
        matches!(self, ScopeKind::Action)
    }
}

/// One member of the setup function's returned scope object.
#[derive(Debug, Clone)]
pub struct ScopeMember {
    pub name: String,
    /// Span of the key identifier (for error pointing).
    pub key_span: Span,
    /// Span of the value expression. For a shorthand entry (`{ foo }`)
    /// this points at the resolved local binding's right-hand side
    /// (found by scanning the setup body for `const foo = …` / `function
    /// foo …`) when one is found, else at the shorthand ident itself.
    pub value_span: Span,
    pub kind: ScopeKind,
}

/// A parsed `controller((ctx, host?) => {...})` call. Spans point into
/// [`Self::source`].
#[derive(Debug)]
pub struct ParsedController {
    /// Absolute or relative path to the `.ts` file. Surfaced in errors.
    pub path: PathBuf,
    /// Full source text of the controller file. Owned so callers can
    /// keep the parse around without juggling lifetimes.
    pub source: String,
    /// Span of the setup function passed to `controller(...)` — from the
    /// start of its parameter list to the end of its body.
    pub setup_span: Span,
    /// The setup function's first parameter, as authored (trimmed of any
    /// type annotation / default value) — a bare identifier (`ctx`) or a
    /// destructuring pattern's raw source text (`{ ship, route }`). Empty
    /// when the setup function declares no parameter.
    pub ctx_param: String,
    /// The setup function's second parameter (`host`), as authored, if
    /// present.
    pub host_param: Option<String>,
    /// Every member the setup function's returned object literal
    /// contributes, in source order.
    pub returned: Vec<ScopeMember>,
}

impl ParsedController {
    /// Look up a returned scope member by name.
    pub fn member(&self, name: &str) -> Option<&ScopeMember> {
        self.returned.iter().find(|m| m.name == name)
    }

    /// The slice of `source` covered by `span`.
    pub fn slice(&self, span: Span) -> &str {
        &self.source[span.start..span.end]
    }
}

/// If a paired controller file exists next to `source_path`, read and
/// parse it. Returns `None` if no paired file, the file isn't readable,
/// or it doesn't use the `controller(fn)` shape (e.g.
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
    let bytes = source.as_bytes();
    let end_all = bytes.len();

    let mut k = skip_trivia(bytes, call_open + 1);
    // Optional generic before the fn, e.g. `controller<Ctx, SC>((ctx) => …)`.
    if k < end_all && bytes[k] == b'<' {
        k = skip_generic_args(bytes, k, end_all);
        k = skip_trivia(bytes, k);
    }
    let setup_start = k;
    let (ctx_param, host_param, after_params) = parse_params(bytes, k, end_all)?;
    let mut m = skip_trivia(bytes, after_params);
    // Optional return-type annotation: `(ctx): SC => …`.
    if m < end_all && bytes[m] == b':' {
        m = skip_type_annotation(bytes, m + 1, end_all);
        m = skip_trivia(bytes, m);
    }
    if m + 1 >= end_all || bytes[m] != b'=' || bytes[m + 1] != b'>' {
        return None;
    }
    let body_start = skip_trivia(bytes, m + 2);
    let (returned, setup_end) = harvest_body(bytes, body_start, end_all)?;

    Some(ParsedController {
        path,
        source,
        setup_span: Span::new(setup_start, setup_end),
        ctx_param,
        host_param,
        returned,
    })
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

/// Parse the setup function's parameter list starting at `start` (which
/// may point at `(` for a parenthesised list, or at a bare identifier for
/// a single-param arrow with no parens: `ctx => {...}`).
///
/// Returns `(ctx_param, host_param, offset_after_params)`.
fn parse_params(bytes: &[u8], start: usize, end: usize) -> Option<(String, Option<String>, usize)> {
    if start >= end {
        return None;
    }
    if bytes[start] == b'(' {
        let close = skip_balanced(bytes, start, end);
        let inner_start = start + 1;
        let inner_end = close.saturating_sub(1).min(end);
        let parts = split_top_level(bytes, inner_start, inner_end);
        let ctx_param = parts
            .first()
            .map(|s| param_name_text(bytes, *s))
            .unwrap_or_default();
        let host_param = parts.get(1).map(|s| param_name_text(bytes, *s));
        Some((ctx_param, host_param, close))
    } else if is_ident_start(bytes[start]) {
        let s = start;
        let mut k = start;
        while k < end && is_ident_cont(bytes[k]) {
            k += 1;
        }
        let name = std::str::from_utf8(&bytes[s..k]).unwrap_or("").to_string();
        Some((name, None, k))
    } else {
        None
    }
}

/// Split a parameter-list interior on top-level commas (respecting nested
/// `()`/`{}`/`[]`/`<>` groups and string/template literals). Returns each
/// param's raw `(start, end)` byte span.
fn split_top_level(bytes: &[u8], start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = skip_trivia(bytes, start);
    let mut seg_start = i;
    let mut depth: i32 = 0;
    while i < end {
        let c = bytes[i];
        match c {
            b'"' | b'\'' | b'`' => {
                i = skip_string(bytes, i, end, c);
                continue;
            }
            b'(' | b'{' | b'[' | b'<' => depth += 1,
            b')' | b'}' | b']' | b'>' => depth -= 1,
            b',' if depth == 0 => {
                out.push((seg_start, i));
                i += 1;
                seg_start = skip_trivia(bytes, i);
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    if seg_start < end {
        out.push((seg_start, end));
    }
    out
}

/// Trim a raw param span down to just its binding pattern: strip a
/// trailing `: Type` annotation and/or `= default` value, and leading/
/// trailing trivia. Preserves destructuring patterns (`{ ship, route }`)
/// verbatim.
fn param_name_text(bytes: &[u8], span: (usize, usize)) -> String {
    let (start, end) = span;
    let mut i = start;
    let mut depth: i32 = 0;
    let mut cut = end;
    while i < end {
        let c = bytes[i];
        match c {
            b'"' | b'\'' | b'`' => {
                i = skip_string(bytes, i, end, c);
                continue;
            }
            b'(' | b'{' | b'[' | b'<' => depth += 1,
            b')' | b'}' | b']' | b'>' => depth -= 1,
            b':' | b'=' if depth == 0 => {
                cut = i;
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let s = skip_trivia(bytes, start);
    let mut e = cut;
    while e > s && bytes[e - 1].is_ascii_whitespace() {
        e -= 1;
    }
    if e <= s {
        return String::new();
    }
    std::str::from_utf8(&bytes[s..e]).unwrap_or("").trim().to_string()
}

/// Shape + members of the setup function's body. Returns `(members,
/// setup_end)` where `setup_end` is the offset just past the arrow
/// function's body (`}` for a block body, `)` for a parenthesised
/// expression body).
fn harvest_body(bytes: &[u8], body_start: usize, end: usize) -> Option<(Vec<ScopeMember>, usize)> {
    if body_start >= end {
        return None;
    }
    if bytes[body_start] == b'{' {
        let block_end = skip_balanced(bytes, body_start, end);
        let members = match find_last_top_level_return_object(bytes, body_start, block_end) {
            Some(obj_start) => {
                let obj_end = skip_balanced(bytes, obj_start, block_end);
                walk_and_classify(
                    bytes,
                    obj_start + 1,
                    obj_end.saturating_sub(1),
                    body_start,
                    block_end,
                )
            }
            None => Vec::new(),
        };
        Some((members, block_end))
    } else if bytes[body_start] == b'(' {
        // Expression body wrapped in parens: `() => ({...})`.
        let close = skip_balanced(bytes, body_start, end);
        let inner_start = skip_trivia(bytes, body_start + 1);
        let members = if inner_start < close && bytes[inner_start] == b'{' {
            let obj_end = skip_balanced(bytes, inner_start, close.saturating_sub(1).max(inner_start));
            walk_and_classify(
                bytes,
                inner_start + 1,
                obj_end.saturating_sub(1),
                body_start,
                close,
            )
        } else {
            Vec::new()
        };
        Some((members, close))
    } else {
        // Not an object-literal-returning arrow (bare expression body, or
        // something we don't model) — not a controller shape we recognise.
        None
    }
}

/// Walk `{ key: value, … }` entries within `start..end` (excluding the
/// braces) and classify each into a [`ScopeMember`]. Shorthand entries
/// (`{ foo }`) are resolved by scanning `body_start..body_end` (the whole
/// setup-function body) for a top-level `const foo = …` / `function foo
/// …` declaration.
fn walk_and_classify(
    bytes: &[u8],
    start: usize,
    end: usize,
    body_start: usize,
    body_end: usize,
) -> Vec<ScopeMember> {
    let mut out = Vec::new();
    walk_object_entries(bytes, start, end, |name, key_offset, value_start, value_end| {
        let key_span = Span::new(key_offset, key_offset + name.len());
        let shorthand = is_shorthand(bytes, name, value_start, value_end);
        let (value_span, kind) = if shorthand {
            match resolve_local_binding_span(bytes, name, body_start, body_end) {
                Some(span) => (span, classify_scope_value(bytes, span.start, span.end)),
                None => (Span::new(value_start, value_end), ScopeKind::Other),
            }
        } else {
            let span = Span::new(value_start, value_end);
            (span, classify_scope_value(bytes, span.start, span.end))
        };
        out.push(ScopeMember { name: name.to_string(), key_span, value_span, kind });
    });
    out
}

/// Returns true when the value-span is just the key ident (shorthand
/// property-init like `{ foo }`).
fn is_shorthand(bytes: &[u8], name: &str, start: usize, end: usize) -> bool {
    let s = skip_trivia(bytes, start);
    if s >= end || s + name.len() > end {
        return false;
    }
    if !std::str::from_utf8(&bytes[s..s + name.len()])
        .map(|w| w == name)
        .unwrap_or(false)
    {
        return false;
    }
    let after = skip_trivia(bytes, s + name.len());
    after >= end
}

/// Walk the setup-function body for a top-level `const NAME = …` /
/// `let NAME = …` / `function NAME …` declaration and return the span
/// covering its right-hand side (for `const`/`let`) or its whole
/// declaration (for `function`). The span is TIGHTLY bounded to the
/// binding's own initializer/body — NOT left open to `end` (the rest of
/// the setup function) — because callers scan this span for purity
/// (`pure_derived`) and type inference; an over-long span would false-
/// positive on unrelated code appearing later in the same function body.
/// Returns `None` when no such binding is found.
fn resolve_local_binding_span(bytes: &[u8], name: &str, start: usize, end: usize) -> Option<Span> {
    let nb = name.as_bytes();
    let mut i = start;
    while i < end {
        if bytes[i..].starts_with(b"function") {
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let after_kw = i + 8;
            if prev_ok && after_kw < end && !is_ident_cont(bytes[after_kw]) {
                let j = skip_trivia(bytes, after_kw);
                if j + nb.len() <= end
                    && &bytes[j..j + nb.len()] == nb
                    && (j + nb.len() == end || !is_ident_cont(bytes[j + nb.len()]))
                {
                    let bound = bound_expr_end(bytes, i, end);
                    return Some(Span::new(i, bound));
                }
            }
        }
        let kw_len = if bytes[i..].starts_with(b"const") {
            5
        } else if bytes[i..].starts_with(b"let") {
            3
        } else {
            0
        };
        if kw_len > 0 {
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let after_kw = i + kw_len;
            let post_ok = after_kw < end && !is_ident_cont(bytes[after_kw]);
            if prev_ok && post_ok {
                let j = skip_trivia(bytes, after_kw);
                if j + nb.len() <= end
                    && &bytes[j..j + nb.len()] == nb
                    && (j + nb.len() >= end || !is_ident_cont(bytes[j + nb.len()]))
                {
                    let mut k = skip_trivia(bytes, j + nb.len());
                    if k < end && bytes[k] == b':' {
                        k = skip_type_annotation(bytes, k + 1, end);
                        k = skip_trivia(bytes, k);
                    }
                    if k < end && bytes[k] == b'=' {
                        let rhs_start = skip_trivia(bytes, k + 1);
                        let bound = bound_expr_end(bytes, rhs_start, end);
                        return Some(Span::new(rhs_start, bound));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Best-effort tight upper bound for a single JS expression/declaration
/// starting at `start`. Recognises the shapes scope-entry initializers
/// actually take:
///   - `function NAME(...) {...}` — through the matching body `}`.
///   - `(params) => {...}` / `(params) => expr` — arrow: through the
///     block body's `}`, or (concise body) to the next top-level
///     statement boundary.
///   - `ident(...)` / `ident<T>(...)` (e.g. `signal(...)`, `derived(...)`)
///     — through the matching call `)`.
///   - anything else — to the next top-level statement boundary (a `;`
///     or a newline at bracket depth 0, whichever comes first).
fn bound_expr_end(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut i = skip_trivia(bytes, start);
    if i >= end {
        return end;
    }
    if let Some(after) = strip_kw(bytes, i, end, b"async") {
        i = skip_trivia(bytes, after);
        if i >= end {
            return end;
        }
    }
    if strip_kw(bytes, i, end, b"function").is_some() {
        // Walk to the body's opening `{`, then to its matching `}`.
        let mut j = i;
        while j < end && bytes[j] != b'{' {
            j += 1;
        }
        if j < end {
            return skip_balanced(bytes, j, end);
        }
        return statement_boundary(bytes, i, end);
    }
    if bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let mut after = skip_trivia(bytes, close);
        if after < end && bytes[after] == b':' {
            after = skip_type_annotation(bytes, after + 1, end);
            after = skip_trivia(bytes, after);
        }
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            let body_start = skip_trivia(bytes, after + 2);
            if body_start < end && bytes[body_start] == b'{' {
                return skip_balanced(bytes, body_start, end);
            }
            return statement_boundary(bytes, body_start, end);
        }
        return statement_boundary(bytes, i, end);
    }
    if is_ident_start(bytes[i]) {
        let s = i;
        while i < end && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let mut after = skip_trivia(bytes, i);
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            // `name => body` shorthand arrow.
            let body_start = skip_trivia(bytes, after + 2);
            if body_start < end && bytes[body_start] == b'{' {
                return skip_balanced(bytes, body_start, end);
            }
            return statement_boundary(bytes, body_start, end);
        }
        if after < end && bytes[after] == b'<' {
            after = skip_generic_args(bytes, after, end);
            after = skip_trivia(bytes, after);
        }
        if after < end && bytes[after] == b'(' {
            return skip_balanced(bytes, after, end);
        }
        let _ = s;
        return statement_boundary(bytes, i, end);
    }
    statement_boundary(bytes, i, end)
}

/// Walk to the next top-level (bracket-depth-0, outside strings/comments)
/// `;` or newline, whichever comes first. Used as a conservative bound
/// for expression shapes `bound_expr_end` doesn't specifically model.
fn statement_boundary(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut i = start;
    let mut depth: i32 = 0;
    while i < end {
        let c = bytes[i];
        match c {
            b'"' | b'\'' | b'`' => {
                i = skip_string(bytes, i, end, c);
                continue;
            }
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b';' if depth <= 0 => return i,
            b'\n' if depth <= 0 => return i,
            _ => {}
        }
        i += 1;
    }
    end
}

fn classify_scope_value(bytes: &[u8], start: usize, end: usize) -> ScopeKind {
    let mut i = skip_trivia(bytes, start);
    if i >= end {
        return ScopeKind::Other;
    }
    if let Some(after) = strip_kw(bytes, i, end, b"async") {
        i = skip_trivia(bytes, after);
        if i >= end {
            return ScopeKind::Other;
        }
    }
    if let Some(after) = strip_kw(bytes, i, end, b"function") {
        let mut j = skip_trivia(bytes, after);
        if j < end && bytes[j] == b'*' {
            j = skip_trivia(bytes, j + 1);
        }
        if j < end && is_ident_start(bytes[j]) {
            while j < end && is_ident_cont(bytes[j]) {
                j += 1;
            }
            j = skip_trivia(bytes, j);
        }
        if j < end && bytes[j] == b'<' {
            j = skip_generic_args(bytes, j, end);
            j = skip_trivia(bytes, j);
        }
        if j < end && bytes[j] == b'(' {
            return ScopeKind::Action;
        }
        return ScopeKind::Other;
    }
    if bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let mut after = skip_trivia(bytes, close);
        if after < end && bytes[after] == b':' {
            after = skip_type_annotation(bytes, after + 1, end);
            after = skip_trivia(bytes, after);
        }
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            return ScopeKind::Action;
        }
    }
    if is_ident_start(bytes[i]) {
        let s = i;
        while i < end && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let word = std::str::from_utf8(&bytes[s..i]).unwrap_or("");
        let mut after = skip_trivia(bytes, i);
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            return ScopeKind::Action;
        }
        if after < end && bytes[after] == b'<' {
            after = skip_generic_args(bytes, after, end);
            after = skip_trivia(bytes, after);
        }
        if after < end && bytes[after] == b'(' {
            return match word {
                "ref" => ScopeKind::Ref,
                "signal" => ScopeKind::Signal,
                "derived" | "computed" => ScopeKind::Derived,
                _ => ScopeKind::Other,
            };
        }
    }
    ScopeKind::Other
}

/// If `bytes[start..]` begins with `kw` (followed by a non-ident
/// boundary or end-of-slice), return the index past the keyword.
fn strip_kw(bytes: &[u8], start: usize, end: usize, kw: &[u8]) -> Option<usize> {
    if start + kw.len() > end {
        return None;
    }
    if &bytes[start..start + kw.len()] != kw {
        return None;
    }
    let after = start + kw.len();
    if after < end && is_ident_cont(bytes[after]) {
        return None;
    }
    Some(after)
}

/// Walk a TS type annotation starting at the character after the
/// leading `:`. Stops when an `=>` token or top-level `=`/`,`/`)`/`}`
/// is reached.
fn skip_type_annotation(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth: i32 = 0;
    while k < end {
        let c = bytes[k];
        if depth == 0 {
            if c == b'=' && k + 1 < end && bytes[k + 1] == b'>' {
                return k;
            }
            if matches!(c, b',' | b';' | b'=' | b')' | b'}' | b']') {
                return k;
            }
        }
        match c {
            b'<' | b'(' | b'{' | b'[' => depth += 1,
            b'>' | b')' | b'}' | b']' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return k;
        }
        k += 1;
    }
    end
}

/// Skip a TypeScript generic-args block starting at `bytes[start] == b'<'`.
fn skip_generic_args(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth = 0i32;
    while k < end {
        match bytes[k] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return k + 1;
                }
            }
            b'"' | b'\'' | b'`' => return end,
            _ => {}
        }
        k += 1;
    }
    end
}

/// Advance past a string / template literal starting at `i` (which must
/// point at the opening quote). Handles `\` escapes and `${…}` template
/// substitutions. Returns the offset just past the closing quote.
fn skip_string(bytes: &[u8], i: usize, end: usize, quote: u8) -> usize {
    let mut k = i + 1;
    while k < end {
        let c = bytes[k];
        if c == b'\\' {
            k = (k + 2).min(end);
            continue;
        }
        if c == quote {
            return k + 1;
        }
        if quote == b'`' && c == b'$' && k + 1 < end && bytes[k + 1] == b'{' {
            k = skip_balanced(bytes, k + 1, end);
            continue;
        }
        k += 1;
    }
    end
}

/// Walk `bytes[start..end]` (which spans the setup function's block body,
/// starting at its opening `{`) looking for the last top-level `return {`
/// outside of strings, template literals, and comments. Returns the
/// offset of the `{`.
fn find_last_top_level_return_object(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    if start >= end || bytes[start] != b'{' {
        return None;
    }
    let return_kw = b"return";
    let target_depth: i32 = 1;
    let mut i = start;
    let mut last: Option<usize> = None;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    while i < end {
        let c = bytes[i];
        if c == b'"' || c == b'\'' || c == b'`' {
            i = skip_string(bytes, i, end, c);
            continue;
        }
        if c == b'/' && i + 1 < end && bytes[i + 1] == b'/' {
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && i + 1 < end && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(end);
            continue;
        }
        if c == b'{' {
            brace_depth += 1;
            i += 1;
            continue;
        }
        if c == b'}' {
            brace_depth -= 1;
            i += 1;
            if brace_depth < target_depth {
                break;
            }
            continue;
        }
        if c == b'(' {
            paren_depth += 1;
            i += 1;
            continue;
        }
        if c == b')' {
            paren_depth -= 1;
            i += 1;
            continue;
        }
        if brace_depth == target_depth
            && paren_depth == 0
            && i + return_kw.len() <= end
            && bytes[i..].starts_with(return_kw)
        {
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let next = i + return_kw.len();
            let post_ok = next >= end || !is_ident_cont(bytes[next]);
            if prev_ok && post_ok {
                let after = skip_trivia(bytes, next);
                if after < end && bytes[after] == b'{' {
                    last = Some(after);
                }
            }
        }
        i += 1;
    }
    last
}

/// Visit `key: value` entries within an object literal slice. The
/// callback receives the key name, the key's byte offset, and the
/// value-span range.
fn walk_object_entries<F>(bytes: &[u8], start: usize, end: usize, mut f: F)
where
    F: FnMut(&str, usize, usize, usize),
{
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
        let key_start = i;
        let key;
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            let s = i;
            while i < end && bytes[i] != q {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            key = std::str::from_utf8(&bytes[s..i]).unwrap_or("").to_string();
            if i < end {
                i += 1;
            }
        } else if is_ident_start(bytes[i]) {
            while i < end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            key = std::str::from_utf8(&bytes[key_start..i]).unwrap_or("").to_string();
        } else {
            i = next_entry(bytes, i, end);
            continue;
        }
        let after_key = i;
        i = skip_trivia(bytes, i);
        if i < end && bytes[i] == b',' {
            f(&key, key_start, key_start, after_key);
            i += 1;
            continue;
        }
        if i >= end || bytes[i] != b':' {
            f(&key, key_start, key_start, after_key);
            i = next_entry(bytes, i, end);
            continue;
        }
        i += 1;
        i = skip_trivia(bytes, i);
        let value_start = i;
        let value_end = next_entry(bytes, i, end);
        f(&key, key_start, value_start, value_end);
        i = value_end;
        if i < end && bytes[i] == b',' {
            i += 1;
        }
    }
}

/// Walk to the next top-level `,` or to `end`.
fn next_entry(bytes: &[u8], start: usize, end: usize) -> usize {
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
            b'(' | b'{' | b'[' => {
                k = skip_balanced(bytes, k, end);
            }
            b',' => return k,
            _ => k += 1,
        }
    }
    end
}
