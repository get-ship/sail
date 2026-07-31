//! The SFC contract at the `compile()`/`check_all()` boundary: the inline
//! `<controller>` block IS the scope the template is checked against, and a
//! diagnostic's line/column point at the row the author wrote.
//!
//! Both properties failed together once. `compile()` blanks the block itself, so
//! a caller that pre-stripped and passed the stripped copy handed it a source
//! with no block — every controller-provided name read as undeclared, and the
//! whole template was reported N lines above its true position, N being the size
//! of the block.

use std::path::PathBuf;

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

const SFC: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const phase = signal('idle')
  return { phase }
})
</controller>

<b>build</b>
<span>{phase()}</span>
"#;

fn opts(name: &str) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: Some(format!("./{name}.sail.controller.ts")),
        mode: CompileMode::Component,
        source_path: Some(PathBuf::from(format!("/x/{name}.sail"))),
        component_meta: Some(ComponentMeta {
            name: name.to_string(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn the_inline_block_is_the_templates_scope() {
    compile(SFC, opts("sfc-a")).expect("a name the <controller> returns resolves in the template");
}

#[test]
fn an_undeclared_name_still_fails() {
    let src = SFC.replace("{phase()}", "{nope()}");
    let err = compile(&src, opts("sfc-b")).unwrap_err();
    assert!(err.message.contains("'nope'"), "{}", err.message);
    // The scope came from the block, so the message can name what IS available.
    assert!(err.message.contains("phase"), "{}", err.message);
}

#[test]
fn diagnostic_lines_point_at_the_authored_row() {
    let src = SFC.replace("{phase()}", "{nope()}");
    let err = compile(&src, opts("sfc-c")).unwrap_err();
    let rendered = err.render(&src);
    // `<span>{nope()}</span>` is line 10 of the source above — under the block,
    // not shifted up by it.
    assert!(
        rendered.contains("line 10"),
        "expected the error on the row the author wrote, got: {rendered}"
    );
}

#[test]
fn blanking_preserves_every_offset() {
    let (template, body) = sail_compiler::parse::preprocess::extract_controller_block(SFC);
    assert!(body.is_some());
    assert_eq!(
        template.len(),
        SFC.len(),
        "the template must not shift the source's bytes"
    );
    assert_eq!(
        template.lines().count(),
        SFC.lines().count(),
        "the template must not shift the source's lines"
    );
    assert!(
        !template.contains("signal('idle')"),
        "the block's body must not reach the parser"
    );
    assert!(
        template.contains("<span>{phase()}</span>"),
        "the template survives verbatim"
    );
}
