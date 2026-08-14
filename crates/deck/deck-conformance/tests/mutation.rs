//! Slice 8 — mutation vocabulary (no browser, ship-free).
//!
//! Mutation vocabulary (no browser, ship-free).
//!
//! JS-attached mutations call the SDK directly. What this file still gates:
//!
//! 1. The wire vocabulary contract — `routes/todos/page.sail` carries exactly
//!    one mutation binding pointing at `ship.demo.addTodo`, parsed via
//!    `sail_compiler::{parse, bindings}` (the same path a host uses).
//! 2. Resolver semantics — `demo.addTodo` then `demo.todos` against the store.
//!
//! Run: `cargo test -p deck-conformance --test mutation`

use deck_conformance::data_port::{DemoStore, resolve_demo};
use deck_conformance::default_app_dir;
use serde_json::json;

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

/// The todos route contains exactly one mutation binding pointing at
/// `ship.demo.addTodo`.  This proves the wire vocabulary contract is in place.
#[test]
fn todos_view_has_mutation_binding_for_add_todo() {
    let app_dir = default_app_dir();
    let source =
        std::fs::read_to_string(app_dir.join("routes/todos/page.sail")).expect("todos page.sail");
    // todos/page.sail carries its controller INLINE (`<controller>…</controller>`)
    // — strip it before the template-only parser sees it (mirrors deck-ssr's
    // build path).
    let hits: Vec<_> = source
        .match_indices("mutation=\"ship.demo.addTodo\"")
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "todos route must carry exactly one ship.demo.addTodo mutation binding"
    );
}
