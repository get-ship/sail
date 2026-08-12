//! Anonymous diagnostic ratchet (SPEC-LINTS-1 Phase 0).
//!
//! Every compiler throw site must carry a stable `sail-*` id. Anonymous
//! sites must stay exactly 0; new unregistered diagnostics fail CI.
//!
//! # API
//!
//! | Symbol | Contract |
//! |--------|----------|
//! | [`is_stable_rule_id`] | `true` iff `id` matches `^sail-[a-z0-9-]+$` |
//! | [`classify_snippet`] | Classify a throw-site source window as id'd or anonymous |
//! | [`count_anonymous_sites`] | Scan live `crates/sail/compiler/src/**/*.rs` throw sites |
//! | [`idd_rule_ids`] | Unique `sail-*` ids attached at live Idd sites |
//! | [`anonymous_baseline`] | Product pin (`0`); do not raise |
//!
//! A construction site is **id'd** iff its emitted rule/category resolves to a
//! stable id matching `^sail-[a-z0-9-]+$` via `.with_category("sail-…")`,
//! `rule: Some("sail-…")`, or a same-file `const` that is that string.
//!
//! Everything else (missing category, free-form location strings, `expression`,
//! `manifest`, `for-iterable`, bare messages) is **anonymous**.
//!
//! Throw construction patterns to count:
//! - `CompileError::new(`
//! - `Diagnostic {` message-bearing product emissions (not the type definition)
//!
//! # Baseline policy
//!
//! Assert `count_anonymous_sites() == 0`. Do not raise [`ANONYMOUS_BASELINE`].
//! Attach a `sail-*` id and a DiagSpec row to every new throw site.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Classification of one diagnostic throw construction site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteClass {
    /// Rule/category resolves to `^sail-[a-z0-9-]+$`.
    Idd,
    /// Missing, free-form, or non-`sail-*` category/rule.
    Anonymous,
}

/// Product pin: anonymous sites must stay exactly 0. Do not raise this.
const ANONYMOUS_BASELINE: usize = 0;

/// Return `true` when `id` is a stable sail rule id (`^sail-[a-z0-9-]+$`).
pub fn is_stable_rule_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("sail-") else {
        return false;
    };
    !rest.is_empty()
        && rest
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Classify a throw-site source window (fixture snippet or live file slice).
///
/// Detects id attachment on **this** construction only (not a neighbor):
/// - `.with_category("sail-…")` string literal
/// - `.with_category(CONST)` resolving to `sail-…` via same-file const
/// - `rule: Some("sail-…")` or `rule: Some(CONST)` resolving to `sail-…`
///
/// When `source` contains a construction site, classification is scoped to that
/// site's statement (`CompileError` builder chain) or brace block (`Diagnostic`).
/// Missing category / free-form categories → [`SiteClass::Anonymous`].
pub fn classify_snippet(source: &str) -> SiteClass {
    let consts = collect_str_consts(source);
    let window = first_site_window(source).unwrap_or(source);
    classify_with_consts(window, &consts)
}

/// Count anonymous throw sites under live `crates/sail/compiler/src/**/*.rs`.
pub fn count_anonymous_sites() -> usize {
    anonymous_site_locations().len()
}

