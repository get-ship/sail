//! stdio transport — newline-delimited JSON-RPC (MCP 2026-07-28).
//!
//! Also accepts Content-Length framed messages (dual-era clients).

use std::io::{self, Read, Write};

use serde_json::{Value, json};

use crate::protocol::{handle_message, tool_descriptors};

/// Distinguish true EOF from a recoverable framing error so a bad request
/// does not tear down the process (stateless: next message is independent).
enum ReadOutcome {
    Message(String),
    /// Client closed stdin.
    Eof,
    /// Headers present but Content-Length missing/invalid — keep listening.
    FramingError(String),
}

/// Run the stdio MCP server until stdin closes.
///
/// Accepts Content-Length framed messages and bare JSON objects (Grok /
/// newline-delimited). Same entry as the `sail-mcp` package binary and
/// `sail mcp` subcommand.
pub fn run_stdio() {
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

pub fn print_stdio_help() {
    eprintln!(
        "sail mcp — MCP server for Sail authoring (2026-07-28 stateless)\n\n\
         Transports:\n  \
           stdio (default)     newline-delimited JSON-RPC\n  \
           --http [ADDR]       Streamable HTTP single endpoint (default 127.0.0.1:3920)\n\n\
         Tools: generated from sail-ops METHOD_CONTRACTS (sail_surface, sail_check,\n\
                sail_edit, sail_inspect, sail_render/screenshot, theme/task/dev, …).\n\
         Handlers: sail-ops (same as CLI `sail op` and Connect HTTP).\n\n\
         Protocol: no initialize required — send server/discover or tools/* with\n\
         per-request _meta.io.modelcontextprotocol/protocolVersion = \"2026-07-28\".\n\n\
         Usage:\n  \
           sail mcp                 # stdio\n  \
           sail mcp --http          # 127.0.0.1:3920 POST /mcp\n  \
           sail mcp --http :3921    # custom bind\n  \
           sail mcp --list-tools    # print tool descriptors as JSON\n\n\
         Bare JSON objects accepted without trailing newline.\n"
    );
}

/// Print tool descriptors (for `--list-tools`).
pub fn print_list_tools() {
    println!(
        "{}",
        serde_json::to_string_pretty(&tool_descriptors()).unwrap_or_default()
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

/// Write one newline-delimited JSON-RPC message (2026-07-28 stdio framing).
fn write_message(out: &mut impl Write, value: &Value) {
    let body = serde_json::to_string(value).unwrap_or_else(|_| "{}".into());
    let _ = write!(out, "{body}\n");
    let _ = out.flush();
}
