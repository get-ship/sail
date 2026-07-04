//! `sui-lsp` — Language Server for `.sui` templates.
//!
//! A thin LSP/stdio shell over the `sui-compiler` analysis facade. All
//! intelligence (diagnostics, hover, completion, definition) comes from the
//! one Rust grammar — no TypeScript, no WASM. This single binary serves every
//! LSP client: Claude Code (`.lsp.json`), VS Code (`sui.serverPath`), and any
//! editor.
//!
//! Transport is the synchronous `lsp-server` loop (rust-analyzer lineage);
//! `lsp-types` is used only to (de)serialize protocol payloads. Capabilities
//! are pushed diagnostics on open/change/save, plus hover / completion /
//! go-to-definition.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, Diagnostic,
    DiagnosticSeverity, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, Location, MarkupContent, MarkupKind, NumberOrString,
    OneOf, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};

type Docs = HashMap<Url, String>;

fn main() -> anyhow::Result<()> {
    let (connection, io_threads) = Connection::stdio();
    let capabilities = serde_json::to_value(server_capabilities())?;
    let _init = connection.initialize(capabilities)?;
    main_loop(&connection)?;
    io_threads.join()?;
    Ok(())
}

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // FULL sync: each change resends the whole document. `.sui` files are
        // small; incremental sync isn't worth the complexity here.
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec!["-".into(), "(".into()]),
            ..Default::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    }
}

fn main_loop(conn: &Connection) -> anyhow::Result<()> {
    let mut docs: Docs = HashMap::new();
    for msg in &conn.receiver {
        match msg {
            Message::Request(req) => {
                if conn.handle_shutdown(&req)? {
                    return Ok(());
                }
                handle_request(conn, &docs, req)?;
            }
            Message::Notification(note) => handle_notification(conn, &mut docs, note)?,
            Message::Response(_) => {}
        }
    }
    Ok(())
}

// ── notifications: document sync → push diagnostics ────────────────────────

fn handle_notification(
    conn: &Connection,
    docs: &mut Docs,
    note: lsp_server::Notification,
) -> anyhow::Result<()> {
    match note.method.as_str() {
        "textDocument/didOpen" => {
            let p: lsp_types::DidOpenTextDocumentParams = serde_json::from_value(note.params)?;
            let uri = p.text_document.uri.clone();
            docs.insert(uri.clone(), p.text_document.text.clone());
            publish_diagnostics(conn, &uri, &p.text_document.text)?;
        }
        "textDocument/didChange" => {
            let p: lsp_types::DidChangeTextDocumentParams = serde_json::from_value(note.params)?;
            // FULL sync: the last change carries the entire new text.
            if let Some(change) = p.content_changes.into_iter().last() {
                let uri = p.text_document.uri.clone();
                docs.insert(uri.clone(), change.text.clone());
                publish_diagnostics(conn, &uri, &change.text)?;
            }
        }
        "textDocument/didSave" => {
            let p: lsp_types::DidSaveTextDocumentParams = serde_json::from_value(note.params)?;
            let uri = p.text_document.uri;
            if let Some(text) = docs.get(&uri).cloned() {
                publish_diagnostics(conn, &uri, &text)?;
            }
        }
        "textDocument/didClose" => {
            let p: lsp_types::DidCloseTextDocumentParams = serde_json::from_value(note.params)?;
            docs.remove(&p.text_document.uri);
        }
        _ => {}
    }
    Ok(())
}

fn publish_diagnostics(conn: &Connection, uri: &Url, text: &str) -> anyhow::Result<()> {
    let path = uri.to_file_path().ok();
    let analysis = sui_compiler::analyze(text, path.as_deref(), &[]);
    let diagnostics: Vec<Diagnostic> = analysis.diagnostics.iter().map(diag_to_lsp).collect();
    let params = PublishDiagnosticsParams { uri: uri.clone(), diagnostics, version: None };
    conn.sender.send(Message::Notification(lsp_server::Notification {
        method: "textDocument/publishDiagnostics".to_string(),
        params: serde_json::to_value(params)?,
    }))?;
    Ok(())
}

