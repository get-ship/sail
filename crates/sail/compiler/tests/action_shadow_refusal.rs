//! Regression pin for the canvas-detail-crash incident class: a template
//! expression that references a name shared between a setup-returned scope
//! member and an imported (built-in) `use:` action must REFUSE at compile,
//! never silently pick a winner.
//!
//! Incident: `<sail-canvas>` declared a setup-returned value named
//! `worldTransform` — identical to the imported `use:worldTransform` action
//! consumed via `use:worldTransform={worldTransform()}`. The compiler's
//! expr-rewrite leaves built-in action names bare (they must resolve as
//! real ESM bindings for `runAction` to receive the function), so the value
//! expression silently compiled to an argless self-call of the action
//! import — `runAction(el, worldTransform, () => worldTransform())` — node
//! came in `undefined`, and the action threw reading `.style`. Three
//! composing components (`sail-board`, `sail-canvas`, `sail-draw`) crashed on
//! mount for weeks with no compiler diagnostic.
//!
//! `git show` of the actual buggy commit (`c7370139^`) uses the
//! pre-`controller-collapse` object-form `controller({ setup, on, mount })`
//! shape, which this branch's `controller_parse` no longer understands
//! (fails first on an unrelated "unbound `pointerDown`" error, since the
//! `on: {...}` section format predates the current single-function
//! controller). The fixture below reproduces the SAME collision — a
//! setup-returned scope member named identically to the `use:` action it
//! feeds — under the CURRENT `controller((ctx) => {...})` shape, which is
//! `<sail-canvas>`'s shape today with `worldTransformCss` reverted to its
//! pre-fix name `worldTransform`.

use std::path::PathBuf;

use sail_compiler::{compile, CompileMode, CompileOpts, ComponentMeta};

fn opts(name: &str) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: Some(format!("./{name}.sail.controller.ts")),
        mode: CompileMode::Component,
        source_path: Some(PathBuf::from(format!("/x/{name}.sail"))),
        component_meta: Some(ComponentMeta { name: name.to_string() }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

/// `<sail-canvas>`'s actual shape, today, with `worldTransformCss` reverted
/// to `worldTransform` — the exact pre-fix collision, current controller
/// form.
const SUI_CANVAS_COLLISION_SHAPE: &str = r#"<scope>
  props: {
    value = ''
  }
</scope>
<controller>
import { controller, signal, derived } from '@ship/sail-runtime'
import type { Scope } from './sail-canvas.sail'

export default controller(({ value }: Scope) => {
  const viewportEl = signal<HTMLDivElement | null>(null)
  const canvas = signal<{ transform: () => { x: number; y: number; scale: number } }>(
    null as unknown as { transform: () => { x: number; y: number; scale: number } },
  )
  const worldTransform = derived<string>(() => {
    const t = canvas().transform()
    return `translate(${t.x}px, ${t.y}px) scale(${t.scale})`
  })
  return { canvas, viewportEl, worldTransform }
})
</controller>
<div bind:this={viewportEl} sx="relative">
  <div use:worldTransform={worldTransform()} sx="absolute">
    <slot></slot>
  </div>
</div>
"#;

#[test]
fn sui_canvas_shape_self_call_collision_refuses_at_compile() {
    let err = compile(SUI_CANVAS_COLLISION_SHAPE, opts("sail-canvas"))
        .expect_err("a scope member shadowing the `use:` action it feeds must refuse, not self-call");
    assert!(
        err.message.contains("worldTransform"),
        "diagnostic must name the colliding identifier: {}",
        err.message
    );
    assert!(
        err.message.contains("use:worldTransform"),
        "diagnostic must name the `use:` directive: {}",
        err.message
    );
    assert!(
        err.message.to_lowercase().contains("canvas-detail-crash"),
        "diagnostic should teach the incident shape by name: {}",
        err.message
    );
}

/// The same collision, OUTSIDE a `use:` directive — a scope member reusing
/// a built-in action's name, referenced as a bare identifier elsewhere in
/// the template. The general precedence rule (built-in action import wins
/// over a same-named scope member) makes the scope member permanently
/// unreachable from any template expression, so this must refuse too, not
/// just the `use:X={X()}` special case.
const SCOPE_IMPORT_COLLISION_OUTSIDE_USE: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const tooltip = signal('hello')
  return { tooltip }
})
</controller>
<p>{tooltip()}</p>
"#;

#[test]
fn scope_import_collision_outside_use_directive_refuses() {
    let err = compile(SCOPE_IMPORT_COLLISION_OUTSIDE_USE, opts("collision-outside-use"))
        .expect_err("a scope member named identically to a built-in action must refuse anywhere in the template, not only inside its own `use:` value");
    assert!(
        err.message.contains("tooltip"),
        "diagnostic must name the colliding identifier: {}",
        err.message
    );
    assert!(
        err.message.contains("ambiguous"),
        "diagnostic must explain the collision, not report a generic unbound name: {}",
        err.message
    );
}

/// Clean case: a scope member's name is never referenced in ANY template
/// expression — collides with a built-in action name in name only, never
/// observed by the compiler's template walk. No error: the compiler only
/// refuses ambiguity it can actually see materialize.
const SAME_NAME_NEVER_USED_IN_TEMPLATE: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const tooltip = signal('hello')
  const label = signal('shown')
  return { tooltip, label }
})
</controller>
<p>{label()}</p>
"#;

#[test]
fn colliding_name_never_referenced_in_template_compiles_clean() {
    compile(SAME_NAME_NEVER_USED_IN_TEMPLATE, opts("collision-unused"))
        .expect("a scope/action name collision that's never read from a template expression is not observable, so it must not error");
}

/// Clean case: a custom (non-built-in) action name shares no collision
/// with anything — `use:X={expr}` where `expr` legitimately reads a
/// DIFFERENTLY-named scope value is the normal, correct shape and must
/// keep compiling.
const NO_COLLISION_USE_DIRECTIVE: &str = r#"<controller>
import { controller, derived, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const scale = signal(1)
  const worldTransformCss = derived<string>(() => `scale(${scale()})`)
  return { worldTransformCss }
})
</controller>
<div use:worldTransform={worldTransformCss()}></div>
"#;

#[test]
fn distinctly_named_scope_value_fed_to_use_directive_compiles_clean() {
    compile(NO_COLLISION_USE_DIRECTIVE, opts("no-collision"))
        .expect("a use: value expression reading a distinctly-named scope value is the correct shape and must compile");
}
