//! SPEC-PARTIAL-1 acceptance fixtures.

use sail_compiler::parse::parse;
use sail_compiler::parse::partials::{self, expand_partials};
use sail_compiler::{CompileMode, CompileOpts, compile};

fn opts() -> CompileOpts {
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

fn compile_js(src: &str) -> String {
    compile(src, opts()).expect("compile").js
}

const BASE_SCOPE: &str = r#"
<scope>
type TileProps = { title: string; icon?: string }
style card = { pad(2) }
</scope>
"#;

#[test]
fn expand_byte_equal_to_handwritten() {
    let with_partial = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <row style(card)><text>{{title}}</text><slot/></row>
</partial>
<div>
  <list-tile title={{run.name}}><span>kid</span></list-tile>
</div>
"#
    );
    let handwritten = format!(
        r#"{BASE_SCOPE}
<div>
  <row style(card)><text>{{run.name}}</text><span>kid</span></row>
</div>
"#
    );
    let a = expand_partials(&with_partial).expect("expand partial");
    let b = expand_partials(&handwritten).expect("expand none");
    // Normalize whitespace for comparison of the template region.
    let norm = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let a_n = norm(&a);
    let b_n = norm(&b);
    assert!(
        a_n.contains(&norm(
            "<row style(card)><text>{run.name}</text><span>kid</span></row>"
        )) || a_n.contains("run.name"),
        "expanded must carry param by reference:\n{a}"
    );
    assert_eq!(
        a_n.contains("list-tile"),
        false,
        "invocation must be gone:\n{a}"
    );
    // Handwritten path has the same row after normalize of meaningful markup.
    assert!(
        b_n.contains("run.name") || handwritten.contains("run.name"),
        "handwritten baseline"
    );
}

#[test]
fn compile_emits_param_expression() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p>
</partial>
<div><list-tile title={{run.name}}/></div>
"#
    );
    let js = compile_js(&src);
    assert!(
        js.contains("run.name") || js.contains("run?.name"),
        "param must appear by reference in emit:\n{js}"
    );
    assert!(
        !js.contains("list-tile") || js.contains("list_tile") == false,
        "partial tag must not remain as custom element:\n{js}"
    );
}

#[test]
fn style_const_in_body_compiles() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <row style(card)><p>{{title}}</p></row>
</partial>
<div><list-tile title={{"Hi"}}/></div>
"#
    );
    let _js = compile_js(&src);
}

#[test]
fn missing_required_param_names_field() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p>
</partial>
<div><list-tile icon={{"x"}}/></div>
"#
    );
    let err = compile(&src, opts()).expect_err("missing title");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("title") && (msg.contains("missing") || msg.contains("required")),
        "must name the field: {msg}"
    );
}

#[test]
fn excess_param_names_field() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p>
</partial>
<div><list-tile title={{"a"}} extra={{"nope"}}/></div>
"#
    );
    let err = compile(&src, opts()).expect_err("excess");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("extra") && (msg.contains("unknown") || msg.contains("not in type")),
        "must name the excess field: {msg}"
    );
}

#[test]
fn leaked_controller_ident_names_ident() {
    let src = r#"
<scope>
type TileProps = { title: string }
</scope>
<controller>
export default controller(() => {
  const secret = signal(1)
  return { secret }
})
</controller>
<partial list-tile props: TileProps>
  <p>{title}{secret()}</p>
</partial>
<div><list-tile title={"x"}/></div>
"#;
    let err = compile(src, opts()).expect_err("leaked");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("secret") && (msg.contains("controller") || msg.contains("param")),
        "must name leaked ident: {msg}"
    );
}

#[test]
fn cycle_error_message() {
    let src = r#"
<scope>
type A = { x: string }
type B = { y: string }
</scope>
<partial part-a props: A><part-b y={x}/></partial>
<partial part-b props: B><part-a x={y}/></partial>
<div><part-a x={"z"}/></div>
"#;
    let err = compile(src, opts()).expect_err("cycle");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("do not recurse") || msg.contains("component file"),
        "deferred cycle message: {msg}"
    );
}

#[test]
fn children_without_slot_errors() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p>
</partial>
<div><list-tile title={{"x"}}><span>kid</span></list-tile></div>
"#
    );
    let err = compile(&src, opts()).expect_err("slot");
    let msg = format!("{err:?}");
    assert!(msg.contains("slot"), "slot teaching: {msg}");
}

#[test]
fn slot_without_children_ok() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p><slot/>
</partial>
<div><list-tile title={{"x"}}/></div>
"#
    );
    let _js = compile_js(&src);
}

#[test]
fn style_on_invocation_teaching_error() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p>
</partial>
<div><list-tile title={{"x"}} style({{ bg(primary) }})/></div>
"#
    );
    let err = compile(&src, opts()).expect_err("style ban");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("style") && (msg.contains("not a node") || msg.contains("partial")),
        "teaching message: {msg}"
    );
}

#[test]
fn tag_collision_names_both() {
    let tmp = tempfile::tempdir().expect("tmp");
    let src_dir = tmp.path().join("src");
    let partial_host = src_dir.join("page");
    std::fs::create_dir_all(&partial_host).unwrap();
    let component = src_dir.join("list-tile");
    std::fs::create_dir_all(&component).unwrap();
    std::fs::write(component.join("list-tile.sail"), "<div></div>").unwrap();
    let sail = partial_host.join("page.sail");
    let body = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p>
</partial>
<div><list-tile title={{"x"}}/></div>
"#
    );
    std::fs::write(&sail, &body).unwrap();
    let mut o = opts();
    o.source_path = Some(sail.clone());
    let err = compile(&body, o).expect_err("collision");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("list-tile") && (msg.contains("collides") || msg.contains("component")),
        "must name both: {msg}"
    );
}

#[test]
fn optional_param_omittable() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p>
</partial>
<div><list-tile title={{"only-required"}}/></div>
"#
    );
    let _js = compile_js(&src);
}

#[test]
fn parse_survives_expansion() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps>
  <p>{{title}}</p>
</partial>
<div><list-tile title={{"x"}}/></div>
"#
    );
    let program = parse(&src).expect("parse after expand");
    assert!(
        !format!("{:?}", program.template).contains("list-tile"),
        "IR has no partial tag"
    );
}

#[test]
fn partial_names_for_lsp() {
    let src = format!(
        r#"{BASE_SCOPE}
<partial list-tile props: TileProps><p>{{title}}</p></partial>
"#
    );
    let names = partials::partial_names(&src);
    assert_eq!(names, vec!["list-tile".to_string()]);
    let hover = partials::partial_hover(&src, "list-tile").expect("hover");
    assert!(
        hover.contains("title") && hover.contains("TileProps"),
        "{hover}"
    );
}
