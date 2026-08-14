//! Lean test-contract map (SPEC-TEST-2 Phase A).
//!
//! Dev artifact only: the queryable surface a test may name — stamped
//! roles, accessible names, and semantic states. No IR dump, no
//! `data-*` selectors, no class lists.

use serde::Serialize;

use crate::ir::{AttrInstr, ElementInstr, Instr, Program};
use crate::semantic_props::{host_anatomy, lookup as lookup_semantic_prop};

/// One queryable host on a compiled `.sail` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestEntry {
    /// ARIA / implicit role (`button`, `status`, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Accessible name (author `label` or fixed anatomy label).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Semantic state prop names present on the host (`pressed`, …).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<String>,
}

/// Per-file contract. Sized to the queryable set, not the IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestMap {
    pub version: u32,
    pub file: String,
    pub entries: Vec<TestEntry>,
}

/// Build the lean map for `program`. `file` is a workspace-relative label.
pub fn emit_test_map(program: &Program, file: &str) -> TestMap {
    let mut entries = Vec::new();
    collect_instrs(&program.template, &mut entries);
    TestMap {
        version: 1,
        file: file.replace('\\', "/"),
        entries,
    }
}

const MAP_KEYS: &[&str] = &["version", "file", "entries"];
const ENTRY_KEYS: &[&str] = &["role", "name", "states"];

/// Fail if the map grew a non-queryable field (only role / name / states).
pub fn validate_test_map(map: &TestMap) -> Result<(), String> {
    if map.version != 1 {
        return Err(format!("unsupported test-map version {}", map.version));
    }
    validate_test_map_value(&serde_json::to_value(map).map_err(|e| e.to_string())?)
}

/// Same key allowlist against a JSON value (compiler emit + hand-written maps).
pub fn validate_test_map_value(value: &serde_json::Value) -> Result<(), String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "test map must be an object".to_string())?;
    for k in obj.keys() {
        if !MAP_KEYS.contains(&k.as_str()) {
            return Err(format!("unknown map field `{k}`"));
        }
    }
    let version = obj.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
    if version != 1 {
        return Err(format!("unsupported test-map version {version}"));
    }
    let entries = obj
        .get("entries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "test map entries must be an array".to_string())?;
    for (i, entry) in entries.iter().enumerate() {
        let e = entry
            .as_object()
            .ok_or_else(|| format!("entries[{i}] must be an object"))?;
        for k in e.keys() {
            if !ENTRY_KEYS.contains(&k.as_str()) {
                return Err(format!("entries[{i}] unknown field `{k}`"));
            }
        }
        let role = e.get("role").and_then(|v| v.as_str());
        let name = e.get("name").and_then(|v| v.as_str());
        if role.is_none() && name.is_none() {
            return Err(format!("entries[{i}] has no role or name"));
        }
        if let Some(states) = e.get("states") {
            let arr = states
                .as_array()
                .ok_or_else(|| format!("entries[{i}].states must be an array"))?;
            for s in arr {
                let Some(name) = s.as_str() else {
                    return Err(format!("entries[{i}].states must be strings"));
                };
                if lookup_semantic_prop(name).is_none() {
                    return Err(format!("entries[{i}] unknown state `{name}`"));
                }
            }
        }
    }
    Ok(())
}

fn collect_instrs(instrs: &[Instr], out: &mut Vec<TestEntry>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                if let Some(e) = entry_for_element(el) {
                    out.push(e);
                }
                collect_instrs(&el.children, out);
            }
            Instr::If(i) => {
                for b in &i.branches {
                    collect_instrs(&b.body, out);
                }
                if let Some(els) = &i.else_body {
                    collect_instrs(els, out);
                }
            }
            Instr::For(f) => {
                collect_instrs(&f.body, out);
                if let Some(els) = &f.else_body {
                    collect_instrs(els, out);
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    collect_instrs(&c.body, out);
                }
                if let Some(def) = &m.default {
                    collect_instrs(def, out);
                }
            }
            Instr::When(w) => {
                collect_instrs(&w.body, out);
                if let Some(els) = &w.else_body {
                    collect_instrs(els, out);
                }
            }
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => {}
        }
    }
}

fn entry_for_element(el: &ElementInstr) -> Option<TestEntry> {
    let tag = el.tag.to_ascii_lowercase();
    // After L1 splice the host tag is often a native element; recover
    // the language name from the compiler stamp without publishing it.
    let l1 = static_attr(el, "data-sail-host").unwrap_or(tag.as_str());
    let anatomy = host_anatomy(l1);
    let mut states = Vec::new();
    let mut name = anatomy.and_then(|a| a.label.map(str::to_string));
    for attr in &el.attrs {
        let (n, static_val) = match attr {
            AttrInstr::Static { name, value, .. } => (name.as_str(), Some(value.as_str())),
            AttrInstr::Expr { name, .. } => (name.as_str(), None),
            _ => continue,
        };
        if n.starts_with("data-") || n == "class" || n == "style" {
            continue;
        }
        if let Some(prop) = lookup_semantic_prop(n) {
            if prop.name == "label" {
                if let Some(v) = static_val {
                    if !v.is_empty() {
                        name = Some(v.to_string());
                    }
                }
            } else if !states.iter().any(|s| s == prop.name) {
                states.push(prop.name.to_string());
            }
        }
    }
    let role = anatomy
        .and_then(|a| a.role.map(str::to_string))
        .or_else(|| implicit_role(&tag, el));
    if role.is_none() && name.is_none() {
        return None;
    }
    states.sort();
    Some(TestEntry { role, name, states })
}

