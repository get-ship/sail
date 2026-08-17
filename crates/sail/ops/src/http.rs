//! Connect unary JSON router (axum): `POST /sail.ops.v1.<Service>/<Method>`.
//! Also: `GET /sail.ops.v1.health`, `GET /sail.ops.v1.schema`, `GET /sail.ops.v1.watch` (SSE),
//! `GET /sail.ops.v1.modules`, module activity surfaces under `/_studio/x/{id}`.

use std::sync::Arc;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde_json::{Value, json};

use crate::contracts::METHOD_CONTRACTS;
use crate::ctx::Ctx;
use crate::module;
use crate::typed::schema_catalog;
use crate::watch;
use crate::{call_rpc, health_json, parse_connect_path};

/// Shared server state for Connect handlers.
#[derive(Clone)]
pub struct OpsState {
    pub ctx: Arc<Ctx>,
}

/// Mount all Connect unary routes + health + schema + watch for the default env ctx.
pub fn connect_router() -> Router {
    connect_router_with_ctx(Ctx::from_env())
}

/// Mount Connect unary routes bound to an explicit [`Ctx`] (e.g. sail dev).
pub fn connect_router_with_ctx(ctx: Ctx) -> Router {
    module::ensure_bootstrapped(&ctx.workspace_root, &ctx.project_root);
    let state = OpsState { ctx: Arc::new(ctx) };
    // Explicit routes for every known method (axum path matching is exact).
    let mut router = Router::new()
        .route("/sail.ops.v1.health", get(health_handler))
        .route("/sail.ops.v1.schema", get(schema_handler))
        .route("/sail.ops.v1.modules", get(modules_handler))
        .route("/sail.ops.v1.watch", get(watch::watch_sse))
        .route("/sail.ops.v1.SailAuthoring/Surface", post(unary))
        .route("/sail.ops.v1.SailAuthoring/Check", post(unary))
        .route("/sail.ops.v1.SailAuthoring/Hover", post(unary))
        .route("/sail.ops.v1.SailAuthoring/Complete", post(unary))
        .route("/sail.ops.v1.SailAuthoring/Edit", post(unary))
        .route("/sail.ops.v1.SailAuthoring/LayoutCheck", post(unary))
        .route("/sail.ops.v1.SailAuthoring/ListFiles", post(unary))
        .route("/sail.ops.v1.SailAuthoring/Policy", post(unary))
        .route("/sail.ops.v1.SailInspect/Inspect", post(unary))
        .route("/sail.ops.v1.SailInspect/Model", post(unary))
        .route("/sail.ops.v1.SailInspect/Tree", post(unary))
        .route("/sail.ops.v1.SailInspect/State", post(unary))
        .route("/sail.ops.v1.SailInspect/TokenUsage", post(unary))
        .route("/sail.ops.v1.SailTheme/Graph", post(unary))
        .route("/sail.ops.v1.SailTheme/List", post(unary))
        .route("/sail.ops.v1.SailTheme/Read", post(unary))
        .route("/sail.ops.v1.SailTheme/Write", post(unary))
        .route("/sail.ops.v1.SailTheme/Delete", post(unary))
        .route("/sail.ops.v1.SailTheme/Fork", post(unary))
        .route("/sail.ops.v1.SailTheme/Generate", post(unary))
        .route("/sail.ops.v1.SailTheme/EffectiveIds", post(unary))
        .route("/sail.ops.v1.SailTheme/DeriveScale", post(unary))
        .route("/sail.ops.v1.SailTheme/CheckTheme", post(unary))
        .route("/sail.ops.v1.SailTheme/Registry", post(unary))
        .route("/sail.ops.v1.SailTheme/FontSearch", post(unary))
        .route("/sail.ops.v1.SailTheme/FontAdd", post(unary))
        .route("/sail.ops.v1.SailDevServer/Start", post(unary))
        .route("/sail.ops.v1.SailDevServer/Status", post(unary))
        .route("/sail.ops.v1.SailDevServer/Stop", post(unary))
        .route("/sail.ops.v1.SailTask/List", post(unary))
        .route("/sail.ops.v1.SailTask/Get", post(unary))
        .route("/sail.ops.v1.SailTask/Create", post(unary))
        .route("/sail.ops.v1.SailTask/Update", post(unary))
        .route("/sail.ops.v1.SailTask/Comment", post(unary))
        .route("/sail.ops.v1.SailTask/AddArtifact", post(unary))
        .route("/sail.ops.v1.SailTask/AgentNote", post(unary))
        .route("/sail.ops.v1.SailTask/ThreadCreate", post(unary))
        .route("/sail.ops.v1.SailTask/ThreadReply", post(unary))
        .route("/sail.ops.v1.SailTask/ThreadList", post(unary))
        .route("/sail.ops.v1.SailTask/Reanchor", post(unary))
        .route("/sail.ops.v1.SailTask/CommitIntent", post(unary))
        .route("/sail.ops.v1.SailRender/Screenshot", post(unary))
        // Module activity surfaces (additive rail mounts).
        .route("/_studio/x/{activity_id}", get(module_surface_handler));

    // Mount each loaded module Connect path (dynamic extension; no core codegen).
    for path in module::module_connect_paths() {
        router = router.route(&path, post(unary));
    }

    router.with_state(state)
}

