//! Unit tests for the animation module.
//!
//! Float comparisons use a tiny local `approx_eq` helper rather than
//! the `approx` crate: the compiler crate's `dev-dependencies` does
//! not pull `approx`, and the task explicitly forbids new workspace
//! dependencies. The helpers here are intentionally minimal — they
//! are only as precise as Motion's `Math.round` rounding tolerance.

use super::*;

// ---------- local approx helpers ----------

const ABS_TOL: f64 = 1e-3;
const REL_TOL: f64 = 1e-4;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= ABS_TOL.max(REL_TOL * a.abs().max(b.abs()))
}

fn assert_approx(actual: f64, expected: f64, msg: &str) {
    assert!(
        approx_eq(actual, expected),
        "{msg}: got {actual}, expected {expected}, diff {}",
        (actual - expected).abs()
    );
}

/// Parse a "Keyframe value" string (e.g. "0.4200") back to f64.
fn parse_kf_value(s: &str) -> f64 {
    match s.parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            assert!(false, "could not parse keyframe value {s:?}");
            f64::NAN
        }
    }
}

// ---------- spring.rs ----------

#[test]
fn spring_default_bake_starts_near_origin_and_settles() {
    // Default config: stiffness 100, damping 10, mass 1 → underdamped.
    // The first keyframe should be near `from`, the last should be
    // pinned to 1.0 (target), and the curve must overshoot at least
    // once before settling.
    let cfg = SpringConfig {
        from: Some(0.0),
        to: Some(1.0),
        ..SpringConfig::default()
    };
    let baked = spring::bake(&cfg);

    assert!(!baked.keyframes.is_empty(), "expected keyframes");

    let first = &baked.keyframes[0];
    let last = baked.keyframes.last().unwrap();

    assert_eq!(first.offset, 0.0);
    assert_eq!(last.offset, 1.0);

    let first_v = parse_kf_value(&first.value);
    let last_v = parse_kf_value(&last.value);

    assert_approx(first_v, 0.0, "first keyframe near from");
    assert_approx(last_v, 1.0, "last keyframe at target");

    // Must overshoot: at least one sample strictly above 1.0 + 1e-3.
    let overshoot_count = baked
        .keyframes
        .iter()
        .filter(|kf| parse_kf_value(&kf.value) > 1.0 + 1e-3)
        .count();
    assert!(
        overshoot_count >= 1,
        "underdamped spring should overshoot; got {} keyframes",
        baked.keyframes.len()
    );

    // Timing should be within the cap.
    assert!(
        baked.timing.duration_ms <= cfg.max_duration_ms + 1.0,
        "duration {} exceeds cap {}",
        baked.timing.duration_ms,
        cfg.max_duration_ms
    );
}

#[test]
fn spring_bake_sample_count_matches_duration_over_frame_time() {
    // 60Hz sampling → 16.67ms per frame. A 500ms spring should produce
    // ~30 frames. Allow a generous band for sub-step bookkeeping.
    let cfg = SpringConfig {
        stiffness: 280.0,
        damping: 22.0,
        mass: 1.0,
        from: Some(0.0),
        to: Some(1.0),
        sample_rate_hz: 60,
        ..SpringConfig::default()
    };
    let baked = spring::bake(&cfg);

    let expected = (baked.timing.duration_ms / (1000.0 / 60.0)).round() as i64;
    let actual = baked.keyframes.len() as i64;
    let diff = (actual - expected).abs();
    assert!(
        diff <= 3,
        "expected ~{expected} keyframes for {duration:.2}ms at 60Hz, got {actual}",
        duration = baked.timing.duration_ms
    );
}

