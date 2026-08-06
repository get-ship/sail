//! `sail-lsp` — Language Server for `.sail` templates.
//!
//! A thin LSP/stdio shell over the `sail-compiler` analysis facade, plus a
//! **tsgo** sidecar for TypeScript intelligence inside `<controller>`,
//! `<scope>`, and `{…}` template expressions (SPEC-LSP-1). Structure
//! intelligence (elements, props, icons, sail diagnostics) stays in Rust;
//! position-based TS requests are proxied to tsgo against a per-file virtual
//! TS document and mapped back to `.sail` offsets.
//!
//! Transport is the synchronous `lsp-server` loop (rust-analyzer lineage);
//! `lsp-types` is used only to (de)serialize protocol payloads. Capabilities
//! are pushed diagnostics on open/change/save, plus hover / completion /
//! go-to-definition, code actions (auto-fix), and icon-name completion.
//!
//! When `tsgo` is missing the server degrades to structure-only (one log line).
//!
//! Library callers (Studio via sail-ops) use [`intelligence`] — the same
//! virtual_ts + tsgo path as this stdio shell.

mod intelligence;
mod tsgo;
mod virtual_ts;

/// Public library API: hover / complete / diagnostics without stdio LSP.
pub use intelligence::{
    CompletionItemPayload, DiagPayload, HoverPayload, Intelligence, RangePayload, complete_sail,
    complete_with_session, completion_payloads_from_tsgo, diagnostics_sail,
    diagnostics_with_session, doc_key, hover_payload_from_tsgo, hover_sail, hover_with_session,
    with_global_session,
};

// Integration tests (`tests/tsgo_proxy.rs`) exercise the public surface of the
// virtual-doc builder and tsgo session without going through stdio.
#[doc(hidden)]
pub mod tsgo_test {
    pub use super::tsgo::{TsgoSession, discover_tsgo};
}
#[doc(hidden)]
pub mod virtual_ts_test {
    pub use super::virtual_ts::{
        RegionKind, VirtualFile, VirtualTs, build_virtual_ts, offset_to_lsp_position,
    };
}

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, CompletionItem, CompletionItemKind,
    CompletionOptions, CompletionParams, Diagnostic, DiagnosticSeverity, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability, Location,
    MarkupContent, MarkupKind, NumberOrString, OneOf, OptionalVersionedTextDocumentIdentifier,
    Position, PublishDiagnosticsParams, Range, ServerCapabilities, TextDocumentEdit,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url, WorkspaceEdit,
};

use tsgo::TsgoSession;

type Docs = HashMap<Url, String>;

pub fn run() -> anyhow::Result<()> {
    let (connection, io_threads) = Connection::stdio();
    let capabilities = serde_json::to_value(server_capabilities())?;
    let _init = connection.initialize(capabilities)?;
    main_loop(&connection)?;
    io_threads.join()?;
    Ok(())
}

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // FULL sync: each change resends the whole document. `.sail` files are
        // small; incremental sync isn't worth the complexity here.
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec!["-".into(), "(".into(), "\"".into(), "'".into()]),
            ..Default::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn main_loop(conn: &Connection) -> anyhow::Result<()> {
    let mut docs: Docs = HashMap::new();
    // Discover tsgo from CWD / repo; degrades cleanly when absent.
    let mut tsgo = TsgoSession::start(std::env::current_dir().ok().as_deref());
    for msg in &conn.receiver {
        match msg {
            Message::Request(req) => {
                if conn.handle_shutdown(&req)? {
                    return Ok(());
                }
                handle_request(conn, &docs, &mut tsgo, req)?;
            }
            Message::Notification(note) => handle_notification(conn, &mut docs, &mut tsgo, note)?,
            Message::Response(_) => {}
        }
    }
    Ok(())
}

// ── notifications: document sync → push diagnostics ────────────────────────

fn handle_notification(
    conn: &Connection,
    docs: &mut Docs,
    tsgo: &mut TsgoSession,
    note: lsp_server::Notification,
) -> anyhow::Result<()> {
    match note.method.as_str() {
        "textDocument/didOpen" => {
            let p: lsp_types::DidOpenTextDocumentParams = serde_json::from_value(note.params)?;
            let uri = p.text_document.uri.clone();
            docs.insert(uri.clone(), p.text_document.text.clone());
            tsgo.on_sail_change(uri.as_str(), &p.text_document.text);
            publish_diagnostics(conn, tsgo, &uri, &p.text_document.text)?;
        }
        "textDocument/didChange" => {
            let p: lsp_types::DidChangeTextDocumentParams = serde_json::from_value(note.params)?;
            // FULL sync: the last change carries the entire new text.
            if let Some(change) = p.content_changes.into_iter().last() {
                let uri = p.text_document.uri.clone();
                docs.insert(uri.clone(), change.text.clone());
                tsgo.on_sail_change(uri.as_str(), &change.text);
                publish_diagnostics(conn, tsgo, &uri, &change.text)?;
            }
        }
        "textDocument/didSave" => {
            let p: lsp_types::DidSaveTextDocumentParams = serde_json::from_value(note.params)?;
            let uri = p.text_document.uri;
            if let Some(text) = docs.get(&uri).cloned() {
                publish_diagnostics(conn, tsgo, &uri, &text)?;
            }
        }
        "textDocument/didClose" => {
            let p: lsp_types::DidCloseTextDocumentParams = serde_json::from_value(note.params)?;
            tsgo.on_sail_close(p.text_document.uri.as_str());
            docs.remove(&p.text_document.uri);
        }
        _ => {}
    }
    Ok(())
}

