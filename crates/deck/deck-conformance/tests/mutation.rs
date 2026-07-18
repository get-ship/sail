//! Slice 8 — mutation vocabulary (no browser, ship-free).
//!
//! The progressive-enhancement form endpoint (`POST /_deck/mutation/{name}`)
//! retired framework-wide: deck's router answers it with a 410 tombstone and
//! JS-attached mutations call the SDK directly. What this file still gates:
//!
//! 1. The wire vocabulary contract — `routes/todos/page.sail` carries exactly
//!    one mutation binding pointing at `ship.demo.addTodo`, parsed via
//!    `sail_compiler::{parse, bindings}` (the same path a host uses).
//! 2. Resolver semantics — `demo.addTodo` then `demo.todos` against the store.
//! 3. The tombstone itself — a form POST receives 410 Gone, not a 404 (deck
//!    still composes this URL as the no-JS `<form>` action).
//!
//! Run: `cargo test -p deck-conformance --test mutation`

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use deck_conformance::data_port::{DemoStore, resolve_demo};
use deck_conformance::{deck_app, default_app_dir};
use serde_json::json;
use sail_compiler::{bindings, keys::BindingKind, parse};
use tower::ServiceExt;

/// The PE form endpoint is a deliberate 410 tombstone in deck's router —
/// the URL is still composed as the no-JS `<form>` action, so it must answer
/// Gone (the surface existed and was retired), never 404.
#[tokio::test(flavor = "multi_thread")]
async fn mutation_post_is_tombstoned_410() {
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_deck/mutation/todos?form=b0")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::REFERER, "http://localhost:7878/_deck/view/todos")
                .body(Body::from("text=hi"))
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(
        resp.status(),
        StatusCode::GONE,
        "PE form mutations are retired; the tombstone must answer 410"
    );
}

/// After a successful mutation, the store contains the new item.
/// This test drives the store directly (no HTTP round-trip) to verify
/// the resolver semantics independently of the HTTP layer.
#[tokio::test]
async fn resolve_demo_shows_added_todo_after_add() {
    let store = DemoStore::new();

    // Add via resolver.
    resolve_demo("demo.addTodo", &json!({"text": "hi"}), &store)
        .await
        .unwrap();

    // List should show the item.
    let v = resolve_demo("demo.todos", &json!({}), &store)
        .await
        .unwrap();
    let items = v["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "one item after addTodo");
    assert_eq!(items[0]["text"], "hi");
}

/// The todos view contains exactly one mutation binding pointing at
/// `ship.demo.addTodo`.  This proves the wire vocabulary contract is in place.
#[test]
fn todos_view_has_mutation_binding_for_add_todo() {
    let app_dir = default_app_dir();
    let source =
        std::fs::read_to_string(app_dir.join("routes/todos/page.sail")).expect("todos page.sail");
    // todos/page.sail carries its controller INLINE (`<controller>…</controller>`)
    // — strip it before the template-only parser sees it (mirrors deck-ssr's
    // build path).
    let (stripped, _inline_controller) =
        sail_compiler::parse::preprocess::extract_controller_block(&source);
    let program = parse::parse(&stripped).expect("parse");
    let reg = bindings::build(&program);
    let mutations: Vec<_> = reg
        .iter()
        .filter(|b| matches!(b.kind, BindingKind::Mutation))
        .collect();
    assert_eq!(mutations.len(), 1, "todos view must have exactly one mutation");
    assert_eq!(
        mutations[0].expr, "ship.demo.addTodo",
        "mutation must point at ship.demo.addTodo; got: {}",
        mutations[0].expr
    );
}