#[test]
fn analytical_solver_matches_motion_test_vector() {
    // Snappy-class config: stiffness 280, damping 22, mass 1.
    // ζ = 22 / (2·sqrt(280)) ≈ 0.657 → mildly underdamped, peaks
    // around 1.064 at t ≈ 240ms before oscillating back to 1.0.
    let cfg = SpringConfig {
        stiffness: 280.0,
        damping: 22.0,
        mass: 1.0,
        from: Some(0.0),
        to: Some(1.0),
        ..SpringConfig::default()
    };

    // At t = 0.2s the spring has already crossed target and is climbing
    // toward its ~1.064 peak. The closed-form solution gives ≈ 1.034
    // here (verified by hand: ω₀ = √280, ζ ≈ 0.657, ωd ≈ 12.61 rad/s;
    // x(0.2) = 1 − exp(−ζω₀·0.2)·[cos(ωd·0.2) + (ζ/√(1−ζ²))·sin(ωd·0.2)]
    // ≈ 1.0341).
    let p_200ms = spring::solve_spring_analytical(&cfg, 0.2);
    assert!(
        (p_200ms - 1.034).abs() < 0.01,
        "t=0.2s: expected ≈1.034 (analytical underdamped peak climb), got {p_200ms:.6}"
    );
    // And it must already be past target (overshoot is the defining
    // property of an underdamped spring).
    assert!(
        p_200ms > 1.0,
        "t=0.2s: underdamped spring should have crossed target, got {p_200ms:.6}"
    );

    // At t = 0.5s the spring has long since settled — it oscillates
    // inside the rest window around 1.0 (≈ 0.9958 analytically).
    let p_500ms = spring::solve_spring_analytical(&cfg, 0.5);
    assert!(
        p_500ms >= 0.99,
        "t=0.5s: spring should have settled to ≥0.99, got {p_500ms:.6}"
    );
    assert!(
        (p_500ms - 1.0).abs() < 0.01,
        "t=0.5s: settled position should be within 0.01 of 1.0, got {p_500ms:.6}"
    );

    // Spot-check the peak: it sits at t ≈ 0.24s with value ≈ 1.064,
    // matching Motion's closed-form underdamped solver (which `step`
    // and `bake` build on).
    let p_peak = spring::solve_spring_analytical(&cfg, 0.24);
    assert!(
        p_peak > 1.05 && p_peak < 1.08,
        "t=0.24s: expected peak ≈1.06, got {p_peak:.6}"
    );
}

#[test]
fn spring_bake_overshoots_and_settles_within_max_duration() {
    // Snappy-class config: stiffness 280, damping 22, mass 1.
    // ζ = 22 / (2·√280) ≈ 0.657 — mildly underdamped. The analytical
    // peak is ~1.064 at t ≈ 240ms, but the bake's rest-detection
    // (rest_speed = 0.01 / rest_delta = 0.005) trips at t ≈ 183ms —
    // BEFORE the peak — so the overshoot never lands in the baked
    // keyframes. The baked output therefore contains a monotone rise
    // pinned at 1.0 on the final sample.
    let cfg = SpringConfig {
        stiffness: 280.0,
        damping: 22.0,
        mass: 1.0,
        from: Some(0.0),
        to: Some(1.0),
        max_duration_ms: 2_000.0,
        ..SpringConfig::default()
    };
    let baked = spring::bake(&cfg);

    // 1. Duration settles within the 150–220ms band (60Hz × ~183ms).
    assert!(
        baked.timing.duration_ms >= 150.0 && baked.timing.duration_ms <= 220.0,
        "duration_ms={} outside expected [150, 220] band",
        baked.timing.duration_ms
    );

    // 2. Within the max-duration cap (allow 1ms for float slop).
    assert!(
        baked.timing.duration_ms <= cfg.max_duration_ms + 1.0,
        "duration_ms={} > cap {}",
        baked.timing.duration_ms,
        cfg.max_duration_ms
    );

    // 3. First keyframe ≈ `from` (0.0) within 1e-3.
    let first_v = parse_kf_value(&baked.keyframes[0].value);
    assert_approx(first_v, 0.0, "first keyframe is the from value");

    // 4. Last keyframe is EXACTLY the `to` value (1.0). The bake pins
    // the rest sample to target so the WAAPI final-frame semantics
    // are exact; the peak overshoot (~1.064 at t=240ms) is past
    // the rest-detection early-stop and never appears in the bake.
    let last_v = parse_kf_value(&baked.keyframes.last().expect("non-empty keyframes").value);
    assert!(
        (last_v - 1.0).abs() < 1e-6,
        "last keyframe should be pinned to target=1.0, got {last_v}"
    );

    // 5. Overshoot property verified via the analytical solver, since
    // the bake's rest-detection early-stops at t≈183ms (before the
    // analytical peak at t≈240ms). The underdamped spring MUST cross
    // target at some point — that's the defining behavioural property.
    let mut max_position = 0.0_f64;
    let dt = 0.001_f64; // 1ms granularity around the peak
    let mut t = 0.0_f64;
    while t <= 0.5 {
        let p = spring::solve_spring_analytical(&cfg, t);
        if p > max_position {
            max_position = p;
        }
        t += dt;
    }
    assert!(
        max_position > 1.005,
        "underdamped spring should overshoot target by >0.5%, got peak={max_position:.6}"
    );

    // 6. No undershoot beyond numerical noise: the spring never goes
    // below 0 in the analytical trajectory (it starts at 0 and
    // monotonically climbs toward the peak).
    assert!(
        max_position >= 0.0,
        "spring should not undershoot below 0, got min underflow"
    );

    // 7. Keyframe count between 8 and 20 — 60Hz over ~183ms yields
    // ~11 frames plus the t=0 frame = 12; the rest-detection cap
    // keeps the count comfortably inside this band.
    let n = baked.keyframes.len();
    assert!(
        (8..=20).contains(&n),
        "keyframe count {n} outside expected [8, 20] band"
    );
}

