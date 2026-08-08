//! Analysis facade — the read surface the LSP server and agent (MCP) tools
//! consume. One Rust entry point over the compiler's existing intelligence:
//! `check_all` (multi-diagnostics), `SymbolTable` (scope names + kinds +
//! declaration offsets), and an IR walk for tag/event dependencies.
//!
//! No TypeScript, no WASM — the controller `.ts` is read structurally by the
//! `SymbolTable` byte-walker (the same one the type-strengthening checks use).
//! Positions are 1-based, matching the rest of the compiler; the LSP transport
//! converts to 0-based protocol coordinates.

use std::collections::BTreeSet;
use std::path::Path;

use crate::check::controller_parse::ParsedController;
use crate::check::symbol_table::SymbolTable;
use crate::diagnostics::{Diagnostic, line_col};
use crate::ir::*;

/// The source's controller: its inline `<controller>` block. There is no
/// paired-file fallback — a sibling `<stem>.controller.ts` is a compile error
/// (`sail-paired-controller`).
fn inline_controller(
    source: &str,
    source_path: Option<&std::path::Path>,
) -> Option<ParsedController> {
    let body = crate::parse::preprocess::extract_controller_block(source).1?;
    crate::check::controller_parse::parse(
        source_path.map(|p| p.to_path_buf()).unwrap_or_default(),
        body,
    )
}

/// A name in scope for a `.sail` template, with where it came from and (when
/// controller-declared) where to jump to.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Symbol {
    pub name: String,
    /// Binding shape: `data` | `prop` | `signal` | `ref` | `derived` |
    /// `action` | `value` | `on` | `use`.
    pub kind: String,
    /// Origin section: `data` | `prop` | `scope` | `derived` | `on` | `use`.
    pub source: String,
    /// 1-based declaration position in the controller. `(0, 0)` when the
    /// symbol is declared in the `.sail` itself (`<prop>`).
    pub line: usize,
    pub col: usize,
}

/// Hover result for an identifier under the cursor.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Hover {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub line: usize,
    pub col: usize,
}

/// SSR tier badge for a scope binding (TOOL-2 model state section).
///
/// - **S** — server-safe: scope-init + pure subset only
/// - **I** — interactive / intermediate (default)
/// - **C** — client-only: browser API / timer / storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SsrTier {
    S,
    I,
    C,
}

impl SsrTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::S => "S",
            Self::I => "I",
            Self::C => "C",
        }
    }
}

/// One query/mutation key derived from controller source.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryKeyInfo {
    /// Kind: `query` | `mutation` | `invalidate`.
    pub kind: String,
    /// Static member path when resolvable (e.g. `crm.deals.list`).
    pub key: String,
    /// True when the call site has no client-only markers in its thunk body.
    pub ssr_fetchable: bool,
    pub line: usize,
    pub col: usize,
}

/// Everything the tooling needs about one `.sail` source in a single pass.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Analysis {
    pub diagnostics: Vec<Diagnostic>,
    pub symbols: Vec<Symbol>,
    /// Custom-element tags the template instantiates (e.g. `sail-button`).
    pub tags: Vec<String>,
    /// `on:` event names the template handles (e.g. `click`).
    pub events: Vec<String>,
    /// Resolved paired-controller path, when one exists on disk.
    pub controller_path: Option<String>,
    /// SSR tier per returned scope symbol name.
    pub ssr_tiers: Vec<(String, SsrTier)>,
    /// Query keys found in the controller module.
    pub query_keys: Vec<QueryKeyInfo>,
}

/// Full analysis of a `.sail` source: diagnostics + scope symbols + the tag /
/// event dependency surface. `source_path` locates the paired controller.
///
/// Diagnostics use [`crate::DiagnosticTier::OnSave`] (includes taffy). For
/// LSP on-type, call [`analyze_with_tier`] with
/// [`crate::DiagnosticTier::OnType`].
pub fn analyze(
    source: &str,
    source_path: Option<&Path>,
    overrides: &[crate::LintOverride],
) -> Analysis {
    analyze_with_tier(
        source,
        source_path,
        overrides,
        crate::DiagnosticTier::OnSave,
    )
}