/// Stable `sail-*` ids attached at live Idd throw sites (unique, sorted).
pub fn idd_rule_ids() -> Vec<String> {
    let mut ids: Vec<String> = scan_compiler_src()
        .into_iter()
        .filter(|s| s.class == SiteClass::Idd)
        .filter_map(|s| s.id)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// File + line of each anonymous throw site (for ratchet diagnosis).
pub fn anonymous_site_locations() -> Vec<(String, usize)> {
    scan_compiler_src()
        .into_iter()
        .filter(|s| s.class == SiteClass::Anonymous)
        .map(|s| {
            (
                s.path
                    .strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"))
                    .unwrap_or(&s.path)
                    .display()
                    .to_string(),
                s.line,
            )
        })
        .collect()
}

/// Product pin. Must stay 0 — do not raise [`ANONYMOUS_BASELINE`].
pub fn anonymous_baseline() -> usize {
    ANONYMOUS_BASELINE
}

// ---------------------------------------------------------------------------
// Classifier internals
// ---------------------------------------------------------------------------

fn classify_with_consts(source: &str, consts: &HashMap<String, String>) -> SiteClass {
    if resolved_id(source, consts).is_some() {
        SiteClass::Idd
    } else {
        SiteClass::Anonymous
    }
}

fn resolved_id(source: &str, consts: &HashMap<String, String>) -> Option<String> {
    if let Some(id) = extract_with_category(source) {
        return resolved_stable_id(&id, consts);
    }
    if let Some(id) = extract_rule_some(source) {
        return resolved_stable_id(&id, consts);
    }
    None
}

fn resolved_stable_id(token: &str, consts: &HashMap<String, String>) -> Option<String> {
    if is_stable_rule_id(token) {
        return Some(token.to_string());
    }
    consts.get(token).filter(|v| is_stable_rule_id(v)).cloned()
}

/// Collect `const NAME: &str = "…"` (optionally `pub`) string bindings.
fn collect_str_consts(source: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip comments / strings so we don't harvest from docs.
        if let Some(next) = skip_non_code(bytes, i) {
            i = next_char_boundary(source, next);
            continue;
        }
        if match_keyword_at(bytes, i, b"const") {
            let after_const = i + 5;
            if after_const < bytes.len() && is_ident_byte(bytes[after_const]) {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            let mut j = skip_ws(bytes, after_const);
            // optional nothing; name
            let name_start = j;
            while j < bytes.len() && is_ident_byte(bytes[j]) {
                j += 1;
            }
            if j == name_start {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            let name = &source[name_start..j];
            j = skip_ws(bytes, j);
            if j >= bytes.len() || bytes[j] != b':' {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            j = skip_ws(bytes, j + 1);
            // expect &str (with optional whitespace: `& str`)
            if j >= bytes.len() || !source[j..].starts_with('&') {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            j = skip_ws(bytes, j + 1);
            if j >= bytes.len() || !source[j..].starts_with("str") {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            j += 3;
            j = skip_ws(bytes, j);
            if j >= bytes.len() || bytes[j] != b'=' {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            j = skip_ws(bytes, j + 1);
            if j >= bytes.len() || bytes[j] != b'"' {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            if let Some((lit, next)) = parse_double_string(source, j) {
                map.insert(name.to_string(), lit);
                i = next;
                continue;
            }
        }
        // also accept `pub const`
        if match_keyword_at(bytes, i, b"pub") {
            let k = skip_ws(bytes, i + 3);
            if match_keyword_at(bytes, k, b"const") {
                i = k;
                continue;
            }
        }
        i = next_char_boundary(source, i + 1);
    }
    map
}

/// `.with_category("…")` or `.with_category(IDENT…)`
fn extract_with_category(source: &str) -> Option<String> {
    let needle = ".with_category";
    let mut search_from = 0;
    while let Some(rel) = source[search_from..].find(needle) {
        let at = search_from + rel;
        let after = at + needle.len();
        let bytes = source.as_bytes();
        let mut j = skip_ws(bytes, after);
        if j >= bytes.len() || bytes[j] != b'(' {
            search_from = after;
            continue;
        }
        j = skip_ws(bytes, j + 1);
        if j >= bytes.len() {
            return None;
        }
        if bytes[j] == b'"' {
            if let Some((lit, _)) = parse_double_string(source, j) {
                return Some(lit);
            }
            search_from = after;
            continue;
        }
        // identifier (const ref); ignore method chains after, e.g. RULE.to_string()
        let name_start = j;
        if !is_ident_start(bytes[j]) {
            search_from = after;
            continue;
        }
        j += 1;
        while j < bytes.len() && is_ident_byte(bytes[j]) {
            j += 1;
        }
        return Some(source[name_start..j].to_string());
    }
    None
}

/// `rule: Some("…")` or `rule: Some(IDENT…)`
fn extract_rule_some(source: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(rel) = source[search_from..].find("rule") {
        let at = search_from + rel;
        // word boundary: previous char not ident
        if at > 0 && is_ident_byte(source.as_bytes()[at - 1]) {
            search_from = at + 4;
            continue;
        }
        let bytes = source.as_bytes();
        let mut j = at + 4;
        if j < bytes.len() && is_ident_byte(bytes[j]) {
            search_from = at + 4;
            continue;
        }
        j = skip_ws(bytes, j);
        if j >= bytes.len() || bytes[j] != b':' {
            search_from = at + 4;
            continue;
        }
        j = skip_ws(bytes, j + 1);
        if !source[j..].starts_with("Some") {
            search_from = at + 4;
            continue;
        }
        j += 4;
        j = skip_ws(bytes, j);
        if j >= bytes.len() || bytes[j] != b'(' {
            search_from = at + 4;
            continue;
        }
        j = skip_ws(bytes, j + 1);
        if j >= bytes.len() {
            return None;
        }
        if bytes[j] == b'"' {
            if let Some((lit, _)) = parse_double_string(source, j) {
                return Some(lit);
            }
            search_from = at + 4;
            continue;
        }
        let name_start = j;
        if !is_ident_start(bytes[j]) {
            search_from = at + 4;
            continue;
        }
        j += 1;
        while j < bytes.len() && is_ident_byte(bytes[j]) {
            j += 1;
        }
        return Some(source[name_start..j].to_string());
    }
    None
}

// ---------------------------------------------------------------------------
// Live source scan
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Site {
    #[allow(dead_code)]
    path: PathBuf,
    #[allow(dead_code)]
    line: usize,
    class: SiteClass,
    id: Option<String>,
}

fn scan_compiler_src() -> Vec<Site> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sites = Vec::new();
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    files.sort();
    for path in files {
        let Ok(src) = fs::read_to_string(&path) else {
            continue;
        };
        scan_file(&path, &src, &mut sites);
    }
    sites
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn scan_file(path: &Path, src: &str, sites: &mut Vec<Site>) {
    let consts = collect_str_consts(src);
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        debug_assert!(src.is_char_boundary(i));
        if let Some(next) = skip_non_code(bytes, i) {
            i = next_char_boundary(src, next);
            continue;
        }
        // CompileError::new(
        if src[i..].starts_with("CompileError::new(") {
            let line = line_number(src, i);
            let window = compile_error_statement_window(src, i);
            let id = resolved_id(window, &consts);
            let class = if id.is_some() {
                SiteClass::Idd
            } else {
                SiteClass::Anonymous
            };
            sites.push(Site {
                path: path.to_path_buf(),
                line,
                class,
                id,
            });
            i += "CompileError::new(".len();
            continue;
        }
        // Diagnostic {  (message-bearing product emissions only)
        if src[i..].starts_with("Diagnostic") {
            let after_name = i + "Diagnostic".len();
            // not DiagnosticTier / Diagnostic:: / etc.
            if after_name < bytes.len() && is_ident_byte(bytes[after_name]) {
                i = next_char_boundary(src, i + 1);
                continue;
            }
            // Foreign types (`sail_layout::Diagnostic`) are not compiler throw sites.
            if i >= 2 && bytes[i - 1] == b':' && bytes[i - 2] == b':' {
                i = next_char_boundary(src, i + 1);
                continue;
            }
            let j = skip_ws(bytes, after_name);
            if j < bytes.len() && bytes[j] == b'{' {
                let line_start = src[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let line_end = src[i..].find('\n').map(|p| i + p).unwrap_or(src.len());
                let line_text = &src[line_start..line_end];
                // skip type definitions: `pub struct Diagnostic {`
                if line_text.contains("struct") {
                    i = next_char_boundary(src, j + 1);
                    continue;
                }
                let window = diagnostic_brace_window(src, i, j);
                // product emissions set `message:`; type fields alone aren't emissions
                if has_message_field(window) {
                    let line = line_number(src, i);
                    let id = resolved_id(window, &consts);
                    let class = if id.is_some() {
                        SiteClass::Idd
                    } else {
                        SiteClass::Anonymous
                    };
                    sites.push(Site {
                        path: path.to_path_buf(),
                        line,
                        class,
                        id,
                    });
                }
                i = next_char_boundary(src, j + 1);
                continue;
            }
        }
        i = next_char_boundary(src, i + 1);
    }
}

/// Advance `i` to the next UTF-8 char boundary at or after `i`.
fn next_char_boundary(src: &str, mut i: usize) -> usize {
    let len = src.len();
    if i >= len {
        return len;
    }
    while i < len && !src.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn has_message_field(window: &str) -> bool {
    let bytes = window.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(next) = skip_non_code(bytes, i) {
            i = next_char_boundary(window, next);
            continue;
        }
        if window[i..].starts_with("message") {
            let after = i + "message".len();
            if after < bytes.len() && is_ident_byte(bytes[after]) {
                i = next_char_boundary(window, i + 1);
                continue;
            }
            let j = skip_ws(bytes, after);
            if j < bytes.len() && bytes[j] == b':' {
                // Type-def field is `message: String`; emissions assign a value.
                let k = skip_ws(bytes, j + 1);
                if k < bytes.len() {
                    let rest = &window[k..];
                    // skip pure type annotations used only in struct defs
                    if rest.starts_with("String") {
                        let after_ty = k + "String".len();
                        if after_ty >= bytes.len()
                            || matches!(bytes[after_ty], b',' | b'\n' | b'/' | b'}')
                            || bytes[after_ty].is_ascii_whitespace()
                        {
                            i = next_char_boundary(window, after);
                            continue;
                        }
                    }
                    return true;
                }
            }
        }
        i = next_char_boundary(window, i + 1);
    }
    false
}

fn line_number(src: &str, offset: usize) -> usize {
    src[..offset].bytes().filter(|&b| b == b'\n').count() + 1
}

/// First construction site's scoped window in `source`, if any.
///
/// Used by [`classify_snippet`] so multi-site fixture strings do not inherit a
/// neighbor's category/rule.
fn first_site_window(source: &str) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(next) = skip_non_code(bytes, i) {
            i = next_char_boundary(source, next);
            continue;
        }
        if source[i..].starts_with("CompileError::new(") {
            return Some(compile_error_statement_window(source, i));
        }
        if source[i..].starts_with("Diagnostic") {
            let after_name = i + "Diagnostic".len();
            if after_name < bytes.len() && is_ident_byte(bytes[after_name]) {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            if i >= 2 && bytes[i - 1] == b':' && bytes[i - 2] == b':' {
                i = next_char_boundary(source, i + 1);
                continue;
            }
            let j = skip_ws(bytes, after_name);
            if j < bytes.len() && bytes[j] == b'{' {
                let line_start = source[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let line_end = source[i..]
                    .find('\n')
                    .map(|p| i + p)
                    .unwrap_or(source.len());
                let line_text = &source[line_start..line_end];
                if line_text.contains("struct") {
                    i = next_char_boundary(source, j + 1);
                    continue;
                }
                return Some(diagnostic_brace_window(source, i, j));
            }
        }
        i = next_char_boundary(source, i + 1);
    }
    None
}

/// Scope a `CompileError::new(` site to **this** statement / builder chain only.
///
/// Walks from `start` (at `CompileError`) tracking paren/bracket/brace depth.
/// Ends at:
/// - `;` at relative depth 0 (statement terminator), or
/// - `,` at relative depth 0 after the call has opened (arg / match-arm separator), or
/// - a closing `)`/`]`/`}` that would pop below relative depth 0 (outer context), or
/// - EOF.
///
/// Never extends into a neighboring throw site.
fn compile_error_statement_window(src: &str, start: usize) -> &str {
    let bytes = src.as_bytes();
    let mut i = start;
    let mut depth: i32 = 0;
    let mut opened = false;
    while i < bytes.len() {
        if let Some(next) = skip_non_code(bytes, i) {
            i = next;
            continue;
        }
        match bytes[i] {
            b'(' | b'[' | b'{' => {
                depth += 1;
                opened = true;
                i += 1;
            }
            b')' | b']' | b'}' => {
                if depth == 0 {
                    // Closes an outer delimiter opened before this site.
                    return &src[start..i];
                }
                depth -= 1;
                i += 1;
            }
            b';' if opened && depth == 0 => {
                return &src[start..i + 1];
            }
            b',' if opened && depth == 0 => {
                return &src[start..i];
            }
            _ => {
                i = next_char_boundary(src, i + 1);
            }
        }
    }
    &src[start..]
}

/// Scope a `Diagnostic { … }` site to the matching brace block only.
///
/// `start` points at `Diagnostic`; `brace_open` at the opening `{`.
/// Includes from `Diagnostic` through the matching `}` so `rule: Some(…)` is
/// visible without neighboring struct literals.
fn diagnostic_brace_window(src: &str, start: usize, brace_open: usize) -> &str {
    let bytes = src.as_bytes();
    let mut i = brace_open;
    let mut depth: i32 = 0;
    while i < bytes.len() {
        if let Some(next) = skip_non_code(bytes, i) {
            i = next;
            continue;
        }
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    return &src[start..i];
                }
            }
            _ => {
                i = next_char_boundary(src, i + 1);
            }
        }
    }
    &src[start..]
}

// ---------------------------------------------------------------------------
// Lex helpers
// ---------------------------------------------------------------------------

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn match_keyword_at(bytes: &[u8], i: usize, kw: &[u8]) -> bool {
    if i + kw.len() > bytes.len() {
        return false;
    }
    if &bytes[i..i + kw.len()] != kw {
        return false;
    }
    if i > 0 && is_ident_byte(bytes[i - 1]) {
        return false;
    }
    let after = i + kw.len();
    if after < bytes.len() && is_ident_byte(bytes[after]) {
        return false;
    }
    true
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// If `i` starts a comment or string, return the index just past it.
fn skip_non_code(bytes: &[u8], i: usize) -> Option<usize> {
    if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
        let mut j = i + 2;
        while j < bytes.len() && bytes[j] != b'\n' {
            j += 1;
        }
        return Some(j);
    }
    if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
        let mut j = i + 2;
        while j + 1 < bytes.len() {
            if bytes[j] == b'*' && bytes[j + 1] == b'/' {
                return Some(j + 2);
            }
            j += 1;
        }
        return Some(bytes.len());
    }
    if bytes[i] == b'"' {
        return skip_double_string_bytes(bytes, i);
    }
    if bytes[i] == b'\'' {
        let mut j = i + 1;
        while j < bytes.len() {
            if bytes[j] == b'\\' {
                j += 2;
                continue;
            }
            if bytes[j] == b'\'' {
                return Some(j + 1);
            }
            j += 1;
        }
        return Some(bytes.len());
    }
    // raw strings r"…" / r#"…"# / br"…"
    if bytes[i] == b'r' || (bytes[i] == b'b' && i + 1 < bytes.len() && bytes[i + 1] == b'r') {
        let mut j = if bytes[i] == b'b' { i + 2 } else { i + 1 };
        let mut hashes = 0;
        while j < bytes.len() && bytes[j] == b'#' {
            hashes += 1;
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'"' {
            j += 1;
            loop {
                if j >= bytes.len() {
                    return Some(bytes.len());
                }
                if bytes[j] == b'"' {
                    let mut k = 0;
                    while k < hashes && j + 1 + k < bytes.len() && bytes[j + 1 + k] == b'#' {
                        k += 1;
                    }
                    if k == hashes {
                        return Some(j + 1 + hashes);
                    }
                }
                j += 1;
            }
        }
    }
    None
}

fn skip_double_string_bytes(bytes: &[u8], i: usize) -> Option<usize> {
    let mut j = i + 1;
    while j < bytes.len() {
        if bytes[j] == b'\\' {
            j += 2;
            continue;
        }
        if bytes[j] == b'"' {
            return Some(j + 1);
        }
        j += 1;
    }
    Some(bytes.len())
}

fn parse_double_string(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    if start >= bytes.len() || bytes[start] != b'"' {
        return None;
    }
    let mut j = start + 1;
    let mut out = String::new();
    while j < bytes.len() {
        match bytes[j] {
            b'\\' => {
                j += 1;
                if j >= bytes.len() {
                    break;
                }
                let esc = match bytes[j] {
                    b'n' => '\n',
                    b'r' => '\r',
                    b't' => '\t',
                    b'0' => '\0',
                    b'\\' => '\\',
                    b'"' => '"',
                    b'\'' => '\'',
                    other => other as char,
                };
                out.push(esc);
                j += 1;
            }
            b'"' => return Some((out, j + 1)),
            c => {
                out.push(c as char);
                j += 1;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_ids() {
        assert!(is_stable_rule_id("sail-foo"));
        assert!(is_stable_rule_id("sail-light-shadow-root"));
        assert!(is_stable_rule_id("sail-internal-lint-gate"));
        assert!(!is_stable_rule_id("expression"));
        assert!(!is_stable_rule_id(""));
        assert!(!is_stable_rule_id("sail-"));
        assert!(!is_stable_rule_id("Sail-foo"));
    }

    #[test]
    fn classify_literal_category() {
        let src = r#"
            CompileError::new("bad")
                .with_category("sail-query-key")
        "#;
        assert_eq!(classify_snippet(src), SiteClass::Idd);
    }

    #[test]
    fn classify_bare_new() {
        assert_eq!(
            classify_snippet(r#"CompileError::new("x")"#),
            SiteClass::Anonymous
        );
    }

    #[test]
    fn classify_rule_some() {
        let src = r#"
            Diagnostic {
                message: "m".into(),
                rule: Some("sail-layout".into()),
            }
        "#;
        assert_eq!(classify_snippet(src), SiteClass::Idd);
    }

    #[test]
    fn classify_const_ref() {
        let src = r#"
            const RULE_ID: &str = "sail-paired-controller";
            CompileError::new("x").with_category(RULE_ID)
        "#;
        assert_eq!(classify_snippet(src), SiteClass::Idd);
    }

    #[test]
    fn freeform_category_anonymous() {
        let src = r#"CompileError::new("x").with_category("expression")"#;
        assert_eq!(classify_snippet(src), SiteClass::Anonymous);
    }

    /// Neighbor id attachment must not launder an earlier bare throw.
    ///
    /// Pre-fix: a flat 20-line window would see the second site's
    /// `.with_category("sail-x")` and mis-classify the first as Idd.
    #[test]
    fn neighbor_category_does_not_launder_anonymous() {
        let src = r#"
            fn f() {
                return Err(CompileError::new("bare anonymous"));
                return Err(CompileError::new("id'd neighbor")
                    .with_category("sail-neighbor-id"));
            }
        "#;
        assert_eq!(
            classify_snippet(src),
            SiteClass::Anonymous,
            "first CompileError::new must stay anonymous; neighbor sail-* must not leak"
        );

        // Live-scan path: both sites recorded, only the second is Idd.
        let mut sites = Vec::new();
        scan_file(Path::new("fixture.rs"), src, &mut sites);
        assert_eq!(sites.len(), 2, "expected two construction sites");
        assert_eq!(sites[0].class, SiteClass::Anonymous);
        assert_eq!(sites[1].class, SiteClass::Idd);
    }

    #[test]
    fn neighbor_rule_does_not_launder_anonymous_diagnostic() {
        let src = r#"
            out.push(Diagnostic {
                message: "anonymous".into(),
                rule: None,
            });
            out.push(Diagnostic {
                message: "id'd".into(),
                rule: Some("sail-layout".into()),
            });
        "#;
        assert_eq!(
            classify_snippet(src),
            SiteClass::Anonymous,
            "first Diagnostic must stay anonymous; neighbor rule must not leak"
        );

        let mut sites = Vec::new();
        scan_file(Path::new("fixture.rs"), src, &mut sites);
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].class, SiteClass::Anonymous);
        assert_eq!(sites[1].class, SiteClass::Idd);
    }

    /// `sail_layout::Diagnostic` is a foreign type, not a compiler throw site.
    #[test]
    fn foreign_layout_diagnostic_is_not_a_compiler_site() {
        let src = r#"
            let d = sail_layout::Diagnostic {
                message: "layout".into(),
                rule: None,
            };
        "#;
        let mut sites = Vec::new();
        scan_file(Path::new("fixture.rs"), src, &mut sites);
        assert!(
            sites.is_empty(),
            "sail_layout::Diagnostic must not count as a compiler throw site, got {sites:?}"
        );
    }

    #[test]
    fn statement_window_includes_builder_chain() {
        let src = r#"
            CompileError::new("bad")
                .with_span(span)
                .with_category("sail-query-key");
            CompileError::new("other");
        "#;
        assert_eq!(classify_snippet(src), SiteClass::Idd);
    }

    #[test]
    fn baseline_is_zero() {
        let count = count_anonymous_sites();
        assert_eq!(
            count,
            0,
            "anonymous diagnostic sites must be zero (found {count}): {:?}",
            anonymous_site_locations()
        );
        assert_eq!(anonymous_baseline(), 0);
    }
}