fn publish_diagnostics(
    conn: &Connection,
    tsgo: &mut TsgoSession,
    uri: &Url,
    text: &str,
) -> anyhow::Result<()> {
    let path = uri.to_file_path().ok();
    let analysis = sail_compiler::analyze(text, path.as_deref(), &[]);
    let mut diagnostics: Vec<Diagnostic> = analysis.diagnostics.iter().map(diag_to_lsp).collect();
    // Merge mapped tsgo diagnostics (controller / template expression typos).
    if tsgo.is_live() {
        diagnostics.extend(tsgo.diagnostics_mapped(uri.as_str(), text));
    }
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics,
        version: None,
    };
    conn.sender
        .send(Message::Notification(lsp_server::Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params: serde_json::to_value(params)?,
        }))?;
    Ok(())
}

// ── requests: hover / completion / definition ──────────────────────────────

fn handle_request(
    conn: &Connection,
    docs: &Docs,
    tsgo: &mut TsgoSession,
    req: Request,
) -> anyhow::Result<()> {
    let resp = match req.method.as_str() {
        "textDocument/hover" => respond(req, |id, params| on_hover(docs, tsgo, id, params)),
        "textDocument/completion" => {
            respond(req, |id, params| on_completion(docs, tsgo, id, params))
        }
        "textDocument/definition" => {
            respond(req, |id, params| on_definition(docs, tsgo, id, params))
        }
        "textDocument/codeAction" => respond(req, |id, params| on_code_action(docs, id, params)),
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
            None => Some(Response {
                id: id2,
                result: Some(serde_json::Value::Null),
                error: None,
            }),
        },
        Err(_) => Some(Response {
            id,
            result: Some(serde_json::Value::Null),
            error: None,
        }),
    }
}

fn cast<P: serde::de::DeserializeOwned>(
    req: Request,
) -> Result<(RequestId, P), ExtractError<Request>> {
    let id = req.id.clone();
    let params = serde_json::from_value(req.params).map_err(|_| ExtractError::JsonError {
        method: req.method.clone(),
        error: serde_json::Error::io(std::io::Error::other("bad params")),
    })?;
    Ok((id, params))
}

fn on_hover(
    docs: &Docs,
    tsgo: &mut TsgoSession,
    _id: RequestId,
    params: HoverParams,
) -> Option<Hover> {
    let tdp = params.text_document_position_params;
    let (text, path) = doc(docs, &tdp.text_document.uri)?;
    // Prefer tsgo with correct UTF-16 → byte offset (LSP positions are UTF-16).
    if tsgo.is_live() {
        let col_byte = utf16_col_to_byte(
            &text,
            tdp.position.line as usize,
            tdp.position.character as usize,
        );
        let sail_offset = line_byte_offset(&text, tdp.position.line as usize, col_byte);
        if let Some(raw) = tsgo.hover_mapped(tdp.text_document.uri.as_str(), &text, sail_offset) {
            if let Some(p) = intelligence::hover_payload_from_tsgo(raw) {
                return Some(hover_from_payload(p));
            }
        }
    }
    // Structure fallback via shared library path.
    let (line, col) = from_lsp_pos(tdp.position);
    let payload = intelligence::hover_with_session(
        tsgo,
        &text,
        path.as_deref(),
        tdp.text_document.uri.as_str(),
        line as u32,
        col as u32,
    )?;
    Some(hover_from_payload(payload))
}

fn hover_from_payload(p: intelligence::HoverPayload) -> Hover {
    let range = p.range.map(|r| {
        Range::new(
            Position::new(
                r.start_line.saturating_sub(1),
                r.start_col.saturating_sub(1),
            ),
            Position::new(r.end_line.saturating_sub(1), r.end_col.saturating_sub(1)),
        )
    });
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: p.contents,
        }),
        range,
    }
}

fn on_completion(
    docs: &Docs,
    tsgo: &mut TsgoSession,
    _id: RequestId,
    params: CompletionParams,
) -> Option<Vec<CompletionItem>> {
    let tdp = params.text_document_position;
    let (text, path) = doc(docs, &tdp.text_document.uri)?;
    let pos = tdp.position;
    let col = utf16_col_to_byte(&text, pos.line as usize, pos.character as usize);
    if is_css_token_context(&text, pos.line as usize, col) {
        return Some(token_completion_items(&text, pos.line as usize, col));
    }
    if let Some(prefix) = icon_name_prefix(&text, pos.line as usize, col) {
        return Some(icon_completion_items(&prefix));
    }
    if let Some(prefix) = sx_attr_prefix(&text, pos.line as usize, col) {
        return Some(sx_completion_items(prefix));
    }
    // L1 element surface — when the user is typing a tag open (`<partial`
    // or `<>`), offer the language's first-class elements. The list is
    // the same one the skill documents and the consistency test
    // enforces, so an element the LSP doesn't know about is a test
    // failure, not a silent gap.
    if let Some(prefix) = tag_open_prefix(&text, pos.line as usize, col) {
        return Some(l1_completion_items(&prefix));
    }
    // Proxy to tsgo inside TS regions (UTF-16-correct offset), then structure fallback.
    if tsgo.is_live() {
        let sail_offset = line_byte_offset(&text, pos.line as usize, col);
        if let Some(raw) =
            tsgo.completion_mapped(tdp.text_document.uri.as_str(), &text, sail_offset)
        {
            let payloads = intelligence::completion_payloads_from_tsgo(raw);
            if !payloads.is_empty() {
                return Some(
                    payloads
                        .into_iter()
                        .map(completion_item_from_payload)
                        .collect(),
                );
            }
        }
    }
    let (line, col_1) = from_lsp_pos(pos);
    let payloads = intelligence::complete_with_session(
        // Pass a non-live path for structure-only: session still consulted but
        // tsgo already tried above; complete_with_session will re-try tsgo which
        // is fine (cached virtuals) and fall through to structure symbols.
        tsgo,
        &text,
        path.as_deref(),
        tdp.text_document.uri.as_str(),
        line as u32,
        col_1 as u32,
    );
    Some(
        payloads
            .into_iter()
            .map(completion_item_from_payload)
            .collect(),
    )
}

