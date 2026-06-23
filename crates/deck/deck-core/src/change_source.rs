//! Server-authoritative change ticks — the seam between deck's multiplex
//! re-resolve loop and the host's per-stream "something changed" signal.
//!
//! One `ChangeFeed` per multiplex stream. Each `recv().await` resolves to
//! `Some(())` (a change occurred — re-resolve every sub) or `None` (feed
//! ended — deck closes the stream). The tick is payload-free on purpose:
//! the transport is server-authoritative; deck re-resolves from
//! [`DataPort`](crate::data_port::DataPort) on every tick rather than
//! trusting the event body.
//!
//! The host's impl (e.g. shipd's `ShipChangeSource`) typically spawns a kernel
//! actor subscribed to the `config` namespace and forwards each mailbox recv
//! as a tick. Dropping the feed tears down the host's spawned actor (the
//! `ActorGuard` analogue in deck-axum) so per-stream resources don't leak.

use std::pin::Pin;

/// One live change feed for a single multiplex stream. Each `recv().await`
/// resolves to `Some(())` (a change occurred) or `None` (feed ended — deck
/// closes the stream). Dropping the feed tears down the host's spawned
/// resource (e.g. kernel actor); the host's impl wires that via `Drop`.
pub trait ChangeFeed: Send {
    /// Await the next change tick.
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Option<()>> + Send + '_>>;
}

/// Provider of `ChangeFeed` instances — one per multiplex stream.
///
/// The host wires this as a `Box<dyn Fn() -> Pin<Box<dyn Future<...>>>>` (or
/// similar) into the multiplex mount; deck's stream handler invokes it to
/// open a feed for each open stream. Kept as a free function pointer shape
/// so the host can build it inline at router-mount time without an extra
/// struct.
pub trait ChangeSource: Send + Sync + 'static {
    /// Open a new change feed for one multiplex stream. Returning `Err`
    /// surfaces as an explicit error to the client.
    fn open_feed(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn ChangeFeed>, String>> + Send + '_>>;
}
