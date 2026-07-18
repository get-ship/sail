//! Compile-time behavior of the `sail-prefer-primitive` lint through the
//! compiler's public `analyze()` / `compile()` surface.
//!
//! Contract pinned by these tests:
//!
//!   1. A consumer component with `display: grid` (or `flex`) in its `<style>`
//!      block surfaces a `sail-prefer-primitive` **Warning** diagnostic. The
//!      message names the layout primitives and the skill.
//!   2. `sail-prefer-primitive` is advisory ONLY — it never escalates to an
//!      Error on its own (its finding is always a `Warning`). The build it lives
//!      in may still be refused for an UNRELATED reason: the ad-hoc layout lives
//!      in an authored `<style>` block, and an authored `<style>` is a floor
//!      violation (`sail-no-style-block`) the compiler refuses unconditionally.
//!      So the floor — not prefer-primitive — is what stops the build.
//!   3. Components under `packages/components/sail/primitives/**` are exempt —
//!      the primitives pack legitimately hand-rolls flex/grid internally.
//!   4. A component that composes `<sail-grid>` (no ad-hoc layout CSS) emits no
//!      `sail-prefer-primitive` finding.
//!
//! Note: the test sources used `class="wrap"` to mark the styled div; with the
//! floor now banning `class=`, the markup is bare. The lint scans the `<style>`
//! content (the only thing it cares about), so the class name is not load-bearing.

use std::path::PathBuf;

use sail_compiler::{analyze, compile, CompileMode, CompileOpts, ComponentMeta, DiagSeverity};

const RULE: &str = "sail-prefer-primitive";

fn opts(name: &str, path: &str) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(PathBuf::from(path)),
        component_meta: Some(ComponentMeta {
            name: name.into(),

        }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

/// The `sail-prefer-primitive` diagnostics for `source` compiled at `path`.
fn prefer_primitive_diags(source: &str, path: &str) -> Vec<(DiagSeverity, String)> {
    analyze(source, Some(&PathBuf::from(path)), &[])
        .diagnostics
        .into_iter()
        .filter(|d| d.rule.as_deref() == Some(RULE))
        .map(|d| (d.severity, d.message))
        .collect()
}

const CONSUMER: &str = "packages/components/sail/charts/src/sail-gauge/sail-gauge.sail";
const PRIMITIVE: &str = "packages/components/sail/primitives/src/sail-card/sail-card.sail";

#[test]
fn consumer_display_grid_surfaces_a_warning() {
    let src = r#"<style>
  .wrap { display: grid; gap: var(--space-3); }
</style>
<div></div>"#;
    let diags = prefer_primitive_diags(src, CONSUMER);
    assert_eq!(diags.len(), 1, "expected one prefer-primitive finding, got {diags:?}");
    assert_eq!(diags[0].0, DiagSeverity::Warning, "must be a Warning, never Error: {diags:?}");
    assert!(
        diags[0].1.contains("<grid>") && diags[0].1.contains("sail skill"),
        "message should name the layout keywords + skill: {}",
        diags[0].1
    );
}

#[test]
fn prefer_primitive_is_advisory_but_floor_refuses_the_style_block() {
    // `sail-prefer-primitive` on its own is only a Warning (it never escalates) —
    // proven on the finding stream. But the ad-hoc grid lives in an authored
    // `<style>` block, so the FLOOR (`sail-no-style-block`) refuses the build
    // unconditionally. The floor, not prefer-primitive, is what stops it.
    let src = r#"<style>
  .wrap { display: grid; }
</style>
<div></div>"#;
    // prefer-primitive stays a Warning — advisory, never an Error.
    let diags = prefer_primitive_diags(src, CONSUMER);
    assert_eq!(diags.len(), 1, "expected one prefer-primitive finding, got {diags:?}");
    assert_eq!(diags[0].0, DiagSeverity::Warning, "prefer-primitive is advisory: {diags:?}");
    // ...but the authored `<style>` is a floor violation, so the build refuses.
    let err = compile(src, opts("sail-gauge", CONSUMER))
        .expect_err("authored <style> must be refused by the floor");
    assert_eq!(
        err.category.as_deref(),
        Some("sail-no-style-block"),
        "the floor rule (not prefer-primitive) refuses the build: {err:?}"
    );
}

#[test]
fn primitives_pack_component_is_exempt() {
    // The SAME grid CSS under the primitives pack produces no finding.
    let src = r#"<style>
  .wrap { display: grid; gap: var(--space-3); }
</style>
<div></div>"#;
    let diags = prefer_primitive_diags(src, PRIMITIVE);
    assert!(diags.is_empty(), "primitives pack must be exempt: {diags:?}");
}

#[test]
fn composing_sui_grid_emits_no_finding() {
    // Composing the primitive is exactly what the rule wants — no warning.
    let src = r#"<style>
  .title { color: var(--foreground); }
</style>
<sail-grid gap="3"><div>a</div><div>b</div></sail-grid>"#;
    let diags = prefer_primitive_diags(src, CONSUMER);
    assert!(diags.is_empty(), "composing <sail-grid> should be silent: {diags:?}");
}
