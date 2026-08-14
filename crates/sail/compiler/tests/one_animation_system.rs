//! ONE ANIMATION SYSTEM.
//!
//! Author form is the keyed `animate:` map. The concern inside is `play`.
//! Lifecycle (`enter`/`exit`), interaction (`hover`/`press`/…), timeline
//! (`reveal`/`scroll`), and ambient (`loop`) are keys — not a second syntax.

use sail_compiler::{CompileMode, CompileOpts, compile};

fn opts() -> CompileOpts {
    CompileOpts {
        mode: CompileMode::Component,
        ..Default::default()
    }
}

fn err_for(src: &str) -> String {
    let e = compile(src, opts()).expect_err("must be rejected");
    e.message
}

/// The lifecycle triggers exist and bake real animations — this is the capability
/// `transition:` used to own, now expressed in the one system.
#[test]
fn motion_enter_and_exit_bake_animations() {
    let src = r#"<card style({ animate: { enter: play(fade), exit: play(fade) } })> x</card>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(
        js.contains(r#"trigger: "enter""#),
        "enter trigger emitted:\n{js}"
    );
    assert!(
        js.contains(r#"trigger: "exit""#),
        "exit trigger emitted:\n{js}"
    );
    assert!(
        !js.contains("el.animate("),
        "no AOT animation — runtime owns creation:\n{js}"
    );
    assert!(
        js.contains("__sail_motion_attach"),
        "wired through the motion attach:\n{js}"
    );
}

/// The interaction triggers still work — enter/exit are additions, not a rewrite.
#[test]
fn the_interaction_triggers_are_untouched() {
    let src = r#"<button style({ animate: { hover: play(fade) } })> x</button>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(js.contains(r#"trigger: "hover""#), "{js}");
    assert!(
        js.contains("__sail_motion_attach"),
        "runtime attach present:\n{js}"
    );
}

/// An unknown play is a COMPILE error, not a silent runtime fallback.
#[test]
fn an_unknown_preset_fails_the_build() {
    let msg = err_for(r#"<card style({ animate: { exit: play(wobble) } })> x</card>"#);
    assert!(
        msg.contains("wobble") || msg.to_lowercase().contains("play"),
        "unknown play must name itself: {msg}"
    );
}

/// Second-system author forms are not sail vocabulary — generic unknown
/// directive (Style motion is the product surface).
#[test]
fn the_second_animation_system_is_unknown_vocabulary() {
    for src in [
        r#"<card transition:fade>x</card>"#,
        r#"<card in:slide>x</card>"#,
        r#"<card out:fade>x</card>"#,
    ] {
        let msg = err_for(src);
        assert!(
            msg.contains("unknown directive") || msg.contains("style."),
            "must be unknown-directive or Style pointer: {msg}"
        );
        assert!(
            !msg.to_ascii_lowercase().contains("retired"),
            "must not embalm: {msg}"
        );
    }
}

#[test]
fn retired_trigger_call_form_teaches_the_play_map() {
    let msg = err_for(r#"<card style({ animate: enter({ opacity: 'fade' }) })> x</card>"#);
    assert!(
        msg.contains("animate:") && msg.contains("play"),
        "must teach the play map: {msg}"
    );
}

/// THE DRIFT GUARD. Every advertised trigger must be writable as a map key
/// with `play(fade)`. A trigger added to `MOTION_TRIGGERS` and nowhere else
/// fails here rather than in someone's editor.
#[test]
fn the_animation_vocabulary_has_exactly_one_source() {
    for trigger in sail_lint::rules::animate_token_only::MOTION_TRIGGERS {
        let src = format!(r#"<card style({{ animate: {{ {trigger}: play(fade) }} }})> x</card>"#);
        assert!(
            compile(&src, opts()).is_ok(),
            "lint publishes `{trigger}` but the parser rejects it — the lists have drifted"
        );
    }
}

// ── TIMELINE TRIGGERS ───────────────────────────────────────────────────────

#[test]
fn reveal_and_scroll_are_triggers_like_any_other() {
    for trigger in ["reveal", "scroll"] {
        let src = format!(r#"<card style({{ animate: {{ {trigger}: play(fade) }} }})> x</card>"#);
        let js = compile(&src, opts()).expect("compiles").js;
        assert!(js.contains(&format!(r#"trigger: "{trigger}""#)), "{js}");
        assert!(
            js.contains("__sail_motion_attach"),
            "runtime attach present:\n{js}"
        );
    }
}

/// Parallax is `play(slide-up)` on a reveal timeline.
#[test]
fn parallax_is_one_directive() {
    let src = r#"<card style({ animate: { reveal: play(slide-up) } })> x</card>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(js.contains(r#"trigger: "reveal""#), "{js}");
    let motion_count = js.matches("property:").count();
    assert!(
        motion_count >= 2,
        "slide-up expands to opacity + translate-y:\n{js}"
    );
}

/// Every trigger the compiler advertises must be writable.
#[test]
fn every_advertised_trigger_can_actually_be_written() {
    let mut unwritable = Vec::new();
    for trigger in sail_lint::rules::animate_token_only::MOTION_TRIGGERS {
        let src = format!(r#"<card style({{ animate: {{ {trigger}: play(fade) }} }})> x</card>"#);
        match compile(&src, opts()) {
            Ok(out) => assert!(
                out.js.contains(&format!(r#"trigger: "{trigger}""#)),
                "`{trigger}` compiled but did not reach the runtime:\n{}",
                out.js
            ),
            Err(e) => unwritable.push(format!("  animate.{{ {trigger} }} -> {}", e.message)),
        }
    }
    assert!(
        unwritable.is_empty(),
        "the compiler lists these triggers and then refuses them — vocabulary you \
         cannot spell is worse than vocabulary you do not have:\n{}",
        unwritable.join("\n")
    );
}
