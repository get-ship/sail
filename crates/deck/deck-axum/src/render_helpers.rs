//! View-source render helpers + the axum `not_found` shim.
//!
//! The pure helpers (`is_valid_view_name`, `render_view_source`) live in
//! deck-core; this module re-exports them so `deck::render_helpers::*`
//! keeps resolving. `not_found` returns an `axum::response::Response`, so
//! it belongs in the adapter, not deck-core.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

pub use deck_core::render_helpers::{is_valid_view_name, render_view_source};

/// Plain-text 404 axum response. Used by state-aware SUI handlers in shipd
/// (sui_mutation, sui_sse) that build axum responses directly.
pub fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}
