//! M1: privilege None on product routes. Identities slot-reject is off.
//! A route without Style still compiles. Matrix fixtures stay green.

use std::path::PathBuf;

use sail_compiler::{CompileMode, CompileOpts, check_all, compile};

fn opts(path: &str) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Auto,
        source_path: Some(PathBuf::from(path)),
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn product_route_without_style_compiles() {
    let src = r#"<column gap="4"><h1>Hello</h1><p>No Style here.</p></column>"#;
    let path = "apps/sail-site/routes/docs/page.sail";
    compile(src, opts(path)).expect("route without Style must compile");
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-style-privilege")),
        "clean route must not emit privilege: {diags:?}"
    );
}

#[test]
fn product_route_name_and_geometry_compile() {
    let src = r#"<column style({ size(measure).pad(page) })><h1>x</h1></column>"#;
    let path = "apps/sail-site/routes/docs/page.sail";
    compile(src, opts(path)).expect("L3 may spend names + geometry");
}

#[test]
fn product_route_identity_slots_are_an_error() {
    let src = r#"<column style({ type({ size: xs }) })><h1>x</h1></column>"#;
    let path = "apps/sail-site/routes/docs/page.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-style-privilege")),
        "L3 slots must error: {diags:?}"
    );
}

#[test]
fn l2_size_grow_compiles() {
    let src = r#"
<scope>
  style fill = { size(grow) }
</scope>
<div style(fill)>x</div>
"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    compile(src, opts(path)).expect("L2 size(grow) must compile after M2b");
}

#[test]
fn l2_preset_style_still_compiles() {
    let src = r#"
<scope>
  style eyebrow = { type(label).fg(muted) }
</scope>
<p style(eyebrow)>Section</p>
"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    compile(src, opts(path)).expect("L2 preset Style must compile");
}

#[test]
fn l2_on_scale_pad_slots_are_legal() {
    let src = r#"<div style({ pad({ x: 4 }).bg(surface) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        !diags
            .iter()
            .any(|d| d.message.contains("is not a space step")
                || d.rule.as_deref() == Some("sail-style-privilege")),
        "on-scale pad slots must be legal on L2: {diags:?}"
    );
}

#[test]
fn l2_off_scale_pad_names_the_space_steps() {
    let src = r#"<div style({ pad({ x: 3.5 }) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    let scale: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("3.5 is not a space step"))
        .collect();
    assert_eq!(scale.len(), 1, "{diags:?}");
    assert!(
        scale[0].message.contains("3.5 is not a space step: 3 | 4"),
        "{scale:?}"
    );
    assert!(
        !scale[0].message.contains("3.5 is not a space step: 3 | 4;"),
        "scale sentence must appear once: {scale:?}"
    );
}

#[test]
fn l2_slot_style_is_an_error() {
    let src = r#"
<scope>
  style eyebrow = { type({ size: xs, weight: semibold }) }
</scope>
<p style(eyebrow)>Section</p>
"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-style-privilege")),
        "L2 slots must error: {diags:?}"
    );
}

#[test]
fn matrix_fixture_route_style_still_compiles() {
    let src = r#"
<scope>
  style shell = { pad(6).bg(background) }
</scope>
<column style(shell)><h1>button matrix</h1></column>
"#;
    let path = "crates/sail/compiler/tests/fixtures/l1-matrix/routes/button/page.sail";
    compile(src, opts(path)).expect("M1 must not change matrix fixture compile");
}
