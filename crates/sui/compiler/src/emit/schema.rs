#![allow(clippy::unwrap_used)]
//! Emit a JSON-serializable description of a file's directive
//! declarations: `<data>`, `<prop>`. Used by tooling (HMR, error
//! overlay, devserver) that needs a quick semantic summary without
//! the JS bundle.

use crate::ir::*;
use crate::CompileOpts;
use serde_json::{json, Value};

pub fn emit_schema(program: &Program, opts: &CompileOpts) -> Value {
    let data: Vec<Value> = program
        .data
        .iter()
        .map(|d| json!({ "name": d.name, "source": d.source.source, "live": d.live }))
        .collect();
    let component = opts.component_meta.as_ref().map(|m| component_value(&m.name, &program.props));

    json!({
        "data": data,
        "component": component,
        "_compilerVersion": crate::compiler_version().to_json(),
    })
}

fn component_value(tag: &str, props: &[PropDecl]) -> Value {
    let prop_values: Vec<Value> = props.iter().map(prop_value).collect();
    let form_associated = props.iter().any(|p| p.form_value || p.form_disabled);
    let mut obj = json!({ "tag": tag, "props": prop_values });
    if form_associated {
        obj.as_object_mut().unwrap().insert(
            "formAssociated".into(),
            Value::Bool(true),
        );
    }
    obj
}

fn prop_value(p: &PropDecl) -> Value {
    let kind = match &p.kind {
        PropKind::String => json!({ "type": "string" }),
        PropKind::Bool => json!({ "type": "bool" }),
        PropKind::Number => json!({ "type": "number" }),
        PropKind::Union(values) => json!({ "type": "union", "values": values }),
    };
    let mut entry = json!({
        "name": p.name,
        "default": p.default,
        "reflect": p.reflect,
        "bindable": p.bindable,
    });
    let merged = entry.as_object_mut().unwrap();
    if p.form_value {
        merged.insert("formValue".into(), Value::Bool(true));
    }
    if p.form_disabled {
        merged.insert("formDisabled".into(), Value::Bool(true));
    }
    for (k, v) in kind.as_object().unwrap() {
        merged.insert(k.clone(), v.clone());
    }
    Value::Object(merged.clone())
}
