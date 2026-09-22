#[test]
fn popover_structure_is_native_popover() {
    let out =
        sail_compiler::compile_element(include_str!("../stdlib/popover/popover.sail"), "popover")
            .expect("popover compiles");
    assert!(
        out.structure_html.contains("popover"),
        "popover structure keeps the native popover attr; got:\n{}",
        out.structure_html
    );
}

#[test]
fn page_with_popover_compiles() {
    let src = r#"
<button popovertarget="p">Open</button>
<popover id="p">Hello</popover>
"#;
    sail_compiler::compile(src, sail_compiler::CompileOpts::default()).expect("page compiles");
}

/// Regression: bindIf/for/when wire helpers used to name their Comment
/// parameter `anchor`, shadowing the imported `use:anchor` action so
/// `runAction(el, anchor)` inside a branch received a Comment and
/// popovers never positioned.
#[test]
fn if_branch_wire_param_does_not_shadow_use_anchor() {
    let src = r#"
<controller>
export default controller(() => {
  const open = signal(true)
  return { open }
})
</controller>
<if {open()}>
  <button popovertarget="p">Open</button>
  <popover id="p">Hello</popover>
</if>
"#;
    let out =
        sail_compiler::compile(src, sail_compiler::CompileOpts::default()).expect("page compiles");
    assert!(
        out.js.contains("cfAnchor") || out.js.contains("anchor: cfAnchor"),
        "wireBoundary param must not be bare `anchor` (shadows use:anchor); js:\n{}",
        out.js
    );
    // Must not be `(anchor) => bindIf({` with shorthand `anchor,` alone
    // as the only form — that was the shadowing bug.
    assert!(
        !out.js.contains("(anchor) => bindIf({"),
        "bindIf wire must not use param name `anchor`: {}",
        out.js
    );
}