// ── requests: hover / completion / definition ──────────────────────────────

fn handle_request(conn: &Connection, docs: &Docs, req: Request) -> anyhow::Result<()> {
    let resp = match req.method.as_str() {
        "textDocument/hover" => respond(req, |id, req| on_hover(docs, id, req)),
        "textDocument/completion" => respond(req, |id, req| on_completion(docs, id, req)),
        "textDocument/definition" => respond(req, |id, req| on_definition(docs, id, req)),
        _ => None,
    };
    if let Some(resp) = resp {
        conn.sender.send(Message::Response(resp))?;
    }
    Ok(())
}

/// Extract typed params and build a `Response`, replying with `null` on a
/// deserialize failure rather than dropping the request.
fn respond<P, R>(req: Request, f: impl FnOnce(RequestId, P) -> Option<R>) -> Option<Response>
where
    P: serde::de::DeserializeOwned,
    R: serde::Serialize,
{
    let id = req.id.clone();
    match cast::<P>(req) {
        Ok((id2, params)) => match f(id2.clone(), params) {
            Some(result) => Some(Response {
                id: id2,
                result: Some(serde_json::to_value(result).unwrap_or(serde_json::Value::Null)),
                error: None,
            }),
            None => Some(Response { id: id2, result: Some(serde_json::Value::Null), error: None }),
        },
        Err(_) => Some(Response { id, result: Some(serde_json::Value::Null), error: None }),
    }
}

fn cast<P: serde::de::DeserializeOwned>(req: Request) -> Result<(RequestId, P), ExtractError<Request>> {
    let id = req.id.clone();
    let params = serde_json::from_value(req.params).map_err(|_| ExtractError::JsonError {
        method: req.method.clone(),
        error: serde_json::Error::io(std::io::Error::other("bad params")),
    })?;
    Ok((id, params))
}

fn on_hover(docs: &Docs, _id: RequestId, params: HoverParams) -> Option<Hover> {
    let tdp = params.text_document_position_params;
    let (text, path) = doc(docs, &tdp.text_document.uri)?;
    let (line, col) = from_lsp_pos(tdp.position);
    let h = sui_compiler::hover_at(&text, path.as_deref(), line, col)?;
    let loc = if h.line > 0 {
        format!("\n\nDeclared at controller {}:{}", h.line, h.col)
    } else {
        String::new()
    };
    let value = format!("**{}** · `{}` (from `{}`){loc}", h.name, h.kind, h.source);
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value }),
        range: None,
    })
}

fn on_completion(docs: &Docs, _id: RequestId, params: CompletionParams) -> Option<Vec<CompletionItem>> {
    let tdp = params.text_document_position;
    let (text, path) = doc(docs, &tdp.text_document.uri)?;
    let pos = tdp.position;
    let col = pos.character as usize;
    if is_css_token_context(&text, pos.line as usize, col) {
        return Some(token_completion_items(&text, pos.line as usize, col));
    }
    let items = sui_compiler::completions_at(&text, path.as_deref())
        .into_iter()
        .map(|s| CompletionItem {
            label: s.name,
            kind: Some(completion_kind(&s.kind)),
            detail: Some(format!("{} ({})", s.kind, s.source)),
            ..Default::default()
        })
        .collect();
    Some(items)
}

fn on_definition(
    docs: &Docs,
    _id: RequestId,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let tdp = params.text_document_position_params;
    let (text, path) = doc(docs, &tdp.text_document.uri)?;
    let (line, col) = from_lsp_pos(tdp.position);
    let h = sui_compiler::hover_at(&text, path.as_deref(), line, col)?;
    // Only controller-declared symbols carry a jump target (line > 0).
    if h.line == 0 {
        return None;
    }
    let analysis = sui_compiler::analyze(&text, path.as_deref(), &[]);
    let ctrl = analysis.controller_path?;
    let uri = Url::from_file_path(&ctrl).ok()?;
    let pos = to_lsp_pos(h.line, h.col);
    Some(GotoDefinitionResponse::Scalar(Location { uri, range: Range::new(pos, pos) }))
}