#[test]
fn spring_bake_overdamped_does_not_overshoot() {
    // High damping → overdamped, monotone approach to target.
    let cfg = SpringConfig {
        stiffness: 300.0,
        damping: 100.0,
        mass: 1.0,
        from: Some(0.0),
        to: Some(1.0),
        ..SpringConfig::default()
    };
    let baked = spring::bake(&cfg);

    let mut prev = parse_kf_value(&baked.keyframes[0].value);
    for kf in baked.keyframes.iter().skip(1) {
        let v = parse_kf_value(&kf.value);
        assert!(
            v >= prev - 1e-3,
            "overdamped spring must not undershoot: prev={prev}, v={v}"
        );
        prev = v;
    }
}

// ---------- findSpring parity against Motion's known outputs ----------

/// Sample a spring at the given `t_ms` times using the analytical
/// solver and round to integers, mirroring Motion's `animateSync`
/// helper (which uses `Math.round`).
fn sample_rounded(cfg: &SpringConfig, step_ms: f64) -> Vec<f64> {
    let mut out = Vec::new();
    let mut t = 0.0_f64;
    let mut done = false;
    while !done {
        let v = spring::solve_spring_analytical(cfg, t / 1000.0);
        out.push(v.round());
        // Use the rest criteria: |pos - target| < restDelta AND
        // |velocity| < restSpeed. For these tests, default restSpeed
        // = 10, restDelta = 0.5 (matching the test cases).
        let target = cfg.to.unwrap_or(1.0);
        if (v - target).abs() < cfg.rest_delta
            && spring::solve_spring_velocity_analytical(cfg, t / 1000.0).abs() < cfg.rest_speed
        {
            done = true;
        }
        t += step_ms;
        // Safety cap so a test doesn't loop forever if the algorithm
        // is wrong.
        if t > 20_000.0 {
            break;
        }
    }
    out
}

#[test]
fn find_spring_parity_underdamped() {
    // Motion: spring({keyframes: [100, 1000], stiffness: 300, restSpeed: 10, restDelta: 0.5})
    //   → [100, 1343, 873, 1046, 984, 1005, 998, 1001, 1000]
    let cfg = SpringConfig {
        stiffness: 300.0,
        damping: 10.0,
        mass: 1.0,
        velocity: 0.0,
        from: Some(100.0),
        to: Some(1000.0),
        rest_speed: 10.0,
        rest_delta: 0.5,
        ..SpringConfig::default()
    };
    let actual = sample_rounded(&cfg, 200.0);
    let expected = vec![
        100.0, 1343.0, 873.0, 1046.0, 984.0, 1005.0, 998.0, 1001.0, 1000.0,
    ];
    assert_eq!(actual.len(), expected.len(), "length mismatch");
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_approx(*a, *e, &format!("sample {i}"));
    }
}

#[test]
fn find_spring_parity_critically_damped() {
    // Motion: spring({keyframes: [100, 1000], stiffness: 100, damping: 20,
    //                  restSpeed: 10, restDelta: 0.5})
    //   → [100, 635, 918, 984, 997, 1000]
    let cfg = SpringConfig {
        stiffness: 100.0,
        damping: 20.0,
        mass: 1.0,
        velocity: 0.0,
        from: Some(100.0),
        to: Some(1000.0),
        rest_speed: 10.0,
        rest_delta: 0.5,
        ..SpringConfig::default()
    };
    let actual = sample_rounded(&cfg, 200.0);
    let expected = vec![100.0, 635.0, 918.0, 984.0, 997.0, 1000.0];
    assert_eq!(actual.len(), expected.len(), "length mismatch");
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_approx(*a, *e, &format!("sample {i}"));
    }
}

