//! Streamable HTTP transport — single MCP endpoint (MCP 2026-07-28).
//!
//! `POST /mcp` (configurable path). No sessions, no GET stream, no
//! `Mcp-Session-Id`. Each request is independent; replies are
//! `application/json` (tools are sync; SSE not required for this server).

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use sail_ops::Ctx;
use serde_json::{Value, json};
use tokio::net::TcpListener;

use crate::protocol::{
    PROTOCOL_VERSION, SUPPORTED_VERSIONS, handle_message_with_ctx, jsonrpc_error,
    request_protocol_version,
};

const DEFAULT_BIND: &str = "127.0.0.1:3920";
const DEFAULT_PATH: &str = "/mcp";

/// Shared HTTP server state.
#[derive(Clone)]
struct McpHttpState {
    ctx: Arc<Ctx>,
    path: String,
}

/// Parse bind address from CLI (`:3920`, `127.0.0.1:3920`, bare port).
pub fn parse_bind(spec: Option<&str>) -> Result<SocketAddr, String> {
    let raw = spec.unwrap_or(DEFAULT_BIND).trim();
    let normalized = if raw.is_empty() {
        DEFAULT_BIND.to_string()
    } else if raw.starts_with(':') {
        format!("127.0.0.1{raw}")
    } else if raw.chars().all(|c| c.is_ascii_digit()) {
        format!("127.0.0.1:{raw}")
    } else {
        raw.to_string()
    };
    normalized
        .parse::<SocketAddr>()
        .map_err(|e| format!("invalid --http bind `{raw}`: {e}"))
}

/// Build the Streamable HTTP router (single POST endpoint + 405 for GET/DELETE).
pub fn mcp_http_router(ctx: Ctx, path: &str) -> Router {
    let path = if path.is_empty() {
        DEFAULT_PATH.to_string()
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let state = McpHttpState {
        ctx: Arc::new(ctx),
        path: path.clone(),
    };
    Router::new()
        .route(
            &path,
            post(mcp_post)
                .get(method_not_allowed)
                .delete(method_not_allowed),
        )
        // Health-ish probe for operators (not part of MCP wire).
        .route("/healthz", get(healthz))
        .with_state(state)
}

/// Serve Streamable HTTP until the process is killed.
pub async fn run_http(bind: SocketAddr, ctx: Ctx) -> Result<(), String> {
    let app = mcp_http_router(ctx, DEFAULT_PATH);
    let listener = TcpListener::bind(bind)
        .await
        .map_err(|e| format!("bind {bind}: {e}"))?;
    eprintln!(
        "sail-mcp Streamable HTTP listening on http://{bind}{DEFAULT_PATH} (MCP {PROTOCOL_VERSION} stateless)"
    );
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("http serve: {e}"))
}

/// Blocking entry for CLI / binary (`tokio` runtime).
pub fn run_http_blocking(bind: SocketAddr) -> Result<(), String> {
    let ctx = Ctx::from_env();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("tokio runtime: {e}"))?;
    rt.block_on(run_http(bind, ctx))
}

async fn healthz(State(state): State<McpHttpState>) -> Response {
    json_response(
        StatusCode::OK,
        json!({
            "ok": true,
            "protocol": PROTOCOL_VERSION,
            "path": state.path,
            "supportedVersions": SUPPORTED_VERSIONS,
        }),
    )
}

async fn method_not_allowed() -> Response {
    // 2026-07-28: GET/DELETE on MCP endpoint → 405 (no session stream / no session delete).
    let mut res = (
        StatusCode::METHOD_NOT_ALLOWED,
        "Method Not Allowed — MCP 2026-07-28 uses POST only on this endpoint\n",
    )
        .into_response();
    res.headers_mut()
        .insert(header::ALLOW, HeaderValue::from_static("POST"));
    res
}

