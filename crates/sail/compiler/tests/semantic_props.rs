//! SPEC-ARIA-1 — semantic prop lowering + host anatomy + icon-only label.

use sail_compiler::parse::preprocess::preprocess as preprocess_src;
use sail_compiler::{CompileMode, CompileOpts, compile, compile_element};

fn pp(src: &str) -> String {
    preprocess_src(src).expect("preprocess")
}

fn component_opts() -> CompileOpts {
    CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn label_lowers_to_aria_label_in_emit() {
    let out = compile(
        r#"<button type="button" label="Save">Save</button>"#,
        component_opts(),
    )
    .expect("compile");
    assert!(
        out.js.contains("aria-label") && out.js.contains("Save"),
        "label must stamp aria-label:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("setAttribute(\"label\""),
        "author prop name must not remain as DOM attr:\n{}",
        out.js
    );
}

#[test]
fn current_lowers_to_aria_current() {
    let out = compile(r#"<a href="/" current="page">Home</a>"#, component_opts()).expect("compile");
    assert!(
        out.js.contains("aria-current") && out.js.contains("page"),
        "current must stamp aria-current:\n{}",
        out.js
    );
}

#[test]
fn hidden_lowers_to_aria_hidden() {
    let out = compile(r#"<span hidden="true">x</span>"#, component_opts()).expect("compile");
    assert!(
        out.js.contains("aria-hidden"),
        "hidden must stamp aria-hidden:\n{}",
        out.js
    );
}

#[test]
fn pressed_lowers_to_aria_pressed() {
    let out = compile(
        r#"<button type="button" pressed="true">On</button>"#,
        component_opts(),
    )
    .expect("compile");
    assert!(
        out.js.contains("aria-pressed"),
        "pressed must stamp aria-pressed:\n{}",
        out.js
    );
}

#[test]
fn selected_dual_stamps() {
    let out = compile(
        r#"<button type="button" selected="true">Row</button>"#,
        component_opts(),
    )
    .expect("compile");
    assert!(
        out.js.contains("selected=") || out.js.contains("selected\\="),
        "selected design attr kept:\n{}",
        out.js
    );
    assert!(
        out.js.contains("aria-selected"),
        "selected dual-stamps aria-selected:\n{}",
        out.js
    );
}

#[test]
fn icon_only_button_requires_label() {
    let err = compile(
        r#"<button type="button"><svg></svg></button>"#,
        component_opts(),
    )
    .expect_err("icon-only without label");
    assert!(
        err.message.contains("label") || err.to_string().contains("label"),
        "must name label requirement: {err}"
    );
}

#[test]
fn icon_only_button_with_label_ok() {
    compile(
        r#"<button type="button" label="Close"><svg></svg></button>"#,
        component_opts(),
    )
    .expect("icon-only with label compiles");
}

#[test]
fn navbar_anatomy_stamps_navigation_role() {
    let out = compile_element(include_str!("../stdlib/navbar/navbar.sail"), "navbar")
        .expect("navbar compiles");
    assert!(
        out.structure_html.contains(r#"role="navigation""#),
        "host anatomy must stamp role=navigation:\n{}",
        out.structure_html
    );
    assert!(
        out.manifest
            .pointer("/anatomy/host/role")
            .and_then(|v| v.as_str())
            == Some("navigation"),
        "manifest anatomy.host.role: {:?}",
        out.manifest
    );
}

#[test]
fn spinner_anatomy_stamps_status_and_label() {
    let out = compile_element(include_str!("../stdlib/spinner/spinner.sail"), "spinner")
        .expect("spinner compiles");
    assert!(
        out.structure_html.contains(r#"role="status""#),
        "spinner role:\n{}",
        out.structure_html
    );
    assert!(
        out.structure_html.contains(r#"aria-label="Loading""#),
        "spinner label:\n{}",
        out.structure_html
    );
}

#[test]
fn alert_anatomy_stamps_live_region() {
    let out = compile_element(include_str!("../stdlib/alert/alert.sail"), "alert").expect("alert");
    assert!(
        out.structure_html.contains(r#"role="status""#) && out.structure_html.contains("aria-live"),
        "alert live anatomy:\n{}",
        out.structure_html
    );
    // Dismiss uses semantic label prop → aria-label in structure
    assert!(
        out.structure_html.contains(r#"aria-label="Dismiss""#)
            || out.structure_html.contains(r#"label="Dismiss""#),
        "dismiss label:\n{}",
        out.structure_html
    );
}

#[test]
fn stdlib_sources_have_no_free_aria_attrs() {
    // Drift: stdlib .sail must not hand-write aria-* attributes
    // (semantic props / anatomy only). Free role= attrs are also banned —
    // host roles come from HOST_ANATOMY; multi-part roles from closed use: actions.
    for (tag, src) in sail_compiler::stdlib::sources() {
        let mut in_block_comment = false;
        for (i, line) in src.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("<!--") {
                in_block_comment = true;
            }
            if in_block_comment {
                if trimmed.contains("-->") {
                    in_block_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("//") {
                continue;
            }
            // Attribute form: whitespace + aria-
            if line.contains(" aria-") {
                panic!(
                    "stdlib `{tag}` line {}: free aria-* attr — use semantic props / anatomy\n  {line}",
                    i + 1
                );
            }
            // Free role= on structure (not comments, not prose about role=menu contracts).
            // Allow bare word "role" in comments already skipped; attribute form only.
            if line.contains(" role=") || line.contains("\trole=") {
                panic!(
                    "stdlib `{tag}` line {}: free role= attr — use HOST_ANATOMY or closed use: action stamp\n  {line}",
                    i + 1
                );
            }
        }
    }
}

#[test]
fn preprocess_label_on_native_button() {
    // After preprocess (stdlib stamp), author label must survive as attribute
    // that the later emit lowers — preprocess keeps author attrs.
    let out = pp(r#"<button type="button" label="Go">Go</button>"#);
    assert!(
        out.contains("label=") || out.contains("aria-label="),
        "label attr present for emit:\n{out}"
    );
}
