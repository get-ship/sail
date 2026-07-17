//! Compile-time acceptance for `var(--name)` references inside a
//! component's `<style>` block.
//!
//! Contract pinned by these tests:
//!
//!   1. `var(--known-token)` — declared in `tokens.css` or defined
//!      locally in the same `<style>` block — produces zero
//!      `sail-css-token` findings. Silent at the build gate.
//!   2. `var(--legitimate-convention)` — names the canonical
//!      `@ship/tokens` source uses in fallback chains but never
//!      declares (e.g. `--font-sans`, referenced as
//!      `var(--font-sans, system-ui, sans-serif)` across every theme
//!      in `packages/sail/tokens/src/themes.css`) — also produces zero
//!      findings. Surfaced by the docs-site tutorial, lesson 25.
//!   3. `var(--genuinely-unknown)` — a name that is neither in the
//!      theme registry nor a local convention — surfaces an **Error**
//!      finding by default. The build hard-fails. This is the strict
//!      typo-catcher behavior the internal monorepo relies on.
//!   4. The strict default is *opt-out* via `CompileOpts::lint_overrides`.
//!      An external SUI consumer that wants the softer behavior passes
//!      an `OverrideSeverity::Warn` (or `Off`) override for
//!      `sail-css-token`. The rule itself stays strict; the override is
//!      a build-host configuration.
//!
//! NOTE ON THE FLOOR: `sail-css-token` lints `var(--…)` references inside an
//! authored `<style>` block — and an authored `<style>` block is itself a floor
//! violation (`sail-no-style-block`), which the compiler now refuses
//! UNCONDITIONALLY. So `compile()` no longer succeeds for these sources
//! regardless of the token: the floor stops them. The `sail-css-token` CONTRACT
//! (known = silent, unknown = Error, Warn/Off override softens) is therefore
//! pinned here on the finding stream via `analyze` (`css_token_findings`), which
//! reports every rule's findings independently of the floor's refusal. The
//! `compile()`-level Error signal for an unknown token still holds because
//! `css_token::check` runs in `check::run`, ahead of the floor gate — see
//! `truly_unknown_token_strict_default_blocks_build`.
//!
//! Confined to `crates/sail/compiler/tests/**` per the job spec.

use sail_compiler::{
    analyze, compile, CompileMode, CompileOpts, ComponentMeta, DiagSeverity, LintOverride,
    OverrideSeverity,
};

fn opts(component_name: &str) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(format!("components/{component_name}/{component_name}.sui").into()),
        component_meta: Some(ComponentMeta {
            name: component_name.into(),

        }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

/// Count `sail-css-token` findings for `source`. Zero = silent.
fn css_token_findings(source: &str, overrides: &[LintOverride]) -> Vec<(DiagSeverity, String)> {
    analyze(source, None, overrides)
        .diagnostics
        .into_iter()
        .filter(|d| d.rule.as_deref() == Some("sail-css-token"))
        .map(|d| (d.severity, d.message))
        .collect()
}

#[test]
fn known_theme_token_is_silent() {
    // `--foreground` is declared in `tokens.css`. The lint must NOT surface a
    // finding for it.
    let src = r#"<scope>
  props: {
    title = ''
  }
</scope>
<style>
  .card { color: var(--foreground); }
</style>
<div>{title()}</div>"#;
    let findings = css_token_findings(src, &[]);
    assert!(
        findings.is_empty(),
        "var(--foreground) should produce zero sail-css-token findings, got: {findings:?}"
    );
}

#[test]
fn locally_defined_token_is_silent() {
    // `--btn-h` is defined in the same `<style>` block. No finding.
    let src = r#"<scope>
  props: {
    title = ''
  }
</scope>
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
    // `packages/sail/tokens/src/themes.css`. It is listed in
    // `sail_lint::tokens::CONVENTION_TOKENS` so the lint recognizes it
    // without an Error.
    let src = r#"<scope>
  props: {
    title = ''
  }
</scope>
<style>
  .kind { font-family: var(--font-sans, sans-serif); }
</style>
<div>{title()}</div>"#;
    // The floor refuses the authored `<style>` (`sail-no-style-block`), so the css
    // token contract is pinned on the finding stream, not on `compile()` success.
    let findings = css_token_findings(src, &[]);
    assert!(
        findings.is_empty(),
        "var(--font-sans) is a documented @ship/tokens convention — must produce zero \
         sail-css-token findings, got {findings:?}"
    );
}

#[test]
fn font_sans_convention_token_without_fallback_is_silent() {
    // The reference can stand without a fallback — `--font-sans` is
    // still a real, recognized name. (Verified on the finding stream: the
    // authored `<style>` vehicle is refused by the floor.)
    let src = r#"<scope>
  props: {
    title = ''
  }
</scope>
<style>
  .card { font-family: var(--font-sans); }
</style>
<div>{title()}</div>"#;
    let findings = css_token_findings(src, &[]);
    assert!(
        findings.is_empty(),
        "var(--font-sans) must produce zero sail-css-token findings, got: {findings:?}"
    );
}

#[test]
fn truly_unknown_token_strict_default_blocks_build() {
    // The strict default: a name that is neither in the theme registry
    // nor a local convention is Error. `compile()` rejects; the build
    // hard-fails. This is the internal monorepo's contract — it catches
    // typos at the build line, not three weeks later in the browser.
    let src = r#"<scope>
  props: {
    title = ''
  }
</scope>
<style>
  .card { color: var(--definitely-not-a-real-token); }
</style>
<div>{title()}</div>"#;
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
fn truly_unknown_token_warn_override_downgrades_to_warning() {
    // Same source as the strict-default case. The build host (a third-party SUI
    // consumer) passes an `OverrideSeverity::Warn` override for `sail-css-token`.
    // The css-token finding surfaces as a Warning so the LSP / agent can still
    // show the typo suggestion — the rule itself is unchanged. (The build is
    // still refused: the authored `<style>` is a floor violation; this pins the
    // css-token severity on the finding stream.)
    let src = r#"<scope>
  props: {
    title = ''
  }
</scope>
<style>
  .card { color: var(--definitely-not-a-real-token); }
</style>
<div>{title()}</div>"#;
    let overrides = vec![LintOverride {
        rule: "sail-css-token".into(),
        severity: OverrideSeverity::Warn,
    }];
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
    // `sail-css-token`. The build proceeds; the finding is dropped at
    // the build gate. (The lint still emits it; `sail_lint::lint_path`
    // consumers like the LSP still see the raw Error finding — the
    // override only affects this build's gate.)
    let src = r#"<scope>
  props: {
    title = ''
  }
</scope>
<style>
  .card { color: var(--definitely-not-a-real-token); }
</style>
<div>{title()}</div>"#;
    let overrides = vec![LintOverride {
        rule: "sail-css-token".into(),
        severity: OverrideSeverity::Off,
    }];
    // An Off override drops the css-token finding from the gate stream. (The
    // build is still refused by the floor's `<style>` ban — this pins that the
    // NON-floor css-token rule is softenable, unlike the floor.)
    let findings = css_token_findings(src, &overrides);
    assert!(
        findings.is_empty(),
        "Off override must drop the rule's findings, got: {findings:?}"
    );
}
