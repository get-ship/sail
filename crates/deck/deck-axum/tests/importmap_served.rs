//! importmap↔served agreement: every bare specifier the bundler writes into
//! the generated importmap (`FRAMEWORK_ASSETS`) must be served by the dev
//! daemon. Without this, a fresh clone's importmap can point at
//! `/_deck/compiler-wasm.js` and the browser dies with
//! "Failed to resolve module specifier".

use std::collections::HashSet;

async fn serve() -> std::net::SocketAddr {
    let app = deck_axum::router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    addr
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_importmap_dist_file_is_served() {
    let addr = serve().await;
    let dists: HashSet<_> = deck_bundler::app::framework_assets()
        .iter()
        .chain(deck_bundler::app::editor_framework_assets().iter())
        .map(|(_, _, dist_rel)| format!("/{dist_rel}"))
        .collect();
    assert!(!dists.is_empty(), "FRAMEWORK_ASSETS must not be empty");

    let client = reqwest::Client::new();
    for path in dists {
        let url = format!("http://{addr}{path}");
        let res = client
            .get(&url)
            .send()
            .await
            .unwrap_or_else(|e| panic!("request failed for {url}: {e}"));
        assert_eq!(
            res.status(),
            200,
            "importmap entry {path} must be served (200), got {}",
            res.status()
        );
        let ct = res
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("javascript"),
            "importmap entry {path} must be served as JS, got {ct}"
        );
    }
}