/// Like [`analyze`], with an explicit [`crate::DiagnosticTier`]
/// (SPEC-TOOLING-1 §WI6).
pub fn analyze_with_tier(
    source: &str,
    source_path: Option<&Path>,
    overrides: &[crate::LintOverride],
    tier: crate::DiagnosticTier,
) -> Analysis {
    let diagnostics = crate::check_all_with_tier(source, source_path, overrides, None, tier);
    let Ok(program) = crate::parse::parse(source) else {
        // Without an AST we can still return the parse diagnostic.
        return Analysis {
            diagnostics,
            symbols: Vec::new(),
            tags: Vec::new(),
            events: Vec::new(),
            controller_path: None,
            ssr_tiers: Vec::new(),
            query_keys: Vec::new(),
        };
    };
    let controller = inline_controller(source, source_path);
    let table = SymbolTable::build(&program, controller.as_ref());
    let symbols = symbols_from_table(&table, controller.as_ref());
    let (tags, events) = collect_tags_events(&program);
    let ssr_tiers = ssr_tiers_from_controller(controller.as_ref());
    let query_keys = controller
        .as_ref()
        .map(|c| extract_query_keys(&c.source))
        .unwrap_or_default();
    Analysis {
        diagnostics,
        symbols,
        tags,
        events,
        controller_path: controller.as_ref().map(|c| c.path.display().to_string()),
        ssr_tiers,
        query_keys,
    }
}

/// Classify a value/expression slice for SSR tier badges.
///
/// Client markers → **C**; pure derived (no client markers, looks like
/// `derived`/`computed`/literal) → **S**; otherwise **I**.
pub fn classify_ssr_tier(value_src: &str) -> SsrTier {
    let s = value_src;
    if CLIENT_MARKERS.iter().any(|m| contains_token(s, m)) {
        return SsrTier::C;
    }
    let trimmed = s.trim();
    if trimmed.starts_with("derived(")
        || trimmed.starts_with("computed(")
        || trimmed.starts_with("() =>")
        || trimmed.starts_with("function")
    {
        // Pure-looking derived / getter with no client markers.
        if !contains_token(s, "await") && !contains_token(s, "fetch(") {
            return SsrTier::S;
        }
    }
    // Bare signal(initial) with literal-ish init is server-safe for first paint.
    if let Some(rest) = trimmed.strip_prefix("signal(") {
        let inner = rest.trim_end_matches(')').trim();
        if looks_like_literal_init(inner) {
            return SsrTier::S;
        }
    }
    SsrTier::I
}

/// Extract `query` / `mutation` / `invalidate` call sites from controller text.
pub fn extract_query_keys(controller_source: &str) -> Vec<QueryKeyInfo> {
    let bytes = controller_source.as_bytes();
    let mut out = Vec::new();
    let kinds: &[&[u8]] = &[b"query", b"mutation", b"invalidate"];
    let mut i = 0usize;
    while i < bytes.len() {
        let mut hit: Option<(&str, usize)> = None;
        for k in kinds {
            if i + k.len() <= bytes.len() && &bytes[i..i + k.len()] == *k {
                let after = i + k.len();
                let prev_ok = i == 0 || !is_ident(bytes[i - 1]);
                let next_ok = after < bytes.len() && {
                    let mut p = after;
                    while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                        p += 1;
                    }
                    p < bytes.len() && bytes[p] == b'('
                };
                if prev_ok && next_ok {
                    // SAFETY: k is pure ASCII.
                    let kind = std::str::from_utf8(k).unwrap_or("query");
                    hit = Some((kind, after));
                    break;
                }
            }
        }
        let Some((kind, after_name)) = hit else {
            i += 1;
            continue;
        };
        let mut p = after_name;
        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        if p >= bytes.len() || bytes[p] != b'(' {
            i += 1;
            continue;
        }
        let open = p;
        let close = match skip_balanced_paren(bytes, open) {
            Some(c) => c,
            None => {
                i += 1;
                continue;
            }
        };
        // Paren body is between open+1 and close (ASCII delimiters).
        let call_body = std::str::from_utf8(&bytes[open + 1..close]).unwrap_or("");
        let key = extract_dot_chain_key(call_body).unwrap_or_else(|| format!("{kind}(…)"));
        let ssr_fetchable =
            kind != "mutation" && !CLIENT_MARKERS.iter().any(|m| contains_token(call_body, m));
        let (line, col) = line_col(controller_source, i);
        out.push(QueryKeyInfo {
            kind: kind.to_string(),
            key,
            ssr_fetchable,
            line,
            col,
        });
        i = close + 1;
    }
    out
}

