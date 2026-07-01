//! Compile-time acceptance for `var(--name)` references inside a
//! component's `<style>` block.
//!
//! Contract pinned by these tests:
//!
//!   1. `var(--known-token)` — present in `tokens.json` or defined
//!      locally in the same `<style>` block — produces zero
//!      `sui-css-token` findings. Silent at the build gate.
//!   2. `var(--legitimate-convention)` — names the canonical
//!      `@ship/tokens` source uses in fallback chains but never
//!      declares (e.g. `--font-sans`, referenced as
//!      `var(--font-sans, system-ui, sans-serif)` across every theme
//!      in `packages/sui/tokens/src/themes.css`) — also produces zero
//!      findings. Surfaced by the docs-site tutorial, lesson 25.
//!   3. `var(--genuinely-unknown)` — a name that is neither in the
//!      theme registry nor a local convention — surfaces an **Error**
//!      finding by default. The build hard-fails. This is the strict
//!      typo-catcher behavior the internal monorepo relies on.
//!   4. The strict default is *opt-out* via `CompileOpts::lint_overrides`.
//!      An external SUI consumer that wants the softer behavior passes
//!      an `OverrideSeverity::Warn` (or `Off`) override for
//!      `sui-css-token`. The rule itself stays strict; the override is
//!      a build-host configuration.
//!
//! Confined to `crates/sui/compiler/tests/**` per the job spec.

use sui_compiler::{
    analyze, compile, CompileMode, CompileOpts, ComponentMeta, DiagSeverity, LintOverride,
    OverrideSeverity,
};

