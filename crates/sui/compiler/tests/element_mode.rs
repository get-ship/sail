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

#[test]
fn element_skins_css_matches_registry() {
    // The materialized framework stylesheet half must equal the registry's
    // live derivation — regenerate with `cargo run -p sui-compiler --
    // framework-css` when this fails (the stdlib registry is the SSOT).
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sui/tokens/src/element-skins.css"
    );
    let file = std::fs::read_to_string(path).expect("element-skins.css must exist");
    // Strip the generated banner (everything through the first `*/\n`).
    let body = &file[file.find("*/\n").expect("banner present") + 3..];
    assert_eq!(
        body,
        sui_compiler::stdlib::skins_css(),
        "element-skins.css is stale — run: cargo run -p sui-compiler -- framework-css"
    );
}

#[test]
fn element_void_root_takes_no_closing_tag() {
    // separator's root is <hr> — a void element. The structure must NOT
    // carry </hr> (invalid HTML the template parser rejects when the lowered
    // structure is spliced into a page — the /language showcase find).
    let out = compile_element("<hr class=\"sep\">", "separator").unwrap();
    assert!(out.structure_html.contains("<hr"), "{}", out.structure_html);
    assert!(!out.structure_html.contains("</hr>"), "void: {}", out.structure_html);
}

#[test]
fn element_void_with_children_is_an_error() {
    // A void root with content is rejected — the template parser catches the
    // stray </hr> first (belt to the emitter's void-children suspenders).
    assert!(compile_element("<hr class=\"x\">nope</hr>", "sep").is_err());
}
