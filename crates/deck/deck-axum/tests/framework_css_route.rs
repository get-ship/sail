//! End-to-end proof of the framework-stylesheet route over real HTTP —
//! the `deck` binary can't link right now (an unrelated deck-ssr / ship-exec
//! signature drift in the ship repo), so this boots `deck_axum::router()`
//! directly to verify W0 style delivery independent of the SSR engine.

use tokio::net::TcpListener;

async fn serve() -> std::net::SocketAddr {
    let app = deck_axum::router();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    addr
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn serves_framework_stylesheet_with_both_halves() {
    let addr = serve().await;
    let res = reqwest::Client::new()
        .get(format!("http://{addr}/api/assets/sui.css"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    assert!(
        res.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .starts_with("text/css"),
        "served as CSS",
    );
    let body = res.text().await.unwrap();
    // Contracts half (a tone channel) + element-skins half (spinner skin).
    assert!(body.contains("[tone="), "contracts present");
    assert!(body.contains(".sui-spinner"), "element skins present");
    // The widened icon vocabulary rode along the skins.
    assert!(body.contains(".sui-icon"), "icon skin present");
}
