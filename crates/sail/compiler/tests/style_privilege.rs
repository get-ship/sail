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
fn bare_full_is_not_the_flex_trap() {
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    for src in [
        r#"<column style(full)>x</column>"#,
        r#"<column style({ play(full) })>x</column>"#,
    ] {
        let diags = check_all(src, Some(path.as_ref()), &[], None);
        assert!(
            !diags
                .iter()
                .any(|d| d.message.contains("`size(full)` is a DTCG CSS trap")),
            "bare `full` must stay generic unknown-vocabulary, not flex-trap: {src} {diags:?}"
        );
    }
}

#[test]
fn size_full_is_a_flex_trap() {
    let src = r#"<row><column style({ size(full) })>x</column></row>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        diags.iter().any(|d| {
            d.message.contains("`size(full)` is a DTCG CSS trap")
                && d.message.contains("size(grow)")
                && d.message.contains("size(max)")
        }),
        "size(full) must point at grow/max: {diags:?}"
    );
}

#[test]
fn size_slot_full_is_retired() {
    let src = r#"<div style({ size({ w: full }) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        diags.iter().any(|d| d
            .message
            .contains("size.full retired — use size(grow)|size(max) or free 100%")),
        "w: full must teach retirement: {diags:?}"
    );
}

#[test]
fn size_free_percent_compiles() {
    let src = r#"<div style({ size({ w: 100% }) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    compile(src, opts(path)).expect("size({{ w: 100% }}) is free geometry");
}

#[test]
fn radius_full_compiles() {
    let src = r#"<div style({ radius(full).bg(primary) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    compile(src, opts(path)).expect("radius(full) stays a pill");
}

#[test]
fn size_grow_outside_host_is_an_error() {
    let src = r#"<column style({ size(grow) })>x</column>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        diags.iter().any(|d| {
            d.rule.as_deref() == Some("sail-grow-outside-host")
                && d.message
                    .contains("`size(grow)` shares leftover inside a host")
        }),
        "grow outside a host must error: {diags:?}"
    );
}

#[test]
fn size_grow_inside_row_is_legal() {
    let src = r#"
<row>
  <column style({ size(hug) })>hug</column>
  <column style({ size(grow) })>grow</column>
</row>
"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-grow-outside-host")),
        "grow inside a host must be legal: {diags:?}"
    );
}

#[test]
fn l2_size_grow_compiles() {
    let src = r#"
<scope>
  style fill = { size(grow) }
</scope>
<row>
  <div style(fill)>x</div>
</row>
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
fn l2_on_scale_margin_slots_are_legal() {
    let src = r#"<div style({ margin({ t: 4 }).bg(primary) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        !diags
            .iter()
            .any(|d| d.message.contains("is not a space step")
                || d.rule.as_deref() == Some("sail-style-privilege")),
        "on-scale margin slots must be legal on L2: {diags:?}"
    );
}

#[test]
fn l2_position_slots_are_legal() {
    let src = r#"<div style({ position({ scheme: absolute, t: 4, l: 4 }).bg(accent) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-style-privilege")),
        "position slots must be legal on L2: {diags:?}"
    );
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
fn l2_off_scale_margin_names_the_space_steps() {
    let src = r#"<div style({ margin({ t: 3.5 }) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    let scale: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("3.5 is not a space step"))
        .collect();
    assert_eq!(scale.len(), 1, "{diags:?}");
    assert!(
        scale[0].message.contains(
            "3.5 is not a space step: did you mean 3 | 4 — or is this an exception: raw(3.5)?"
        ),
        "{scale:?}"
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
        scale[0].message.contains(
            "3.5 is not a space step: did you mean 3 | 4 — or is this an exception: raw(3.5)?"
        ),
        "{scale:?}"
    );
    assert!(
        !scale[0].message.contains("3.5 is not a space step: 3 | 4;"),
        "scale sentence must appear once: {scale:?}"
    );
}

#[test]
fn l2_raw_pad_compiles_silently() {
    let src = r#"<div style({ pad({ x: raw(3.5) }) })>x</div>"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    compile(src, opts(path)).expect("raw() must compile silently");
    let diags = check_all(src, Some(path.as_ref()), &[], None);
    assert!(
        !diags
            .iter()
            .any(|d| d.message.contains("is not a space step")),
        "raw() must not emit a scale sentence: {diags:?}"
    );
    let lowered = sail_compiler::parse::preprocess::preprocess(src).expect("raw() must lower");
    assert!(
        lowered.contains("3.5px"),
        "numeric raw in a length slot lowers as px: {lowered}"
    );
}

#[test]
fn named_const_off_scale_fires_scale_sentence_at_declaration() {
    let src = r#"
<scope>
  style base = { pad(page) }
  style box = { pad({ x: 3.5 }) }
</scope>
<column style(base)>x</column>
"#;
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    let err = compile(src, opts(path)).expect_err("off-scale named const is terminal");
    let err_s = format!("{err:?}");
    assert!(
        err_s.contains("3.5 is not a space step") && err_s.contains("raw(3.5)"),
        "compile must carry the two-doors sentence: {err_s}"
    );
    assert!(
        !err_s.contains("unknown vocabulary `base`"),
        "must not poison sibling `base`: {err_s}"
    );

    let diags = check_all(src, Some(path.as_ref()), &[], None);
    let joined = diags
        .iter()
        .map(|d| format!("{} @{}:{}", d.message, d.line, d.col))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("3.5 is not a space step") && joined.contains("raw(3.5)"),
        "check_all must carry the two-doors sentence: {joined}"
    );
    assert!(
        !joined.contains("unknown vocabulary `base`"),
        "must not poison sibling `base`: {joined}"
    );
    assert!(
        diags
            .iter()
            .any(|d| { d.message.contains("3.5 is not a space step") && d.line == 4 }),
        "scale sentence must sit on `style box`: {joined}"
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

#[test]
fn identities_open_paint_forms_are_legal() {
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    for src in [
        r#"<div style({ bg({ gradient: primary }) })>x</div>"#,
        r#"<div style({ bg({ ref: primary }) })>x</div>"#,
        r#"<div style({ bg({ gradient : primary }) })>x</div>"#,
    ] {
        let diags = check_all(src, Some(path.as_ref()), &[], None);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule.as_deref() == Some("sail-style-privilege")),
            "opened paint form must be legal: {src} {diags:?}"
        );
    }
}

#[test]
fn identities_closed_paint_slots_stay_red() {
    let path = "apps/studio/src/studio-panel/studio-panel.sail";
    for src in [
        r#"<div style({ bg({ color: primary }) })>x</div>"#,
        r#"<p style({ type({ size: xs }) })>x</p>"#,
        r#"<div style({ filter({ opacity: 50 }) })>x</div>"#,
    ] {
        let diags = check_all(src, Some(path.as_ref()), &[], None);
        assert!(
            diags
                .iter()
                .any(|d| d.rule.as_deref() == Some("sail-style-privilege")),
            "closed slot must stay privilege: {src} {diags:?}"
        );
    }
}
