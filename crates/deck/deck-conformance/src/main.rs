//! deck-conformance — a ship-free deck server.
//!
//! The host contract lives in `lib.rs` ([`deck_conformance::deck_app`]); this binary
//! is just the listener. It serves the app under `app/` with NO daemon, NO
//! ship domains, NO Ship component pack — the deck framework standing on its
//! own (effectively the `deck serve` the CLI doesn't have yet).
//!
//! Run: `cargo run -p deck-conformance`. It exposes deck's own surface (framework
//! assets, the live-data port, the mutation tombstone). The named-route surface
//! (`/_deck/view(s)/*`) moved to the separate `ship-views` repo and are not served
//! here.

#[tokio::main]
async fn main() {
    let app_dir = deck_conformance::default_app_dir();
    // The dev server starts the FS watcher, so editing a route hot-reloads.
    let app = deck_conformance::deck_dev_server(app_dir.clone()).await;

    let addr = std::env::var("DECK_DEMO_ADDR").unwrap_or_else(|_| "127.0.0.1:7878".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("deck-conformance: bind {addr}: {e}"));
    println!(
        "deck-conformance: serving {} on http://{addr}",
        app_dir.display()
    );
    axum::serve(listener, app)
        .await
        .expect("deck-conformance: serve");
}
