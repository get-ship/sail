//! AOT spring-physics bake.
//!
//! Ports Motion's `findSpring` and the analytical/closed-form spring
//! solver from `packages/motion-dom/src/animation/generators/spring.ts`
//! into Rust, plus a separate RK4 integrator used by the bake.
//!
//! # AOT-bake convention
//!
//! The browser does not run physics at playback time — it gets a
//! pre-computed list of `(offset, value)` pairs and a single linear
//! `easing: "linear"` token. The spring's *shape* is in the keyframes,
//! not in the easing string. This keeps playback O(n) in the number of
//! samples and frees the runtime from any solver code.
//!
//! The snappy preset (high stiffness, low damping) is the one case where
//! the spec asks for a `cubic-bezier(0.2, 0, 0, 1)` easing token; the
//! keyframes are normalised to 0..=1 and the browser applies the
//! ease-out envelope. See [`bake`].
//!
//! # Units
//!
//! All inputs and outputs are in value units (px, %, etc.) and
//! milliseconds. Internally the analytical solver converts to seconds
//! because Motion's formulae are written in SI units; the RK4 bake
//! works in ms directly.

/// Closed-vocabulary defaults matching `springDefaults` in Motion.
/// `STIFFNESS`, `DAMPING`, `MASS` are the spring physics defaults;
/// `VELOCITY` is the default initial velocity. The duration/bounce
/// and rest-threshold defaults are not referenced here — they live
/// in the JSON preset catalogue, not the bake path.
mod defaults {
    pub const STIFFNESS: f64 = 100.0;
    pub const DAMPING: f64 = 10.0;
    pub const MASS: f64 = 1.0;
    pub const VELOCITY: f64 = 0.0;

    /// Granular rest-speed threshold — used when `|initialDelta| < 5`
    /// (i.e. small-range animations). Matches Motion's `restSpeed.granular`.
    pub const REST_SPEED_GRANULAR: f64 = 0.01;
    /// Granular rest-delta threshold. Matches Motion's `restDelta.granular`.
    pub const REST_DELTA_GRANULAR: f64 = 0.005;

    pub const MIN_DURATION_S: f64 = 0.01;
    pub const MAX_DURATION_S: f64 = 10.0;
    pub const MIN_DAMPING_RATIO: f64 = 0.05;
    pub const MAX_DAMPING_RATIO: f64 = 1.0;
    pub const MAX_GENERATOR_MS: f64 = 20_000.0;
}

/// Input configuration for a spring bake.
#[derive(Debug, Clone, PartialEq)]
pub struct SpringConfig {
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
    /// Velocity at t=0 in value/ms. Positive = moving away from `to`.
    pub velocity: f64,
    /// Optional explicit "from" / "to" values for the animated property.
    /// If both are `None`, the keyframes are 0..=1 normalised (the
    /// caller maps them to actual values). If provided, the keyframes
    /// interpolate directly.
    pub from: Option<f64>,
    pub to: Option<f64>,
    /// Target sampling rate in Hz. Default 60 (60fps). Clamped to
    /// `[12, 240]` to keep keyframe counts sensible.
    pub sample_rate_hz: u32,
    /// Rest thresholds. The spring is considered settled when both
    /// `|velocity|` is below `rest_speed` (value/ms) AND
    /// `|position - target|` is below `rest_delta`. Defaults match
    /// Motion's granular thresholds (0.01 / 0.005).
    pub rest_speed: f64,
    pub rest_delta: f64,
    /// Hard cap on total duration in ms. Default 10_000.
    pub max_duration_ms: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: defaults::STIFFNESS,
            damping: defaults::DAMPING,
            mass: defaults::MASS,
            velocity: defaults::VELOCITY,
            from: None,
            to: None,
            sample_rate_hz: 60,
            rest_speed: defaults::REST_SPEED_GRANULAR,
            rest_delta: defaults::REST_DELTA_GRANULAR,
            max_duration_ms: 10_000.0,
        }
    }
}

// ---------- `findSpring` port (duration/bounce -> stiffness/damping) ----------