fn completion_item_from_payload(p: intelligence::CompletionItemPayload) -> CompletionItem {
    CompletionItem {
        label: p.label,
        kind: p
            .kind
            .as_deref()
            .map(completion_kind)
            .or(Some(CompletionItemKind::VALUE)),
        detail: p.detail,
        insert_text: p.insert_text,
        documentation: p.documentation.map(|d| {
            lsp_types::Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: d,
            })
        }),
        ..Default::default()
    }
}

fn on_code_action(
    docs: &Docs,
    _id: RequestId,
    params: CodeActionParams,
) -> Option<CodeActionResponse> {
    let uri = params.text_document.uri;
    let (text, path) = doc(docs, &uri)?;
    let file_key = path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("_buffer.sail");
    let plans = sail_compiler::edit::suggest_fixes_for_source(&text, file_key, path.as_deref());
    if plans.is_empty() {
        return Some(vec![]);
    }
    let mut out: CodeActionResponse = Vec::new();
    for plan in plans {
        let Some(patch) = plan.text_patch.as_ref() else {
            continue;
        };
        // Only offer when the diagnostic range overlaps the requested range.
        let start = offset_to_position(&text, patch.start);
        let end = offset_to_position(&text, patch.end);
        let edit_range = Range::new(start, end);
        if !ranges_overlap(edit_range, params.range) && !params.context.diagnostics.is_empty() {
            // Still offer if any context diagnostic rule matches.
            let rule_match = params.context.diagnostics.iter().any(|d| {
                d.code
                    .as_ref()
                    .map(|c| match c {
                        NumberOrString::String(s) => s == &plan.rule,
                        NumberOrString::Number(_) => false,
                    })
                    .unwrap_or(false)
            });
            if !rule_match {
                continue;
            }
        }
        let action = CodeAction {
            title: format!("Fix {}: {}", plan.rule, plan.message),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(WorkspaceEdit {
                changes: None,
                document_changes: Some(lsp_types::DocumentChanges::Edits(vec![TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: uri.clone(),
                        version: None,
                    },
                    edits: vec![lsp_types::OneOf::Left(TextEdit {
                        range: edit_range,
                        new_text: patch.replacement.clone(),
                    })],
                }])),
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(plan.rule == "sail-participation-style"),
            disabled: None,
            data: None,
        };
        out.push(CodeActionOrCommand::CodeAction(action));
    }
    Some(out)
}

fn offset_to_position(text: &str, offset: usize) -> Position {
    let (line, col) = sail_compiler::diagnostics::line_col(text, offset.min(text.len()));
    to_lsp_pos(line, col)
}

fn ranges_overlap(a: Range, b: Range) -> bool {
    // Inclusive-ish overlap in LSP positions.
    !(a.end.line < b.start.line
        || (a.end.line == b.start.line && a.end.character < b.start.character)
        || b.end.line < a.start.line
        || (b.end.line == a.start.line && b.end.character < a.start.character))
}

/// Detect `name="…"` / `name='…'` on an `<icon` open tag. Returns the partial
/// name the user is typing.
fn icon_name_prefix(text: &str, line_idx: usize, col: usize) -> Option<String> {
    let line = text.lines().nth(line_idx)?;
    let prefix = &line[..col.min(line.len())];
    // Find last `<icon` before cursor on this line or earlier lines (simple).
    let before = {
        let mut acc = String::new();
        for (i, l) in text.lines().enumerate() {
            if i < line_idx {
                acc.push_str(l);
                acc.push('\n');
            } else if i == line_idx {
                acc.push_str(prefix);
                break;
            }
        }
        acc
    };
    let lower = before.to_ascii_lowercase();
    let icon_at = lower.rfind("<icon")?;
    let after_icon = &before[icon_at..];
    // Still inside the open tag (no `>` after <icon yet).
    if after_icon.find('>').is_some() {
        return None;
    }
    // name=" partial
    for quote in ['"', '\''] {
        let key = format!("name={quote}");
        if let Some(rel) = after_icon.rfind(&key) {
            let start = rel + key.len();
            let partial = &after_icon[start..];
            // no closing quote yet
            if partial.contains(quote) {
                return None;
            }
            return Some(partial.to_string());
        }
    }
    None
}

fn icon_completion_items(prefix: &str) -> Vec<CompletionItem> {
    let lower = prefix.to_ascii_lowercase();
    ICON_NAMES
        .iter()
        .filter(|n| n.starts_with(&lower) || lower.is_empty())
        .take(200)
        .map(|name| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some("lucide icon".into()),
            insert_text: Some(name.clone()),
            ..Default::default()
        })
        .collect()
}

/// Lucide icon registry (TOOL-10 asset). Keys only — SVG bodies unused here.
const LUCIDE_ICONS_JSON: &str = include_str!("../../../deck/deck-core/assets/lucide-icons.json");

static ICON_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let v: serde_json::Value =
        serde_json::from_str(LUCIDE_ICONS_JSON).unwrap_or(serde_json::json!({}));
    let mut names: Vec<String> = v
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    names.sort();
    names
});

/// Detect a tag-open position: a `<` followed by a partial tag name on
/// the same line, with the cursor at the end. Returns the partial name
/// the user has typed (so completion can scope to it) or `None` when
/// we're not in a tag-open position. Doesn't try to be perfect; a
/// partial `<partial` inside an attribute value would also match — the
/// LSP completion is "offer the L1 surface where it's most useful", not
/// a parser.
fn tag_open_prefix(text: &str, line_idx: usize, col: usize) -> Option<String> {
    if line_idx >= text.lines().count() {
        return None;
    }
    let line = text.lines().nth(line_idx)?;
    // Walk back from the cursor to the start of the line, building
    // the partial tag name char by char. Stop at the first non-tag
    // char. If we hit a `<` before that, we're in a tag-open position.
    let bytes = line.as_bytes();
    if col > bytes.len() {
        return None;
    }
    let mut start = col;
    while start > 0 {
        let c = bytes[start - 1];
        if c.is_ascii_alphanumeric() || c == b'-' {
            start -= 1;
        } else {
            break;
        }
    }
    if start == 0 || bytes[start - 1] != b'<' {
        return None;
    }
    Some(line[start..col].to_string())
}

