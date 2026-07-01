//! Compile-time acceptance for `var(--name)` references inside a
//! component's `<style>` block.
//!
//! Background: the css-token lint (`sui-css-token`, in `sui-lint`) flags
//! `var(--unknown)` so authors catch typos against the theme registry
//! (`crates/sui/compiler/tokens.json`) and local `--name:` defs. That
//! is a useful *finding*, but a custom property is still valid CSS —
//! the build must not refuse.
//!
//! Contract pinned by these tests:
//!
//!   1. `var(--known-token)` — present in `tokens.json` or defined
//!      locally in the same `<style>` block — produces zero
//!      `sui-css-token` findings (silent).
//!   2. `var(--legitimate-convention)` — names the canonical
//!      `@ship/tokens` source uses in fallback chains but never
//!      declares (e.g. `--font-sans`, referenced as
//!      `var(--font-sans, system-ui, sans-serif)` across every theme
//!      in `packages/sui/tokens/src/themes.css`) — also produces zero
//!      findings. Surfaced by the docs-site tutorial, lesson 25.
//!   3. `var(--genuinely-unknown)` — a name that is neither in the
//!      theme registry nor a local convention — surfaces a Warn with a
//!      Levenshtein close-match suggestion. The Warn does NOT block
//!      the build (`compile()` still succeeds); the LSP surfaces it.
//!
//!   - `compile()` succeeds in all three cases (no Error finding).
//!
//! Confined to `crates/sui/compiler/tests/**` per the job spec.

use sui_compiler::{analyze, compile, CompileMode, CompileOpts, ComponentMeta, DiagSeverity};

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
    }
}

/// Count `sui-css-token` findings for `source`. Zero = silent.
fn css_token_findings(source: &str) -> Vec<(DiagSeverity, String)> {
    analyze(source, None)
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
    let findings = css_token_findings(src);
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
    let findings = css_token_findings(src);
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
    // without a Warn.
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
    let findings = css_token_findings(src);
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
    let findings = css_token_findings(src);
    assert!(
        findings.is_empty(),
        "var(--font-sans) must compile silently, got: {findings:?}"
    );
}

#[test]
fn truly_unknown_token_still_warns_but_does_not_block() {
    // The acceptance criterion narrows var() acceptance to the
    // recognized set. A name that is neither in the theme registry
    // nor a local convention is still a likely typo — the lint
    // surfaces a Warn (so the LSP / agent tooling can suggest a
    // close match), but the build is not blocked.
    let src = r#"<prop name="title" type="string" default="" />
<style>
  .card { color: var(--definitely-not-a-real-token); background: var(--also-fake-xyz); }
</style>
<div class="card">{title()}</div>"#;
    let out = compile(src, opts("unknown-tokens-warn"))
        .expect("Warn-severity findings must not block the build");
    assert!(
        out.js.contains("var(--definitely-not-a-real-token)"),
        "emitted JS should still carry the var() reference: {}",
        out.js
    );
    let findings = css_token_findings(src);
    assert_eq!(
        findings.len(),
        2,
        "expected one Warn per unknown token (2 tokens), got {findings:?}"
    );
    for (sev, _) in &findings {
        assert_eq!(
            *sev,
            DiagSeverity::Warning,
            "unknown tokens must be Warn, not Error: {findings:?}"
        );
    }
}