/// Clamp `value` to `[min, max]`. Matches Motion's `clamp` helper.
fn clamp(min: f64, max: f64, value: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Newton iteration to find the root of `envelope(x) - safe_min = 0`.
/// Mirrors Motion's `approximateRoot` (12 iterations, hard cap on `safeMin`).
fn approximate_root(
    envelope: impl Fn(f64) -> f64,
    derivative: impl Fn(f64) -> f64,
    initial_guess: f64,
) -> f64 {
    const ROOT_ITERATIONS: usize = 12;
    const SAFE_MIN: f64 = 0.001;
    let _ = SAFE_MIN; // documented constant; not needed in this Rust port
    let mut result = initial_guess;
    for _ in 1..ROOT_ITERATIONS {
        let d = derivative(result);
        if d == 0.0 {
            break;
        }
        result -= envelope(result) / d;
    }
    result
}

/// Convert the Newton-iteration root into the public
/// `(stiffness, damping, duration_ms)` triple. NaN/infinity falls
/// back to Motion's default physics.
fn finalize(
    undamped_freq: f64,
    mass: f64,
    damping_ratio: f64,
    duration_ms: f64,
) -> (f64, f64, f64) {
    if undamped_freq.is_nan() || !undamped_freq.is_finite() {
        (defaults::STIFFNESS, defaults::DAMPING, duration_ms)
    } else {
        let stiffness = undamped_freq * undamped_freq * mass;
        let damping = damping_ratio * 2.0 * (mass * stiffness).sqrt();
        (stiffness, damping, duration_ms)
    }
}

/// Closed-form duration + bounce → (stiffness, damping, duration_ms).
///
/// Port of `findSpring` in `spring.ts`. Returns Motion's default
/// physics when the Newton iteration cannot find a root (NaN branch in
/// the original).
pub fn find_spring(duration_ms: f64, bounce: f64, velocity: f64, mass: f64) -> (f64, f64, f64) {
    // Clamp the damping ratio and duration. `dampingRatio = 1 - bounce`,
    // matching Motion.
    let mut damping_ratio = 1.0 - bounce;
    damping_ratio = clamp(
        defaults::MIN_DAMPING_RATIO,
        defaults::MAX_DAMPING_RATIO,
        damping_ratio,
    );

    let duration_s = clamp(
        defaults::MIN_DURATION_S,
        defaults::MAX_DURATION_S,
        duration_ms / 1000.0,
    );

    if damping_ratio < 1.0 {
        // Underdamped branch.
        fn envelope_ud(
            damping_ratio: f64,
            duration_s: f64,
            velocity: f64,
            undamped_freq: f64,
        ) -> f64 {
            let safe_min = 0.001_f64;
            let exponential_decay = undamped_freq * damping_ratio;
            let delta = exponential_decay * duration_s;
            let a = exponential_decay - velocity;
            let angular_freq =
                undamped_freq * (1.0 - damping_ratio * damping_ratio).max(0.0).sqrt();
            let b = angular_freq;
            let c = (-delta).exp();
            safe_min - (a / b) * c
        }
        fn derivative_ud(
            damping_ratio: f64,
            duration_s: f64,
            velocity: f64,
            undamped_freq: f64,
        ) -> f64 {
            let safe_min = 0.001_f64;
            let exponential_decay = undamped_freq * damping_ratio;
            let delta = exponential_decay * duration_s;
            let d = delta * velocity + velocity;
            let e = damping_ratio * damping_ratio * undamped_freq * undamped_freq * duration_s;
            let f = (-delta).exp();
            // `g = sqrt(undamped_freq^2 * (1 - damping_ratio^2))` — equivalent
            // to `undamped_freq * sqrt(1 - damping_ratio^2)` (which is `b`).
            let g = (undamped_freq * undamped_freq * (1.0 - damping_ratio * damping_ratio))
                .max(0.0)
                .sqrt();
            let env = envelope_ud(damping_ratio, duration_s, velocity, undamped_freq);
            let factor = if -env + safe_min > 0.0 { -1.0 } else { 1.0 };
            (factor * ((d - e) * f)) / g
        }
        let envelope = move |x: f64| envelope_ud(damping_ratio, duration_s, velocity, x);
        let derivative = move |x: f64| derivative_ud(damping_ratio, duration_s, velocity, x);
        let initial_guess = 5.0 / duration_s;
        let undamped_freq = approximate_root(envelope, derivative, initial_guess);
        finalize(undamped_freq, mass, damping_ratio, duration_s * 1000.0)
    } else {
        // Critically-damped branch (damping_ratio == 1.0 after clamp).
        fn envelope_cd(duration_s: f64, velocity: f64, undamped_freq: f64) -> f64 {
            let safe_min = 0.001_f64;
            let a = (-undamped_freq * duration_s).exp();
            let b = (undamped_freq - velocity) * duration_s + 1.0;
            -safe_min + a * b
        }
        fn derivative_cd(duration_s: f64, velocity: f64, undamped_freq: f64) -> f64 {
            let a = (-undamped_freq * duration_s).exp();
            let b = (velocity - undamped_freq) * (duration_s * duration_s);
            a * b
        }
        let envelope = move |x: f64| envelope_cd(duration_s, velocity, x);
        let derivative = move |x: f64| derivative_cd(duration_s, velocity, x);
        let initial_guess = 5.0 / duration_s;
        let undamped_freq = approximate_root(envelope, derivative, initial_guess);
        finalize(undamped_freq, mass, damping_ratio, duration_s * 1000.0)
    }
}

/// `visualDuration` (in seconds) → stiffness/damping using the same
/// derivation as Motion's `getSpringOptions` branch when `visualDuration`
/// is provided.
pub fn visual_duration_to_physics(visual_duration_s: f64, bounce: f64) -> (f64, f64) {
    let root = (2.0 * std::f64::consts::PI) / (visual_duration_s * 1.2);
    let stiffness = root * root;
    let damping_ratio = clamp(
        defaults::MIN_DAMPING_RATIO,
        defaults::MAX_DAMPING_RATIO,
        1.0 - bounce,
    );
    let damping = 2.0 * damping_ratio * stiffness.sqrt();
    (stiffness, damping)
}

// ---------- Analytical solver (closed-form position at time t) ----------

/// Closed-form spring position at time `t_seconds`.
///
/// Mirrors Motion's `resolveSpring` function. Used by the parity tests
/// against Motion's expected sample values; the bake itself uses RK4
/// to keep the integrator branch self-contained.
pub fn solve_spring_analytical(cfg: &SpringConfig, t_seconds: f64) -> f64 {
    let stiffness = cfg.stiffness;
    let damping = cfg.damping;
    let mass = cfg.mass;

    let origin = cfg.from.unwrap_or(0.0);
    let target = cfg.to.unwrap_or(1.0);

    let initial_velocity = cfg.velocity; // value/ms
    let initial_velocity_s = initial_velocity * 1000.0; // value/s

    let undamped_freq = (stiffness / mass).sqrt(); // 1/s
    let damping_ratio = damping / (2.0 * (stiffness * mass).sqrt());

    let initial_delta = target - origin;

    if damping_ratio < 1.0 {
        let angular_freq = undamped_freq * (1.0 - damping_ratio * damping_ratio).max(0.0).sqrt();
        let a_coeff =
            (initial_velocity_s + damping_ratio * undamped_freq * initial_delta) / angular_freq;

        let envelope = (-damping_ratio * undamped_freq * t_seconds).exp();
        target
            - envelope
                * (a_coeff * (angular_freq * t_seconds).sin()
                    + initial_delta * (angular_freq * t_seconds).cos())
    } else if (damping_ratio - 1.0).abs() < f64::EPSILON {
        // Critically damped.
        let envelope = (-undamped_freq * t_seconds).exp();
        let c = initial_velocity_s + undamped_freq * initial_delta;
        target - envelope * (initial_delta + c * t_seconds)
    } else {
        // Overdamped.
        let damped_angular_freq =
            undamped_freq * (damping_ratio * damping_ratio - 1.0).max(0.0).sqrt();

        let envelope = (-damping_ratio * undamped_freq * t_seconds).exp();
        let freq_for_t = (damped_angular_freq * t_seconds).min(300.0);

        let _p = (initial_velocity_s + damping_ratio * undamped_freq * initial_delta)
            / damped_angular_freq;

        target
            - (envelope
                * ((initial_velocity_s + damping_ratio * undamped_freq * initial_delta)
                    * freq_for_t.sinh()
                    + damped_angular_freq * initial_delta * freq_for_t.cosh()))
                / damped_angular_freq
    }
}

/// Closed-form spring velocity at time `t_seconds`, in value/s.
/// Mirrors Motion's `resolveVelocity`.
#[allow(dead_code)]
pub fn solve_spring_velocity_analytical(cfg: &SpringConfig, t_seconds: f64) -> f64 {
    let stiffness = cfg.stiffness;
    let damping = cfg.damping;
    let mass = cfg.mass;

    let origin = cfg.from.unwrap_or(0.0);
    let target = cfg.to.unwrap_or(1.0);

    let initial_velocity_s = cfg.velocity * 1000.0;

    let undamped_freq = (stiffness / mass).sqrt();
    let damping_ratio = damping / (2.0 * (stiffness * mass).sqrt());
    let initial_delta = target - origin;

    if damping_ratio < 1.0 {
        let angular_freq = undamped_freq * (1.0 - damping_ratio * damping_ratio).max(0.0).sqrt();
        let a_coeff =
            (initial_velocity_s + damping_ratio * undamped_freq * initial_delta) / angular_freq;
        let sin_coeff = damping_ratio * undamped_freq * a_coeff + initial_delta * angular_freq;
        let cos_coeff = damping_ratio * undamped_freq * initial_delta - a_coeff * angular_freq;

        let envelope = (-damping_ratio * undamped_freq * t_seconds).exp();
        envelope
            * (sin_coeff * (angular_freq * t_seconds).sin()
                + cos_coeff * (angular_freq * t_seconds).cos())
    } else if (damping_ratio - 1.0).abs() < f64::EPSILON {
        let envelope = (-undamped_freq * t_seconds).exp();
        let c = initial_velocity_s + undamped_freq * initial_delta;
        envelope * (undamped_freq * c * t_seconds - initial_velocity_s)
    } else {
        let damped_angular_freq =
            undamped_freq * (damping_ratio * damping_ratio - 1.0).max(0.0).sqrt();
        let envelope = (-damping_ratio * undamped_freq * t_seconds).exp();
        let freq_for_t = (damped_angular_freq * t_seconds).min(300.0);

        let p = (initial_velocity_s + damping_ratio * undamped_freq * initial_delta)
            / damped_angular_freq;
        let sinh_coeff = damping_ratio * undamped_freq * p - initial_delta * damped_angular_freq;
        let cosh_coeff = damping_ratio * undamped_freq * initial_delta - p * damped_angular_freq;

        envelope * (sinh_coeff * freq_for_t.sinh() + cosh_coeff * freq_for_t.cosh())
    }
}

// ---------- RK4 integrator ----------

/// One classical Runge-Kutta 4 step of the spring ODE
/// `ẍ = -(k/m)·x - (c/m)·ẋ` over `dt_s` seconds. Inputs/outputs in
/// value units and seconds. The integrator assumes `stiffness` is in
/// SI units (1/s²) and `damping` is in value/s — the same convention
/// Motion uses in its analytical solver. Callers convert from
/// ms / value/ms before invoking.
fn rk4_step_s(position: f64, velocity_s: f64, k: f64, c: f64, m: f64, dt_s: f64) -> (f64, f64) {
    // Acceleration as a function of (position, velocity).
    let acc = |x: f64, v: f64| -> f64 { (-k * x - c * v) / m };

    let k1v = velocity_s;
    let k1a = acc(position, velocity_s);

    let k2v = velocity_s + 0.5 * dt_s * k1a;
    let k2a = acc(position + 0.5 * dt_s * k1v, k2v);

    let k3v = velocity_s + 0.5 * dt_s * k2a;
    let k3a = acc(position + 0.5 * dt_s * k2v, k3v);

    let k4v = velocity_s + dt_s * k3a;
    let k4a = acc(position + dt_s * k3v, k4v);

    let new_pos = position + (dt_s / 6.0) * (k1v + 2.0 * k2v + 2.0 * k3v + k4v);
    let new_vel_s = velocity_s + (dt_s / 6.0) * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
    (new_pos, new_vel_s)
}

/// Spring's value at time `t_ms`. Uses RK4 over the ODE so the
/// integrator path is independent of the analytical solver above.
///
/// Internally the ODE is integrated in seconds (SI units, matching
/// Motion's `stiffness` convention); the caller's `t_ms` and the
/// `cfg.velocity` (value/ms) are converted at the boundary.
pub fn step(t_ms: f64, cfg: &SpringConfig) -> f64 {
    let stiffness = cfg.stiffness;
    let damping = cfg.damping;
    let mass = cfg.mass;

    let origin = cfg.from.unwrap_or(0.0);
    let target = cfg.to.unwrap_or(1.0);
    let total_delta = target - origin;

    // Solve y = position - target, so y(0) = -total_delta and y(∞) = 0.
    let initial_y = -total_delta;
    let initial_velocity_s = cfg.velocity * 1000.0; // value/ms → value/s

    let dt_s = t_ms.max(0.0) / 1000.0;
    let sub_step_s = 0.004_f64; // 4ms sub-steps for stability with stiff springs
    let sub_steps = ((dt_s / sub_step_s).ceil() as usize).max(1);
    let h = dt_s / sub_steps as f64;

    let mut pos = initial_y;
    let mut vel_s = initial_velocity_s;
    for _ in 0..sub_steps {
        let (np, nv) = rk4_step_s(pos, vel_s, stiffness, damping, mass, h);
        pos = np;
        vel_s = nv;
    }

    pos + target
}

// ---------- Bake ----------

/// Convert a normalised sample to the `value` field of a [`Keyframe`].
fn format_value(normalised: f64, from: Option<f64>, to: Option<f64>) -> String {
    match (from, to) {
        (Some(f), Some(t)) => {
            let v = f + (t - f) * normalised;
            // 4 decimal places — enough resolution for CSS animation.
            format!("{v:.4}")
        }
        _ => format!("{normalised:.4}"),
    }
}

/// Bake a spring config into a [`BakedAnimation`].
///
/// Samples the spring at `cfg.sample_rate_hz` Hz (default 60), walking
/// forward with RK4 until one of:
/// - the spring is at rest (`|velocity| < rest_speed` AND
///   `|position - target| < rest_delta`),
/// - `t` exceeds `cfg.max_duration_ms`.
///
/// The first keyframe is at `offset = 0.0`; the last is at `1.0`; the
/// offsets between are linear in `t`. The browser interpolates
/// linearly between samples; with a 60Hz sample rate the curve looks
/// smooth.
///
/// The `easing` field of the returned [`super::AnimationTiming`] is
/// `"linear"` for a regular spring (shape is in the keyframes) or
/// `"cubic-bezier(0.2, 0, 0, 1)"` for the snappy preset — see the
/// module doc comment for why.
pub fn bake(cfg: &SpringConfig) -> super::BakedAnimation {
    let sample_rate_hz = cfg.sample_rate_hz.clamp(12, 240);
    let dt_ms = 1000.0 / sample_rate_hz as f64;

    let origin = cfg.from.unwrap_or(0.0);
    let target = cfg.to.unwrap_or(1.0);
    let total_delta = target - origin;

    // Solve the homogeneous ODE for `y = position - target`, so the rest
    // value is 0 and the relative initial displacement is `-total_delta`.
    let stiffness = cfg.stiffness;
    let damping = cfg.damping;
    let mass = cfg.mass;
    // Solve y = position - target, so y(0) = -total_delta and y(∞) = 0.
    let initial_y = -total_delta;
    let initial_velocity_s = cfg.velocity * 1000.0; // value/ms → value/s

    let rest_speed = cfg.rest_speed;
    let rest_delta = cfg.rest_delta;
    let max_duration_ms = cfg.max_duration_ms.clamp(10.0, defaults::MAX_GENERATOR_MS);

    let dt_s_per_frame = dt_ms / 1000.0;
    let sub_step_s = 0.004_f64;
    let sub_steps = ((dt_s_per_frame / sub_step_s).ceil() as usize).max(1);
    let h = dt_s_per_frame / sub_steps as f64;

    let mut samples: Vec<(f64, f64)> = Vec::with_capacity(1024);
    let mut pos = initial_y;
    let mut vel_s = initial_velocity_s;
    let mut t = 0.0_f64;

    samples.push((0.0, pos + target));

    let mut settled = false;
    let mut end_t = max_duration_ms;

    while t < max_duration_ms {
        for _ in 0..sub_steps {
            let (np, nv) = rk4_step_s(pos, vel_s, stiffness, damping, mass, h);
            pos = np;
            vel_s = nv;
        }
        t += dt_ms;
        // Pin the recorded sample to target when the spring has
        // settled — the integrator may land a few µm above target
        // inside the rest window, but the WAAPI final-frame semantics
        // are exact.
        let recorded_pos = if (vel_s / 1000.0).abs() < rest_speed && pos.abs() < rest_delta {
            target
        } else {
            pos + target
        };
        samples.push((t, recorded_pos));

        // Rest check: velocity in value/ms (`vel_s / 1000`) AND
        // |pos - target| < rest_delta.
        if (vel_s / 1000.0).abs() < rest_speed && pos.abs() < rest_delta {
            settled = true;
            end_t = t;
            break;
        }
    }

    // Build keyframes. Normalise t to [0, 1] across the recorded range.
    let duration_ms = if settled { end_t } else { max_duration_ms };
    let last_idx = samples.len() - 1;

    let keyframes: Vec<super::Keyframe> = samples
        .iter()
        .enumerate()
        .map(|(i, (sample_t, sample_pos))| {
            let offset = if i == 0 {
                0.0
            } else if i == last_idx {
                1.0
            } else {
                (sample_t / duration_ms).clamp(0.0, 1.0)
            };
            // Map absolute position back to normalised 0..=1 if from/to were None.
            let normalised = match (cfg.from, cfg.to) {
                (Some(_), Some(_)) => {
                    let raw = *sample_pos;
                    if total_delta.abs() < f64::EPSILON {
                        1.0
                    } else {
                        (raw - origin) / total_delta
                    }
                }
                _ => {
                    let raw = *sample_pos;
                    if total_delta.abs() < f64::EPSILON {
                        1.0
                    } else {
                        (raw - origin) / total_delta
                    }
                }
            };
            super::Keyframe {
                property: "value".into(),
                offset,
                value: format_value(normalised, cfg.from, cfg.to),
            }
        })
        .collect();

    // Easing: linear for a normal spring (shape baked into keyframes).
    // For the snappy preset (high stiffness + low damping) emit the
    // cubic-bezier envelope; the browser applies the ease-out to the
    // linearly-interpolated samples. Heuristic: snappy iff stiffness
    // is large and damping ratio is low.
    let damping_ratio = damping / (2.0 * (stiffness * mass).sqrt());
    let snappy = stiffness >= 250.0 && damping_ratio < 0.25;
    let easing = if snappy {
        "cubic-bezier(0.2, 0, 0, 1)".to_string()
    } else {
        "linear".to_string()
    };

    super::BakedAnimation {
        keyframes,
        timing: super::AnimationTiming {
            duration_ms,
            easing,
            delay_ms: 0.0,
            iterations: 1.0,
            direction: "normal".to_string(),
            fill: "both".to_string(),
        },
    }
}

/// Maximum spring duration supported by the bake loop (matches
/// `maxGeneratorDuration` in Motion's `calc-duration.ts`).
pub const MAX_GENERATOR_DURATION_MS: f64 = defaults::MAX_GENERATOR_MS;
