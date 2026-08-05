#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Custom-elements-manifest entry. Conforms to
//! <https://github.com/webcomponents/custom-elements-manifest>.

use crate::CompileOpts;
use crate::ir::*;
use serde_json::{Value, json};

pub fn emit_cem(program: &Program, opts: &CompileOpts) -> Value {
    let meta = opts
        .component_meta
        .as_ref()
        .expect("component_meta required for CEM");
    let tag = &meta.name;
    let path = opts
        .source_path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{tag}.sail"));

    // Identity props are plain properties, not attributes — they never appear
    // in the element's attribute surface (only as fields/members below).
    let attributes: Vec<Value> = program
        .props
        .iter()
        .filter(|p| !matches!(p.kind, PropKind::Identity(_)))
        .map(attr_entry)
        .collect();
    let members: Vec<Value> = program.props.iter().map(member_entry).collect();
    let mut declaration = json!({
        "kind": "class",
        "name": class_name_for(tag),
        "tagName": tag,
        "customElement": true,
        "superclass": { "name": "HTMLElement", "package": "global" },
        "attributes": attributes,
        "members": members,
    });
    // ANATOMY — the sail extension over standard CEM. Classes + slots are the
    // published styling contract (classes ARE the anatomy in the light-DOM
    // architecture; skins are written against them), `dom` records which side
    // of the light/shadow seam this component compiles to. States live on the
    // standard `attributes`/`members` entries (reflects: true).
    {
        let (classes, slots) = collect_anatomy(&program.template);
        let obj = declaration.as_object_mut().unwrap();
        obj.insert(
            "anatomy".into(),
            json!({
                "dom": "light",
                "classes": classes,
                "slots": slots,
            }),
        );
    }

    let module = json!({
        "kind": "javascript-module",
        "path": path,
        "declarations": [declaration],
        "exports": [{
            "kind": "custom-element-definition",
            "name": tag,
            "declaration": { "name": class_name_for(tag), "module": path },
        }],
    });

    json!({
        "schemaVersion": "1.0.0",
        "readme": "",
        "modules": [module],
        "_compilerVersion": crate::compiler_version().to_json(),
    })
}

/// Walk the template collecting the anatomy: author-authored class names
/// (compile-generated `sx-*`/`sail-*` atomics excluded — they are emission
/// detail, not contract) and slot names (`(default)` for the unnamed slot).
/// Sorted + deduped for deterministic manifests.
pub(crate) fn collect_anatomy(template: &[Instr]) -> (Vec<String>, Vec<String>) {
    let mut classes = std::collections::BTreeSet::new();
    let mut slots = std::collections::BTreeSet::new();
    walk(template, &mut classes, &mut slots);
    fn walk(
        instrs: &[Instr],
        classes: &mut std::collections::BTreeSet<String>,
        slots: &mut std::collections::BTreeSet<String>,
    ) {
        for instr in instrs {
            let children: &[Instr] = match instr {
                Instr::Element(el) => {
                    if el.tag == "slot" {
                        let name = el.attrs.iter().find_map(|a| match a {
                            AttrInstr::Static { name, value, .. } if name == "name" => {
                                Some(value.clone())
                            }
                            _ => None,
                        });
                        slots.insert(name.unwrap_or_else(|| "(default)".into()));
                    }
                    for a in &el.attrs {
                        if let AttrInstr::Static { name, value, .. } = a
                            && name == "class"
                        {
                            for c in value.split_whitespace() {
                                if !c.starts_with("sx-") && !c.starts_with("sail-") {
                                    classes.insert(c.to_string());
                                }
                            }
                        }
                    }
                    &el.children
                }
                Instr::If(i) => {
                    for b in &i.branches {
                        walk(&b.body, classes, slots);
                    }
                    &[]
                }
                Instr::For(f) => &f.body,
                Instr::Match(m) => {
                    for c in &m.cases {
                        walk(&c.body, classes, slots);
                    }
                    if let Some(d) = &m.default {
                        walk(d, classes, slots);
                    }
                    &[]
                }
                Instr::When(w) => {
                    walk(&w.body, classes, slots);
                    if let Some(e) = &w.else_body {
                        walk(e, classes, slots);
                    }
                    &[]
                }
                Instr::WhileLoading(w) => &w.body,
                Instr::WhileError(w) => &w.body,
                _ => &[],
            };
            walk(children, classes, slots);
        }
    }
    (classes.into_iter().collect(), slots.into_iter().collect())
}

