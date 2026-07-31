//! TOOL-11 conformance: reference module parity + capability / signature gates.
//!
//! Global module registry is process-wide — tests take a mutex so parallel
//! cargo workers do not deadlock or clobber each other.

use sail_ops::{
    Ctx, ERR_API_UNSUPPORTED, ERR_CAPABILITY, ERR_UNSIGNED, ModuleMode, call_mcp_tool, call_rpc,
    clear_modules, connect_router_with_ctx, install_modules, load_module_dir, modules_json,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Mutex;

static REGISTRY_LOCK: Mutex<()> = Mutex::new(());

fn repo_hello() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/sail/ops → repo root
    dir.pop();
    dir.pop();
    dir.pop();
    dir.join("examples/module-hello")
}

fn with_registry<T>(f: impl FnOnce() -> T) -> T {
    let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    clear_modules();
    let out = f();
    clear_modules();
    out
}

fn load_hello_signed() {
    let m = load_module_dir(&repo_hello(), ModuleMode::Production).expect("load hello");
    install_modules(vec![m]);
}

#[test]
fn reference_module_ping_cli_rpc() {
    with_registry(|| {
        load_hello_signed();
        let ctx = Ctx::from_env();
        let v = call_rpc("ModuleHello", "Ping", &json!({}), &ctx).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["message"], "pong");
    });
}

#[test]
fn reference_module_ping_mcp_parity() {
    with_registry(|| {
        load_hello_signed();
        let ctx = Ctx::from_env();
        let via_rpc = call_rpc("ModuleHello", "Ping", &json!({}), &ctx).unwrap();
        let via_mcp = call_mcp_tool("module_hello_ping", &json!({}), &ctx).unwrap();
        assert_eq!(via_rpc, via_mcp);
    });
}

#[tokio::test]
async fn reference_module_ping_http_parity() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let (via_rpc, via_http) = {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        clear_modules();
        load_hello_signed();
        let ctx = Ctx::from_env();
        let via_rpc = call_rpc("ModuleHello", "Ping", &json!({}), &ctx).unwrap();

        let app = connect_router_with_ctx(ctx);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sail.ops.v1.ModuleHello/Ping")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let via_http: Value = serde_json::from_slice(&bytes).unwrap();
        clear_modules();
        (via_rpc, via_http)
    };
    assert_eq!(via_http, via_rpc);
}

#[tokio::test]
async fn module_activity_surface_served() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let html = {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        clear_modules();
        load_hello_signed();
        let ctx = Ctx::from_env();
        let app = connect_router_with_ctx(ctx);
        let res = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/_studio/x/hello")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        clear_modules();
        html
    };
    assert!(html.contains("data-module-surface=\"hello\""), "{html}");
}

#[test]
fn modules_json_lists_hello_activity() {
    with_registry(|| {
        load_hello_signed();
        let v = modules_json();
        let acts = v["activities"].as_array().unwrap();
        assert!(acts.iter().any(|a| a["id"] == "hello"), "{v}");
    });
}

#[test]
fn unsigned_refused_outside_dev_named_error() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("sail-module.json"),
        r#"{
          "name": "unsigned-mod",
          "version": "0.0.1",
          "api": 1,
          "provides": {
            "ops": { "namespace": "UnsignedMod", "methods": [{ "name": "X" }] }
          },
          "capabilities": { "fs": [], "net": [] },
          "backend": { "type": "static", "methods": { "X": { "ok": true } } }
        }"#,
    )
    .unwrap();
    let err = load_module_dir(root, ModuleMode::Production).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains(ERR_UNSIGNED), "{msg}");
}

#[test]
fn capability_less_fs_write_denied() {
    with_registry(|| {
        load_hello_signed();
        let mods = sail_ops::loaded_modules();
        let m = mods
            .iter()
            .find(|m| m.manifest.name == "module-hello")
            .unwrap();
        let err = m.check_fs_write(".sail/secret").unwrap_err();
        assert!(err.to_string().contains(ERR_CAPABILITY), "{err}");
        // Call-edge auto-enforce (not only helpers).
        let err = m
            .call(
                "Ping",
                &serde_json::json!({"path": ".sail/secret", "write": true}),
            )
            .unwrap_err();
        assert!(err.to_string().contains(ERR_CAPABILITY), "{err}");
        // Empty args still succeed (no power intent).
        m.call("Ping", &serde_json::json!({})).unwrap();
    });
}

#[test]
fn loader_rejects_api_gt_1() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("sail-module.json"),
        r#"{
          "name": "future",
          "version": "9.0.0",
          "api": 99,
          "provides": {},
          "signature": "x",
          "backend": { "type": "static", "methods": {} }
        }"#,
    )
    .unwrap();
    let err = load_module_dir(root, ModuleMode::Dev).unwrap_err();
    assert!(err.to_string().contains(ERR_API_UNSUPPORTED), "{err}");
}

#[test]
fn module_ts_extension_without_core_codegen() {
    with_registry(|| {
        load_hello_signed();
        let mods = sail_ops::loaded_modules();
        let ts = sail_ops::generate_module_ts_extension(&mods);
        assert!(ts.contains("moduleHelloPing"));
        assert!(ts.contains("callRpc('ModuleHello', 'Ping'"));
        // Core METHOD_CONTRACTS must not gain ModuleHello
        assert!(
            !sail_ops::METHOD_CONTRACTS
                .iter()
                .any(|c| c.service == "ModuleHello")
        );
    });
}
