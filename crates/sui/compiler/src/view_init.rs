//! First-paint hydration seed for a `.sui` view.
//!
//! This is the render-engine-independent half of serving a view: the
//! initial scope JSON the client hydrates from, whether or not the page
//! was server-rendered. Each `<data>` declaration seeds a
//! `{ "kind": "loading" }` envelope; the daemon swaps in real values over
//! the live wire after hydrate. `<client>` names and a live-data flag ride
//! along so the host can fold in its subscribe URL without re-parsing.
//!
//! The actual first-paint HTML comes from the JS SSR engine (ship-js).
//! Nothing here evaluates expressions — that was render-eval's job, now
//! removed.

use serde_json::{Map, Value, json};

use crate::diagnostics::CompileError;
use crate::ir::Program;

/// Everything a host needs to ship a view payload, minus the HTML body
/// (which the SSR engine produces, or which the client builds on its own).
#[derive(Debug, Clone)]
pub struct ViewInit {
    /// Initial scope JSON for the `__set_init__` hydration island —
    /// one `{ "kind": "loading" }` envelope per `<data>` declaration.
    pub init: Value,
    /// Whether any `<data>` is `live` (the host folds in a subscribe URL).
    pub has_live_data: bool,
    /// Declared `<client>` names, in source order.
    pub clients: Vec<String>,
}

/// Build the hydration seed from an already-parsed program.
pub fn build(program: &Program) -> ViewInit {
    let mut init: Map<String, Value> = Map::new();
    for data in &program.data {
        init.insert(data.name.clone(), json!({ "kind": "loading" }));
    }
    ViewInit {
        init: Value::Object(init),
        has_live_data: program.data.iter().any(|d| d.live),
        clients: program.clients.iter().map(|c| c.name.clone()).collect(),
    }
}

/// Parse a `.sui` source and build its hydration seed. The serve-path
/// counterpart to [`crate::compile`] — hosts call this instead of reaching
/// into `parse` themselves.
pub fn view_init(source: &str) -> Result<ViewInit, CompileError> {
    let program = crate::parse::parse(source)?;
    Ok(build(&program))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn data_seeds_loading_envelopes() {
        let vi = view_init(
            "<data name=\"agents\" source=\"ship.agents()\" live />\n<p>hi</p>",
        )
        .expect("view_init");
        assert_eq!(vi.init.get("agents").unwrap(), &json!({ "kind": "loading" }));
        assert!(vi.has_live_data);
    }

    #[test]
    fn no_data_is_empty_seed() {
        let p = parse("<p>static</p>").expect("parse");
        let vi = build(&p);
        assert_eq!(vi.init, json!({}));
        assert!(!vi.has_live_data);
        assert!(vi.clients.is_empty());
    }
}
