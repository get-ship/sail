//! MCP 2026-07-28 **stateless** protocol core.
//!
//! No sessions, no required `initialize` handshake. Each request is self-contained:
//! protocol version + client capabilities ride in `params._meta`. Tool list is a
//! generated projection of `sail_ops::METHOD_CONTRACTS` (see `mcp_tool_descriptors`).

use sail_ops::{Ctx, call_mcp_tool, mcp_tool_descriptors};
use serde_json::{Value, json};

/// Primary (modern) protocol version — SPEC-TOOLING-1 §WI9 / MCP 2026-07-28.
pub const PROTOCOL_VERSION: &str = "2026-07-28";

/// Versions this server accepts on modern (per-request `_meta`) requests.
pub const SUPPORTED_VERSIONS: &[&str] = &[PROTOCOL_VERSION];

/// Legacy handshake revisions answered by dual-era `initialize` (no session mint).
const LEGACY_INITIALIZE_VERSIONS: &[&str] =
    &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const META_PROTOCOL_VERSION: &str = "io.modelcontextprotocol/protocolVersion";
const META_CLIENT_INFO: &str = "io.modelcontextprotocol/clientInfo";
const META_CLIENT_CAPS: &str = "io.modelcontextprotocol/clientCapabilities";
const META_SERVER_INFO: &str = "io.modelcontextprotocol/serverInfo";

/// Tool descriptors for MCP `tools/list` (live: core contracts + modules).
pub fn tool_descriptors() -> Value {
    mcp_tool_descriptors()
}

/// In-process `sail mcp --list-tools` — same JSON array the CLI prints.
///
/// Fails when the generated list is missing or empty. Does not read host
/// MCP config and does not require a `sail` binary on PATH.
pub fn list_tools() -> Result<Vec<Value>, String> {
    let tools = tool_descriptors();
    let arr = tools
        .as_array()
        .ok_or_else(|| "list-tools: expected a JSON array".to_string())?;
    if arr.is_empty() {
        return Err("list-tools: empty".into());
    }
    Ok(arr.clone())
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

fn server_info() -> Value {
    json!({
        "name": "sail-mcp",
        "version": env!("CARGO_PKG_VERSION"),
    })
}

fn server_meta() -> Value {
    json!({ META_SERVER_INFO: server_info() })
}

fn server_capabilities() -> Value {
    json!({
        "tools": { "listChanged": false }
    })
}

/// Extract `_meta` object from request params (or empty object).
fn params_meta(msg: &Value) -> &Value {
    msg.pointer("/params/_meta")
        .or_else(|| msg.get("_meta"))
        .unwrap_or(&Value::Null)
}

/// Protocol version declared on this request (body `_meta` first).
pub fn request_protocol_version(msg: &Value) -> Option<&str> {
    params_meta(msg)
        .get(META_PROTOCOL_VERSION)
        .and_then(|v| v.as_str())
}

/// Build a standard JSON-RPC error response.
pub fn jsonrpc_error(
    id: Value,
    code: i64,
    message: impl Into<String>,
    data: Option<Value>,
) -> Value {
    let mut err = json!({
        "code": code,
        "message": message.into(),
    });
    if let Some(d) = data {
        err.as_object_mut().unwrap().insert("data".into(), d);
    }
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": err,
    })
}

fn unsupported_version_error(id: Value, requested: &str) -> Value {
    jsonrpc_error(
        id,
        -32022,
        "Unsupported protocol version",
        Some(json!({
            "supported": SUPPORTED_VERSIONS,
            "requested": requested,
        })),
    )
}

/// Validate modern per-request `_meta` (protocol version + client capabilities).
///
/// Soft-compat: if `_meta` is entirely absent, treat as [`PROTOCOL_VERSION`] so
/// transitional hosts and unit harnesses still complete a tools round-trip.
/// When a version *is* declared, it must be in [`SUPPORTED_VERSIONS`].
fn negotiate_modern_version(msg: &Value, id: &Value) -> Result<String, Value> {
    let meta = params_meta(msg);
    let declared = meta.get(META_PROTOCOL_VERSION).and_then(|v| v.as_str());
    match declared {
        Some(v) if SUPPORTED_VERSIONS.contains(&v) => Ok(v.to_string()),
        Some(v) => Err(unsupported_version_error(id.clone(), v)),
        None => {
            // Soft-compat: no _meta → modern default (session-less path still works).
            Ok(PROTOCOL_VERSION.to_string())
        }
    }
}

/// Handle one JSON-RPC message. Returns `None` for notifications (no response).
pub fn handle_message(msg: &Value) -> Option<Value> {
    handle_message_with_ctx(msg, &Ctx::from_env())
}