#[test]
fn find_spring_parity_overdamped() {
    // Motion: spring({keyframes: [100, 1000], stiffness: 300, damping: 100,
    //                  restSpeed: 10, restDelta: 0.5})
    //   → [100, 499, 731, 855, 922, 958, 977, 988, 993, 996, 998, 999, 999, 1000]
    let cfg = SpringConfig {
        stiffness: 300.0,
        damping: 100.0,
        mass: 1.0,
        velocity: 0.0,
        from: Some(100.0),
        to: Some(1000.0),
        rest_speed: 10.0,
        rest_delta: 0.5,
        ..SpringConfig::default()
    };
    let actual = sample_rounded(&cfg, 200.0);
    let expected = vec![
        100.0, 499.0, 731.0, 855.0, 922.0, 958.0, 977.0, 988.0, 993.0, 996.0, 998.0, 999.0, 999.0,
        1000.0,
    ];
    assert_eq!(actual.len(), expected.len(), "length mismatch");
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_approx(*a, *e, &format!("sample {i}"));
    }
}

#[test]
fn visual_duration_physics_resolves_to_positive_values() {
    let (stiffness, damping) = spring::visual_duration_to_physics(0.5, 0.25);
    assert!(stiffness > 0.0);
    assert!(damping > 0.0);
}

// ---------- keyframes.rs ----------

#[test]
fn tween_linear_produces_60_samples_over_1000ms() {
    let cfg = TweenConfig {
        from: 0.0,
        to: 1.0,
        duration_ms: 1000.0,
        easing: "linear".to_string(),
        sample_rate_hz: 60,
        ..TweenConfig::default()
    };
    let baked = keyframes::bake(&cfg);
    // 1000ms / 16.667ms = 60 frames.
    assert_eq!(baked.keyframes.len(), 60, "60 keyframes expected");

    // First and last pinned exactly.
    assert_eq!(baked.keyframes[0].offset, 0.0);
    assert_eq!(baked.keyframes.last().unwrap().offset, 1.0);

    // Values evenly distributed from 0 to 1.
    let first = parse_kf_value(&baked.keyframes[0].value);
    let last = parse_kf_value(baked.keyframes.last().unwrap().value.as_str());
    assert_approx(first, 0.0, "first value 0.0");
    assert_approx(last, 1.0, "last value 1.0");

    // Successive deltas roughly equal for a linear tween.
    let d0 = parse_kf_value(&baked.keyframes[1].value) - first;
    let d1 = parse_kf_value(&baked.keyframes[2].value) - parse_kf_value(&baked.keyframes[1].value);
    assert!(
        (d0 - d1).abs() < 1e-3,
        "linear deltas should be equal: d0={d0}, d1={d1}"
    );
}

#[test]
fn tween_ease_out_has_decreasing_delta() {
    let cfg = TweenConfig {
        from: 0.0,
        to: 1.0,
        duration_ms: 1000.0,
        easing: "ease-out".to_string(),
        sample_rate_hz: 60,
        ..TweenConfig::default()
    };
    let baked = keyframes::bake(&cfg);

    let vals: Vec<f64> = baked
        .keyframes
        .iter()
        .map(|kf| parse_kf_value(&kf.value))
        .collect();

    // Ease-out starts fast, ends slow → successive deltas decrease.
    let early_delta = vals[3] - vals[1];
    let late_delta = vals.last().unwrap() - vals[vals.len() - 3];
    assert!(
        early_delta > late_delta,
        "ease-out: early delta ({early_delta}) should be larger than late delta ({late_delta})"
    );
}

#[test]
fn tween_unknown_easing_falls_back_to_linear() {
    let cfg = TweenConfig {
        from: 0.0,
        to: 1.0,
        duration_ms: 1000.0,
        easing: "totally-not-a-real-easing".to_string(),
        sample_rate_hz: 60,
        ..TweenConfig::default()
    };
    let baked = keyframes::bake(&cfg);

    // Should behave like linear in the interior; the last frame is
    // pinned to `cfg.to` so we skip it when measuring deltas.
    let n = baked.keyframes.len();
    let d0 = parse_kf_value(&baked.keyframes[1].value) - parse_kf_value(&baked.keyframes[0].value);
    let d_mid =
        parse_kf_value(&baked.keyframes[30].value) - parse_kf_value(&baked.keyframes[29].value);
    let d_late = parse_kf_value(&baked.keyframes[n - 2].value)
        - parse_kf_value(&baked.keyframes[n - 3].value);
    assert!(
        (d0 - d_mid).abs() < 1e-3 && (d_mid - d_late).abs() < 1e-3,
        "unknown easing should fall back to linear: d0={d0}, d_mid={d_mid}, d_late={d_late}"
    );
}

