//! SPEC-TICKER-1 — the loop is compile-time anatomy (animate: play(marquee)),
//! not a use: action. Track doubles at compile time; the seam copy is hidden.

use sail_compiler::{CompileOpts, compile};

#[test]
fn ticker_compiles_to_dual_track_with_motion_attach() {
    let out = compile(
        r#"<ticker><p>tokens · signals</p></ticker>"#,
        CompileOpts::default(),
    )
    .expect("ticker compiles");
    assert!(
        out.js.contains("__sail_motion_attach"),
        "the loop is motion engine anatomy at compile time:\n{}",
        out.js
    );
    assert!(out.js.contains("loop"), "loop trigger present:\n{}", out.js);
    assert!(
        out.js.matches("<span").count() >= 2,
        "compile-time dual track:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("use:marquee"),
        "no use:marquee stamp survives (ticker is pure structure):\n{}",
        out.js
    );
    assert!(
        !out.js.contains("marquee-action"),
        "no runtime marquee action import:\n{}",
        out.js
    );
}

#[test]
fn ticker_has_no_duration_or_reverse_props() {
    let out = compile(r#"<ticker><p>tokens</p></ticker>"#, CompileOpts::default())
        .expect("ticker compiles");
    assert!(
        !out.js.contains("duration"),
        "duration prop gone from ticker (canonical loop):\n{}",
        out.js
    );
    assert!(
        !out.js.contains("reverse"),
        "reverse prop gone from ticker (canonical loop):\n{}",
        out.js
    );
}

#[test]
fn ticker_track_translates_by_fifty_percent() {
    let out = compile(
        r#"<ticker><p>seamless</p></ticker>"#,
        CompileOpts::default(),
    )
    .expect("ticker compiles");
    assert!(
        out.js.contains("-50") || out.js.contains("-50%"),
        "the loop travels the half-track for the seamless seam:\n{}",
        out.js
    );
}
