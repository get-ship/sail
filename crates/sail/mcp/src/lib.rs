//! Sail MCP transport — thin stdio JSON-RPC adapter over `sail-ops`.
//!
//! All tool logic lives in `sail_ops` (same handlers as CLI `sail op` and
//! Connect HTTP under `sail dev`). This crate only maps MCP tool names to
//! RPCs and frames stdio messages.

use sail_ops::{Ctx, call_mcp_tool, mcp_tool_descriptors};
use serde_json::{Value, json};

// Re-exports for tests / downstream that still import sail_mcp surface types.
pub use sail_ops::authoring::{run_check as sail_check, sail_layout_check};
pub use sail_ops::{CheckDiag, SurfaceTopic, surface_markdown};

// ── public API used by unit tests and `sail mcp` ────────────────────────────

/// Tool descriptors for MCP `tools/list` (from sail-ops registry).
pub fn tool_descriptors() -> Value {
    mcp_tool_descriptors()
}

/// Dispatch a tool call by name + JSON args → structured result.
///
/// Uses default [`Ctx`] from env (`SAIL_MCP_ROOT` / cwd / workspace).
pub fn call_tool(name: &str, args: &Value) -> Result<Value, String> {
    call_tool_with_ctx(name, args, &Ctx::from_env())
}

/// Same as [`call_tool`] with an explicit context (tests, embedded hosts).
pub fn call_tool_with_ctx(name: &str, args: &Value, ctx: &Ctx) -> Result<Value, String> {
    call_mcp_tool(name, args, ctx).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sail_ops::{Ctx, call_rpc};
    use serde_json::json;

    #[test]
    fn call_tool_surface_roundtrip() {
        let out = call_tool("sail_surface", &json!({"topic": "core"})).unwrap();
        assert_eq!(out["topic"], "core");
        assert!(out["markdown"].as_str().unwrap().contains("Hard rules"));
    }

    #[test]
    fn mcp_tool_and_call_rpc_same_result() {
        let ctx = Ctx::from_env();
        let args = json!({"topic": "layout"});
        let mcp = call_tool_with_ctx("sail_surface", &args, &ctx).unwrap();
        let rpc = call_rpc("SailAuthoring", "Surface", &args, &ctx).unwrap();
        assert_eq!(mcp, rpc);
    }

    #[test]
    fn sail_edit_registered_and_replaces_apply() {
        let tools = tool_descriptors();
        let names: Vec<_> = tools
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"sail_edit"), "{names:?}");
        assert!(!names.contains(&"sail_apply"), "sail_apply must be gone");

        // MCP ≡ RPC for Edit (dry-run on a known workspace file if present; else just registry).
        let ctx = Ctx::from_env();
        let err = call_tool_with_ctx(
            "sail_edit",
            &json!({
                "path": "missing-tool9.sail",
                "ops": [],
                "dryRun": true
            }),
            &ctx,
        );
        // Missing file → NotFound-ish, not unknown tool.
        assert!(err.is_err(), "expected path error, got {err:?}");
        let msg = err.unwrap_err();
        assert!(
            !msg.contains("unknown tool"),
            "sail_edit must be dispatchable: {msg}"
        );
    }

    #[test]
    fn theme_tools_in_descriptors() {
        let tools = tool_descriptors();
        let names: Vec<_> = tools
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"sail_theme_write"));
        assert!(names.contains(&"sail_tokens_generate"));
    }
}

// ── stdio JSON-RPC MCP server (also used by `sail mcp`) ─────────────────────

use std::io::{self, Read, Write};

/// Distinguish true EOF from a recoverable framing error so a bad request
/// does not tear down the whole MCP session.
enum ReadOutcome {
    Message(String),
    /// Client closed stdin.
    Eof,
    /// Headers present but Content-Length missing/invalid — keep session alive.
    FramingError(String),
}

/// Run the stdio MCP server until stdin closes.
///
/// Accepts Content-Length framed messages and bare JSON objects (Grok).
/// Same entry as the `sail-mcp` package binary and `sail mcp` subcommand.
pub fn run_stdio() {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        print_stdio_help();
        return;
    }
    if std::env::args().any(|a| a == "--list-tools") {
        println!(
            "{}",
            serde_json::to_string_pretty(&tool_descriptors()).unwrap_or_default()
        );
        return;
    }

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();

    loop {
        match read_message(&mut stdin) {
            Ok(ReadOutcome::Message(payload)) => {
                let msg: Value = match serde_json::from_str(&payload) {
                    Ok(v) => v,
                    Err(e) => {
                        write_message(
                            &mut stdout,
                            &json!({
                                "jsonrpc": "2.0",
                                "id": null,
                                "error": { "code": -32700, "message": format!("parse error: {e}") }
                            }),
                        );
                        continue;
                    }
                };
                if let Some(resp) = handle_message(&msg) {
                    write_message(&mut stdout, &resp);
                }
            }
            Ok(ReadOutcome::Eof) => break,
            Ok(ReadOutcome::FramingError(msg)) => {
                write_message(
                    &mut stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": { "code": -32600, "message": msg }
                    }),
                );
            }
            Err(_) => break,
        }
    }
}

