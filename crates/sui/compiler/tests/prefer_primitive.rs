//! Compile-time behavior of the `sui-prefer-primitive` lint through the
//! compiler's public `analyze()` / `compile()` surface.
//!
//! Contract pinned by these tests:
//!
//!   1. A consumer component with `display: grid` (or `flex`) in its `<style>`
//!      block surfaces a `sui-prefer-primitive` **Warning** diagnostic. The
//!      message names the layout primitives and the skill.
//!   2. `sui-prefer-primitive` is advisory ONLY — it never escalates to an
//!      Error on its own (its finding is always a `Warning`). The build it lives
//!      in may still be refused for an UNRELATED reason: the ad-hoc layout lives
//!      in an authored `<style>` block, and an authored `<style>` is a floor
//!      violation (`sui-no-style-block`) the compiler refuses unconditionally.
//!      So the floor — not prefer-primitive — is what stops the build.
//!   3. Components under `packages/components/sui/primitives/**` are exempt —
//!      the primitives pack legitimately hand-rolls flex/grid internally.
//!   4. A component that composes `<sui-grid>` (no ad-hoc layout CSS) emits no
//!      `sui-prefer-primitive` finding.
//!
//! Note: the test sources used `class="wrap"` to mark the styled div; with the
//! floor now banning `class=`, the markup is bare. The lint scans the `<style>`
//! content (the only thing it cares about), so the class name is not load-bearing.

use std::path::PathBuf;

use sui_compiler::{analyze, compile, CompileMode, CompileOpts, ComponentMeta, DiagSeverity};

const RULE: &str = "sui-prefer-primitive";

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

/// The `sui-prefer-primitive` diagnostics for `source` compiled at `path`.
fn prefer_primitive_diags(source: &str, path: &str) -> Vec<(DiagSeverity, String)> {
    analyze(source, Some(&PathBuf::from(path)), &[])
        .diagnostics
        .into_iter()
        .filter(|d| d.rule.as_deref() == Some(RULE))
        .map(|d| (d.severity, d.message))
        .collect()
}

const CONSUMER: &str = "packages/components/sui/charts/src/sui-gauge/sui-gauge.sui";
const PRIMITIVE: &str = "packages/components/sui/primitives/src/sui-card/sui-card.sui";

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
        diags[0].1.contains("<sui-grid>") && diags[0].1.contains("sui skill"),
        "message should name the primitives + skill: {}",
        diags[0].1
    );
}

#[test]
fn prefer_primitive_is_advisory_but_floor_refuses_the_style_block() {
    // `sui-prefer-primitive` on its own is only a Warning (it never escalates) —
    // proven on the finding stream. But the ad-hoc grid lives in an authored
    // `<style>` block, so the FLOOR (`sui-no-style-block`) refuses the build
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
    let err = compile(src, opts("sui-gauge", CONSUMER))
        .expect_err("authored <style> must be refused by the floor");
    assert_eq!(
        err.category.as_deref(),
        Some("sui-no-style-block"),
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
<sui-grid gap="3"><div>a</div><div>b</div></sui-grid>"#;
    let diags = prefer_primitive_diags(src, CONSUMER);
    assert!(diags.is_empty(), "composing <sui-grid> should be silent: {diags:?}");
}
