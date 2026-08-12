//! Behavioral tests for structural `<when {token}>` blocks.

use sail_compiler::parse::parse;
use sail_compiler::{CompileMode, CompileOpts, compile};

fn opts() -> CompileOpts {
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

fn compile_js(src: &str) -> String {
    compile(src, opts()).expect("compile").js
}

#[test]
fn parses_when_else_and_nests_in_if() {
    let src = r#"
<div>
  <when {tablet}>
    <p>wide</p>
    {else => 
      <if {open()}><span>narrow-open</span></if>
    }
  </when>
</div>
"#;
    let program = parse(src).expect("parse");
    assert!(
        matches!(
            program.template.iter().find(|i| matches!(i, sail_compiler::ir::Instr::Element(_))),
            Some(sail_compiler::ir::Instr::Element(el)) if el.children.iter().any(|c| matches!(c, sail_compiler::ir::Instr::When(_)))
        ),
        "expected When inside div"
    );
}

#[test]
fn emits_bind_when_with_media_query() {
    let js = compile_js(r#"<div><when {tablet}><p>wide</p>{else => <p>narrow</p>}</when></div>"#);
    assert!(js.contains("bindWhen"), "emit must call bindWhen:\n{js}");
    assert!(
        js.contains("(min-width: 768px)") || js.contains("min-width: 768px"),
        "tablet media query must appear:\n{js}"
    );
    assert!(
        js.contains("then_") && js.contains("else_"),
        "both branches must be emitted:\n{js}"
    );
}

#[test]
fn rejects_raw_px() {
    let err = compile(r#"<div><when {768px}><p>x</p></when></div>"#, opts())
        .expect_err("raw px must fail");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("rejects") || msg.contains("breakpoint") || msg.contains("768"),
        "error should refuse raw px: {msg}"
    );
}

#[test]
fn rejects_unknown_token_and_names_scale() {
    let err = compile(r#"<div><when {bp-md}><p>x</p></when></div>"#, opts())
        .expect_err("unknown token must fail");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("mobile") && msg.contains("tablet") && msg.contains("desktop"),
        "error must name the theme breakpoint list: {msg}"
    );
}

#[test]
fn nests_inside_for() {
    let js = compile_js(
        r#"
<div>
  <for {items() as item} key={item.id}>
    <when {desktop}><span>{item.id}</span>{else => <em>{item.id}</em>}</when>
  </for>
</div>
"#,
    );
    assert!(js.contains("bindWhen"), "nested when must emit:\n{js}");
    assert!(js.contains("bindFor"), "for still emits:\n{js}");
}

#[test]
fn when_without_else_emits_then_only() {
    let js = compile_js(r#"<div><when {mobile}><p>only-wide-ish</p></when></div>"#);
    assert!(js.contains("bindWhen"), "must emit bindWhen:\n{js}");
    assert!(
        !js.contains("else_:") && !js.contains("else_: "),
        "no else branch when omitted:\n{js}"
    );
}