fn opts(component_name: &str) -> CompileOpts {
    CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(format!("components/{component_name}/{component_name}.sui").into()),
        styles: None,
        component_meta: Some(ComponentMeta {
            name: component_name.into(),
            shadow: true,
        }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn opts_with_override(component_name: &str, overrides: Vec<LintOverride>) -> CompileOpts {
    let mut o = opts(component_name);
    o.lint_overrides = overrides;
    o
}

/// Count `sui-css-token` findings for `source`. Zero = silent.
fn css_token_findings(source: &str, overrides: &[LintOverride]) -> Vec<(DiagSeverity, String)> {
    analyze(source, None, overrides)
        .diagnostics
        .into_iter()
        .filter(|d| d.rule.as_deref() == Some("sui-css-token"))
        .map(|d| (d.severity, d.message))
        .collect()
}

#[test]
fn known_theme_token_is_silent() {
    // `--foreground` is in `tokens.json`. The lint must NOT surface a
    // finding for it.
    let src = r#"<prop name="title" type="string" default="" />
<style>
  .card { color: var(--foreground); }
</style>
<div class="card">{title()}</div>"#;
    let findings = css_token_findings(src, &[]);
    assert!(
        findings.is_empty(),
        "var(--foreground) should produce zero sui-css-token findings, got: {findings:?}"
    );
}

#[test]
fn locally_defined_token_is_silent() {
    // `--btn-h` is defined in the same `<style>` block. No finding.
    let src = r#"<prop name="title" type="string" default="" />
<style>
  :host { --btn-h: 34px; height: var(--btn-h); }
</style>
<div>{title()}</div>"#;
    let findings = css_token_findings(src, &[]);
    assert!(
        findings.is_empty(),
        "var(--btn-h) with a local `--btn-h:` def should be silent, got: {findings:?}"
    );
}

#[test]
fn font_sans_convention_token_is_silent() {
    // The lesson-25 shape. `--font-sans` is a documented @ship/tokens
    // convention used in `var(--font-sans, system-ui, sans-serif)`
    // fallback chains across every theme-preview block in
    // `packages/sui/tokens/src/themes.css`. It is listed in
    // `crates/sui/compiler/tokens.json` so the lint recognizes it
    // without an Error.
    let src = r#"<prop name="title" type="string" default="" />
<style>
  .kind { font-family: var(--font-sans, sans-serif); }
</style>
<div class="kind">{title()}</div>"#;
    let out = compile(src, opts("font-sans-convention"))
        .expect("var(--font-sans) should compile");
    assert!(
        out.js.contains("font-family: var(--font-sans, sans-serif)"),
        "emitted JS should carry the var() reference with fallback: {}",
        out.js
    );
    let findings = css_token_findings(src, &[]);
    assert!(
        findings.is_empty(),
        "var(--font-sans) is a documented @ship/tokens convention — must compile silently, \
         got {findings:?}"
    );
}

#[test]
fn font_sans_convention_token_compiles_without_fallback() {
    // The reference can stand without a fallback — `--font-sans` is
    // still a real, recognized name.
    let src = r#"<prop name="title" type="string" default="" />
<style>
  .card { font-family: var(--font-sans); }
</style>
<div class="card">{title()}</div>"#;
    let out = compile(src, opts("font-sans-no-fallback"))
        .expect("var(--font-sans) should compile");
    assert!(
        out.js.contains("font-family: var(--font-sans)"),
        "emitted JS should carry the var() reference: {}",
        out.js
    );
    let findings = css_token_findings(src, &[]);
    assert!(
        findings.is_empty(),
        "var(--font-sans) must compile silently, got: {findings:?}"
    );
}

#[test]
fn truly_unknown_token_strict_default_blocks_build() {
    // The strict default: a name that is neither in the theme registry
    // nor a local convention is Error. `compile()` rejects; the build
    // hard-fails. This is the internal monorepo's contract — it catches
    // typos at the build line, not three weeks later in the browser.
    let src = r#"<prop name="title" type="string" default="" />
<style>
  .card { color: var(--definitely-not-a-real-token); }
</style>
<div class="card">{title()}</div>"#;
    let err = compile(src, opts("unknown-tokens-strict"))
        .expect_err("strict default must reject an unknown token");
    assert!(
        err.message.contains("--definitely-not-a-real-token"),
        "error must name the unknown token: {err:?}"
    );
    let findings = css_token_findings(src, &[]);
    assert_eq!(
        findings.len(),
        1,
        "expected one finding for the one unknown token, got {findings:?}"
    );
    for (sev, _) in &findings {
        assert_eq!(
            *sev,
            DiagSeverity::Error,
            "unknown tokens must be Error by default: {findings:?}"
        );
    }
}

#[test]
fn truly_unknown_token_warn_override_lets_build_pass() {
    // Same source as the strict-default case. The build host (a
    // third-party SUI consumer) passes an `OverrideSeverity::Warn`
    // override for `sui-css-token`. The build now succeeds; the
    // finding surfaces as a Warning so the LSP / agent can still show
    // the typo suggestion. The rule itself is unchanged.
    let src = r#"<prop name="title" type="string" default="" />
<style>
  .card { color: var(--definitely-not-a-real-token); }
</style>
<div class="card">{title()}</div>"#;
    let overrides = vec![LintOverride {
        rule: "sui-css-token".into(),
        severity: OverrideSeverity::Warn,
    }];
    let out = compile(src, opts_with_override("unknown-tokens-warn", overrides.clone()))
        .expect("Warn override must let the build proceed");
    assert!(
        out.js.contains("var(--definitely-not-a-real-token)"),
        "emitted JS should still carry the var() reference: {}",
        out.js
    );
    let findings = css_token_findings(src, &overrides);
    assert_eq!(
        findings.len(),
        1,
        "expected one finding for the one unknown token, got {findings:?}"
    );
    for (sev, _) in &findings {
        assert_eq!(
            *sev,
            DiagSeverity::Warning,
            "Warn override must downgrade Error to Warning, not remove: {findings:?}"
        );
    }
}

#[test]
fn truly_unknown_token_off_override_silences() {
    // The off switch: the build host passes `OverrideSeverity::Off` for
    // `sui-css-token`. The build proceeds; the finding is dropped at
    // the build gate. (The lint still emits it; `sui_lint::lint_path`
    // consumers like the LSP still see the raw Error finding — the
    // override only affects this build's gate.)
    let src = r#"<prop name="title" type="string" default="" />
<style>
  .card { color: var(--definitely-not-a-real-token); }
</style>
<div class="card">{title()}</div>"#;
    let overrides = vec![LintOverride {
        rule: "sui-css-token".into(),
        severity: OverrideSeverity::Off,
    }];
    let out = compile(src, opts_with_override("unknown-tokens-off", overrides.clone()))
        .expect("Off override must let the build proceed");
    assert!(
        out.js.contains("var(--definitely-not-a-real-token)"),
        "emitted JS should still carry the var() reference: {}",
        out.js
    );
    let findings = css_token_findings(src, &overrides);
    assert!(
        findings.is_empty(),
        "Off override must drop the rule's findings, got: {findings:?}"
    );
}