/// Same as [`handle_message`] with an explicit ops context.
pub fn handle_message_with_ctx(msg: &Value, ctx: &Ctx) -> Option<Value> {
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    // Notifications have no id → no response.
    let id = match msg.get("id") {
        Some(id) if !id.is_null() => id.clone(),
        Some(_) | None if method.starts_with("notifications/") => return None,
        None => {
            // Malformed request without id — still answer with error (id null).
            return Some(jsonrpc_error(
                Value::Null,
                -32600,
                "invalid request: missing id",
                None,
            ));
        }
        Some(id) => id.clone(),
    };

    match method {
        // ── Modern 2026-07-28 ───────────────────────────────────────────────
        "server/discover" => Some(handle_discover(msg, id)),
        "tools/list" => Some(handle_tools_list(msg, id)),
        "tools/call" => Some(handle_tools_call(msg, id, ctx)),
        "ping" => Some(handle_ping(msg, id)),

        // ── Dual-era legacy handshake (no session; optional for old hosts) ──
        "initialize" => Some(handle_initialize(msg, id)),
        "shutdown" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": null
        })),

        // notifications/initialized etc. — no response
        m if m.starts_with("notifications/") => None,

        other => Some(jsonrpc_error(
            id,
            -32601,
            format!("method not found: {other}"),
            None,
        )),
    }
}

fn handle_discover(msg: &Value, id: Value) -> Value {
    if let Err(e) = negotiate_modern_version(msg, &id) {
        return e;
    }
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "resultType": "complete",
            "supportedVersions": SUPPORTED_VERSIONS,
            "capabilities": server_capabilities(),
            "instructions": "Sail authoring MCP — tools are a generated projection of sail-ops METHOD_CONTRACTS (same handlers as `sail op` and Connect). Prefer tools/list then tools/call; no initialize required (2026-07-28).",
            "ttlMs": 3_600_000,
            "cacheScope": "public",
            "_meta": server_meta(),
        }
    })
}

fn handle_ping(msg: &Value, id: Value) -> Value {
    if let Err(e) = negotiate_modern_version(msg, &id) {
        return e;
    }
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "resultType": "complete",
            "_meta": server_meta(),
        }
    })
}

fn handle_tools_list(msg: &Value, id: Value) -> Value {
    if let Err(e) = negotiate_modern_version(msg, &id) {
        return e;
    }
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "resultType": "complete",
            "tools": tool_descriptors(),
            "ttlMs": 300_000,
            "cacheScope": "public",
            "_meta": server_meta(),
        }
    })
}

fn handle_tools_call(msg: &Value, id: Value, ctx: &Ctx) -> Value {
    if let Err(e) = negotiate_modern_version(msg, &id) {
        return e;
    }
    let params = msg.get("params").cloned().unwrap_or(json!({}));
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    if name.is_empty() {
        return jsonrpc_error(id, -32602, "tools/call requires params.name", None);
    }
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match call_tool_with_ctx(name, &args, ctx) {
        Ok(value) => {
            let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "resultType": "complete",
                    "content": [{ "type": "text", "text": text }],
                    "structuredContent": value,
                    "isError": false,
                    "_meta": server_meta(),
                }
            })
        }
        Err(e) => {
            // Unknown tool → protocol error (-32602). Other failures → tool execution error.
            let lower = e.to_lowercase();
            if lower.contains("unknown tool") {
                return jsonrpc_error(id, -32602, e, None);
            }
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "resultType": "complete",
                    "content": [{ "type": "text", "text": e }],
                    "isError": true,
                    "_meta": server_meta(),
                }
            })
        }
    }
}

/// Dual-era: answer legacy `initialize` without minting a session.
///
/// Modern clients should use `server/discover` instead. When a legacy client
/// opens with `initialize`, we echo a mutually supported legacy version when
/// possible; otherwise advertise modern support in the error payload.
fn handle_initialize(msg: &Value, id: Value) -> Value {
    let client_proto = msg
        .pointer("/params/protocolVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Modern client accidentally sending initialize with modern _meta — point them.
    if let Some(modern) = request_protocol_version(msg) {
        if modern == PROTOCOL_VERSION || SUPPORTED_VERSIONS.contains(&modern) {
            return jsonrpc_error(
                id,
                -32601,
                "initialize is not used on MCP 2026-07-28; call server/discover or tools/* with per-request _meta",
                Some(json!({ "supported": SUPPORTED_VERSIONS })),
            );
        }
    }

    let protocol_version = if LEGACY_INITIALIZE_VERSIONS.contains(&client_proto) {
        client_proto
    } else if client_proto.is_empty() {
        // Old clients sometimes omit; pick a widely supported legacy default.
        "2025-06-18"
    } else if client_proto == PROTOCOL_VERSION {
        // Client asked for modern via initialize field — accept modern capabilities
        // but note they should migrate to server/discover.
        PROTOCOL_VERSION
    } else {
        // Unknown — still answer with modern so dual-era hosts can continue.
        PROTOCOL_VERSION
    };

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": protocol_version,
            "capabilities": server_capabilities(),
            "serverInfo": server_info(),
            "instructions": "Sail MCP dual-era: prefer 2026-07-28 server/discover (stateless). This initialize response mints no session.",
        }
    })
}

