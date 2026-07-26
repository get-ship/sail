//! ELEMENT MODE — the L1 compile path. One authoring technology per layer:
//! element sources are normal `.sail` (a `<scope>` block, a template, and a
//! `<style>` skin); the mode changes the outputs and enforces the seam test.
//!
//! Variant paint is **CVA-in-props**: a prop whose value is a key→sx map types
//! as the key union, reflects as an attribute, and lowers through the same sx
//! atom pipeline as host `sx=` (not hand attr-keyed CSS in `<style>`). Element
//! mode keys the skin on `.sail-<name>` (light DOM; the element is the host).

use sail_compiler::compile_element;

const SPINNER: &str = r#"
<scope>
  props: {
    tone: 'default' | 'primary' | 'destructive' = 'default'
    label = 'Loading'
  }
</scope>

<style>
  .sail-spinner[tone='primary']{color:var(--primary);}
  .sail-spinner[tone='destructive']{color:var(--destructive);}
</style>

<span class="spin" role="status" aria-label="Loading">
  <span class="ring" aria-hidden="true"></span>
  <slot></slot>
</span>
"#;

#[test]
fn element_emits_structure_skin_and_manifest() {
    let el = compile_element(SPINNER, "spinner").expect("element compiles");
    assert_eq!(el.root_class, "sail-spinner");
    // Structure: single static root, root class stamped, slot preserved.
    assert!(
        el.structure_html.starts_with("<span class=\"sail-spinner spin\""),
        "root class stamped: {}",
        el.structure_html
    );
    assert!(el.structure_html.contains("<slot></slot>"), "{}", el.structure_html);
    assert!(el.structure_html.contains("class=\"ring\""), "{}", el.structure_html);
    // Skin: the variant rules survive, keyed on the root class.
    assert!(
        el.skin_css.contains(".sail-spinner[tone='primary']{color:var(--primary);}"),
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
fn element_rejects_the_sui_component_namespace() {
    // Hyphens are FINE — `tab-list`, `menu-item` and `context-menu` are all L1
    // elements. What is not fine is the `sail-` prefix: that is the legacy component
    // namespace L1 replaces, and the compiler stamps its own `sail-` onto the root
    // class, so letting one through produced `.sail-sail-spinner`.
    let err = compile_element("<span class=\"x\">y</span>", "sail-spinner").unwrap_err();
    assert!(
        format!("{err:?}").contains("component namespace"),
        "a sail-prefixed name is a component, not an element: {err:?}"
    );
    // ...and a hyphenated language element still compiles.
    compile_element("<span class=\"x\">y</span>", "tab-list").expect("hyphenated L1 name is legal");
}

#[test]
fn element_skins_css_matches_registry() {
    // The materialized framework stylesheet half must equal the registry's
    // live derivation — regenerate with `cargo run -p sail-compiler --
    // framework-css` when this fails (the stdlib registry is the SSOT).
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/element-skins.css"
    );
    let file = std::fs::read_to_string(path).expect("element-skins.css must exist");
    // Strip the generated banner (through the first `*/\n`) and the `@layer
    // sail { … }` wrapper, then compare the rule body to the registry.
    let after_banner = &file[file.find("*/\n").expect("banner present") + 3..];
    let inner = after_banner
        .strip_prefix("@layer sail {\n")
        .and_then(|s| s.strip_suffix("}\n"))
        .expect("element-skins.css is wrapped in @layer sail");
    assert_eq!(
        inner,
        sail_compiler::stdlib::skins_css(),
        "element-skins.css is stale — run: cargo run -p sail-compiler -- framework-css"
    );
}

#[test]
fn ticker_loops_via_two_slots_and_marquee_action() {
    let src = sail_compiler::stdlib::sources()
        .iter()
        .find(|(tag, _)| *tag == "ticker")
        .map(|(_, src)| *src)
        .expect("ticker registered in stdlib SOURCES");
    let el = compile_element(src, "ticker").expect("ticker compiles");
    assert_eq!(el.root_class, "sail-ticker");
    assert_eq!(
        el.structure_html.matches("<slot></slot>").count(),
        2,
        "two independently-filled slots build the seamless-loop duplicate track: {}",
        el.structure_html
    );
    // Track animation is use:marquee (fx:loop), not contracts PE.
    assert!(
        el.structure_html.contains("use:marquee"),
        "track stamps use:marquee for duration/reverse-driven fx: {}",
        el.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains("var(--animate-ticker)")
            && !contracts.contains("sail-ticker-scroll")
            && !contracts
                .lines()
                .any(|l| l.contains("sail-ticker") && l.contains("animation")),
        "ticker animation PE must be gone from contracts: {contracts}"
    );
    // Layout track: nowrap row + w-max on structure
    assert!(
        el.structure_html.contains("sail-nowrap") || el.skin_css.contains("nowrap"),
        "track is a nowrap row: struct={} skin={}",
        el.structure_html,
        el.skin_css
    );
    let attrs = el.manifest["attributes"].as_array().unwrap();
    assert!(attrs.iter().any(|a| a["name"] == "duration"), "{}", el.manifest);
    assert!(attrs.iter().any(|a| a["name"] == "reverse"), "{}", el.manifest);
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

#[test]
fn stdlib_elements_can_self_nest() {
    // card-in-card: use-site lowering must find the DEPTH-MATCHED close, not
    // the first </card>. Regression — after card was promoted to L1 the old
    // `find("</card>")` paired the inner close with the outer card and broke
    // the /components showcase (a card of cards is an essential pattern).
    use sail_compiler::{compile, CompileMode, CompileOpts};
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    // Multibyte chars (⌘/—/°) around the nested tags — the depth scan walks
    // byte-by-byte, so it must compare on BYTES; slicing the &str mid-char
    // would panic (the real-page regression that killed the daemon build).
    let out = compile("<card>⌘ <card>— inner °</card> tail</card>", opts)
        .expect("nested cards with multibyte chars compile");
    // Both cards lowered — the card root class appears for each.
    let n = out.js.matches("sail-card").count();
    assert!(n >= 2, "expected both cards lowered, got {n}:\n{}", out.js);
}

#[test]
fn element_rejects_controller_block() {
    // `<controller>` is the L2-component behavior channel. An element carries
    // behavior as stacked `use:<behavior>` stamps on its static structure —
    // an inline block is a targeted compile error, not a captured module.
    let src = r#"
<scope>
  props: { selected = false }
</scope>
<style>
  .sail-chip { display: inline-flex; }
</style>
<controller>
export default function chip(node: Element) {
  (node as HTMLElement).setAttribute('role', 'button')
}
</controller>
<span>label</span>
"#;
    let err = compile_element(src, "chip").unwrap_err();
    assert!(
        format!("{err:?}").contains(
            "element mode: `<controller>` is not supported — attach behavior with `use:<behavior>`."
        ),
        "targeted rejection: {err:?}"
    );
}

#[test]
fn stdlib_chip_compiles_with_use_stamps() {
    let src = include_str!("../stdlib/chip.sail");
    let el = compile_element(src, "chip").expect("stdlib chip compiles");
    for stamp in ["use:pressable", "use:pressed", "use:dismissible"] {
        assert!(
            el.structure_html.contains(stamp),
            "{stamp} on chip root: {}",
            el.structure_html
        );
    }
    assert!(
        !el.structure_html.contains("data-sail"),
        "no discovery stamp: {}",
        el.structure_html
    );
    assert!(
        el.manifest.get("controller").is_none()
            && el.manifest.get("controllerDiscovery").is_none(),
        "no controller keys in manifest: {}",
        el.manifest
    );
    assert_eq!(el.root_class, "sail-chip");
}

#[test]
fn stdlib_element_sources_have_no_author_style_blocks() {
    // L1 elements author structure + sx + layout + capability use: only.
    // Paint is host sx + CVA-in-props (and contracts for shared tone/size tokens).
    for (tag, src) in sail_compiler::stdlib::sources() {
        let mut in_comment = false;
        let bytes = src.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if !in_comment && i + 4 <= bytes.len() && &bytes[i..i + 4] == b"<!--" {
                in_comment = true;
                i += 4;
                continue;
            }
            if in_comment && i + 3 <= bytes.len() && &bytes[i..i + 3] == b"-->" {
                in_comment = false;
                i += 3;
                continue;
            }
            if !in_comment && i + 7 <= bytes.len() && &bytes[i..i + 7] == b"<style>" {
                panic!(
                    "stdlib element {tag} still has an author <style> block — relocate to residues/sx/contracts"
                );
            }
            i += 1;
        }
    }
}

#[test]
fn native_label_folds_host_sx_into_tag_rule() {
    // Native <label> never stamps host classes at use sites — host sx must
    // land as tag-keyed decls so font-(--label-weight) paints every bare label.
    let out = sail_compiler::compile_element(
        include_str!("../stdlib/label.sail"),
        "label",
    )
    .expect("label compiles");
    assert!(out.native, "label is native-kind");
    assert!(
        out.skin_css.contains("font-weight:var(--label-weight)")
            || out.skin_css.contains("font-weight: var(--label-weight)"),
        "weight channel must be on the tag rule:\n{}",
        out.skin_css
    );
    // Prefer tag-keyed host rule over orphaned atomic that never stamps.
    assert!(
        out.skin_css.contains("label {") || out.skin_css.contains("label{"),
        "native skin keys on bare tag:\n{}",
        out.skin_css
    );
}