// ── pure mappers (unit-tested) ─────────────────────────────────────────────

fn doc(docs: &Docs, uri: &Url) -> Option<(String, Option<std::path::PathBuf>)> {
    let text = docs.get(uri)?.clone();
    Some((text, uri.to_file_path().ok()))
}

/// Compiler 1-based `(line, col)` → LSP 0-based `Position`.
fn to_lsp_pos(line: usize, col: usize) -> Position {
    Position::new(line.saturating_sub(1) as u32, col.saturating_sub(1) as u32)
}

/// LSP 0-based `Position` → compiler 1-based `(line, col)`.
fn from_lsp_pos(p: Position) -> (usize, usize) {
    (p.line as usize + 1, p.character as usize + 1)
}

fn diag_to_lsp(d: &sui_compiler::Diagnostic) -> Diagnostic {
    Diagnostic {
        range: Range::new(to_lsp_pos(d.line, d.col), to_lsp_pos(d.end_line, d.end_col)),
        severity: Some(match d.severity {
            sui_compiler::DiagSeverity::Error => DiagnosticSeverity::ERROR,
            sui_compiler::DiagSeverity::Warning => DiagnosticSeverity::WARNING,
        }),
        code: d.rule.clone().map(NumberOrString::String),
        source: Some("sui".to_string()),
        message: d.message.clone(),
        ..Default::default()
    }
}

fn completion_kind(kind: &str) -> CompletionItemKind {
    match kind {
        "signal" | "ref" => CompletionItemKind::VARIABLE,
        "derived" => CompletionItemKind::PROPERTY,
        "action" => CompletionItemKind::FUNCTION,
        "data" | "prop" => CompletionItemKind::FIELD,
        _ => CompletionItemKind::VALUE,
    }
}

// ── design-token completion ────────────────────────────────────────────────

/// Canonical token source — `--name: value;` declarations under `:root` in
/// `packages/sui/tokens/src/tokens.css`. Embedded at build time so the LSP
/// has zero runtime IO and a missing token file is a compile error, not a
/// silent empty completion. This is the single source; the css-token lint
/// parses the same file (see `sui_lint::tokens`).
const TOKENS_CSS: &str = include_str!("../../../../packages/sui/tokens/src/tokens.css");

/// `(name, value)` pairs with leading `--` stripped. Sorted by name so
/// completion order is stable across builds.
type TokenTable = Vec<(String, String)>;

static TOKENS: LazyLock<TokenTable> = LazyLock::new(|| {
    let mut by_name = parse_tokens(TOKENS_CSS);
    // Contextual tokens with no `:root` value (tone channel, floating-primitive
    // anchors, app-shell) — completable, but valued at runtime. Shared verbatim
    // with the css-token lint so the two registries can never diverge.
    for &name in sui_lint::tokens::CONVENTION_TOKENS {
        if !by_name.iter().any(|(n, _)| n == name) {
            by_name.push((name.to_string(), String::new()));
        }
    }
    by_name.sort_by(|a, b| a.0.cmp(&b.0));
    by_name
});