/// L1 element completions. Sourced from the same manifest the skill
/// documents; the consistency test in
/// `crates/sail/compiler/tests/l1_manifest.rs` keeps the list honest.
fn l1_completion_items(prefix: &str) -> Vec<CompletionItem> {
    let lower = prefix.to_ascii_lowercase();
    let mut out: Vec<CompletionItem> = Vec::new();
    for entry in sail_compiler::l1_entries() {
        if !entry.tag.to_ascii_lowercase().starts_with(&lower) {
            continue;
        }
        let kind = match entry.kind {
            sail_compiler::L1Kind::StdlibElement => "L1 stdlib",
            sail_compiler::L1Kind::NativeReskin => "L1 reskin",
        };
        out.push(CompletionItem {
            label: entry.tag.clone(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(format!("{kind} — {}", entry.description)),
            ..Default::default()
        });
    }
    out
}

fn on_definition(
    docs: &Docs,
    tsgo: &mut TsgoSession,
    _id: RequestId,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let tdp = params.text_document_position_params;
    let (text, path) = doc(docs, &tdp.text_document.uri)?;
    if tsgo.is_live() {
        let col_byte = utf16_col_to_byte(
            &text,
            tdp.position.line as usize,
            tdp.position.character as usize,
        );
        let sail_offset = line_byte_offset(&text, tdp.position.line as usize, col_byte);
        if let Some(raw) =
            tsgo.definition_mapped(tdp.text_document.uri.as_str(), &text, sail_offset)
        {
            if let Some(resp) = definition_from_tsgo(raw) {
                return Some(resp);
            }
        }
    }
    let (line, col) = from_lsp_pos(tdp.position);
    let h = sail_compiler::hover_at(&text, path.as_deref(), line, col)?;
    // Only controller-declared symbols carry a jump target (line > 0).
    if h.line == 0 {
        return None;
    }
    let analysis = sail_compiler::analyze(&text, path.as_deref(), &[]);
    let ctrl = analysis.controller_path?;
    let uri = Url::from_file_path(&ctrl).ok()?;
    let pos = to_lsp_pos(h.line, h.col);
    Some(GotoDefinitionResponse::Scalar(Location {
        uri,
        range: Range::new(pos, pos),
    }))
}

/// Byte offset of `(line, col_byte)` in `text` (line 0-based, col is a byte
/// index within the line from [`utf16_col_to_byte`]).
fn line_byte_offset(text: &str, line_idx: usize, col_byte: usize) -> usize {
    let mut idx = 0usize;
    for (i, row) in text.split_inclusive('\n').enumerate() {
        if i == line_idx {
            let content_len = row.strip_suffix('\n').map(|s| s.len()).unwrap_or(row.len());
            return idx + col_byte.min(content_len);
        }
        idx += row.len();
    }
    text.len()
}

fn definition_from_tsgo(raw: serde_json::Value) -> Option<GotoDefinitionResponse> {
    serde_json::from_value(raw).ok()
}

// ── pure mappers (unit-tested) ─────────────────────────────────────────────

fn doc(docs: &Docs, uri: &Url) -> Option<(String, Option<std::path::PathBuf>)> {
    let text = docs.get(uri)?.clone();
    Some((text, uri.to_file_path().ok()))
}

/// LSP `character` (UTF-16 code units — the protocol default; we negotiate
/// no other `position_encoding`) → byte offset within the line, clamped to
/// the line length. Always a char boundary: using the raw UTF-16 count as a
/// byte index slices mid-char on any line with non-ASCII before the cursor.
fn utf16_col_to_byte(text: &str, line_idx: usize, utf16_col: usize) -> usize {
    let Some(line) = text.lines().nth(line_idx) else {
        return 0;
    };
    let mut units = 0usize;
    for (i, ch) in line.char_indices() {
        if units >= utf16_col {
            return i;
        }
        units += ch.len_utf16();
    }
    line.len()
}

/// Compiler 1-based `(line, col)` → LSP 0-based `Position`.
fn to_lsp_pos(line: usize, col: usize) -> Position {
    Position::new(line.saturating_sub(1) as u32, col.saturating_sub(1) as u32)
}

/// LSP 0-based `Position` → compiler 1-based `(line, col)`.
fn from_lsp_pos(p: Position) -> (usize, usize) {
    (p.line as usize + 1, p.character as usize + 1)
}

fn diag_to_lsp(d: &sail_compiler::Diagnostic) -> Diagnostic {
    Diagnostic {
        range: Range::new(to_lsp_pos(d.line, d.col), to_lsp_pos(d.end_line, d.end_col)),
        severity: Some(match d.severity {
            sail_compiler::DiagSeverity::Error => DiagnosticSeverity::ERROR,
            sail_compiler::DiagSeverity::Warning => DiagnosticSeverity::WARNING,
            sail_compiler::DiagSeverity::Fix => DiagnosticSeverity::INFORMATION,
        }),
        code: d.rule.clone().map(NumberOrString::String),
        source: Some("sail".to_string()),
        message: d.message.clone(),
        ..Default::default()
    }
}

fn completion_kind(kind: &str) -> CompletionItemKind {
    match kind {
        // Structure (sail_compiler) kinds
        "signal" | "ref" | "variable" => CompletionItemKind::VARIABLE,
        "derived" | "property" => CompletionItemKind::PROPERTY,
        "action" | "function" | "method" => CompletionItemKind::FUNCTION,
        "data" | "prop" | "field" => CompletionItemKind::FIELD,
        // tsgo CompletionItemKind names
        "class" => CompletionItemKind::CLASS,
        "interface" => CompletionItemKind::INTERFACE,
        "module" => CompletionItemKind::MODULE,
        "keyword" => CompletionItemKind::KEYWORD,
        "snippet" => CompletionItemKind::SNIPPET,
        "constant" | "enumMember" => CompletionItemKind::CONSTANT,
        "enum" => CompletionItemKind::ENUM,
        "constructor" => CompletionItemKind::CONSTRUCTOR,
        "typeParameter" => CompletionItemKind::TYPE_PARAMETER,
        "text" => CompletionItemKind::TEXT,
        "color" => CompletionItemKind::COLOR,
        "file" => CompletionItemKind::FILE,
        "folder" => CompletionItemKind::FOLDER,
        "reference" => CompletionItemKind::REFERENCE,
        "unit" => CompletionItemKind::UNIT,
        "value" => CompletionItemKind::VALUE,
        "struct" => CompletionItemKind::STRUCT,
        "event" => CompletionItemKind::EVENT,
        "operator" => CompletionItemKind::OPERATOR,
        _ => CompletionItemKind::VALUE,
    }
}

// ── design-token completion ────────────────────────────────────────────────

/// Canonical token source — `--name: value;` declarations under `:root` in
/// `packages/sail/tokens/src/tokens.css`. Embedded at build time so the LSP
/// has zero runtime IO and a missing token file is a compile error, not a
/// silent empty completion. This is the single source; the css-token lint
/// parses the same file (see `sail_lint::tokens`).
const TOKENS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/tokens.css");

/// `(name, value)` pairs with leading `--` stripped. Sorted by name so
/// completion order is stable across builds.
type TokenTable = Vec<(String, String)>;

static TOKENS: LazyLock<TokenTable> = LazyLock::new(|| {
    let mut by_name = parse_tokens(TOKENS_CSS);
    // Contextual tokens with no `:root` value (tone channel, floating-primitive
    // anchors, app-shell) — completable, but valued at runtime. Shared verbatim
    // with the css-token lint so the two registries can never diverge.
    for &name in sail_lint::tokens::CONVENTION_TOKENS {
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
    //   - `root_depths` — the depths at which `:root` blocks opened, so a
    //     `}` only exits a `:root` when it closes THAT block.
    // `:root` (with or without a `[data-…]` attribute selector) collects
    // tokens; `.dark`, media queries, utility classes don't. Skipping
    // `.dark` overrides is what gets us the canonical `:root` leaves instead
    // of the larger count you'd get reading every `--name: value;` in the file
    // (each `.dark`/media override redeclares names already collected).
    // Comments are skipped BEFORE anything else: prose in tokens.css quotes
    // code (`fx:loop={{ … }}`), and counting those braces once knocked
    // the walker out of `:root` mid-block, silently dropping every token
    // after the comment.
    let mut block_depth: i32 = 0;
    let mut root_depths: Vec<i32> = Vec::new();
    // Selector being built for a not-yet-opened block. When we see `{`,
    // the selector is committed and its depth is pushed onto `root_depths`
    // if the selector starts with `:root`.
    let mut selector = String::new();
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = '\0';
            for n in chars.by_ref() {
                if prev == '*' && n == '/' {
                    break;
                }
                prev = n;
            }
            continue;
        }
        match c {
            '{' => {
                let s = selector.trim();
                block_depth += 1;
                if s.starts_with(":root") {
                    root_depths.push(block_depth);
                }
                selector.clear();
            }
            '}' => {
                if root_depths.last() == Some(&block_depth) {
                    root_depths.pop();
                }
                if block_depth > 0 {
                    block_depth -= 1;
                }
            }
            _ if !root_depths.is_empty() && c == '-' && chars.peek() == Some(&'-') => {
                chars.next(); // consume second '-'
                // Name
                let mut name = String::new();
                for c in chars.by_ref() {
                    match c {
                        ':' => break,
                        _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' => name.push(c),
                        _ => {
                            name.clear();
                            break;
                        }
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
                            if !value.is_empty() {
                                value.push(' ');
                            }
                            value.push_str(buf.trim());
                        }
                        break;
                    }
                    if c == '\n' || c == '\r' {
                        if started && !buf.trim().is_empty() {
                            if !value.is_empty() {
                                value.push(' ');
                            }
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
    let Some(line) = text.lines().nth(line_idx) else {
        return false;
    };
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
    let Some(line) = text.lines().nth(line_idx) else {
        return false;
    };
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

/// Detect whether `pos` is inside an `style="…"` attribute value. Returns
/// the prefix (the partial atom the user is currently typing) so the
/// completion can scope to it. Mirrors `in_style_attr_value`'s scan but
/// also returns the prefix on the same line, up to the cursor.
fn sx_attr_prefix(text: &str, line_idx: usize, col: usize) -> Option<String> {
    let mut in_sx = false;
    let mut value_start: Option<usize> = None;
    for (i, line) in text.lines().enumerate() {
        let bytes = line.as_bytes();
        let mut j = 0;
        while j < bytes.len() {
            if i == line_idx && j == col {
                if !in_sx {
                    return None;
                }
                // The prefix the user is mid-typing: the last whitespace-
                // delimited token WITHIN the attribute value (not the `style="`
                // attribute name itself).
                let start = value_start.unwrap_or(j);
                let prefix = &line[start..j];
                let last = prefix
                    .rfind(char::is_whitespace)
                    .map(|k| start + k + 1)
                    .unwrap_or(start);
                return Some(line[last..j].to_string());
            }
            if !in_sx {
                if j + 4 <= bytes.len() && &bytes[j..j + 4] == b"sx=\"" {
                    in_sx = true;
                    value_start = Some(j + 4);
                    j += 4;
                    continue;
                }
            } else if bytes[j] == b'"' {
                in_sx = false;
                value_start = None;
                j += 1;
                continue;
            } else {
                j += 1;
                continue;
            }
            j += 1;
        }
    }
    None
}

/// sx completion items, sourced from `sail_lint::sx_vocab` — the same
/// tables the lint validates and the docs teach. So what an agent is
/// offered is what the lint will accept, by construction.
///
/// Suggests:
///   - bare atom names (`hidden`, `truncate`, `italic`)
///   - one item per (name, value) pair from `vocabulary()`
///   - refused atoms (with detail naming the replacement)
fn sx_completion_items(prefix: String) -> Vec<CompletionItem> {
    let mut out: Vec<CompletionItem> = Vec::new();
    let lower = prefix.to_ascii_lowercase();

    // Bare atoms: the atom itself is the value. Surfaced as `hidden`,
    // `truncate`, `italic`, etc. so the user can see the keyword shape.
    for name in sail_lint::sx_vocab::names() {
        if name.starts_with(&lower) && !name.contains('-') {
            out.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("bare atom".into()),
                ..Default::default()
            });
        }
    }

    // (name, values) — one completion per legal (name-value) pair. The
    // label is the full atom (`p-4`, `bg-surface-1`); the detail is the
    // CSS shape it lowers to.
    for (name, values) in sail_lint::sx_vocab::vocabulary() {
        let hyphen = format!("{name}-");
        if !lower.is_empty() && !name.starts_with(&lower) && !hyphen.starts_with(&lower) {
            continue;
        }
        for value in values {
            let label = format!("{name}-{value}");
            out.push(CompletionItem {
                label,
                kind: Some(CompletionItemKind::VALUE),
                detail: Some(format!("{name} = {value}")),
                ..Default::default()
            });
        }
    }

    // Refusals — atoms the lint rejects, with the thing to use instead.
    // Surfaced so the agent sees WHY a Tailwind-prior favourite fails.
    for (atom, instead) in sail_lint::sx_vocab::refusals() {
        if !lower.is_empty() && !atom.starts_with(&lower) {
            continue;
        }
        out.push(CompletionItem {
            label: atom.clone(),
            kind: Some(CompletionItemKind::REFERENCE),
            detail: Some(format!("REFUSED — {instead}")),
            ..Default::default()
        });
    }
    out
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
/// attribute without quotes — uncommon but legal in `.sail`).
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
/// (-1). The tag match is naive (substring) — `.sail` files don't embed
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
        let d = sail_compiler::Diagnostic {
            line: 2,
            col: 3,
            end_line: 2,
            end_col: 6,
            severity: sail_compiler::DiagSeverity::Error,
            message: "boom".into(),
            rule: Some("sail-for-key".into()),
        };
        let l = diag_to_lsp(&d);
        assert_eq!(l.range.start, Position::new(1, 2));
        assert_eq!(l.range.end, Position::new(1, 5));
        assert_eq!(l.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(l.code, Some(NumberOrString::String("sail-for-key".into())));
        assert_eq!(l.source.as_deref(), Some("sail"));
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
    fn parses_design_tokens_from_canonical_css() {
        // Robust against token growth: assert a FLOOR (the table only ever
        // grows as the design system adds tokens; it was 247, is 374 today)
        // plus the presence of key token FAMILIES and a few pinned anchor
        // values. A magic exact count rots on every token add — a floor +
        // family coverage proves the parser is still reading `:root` leaves
        // correctly without breaking on unrelated additions.
        assert!(
            TOKENS.len() >= 300,
            "token table shrank unexpectedly — expected >= 300 :root leaves, got {}. \
             A DROP means the `:root` extractor regressed (or tokens.css lost blocks); \
             investigate before lowering this floor.",
            TOKENS.len()
        );

        // Key families an author completes against must each be non-empty.
        // (One representative prefix per major axis: color scales, spacing,
        // radii, typography, motion, elevation, and the tone channel.)
        // Families an author completes against — must each be non-empty.
        // (`tone-*` ramps were retired; use surface/primary + glow instead.)
        for family in [
            "space-",
            "radius",
            "primary",
            "surface-",
            "font-",
            "duration-",
            "shadow-",
        ] {
            assert!(
                TOKENS.iter().any(|(n, _)| n.starts_with(family)),
                "expected at least one `--{family}*` token family member; \
                 none found — the parser or tokens.css regressed"
            );
        }

        // Pinned anchors — stable, load-bearing values. If these change the
        // design system changed deliberately; update them here.
        let primary = TOKENS
            .iter()
            .find(|(n, _)| n == "primary")
            .expect("--primary");
        assert!(
            primary.1.contains("oklch("),
            "primary value should resolve to oklch: {:?}",
            primary.1
        );
        let space_3 = TOKENS
            .iter()
            .find(|(n, _)| n == "space-3")
            .expect("--space-3");
        assert_eq!(space_3.1, "12px");
        let radius = TOKENS
            .iter()
            .find(|(n, _)| n == "radius")
            .expect("--radius");
        assert_eq!(radius.1, "10px");

        // Convention tokens (no `:root` value) must still be merged in.
        for &name in sail_lint::tokens::CONVENTION_TOKENS {
            assert!(
                TOKENS
                    .iter()
                    .any(|(n, _)| n == name.trim_start_matches("--")),
                "convention token `{name}` missing from the completion table"
            );
        }
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

    fn degraded_tsgo() -> TsgoSession {
        // Structure-only session: no tsgo subprocess.
        TsgoSession::start(Some(std::path::Path::new("/nonexistent-sail-tsgo-root")))
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
        let mut tsgo = degraded_tsgo();
        on_completion(&docs, &mut tsgo, RequestId::from(1), params).expect("completion")
    }

    // Rust's `\<newline>` line-continuation in string literals strips all
    // leading whitespace on the next line, which would eat the `  .x` indent
    // we want to test. Inline the literal on a single line.
    const TEST_DOC: &str =
        "<style>\n  .x { color: var(--p); padding: 12px; }\n</style>\n<div>{count()}</div>\n";

    #[test]
    fn css_context_emits_token_completions_with_color_kind() {
        // `<style>` is open on line 0, unclosed on line 1.
        // Cursor sits right after `var(--` on line 1 (0-indexed) col 20.
        // `<style>` is open on line 0, unclosed on line 1.
        let items = build_item("file:///tmp/test.sail", 1, 20);
        assert!(
            !items.is_empty(),
            "expected token completions inside var(--)"
        );
        let primary = items
            .iter()
            .find(|i| i.label == "--primary")
            .expect("--primary item");
        assert_eq!(primary.kind, Some(CompletionItemKind::COLOR));
        let space_3 = items
            .iter()
            .find(|i| i.label == "--space-3")
            .expect("--space-3 item");
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
        let url = Url::parse("file:///tmp/test_prefix.sail").unwrap();
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
        let mut tsgo = degraded_tsgo();
        let items =
            on_completion(&docs, &mut tsgo, RequestId::from(1), params).expect("completion");
        let space_3 = items
            .iter()
            .find(|i| i.label == "--space-3")
            .expect("--space-3 item");
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
        let url = Url::parse("file:///tmp/test_attr.sail").unwrap();
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
        let mut tsgo = degraded_tsgo();
        let items =
            on_completion(&docs, &mut tsgo, RequestId::from(1), params).expect("completion");
        let primary = items
            .iter()
            .find(|i| i.label == "--primary")
            .expect("--primary item");
        assert_eq!(primary.kind, Some(CompletionItemKind::COLOR));
    }

    #[test]
    fn markup_position_does_not_emit_token_completions() {
        // Line 3 of TEST_DOC is `<div>{count()}</div>` — markup, no tokens.
        let items = build_item("file:///tmp/test.sail", 3, 13);
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
        let url = Url::parse("file:///tmp/test_plain.sail").unwrap();
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
        let mut tsgo = degraded_tsgo();
        let items =
            on_completion(&docs, &mut tsgo, RequestId::from(1), params).expect("completion");
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
        assert!(
            triggers.contains(&"\"".to_string()),
            "missing quote trigger"
        );
        assert!(
            caps.code_action_provider.is_some(),
            "code actions required for TOOL-9 auto-fix"
        );
    }

    #[test]
    fn icon_name_prefix_detects_partial() {
        let src = r#"<icon name="sea"#;
        let p = icon_name_prefix(src, 0, src.len()).expect("prefix");
        assert_eq!(p, "sea");
        assert!(icon_name_prefix(r#"<div name="sea"#, 0, 14).is_none());
    }

    #[test]
    fn icon_completion_from_lucide_registry() {
        let items = icon_completion_items("sear");
        assert!(
            items.iter().any(|i| i.label == "search"),
            "expected search in lucide registry: {:?}",
            items.iter().map(|i| &i.label).take(10).collect::<Vec<_>>()
        );
    }

    #[test]
    fn code_action_offers_participation_fix() {
        // Product form: bare participation attr → box channel (not retired style={style.grow}).
        let src = r#"<div grow>x</div>"#;
        let mut docs = Docs::new();
        let uri = Url::parse("file:///t.sail").unwrap();
        docs.insert(uri.clone(), src.into());
        let params = CodeActionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
            range: Range::new(Position::new(0, 0), Position::new(0, 16)),
            context: lsp_types::CodeActionContext {
                diagnostics: vec![Diagnostic {
                    range: Range::new(Position::new(0, 5), Position::new(0, 9)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("sail-participation-style".into())),
                    message: "grow".into(),
                    ..Default::default()
                }],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let actions = on_code_action(&docs, RequestId::from(1), params).unwrap();
        assert!(
            !actions.is_empty(),
            "expected participation-style code action"
        );
        let CodeActionOrCommand::CodeAction(a) = &actions[0] else {
            panic!("expected CodeAction");
        };
        assert!(
            a.title.contains("participation")
                || a.title.contains("grow")
                || a.title.contains("box"),
            "title: {}",
            a.title
        );
        assert!(a.edit.is_some());
        // Apply the edit and ensure bare attr is gone / box channel lands.
        let edit = a.edit.as_ref().unwrap();
        let changes = edit.document_changes.as_ref().unwrap();
        let lsp_types::DocumentChanges::Edits(edits) = changes else {
            panic!("expected Edits");
        };
        let lsp_types::OneOf::Left(te) = &edits[0].edits[0] else {
            panic!("expected TextEdit");
        };
        let start = te.range.start;
        let end = te.range.end;
        // rebuild source with replacement
        let start_off = {
            let mut line = 0u32;
            let mut col = 0u32;
            let mut off = 0usize;
            for (i, ch) in src.char_indices() {
                if line == start.line && col == start.character {
                    off = i;
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += ch.len_utf16() as u32;
                }
                off = i + ch.len_utf8();
            }
            off
        };
        let end_off = {
            let mut line = 0u32;
            let mut col = 0u32;
            let mut off = src.len();
            for (i, ch) in src.char_indices() {
                if line == end.line && col == end.character {
                    off = i;
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += ch.len_utf16() as u32;
                }
                off = i + ch.len_utf8();
            }
            off
        };
        let mut next = String::new();
        next.push_str(&src[..start_off]);
        next.push_str(&te.new_text);
        next.push_str(&src[end_off..]);
        assert!(
            next.contains("box:") && next.contains("grow(1)") && !next.contains("<div grow"),
            "fixed source: {next}"
        );
    }

    #[test]
    fn utf16_col_lands_on_char_boundaries() {
        // `°` is 1 UTF-16 unit but 2 UTF-8 bytes — a raw `character` used as
        // a byte index lands on the continuation byte and panics any slice.
        let src = "content: \"°°\";";
        assert_eq!(utf16_col_to_byte(src, 0, 10), 10); // ASCII prefix: 1:1
        assert_eq!(utf16_col_to_byte(src, 0, 11), 12); // between the two °
        assert_eq!(utf16_col_to_byte(src, 0, 12), 14); // after both
        assert_eq!(utf16_col_to_byte(src, 0, 99), src.len()); // clamped
        assert_eq!(utf16_col_to_byte(src, 5, 0), 0); // line out of range
        // The completion helpers must not panic mid-char anymore.
        assert!(!line_prefix_ends_with_dd(
            src,
            0,
            utf16_col_to_byte(src, 0, 11)
        ));
    }

    #[test]
    fn sx_attr_prefix_detects_inside_quoted_sx() {
        // Detector keys on `sx="…"` (mint / foreign-idiom surface), not `style=`.
        // Cursor inside `<div sx="p">` — at the `p` (col 10).
        let src = r#"<div sx="p">x</div>"#;
        let prefix = sx_attr_prefix(src, 0, 10).expect("inside sx attr");
        assert_eq!(prefix, "p", "prefix should be the partial atom");
    }

    #[test]
    fn sx_attr_prefix_returns_none_outside_sx() {
        // Cursor inside `<div class="x">` — should not detect sx.
        let src = r#"<div class="x">y</div>"#;
        assert!(sx_attr_prefix(src, 0, 13).is_none());
    }

    #[test]
    fn sx_completion_items_emits_atoms_refusals_and_values() {
        // Empty prefix — every legal atom, every value, every refusal.
        let items = sx_completion_items(String::new());
        // Some legal atom must appear.
        assert!(
            items.iter().any(|i| i.label == "p-4"),
            "expected `p-4` (canonical p-name-value) in completions"
        );
        assert!(
            items.iter().any(|i| i.label == "bg-surface-1"),
            "expected `bg-surface-1` in completions"
        );
        // A refused atom (Tailwind-prior) must appear, with the replacement.
        let flex = items
            .iter()
            .find(|i| i.label == "flex")
            .expect("`flex` refusal should be surfaced");
        let detail = flex.detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("REFUSED"),
            "refusal detail must mark REFUSED: {detail}"
        );
        assert!(
            detail.contains("<row>"),
            "refusal must name the replacement: {detail}"
        );
    }

    #[test]
    fn sx_completion_items_scopes_to_typed_prefix() {
        // Typing `bg-` should narrow to bg-prefixed completions only.
        let items = sx_completion_items("bg-".to_string());
        assert!(!items.is_empty(), "bg- prefix should produce items");
        // Every non-refusal label should start with `bg-`.
        for item in &items {
            let is_refusal = item
                .detail
                .as_deref()
                .map(|d| d.contains("REFUSED"))
                .unwrap_or(false);
            if !is_refusal {
                assert!(
                    item.label.starts_with("bg-") || item.label == "bg",
                    "non-refusal item under `bg-` prefix should start with `bg-`: {item:?}"
                );
            }
        }
    }

    #[test]
    fn sx_attr_prefix_picks_last_atom_after_whitespace() {
        // Cursor mid-typing the second atom: typed `bg` already, cursor
        // at col 15 (after the `g`, before the closing quote).
        let src = r#"<div sx="p-4 bg">x</div>"#;
        let prefix = sx_attr_prefix(src, 0, 15).expect("inside sx attr");
        assert_eq!(prefix, "bg", "prefix should be the typed prefix `bg`");
    }

    #[test]
    fn sx_attr_prefix_does_not_match_sxy_or_class() {
        // `sxy=` is not sx; `class=` is not sx.
        assert!(sx_attr_prefix(r#"<div sxy="x">y</div>"#, 0, 11).is_none());
        assert!(sx_attr_prefix(r#"<div class="x">y</div>"#, 0, 13).is_none());
    }

    // ── L1 element completion (the spec's "make the surface enumerable") ──

    #[test]
    fn tag_open_prefix_detects_partial_tag() {
        // Cursor right after `<car` on line 0.
        let src = "<card>x</card>";
        assert_eq!(tag_open_prefix(src, 0, 4), Some("car".to_string()));
        assert_eq!(tag_open_prefix(src, 0, 5), Some("card".to_string()));
        // Right after `<` — empty prefix.
        assert_eq!(tag_open_prefix(src, 0, 1), Some(String::new()));
    }

    #[test]
    fn tag_open_prefix_ignores_non_tag_open() {
        // Inside a tag (cursor after `<card` on an attribute).
        let src = "<card>x</card>";
        // No `<` immediately before the cursor.
        assert_eq!(tag_open_prefix(src, 0, 6), None);
        // After `</`.
        assert_eq!(tag_open_prefix("</card>", 0, 6), None);
        // No `<` at all.
        assert_eq!(tag_open_prefix("plain text", 0, 5), None);
    }

    #[test]
    fn l1_completion_items_offers_stdlib_surface() {
        let empty = l1_completion_items("");
        assert!(
            empty.iter().any(|i| i.label == "card"),
            "<card> must be offered"
        );
        assert!(
            empty.iter().any(|i| i.label == "button"),
            "<button> must be offered"
        );
        // Post SPEC-L1-1: kbd is a proper stdlib element; hr reskin is gone
        // (use <separator>).
        assert!(
            empty.iter().any(|i| i.label == "kbd"),
            "<kbd> stdlib must be offered"
        );
        assert!(
            empty.iter().any(|i| i.label == "separator"),
            "<separator> must be offered"
        );
        assert!(
            empty.iter().any(|i| i.label == "accordion"),
            "<accordion> must be offered"
        );
        assert!(
            empty.iter().all(|i| i.label != "hr"),
            "bare <hr> reskin must not be offered"
        );
        let kbd = empty.iter().find(|i| i.label == "kbd").unwrap();
        assert!(kbd.detail.as_deref().unwrap().contains("stdlib"));
        let card = empty.iter().find(|i| i.label == "card").unwrap();
        assert!(card.detail.as_deref().unwrap().contains("stdlib"));
    }

    #[test]
    fn l1_completion_items_scopes_to_typed_prefix() {
        // Typing `<but` should narrow to but-prefixed tags.
        let items = l1_completion_items("but");
        assert!(items.iter().any(|i| i.label == "button"));
        for item in &items {
            assert!(
                item.label.to_ascii_lowercase().starts_with("but"),
                "unscoped completion leaked: {}",
                item.label
            );
        }
    }
}
