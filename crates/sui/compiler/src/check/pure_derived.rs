//! Pure-`derived` rule: each entry in `controller({ derived: {...} })`
//! must be a side-effect-free getter. The runtime memo-evaluates
//! deriveds on signal reads — anything async or impure breaks that
//! invariant.
//!
//! Forbidden patterns (textual scan, string-aware):
//!   * `await` keyword
//!   * `Date.now(`, `Math.random(`
//!   * `setTimeout`, `setInterval` (any call)
//!   * `Promise.` member access
//!   * `fetch(` call
//!   * `ship.` member access (SDK calls = side effects)
//!
//! The job spec (phase 3) lists these as the canonical bad set. Move
//! work that needs them to `mount()` or `use()` instead.

use crate::check::controller_parse::ParsedController;
use crate::check::js_scan::{
    is_ident_cont, is_ident_start, next_top_separator, skip_balanced, skip_trivia,
};
use crate::diagnostics::{CompileError, Span};

struct Pattern {
    /// What we render in the error message.
    label: &'static str,
    /// Token text. The match must be at an identifier boundary.
    token: &'static str,
    /// When `true`, also require an immediately-following `.` (member
    /// access). When `false`, no extra requirement.
    require_dot: bool,
    /// When `true`, also require an immediately-following `(`. Mutex
    /// with `require_dot`.
    require_call: bool,
}

const PATTERNS: &[Pattern] = &[
    Pattern { label: "await",         token: "await",        require_dot: false, require_call: false },
    Pattern { label: "Date.now()",    token: "Date.now",     require_dot: false, require_call: true },
    Pattern { label: "Math.random()", token: "Math.random",  require_dot: false, require_call: true },
    Pattern { label: "setTimeout()",  token: "setTimeout",   require_dot: false, require_call: true },
    Pattern { label: "setInterval()", token: "setInterval",  require_dot: false, require_call: true },
    Pattern { label: "Promise.*",     token: "Promise",      require_dot: true,  require_call: false },
    Pattern { label: "fetch()",       token: "fetch",        require_dot: false, require_call: true },
    Pattern { label: "ship.*",        token: "ship",         require_dot: true,  require_call: false },
];

pub fn check(controller: &ParsedController) -> Result<(), CompileError> {
    let Some(derived) = controller.section("derived") else {
        return Ok(());
    };
    // The value span brackets `{ ... }`. Walk its inner top-level
    // entries; each entry's body is scanned for the forbidden patterns.
    let value = controller.slice(derived.value_span);
    if !value.starts_with('{') || !value.ends_with('}') {
        // Atypical shape (`derived: someFactory(...)`) — out of scope
        // for this check; let lowering deal with it.
        return Ok(());
    }
    let inner_start = derived.value_span.start + 1;
    let inner_end = derived.value_span.end - 1;
    let entries = parse_entries(&controller.source, inner_start, inner_end);
    for entry in entries {
        if let Some((label, span)) = scan_for_forbidden(&controller.source, entry.body_span) {
            return Err(CompileError::new(format!(
                "Derived '{}' contains side effect '{}'. Move to mount() or use().",
                entry.name, label
            ))
            .with_span(span)
            .with_path(controller.path.clone())
            .with_category(format!("derived.{}", entry.name)));
        }
    }
    Ok(())
}

struct Entry {
    name: String,
    /// Span over the value text (e.g. the arrow body, or the entire
    /// arrow expression — anything after the `:` and before the
    /// trailing `,`/`}`).
    body_span: Span,
}

