//! Inline-controller SFCs get their template↔controller scope checked even
//! without a `source_path`.
//!
//! Regression guard for the stale `source_path.is_some()` gate in
//! `check::{run,collect}` that skipped every scope check (`scope_bound`,
//! `scope_member`, `on_callable`, `bind_this_target`) whenever a `.sail` was
//! compiled in-memory (no path). Those checks read the passed `controller`,
//! not the path — so the inline `<controller>` block is enough context. Before
//! the fix, an out-of-scope template reference in an in-memory SFC compiled
//! clean and only surfaced at runtime as `$scope.X is not a function`.

use sail_compiler::{compile, CompileMode, CompileOpts};

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

/// Inline controller whose typed scope provides exactly `count`.
const CONTROLLER: &str = "<controller>\n\
    import { controller, signal } from '@ship/sail-runtime'\n\
    export default controller((ctx) => ({ count: signal(0) }))\n\
    </controller>\n";

#[test]
fn inline_controller_in_scope_ref_passes() {
    let src = format!(
        "{CONTROLLER}<button on:click={{() => count.set(count() + 1)}}>{{count()}}</button>"
    );
    compile(&src, opts()).expect("in-scope reference must compile with an inline controller");
}

#[test]
fn inline_controller_out_of_scope_ref_fails_without_source_path() {
    // `countt` is a typo — not in the controller scope, and there is no
    // `<prop>` declaring it. With the inline controller as scope
    // context, this must now be a compile error (it silently passed before).
    let src = format!("{CONTROLLER}<button>{{countt()}}</button>");
    let err = compile(&src, opts())
        .expect_err("out-of-scope reference in an inline-controller SFC must fail to compile");
    assert!(
        err.message.contains("countt"),
        "error should name the unbound identifier `countt`, got: {}",
        err.message
    );
}

#[test]
fn bare_fragment_without_controller_or_path_still_compiles() {
    // NEITHER a controller NOR a source_path: nothing defines the scope, so the
    // scope checks stay skipped (else every free name false-positives). This is
    // the case the original guard protected — it must keep working.
    compile(
        r#"<button on:click={() => count.set(count() + 1)}>{count()}</button>"#,
        opts(),
    )
    .expect("a bare fragment with no scope context must still compile");
}
