//! The host's data resolver — deck's seam to "what does `(domain, op, args)`
//! resolve to right now".
//!
//! Deck passes only the opaque triple; the host binds its own state
//! (e.g. `ApiState`) INSIDE the impl, so host types never cross the deck
//! boundary. Returns the FULL current value (snapshot family), or `None` when
//! the host has no resolver for `(domain, op)` — deck surfaces that to the
//! client as an explicit error, never a silent-empty value.
//!
//! The deck-side multiplex streaming handler (deck-axum) reaches the host via
//! `Extension<Arc<dyn DataPort>>`. The host (e.g. shipd) constructs the impl
//! at router-mount time.

use serde_json::Value;

/// Resolver seam between deck's multiplex transport and the host's data layer.
///
/// `None` is a contract: there is no resolver for `(domain, op)`. Deck must
/// surface that to the client (HTTP error / explicit close frame), never emit a
/// silent `null` data event. The host's own resolve_fn returns `None` for
/// unregistered ops for the same reason (live_resolvers.rs).
pub trait DataPort: Send + Sync + 'static {
    /// Resolve `(domain, op, args)` to its current full value, or `None` when
    /// the host has no resolver registered for `(domain, op)`.
    fn resolve(&self, domain: &str, op: &str, args: &Value) -> Option<Value>;

    /// Whether the host is provisioned (has at least one active workspace).
    ///
    /// Drives deck's first-run routing: `GET /` serves the shell when
    /// provisioned and redirects to `/setup` otherwise; `GET /setup` redirects
    /// to `/` once provisioned. The default (`false`) is the fresh-install
    /// posture — a host with real workspace state overrides this. deck names
    /// no Ship concept; the host binds its own provisioning check inside the
    /// impl. (The shipd impl reads its active-project list.)
    fn is_provisioned(&self) -> bool {
        false
    }
}