async fn mcp_post(State(state): State<McpHttpState>, headers: HeaderMap, body: Bytes) -> Response {
    // Origin check (DNS rebinding). Missing Origin is OK for non-browser clients.
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        if !origin_allowed(origin) {
            return json_response(
                StatusCode::FORBIDDEN,
                json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32600,
                        "message": format!("forbidden Origin: {origin}")
                    }
                }),
            );
        }
    }

    // Ignore prior session / resume headers (do not mint or echo).
    let _ = headers.get("mcp-session-id");
    let _ = headers.get("last-event-id");

    let msg: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return json_response(
                StatusCode::BAD_REQUEST,
                json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("parse error: {e}") }
                }),
            );
        }
    };

    // JSON-RPC notification: method present, id absent → 202 Accepted, no body.
    if msg.get("method").is_some() && msg.get("id").is_none() {
        return StatusCode::ACCEPTED.into_response();
    }

    // Header validation (Streamable HTTP routing headers).
    if let Some(err) = validate_headers(&headers, &msg) {
        return err;
    }

    let Some(resp) = handle_message_with_ctx(&msg, &state.ctx) else {
        // Should not happen for requests with id.
        return StatusCode::ACCEPTED.into_response();
    };

    let status = if resp.get("error").is_some() {
        let code = resp
            .pointer("/error/code")
            .and_then(|c| c.as_i64())
            .unwrap_or(0);
        match code {
            -32601 => StatusCode::NOT_FOUND, // method not found
            -32022 | -32021 | -32020 | -32602 | -32600 | -32700 => StatusCode::BAD_REQUEST,
            _ => StatusCode::BAD_REQUEST,
        }
    } else {
        StatusCode::OK
    };

    json_response(status, resp)
}

fn origin_allowed(origin: &str) -> bool {
    // Local tooling only by default: empty, null, localhost, 127.0.0.1, file://.
    if origin == "null" {
        return true;
    }
    let lower = origin.to_ascii_lowercase();
    lower.starts_with("http://127.0.0.1")
        || lower.starts_with("http://localhost")
        || lower.starts_with("https://127.0.0.1")
        || lower.starts_with("https://localhost")
        || lower.starts_with("file:")
}

fn header_ci<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a HeaderValue> {
    // HeaderMap is case-insensitive for standard lookups.
    headers.get(HeaderName::from_bytes(name.as_bytes()).ok()?)
}

/// Validate MCP-Protocol-Version / Mcp-Method / Mcp-Name against the body.
fn validate_headers(headers: &HeaderMap, msg: &Value) -> Option<Response> {
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");

    let proto_hdr = header_ci(headers, "MCP-Protocol-Version")
        .or_else(|| header_ci(headers, "mcp-protocol-version"))
        .and_then(|v| v.to_str().ok());

    // Body _meta version (source of truth when present).
    let body_ver = request_protocol_version(msg);

    match (proto_hdr, body_ver) {
        (None, _) => {
            // Require header on HTTP for modern compliance.
            return Some(json_response(
                StatusCode::BAD_REQUEST,
                jsonrpc_error(
                    id,
                    -32020,
                    "Header mismatch: missing MCP-Protocol-Version",
                    None,
                ),
            ));
        }
        (Some(h), Some(b)) if h != b => {
            return Some(json_response(
                StatusCode::BAD_REQUEST,
                jsonrpc_error(
                    id,
                    -32020,
                    format!(
                        "Header mismatch: MCP-Protocol-Version header value '{h}' does not match body value '{b}'"
                    ),
                    None,
                ),
            ));
        }
        (Some(h), _) if !SUPPORTED_VERSIONS.contains(&h) => {
            return Some(json_response(
                StatusCode::BAD_REQUEST,
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32022,
                        "message": "Unsupported protocol version",
                        "data": {
                            "supported": SUPPORTED_VERSIONS,
                            "requested": h,
                        }
                    }
                }),
            ));
        }
        _ => {}
    }

    let method_hdr = header_ci(headers, "Mcp-Method")
        .or_else(|| header_ci(headers, "mcp-method"))
        .and_then(|v| v.to_str().ok());
    match method_hdr {
        None => {
            return Some(json_response(
                StatusCode::BAD_REQUEST,
                jsonrpc_error(id, -32020, "Header mismatch: missing Mcp-Method", None),
            ));
        }
        Some(h) if h != method => {
            return Some(json_response(
                StatusCode::BAD_REQUEST,
                jsonrpc_error(
                    id,
                    -32020,
                    format!(
                        "Header mismatch: Mcp-Method header value '{h}' does not match body value '{method}'"
                    ),
                    None,
                ),
            ));
        }
        _ => {}
    }

    // Mcp-Name required for tools/call (and resources/read, prompts/get — N/A here).
    if method == "tools/call" {
        let body_name = msg
            .pointer("/params/name")
            .and_then(|n| n.as_str())
            .unwrap_or("");
        let name_hdr = header_ci(headers, "Mcp-Name")
            .or_else(|| header_ci(headers, "mcp-name"))
            .and_then(|v| v.to_str().ok())
            .map(decode_mcp_name);
        match name_hdr {
            None => {
                return Some(json_response(
                    StatusCode::BAD_REQUEST,
                    jsonrpc_error(id, -32020, "Header mismatch: missing Mcp-Name", None),
                ));
            }
            Some(h) if h != body_name => {
                return Some(json_response(
                    StatusCode::BAD_REQUEST,
                    jsonrpc_error(
                        id,
                        -32020,
                        format!(
                            "Header mismatch: Mcp-Name header value '{h}' does not match body value '{body_name}'"
                        ),
                        None,
                    ),
                ));
            }
            _ => {}
        }
    }

    None
}

