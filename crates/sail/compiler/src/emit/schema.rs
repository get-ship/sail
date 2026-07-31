#![allow(clippy::unwrap_used)]
//! Emit a JSON-serializable description of a file's directive
//! declarations: `<prop>`. Used by tooling (HMR, error overlay,
//! devserver) that needs a quick semantic summary without the JS bundle.
//!
//! `"data"` stays in the wire shape as an always-empty array — the
//! retired `<data>` directive was its only producer, but the field
//! itself is part of the checked-in schema contract downstream tooling
//! reads, so it isn't dropped here.

use crate::CompileOpts;
use crate::ir::*;
use serde_json::{Value, json};

pub fn emit_schema(program: &Program, opts: &CompileOpts) -> Value {
    let component = opts
        .component_meta
        .as_ref()
        .map(|m| component_value(&m.name, &program.props));

    json!({
        "data": Value::Array(Vec::new()),
        "component": component,
        "_compilerVersion": crate::compiler_version().to_json(),
    })
}

fn component_value(tag: &str, props: &[PropDecl]) -> Value {
    let prop_values: Vec<Value> = props.iter().map(prop_value).collect();
    let obj = json!({ "tag": tag, "props": prop_values });
    obj
}

fn prop_value(p: &PropDecl) -> Value {
    let kind = match &p.kind {
        PropKind::String => json!({ "type": "string" }),
        PropKind::Bool => json!({ "type": "bool" }),
        PropKind::Number => json!({ "type": "number" }),
        PropKind::Union(values) => json!({ "type": "union", "values": values }),
        PropKind::Identity(_) => json!({ "type": "property" }),
    };
    let mut entry = json!({
        "name": p.name,
        "default": p.default,
        "reflect": p.reflect,
        "bindable": p.bindable,
    });
    let merged = entry.as_object_mut().unwrap();
    for (k, v) in kind.as_object().unwrap() {
        merged.insert(k.clone(), v.clone());
    }
    Value::Object(merged.clone())
}
