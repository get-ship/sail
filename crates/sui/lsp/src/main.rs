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

use std::collections::HashMap;

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
        completion_provider: Some(CompletionOptions::default()),
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
    let analysis = sui_compiler::analyze(text, path.as_deref());
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
    let analysis = sui_compiler::analyze(&text, path.as_deref());
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
}