/// Decode `=?base64?…?=` sentinel if present; otherwise return as-is.
fn decode_mcp_name(raw: &str) -> String {
    const PRE: &str = "=?base64?";
    const SUF: &str = "?=";
    if let Some(inner) = raw.strip_prefix(PRE).and_then(|s| s.strip_suffix(SUF)) {
        use base64::Engine;
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(inner) {
            if let Ok(s) = String::from_utf8(bytes) {
                return s;
            }
        }
    }
    raw.to_string()
}

fn json_response(status: StatusCode, value: Value) -> Response {
    let body = serde_json::to_vec(&value).unwrap_or_else(|_| b"{}".to_vec());
    let mut res = (status, body).into_response();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::modern_meta;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> Router {
        mcp_http_router(Ctx::from_env(), "/mcp")
    }

    async fn post_json(headers: &[(&str, &str)], body: Value) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(Method::POST).uri("/mcp");
        for (k, v) in headers {
            builder = builder.header(*k, *v);
        }
        let req = builder
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let res = app().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let v: Value = serde_json::from_slice(&bytes).unwrap_or(json!(null));
        (status, v)
    }

    #[tokio::test]
    async fn sessionless_http_tool_roundtrip() {
        let discover = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "server/discover",
            "params": { "_meta": modern_meta() }
        });
        let (st, body) = post_json(
            &[
                ("MCP-Protocol-Version", PROTOCOL_VERSION),
                ("Mcp-Method", "server/discover"),
            ],
            discover,
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{body}");
        assert_eq!(body["result"]["resultType"], "complete");
        assert_eq!(body["result"]["supportedVersions"][0], PROTOCOL_VERSION);

        let call = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "sail_surface",
                "arguments": { "topic": "core" },
                "_meta": modern_meta()
            }
        });
        let (st, body) = post_json(
            &[
                ("MCP-Protocol-Version", PROTOCOL_VERSION),
                ("Mcp-Method", "tools/call"),
                ("Mcp-Name", "sail_surface"),
            ],
            call,
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{body}");
        assert_eq!(body["result"]["isError"], false);
        assert_eq!(body["result"]["structuredContent"]["topic"], "core");
    }

    #[tokio::test]
    async fn get_mcp_returns_405() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/mcp")
            .body(Body::empty())
            .unwrap();
        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn missing_protocol_header_is_header_mismatch() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": { "_meta": modern_meta() }
        });
        let (st, body) = post_json(&[("Mcp-Method", "tools/list")], msg).await;
        assert_eq!(st, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], -32020);
    }
}