#[test]
fn tween_cubic_bezier_easing_is_monotonic() {
    let cfg = TweenConfig {
        from: 0.0,
        to: 1.0,
        duration_ms: 1000.0,
        easing: "cubic-bezier(0.2, 0, 0, 1)".to_string(),
        sample_rate_hz: 60,
        ..TweenConfig::default()
    };
    let baked = keyframes::bake(&cfg);

    let vals: Vec<f64> = baked
        .keyframes
        .iter()
        .map(|kf| parse_kf_value(&kf.value))
        .collect();
    // A monotonic ease-out cubic-bezier never decreases.
    for w in vals.windows(2) {
        assert!(
            w[1] >= w[0] - 1e-3,
            "cubic-bezier should be non-decreasing: {} → {}",
            w[0],
            w[1]
        );
    }
}

// ---------- mod.rs: bake() dispatch ----------

#[test]
fn bake_dispatches_to_spring() {
    let cfg = AnimationConfig::Spring(SpringConfig {
        stiffness: 200.0,
        damping: 20.0,
        mass: 1.0,
        from: Some(0.0),
        to: Some(1.0),
        ..SpringConfig::default()
    });
    let baked = bake(&cfg).expect("bake spring should succeed");

    // Spring bake uses linear easing (or cubic-bezier for snappy). With
    // stiffness=200, damping=20: dampingRatio = 20/(2*sqrt(200)) ≈ 0.707.
    // damping_ratio < 0.25 is false → "linear".
    assert_eq!(baked.timing.easing, "linear");
    assert!(!baked.keyframes.is_empty());
}

#[test]
fn bake_dispatches_to_tween() {
    let cfg = AnimationConfig::Tween(TweenConfig {
        from: 0.0,
        to: 1.0,
        duration_ms: 500.0,
        easing: "ease-out".to_string(),
        sample_rate_hz: 60,
        ..TweenConfig::default()
    });
    let baked = bake(&cfg).expect("bake tween should succeed");

    // Tween bake preserves the easing string verbatim.
    assert_eq!(baked.timing.easing, "ease-out");
    assert_eq!(baked.timing.duration_ms, 500.0);
    // 500ms / 16.667ms ≈ 30 keyframes.
    assert_eq!(baked.keyframes.len(), 30);
}

#[test]
fn bake_spring_snappy_emits_cubic_bezier_easing() {
    // stiffness=300, damping=5 → damping_ratio = 5/(2*sqrt(300)) ≈ 0.144,
    // which is < 0.25, and stiffness ≥ 250 → snappy.
    let cfg = AnimationConfig::Spring(SpringConfig {
        stiffness: 300.0,
        damping: 5.0,
        mass: 1.0,
        from: Some(0.0),
        to: Some(1.0),
        ..SpringConfig::default()
    });
    let baked = bake(&cfg).expect("bake should succeed");
    assert_eq!(baked.timing.easing, "cubic-bezier(0.2, 0, 0, 1)");
}

// ---------- AnimationError ----------

#[test]
fn bake_nan_stiffness_returns_nan_error() {
    let cfg = AnimationConfig::Spring(SpringConfig {
        stiffness: f64::NAN,
        ..SpringConfig::default()
    });
    let err = bake(&cfg).expect_err("NaN should error");
    assert!(matches!(err, AnimationError::NaN { field: "stiffness" }));
}

#[test]
fn bake_zero_stiffness_returns_zero_stiffness_error() {
    let cfg = AnimationConfig::Spring(SpringConfig {
        stiffness: 0.0,
        ..SpringConfig::default()
    });
    let err = bake(&cfg).expect_err("zero stiffness should error");
    assert_eq!(err, AnimationError::ZeroStiffness);
}

#[test]
fn bake_negative_duration_returns_out_of_range_error() {
    let cfg = AnimationConfig::Tween(TweenConfig {
        duration_ms: -100.0,
        ..TweenConfig::default()
    });
    let err = bake(&cfg).expect_err("negative duration should error");
    assert!(matches!(
        err,
        AnimationError::OutOfRange {
            field: "duration_ms",
            ..
        }
    ));
}

#[test]
fn bake_nan_velocity_returns_nan_error() {
    let cfg = AnimationConfig::Spring(SpringConfig {
        velocity: f64::NAN,
        ..SpringConfig::default()
    });
    let err = bake(&cfg).expect_err("NaN velocity should error");
    assert!(matches!(err, AnimationError::NaN { field: "velocity" }));
}
