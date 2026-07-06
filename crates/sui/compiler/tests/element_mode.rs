//! ELEMENT MODE — the L1 compile path. One authoring technology per layer:
//! element sources are normal `.sui` (component block, template, variants/sx
//! skin); the mode changes the outputs and enforces the seam test.

use sui_compiler::compile_element;

const SPINNER: &str = r#"
<component>{
  props: {
    tone: 'default' | 'primary' | 'destructive',
    label: 'Loading',
  },
  variants: {
    tone: { primary: 'fg:primary', destructive: 'fg:destructive' },
  },
}</component>

<span class="spin" role="status" aria-label="Loading">
  <span class="ring" aria-hidden="true"></span>
  <slot></slot>
</span>
"#;

#[test]
fn element_emits_structure_skin_and_manifest() {
    let el = compile_element(SPINNER, "spinner").expect("element compiles");
    assert_eq!(el.root_class, "sui-spinner");
    // Structure: single static root, root class stamped, slot preserved.
    assert!(
        el.structure_html.starts_with("<span class=\"sui-spinner spin\""),
        "root class stamped: {}",
        el.structure_html
    );
    assert!(el.structure_html.contains("<slot></slot>"), "{}", el.structure_html);
    assert!(el.structure_html.contains("class=\"ring\""), "{}", el.structure_html);
    // Skin: variants lowered and re-keyed from :host to the root class.
    assert!(
        el.skin_css.contains(".sui-spinner[tone='primary']{color:var(--primary);}"),
        "skin keyed on root class: {}",
        el.skin_css
    );
    assert!(!el.skin_css.contains(":host"), "no :host survives: {}", el.skin_css);
    // Manifest: contract + anatomy.
    let m = &el.manifest;
    assert_eq!(m["kind"], "element");
    assert_eq!(m["tag"], "spinner");
    assert_eq!(m["anatomy"]["dom"], "light");
    let attrs = m["attributes"].as_array().unwrap();
    assert!(attrs.iter().any(|a| a["name"] == "tone"), "{m}");
    assert!(
        attrs.iter().any(|a| a["name"] == "label" && a["default"] == "Loading"),
        "default literal rides the manifest: {m}"
    );
}

#[test]
fn element_rejects_control_flow_as_component() {
    let src = "<span class=\"x\">{#if open()}<b>y</b>{/if}</span>";
    let err = compile_element(src, "toggle").unwrap_err();
    assert!(
        format!("{err:?}").contains("component"),
        "the seam-test error names it a component: {err:?}"
    );
}

#[test]
fn element_rejects_dynamic_attributes() {
    let src = "<span class=\"x\" aria-label={label()}>y</span>";
    let err = compile_element(src, "chip").unwrap_err();
    assert!(format!("{err:?}").contains("dynamic attribute"), "{err:?}");
}

#[test]
fn element_rejects_hyphenated_names() {
    let err = compile_element("<span class=\"x\">y</span>", "sui-spinner").unwrap_err();
    assert!(
        format!("{err:?}").contains("bare lowercase"),
        "hyphenated tag = custom element = component: {err:?}"
    );
}
