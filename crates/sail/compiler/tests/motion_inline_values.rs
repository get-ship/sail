//! Inline values on the `fx:` surface — the grammar, its diagnostics,
//! and THE ACCEPTANCE.
//!
//! The calibration this file encodes: `fx:` is the ANIMATION LIBRARY's
//! surface. `sx` stays closed because colour and spacing are theming
//! integrity. A duration is a number of milliseconds; an easing is a curve;
//! a magnitude is how far the thing moves. Those are the author's to write,
//! and the duration-token episode (`--duration-cinematic` existing solely so
//! one hero loop could say "3600ms") is the mistake being undone here.
//!
//! The acceptance tests assert the three motion.dev archetypes are
//! expressible EXACTLY inline — no preset minted, no token minted.
//! Optional ground-truth cross-check: set `SAIL_MOTION_CAPTURE_JSON` to a
//! WAAPI capture JSON path when present.

use sail_compiler::{CompileMode, CompileOpts, compile};

fn compile_motion(source: &str) -> Result<String, String> {
    let opts = CompileOpts {
        scope_capabilities: Vec::new(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    compile(source, opts)
        .map(|out| out.js)
        .map_err(|e| format!("{e:?}"))
}

fn ok(source: &str) -> String {
    compile_motion(source).unwrap_or_else(|e| panic!("expected `{source}` to compile, got: {e}"))
}

fn err(source: &str) -> String {
    compile_motion(source).expect_err(&format!("expected `{source}` to be rejected"))
}

// ---------------------------------------------------------------- duration

#[test]
fn inline_duration_is_milliseconds() {
    // A bare number IS milliseconds. The runtime speaks seconds, so the
    // conversion happens once, at the compiler boundary.
    let js = ok(r#"<card> style({ animate: loop({ rotate: 'spin', duration: 3600 }) }) x</card>"#);
    assert!(
        js.contains("duration: 3.6"),
        "3600ms should emit as 3.6s:\n{js}"
    );
}

#[test]
fn inline_duration_overrides_the_presets_baked_token() {
    // `spin` bakes duration-glacial (14000ms). Inline must win — that is
    // what "presets are named DEFAULTS" means.
    let js = ok(r#"<card> style({ animate: loop({ rotate: 'spin', duration: 900 }) }) x</card>"#);
    assert!(
        js.contains("duration: 0.9"),
        "inline duration should win:\n{js}"
    );
    assert!(
        !js.contains("duration: 14"),
        "the preset's token must not survive:\n{js}"
    );
}

#[test]
fn inline_delay_is_milliseconds() {
    let js = ok(r#"<card> style({ animate: loop({ rotate: 'spin', delay: 200 }) }) x</card>"#);
    assert!(
        js.contains("delay: 0.2"),
        "200ms delay should emit as 0.2s:\n{js}"
    );
}

#[test]
fn a_quoted_duration_may_carry_its_unit() {
    let ms = ok(r#"<card> style({ animate: loop({ rotate: 'spin', duration: '3600ms' }) }) x</card>"#);
    let s = ok(r#"<card> style({ animate: loop({ rotate: 'spin', duration: '3.6s' }) }) x</card>"#);
    assert!(ms.contains("duration: 3.6"), "'3600ms' → 3.6s:\n{ms}");
    assert!(s.contains("duration: 3.6"), "'3.6s' → 3.6s:\n{s}");
}

#[test]
fn an_unquoted_unit_is_rejected_and_names_the_quoted_form() {
    let e = err(r#"<card> style({ animate: loop({ rotate: 'spin', duration: 3600ms }) }) x</card>"#);
    assert!(
        e.contains("must be quoted") && e.contains("'3600ms'"),
        "the error should name the quoted form: {e}"
    );
}

#[test]
fn a_negative_duration_is_rejected() {
    let e = err(r#"<card> style({ animate: loop({ rotate: 'spin', duration: -100 }) }) x</card>"#);
    assert!(
        e.contains("must not be negative"),
        "expected a range diagnostic: {e}"
    );
}

// ------------------------------------------------------------------ easing

#[test]
fn inline_cubic_bezier_becomes_the_css_function() {
    let js = ok(r#"<card> style({ animate: loop({ rotate: 'spin', easing: [0.6, 0, 0.2, 1] }) }) x</card>"#);
    assert!(
        js.contains(r#"easing: "cubic-bezier(0.6, 0, 0.2, 1)""#),
        "expected the bezier to emit as a CSS easing function:\n{js}"
    );
}

#[test]
fn named_easing_keywords_still_resolve() {
    let js = ok(r#"<card> style({ animate: loop({ rotate: 'spin', easing: 'in-out' }) }) x</card>"#);
    assert!(
        js.contains(r#"easing: "cubic-bezier(0.4, 0, 0.2, 1)""#),
        "the `in-out` token should resolve through the catalogue:\n{js}"
    );
}

#[test]
fn linear_and_raw_css_easings_pass_through() {
    let js = ok(r#"<card> style({ animate: loop({ rotate: 'spin', easing: 'linear' }) }) x</card>"#);
    assert!(
        js.contains(r#"easing: "linear""#),
        "expected passthrough:\n{js}"
    );
}

#[test]
fn a_malformed_bezier_names_the_correct_form() {
    let e = err(r#"<card> style({ animate: loop({ rotate: 'spin', easing: [0.6, 0, 0.2] }) }) x</card>"#);
    assert!(
        e.contains("exactly four numbers") && e.contains("[0.6, 0, 0.2, 1]"),
        "the error should name the correct bezier form: {e}"
    );
}

#[test]
fn a_bezier_with_out_of_range_x_is_rejected() {
    let e = err(r#"<card> style({ animate: loop({ rotate: 'spin', easing: [1.5, 0, 0.2, 1] }) }) x</card>"#);
    assert!(
        e.contains("x control points must be within 0..1"),
        "CSS will not accept it; catching it here beats a silent no-op: {e}"
    );
}

#[test]
fn a_numeric_easing_is_rejected_with_the_two_valid_forms() {
    let e = err(r#"<card> style({ animate: loop({ rotate: 'spin', easing: 400 }) }) x</card>"#);
    assert!(
        e.contains("keyword") && e.contains("inline bezier"),
        "expected both valid easing forms named: {e}"
    );
}

// -------------------------------------------------------------- magnitudes

#[test]
fn a_bare_inline_object_needs_no_preset() {
    let js = ok(r#"<card> style({ animate: loop({ translate-x: { from: 0, to: 120 } }) }) x</card>"#);
    assert!(js.contains("from: 0"), "expected inline from:\n{js}");
    assert!(js.contains("to: 120"), "expected inline to:\n{js}");
    assert!(
        js.contains(r#"kind: "tween""#),
        "a bare object is a tween:\n{js}"
    );
}

#[test]
fn a_unit_bearing_magnitude_is_preserved_verbatim() {
    // The rows archetype translates by -12%, which no preset could express:
    // the old catalogue's `slip` doc literally records "% is not expressible".
    let js = ok(r#"<card> style({ animate: loop({ translate-x: { from: 0, to: '-12%' } }) }) x</card>"#);
    assert!(
        js.contains(r#"to: "-12%""#),
        "the percent unit must survive:\n{js}"
    );
}

#[test]
fn a_numeric_string_and_a_bare_number_mean_the_same_thing() {
    let a = ok(r#"<card> style({ animate: loop({ scale: { to: 1.75 } }) }) x</card>"#);
    let b = ok(r#"<card> style({ animate: loop({ scale: { to: '1.75' } }) }) x</card>"#);
    assert!(a.contains("to: 1.75"), "bare number:\n{a}");
    assert!(
        b.contains("to: 1.75"),
        "quoted number should not become a string:\n{b}"
    );
}

#[test]
fn a_preset_supplies_defaults_that_inline_fields_override() {
    // `pop` is scale 1 → 1.75. Override just the `to`; `from` stays the
    // preset's. This is the "named default, not a cage" contract.
    let js = ok(r#"<card> style({ animate: loop({ scale: { preset: 'pulse', to: 1.9 } }) }) x</card>"#);
    assert!(
        js.contains("from: 1"),
        "the preset's `from` should survive:\n{js}"
    );
    assert!(js.contains("to: 1.9"), "the inline `to` should win:\n{js}");
}

#[test]
fn an_empty_inline_object_is_rejected() {
    let e = err(r#"<card> style({ animate: loop({ scale: { } }) }) x</card>"#);
    assert!(
        e.contains("declares no motion"),
        "expected a no-motion diagnostic: {e}"
    );
}

#[test]
fn a_bare_number_as_a_channel_value_is_rejected() {
    // `scale: 1.75` is ambiguous — is that a magnitude or a timing? Name the
    // fix rather than guessing.
    let e = err(r#"<card> style({ animate: loop({ scale: 1.75 }) }) x</card>"#);
    assert!(
        e.contains("from") && e.contains("to"),
        "the error should push the author into from/to: {e}"
    );
}

#[test]
fn an_unknown_channel_field_is_rejected() {
    let e = err(r#"<card> style({ animate: loop({ scale: { two: 1.75 } }) }) x</card>"#);
    assert!(
        e.contains("not a field of an inline motion object"),
        "expected field diagnostic: {e}"
    );
}

// ---------------------------------------------------------- per-channel timing

#[test]
fn a_channel_may_override_the_config_level_timing() {
    let js = ok(
        r#"<card> style({ animate: loop({ duration: 3000, scale: { to: 2, duration: 800 }, opacity: 'fade' }) }) x</card>"#,
    );
    assert!(
        js.contains("duration: 0.8"),
        "the channel override should emit:\n{js}"
    );
    assert!(
        js.contains("duration: 3"),
        "the config-level default should also emit:\n{js}"
    );
}

#[test]
fn config_level_timing_applies_to_every_channel() {
    let js = ok(
        r#"<card> style({ animate: loop({ translate-x: { to: 40 }, opacity: { to: 0 }, duration: 2000 }) }) x</card>"#,
    );
    assert_eq!(
        js.matches("duration: 2").count(),
        2,
        "both channels should carry it:\n{js}"
    );
}

#[test]
fn setting_a_timing_key_twice_is_rejected() {
    let e =
        err(r#"<card> style({ animate: loop({ rotate: 'spin', duration: 100, duration: 200 }) }) x</card>"#);
    assert!(
        e.contains("set twice"),
        "expected a duplicate-key diagnostic: {e}"
    );
}

// ------------------------------------------------------------------ stagger

#[test]
fn stagger_accepts_a_bare_number_of_milliseconds() {
    // `stagger: '450ms'` was ALREADY inline time — the number form is the
    // same idea with one less ceremony.
    let js = ok(r#"<row> style({ animate: enter({ opacity: 'fade', stagger: 450 }) }) x</row>"#);
    assert!(
        js.contains("stagger: 0.45"),
        "expected 450ms → 0.45s:\n{js}"
    );
}

#[test]
fn stagger_still_accepts_the_duration_string() {
    let js = ok(r#"<row> style({ animate: enter({ opacity: 'fade', stagger: '450ms' }) }) x</row>"#);
    assert!(
        js.contains("stagger: 0.45"),
        "expected 450ms → 0.45s:\n{js}"
    );
}

// ------------------------------------------------------------------ signals

#[test]
fn a_signal_valued_field_emits_a_reactive_thunk() {
    let js = ok(r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => { const speed = signal(1000); return { speed } })
</controller>
<card> style({ animate: loop({ rotate: 'spin', duration: speed() }) }) x</card>"#);
    assert!(
        js.contains("__sail_motion_attach") && js.contains("() => ({"),
        "a signal in the config should make the whole config a thunk:\n{js}"
    );
    assert!(
        js.contains("/ 1000"),
        "a signal duration is read as ms too:\n{js}"
    );
}

#[test]
fn a_signal_valued_magnitude_emits_a_reactive_thunk() {
    let js = ok(r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => { const dist = signal(120); return { dist } })
</controller>
<card> style({ animate: loop({ translate-x: { to: dist() }) } }) x</card>"#);
    assert!(
        js.contains("() => ({"),
        "expected a reactive config thunk:\n{js}"
    );
}

#[test]
fn a_fully_literal_config_stays_a_plain_object() {
    // No signal means no effect and no overhead — the static path must not
    // regress into reactivity just because the grammar grew.
    let js = ok(r#"<card> style({ animate: loop({ rotate: 'spin', duration: 900 }) }) x</card>"#);
    assert!(
        js.contains("__sail_motion_attach") && !js.contains("() => ({ trigger"),
        "a literal config should stay a plain object:\n{js}"
    );
}

#[test]
fn the_motion_disposer_is_registered_for_cleanup() {
    // It used to be dropped on the floor, which left an ambient loop running
    // against a node the framework had already torn down.
    let js = ok(r#"<card> style({ animate: loop({ rotate: 'spin' }) }) x</card>"#);
    assert!(
        js.contains(".push(__sail_motion_attach("),
        "the disposer must be registered on the cleanup list:\n{js}"
    );
}

#[test]
fn a_signal_that_is_not_in_scope_is_rejected() {
    let e = err(r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => { const speed = signal(1000); return { speed } })
</controller>
<card> style({ animate: loop({ rotate: 'spin', duration: nope() }) }) x</card>"#);
    assert!(
        e.contains("nope"),
        "an undefined signal in a motion config should read like one anywhere else: {e}"
    );
}

// --------------------------------------------------------------- THE ACCEPTANCE
//
// The three motion.dev archetypes, expressed inline, asserted against the
// captured ground truth. 2-stop per channel is the scope here; multi-stop
// timelines use `timeline()` + `(--x)` reflection.

/// Archetype A — the TILE. `span.home-diag-tile`: 3600ms,
/// `cubic-bezier(0.6, 0, 0.2, 1)`, translate 0→120px, rotate 0→15deg,
/// scale 1→1.75, three independent channels on one element.
///
/// Previously this needed THREE minted presets (`drift`/`tilt`/`pop`) and a
/// minted `--duration-cinematic` token. Now it is what it is.
#[test]
fn acceptance_tile_is_expressible_exactly_inline() {
    let js = ok(r#"<card> style({ animate: loop({
 translate-x: { from: 0, to: '120px' },
 rotate: { from: 0, to: '15deg' },
 scale: { from: 1, to: 1.75 },
 duration: 3600,
 easing: [0.6, 0, 0.2, 1]
 }) }) x</card>"#);
    // Ground truth: duration 3600ms.
    assert_eq!(
        js.matches("duration: 3.6").count(),
        3,
        "all three channels run at the captured 3600ms:\n{js}"
    );
    // Ground truth: cubic-bezier(0.6, 0, 0.2, 1) on every keyframe.
    assert_eq!(
        js.matches(r#"easing: "cubic-bezier(0.6, 0, 0.2, 1)""#)
            .count(),
        3,
        "all three channels carry the captured easing:\n{js}"
    );
    // Ground truth magnitudes: translate(120px) rotate(15deg) scale(1.75).
    assert!(
        js.contains(r#"to: "120px""#),
        "captured translate magnitude:\n{js}"
    );
    assert!(
        js.contains(r#"to: "15deg""#),
        "captured rotate magnitude:\n{js}"
    );
    assert!(js.contains("to: 1.75"), "captured scale magnitude:\n{js}");
    // Three independent channels, one attach call.
    assert_eq!(js.matches("property:").count(), 3, "three channels:\n{js}");
    assert_eq!(
        js.matches("__sail_motion_attach(").count(),
        1,
        "one attach:\n{js}"
    );
    // And nothing was minted to get here.
    assert!(!js.contains("drift"), "no bespoke preset needed:\n{js}");
    assert!(
        !js.contains("cinematic"),
        "no bespoke duration token needed:\n{js}"
    );
}

/// Archetype B — the ROWS. `span.home-diag-row`: 3000ms,
/// `cubic-bezier(0.4, 0, 0.2, 1)`, translate 0→-12%, opacity 1→0,
/// staggered 450ms across three siblings.
///
/// Note the `-12%`: the retired `slip` preset's own doc recorded that the
/// percentage was NOT expressible and approximated it in px. Inline values
/// close that.
#[test]
fn acceptance_rows_are_expressible_exactly_inline() {
    let js = ok(r#"<row> style({ animate: loop({
 translate-x: { from: 0, to: '-12%' },
 opacity: { from: 1, to: 0 },
 duration: 3000,
 easing: [0.4, 0, 0.2, 1],
 stagger: 450
 }) }) x</row>"#);
    assert_eq!(
        js.matches("duration: 3").count(),
        2,
        "captured 3000ms on both channels:\n{js}"
    );
    assert_eq!(
        js.matches(r#"easing: "cubic-bezier(0.4, 0, 0.2, 1)""#)
            .count(),
        2,
        "captured easing on both channels:\n{js}"
    );
    assert!(
        js.contains(r#"to: "-12%""#),
        "the captured PERCENTAGE, not a px approximation:\n{js}"
    );
    assert!(js.contains("from: 1"), "captured opacity from:\n{js}");
    assert!(js.contains("to: 0"), "captured opacity to:\n{js}");
    assert!(
        js.contains("stagger: 0.45"),
        "captured 450ms stagger:\n{js}"
    );
}

/// Archetype C — the BARS. `span.home-diag-bar-fill`: 2400ms,
/// `cubic-bezier(0.4, 0, 0.4, 1)`, scaleX 0→1, staggered 120ms across four
/// siblings. `scale-x` is the one genuine language addition the gauntlet
/// made; the timing and easing are now inline rather than a minted preset.
#[test]
fn acceptance_bars_are_expressible_exactly_inline() {
    let js = ok(r#"<row> style({ animate: loop({
 scale-x: { from: 0, to: 1 },
 duration: 2400,
 easing: [0.4, 0, 0.4, 1],
 stagger: 120
 }) }) x</row>"#);
    assert!(js.contains("duration: 2.4"), "captured 2400ms:\n{js}");
    assert!(
        js.contains(r#"easing: "cubic-bezier(0.4, 0, 0.4, 1)""#),
        "captured easing:\n{js}"
    );
    assert!(
        js.contains(r#"property: "scale-x""#),
        "single-axis fill:\n{js}"
    );
    assert!(js.contains("from: 0"), "captured from:\n{js}");
    assert!(js.contains("to: 1"), "captured to:\n{js}");
    assert!(
        js.contains("stagger: 0.12"),
        "captured 120ms stagger:\n{js}"
    );
    assert!(
        !js.contains("saunter"),
        "no bespoke duration token needed:\n{js}"
    );
}

/// Optional ground-truth cross-check against a WAAPI capture JSON.
/// Asserting the numbers keeps the archetype tests honest if a capture is
/// re-taken. Absent `SAIL_MOTION_CAPTURE_JSON` → skip (CI / packaged checkout).
#[test]
fn acceptance_numbers_match_the_captured_ground_truth() {
    let path = match std::env::var("SAIL_MOTION_CAPTURE_JSON") {
        Ok(p) if !p.is_empty() => p,
        _ => {
            eprintln!("SAIL_MOTION_CAPTURE_JSON unset; skipping ground-truth cross-check");
            return;
        }
    };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        eprintln!("ground-truth capture not readable at {path}; skipping cross-check");
        return;
    };
    for (target, duration, easing) in [
        (
            "home-diag-tile",
            "\"duration\": 3600",
            "cubic-bezier(0.6, 0, 0.2, 1)",
        ),
        (
            "home-diag-row",
            "\"duration\": 3000",
            "cubic-bezier(0.4, 0, 0.2, 1)",
        ),
        (
            "home-diag-bar-fill",
            "\"duration\": 2400",
            "cubic-bezier(0.4, 0, 0.4, 1)",
        ),
    ] {
        assert!(raw.contains(target), "capture should contain {target}");
        assert!(
            raw.contains(duration),
            "capture should contain {duration} for {target}"
        );
        assert!(
            raw.contains(easing),
            "capture should contain {easing} for {target}"
        );
    }
    // And the stagger steps the row/bar tests encode.
    assert!(raw.contains("\"delay\": 450"), "rows stagger step");
    assert!(raw.contains("\"delay\": 120"), "bars stagger step");
}
