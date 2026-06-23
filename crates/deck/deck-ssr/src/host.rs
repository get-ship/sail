//! The host-side [`Dispatch`] — the other end of ship-js's one wire.
//!
//! ship-js carries no auth, no data, no routing: a running module reaches the
//! host ONLY through `op_ship_call(op, args)`, which the engine forwards here.
//! So "resolve route/data server-side without a browser" reduces to: answer
//! those ops here, in Rust.
//!
//!   * `route.current` → the URL state the deck router already matched for
//!     this request (the host KNOWS the params — it matched the route to pick
//!     the view; server-side they are data, not a `window.location` re-parse).
//!   * any other op → the real op surface via [`deck_core::DataPort`]. The
//!     conformance data vocabulary is `<root>.demo.<method>`; the op string is
//!     split on its first `.` into `(domain, op)` for `DataPort::resolve`.
//!
//! This is the seam the full view-host plugs the live op surface into.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use deck_core::DataPort;
use serde_json::{json, Value};
use ship_js::{Dispatch, DispatchError};

/// A request-shaped host: the matched route + (optionally) the live data port.
pub struct SsrDispatch {
    route: Value,
    data: Option<Arc<dyn DataPort>>,
    calls: Mutex<Vec<(String, Value)>>,
}

impl SsrDispatch {
    /// Build for a matched request. `params`/`query` are the values the deck
    /// router captured when it picked the view for this pathname.
    pub fn for_request(pathname: &str, params: Value, query: Value) -> Self {
        Self {
            route: json!({ "pathname": pathname, "params": params, "query": query }),
            data: None,
            calls: Mutex::new(Vec::new()),
        }
    }

    /// Attach the live op surface so `<data>`/`<client>` ops resolve.
    pub fn with_data(mut self, data: Arc<dyn DataPort>) -> Self {
        self.data = Some(data);
        self
    }

    /// The recorded op names, in call order — the proof values came through
    /// the wire.
    pub fn call_path(&self) -> Vec<String> {
        self.calls.lock().unwrap().iter().map(|(op, _)| op.clone()).collect()
    }

    /// The params the host will answer `route.current` with.
    pub fn route_params(&self) -> Value {
        self.route.get("params").cloned().unwrap_or(Value::Null)
    }
}

#[async_trait]
impl Dispatch for SsrDispatch {
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
        self.calls.lock().unwrap().push((op.to_string(), args.clone()));
        if op == "route.current" {
            return Ok(self.route.clone());
        }
        if let Some(dp) = &self.data
            && let Some((domain, rest)) = op.split_once('.')
            && let Some(v) = dp.resolve(domain, rest, &args)
        {
            return Ok(v);
        }
        Err(DispatchError::Terminal(format!("deck-ssr: no host op `{op}`")))
    }
}
