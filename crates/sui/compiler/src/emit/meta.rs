//! `.sui.meta.json` sidecar.
//!
//! Phase 3b.5 (gate L): every binding kind is now wired by the
//! compiled `hydrate(root, ctx)` directly — the runtime no longer
//! reads a `bindings` manifest. But LSP/IDE tooling still wants a
//! quick semantic summary of every binding in a source: marker key,
//! kind, raw expression, dependent identifiers, and (for while-* /
//! await) the render-callback name. We emit the same shape as the
//! pre-3b.5 manifest, just into a sidecar file the runtime never
//! loads.
//!
//! Consumers: ship-ui-lsp (hover, go-to-definition), HMR diff
//! tooling, error-overlay binding lookup. Not consumed by attach().
//!
//! Output shape:
//!
//! ```json
//! {
//!   "bindings": [
//!     { "key": "b0", "kind": "text", "expr": "count()", "deps": ["count"] },
//!     { "key": "b1", "kind": "on:click", "expr": "incr",  "deps": ["incr"] },
//!     { "key": "b2", "kind": "while-loading", "expr": "agents", "deps": ["agents"], "renderKey": "__renderWhileLoading_0" }
//!   ]
//! }
//! ```

use crate::ir::Program;
use crate::keys::BindingRegistry;
use serde_json::{json, Value};

pub fn emit_meta(_program: &Program, bindings: &BindingRegistry) -> Value {
    let entries: Vec<Value> = bindings
        .iter()
        .filter(|b| !b.suppressed)
        .map(|b| {
            let mut entry = json!({
                "key": b.key,
                "kind": b.kind.as_marker_str(),
                "expr": b.expr,
                "deps": b.deps,
            });
            if let (Some(rk), Some(obj)) = (&b.render_key, entry.as_object_mut()) {
                obj.insert("renderKey".into(), Value::String(rk.clone()));
            }
            entry
        })
        .collect();
    // `"clients"` stays in the wire shape as an always-empty array — the
    // retired `<client>` directive was its only producer, but the field
    // itself is part of the checked-in sidecar contract downstream tooling
    // (and the compiled module's own default export) reads, so it isn't
    // dropped here.
    json!({
        "bindings": entries,
        "clients": Value::Array(Vec::new()),
        "_compilerVersion": crate::compiler_version().to_json(),
    })
}
