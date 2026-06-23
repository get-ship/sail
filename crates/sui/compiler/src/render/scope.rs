//! Build the initial render scope from a parsed `Program`.
//!
//! Two outputs:
//! * `Scope` — `HashMap<name, Value>` consumed by the evaluator while
//!   walking the template.
//! * `init: Value` — the JSON object emitted into the
//!   `<script type="application/json" id="__set_init__">` island, with
//!   each `<data>` declaration's first-paint value.
//!
//! `<data>` initials are always `{ "kind": "loading" }` envelopes —
//! D-2 is the static path; the daemon-cache integration is Phase F's
//! concern. `live` data and non-live data both render the same
//! envelope.
//!
//! Reactive component state is declared in the controller's `scope:`
//! factory and runs at hydrate; the SSR walker has no visibility into
//! it, so state-bound template expressions evaluate against an
//! unbound scope on first paint.
//!
//! A `route` placeholder with empty `params`/`query` is added so
//! expressions like `route.params.id` evaluate to `undefined` cleanly
//! instead of unbound errors.

use serde_json::{Map, Value, json};

use crate::ir::Program;

use super::eval::Scope;

/// Result of scope construction. The walker reads `scope`; the
/// renderer emits `init` into the JSON island.
pub struct InitialScope {
    pub scope: Scope,
    pub init: Value,
}

pub fn build(program: &Program) -> InitialScope {
    let mut scope: Scope = Scope::new();
    let mut init: Map<String, Value> = Map::new();

    scope.insert(
        "route".into(),
        json!({ "params": {}, "query": {} }),
    );

    for data in &program.data {
        let envelope = json!({ "kind": "loading" });
        scope.insert(data.name.clone(), envelope.clone());
        init.insert(data.name.clone(), envelope);
    }

    InitialScope { scope, init: Value::Object(init) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    fn parse_src(src: &str) -> Program {
        parse(src).expect("parse")
    }

    #[test]
    fn data_emits_loading_envelope() {
        let p = parse_src("<data name=\"agents\" source=\"ship.agents.summaries()\" live />");
        let s = build(&p);
        assert_eq!(s.init.get("agents").unwrap(), &json!({"kind": "loading"}));
        assert_eq!(s.scope.get("agents").unwrap(), &json!({"kind": "loading"}));
    }

    #[test]
    fn route_placeholder_is_present() {
        let p = parse_src("<data name=\"x\" source=\"ship.x()\" />");
        let s = build(&p);
        let route = s.scope.get("route").expect("route in scope");
        assert_eq!(route, &json!({"params": {}, "query": {}}));
    }
}