async fn health_handler(State(state): State<OpsState>) -> Response {
    json_response(StatusCode::OK, health_json(&state.ctx))
}

async fn modules_handler(State(state): State<OpsState>) -> Response {
    module::ensure_bootstrapped(&state.ctx.workspace_root, &state.ctx.project_root);
    json_response(StatusCode::OK, module::modules_json())
}

async fn schema_handler(State(state): State<OpsState>) -> Response {
    module::ensure_bootstrapped(&state.ctx.workspace_root, &state.ctx.project_root);
    let mut catalog = schema_catalog(METHOD_CONTRACTS);
    if let Some(methods) = catalog.get_mut("methods").and_then(|m| m.as_array_mut()) {
        methods.extend(module::module_schema_catalog_entries());
    }
    json_response(StatusCode::OK, catalog)
}

async fn module_surface_handler(Path(activity_id): Path<String>) -> Response {
    match module::activity_surface_html(&activity_id) {
        Some((m, html)) => {
            let mut res = (StatusCode::OK, html).into_response();
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            if m.dev_unsigned {
                res.headers_mut().insert(
                    header::HeaderName::from_static("x-sail-module-unsigned"),
                    HeaderValue::from_static("1"),
                );
            }
            res
        }
        None => connect_error(
            StatusCode::NOT_FOUND,
            "not_found",
            &format!("no module activity `{activity_id}`"),
        ),
    }
}

async fn unary(
    State(state): State<OpsState>,
    uri: axum::http::Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let path = uri.path();
    let Some((service, method)) = parse_connect_path(path) else {
        return connect_error(
            StatusCode::NOT_FOUND,
            "not_found",
            &format!("invalid Connect path: {path}"),
        );
    };

    // Empty body → {}
    let args: Value = if body.is_empty() {
        json!({})
    } else {
        match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                return connect_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_argument",
                    &format!("json: {e}"),
                );
            }
        }
    };

    // Prefer application/json; accept bare POST.
    let _ct = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    // Handlers are sync and may use blocking I/O (reqwest::blocking, fs). Running them
    // on a Tokio worker nests/drops a private runtime and panics ("Cannot drop a runtime
    // in a context where blocking is not allowed"). Always hop to the blocking pool.
    let result =
        tokio::task::spawn_blocking(move || call_rpc(&service, &method, &args, &state.ctx)).await;

    match result {
        Ok(Ok(value)) => json_response(StatusCode::OK, value),
        Ok(Err(e)) => connect_error(
            StatusCode::from_u16(e.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            e.connect_code(),
            &e.to_string(),
        ),
        Err(e) => connect_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            &format!("handler join: {e}"),
        ),
    }
}

fn json_response(status: StatusCode, value: Value) -> Response {
    let body = value.to_string();
    let mut res = (status, body).into_response();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    res
}