fn implicit_role(tag: &str, el: &ElementInstr) -> Option<String> {
    Some(
        match tag {
            "button" => "button",
            "a" if has_attr(el, "href") => "link",
            "nav" => "navigation",
            "main" => "main",
            "header" => "banner",
            "footer" => "contentinfo",
            "aside" => "complementary",
            "form" => "form",
            "img" => "img",
            "table" => "table",
            "ul" | "ol" => "list",
            "li" => "listitem",
            "option" => "option",
            "dialog" => "dialog",
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "heading",
            "textarea" => "textbox",
            "select" => "combobox",
            "progress" => "progressbar",
            "input" => match static_attr(el, "type").unwrap_or("text") {
                "checkbox" => "checkbox",
                "radio" => "radio",
                "submit" | "button" | "reset" | "image" => "button",
                "range" => "slider",
                "number" => "spinbutton",
                "search" => "searchbox",
                _ => "textbox",
            },
            _ => return None,
        }
        .to_string(),
    )
}

fn has_attr(el: &ElementInstr, name: &str) -> bool {
    el.attrs.iter().any(|a| match a {
        AttrInstr::Static { name: n, .. } | AttrInstr::Expr { name: n, .. } => n == name,
        _ => false,
    })
}

fn static_attr<'a>(el: &'a ElementInstr, name: &str) -> Option<&'a str> {
    el.attrs.iter().find_map(|a| match a {
        AttrInstr::Static { name: n, value, .. } if n == name => Some(value.as_str()),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn map_is_only_queryable_contract() {
        let src = r#"
<spinner />
<button label="Save">Save</button>
<div class="nope" data-test="x">plain</div>
"#;
        let program = parse(src).expect("parse");
        let map = emit_test_map(&program, "apps/demo/routes/page.sail");
        validate_test_map(&map).expect("lean");
        assert_eq!(map.version, 1);
        assert!(
            map.entries.iter().any(
                |e| e.role.as_deref() == Some("status") && e.name.as_deref() == Some("Loading")
            ),
            "spinner anatomy: {map:?}"
        );
        assert!(
            map.entries
                .iter()
                .any(|e| e.role.as_deref() == Some("button") && e.name.as_deref() == Some("Save")),
            "author label: {map:?}"
        );
        assert!(
            !map.entries
                .iter()
                .any(|e| e.role.is_none() && e.name.is_none()),
            "no empty rows: {map:?}"
        );
        let json = serde_json::to_value(&map).unwrap();
        let keys: Vec<_> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["version", "file", "entries"]);
        for e in json["entries"].as_array().unwrap() {
            for k in e.as_object().unwrap().keys() {
                assert!(
                    matches!(k.as_str(), "role" | "name" | "states"),
                    "entry key {k}"
                );
            }
        }
        assert!(
            validate_test_map_value(&serde_json::json!({
                "version": 1,
                "file": "x.sail",
                "entries": [{ "role": "button", "class": "sail-btn" }]
            }))
            .is_err()
        );
        assert!(
            validate_test_map_value(&serde_json::json!({
                "version": 1,
                "file": "x.sail",
                "entries": [{ "role": "button", "name": "data-test" }]
            }))
            .is_ok()
        );
    }

    #[test]
    fn compile_fills_lean_test_map() {
        let out = crate::compile(
            r#"<button label="Save">Save</button>"#,
            crate::CompileOpts {
                emit_dts: false,
                ..Default::default()
            },
        )
        .expect("compile");
        let map = out.test_map.expect("test_map");
        validate_test_map(&map).expect("lean");
        assert!(
            map.entries
                .iter()
                .any(|e| e.role.as_deref() == Some("button") && e.name.as_deref() == Some("Save"))
        );
    }

    #[test]
    fn native_link_and_states() {
        let src = r#"<a href="/docs" label="Docs">Docs</a><button pressed>On</button>"#;
        let program = parse(src).expect("parse");
        let map = emit_test_map(&program, "x.sail");
        validate_test_map(&map).expect("lean");
        assert!(
            map.entries
                .iter()
                .any(|e| e.role.as_deref() == Some("link") && e.name.as_deref() == Some("Docs"))
        );
        assert!(map.entries.iter().any(
            |e| e.role.as_deref() == Some("button") && e.states.iter().any(|s| s == "pressed")
        ));
    }
}