/// Build a modern `_meta` object for tests / session-less clients.
pub fn modern_meta() -> Value {
    json!({
        META_PROTOCOL_VERSION: PROTOCOL_VERSION,
        META_CLIENT_INFO: { "name": "sail-mcp-test", "version": "0.0.0" },
        META_CLIENT_CAPS: {},
    })
}

/// JSON-RPC request helper (tests).
pub fn modern_request(id: impl Into<Value>, method: &str, params: Value) -> Value {
    let mut p = params;
    if let Some(obj) = p.as_object_mut() {
        obj.entry("_meta").or_insert_with(modern_meta);
    } else if p.is_null() {
        p = json!({ "_meta": modern_meta() });
    }
    json!({
        "jsonrpc": "2.0",
        "id": id.into(),
        "method": method,
        "params": p,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sail_ops::METHOD_CONTRACTS;

    #[test]
    fn sessionless_discover_and_tool_roundtrip() {
        let ctx = Ctx::from_env();
        // No initialize — first message is server/discover.
        let disc = handle_message_with_ctx(&modern_request(1, "server/discover", json!({})), &ctx)
            .unwrap();
        assert!(disc.get("error").is_none(), "{disc}");
        let result = &disc["result"];
        assert_eq!(result["resultType"], "complete");
        assert_eq!(result["supportedVersions"][0], PROTOCOL_VERSION);
        assert!(result["capabilities"]["tools"].is_object());

        // tools/list without prior handshake.
        let list =
            handle_message_with_ctx(&modern_request(2, "tools/list", json!({})), &ctx).unwrap();
        assert!(list.get("error").is_none(), "{list}");
        let tools = list["result"]["tools"].as_array().unwrap();
        assert!(
            tools.len() >= METHOD_CONTRACTS.len(),
            "expected >= {} tools, got {}",
            METHOD_CONTRACTS.len(),
            tools.len()
        );
        let names: Vec<_> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"sail_surface"));
        assert!(names.contains(&"sail_check"));
        assert!(names.contains(&"sail_edit"));
        assert!(names.contains(&"sail_inspect"));
        assert!(names.contains(&"sail_render"), "WI4 render tool: {names:?}");

        // tools/call session-less.
        let call = handle_message_with_ctx(
            &modern_request(
                3,
                "tools/call",
                json!({
                    "name": "sail_surface",
                    "arguments": { "topic": "core" }
                }),
            ),
            &ctx,
        )
        .unwrap();
        assert!(call.get("error").is_none(), "{call}");
        assert_eq!(call["result"]["resultType"], "complete");
        assert_eq!(call["result"]["isError"], false);
        let structured = &call["result"]["structuredContent"];
        assert_eq!(structured["topic"], "core");
        assert!(
            structured["markdown"]
                .as_str()
                .unwrap()
                .contains("Hard rules")
        );
    }

    #[test]
    fn unsupported_protocol_version_lists_supported() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "tools/list",
            "params": {
                "_meta": {
                    "io.modelcontextprotocol/protocolVersion": "1900-01-01",
                    "io.modelcontextprotocol/clientCapabilities": {}
                }
            }
        });
        let resp = handle_message(&msg).unwrap();
        assert_eq!(resp["error"]["code"], -32022);
        assert_eq!(resp["error"]["data"]["requested"], "1900-01-01");
        assert_eq!(resp["error"]["data"]["supported"][0], PROTOCOL_VERSION);
    }

    #[test]
    fn list_tools_matches_method_contracts() {
        let tools = list_tools().expect("in-process list-tools");
        assert_eq!(tools.len(), METHOD_CONTRACTS.len());
    }

    #[test]
    fn mcp_tools_equiv_contract_methods() {
        // Drift: every METHOD_CONTRACT appears exactly once in generated descriptors.
        let tools = tool_descriptors();
        let arr = tools.as_array().unwrap();
        let names: Vec<&str> = arr
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        for c in METHOD_CONTRACTS {
            assert!(
                names.contains(&c.mcp_tool),
                "contract {}/{} mcp_tool `{}` missing from tools/list",
                c.service,
                c.method,
                c.mcp_tool
            );
        }
        // Core projection length: live list may include modules (≥ contracts).
        assert!(arr.len() >= METHOD_CONTRACTS.len());
        // Generated path: descriptors == mcp_descriptors_from(contracts) for core names.
        let core = sail_ops::mcp_descriptors_from(METHOD_CONTRACTS);
        let core_names: Vec<_> = core
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert_eq!(core_names.len(), METHOD_CONTRACTS.len());
        for c in METHOD_CONTRACTS {
            assert!(core_names.contains(&c.mcp_tool));
        }
    }

    #[test]
    fn legacy_initialize_still_answers_without_session() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "legacy", "version": "0" }
            }
        });
        let resp = handle_message(&msg).unwrap();
        assert!(resp.get("error").is_none(), "{resp}");
        assert_eq!(resp["result"]["protocolVersion"], "2025-06-18");
        // No session id field.
        assert!(resp["result"].get("sessionId").is_none());
        assert!(resp["result"]["serverInfo"]["name"] == "sail-mcp");
    }

    #[test]
    fn notification_yields_no_response() {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": "notifications/cancelled",
            "params": { "requestId": 1 }
        });
        assert!(handle_message(&msg).is_none());
    }

    fn fixture_workspace() -> (std::path::PathBuf, Ctx) {
        let fixtures = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ops/tests/fixtures")
            .canonicalize()
            .expect("crates/sail/ops/tests/fixtures");
        assert!(
            fixtures.join("mcp-smoke.sail").is_file(),
            "missing mcp-smoke.sail under {}",
            fixtures.display()
        );
        let ctx = Ctx::new(&fixtures, &fixtures, &fixtures);
        (fixtures, ctx)
    }

    fn line_col_of(src: &str, needle: &str) -> (u32, u32) {
        let offset = src.find(needle).expect(needle);
        let mut line = 1u32;
        let mut col = 1u32;
        for (i, ch) in src.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn tool_call(id: i64, name: &str, arguments: Value, ctx: &Ctx) -> Value {
        let resp = handle_message_with_ctx(
            &modern_request(
                id,
                "tools/call",
                json!({ "name": name, "arguments": arguments }),
            ),
            ctx,
        )
        .expect("tools/call response");
        assert!(resp.get("error").is_none(), "{name} protocol error: {resp}");
        assert_eq!(resp["result"]["isError"], false, "{name} isError: {resp}");
        resp["result"]["structuredContent"].clone()
    }

    /// Stdio-equivalent smoke: tools/list length == METHOD_CONTRACTS, then
    /// sail_check / sail_hover / sail_inspect(topic=tree) on a local fixture.
    #[test]
    fn stdio_smoke_check_hover_inspect_on_fixture() {
        let (fixtures, ctx) = fixture_workspace();
        let rel = "mcp-smoke.sail";
        let src = std::fs::read_to_string(fixtures.join(rel)).expect("read fixture");

        let list =
            handle_message_with_ctx(&modern_request(1, "tools/list", json!({})), &ctx).unwrap();
        assert!(list.get("error").is_none(), "{list}");
        let tools = list["result"]["tools"].as_array().expect("tools array");
        assert_eq!(
            tools.len(),
            METHOD_CONTRACTS.len(),
            "tools/list length must equal METHOD_CONTRACTS (got {} vs {})",
            tools.len(),
            METHOD_CONTRACTS.len()
        );

        let check = tool_call(
            2,
            "sail_check",
            json!({ "path": rel, "includeTs": false }),
            &ctx,
        );
        assert!(
            check
                .get("diagnostics")
                .and_then(|d| d.as_array())
                .is_some()
        );
        assert!(check.get("count").and_then(|c| c.as_u64()).is_some());

        let (line, column) = line_col_of(&src, "h1");
        let hover = tool_call(
            3,
            "sail_hover",
            json!({
                "path": rel,
                "source": src,
                "line": line,
                "column": column,
            }),
            &ctx,
        );
        assert!(hover.get("contents").is_some(), "hover: {hover}");

        let inspect = tool_call(
            4,
            "sail_inspect",
            json!({ "topic": "tree", "path": rel }),
            &ctx,
        );
        assert_eq!(inspect["topic"], "tree", "{inspect}");
        let nodes = inspect["nodes"].as_array().expect("inspect nodes");
        assert!(!nodes.is_empty(), "inspect tree empty: {inspect}");
        let tags: Vec<&str> = nodes.iter().filter_map(|n| n["tag"].as_str()).collect();
        assert!(
            tags.iter().any(|t| *t == "h1" || *t == "main"),
            "expected h1/main in tree: {tags:?}"
        );
    }
}