fn connect_error(status: StatusCode, code: &str, message: &str) -> Response {
    json_response(
        status,
        json!({
            "code": code,
            "message": message,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn surface_via_http_matches_call_rpc() {
        let ctx = Ctx::from_env();
        let via_rpc =
            call_rpc("SailAuthoring", "Surface", &json!({"topic": "core"}), &ctx).unwrap();

        let app = connect_router_with_ctx(ctx);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sail.ops.v1.SailAuthoring/Surface")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"topic":"core"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let via_http: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(via_http, via_rpc);
    }

    #[tokio::test]
    async fn schema_endpoint_lists_methods() {
        let app = connect_router();
        let res = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/sail.ops.v1.schema")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        let methods = v["methods"].as_array().unwrap();
        // Core contracts always present; modules may add more.
        assert!(methods.len() >= METHOD_CONTRACTS.len());
    }

    #[tokio::test]
    async fn every_contract_path_is_mounted() {
        // Routes must not 404 for known Connect paths (may 400 on empty args).
        // Build once: mounting is static; handlers run via spawn_blocking (no nested RT).
        let app = connect_router();
        for c in METHOD_CONTRACTS {
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(c.connect_path())
                        .header("content-type", "application/json")
                        .body(Body::from("{}"))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_ne!(
                res.status(),
                StatusCode::NOT_FOUND,
                "route missing for {}",
                c.connect_path()
            );
        }
    }

    #[tokio::test]
    async fn bad_json_body_is_invalid_argument() {
        let app = connect_router();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sail.ops.v1.SailAuthoring/Surface")
                    .header("content-type", "application/json")
                    .body(Body::from("{"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["code"], "invalid_argument");
    }

    /// HTTP parity helper: POST Connect path body must equal call_rpc for the same args.
    async fn assert_http_matches_rpc(service: &str, method: &str, args: Value) {
        let ctx = Ctx::from_env();
        let via_rpc = call_rpc(service, method, &args, &ctx).unwrap_or_else(|e| {
            // Some methods need real files; still require HTTP to match the same error shape.
            json!({ "code": e.connect_code(), "message": e.to_string() })
        });

        let app = connect_router_with_ctx(Ctx::from_env());
        let path = format!("/sail.ops.v1.{service}/{method}");
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&path)
                    .header("content-type", "application/json")
                    .body(Body::from(args.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let via_http: Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| json!({ "parse_error": String::from_utf8_lossy(&bytes) }));

        if via_rpc.get("code").is_some() {
            // Error path: both sides report the same Connect code (status may be 4xx).
            assert_ne!(status, StatusCode::NOT_FOUND, "route missing for {path}");
            assert_eq!(
                via_http.get("code"),
                via_rpc.get("code"),
                "{service}/{method} HTTP error code drift: http={via_http} rpc={via_rpc}"
            );
        } else {
            assert_eq!(status, StatusCode::OK, "{service}/{method} HTTP status");
            assert_eq!(via_http, via_rpc, "{service}/{method} HTTP/RPC body drift");
        }
    }

    /// SPEC-STUDIO-CONTRACT-1: Themes surface HTTP ≈ RPC.
    #[tokio::test]
    async fn themes_surface_via_http_matches_call_rpc() {
        assert_http_matches_rpc(
            "SailTheme",
            "FontSearch",
            json!({ "query": "inter", "category": "sans-serif", "limit": 5 }),
        )
        .await;
        assert_http_matches_rpc(
            "SailTheme",
            "List",
            json!({ "path": "packages/sail/tokens/model/themes" }),
        )
        .await;
        assert_http_matches_rpc("SailTheme", "EffectiveIds", json!({})).await;
        assert_http_matches_rpc(
            "SailTheme",
            "DeriveScale",
            json!({
                "anchors": { "primary": { "space": "oklch", "l": 0.54, "c": 0.18, "h": 245 } },
                "mode": "light"
            }),
        )
        .await;
        assert_http_matches_rpc("SailTheme", "Registry", json!({ "section": "catalog" })).await;
    }

    /// SPEC-STUDIO-CONTRACT-1: Tasks surface HTTP ≈ RPC.
    #[tokio::test]
    async fn tasks_surface_via_http_matches_call_rpc() {
        assert_http_matches_rpc(
            "SailTask",
            "ThreadList",
            json!({ "project": "apps/studio", "open": true }),
        )
        .await;
        assert_http_matches_rpc(
            "SailTask",
            "List",
            json!({ "project": "apps/studio", "open": true }),
        )
        .await;
    }

    /// SPEC-STUDIO-CONTRACT-1: Checks surface HTTP ≈ RPC.
    #[tokio::test]
    async fn checks_surface_via_http_matches_call_rpc() {
        assert_http_matches_rpc(
            "SailAuthoring",
            "Check",
            json!({
                "source": "<page><text>hi</text></page>\n",
                "fix": "off"
            }),
        )
        .await;
        assert_http_matches_rpc(
            "SailAuthoring",
            "LayoutCheck",
            json!({ "source": "<page><text>hi</text></page>\n" }),
        )
        .await;
    }

    /// SPEC-STUDIO-CONTRACT-1: Edit surface HTTP ≈ RPC (incl. writeSource field).
    #[tokio::test]
    async fn edit_surface_via_http_matches_call_rpc() {
        assert_http_matches_rpc(
            "SailAuthoring",
            "ListFiles",
            json!({ "project": "apps/studio" }),
        )
        .await;
        // dryRun writeSource: validate-only full-content path (no disk write).
        assert_http_matches_rpc(
            "SailAuthoring",
            "Edit",
            json!({
                "path": "apps/studio/routes/page.sail",
                "ops": [],
                "source": "<page></page>\n",
                "writeSource": true,
                "dryRun": true,
                "actor": "human:studio-contract-test"
            }),
        )
        .await;
        assert_http_matches_rpc(
            "SailInspect",
            "Model",
            json!({ "path": "apps/studio/routes/page.sail" }),
        )
        .await;
    }

    #[tokio::test]
    async fn watch_sse_delivers_file_event_within_500ms() {
        use crate::watch::{WatchKind, publish_file};
        use std::time::{Duration, Instant};
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = connect_router();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let path = format!("routes/sse-proof-{}.sail", crate::watch::current_rev());

        let stream_task = tokio::spawn(async move {
            // Retry connect briefly while server starts.
            let mut stream = None;
            for _ in 0..20 {
                if let Ok(s) = tokio::net::TcpStream::connect(addr).await {
                    stream = Some(s);
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            let stream = stream.expect("connect to ops server");
            let (reader, mut writer) = stream.into_split();
            let req = format!(
                "GET /sail.ops.v1.watch?kinds=file&pathPrefix=routes/ HTTP/1.1\r\n\
                 Host: {addr}\r\n\
                 Accept: text/event-stream\r\n\
                 Connection: close\r\n\r\n"
            );
            writer.write_all(req.as_bytes()).await.unwrap();
            let mut lines = BufReader::new(reader).lines();
            let t0 = Instant::now();
            loop {
                let line = tokio::time::timeout(Duration::from_millis(500), lines.next_line())
                    .await
                    .expect("sse timeout")
                    .expect("io")
                    .expect("eof");
                if let Some(data) = line.strip_prefix("data:") {
                    let v: Value = serde_json::from_str(data.trim()).expect("json");
                    assert_eq!(v["kind"], "file");
                    assert!(
                        v["path"].as_str().unwrap().starts_with("routes/"),
                        "path should be relative: {}",
                        v["path"]
                    );
                    assert!(t0.elapsed() < Duration::from_millis(500));
                    return v;
                }
            }
        });

        // Brief delay so SSE connects before publish.
        tokio::time::sleep(Duration::from_millis(80)).await;
        let sent = publish_file(&path);
        assert_eq!(sent.kind, WatchKind::File);

        let got = stream_task.await.unwrap();
        assert_eq!(got["path"], path);
    }
}
