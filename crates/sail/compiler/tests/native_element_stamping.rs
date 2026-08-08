//! Native-kind stdlib element stamping — a stdlib element whose template
//! root tag EQUALS the element name (label → `<label>`) is never spliced
//! (the tag already IS the element), but its host paint — the `.sail-<name>`
//! root class plus the atomic `s-*` classes its host Style lowered to,
//! the same string `serialize_static` bakes into a spliced root — must be
//! STAMPED onto the use-site tag. Before this pass the skin's atomic rules
//! sat in `element-skins.css` with no class ever carrying them: inert CSS.

use sail_compiler::parse::preprocess::preprocess as preprocess_src;
use sail_compiler::{CompileMode, CompileOpts, compile};

fn pp(src: &str) -> String {
    preprocess_src(src).expect("preprocess")
}

fn component_opts() -> CompileOpts {
    CompileOpts {
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
fn native_label_stamps_host_classes_at_use_site() {
    let out = pp(r#"<label>Email</label>"#);
    assert!(
        out.contains("<label class=\"sail-label "),
        "root class stamped onto the real tag:\n{out}"
    );
    assert!(
        out.contains(r#"data-sail-host="label""#),
        "host identity attr stamped:\n{out}"
    );
    // The element's atomic host sx classes ride along (label.sail authors
    // cursor(pointer) + fg(label-foreground) on the host).
    assert!(
        out.contains("s-cursorpointer") && out.contains("s-fglabelforeground"),
        "atomic host sx classes stamped:\n{out}"
    );
    // No splice: the element's internals (the .req marker span from
    // label.sail's template) must NOT appear at the use site.
    assert!(
        !out.contains("class=\"req\""),
        "native kind stamps, never splices:\n{out}"
    );
}

#[test]
fn native_label_merges_author_class() {
    let out = pp(r#"<label class="hint">Email</label>"#);
    assert!(
        out.contains("sail-label") && out.contains("hint"),
        "author class merges with the stamp:\n{out}"
    );
    assert_eq!(
        out.matches("class=").count(),
        1,
        "one merged class attribute, not two:\n{out}"
    );
}

#[test]
fn native_label_keeps_props_as_attributes() {
    // The skin's match() rules key on `[size='sm']`-style attribute
    // selectors — use-site props must survive stamping as plain attributes.
    let out = pp(r#"<label size="sm" required>Email</label>"#);
    assert!(out.contains("sail-label"), "host classes stamped:\n{out}");
    assert!(
        out.contains(r#"size="sm""#) && out.contains("required"),
        "props remain attributes for the skin's [attr=value] selectors:\n{out}"
    );
}

#[test]
fn native_label_stamping_is_idempotent() {
    // An already-stamped tag (author wrote the classes, or a re-scan) must
    // not accumulate duplicates.
    let out =
        pp(r#"<label class="sail-label s-cursorpointer" data-sail-host="label">Email</label>"#);
    assert_eq!(
        out.matches("sail-label").count(),
        1,
        "root class not duplicated:\n{out}"
    );
    assert_eq!(
        out.matches("s-cursorpointer").count(),
        1,
        "atomic class not duplicated:\n{out}"
    );
    assert_eq!(
        out.matches(r#"data-sail-host="label""#).count(),
        1,
        "host identity not duplicated:\n{out}"
    );
}

#[test]
fn native_label_dynamic_class_rides_expression() {
    // A dynamic class binding REPLACES the attribute at runtime — the stamp
    // must ride inside the expression or it is wiped on the first update.
    let out = pp(r#"<label class={cls}>Email</label>"#);
    assert!(
        out.contains("sail-label") && out.contains("+ (cls)"),
        "stamped classes join the dynamic expression:\n{out}"
    );
}

#[test]
fn anatomy_marked_subtree_is_untouched() {
    // `[anatomy]` marks element internals rendered elsewhere — their paint
    // already rides their own classes; host paint on top would apply twice.
    // The whole subtree (including a nested plain label) passes through
    // unstamped; a later sibling use still stamps.
    let out = pp(r#"<label anatomy><label>inner</label></label><label>outer</label>"#);
    let anatomy_end = out.find("</label>").expect("anatomy label closes");
    let (anatomy_region, rest) = out.split_at(anatomy_end);
    assert!(
        !anatomy_region.contains("sail-label"),
        "anatomy subtree unstamped:\n{out}"
    );
    assert!(
        rest.contains("sail-label"),
        "sibling use still stamps:\n{out}"
    );
}

#[test]
fn spliced_element_behavior_unchanged() {
    // badge is a SPLICED element — its structure snippet still inlines with
    // the root class on the snippet root, exactly as before.
    let out = pp(r#"<badge variant="solid">New</badge>"#);
    assert!(
        out.contains("sail-badge"),
        "spliced element still stamps its snippet root:\n{out}"
    );
    assert!(
        out.contains(r#"data-sail-host="badge""#),
        "spliced element stamps host identity:\n{out}"
    );
    assert!(
        out.contains(">New<"),
        "children still project into the slot:\n{out}"
    );
}

#[test]
fn spliced_tab_stamps_host_identity() {
    // Action discovery keys off data-sail-host, not the CSS class stamp.
    let out = pp(r#"<tab name="a">Profile</tab>"#);
    assert!(
        out.contains(r#"data-sail-host="tab""#),
        "tab host identity for actions:\n{out}"
    );
    assert!(
        out.contains("sail-tab"),
        "CSS host class still present:\n{out}"
    );
}

#[test]
fn native_label_stamps_end_to_end_through_compile() {
    // Full pipeline: the stamped classes must survive parse + emit and land
    // on the real DOM element the component builds.
    let r = compile(r#"<label size="sm">Email</label>"#, component_opts()).expect("compile");
    assert!(
        r.js.contains("sail-label"),
        "stamped classes reach the emitted DOM:\n{}",
        r.js
    );
    assert!(
        r.js.contains("data-sail-host") && r.js.contains("label"),
        "host identity reaches the emitted DOM:\n{}",
        r.js
    );
    assert!(
        r.js.contains("size=") && r.js.contains("sm"),
        "use-site prop survives as an attribute for the skin:\n{}",
        r.js
    );
}
