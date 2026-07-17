//! A JSDoc comment above a prop is its DESCRIPTION, and it must reach the CEM —
//! which is what feeds editor autocomplete, the component showcase's prop tables, and
//! agent tooling. A prop with no comment stays free of the field.
//!
//! This test was `#[ignore]`d through the `<prop>` retirement ("prop descriptions in
//! the component block is a tracked follow-up"), and while it was silenced the
//! feature was not merely missing — it was BROKEN. `parse_object_fields` splits on
//! commas, a comment contains none, so a documented prop arrived as
//!
//!     "/** Picks the active tone */\n    tone: 'default' | 'primary'"
//!
//! and everything before the colon became the prop's NAME. The compiler emitted
//! `__props.____Picks_the_active_tone_` and the prop `tone` did not exist. It compiled
//! clean and shipped a component with no props. Documenting a prop — the first thing
//! anyone writing a real component does — silently destroyed it.

use std::fs;
use std::path::PathBuf;

use sail_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn jsdoc_above_prop_emits_description_in_cem() {
    let path = fixtures_dir().join("jsdoc-prop.sail");
    let source = fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: Some(ComponentMeta {
            name: "jsdoc-prop-test".into(),

        }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
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


/// THE REGRESSION GUARD. The description reaching the CEM is the feature; the prop
/// surviving at all is the bug. A documented prop must still be named `tone` — not
/// `____Picks_the_active_tone_`.
#[test]
fn a_documented_prop_keeps_its_name() {
    let src = r#"<scope>
  props: {
    /** Picks the active tone */
    tone: 'default' | 'primary' = 'default'
    /** Multi-line.
     *  Second line. */
    size: 'sm' | 'lg' = 'sm'
    // a line comment is not documentation, but it must not corrupt the name either
    plain = ''
  }
</scope>
<button>{tone()}{size()}{plain()}</button>"#;

    let out = compile(
        src,
        CompileOpts {
        route_params: None,
            mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "doc-probe".into() }),
            ..Default::default()
        },
    )
    .expect("compiles");

    for name in ["tone", "size", "plain"] {
        assert!(
            out.js.contains(&format!("__props.{name}")),
            "prop `{name}` must survive being documented:\n{}",
            out.js
        );
    }
    assert!(
        !out.js.contains("__props.__"),
        "a comment must never end up in a prop NAME:\n{}",
        out.js
    );

    // ...and the JSDoc reaches the CEM, joined across lines.
    let cem = out.cem.expect("CEM emitted");
    let attrs = cem["modules"][0]["declarations"][0]["attributes"].as_array().unwrap();
    let by = |n: &str| attrs.iter().find(|a| a["name"] == n).unwrap();
    assert_eq!(by("tone")["description"], "Picks the active tone");
    assert_eq!(by("size")["description"], "Multi-line. Second line.");
    assert!(
        by("plain").get("description").is_none(),
        "a `//` comment is not documentation"
    );
}