fn parse_entries(src: &str, start: usize, end: usize) -> Vec<Entry> {
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
        let key_start = i;
        let key;
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            let s = i;
            while i < end && bytes[i] != q {
                if bytes[i] == b'\\' { i += 2; continue; }
                i += 1;
            }
            key = src[s..i].to_string();
            if i < end { i += 1; }
        } else if is_ident_start(bytes[i]) {
            while i < end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            key = src[key_start..i].to_string();
        } else {
            i = next_top_separator(bytes, i, end);
            if i < end && bytes[i] == b',' { i += 1; }
            continue;
        }
        i = skip_trivia(bytes, i);
        // Method form: `foo(args) { body }` — we treat both args + body
        // as the body span.
        if i < end && bytes[i] == b'(' {
            let body_start = i;
            i = skip_balanced(bytes, i, end);
            i = skip_trivia(bytes, i);
            if i < end && bytes[i] == b'{' {
                i = skip_balanced(bytes, i, end);
            }
            out.push(Entry { name: key, body_span: Span::new(body_start, i) });
            i = skip_trivia(bytes, i);
            if i < end && bytes[i] == b',' { i += 1; }
            continue;
        }
        if i >= end || bytes[i] != b':' {
            // Shorthand or unrecognised — skip.
            i = next_top_separator(bytes, i, end);
            if i < end && bytes[i] == b',' { i += 1; }
            continue;
        }
        i += 1;
        i = skip_trivia(bytes, i);
        let body_start = i;
        let body_end = next_top_separator(bytes, i, end);
        out.push(Entry { name: key, body_span: Span::new(body_start, body_end) });
        i = body_end;
        if i < end && bytes[i] == b',' { i += 1; }
    }
    out
}

fn scan_for_forbidden(src: &str, span: Span) -> Option<(&'static str, Span)> {
    let bytes = src.as_bytes();
    let mut i = span.start;
    let end = span.end.min(bytes.len());
    let mut in_str: Option<u8> = None;
    while i < end {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' { i += 2; continue; }
            if c == q { in_str = None; }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); i += 1; continue; }
            b'/' if i + 1 < end && bytes[i + 1] == b'/' => {
                while i < end && bytes[i] != b'\n' { i += 1; }
                continue;
            }
            b'/' if i + 1 < end && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') { i += 1; }
                if i + 1 < end { i += 2; }
                continue;
            }
            _ => {}
        }
        if !is_ident_start(c) {
            i += 1;
            continue;
        }
        // Pick the longest matching pattern at this position.
        let mut hit: Option<(&Pattern, usize)> = None;
        for pat in PATTERNS {
            if !at_token(src, i, pat.token) {
                continue;
            }
            let after = i + pat.token.len();
            if pat.require_dot
                && (after >= end || bytes[after] != b'.') {
                    continue;
                }
            if pat.require_call {
                let mut k = after;
                while k < end && bytes[k].is_ascii_whitespace() { k += 1; }
                if k >= end || bytes[k] != b'(' {
                    continue;
                }
            }
            // No suffix needed: just ensure the token isn't part of a
            // longer identifier. `at_token` already handles that.
            if hit.map(|(h, _)| h.token.len()).unwrap_or(0) < pat.token.len() {
                hit = Some((pat, after));
            }
        }
        if let Some((pat, after)) = hit {
            return Some((pat.label, Span::new(i, after)));
        }
        // Advance past this identifier so we don't re-scan it.
        let mut k = i;
        while k < end && is_ident_cont(bytes[k]) { k += 1; }
        // Allow chained property access (`Date.now`) to be treated as
        // one identifier-like span by patterns above; for advancement
        // we still skip just the leading ident.
        i = if k > i { k } else { i + 1 };
    }
    None
}

/// Match `token` exactly at `i`, with no preceding/following ident
/// continuation — except `token` may itself contain `.` as part of a
/// dotted path (`Date.now`). Each `.`-separated segment must be at an
/// identifier boundary.
fn at_token(src: &str, i: usize, token: &str) -> bool {
    let bytes = src.as_bytes();
    if i + token.len() > bytes.len() {
        return false;
    }
    if !src[i..].starts_with(token) {
        return false;
    }
    // Pre: the byte before `i` must not be ident-continue, OR `i` is
    // at the start.
    if i > 0 && is_ident_cont(bytes[i - 1]) {
        return false;
    }
    // Post: the byte after the token must not be ident-continue.
    let after = i + token.len();
    if after < bytes.len() && is_ident_cont(bytes[after]) {
        return false;
    }
    true
}