const CLIENT_MARKERS: &[&str] = &[
    "window",
    "document",
    "localStorage",
    "sessionStorage",
    "setTimeout",
    "setInterval",
    "requestAnimationFrame",
    "matchMedia",
    "navigator",
    "location.",
    "indexedDB",
    "crypto.subtle",
];

fn ssr_tiers_from_controller(controller: Option<&ParsedController>) -> Vec<(String, SsrTier)> {
    let Some(c) = controller else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for m in &c.returned {
        let slice = safe_slice(&c.source, m.value_span.start, m.value_span.end);
        let mut tier = classify_ssr_tier(slice);
        // Upgrade only when *this* binding is written/updated inside a
        // statement that also uses a client marker (same balanced region).
        if tier != SsrTier::C {
            let name = m.name.as_str();
            // Also check the local `const name = …` initializer when shorthand.
            if let Some(init) = find_const_init(&c.source, name) {
                tier = classify_ssr_tier(init);
            }
            if tier != SsrTier::C && binding_updated_with_client_marker(&c.source, name) {
                tier = SsrTier::C;
            }
        }
        out.push((m.name.clone(), tier));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn safe_slice(src: &str, start: usize, end: usize) -> &str {
    let start = floor_char_boundary(src, start.min(src.len()));
    let end = floor_char_boundary(src, end.min(src.len())).max(start);
    &src[start..end]
}

fn floor_char_boundary(s: &str, i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    let mut j = i;
    while j > 0 && !s.is_char_boundary(j) {
        j -= 1;
    }
    j
}

/// `const name = <init>` / `let name = <init>` RHS when present.
fn find_const_init<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let bytes = src.as_bytes();
    let needle = name.as_bytes();
    let mut i = 0usize;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let prev_ok = i == 0 || !is_ident(bytes[i - 1]);
            let after = i + needle.len();
            let next_ok = after >= bytes.len() || !is_ident(bytes[after]);
            if prev_ok && next_ok {
                // Look back for const/let
                let before = &src[..i];
                let trimmed = before.trim_end();
                if trimmed.ends_with("const") || trimmed.ends_with("let") {
                    let mut p = after;
                    while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                        p += 1;
                    }
                    // optional type annotation
                    if p < bytes.len() && bytes[p] == b':' {
                        while p < bytes.len() && bytes[p] != b'=' {
                            p += 1;
                        }
                    }
                    while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                        p += 1;
                    }
                    if p < bytes.len() && bytes[p] == b'=' {
                        p += 1;
                        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                            p += 1;
                        }
                        let start = p;
                        // end at `;` or newline at depth 0
                        let mut depth = 0i32;
                        let mut in_str: Option<u8> = None;
                        while p < bytes.len() {
                            let c = bytes[p];
                            if let Some(q) = in_str {
                                if c == b'\\' {
                                    p += 2;
                                    continue;
                                }
                                if c == q {
                                    in_str = None;
                                }
                                p += 1;
                                continue;
                            }
                            match c {
                                b'\'' | b'"' | b'`' => in_str = Some(c),
                                b'(' | b'[' | b'{' => depth += 1,
                                b')' | b']' | b'}' => depth -= 1,
                                b';' if depth <= 0 => {
                                    return Some(src.get(start..p).unwrap_or(""));
                                }
                                b'\n' if depth <= 0 => {
                                    return Some(src.get(start..p).unwrap_or(""));
                                }
                                _ => {}
                            }
                            p += 1;
                        }
                        return Some(src.get(start..p).unwrap_or(""));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// True when `name.set(` / `name(...)` write appears in the same balanced
/// statement (`;`-terminated region) as a client marker.
fn binding_updated_with_client_marker(src: &str, name: &str) -> bool {
    let bytes = src.as_bytes();
    let n = name.as_bytes();
    let mut i = 0usize;
    while i + n.len() < bytes.len() {
        if &bytes[i..i + n.len()] != n {
            i += 1;
            continue;
        }
        let prev_ok = i == 0 || !is_ident(bytes[i - 1]);
        let after = i + n.len();
        let next_ok = after >= bytes.len() || !is_ident(bytes[after]);
        if !prev_ok || !next_ok {
            i += 1;
            continue;
        }
        // name.set( or name( — treat as write/update site
        let mut p = after;
        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        let is_write = if p + 3 < bytes.len() && &bytes[p..p + 4] == b".set" {
            true
        } else if p < bytes.len() && bytes[p] == b'(' {
            // call form: ticks() is read; only treat name.set as write for signals
            false
        } else {
            false
        };
        // Also: name is closed over inside setInterval/setTimeout callback that
        // contains name.set — scan the enclosing statement for both.
        let stmt = enclosing_statement(src, i);
        let has_client = CLIENT_MARKERS.iter().any(|m| contains_token(stmt, m));
        let has_set =
            stmt.contains(&format!("{name}.set")) || stmt.contains(&format!("{name} .set"));
        if has_client && (is_write || has_set) {
            return true;
        }
        // effect(() => { setInterval(() => name.set(...)) }) — marker + set
        // in a window around the name (same function body depth).
        if has_client && contains_token(stmt, name) && has_set {
            return true;
        }
        i += 1;
    }
    // Fallback: effect/mount body that both mentions client marker and name.set
    for marker in [
        "setInterval",
        "setTimeout",
        "requestAnimationFrame",
        "matchMedia",
    ] {
        if let Some(pos) = src.find(marker) {
            let region = enclosing_balanced_call(src, pos).unwrap_or("");
            if region.contains(&format!("{name}.set")) {
                return true;
            }
        }
    }
    false
}

fn enclosing_statement(src: &str, at: usize) -> &str {
    let bytes = src.as_bytes();
    let mut start = at;
    while start > 0
        && bytes[start - 1] != b';'
        && bytes[start - 1] != b'{'
        && bytes[start - 1] != b'\n'
    {
        // don't cross too far
        if at - start > 800 {
            break;
        }
        start -= 1;
    }
    let mut end = at;
    let mut depth = 0i32;
    while end < bytes.len() {
        match bytes[end] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            b';' if depth <= 0 => {
                end += 1;
                break;
            }
            _ => {}
        }
        end += 1;
        if end - at > 1200 {
            break;
        }
    }
    src.get(start..end).unwrap_or("")
}

fn enclosing_balanced_call(src: &str, at: usize) -> Option<&str> {
    let bytes = src.as_bytes();
    let mut p = at;
    while p < bytes.len() && bytes[p] != b'(' {
        p += 1;
        if p - at > 40 {
            return None;
        }
    }
    if p >= bytes.len() {
        return None;
    }
    let open = p;
    let close = skip_balanced_paren(bytes, open)?;
    src.get(open..=close)
}

fn looks_like_literal_init(inner: &str) -> bool {
    let t = inner.trim();
    if t.is_empty() {
        return true;
    }
    if t.parse::<f64>().is_ok() {
        return true;
    }
    if (t.starts_with('"') && t.ends_with('"'))
        || (t.starts_with('\'') && t.ends_with('\''))
        || (t.starts_with('`') && t.ends_with('`') && !t.contains("${"))
    {
        return true;
    }
    matches!(t, "true" | "false" | "null" | "undefined" | "[]" | "{}")
}

fn contains_token(hay: &str, needle: &str) -> bool {
    if needle.ends_with('(') || needle.ends_with('.') {
        return hay.contains(needle);
    }
    let bytes = hay.as_bytes();
    let n = needle.as_bytes();
    let mut i = 0usize;
    while i + n.len() <= bytes.len() {
        if &bytes[i..i + n.len()] == n {
            let before = i == 0 || !is_ident(bytes[i - 1]);
            let after = i + n.len() >= bytes.len() || !is_ident(bytes[i + n.len()]);
            if before && after {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn skip_balanced_paren(bytes: &[u8], open: usize) -> Option<usize> {
    if bytes.get(open) != Some(&b'(') {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open;
    let mut in_str: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'\'' | b'"' | b'`' => in_str = Some(c),
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Best-effort static key path from a query/mutation thunk: `crm.deals.list(...)`.
fn extract_dot_chain_key(call_body: &str) -> Option<String> {
    // Look for `=> root.seg.seg(` or `root.seg.seg(` patterns.
    let bytes = call_body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if !is_ident_start(bytes[i]) {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() && is_ident(bytes[i]) {
            i += 1;
        }
        let mut end = i;
        // Consume `.ident` chain
        loop {
            let mut p = end;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                p += 1;
            }
            if p >= bytes.len() || bytes[p] != b'.' {
                break;
            }
            p += 1;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                p += 1;
            }
            if p >= bytes.len() || !is_ident_start(bytes[p]) {
                break;
            }
            p += 1;
            while p < bytes.len() && is_ident(bytes[p]) {
                p += 1;
            }
            end = p;
        }
        let mut p = end;
        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        if p < bytes.len() && bytes[p] == b'(' {
            let chain = call_body[start..end].trim();
            // Skip arrow params and keywords
            if !matches!(
                chain,
                "query" | "mutation" | "invalidate" | "function" | "async" | "return"
            ) && chain.contains('.')
            {
                return Some(chain.to_string());
            }
        }
    }
    None
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b'$'
}

/// Resolve the symbol under a 1-based `(line, col)` cursor. Finds the root of
/// the member chain at the cursor (`items` for a cursor on `items().length`)
/// and looks it up in the scope table.
pub fn hover_at(
    source: &str,
    source_path: Option<&Path>,
    line: usize,
    col: usize,
) -> Option<Hover> {
    let offset = offset_of(source, line, col);
    // SPEC-PARTIAL-1: file-local partial name / invocation tag hover (author
    // source — partials expand away in `parse`, so this path is pre-parse).
    if let Some(tag) = tag_name_at(source, offset)
        && let Some(md) = crate::parse::partials::partial_hover(source, &tag)
    {
        return Some(Hover {
            name: tag,
            kind: "partial".into(),
            source: md,
            line,
            col,
        });
    }
    let program = crate::parse::parse(source).ok()?;
    let controller = inline_controller(source, source_path);
    let table = SymbolTable::build(&program, controller.as_ref());
    let root = root_ident_at(source.as_bytes(), offset)?;
    symbols_from_table(&table, controller.as_ref())
        .into_iter()
        .find(|s| s.name == root)
        .map(|s| Hover {
            name: s.name,
            kind: s.kind,
            source: s.source,
            line: s.line,
            col: s.col,
        })
}

/// Tag name under `offset` when the cursor sits on an element/partial tag.
fn tag_name_at(source: &str, offset: usize) -> Option<String> {
    let bytes = source.as_bytes();
    if offset >= bytes.len() {
        return None;
    }
    // Walk left to `<` or non-name.
    let mut i = offset.min(bytes.len().saturating_sub(1));
    while i > 0 && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
        i -= 1;
    }
    if bytes[i] == b'/' {
        if i == 0 || bytes[i - 1] != b'<' {
            return None;
        }
        i += 1;
    } else if bytes[i] == b'<' {
        i += 1;
    } else if i + 1 <= offset && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
        // land mid-name without finding `<` in this walk — scan further left
        while i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-') {
            i -= 1;
        }
        if i == 0 || bytes[i - 1] != b'<' {
            // maybe `</`
            if i >= 2 && bytes[i - 1] == b'/' && bytes[i - 2] == b'<' {
                // ok
            } else {
                return None;
            }
        }
    } else {
        return None;
    }
    let start = i;
    let mut end = start;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-') {
        end += 1;
    }
    if start >= end || offset < start || offset > end {
        return None;
    }
    Some(source[start..end].to_string())
}

/// Every scope name available for completion, with its binding kind.
pub fn completions_at(source: &str, source_path: Option<&Path>) -> Vec<Symbol> {
    let mut out = Vec::new();
    // SPEC-PARTIAL-1: file-local partial names (this file only).
    for name in crate::parse::partials::partial_names(source) {
        out.push(Symbol {
            name,
            kind: "partial".into(),
            source: "partial".into(),
            line: 0,
            col: 0,
        });
    }
    let Ok(program) = crate::parse::parse(source) else {
        return out;
    };
    let controller = inline_controller(source, source_path);
    let table = SymbolTable::build(&program, controller.as_ref());
    out.extend(symbols_from_table(&table, controller.as_ref()));
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Position-aware completions (SPEC-STYLE-COMPLETE-1).
///
/// When `byte_offset` sits inside a Style region, returns Style items mapped to
/// [`Symbol`] (prefer Style over bare scope). Otherwise returns scope symbols
/// from [`completions_at`].
///
/// `host` is the L1 host tag for component units (`Some("chip")`); `None` for
/// app Style (D1 surface law).
pub fn completions_at_offset(
    source: &str,
    source_path: Option<&Path>,
    byte_offset: usize,
    host: Option<&str>,
) -> Vec<Symbol> {
    let style_items =
        sail_lint::style::style_complete_at(source, source_path, byte_offset, host);
    if !style_items.is_empty() {
        return style_items
            .into_iter()
            .map(|it| Symbol {
                name: it.label,
                kind: it.kind.as_str().to_string(),
                source: "style".into(),
                line: 0,
                col: 0,
            })
            .collect();
    }
    // Style context with empty set (e.g. StyleRef with no consts) still wins
    // over scope when the caret is classified as Style.
    let ctx = sail_lint::style::style_complete_context_at(source, byte_offset, host);
    if !matches!(ctx, sail_lint::style::StyleCompleteContext::Unknown) {
        return Vec::new();
    }
    completions_at(source, source_path)
}

// ── internals ─────────────────────────────────────────────────────────────

fn symbols_from_table(table: &SymbolTable, controller: Option<&ParsedController>) -> Vec<Symbol> {
    let ctrl_src = controller.map(|c| c.source.as_str()).unwrap_or("");
    let mut out: Vec<Symbol> = Vec::new();

    // `<prop>` lives in the .sail — no controller offset (0).
    for (name, e) in &table.props {
        out.push(Symbol {
            name: name.clone(),
            kind: e.kind_label.to_string(),
            source: "prop".into(),
            line: 0,
            col: 0,
        });
    }
    // Controller-declared keys carry a byte offset into the controller
    // source. There is no more `derived:`/`on:`/`use:` section to
    // separate — the setup function returns ONE flat scope object,
    // classified by its value's shape (kind).
    for (name, e) in &table.scope {
        let (l, c) = pos(ctrl_src, e.offset);
        out.push(Symbol {
            name: name.clone(),
            kind: e.kind.label().to_string(),
            source: "setup".into(),
            line: l,
            col: c,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Controller byte offset → 1-based (line, col). Offset 0 means "declared in
/// the .sail", reported as `(0, 0)`.
fn pos(ctrl_src: &str, offset: usize) -> (usize, usize) {
    if offset == 0 || ctrl_src.is_empty() {
        return (0, 0);
    }
    line_col(ctrl_src, offset)
}

/// Recursively collect custom-element tags + `on:` event names.
fn collect_tags_events(program: &Program) -> (Vec<String>, Vec<String>) {
    let mut tags: BTreeSet<String> = BTreeSet::new();
    let mut events: BTreeSet<String> = BTreeSet::new();
    walk(&program.template, &mut tags, &mut events);
    (tags.into_iter().collect(), events.into_iter().collect())
}

fn walk(instrs: &[Instr], tags: &mut BTreeSet<String>, events: &mut BTreeSet<String>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                // Custom elements contain a hyphen (HTML rule). Native tags
                // (div, span) aren't dependencies worth reporting.
                if el.tag.contains('-') {
                    tags.insert(el.tag.clone());
                }
                for a in &el.attrs {
                    if let AttrInstr::On { event, .. } = a {
                        events.insert(event.clone());
                    }
                }
                walk(&el.children, tags, events);
            }
            Instr::If(i) => {
                for b in &i.branches {
                    walk(&b.body, tags, events);
                }
                if let Some(e) = &i.else_body {
                    walk(e, tags, events);
                }
            }
            Instr::For(f) => {
                walk(&f.body, tags, events);
                if let Some(e) = &f.else_body {
                    walk(e, tags, events);
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    walk(&c.body, tags, events);
                }
                if let Some(d) = &m.default {
                    walk(d, tags, events);
                }
            }
            Instr::When(w) => {
                walk(&w.body, tags, events);
                if let Some(e) = &w.else_body {
                    walk(e, tags, events);
                }
            }
            _ => {}
        }
    }
}

/// 1-based `(line, col)` → byte offset into `source`.
fn offset_of(source: &str, target_line: usize, target_col: usize) -> usize {
    let bytes = source.as_bytes();
    let (mut line, mut col, mut i) = (1usize, 1usize, 0usize);
    while i < bytes.len() {
        if line == target_line && col == target_col {
            return i;
        }
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    bytes.len()
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Given `pos` just after a closing `)` or `]`, return the index of its
/// matching opener (so the preceding identifier sits at `..opener`).
fn skip_group_left(bytes: &[u8], pos: usize) -> usize {
    let close = bytes[pos - 1];
    let open = if close == b')' { b'(' } else { b'[' };
    let mut depth = 0i32;
    let mut i = pos;
    while i > 0 {
        let c = bytes[i - 1];
        if c == close {
            depth += 1;
        } else if c == open {
            depth -= 1;
            if depth == 0 {
                return i - 1;
            }
        }
        i -= 1;
    }
    0
}

/// Root identifier of the member chain at `offset`. For a cursor anywhere in
/// `items().length` returns `items`; walks left across `.ident` segments to
/// the chain root. Returns `None` when the cursor isn't on an identifier.
fn root_ident_at(bytes: &[u8], offset: usize) -> Option<String> {
    let len = bytes.len();
    if len == 0 {
        return None;
    }
    // Find the ident token covering (or just left of) the offset.
    let mut end = offset.min(len);
    while end < len && is_ident(bytes[end]) {
        end += 1;
    }
    let mut start = end;
    while start > 0 && is_ident(bytes[start - 1]) {
        start -= 1;
    }
    if start == end {
        return None;
    }
    // Walk left across `.ident` chains to the root, stepping over trailing
    // call/index groups so `items().length` and `rows[0].id` resolve to the
    // chain root (`items`, `rows`).
    loop {
        let mut p = start;
        while p > 0 && bytes[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        if p == 0 || bytes[p - 1] != b'.' {
            break;
        }
        p -= 1; // skip the dot
        while p > 0 && bytes[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        // Step over any `(...)` / `[...]` groups attached to the prior segment.
        while p > 0 && (bytes[p - 1] == b')' || bytes[p - 1] == b']') {
            p = skip_group_left(bytes, p);
            while p > 0 && bytes[p - 1].is_ascii_whitespace() {
                p -= 1;
            }
        }
        let seg_end = p;
        while p > 0 && is_ident(bytes[p - 1]) {
            p -= 1;
        }
        if p == seg_end {
            break; // not an ident before the dot (e.g. a literal)
        }
        start = p; // new root segment; loop checks for another `.ident` left of it
    }
    let mut e = start;
    while e < len && is_ident(bytes[e]) {
        e += 1;
    }
    std::str::from_utf8(&bytes[start..e])
        .ok()
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_reports_tags_and_events() {
        let src = r#"<div><sail-button on:click={go}>Go</sail-button><span>{x}</span></div>"#;
        let a = analyze(src, None, &[]);
        assert!(
            a.tags.contains(&"sail-button".to_string()),
            "tags: {:?}",
            a.tags
        );
        assert!(
            !a.tags.iter().any(|t| t == "div" || t == "span"),
            "native tags leaked: {:?}",
            a.tags
        );
        assert!(
            a.events.contains(&"click".to_string()),
            "events: {:?}",
            a.events
        );
    }

    #[test]
    fn analyze_surfaces_diagnostics() {
        let a = analyze("<p>{new Date()}</p>", None, &[]);
        assert!(!a.diagnostics.is_empty(), "expected a diagnostic for `new`");
    }

    #[test]
    fn root_ident_walks_member_chain() {
        // cursor on `length` in `items().length` → root `items`.
        let src = "items().length";
        let on_length = src.find("length").unwrap();
        assert_eq!(
            root_ident_at(src.as_bytes(), on_length).as_deref(),
            Some("items")
        );
        // cursor on `items` itself → `items`.
        assert_eq!(root_ident_at(src.as_bytes(), 0).as_deref(), Some("items"));
    }

    #[test]
    fn root_ident_none_off_identifier() {
        assert_eq!(root_ident_at(b"  + ", 1), None);
    }

    #[test]
    fn completions_at_offset_style_bg_tokens() {
        // SPEC-STYLE-COMPLETE-1: caret inside `bg(¦)` returns Style tokens.
        let src = r#"<div style({ paint: bg() })>x</div>"#;
        let off = src.find("bg(").expect("bg(") + 3; // inside parens
        let items = completions_at_offset(src, None, off, None);
        assert!(
            items.iter().any(|s| s.name == "primary"),
            "expected primary token: {:?}",
            items.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
        assert!(
            items.iter().any(|s| s.name == "surface-1"),
            "expected surface-1: {:?}",
            items.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
        assert!(
            items.iter().all(|s| s.source == "style"),
            "Style complete must prefer style source: {:?}",
            items
        );
        assert!(
            !items.iter().any(|s| s.name == "chip-foreground"),
            "app host must exclude chip-foreground"
        );
    }

    #[test]
    fn completions_at_offset_outside_style_is_scope() {
        let src = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => ({ count: signal(0) }))
</controller>
<div>{count()}</div>"#;
        let off = src.find("count()").expect("count");
        let items = completions_at_offset(src, None, off, None);
        // Not Style context — may return scope symbols (count) when parse ok.
        assert!(
            !items.iter().any(|s| s.source == "style" && s.kind == "token"),
            "non-style caret must not invent style tokens: {:?}",
            items
        );
    }

    #[test]
    fn ssr_tier_literal_signal_is_s() {
        assert_eq!(classify_ssr_tier("signal(0)"), SsrTier::S);
        assert_eq!(classify_ssr_tier("signal(\"hello\")"), SsrTier::S);
    }

    #[test]
    fn ssr_tier_derived_pure_is_s() {
        assert_eq!(classify_ssr_tier("derived(() => pure() + 1)"), SsrTier::S);
    }

    #[test]
    fn ssr_tier_client_markers_are_c() {
        assert_eq!(classify_ssr_tier("signal(window.innerWidth)"), SsrTier::C);
        assert_eq!(
            classify_ssr_tier("signal(localStorage.getItem('x'))"),
            SsrTier::C
        );
        assert_eq!(
            classify_ssr_tier("matchMedia('(min-width: 1px)').matches"),
            SsrTier::C
        );
    }

    #[test]
    fn ssr_tier_per_binding_not_whole_file() {
        let ctrl = r#"
export default controller((_ctx) => {
  const pure = signal(0)
  const ticks = signal(0)
  effect(() => {
    const t = setInterval(() => ticks.set((n) => n + 1), 1000)
    return () => clearInterval(t)
  })
  return { pure, ticks }
})
"#;
        // Fake a ParsedController-like analysis via extract on returned names
        // by classifying init + binding_updated_with_client_marker.
        assert_eq!(classify_ssr_tier("signal(0)"), SsrTier::S);
        assert!(binding_updated_with_client_marker(ctrl, "ticks"));
        assert!(!binding_updated_with_client_marker(ctrl, "pure"));
    }
}
