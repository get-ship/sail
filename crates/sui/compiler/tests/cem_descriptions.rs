//! Verifies that JSDoc HTML comments above `<prop>` declarations land
//! as `description` strings on the corresponding CEM attribute and
//! member entries, while props without one stay free of the field.

use std::fs;
use std::path::PathBuf;

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn jsdoc_above_prop_emits_description_in_cem() {
    let path = fixtures_dir().join("jsdoc-prop.sui");
    let source = fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        styles: None,
        component_meta: Some(ComponentMeta {
            name: "jsdoc-prop-test".into(),
            shadow: false,
        }),
        emit_dts: false,
        emit_ssr: false,
        emit_component_imports: false,
    };
    let out = compile(&source, opts).expect("compile");
    let cem = out.cem.expect("CEM emitted for component-mode");
    let decl = &cem["modules"][0]["declarations"][0];

    let attrs = decl["attributes"].as_array().unwrap();
    let tone_attr = attrs.iter().find(|a| a["name"] == "tone").unwrap();
    assert_eq!(tone_attr["description"], "Picks the active tone");
    let label_attr = attrs.iter().find(|a| a["name"] == "label").unwrap();
    assert!(
        label_attr.get("description").is_none(),
        "props without JSDoc should not gain a description field, got: {label_attr}",
    );

    let members = decl["members"].as_array().unwrap();
    let tone_member = members.iter().find(|m| m["name"] == "tone").unwrap();
    assert_eq!(tone_member["description"], "Picks the active tone");
}