/// Hand-rolled `--name: value;` extractor. We don't need a CSS parser —
/// tokens.css is a known, well-formed file (canonical token source), and
/// the spec forbids new heavy deps. A regex-free byte walk keeps the
/// binary small and the logic auditable.
fn parse_tokens(src: &str) -> TokenTable {
    let mut out: TokenTable = Vec::with_capacity(256);
    let mut seen: HashSet<String> = HashSet::new();
    // Two state machines interleaved on the char stream:
    //   - `block_depth` — current `{ … }` nesting (0 outside any block).
    //   - `in_root` — how many `:root` blocks we're nested inside.
    // `:root` (with or without a `[data-…]` attribute selector) collects
    // tokens; `.dark`, media queries, utility classes don't. Skipping
    // `.dark` overrides is what gets us the canonical 247 leaves instead
    // of the 318 you'd get reading every `--name: value;` in the file.
    let mut block_depth: i32 = 0;
    let mut in_root: i32 = 0;
    // Selector being built for a not-yet-opened block. When we see `{`,
    // the selector is committed and we set `in_root` if the selector
    // starts with `:root`.
    let mut selector = String::new();
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                let s = selector.trim();
                if s.starts_with(":root") {
                    in_root += 1;
                }
                selector.clear();
                block_depth += 1;
            }
            '}' => {
                if block_depth > 0 { block_depth -= 1; }
                if in_root > 0 { in_root -= 1; }
            }
            _ if in_root > 0 && c == '-' && chars.peek() == Some(&'-') => {
                chars.next(); // consume second '-'
                // Name
                let mut name = String::new();
                for c in chars.by_ref() {
                    match c {
                        ':' => break,
                        _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' => name.push(c),
                        _ => { name.clear(); break; }
                    }
                }
                if name.is_empty() || !seen.insert(name.clone()) {
                    continue;
                }
                // Value up to ';'. Multi-line values join with spaces.
                let mut value = String::new();
                let mut buf = String::new();
                let mut started = false;
                for c in chars.by_ref() {
                    if c == ';' {
                        if started {
                            if !value.is_empty() { value.push(' '); }
                            value.push_str(buf.trim());
                        }
                        break;
                    }
                    if c == '\n' || c == '\r' {
                        if started && !buf.trim().is_empty() {
                            if !value.is_empty() { value.push(' '); }
                            value.push_str(buf.trim());
                            buf.clear();
                        } else if started {
                            buf.clear();
                        }
                        continue;
                    }
                    started = true;
                    buf.push(c);
                }
                if value.is_empty() {
                    value = buf.trim().to_string();
                }
                if !value.is_empty() {
                    out.push((name, value));
                }
            }
            ';' | '\n' | '\r' if block_depth == 0 => {
                // End of an at-rule / top-level statement; reset selector.
                selector.clear();
            }
            _ if block_depth == 0 => selector.push(c),
            _ => {}
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// True when the value looks like a CSS color — `oklch(`, `oklab(`,
/// `rgb(`, `rgba(`, `hsl(`, `hsla(`, `#…`, or a named color. Editors use
/// `CompletionItemKind::COLOR` to render a swatch next to the item.
fn looks_like_color(value: &str) -> bool {
    let v = value.trim();
    let starts_color = |prefix: &str| v.starts_with(prefix);
    starts_color("oklch(")
        || starts_color("oklab(")
        || starts_color("rgb(")
        || starts_color("rgba(")
        || starts_color("hsl(")
        || starts_color("hsla(")
        || starts_color("#")
}

/// Build CompletionItems for every token. `cursor_ends_with_dd` controls
/// whether the inserted text includes the leading `--` — when the user
/// has just typed `--`, we must not double it.
fn token_completion_items(text: &str, line_idx: usize, col: usize) -> Vec<CompletionItem> {
    let prefix_dd = line_prefix_ends_with_dd(text, line_idx, col);
    TOKENS
        .iter()
        .map(|(name, value)| CompletionItem {
            label: format!("--{name}"),
            kind: Some(if looks_like_color(value) {
                CompletionItemKind::COLOR
            } else {
                CompletionItemKind::CONSTANT
            }),
            detail: Some(value.clone()),
            insert_text: if prefix_dd {
                Some(name.clone())
            } else {
                Some(format!("--{name}"))
            },
            ..Default::default()
        })
        .collect()
}

/// The line text up to `col` ends with `--` (cursor sits just after a
/// double-hyphen). Used to avoid inserting a duplicate `--` when the
/// user has already typed the prefix.
fn line_prefix_ends_with_dd(text: &str, line_idx: usize, col: usize) -> bool {
    let Some(line) = text.lines().nth(line_idx) else { return false };
    let prefix = &line[..col.min(line.len())];
    prefix.ends_with("--")
}

