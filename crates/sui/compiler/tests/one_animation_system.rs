//! ONE ANIMATION SYSTEM.
//!
//! There used to be two. `motion:` bakes `el.animate(...)` at compile time from a
//! validated catalog (four springs with real stiffness/damping, five tweens) and
//! ships no runtime animation library at all. Beside it sat `transition:` / `in:` /
//! `out:` — a second catalog (fade/slide/scale), a second registry
//! (`registerTransition`), a second runtime. Two vocabularies to learn, two things
//! to teach an agent, two places for a bug to live.
//!
//! The CAPABILITY the second system had is real and survives: holding an element in
//! the DOM until its exit animation finishes, which CSS cannot do alone. `if.ts` /
//! `for.ts` already await `runLeave` before removing nodes, and that seam is
//! untouched — it is simply backed by `motion:` now. So the fold is: same mechanism,
//! one catalog, one directive.

use sui_compiler::{compile, CompileMode, CompileOpts};

fn opts() -> CompileOpts {
    CompileOpts { mode: CompileMode::Component, ..Default::default() }
}

fn err_for(src: &str) -> String {
    let e = compile(src, opts()).expect_err("must be rejected");
    e.message
}

/// The lifecycle triggers exist and bake real animations — this is the capability
/// `transition:` used to own, now expressed in the one system.
#[test]
fn motion_enter_and_exit_bake_animations() {
    let src = r#"<card motion:enter={{ opacity: 'fade' }} motion:exit={{ opacity: 'fade-out' }}>x</card>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(js.contains(r#"trigger: "enter""#), "enter trigger emitted:\n{js}");
    assert!(js.contains(r#"trigger: "exit""#), "exit trigger emitted:\n{js}");
    // AOT-baked: the keyframes are `el.animate(...)` calls in the output, not a
    // runtime library resolving a preset name at play time.
    assert!(js.contains(".animate("), "animations are baked, not interpreted:\n{js}");
    assert!(js.contains("__sui_motion_attach"), "wired through the motion attach:\n{js}");
}

/// The interaction triggers still work — enter/exit are additions, not a rewrite.
#[test]
fn the_interaction_triggers_are_untouched() {
    let src = r#"<button motion:hover={{ scale: 'spring:snappy' }}>x</button>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(js.contains(r#"trigger: "hover""#), "{js}");
    assert!(js.contains(".animate("), "{js}");
}

/// An unknown preset is a COMPILE error, not a silent runtime fallback. This is why
/// a single catalog is worth having: it can be checked.
#[test]
fn an_unknown_preset_fails_the_build() {
    let msg = err_for(r#"<card motion:exit={{ opacity: 'wobble' }}>x</card>"#);
    assert!(
        msg.contains("wobble") || msg.to_lowercase().contains("preset"),
        "unknown preset must name itself: {msg}"
    );
}

/// The retired directives name their replacement. A retirement error that only says
/// "no" leaves the author (or the agent) guessing; this one teaches.
#[test]
fn the_second_animation_system_is_retired_with_a_teaching_error() {
    for (src, expect) in [
        (r#"<card transition:fade>x</card>"#, "motion:enter + motion:exit"),
        (r#"<card in:slide>x</card>"#, "motion:enter"),
        (r#"<card out:fade>x</card>"#, "motion:exit"),
    ] {
        let msg = err_for(src);
        assert!(
            msg.contains("retired") && msg.contains("one animation system"),
            "must say it is retired and why: {msg}"
        );
        assert!(msg.contains(expect), "must name the replacement `{expect}`: {msg}");
    }
}


// ── ORCHESTRATION ───────────────────────────────────────────────────────────
// Stagger is the difference between a list that APPEARS and a list that ARRIVES,
// and it is the thing you cannot get from a preset catalog or from CSS. It is also
// not a runtime engine: it is `delay = arrivalIndex * step` pushed into an
// already-baked WAAPI animation, so the cascade runs on the compositor like
// everything else here.

/// `stagger` reaches the emitted config as a plain number of milliseconds.
#[test]
fn stagger_bakes_a_delay_step_into_the_attach_config() {
    let src = r#"<card motion:enter={{ opacity: 'fade', stagger: '40ms' }}>x</card>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(js.contains("stagger: 40"), "stagger step emitted in ms:\n{js}");
    assert!(js.contains(r#"trigger: "enter""#), "{js}");
}

/// Seconds work too, and resolve to the same unit.
#[test]
fn stagger_accepts_seconds() {
    let src = r#"<card motion:enter={{ opacity: 'fade', stagger: '0.04s' }}>x</card>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(js.contains("stagger: 40"), "0.04s == 40ms:\n{js}");
}

/// An app that never staggers pays nothing — not even a `stagger: undefined`.
#[test]
fn no_stagger_emits_no_stagger() {
    let src = r#"<card motion:enter={{ opacity: 'fade' }}>x</card>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(!js.contains("stagger"), "unauthored stagger must not appear:\n{js}");
}

/// A garbled duration fails the build rather than silently becoming zero.
#[test]
fn a_bad_stagger_duration_fails_the_build() {
    let msg = err_for(r#"<card motion:enter={{ opacity: 'fade', stagger: 'soon' }}>x</card>"#);
    assert!(msg.contains("duration"), "must name the problem: {msg}");
}

/// THE DRIFT GUARD. The property vocabulary lived in TWO places — the compiler's
/// parser and the `sui-animate-token-only` lint — each with its own hand-copied
/// array, and a comment in the lint asserting they were the same set. They were not:
/// adding `stagger` to the parser compiled fine and then failed the lint, because a
/// comment is not a mechanism. They are one list in sui-lint now, and the preset
/// catalog is one JSON that both read. This test fails if anyone re-copies either.
#[test]
fn the_animation_vocabulary_has_exactly_one_source() {
    // Every property the lint publishes must be accepted by the parser...
    for prop in sui_lint::rules::animate_token_only::MOTION_PROPERTIES {
        let src = format!(r#"<card motion:hover={{{{ {prop}: 'spring:snappy' }}}}>x</card>"#);
        assert!(
            compile(&src, opts()).is_ok(),
            "lint publishes `{prop}` but the parser rejects it — the lists have drifted"
        );
    }
    // ...and a property NEITHER knows is rejected by the parser, naming the set.
    let msg = err_for(r#"<card motion:hover={{ blur: 'spring:snappy' }}>x</card>"#);
    assert!(msg.contains("not supported"), "{msg}");
    assert!(msg.contains("stagger"), "the error must mention the reserved key: {msg}");
}


// ── TIMELINE TRIGGERS ───────────────────────────────────────────────────────
// `motion:view` / `motion:scroll` are PROGRESS-driven, not time-driven: position is
// a function of scroll, not of the clock. Reveal-on-scroll and parallax, at ZERO JS
// runtime cost — no scroll listener, no rAF loop, no IntersectionObserver. The
// platform drives them off the main thread, which is why a scroll effect built this
// way stays smooth under load and one built on a scroll handler does not.
//
// And they are NOT a second emit path: the compiler bakes the same `el.animate(...)`
// it bakes for every other trigger, and the runtime hands the Animation a different
// timeline. One directive, one catalog, one code path — verified against a real
// chromium (opacity 0 -> 0.93 on scroll, driven through the actual runtime).

#[test]
fn view_and_scroll_are_triggers_like_any_other() {
    for trigger in ["view", "scroll"] {
        let src = format!(r#"<card motion:{trigger}={{{{ opacity: 'fade' }}}}>x</card>"#);
        let js = compile(&src, opts()).expect("compiles").js;
        assert!(js.contains(&format!(r#"trigger: "{trigger}""#)), "{js}");
        // baked exactly like the others — no separate CSS emit, no second path
        assert!(js.contains(".animate("), "still AOT-baked:\n{js}");
        assert!(js.contains("__sui_motion_attach"), "still the one attach:\n{js}");
    }
}

/// Parallax is `translate-y` on a view timeline. The point of the whole exercise:
/// the thing people reach for GSAP to do is a directive with a preset.
#[test]
fn parallax_is_one_directive() {
    let src = r#"<card motion:view={{ translate-y: 'spring:gentle', opacity: 'fade' }}>x</card>"#;
    let js = compile(src, opts()).expect("compiles").js;
    assert!(js.contains(r#"trigger: "view""#), "{js}");
    assert_eq!(js.matches(".animate(").count(), 2, "one baked animation per property:\n{js}");
}

/// Every trigger the compiler advertises must be writable. Enumerating the real
/// list is what keeps that honest: a trigger added to `MOTION_TRIGGERS` and
/// nowhere else fails here rather than in someone's editor.
#[test]
fn every_advertised_trigger_can_actually_be_written() {
    let mut unwritable = Vec::new();
    for trigger in sui_lint::rules::animate_token_only::MOTION_TRIGGERS {
        let src = format!(r#"<card motion:{trigger}={{{{ opacity: 'fade' }}}}>x</card>"#);
        match compile(&src, opts()) {
            Ok(out) => assert!(
                out.js.contains(&format!(r#"trigger: "{trigger}""#)),
                "`{trigger}` compiled but did not reach the runtime:\n{}",
                out.js
            ),
            Err(e) => unwritable.push(format!("  motion:{trigger} -> {}", e.message)),
        }
    }
    assert!(
        unwritable.is_empty(),
        "the compiler lists these triggers and then refuses them — vocabulary you \
         cannot spell is worse than vocabulary you do not have:\n{}",
        unwritable.join("\n")
    );
}