fn attr_entry(p: &PropDecl) -> Value {
    let mut e = json!({
        "name": p.name,
        "type": { "text": prop_type_text(&p.kind) },
    });
    if let Some(d) = p.default.as_ref() {
        e.as_object_mut()
            .unwrap()
            .insert("default".into(), Value::String(d.clone()));
    }
    if let Some(desc) = p.description.as_ref() {
        e.as_object_mut()
            .unwrap()
            .insert("description".into(), Value::String(desc.clone()));
    }
    if let PropKind::Union(values) = &p.kind {
        e.as_object_mut()
            .unwrap()
            .insert("values".into(), json!(values));
    }
    e
}

fn member_entry(p: &PropDecl) -> Value {
    let camel = camel_of(&p.name);
    let mut m = json!({
        "kind": "field",
        "name": camel,
        "type": { "text": prop_type_text(&p.kind) },
    });
    // Only attribute-backed props carry an `attribute`; Identity props are
    // property-only and must not advertise one.
    if !matches!(p.kind, PropKind::Identity(_)) {
        m.as_object_mut()
            .unwrap()
            .insert("attribute".into(), Value::String(p.name.clone()));
    }
    if let Some(d) = p.default.as_ref() {
        m.as_object_mut()
            .unwrap()
            .insert("default".into(), Value::String(d.clone()));
    }
    if let Some(desc) = p.description.as_ref() {
        m.as_object_mut()
            .unwrap()
            .insert("description".into(), Value::String(desc.clone()));
    }
    if p.reflect {
        m.as_object_mut()
            .unwrap()
            .insert("reflects".into(), Value::Bool(true));
    }
    m
}

fn prop_type_text(k: &PropKind) -> String {
    match k {
        PropKind::String => "string".to_string(),
        PropKind::Bool => "boolean".to_string(),
        PropKind::Number => "number".to_string(),
        PropKind::Union(values) => values
            .iter()
            .map(|v| format!("'{v}'"))
            .collect::<Vec<_>>()
            .join(" | "),
        PropKind::Identity(t) => t.clone(),
    }
}

fn class_name_for(tag: &str) -> String {
    let camel = camel_of(tag);
    let mut chars = camel.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn camel_of(attr: &str) -> String {
    let mut out = String::with_capacity(attr.len());
    let mut up_next = false;
    for c in attr.chars() {
        if c == '-' {
            up_next = true;
            continue;
        }
        if up_next {
            out.push(c.to_ascii_uppercase());
            up_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Aggregate per-component CEM documents into one standard manifest.
///
/// Input: slice of per-component `{schemaVersion, modules:[...]}` values
/// (as produced by `emit_cem`).
/// Output: `{schemaVersion:"1.0.0", modules:[<all concatenated, sorted by "path">]}`.
/// Pure, deterministic, no I/O. sail owns this format.
pub fn merge_cem(components: &[Value]) -> Value {
    let mut all_modules: Vec<Value> = components
        .iter()
        .filter_map(|v| v["modules"].as_array().cloned())
        .flatten()
        .collect();
    all_modules.sort_by(|a, b| {
        let a_path = a["path"].as_str().unwrap_or("");
        let b_path = b["path"].as_str().unwrap_or("");
        a_path.cmp(b_path)
    });
    json!({
        "schemaVersion": "1.0.0",
        "modules": all_modules,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_two_cems_sorts_by_path() {
        let cem_z = json!({
            "schemaVersion": "1.0.0",
            "modules": [{"path": "z.sail", "kind": "javascript-module"}]
        });
        let cem_a = json!({
            "schemaVersion": "1.0.0",
            "modules": [{"path": "a.sail", "kind": "javascript-module"}]
        });
        let merged = merge_cem(&[cem_z, cem_a]);
        let modules = merged["modules"].as_array().unwrap();
        assert_eq!(modules.len(), 2, "two modules in merged manifest");
        assert_eq!(modules[0]["path"], "a.sail", "a.sail sorts before z.sail");
        assert_eq!(modules[1]["path"], "z.sail", "z.sail sorts after a.sail");
        assert_eq!(merged["schemaVersion"], "1.0.0");
    }

    #[test]
    fn merge_empty_returns_empty_manifest() {
        let merged = merge_cem(&[]);
        assert_eq!(merged["schemaVersion"], "1.0.0");
        let modules = merged["modules"].as_array().unwrap();
        assert!(modules.is_empty(), "no modules for empty input");
    }
}