fn print_stdio_help() {
    eprintln!(
        "sail mcp — MCP server for Sail authoring (stdio JSON-RPC)\n\n\
         Tools: sail_surface, sail_check, sail_edit, sail_layout_check,\n\
                sail_theme_list/read/write/fork, sail_tokens_generate,\n\
                sail_dev_server_start/status/stop\n\
         Handlers: sail-ops (same as CLI `sail op` and Connect HTTP).\n\n\
         Usage:\n  \
           sail mcp                 # unified CLI\n  \
           sail mcp --list-tools    # print tool descriptors as JSON\n  \
           sail-mcp                 # package binary (`cargo run -p sail-mcp`)\n\n\
         Bare JSON objects accepted without trailing newline (Grok-compatible).\n"
    );
}

fn read_message(stdin: &mut impl Read) -> io::Result<ReadOutcome> {
    let first = loop {
        let mut b = [0u8; 1];
        match stdin.read(&mut b)? {
            0 => return Ok(ReadOutcome::Eof),
            _ => {
                let c = b[0];
                if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' {
                    continue;
                }
                break c;
            }
        }
    };

    if first == b'{' {
        return read_bare_json_object(stdin, first);
    }

    let mut headers = vec![first];
    let mut line_rest = Vec::new();
    let n = read_until(stdin, b'\n', &mut line_rest)?;
    if n == 0 {
        return Ok(ReadOutcome::FramingError(
            "incomplete MCP frame: EOF before headers finished".into(),
        ));
    }
    headers.extend_from_slice(&line_rest);
    loop {
        let mut line = Vec::new();
        let n = read_until(stdin, b'\n', &mut line)?;
        if n == 0 {
            return Ok(ReadOutcome::FramingError(
                "incomplete MCP frame: EOF before headers finished".into(),
            ));
        }
        let trimmed = {
            let mut t = line.clone();
            while t.last() == Some(&b'\n') || t.last() == Some(&b'\r') {
                t.pop();
            }
            t
        };
        if trimmed.is_empty() {
            break;
        }
        headers.extend_from_slice(&line);
    }

    let header_str = String::from_utf8_lossy(&headers);
    let mut content_length: Option<usize> = None;
    for h in header_str.lines() {
        let lower = h.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("content-length:") {
            content_length = rest.trim().parse().ok();
        }
    }
    let Some(len) = content_length else {
        return Ok(ReadOutcome::FramingError(
            "invalid MCP frame: missing or non-numeric Content-Length header".into(),
        ));
    };
    let mut buf = vec![0u8; len];
    match stdin.read_exact(&mut buf) {
        Ok(()) => Ok(ReadOutcome::Message(
            String::from_utf8_lossy(&buf).into_owned(),
        )),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(ReadOutcome::FramingError(
            format!("incomplete MCP body: expected {len} bytes"),
        )),
        Err(e) => Err(e),
    }
}

fn read_bare_json_object(stdin: &mut impl Read, first: u8) -> io::Result<ReadOutcome> {
    let mut buf = vec![first];
    let mut depth = 1i32;
    let mut in_string = false;
    let mut escape = false;
    let mut byte = [0u8; 1];
    while depth > 0 {
        match stdin.read(&mut byte)? {
            0 => {
                return Ok(if buf.len() == 1 {
                    ReadOutcome::Eof
                } else {
                    ReadOutcome::FramingError(
                        "incomplete bare JSON: EOF before object closed".into(),
                    )
                });
            }
            _ => {
                let c = byte[0];
                buf.push(c);
                if in_string {
                    if escape {
                        escape = false;
                    } else if c == b'\\' {
                        escape = true;
                    } else if c == b'"' {
                        in_string = false;
                    }
                } else {
                    match c {
                        b'"' => in_string = true,
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(ReadOutcome::Message(
        String::from_utf8_lossy(&buf).into_owned(),
    ))
}

fn read_until(r: &mut impl Read, delim: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
    let mut n = 0usize;
    let mut byte = [0u8; 1];
    loop {
        match r.read(&mut byte)? {
            0 => return Ok(n),
            _ => {
                buf.push(byte[0]);
                n += 1;
                if byte[0] == delim {
                    return Ok(n);
                }
            }
        }
    }
}

fn handle_message(msg: &Value) -> Option<Value> {
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = msg.get("id").cloned()?;
    match method {
        "initialize" => {
            let client_proto = msg
                .pointer("/params/protocolVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("2025-06-18");
            let supported = ["2025-06-18", "2024-11-05"];
            let protocol_version = if supported.contains(&client_proto) {
                client_proto
            } else {
                "2025-06-18"
            };
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": protocol_version,
                    "capabilities": {
                        "tools": { "listChanged": false }
                    },
                    "serverInfo": {
                        "name": "sail-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            }))
        }
        "ping" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": {} })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": tool_descriptors() }
        })),
        "tools/call" => {
            let params = msg.get("params").cloned().unwrap_or(json!({}));
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            match call_tool(name, &args) {
                Ok(value) => {
                    let text =
                        serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": text }],
                            "structuredContent": value,
                            "isError": false
                        }
                    }))
                }
                Err(e) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": e }],
                        "isError": true
                    }
                })),
            }
        }
        "shutdown" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": null })),
        other => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("method not found: {other}")
            }
        })),
    }
}

fn write_message(out: &mut impl Write, value: &Value) {
    let body = serde_json::to_string(value).unwrap_or_else(|_| "{}".into());
    let _ = write!(out, "{body}\n");
    let _ = out.flush();
}
