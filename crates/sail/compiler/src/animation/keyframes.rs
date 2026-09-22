//! AOT keyframe bake for non-spring tweens (CSS-style easing).
//!
//! The browser already understands CSS easing functions, so this
//! module is much simpler than [`super::spring`]: we sample at the
//! configured rate, apply the easing to the progress `p ∈ [0, 1]`,
//! and emit `(offset, eased_value)` pairs.
//!
//! Unknown easing strings fall back to `linear`. We do not call
//! `tracing::warn!` because the compiler crate does not depend on
//! `tracing` (the workspace crate `tracing` is hoisted but only
//! pulled in by members that ask for it; the compiler module does
//! not, and the task explicitly forbids new workspace deps). The
//! fallback is silent — an unknown easing produces a linear animation
//! rather than an error.
//!
//! All functions in this module are pure: no I/O, no threads, no
//! `std::time`. Safe to compile under `target_arch = "wasm32"`.

use super::{AnimationTiming, BakedAnimation, Keyframe};

/// Input configuration for a tween bake.
#[derive(Debug, Clone, PartialEq)]
pub struct TweenConfig {
    pub from: f64,
    pub to: f64,
    pub duration_ms: f64,
    /// CSS easing string verbatim. Recognised (case-insensitive):
    /// `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`,
    /// and `cubic-bezier(x1, y1, x2, y2)`. Anything else falls
    /// back to `linear`.
    pub easing: String,
    pub delay_ms: f64,
    /// `1.0` for once, `f64::INFINITY` for infinite. WAAPI also
    /// accepts a count like `2.0`. We pass through the f64.
    pub iterations: f64,
    /// Target sampling rate in Hz. Default 60, clamped `[12, 240]`.
    pub sample_rate_hz: u32,
}

impl Default for TweenConfig {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 1.0,
            duration_ms: 1000.0,
            easing: "linear".to_string(),
            delay_ms: 0.0,
            iterations: 1.0,
            sample_rate_hz: 60,
        }
    }
}

/// Apply a recognised easing string to a progress value `p ∈ [0, 1]`.
/// Returns the eased progress. Unknown strings → identity (linear).
pub fn apply_easing(easing: &str, p: f64) -> f64 {
    let trimmed = easing.trim();
    let lower = trimmed.to_ascii_lowercase();
    let p = p.clamp(0.0, 1.0);

    match lower.as_str() {
        "linear" => p,
        "ease" => cubic_bezier(0.25, 0.1, 0.25, 1.0, p),
        "ease-in" => cubic_bezier(0.42, 0.0, 1.0, 1.0, p),
        "ease-out" => cubic_bezier(0.0, 0.0, 0.58, 1.0, p),
        "ease-in-out" => cubic_bezier(0.42, 0.0, 0.58, 1.0, p),
        _ => {
            if let Some((x1, y1, x2, y2)) = parse_cubic_bezier(trimmed) {
                cubic_bezier(x1, y1, x2, y2, p)
            } else {
                // Unknown easing — silently fall back to linear.
                p
            }
        }
    }
}

/// Parse a `cubic-bezier(x1, y1, x2, y2)` string. Returns `None` if
/// the string doesn't match the syntax or the numbers don't parse.
fn parse_cubic_bezier(s: &str) -> Option<(f64, f64, f64, f64)> {
    let s = s.trim();
    let prefix = "cubic-bezier(";
    let lower = s.to_ascii_lowercase();
    if !lower.starts_with(prefix) || !s.ends_with(')') {
        return None;
    }
    let inner = &s[prefix.len()..s.len() - 1];
    let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
    if parts.len() != 4 {
        return None;
    }
    let x1 = parts[0].parse::<f64>().ok()?;
    let y1 = parts[1].parse::<f64>().ok()?;
    let x2 = parts[2].parse::<f64>().ok()?;
    let y2 = parts[3].parse::<f64>().ok()?;
    // CSS requires x1 and x2 in [0, 1] for a valid cubic-bezier.
    if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
        return None;
    }
    Some((x1, y1, x2, y2))
}

/// Evaluate a CSS cubic-bezier easing at progress `p`. Uses Newton
/// iteration to invert the bezier curve (the x-axis is time, y-axis
/// is progress), then returns the corresponding y value.
fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, p: f64) -> f64 {
    // Sample t that gives x(t) ≈ p via a few Newton iterations, then
    // evaluate y(t). Falls back to bisection if Newton diverges.
    let sample = |t: f64| -> (f64, f64, f64, f64) {
        let it = 1.0 - t;
        // x(t), y(t), x'(t), y'(t) for the cubic-bezier parametric form.
        let cx = 3.0 * it * it * t * x1 + 3.0 * it * t * t * x2 + t * t * t;
        let cy = 3.0 * it * it * t * y1 + 3.0 * it * t * t * y2 + t * t * t;
        let dx = 3.0 * it * it * x1 + 6.0 * it * t * (x2 - x1) + 3.0 * t * t * (1.0 - x2);
        let dy = 3.0 * it * it * y1 + 6.0 * it * t * (y2 - y1) + 3.0 * t * t * (1.0 - y2);
        (cx, cy, dx, dy)
    };

    // Initial guess: linear.
    let mut t = p;
    for _ in 0..8 {
        let (cx, _, dx, _) = sample(t);
        let err = cx - p;
        if err.abs() < 1e-7 {
            break;
        }
        if dx.abs() < 1e-9 {
            break;
        }
        t -= err / dx;
        t = t.clamp(0.0, 1.0);
    }
    let (_, cy, _, _) = sample(t);
    cy.clamp(0.0, 1.0)
}

/// Bake a tween into a [`BakedAnimation`].
///
/// Samples the easing function at `cfg.sample_rate_hz` Hz (default 60,
/// clamped to `[12, 240]`). The first keyframe is at `offset = 0.0`,
/// the last at `1.0`; the easing is applied to the progress so the
/// emitted `value` field reflects the eased curve, not the linear
/// progress.
pub fn bake(cfg: &TweenConfig) -> BakedAnimation {
    let sample_rate_hz = cfg.sample_rate_hz.clamp(12, 240);
    let dt_ms = 1000.0 / sample_rate_hz as f64;

    // Cap duration at MAX_GENERATOR_DURATION_MS for safety.
    let duration_ms = cfg
        .duration_ms
        .clamp(0.01, super::spring::MAX_GENERATOR_DURATION_MS);

    let num_samples = ((duration_ms / dt_ms).round() as usize).max(2);

    let keyframes: Vec<Keyframe> = (0..num_samples)
        .map(|i| {
            let t = i as f64 * dt_ms;
            let offset = (t / duration_ms).clamp(0.0, 1.0);
            let progress = offset;
            let eased = apply_easing(&cfg.easing, progress);
            let value = cfg.from + (cfg.to - cfg.from) * eased;
            Keyframe {
                property: "value".into(),
                offset,
                value: format!("{value:.4}"),
            }
        })
        .collect();

    // Force first offset 0.0 and last offset 1.0 exactly, in case of
    // rounding at the ends.
    let mut keyframes = keyframes;
    if let Some(first) = keyframes.first_mut() {
        first.offset = 0.0;
    }
    if let Some(last) = keyframes.last_mut() {
        last.offset = 1.0;
        last.value = format!("{:.4}", cfg.to);
    }

    BakedAnimation {
        keyframes,
        timing: AnimationTiming {
            duration_ms,
            easing: cfg.easing.clone(),
            delay_ms: cfg.delay_ms,
            iterations: cfg.iterations,
            direction: "normal".to_string(),
            fill: "both".to_string(),
        },
    }
}