/// Detect whether `pos` is inside a CSS context where a design token
/// would be valid: either inside a `<style>…</style>` block or inside a
/// `style="…"` / `style='…'` / `style: …` attribute value, AND on the
/// current line, somewhere before the cursor, there is an unclosed
/// `var(` or the line text up to the cursor ends with `--`.
fn is_css_token_context(text: &str, line_idx: usize, col: usize) -> bool {
    if !(in_style_block(text, line_idx) || in_style_attr_value(text, line_idx, col)) {
        return false;
    }
    let Some(line) = text.lines().nth(line_idx) else { return false };
    let prefix = &line[..col.min(line.len())];
    if prefix.ends_with("--") {
        return true;
    }
    // Unclosed `var(` between the last `var(` and the cursor.
    if let Some(open) = prefix.rfind("var(") {
        return prefix[open + 4..].find(')').is_none();
    }
    false
}

/// True if line `line_idx` falls inside an open `<style>` block. A
/// `<style>` element is recognised by the opening tag with an optional
/// leading `<` boundary and the matching `</style>` close. Comments and
/// quoted attributes are not specially handled — token.css authoring
/// doesn't use them, and a miscount only suppresses token completion in
/// malformed files, which is acceptable.
fn in_style_block(text: &str, line_idx: usize) -> bool {
    let mut depth: i32 = 0;
    for (i, line) in text.lines().enumerate() {
        if i > line_idx {
            break;
        }
        scan_tag_openings(line, "style", &mut depth);
    }
    depth > 0
}

/// True if the cursor at `(line_idx, col)` sits inside a `style=`
/// attribute value (double- or single-quoted, or a `style:` flow
/// attribute without quotes — uncommon but legal in `.sui`).
fn in_style_attr_value(text: &str, line_idx: usize, col: usize) -> bool {
    let mut in_style = false;
    let mut quote: Option<char> = None;
    for (i, line) in text.lines().enumerate() {
        let bytes = line.as_bytes();
        let mut j = 0;
        while j <= bytes.len() {
            // As we pass the cursor column on the cursor's line, capture
            // whether the attribute is still open. Doing this inside the
            // loop (rather than after) lets us stop before the closing
            // quote is consumed.
            if i == line_idx && j == col {
                return in_style;
            }
            if j == bytes.len() {
                break;
            }
            if !in_style {
                if j + 6 <= bytes.len() && &bytes[j..j + 6] == b"style=" {
                    in_style = true;
                    j += 6;
                    if j < bytes.len() {
                        let q = bytes[j] as char;
                        if q == '"' || q == '\'' {
                            quote = Some(q);
                            j += 1;
                        } else {
                            quote = None; // unquoted
                        }
                    }
                    continue;
                }
            } else if let Some(q) = quote {
                if bytes[j] as char == q {
                    in_style = false;
                    quote = None;
                    j += 1;
                    continue;
                }
            } else {
                // Unquoted: ends at whitespace, `>`, or end of line.
                let c = bytes[j] as char;
                if c.is_whitespace() || c == '>' {
                    in_style = false;
                    quote = None;
                } else {
                    j += 1;
                    continue;
                }
            }
            j += 1;
        }
    }
    // Cursor past the last line — no open attribute there.
    false
}

