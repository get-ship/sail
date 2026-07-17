//! Pure-`derived` rule: every scope member classified as
//! [`ScopeKind::Derived`] (i.e. every `derived(() => …)` / `computed(() =>
//! …)` call in a controller's returned scope) must be a side-effect-free
//! getter. The runtime memo-evaluates deriveds on signal reads — anything
//! async or impure breaks that invariant.
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

use crate::check::controller_parse::{ParsedController, ScopeKind};
use crate::check::js_scan::is_ident_cont;
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
    for member in &controller.returned {
        if member.kind != ScopeKind::Derived {
            continue;
        }
        if let Some((label, span)) = scan_for_forbidden(&controller.source, member.value_span) {
            return Err(CompileError::new(format!(
                "Derived '{}' contains side effect '{}'. Move to mount() or use().",
                member.name, label
            ))
            .with_span(span)
            .with_path(controller.path.clone())
            .with_category(format!("derived.{}", member.name)));
        }
    }
    Ok(())
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
        if !is_ident_start_byte(c) {
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
        i = if k > i { k } else { i + 1 };
    }
    None
}

fn is_ident_start_byte(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b'$'
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
    if i > 0 && is_ident_cont(bytes[i - 1]) {
        return false;
    }
    let after = i + token.len();
    if after < bytes.len() && is_ident_cont(bytes[after]) {
        return false;
    }
    true
}
