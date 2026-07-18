//! compiler-stability-audit #100 item 1: a typo'd/unknown `sx` atom must
//! NEVER become a silent-nothing.
//!
//! `sail-sx` (the vocabulary/typo lint, `crates/sail/lint/src/rules/sx.rs`)
//! used to ride the ordinary override channel, unlike the floor rules
//! (`check::policy::FLOOR_RULES`). A host that downgraded it to `Warn`/`Off`
//! turned `sx="bg-surfce-2"` into a dead class: `parse::preprocess`'s
//! `lower_sx_value` always pushes the sanitized class name into the emitted
//! `class="…"` attribute, and only appends a CSS rule when the atom actually
//! resolves — it has no severity awareness at all, so the class landed with
//! zero matching CSS and (at `Off`) zero diagnostic anywhere.
//!
//! The fix: `sail-sx` is now a floor rule (`check::policy::FLOOR_RULES`) —
//! `Error` under every policy, un-softenable by any `lint_override`. So
//! `compile()` refuses the build before the dead class ever reaches emitted
//! output, regardless of what capability tier or override a host applies.
//!
//! Confined to `crates/sail/compiler/tests/**` per the job spec.

use sail_compiler::{compile, CompileMode, CompileOpts, ComponentMeta, OverrideSeverity};

const TYPO_SRC: &str = r#"<scope>
  props: {
    title = ''
  }
</scope>
<div sx="bg-surfce-2">{title()}</div>"#;

fn opts(component_name: &str) -> CompileOpts {
    CompileOpts {
        component_meta: Some(ComponentMeta { name: component_name.into() }),
        mode: CompileMode::Component,
        ..CompileOpts::default()
    }
}

#[test]
fn typo_atom_is_rejected_under_the_default_tier() {
    // (a) Default tier, no overrides — the pre-existing strict behavior.
    let err = compile(TYPO_SRC, opts("sx-typo-default"))
        .expect_err("a typo'd sx atom must be rejected by default");
    assert!(
        err.message.contains("bg-surfce-2") && err.message.contains("did you mean"),
        "error must name the typo and suggest the fix: {err:?}"
    );
}

#[test]
fn typo_atom_is_still_rejected_under_a_softened_policy() {
    // (b) A host tries to downgrade `sail-sx` to `Off` (or `Warn`) — the
    // shape a third-party consumer would use to soften a strict rule, per
    // `CompileOpts::with_lint_override`. Old behavior: this compiled
    // successfully and silently baked in a dead `sx-bgsurfce2` class with no
    // matching CSS rule and no diagnostic anywhere (proved red against this
    // test before `sail-sx` joined the floor). `sail-sx` is now a floor rule,
    // so no override can touch it — the build must still refuse.
    for severity in [OverrideSeverity::Off, OverrideSeverity::Warn] {
        let err = compile(
            TYPO_SRC,
            opts("sx-typo-softened").with_lint_override("sail-sx", severity),
        )
        .expect_err(&format!(
            "a `sail-sx` override of {severity:?} must NOT let a typo'd atom compile \
             silently — it is a floor rule and cannot be softened"
        ));
        assert!(
            err.message.contains("bg-surfce-2"),
            "even under a softened policy ({severity:?}), the typo must surface as an error, \
             never silently compile a dead class: {err:?}"
        );
    }
}

#[test]
fn no_dead_class_reaches_emitted_output_under_any_policy() {
    // Belt-and-suspenders on the exact failure mode: even if some future
    // change reintroduced a soft path for `sail-sx`, a dead class (present in
    // `class="…"` with no matching CSS rule anywhere) must never reach
    // `compile()`'s output. Since the floor refuses the build outright, there
    // is no `CompileOutput` to inspect — the invariant holds by construction.
    for severity in [OverrideSeverity::Off, OverrideSeverity::Warn] {
        let result = compile(
            TYPO_SRC,
            opts("sx-typo-no-dead-class").with_lint_override("sail-sx", severity),
        );
        assert!(
            result.is_err(),
            "compile() must refuse rather than emit a dead sx class ({severity:?})"
        );
    }
}
