//! A typo'd/unknown Style token must NEVER become a silent-nothing.
//!
//! `sail-sx` is a floor rule (`check::policy::FLOOR_RULES`) — `Error` under
//! every policy, un-softenable by any `lint_override`. So `compile()` refuses
//! the build before a dead class reaches emitted output.
//!
//! Utility-string atoms are residual product (also refuse). This suite pins
//! the Style product fail-closed path for bad tokens.

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, OverrideSeverity, compile};

const TYPO_SRC: &str = r#"<scope>
  props: {
    title = ''
  }
</scope>
<div sx={style.bg(surfce-2)}>{title()}</div>"#;

const UTILITY_STRING_SRC: &str = r#"<div sx="p-4 bg-surface-2">x</div>"#;

fn opts(component_name: &str) -> CompileOpts {
    CompileOpts {
        component_meta: Some(ComponentMeta {
            name: component_name.into(),
        }),
        mode: CompileMode::Component,
        ..CompileOpts::default()
    }
}

#[test]
fn typo_style_token_is_rejected_under_the_default_tier() {
    let err = compile(TYPO_SRC, opts("sx-typo-default"))
        .expect_err("a typo'd Style token must be rejected by default");
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("surfce")
            || msg.contains("token")
            || msg.contains("unknown")
            || msg.contains("style"),
        "error must name the bad token: {err:?}"
    );
}

#[test]
fn typo_style_token_is_still_rejected_under_a_softened_policy() {
    for severity in [OverrideSeverity::Off, OverrideSeverity::Warn] {
        let err = compile(
            TYPO_SRC,
            opts("sx-typo-softened").with_lint_override("sail-sx", severity),
        )
        .expect_err(&format!(
            "a `sail-sx` override of {severity:?} must NOT let a typo'd token compile \
             silently — it is a floor rule and cannot be softened"
        ));
        let msg = err.message.to_lowercase();
        assert!(
            msg.contains("surfce")
                || msg.contains("token")
                || msg.contains("unknown")
                || msg.contains("style"),
            "even under a softened policy ({severity:?}), the typo must surface: {err:?}"
        );
    }
}

#[test]
fn utility_string_sx_is_residual_error() {
    let err = compile(UTILITY_STRING_SRC, opts("sx-util-string"))
        .expect_err("utility-string sx must refuse");
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("utility-string") || msg.contains("style product") || msg.contains("residual"),
        "diagnostic must point at Style product: {err:?}"
    );
}

#[test]
fn no_dead_class_reaches_emitted_output_under_any_policy() {
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