/// For each `<style …>` opening or `</style>` closing tag on `line`,
/// bump `depth` accordingly. `<style …>` opens (+1), `</style>` closes
/// (-1). The tag match is naive (substring) — `.sui` files don't embed
/// `<style>` as text content, so a false positive is extremely unlikely.
fn scan_tag_openings(line: &str, tag: &str, depth: &mut i32) {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut i = 0;
    while let Some(p) = line[i..].find(&open) {
        let abs = i + p;
        // Skip if this is actually the closing tag (e.g. `</style>` has
        // `<style` as a substring after the `/`).
        if abs > 0 && line.as_bytes()[abs - 1] == b'/' {
            i = abs + open.len();
            continue;
        }
        *depth += 1;
        i = abs + open.len();
    }
    let mut j = 0;
    while let Some(p) = line[j..].find(&close) {
        *depth -= 1;
        j = j + p + close.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_round_trips_one_based_to_zero_based() {
        // compiler line 1 col 1 → LSP 0,0; and back.
        assert_eq!(to_lsp_pos(1, 1), Position::new(0, 0));
        assert_eq!(to_lsp_pos(3, 5), Position::new(2, 4));
        assert_eq!(from_lsp_pos(Position::new(0, 0)), (1, 1));
        assert_eq!(from_lsp_pos(Position::new(2, 4)), (3, 5));
    }

    #[test]
    fn diagnostic_maps_severity_and_range() {
        let d = sui_compiler::Diagnostic {
            line: 2,
            col: 3,
            end_line: 2,
            end_col: 6,
            severity: sui_compiler::DiagSeverity::Error,
            message: "boom".into(),
            rule: Some("sui-for-key".into()),
        };
        let l = diag_to_lsp(&d);
        assert_eq!(l.range.start, Position::new(1, 2));
        assert_eq!(l.range.end, Position::new(1, 5));
        assert_eq!(l.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(l.code, Some(NumberOrString::String("sui-for-key".into())));
        assert_eq!(l.source.as_deref(), Some("sui"));
    }

    #[test]
    fn completion_kind_maps_scope_shapes() {
        assert_eq!(completion_kind("signal"), CompletionItemKind::VARIABLE);
        assert_eq!(completion_kind("action"), CompletionItemKind::FUNCTION);
        assert_eq!(completion_kind("derived"), CompletionItemKind::PROPERTY);
        assert_eq!(completion_kind("data"), CompletionItemKind::FIELD);
        assert_eq!(completion_kind("value"), CompletionItemKind::VALUE);
    }

    #[test]
    fn parses_247_tokens_from_canonical_css() {
        assert_eq!(TOKENS.len(), 247, "expected 247 design tokens; got {}", TOKENS.len());
        let primary = TOKENS.iter().find(|(n, _)| n == "primary").expect("--primary");
        assert!(primary.1.contains("oklch("), "primary value should resolve to oklch: {:?}", primary.1);
        let space_3 = TOKENS.iter().find(|(n, _)| n == "space-3").expect("--space-3");
        assert_eq!(space_3.1, "12px");
        let radius = TOKENS.iter().find(|(n, _)| n == "radius").expect("--radius");
        assert_eq!(radius.1, "10px");
    }

    #[test]
    fn color_heuristic_classifies_oklch_and_lengths() {
        assert!(looks_like_color("oklch(0.67 0.16 58)"));
        assert!(looks_like_color("#fff"));
        assert!(looks_like_color("rgb(0,0,0)"));
        assert!(looks_like_color("hsl(0 0% 0%)"));
        assert!(!looks_like_color("12px"));
        assert!(!looks_like_color("400"));
        assert!(!looks_like_color("cubic-bezier(0.16, 1, 0.3, 1)"));
        assert!(!looks_like_color("none"));
    }

    fn build_item(uri: &str, line: u32, character: u32) -> Vec<lsp_types::CompletionItem> {
        let mut docs: Docs = HashMap::new();
        let url = Url::parse(uri).unwrap();
        docs.insert(url.clone(), TEST_DOC.to_string());
        let params = CompletionParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: url },
                position: Position::new(line, character),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        on_completion(&docs, RequestId::from(1), params).expect("completion")
    }

    // Rust's `\<newline>` line-continuation in string literals strips all
    // leading whitespace on the next line, which would eat the `  .x` indent
    // we want to test. Inline the literal on a single line.
    const TEST_DOC: &str = "<style>\n  .x { color: var(--p); padding: 12px; }\n</style>\n<div>{count()}</div>\n";

    #[test]
    fn css_context_emits_token_completions_with_color_kind() {
        // `<style>` is open on line 0, unclosed on line 1.
        // Cursor sits right after `var(--` on line 1 (0-indexed) col 20.
        // `<style>` is open on line 0, unclosed on line 1.
        let items = build_item("file:///tmp/test.sui", 1, 20);
        assert!(!items.is_empty(), "expected token completions inside var(--)");
        let primary = items.iter().find(|i| i.label == "--primary").expect("--primary item");
        assert_eq!(primary.kind, Some(CompletionItemKind::COLOR));
        let space_3 = items.iter().find(|i| i.label == "--space-3").expect("--space-3 item");
        assert_eq!(space_3.detail.as_deref(), Some("12px"));
        // Cursor is `var(--` → already typed `--`, so insert_text strips the dashes.
        assert_eq!(primary.insert_text.as_deref(), Some("primary"));
        assert_eq!(space_3.insert_text.as_deref(), Some("space-3"));
    }

    #[test]
    fn css_context_with_typed_prefix_still_completes() {
        // Cursor inside an unclosed `var(--sp` — should still emit tokens,
        // with insert_text that doesn't double the leading `--`.
        let doc = "<style>\n  .x { color: var(--sp; }\n</style>\n";
        let mut docs: Docs = HashMap::new();
        let url = Url::parse("file:///tmp/test_prefix.sui").unwrap();
        docs.insert(url.clone(), doc.to_string());
        let params = CompletionParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: url },
                position: Position::new(1, 24), // right after `var(--sp`
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        let items = on_completion(&docs, RequestId::from(1), params).expect("completion");
        let space_3 = items.iter().find(|i| i.label == "--space-3").expect("--space-3 item");
        // User has typed `--sp`; insert_text includes the full `--name`
        // because the prefix doesn't end with `--` any more.
        assert_eq!(space_3.insert_text.as_deref(), Some("--space-3"));
    }

    #[test]
    fn css_context_inside_style_attribute_completes() {
        // `</div>` is on a separate line so the `"` close-quote after
        // `var(--` doesn't immediately close the attribute value before
        // our cursor position.
        let doc = "<div style=\"color: var(--\">\n</div>\n";
        let mut docs: Docs = HashMap::new();
        let url = Url::parse("file:///tmp/test_attr.sui").unwrap();
        docs.insert(url.clone(), doc.to_string());
        let params = CompletionParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: url },
                position: Position::new(0, 25), // right after `var(--`, before the closing `"`
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        let items = on_completion(&docs, RequestId::from(1), params).expect("completion");
        let primary = items.iter().find(|i| i.label == "--primary").expect("--primary item");
        assert_eq!(primary.kind, Some(CompletionItemKind::COLOR));
    }

    #[test]
    fn markup_position_does_not_emit_token_completions() {
        // Line 3 of TEST_DOC is `<div>{count()}</div>` — markup, no tokens.
        let items = build_item("file:///tmp/test.sui", 3, 13);
        assert!(
            items.iter().all(|i| !i.label.starts_with("--")),
            "markup position leaked token completions: {items:?}"
        );
    }

    #[test]
    fn css_value_without_var_does_not_complete() {
        // Inside a style block, but the line is `padding: 12px;` — no `var(`,
        // doesn't end with `--`. Tokens should not fire.
        let doc = "<style>\n  .x { padding: 12px; }\n</style>\n";
        let mut docs: Docs = HashMap::new();
        let url = Url::parse("file:///tmp/test_plain.sui").unwrap();
        docs.insert(url.clone(), doc.to_string());
        let params = CompletionParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: url },
                position: Position::new(1, 18), // inside the style block, on `padding: 12px`
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        let items = on_completion(&docs, RequestId::from(1), params).expect("completion");
        assert!(
            items.iter().all(|i| !i.label.starts_with("--")),
            "expected no token completions without var(--); got {items:?}"
        );
    }

    #[test]
    fn trigger_characters_include_dash_and_paren() {
        let caps = server_capabilities();
        let triggers = caps
            .completion_provider
            .as_ref()
            .and_then(|c| c.trigger_characters.as_ref())
            .expect("trigger_characters");
        assert!(triggers.contains(&"-".to_string()), "missing '-' trigger");
        assert!(triggers.contains(&"(".to_string()), "missing '(' trigger");
    }

}
