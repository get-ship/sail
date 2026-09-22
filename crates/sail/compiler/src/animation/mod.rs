//! AOT animation bake — pure functions that turn declarative animation
//! configs into static keyframe arrays + timing the browser can play
//! back via `element.animate()`. No runtime solver.
//!
//! # Why AOT bake
//!
//! `.sail` files compile to ESM that runs in the browser. Running a
//! spring solver per frame would (a) require shipping solver code and
//! (b) tie runtime animation work to the JS thread. Instead, we
//! resolve the spring (or tween) at compile time into a list of
//! `(offset, value)` pairs and a single linear easing token. The
//! browser interpolates linearly between the samples; at 60Hz the
//! curve looks smooth. See [`spring`] for the physics port and
//! [`keyframes`] for CSS-style tweens.
//!
//! # Snappy preset and the cubic-bezier exception
//!
//! A handful of presets — `spring:snappy`, etc. — bake the easing as
//! `cubic-bezier(0.2, 0, 0, 1)` instead of `linear`. In that case
//! the keyframes are normalised to `0..=1` and the browser applies
//! the cubic-bezier to the linearly-interpolated samples. The bake
//! decides which easing to emit based on the resolved stiffness and
//! damping ratio: snappy iff `stiffness ≥ 250` AND `damping_ratio
//! < 0.25`. All other springs use the linear easing + shape-in-
//! keyframes convention.
//!
//! # Public surface
//!
//! The compiler-side IR lowers a `fx:` directive into an
//! [`AnimationConfig`] (after token resolution). Callers invoke
//! [`bake`] and embed the resulting [`BakedAnimation`] in the emitted
//! JS so the runtime can pass it straight to `element.animate()`.
//!
//! All functions are pure and WASM-safe: no filesystem, no threads,
//! no `std::time`. The module compiles under `target_arch = "wasm32"`
//! with no additional feature flags.

pub mod keyframes;
pub mod spring;

#[cfg(test)]
mod tests;

use thiserror::Error;

/// A property+value at a specific point in a keyframe array. Mirrors
/// the shape WAAPI's `Keyframe` constructor accepts, but as plain
/// strings/floats so it can be serialised to JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    /// Property name (e.g. `"transform"`, `"opacity"`, `"translate"`).
    pub property: String,
    /// `0.0..=1.0` offset within the animation timeline.
    pub offset: f64,
    /// Numeric or string value. For transforms, this is a
    /// space-separated list of CSS transform functions
    /// (`"scale(0.96) translateY(4px)"`). For numeric properties
    /// (opacity, translate, etc.) it's a fixed-decimal string like
    /// `"0.4200"`.
    pub value: String,
}

/// Animation timing — mirrors the `options` argument of
/// `element.animate()`. The browser reads these fields verbatim;
/// callers should not transform them.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationTiming {
    pub duration_ms: f64,
    /// `"linear"` for spring bakes (shape is in the keyframes) or
    /// `"cubic-bezier(...)"` for the snappy preset. For tweens this
    /// is whatever CSS easing string the caller passed in.
    pub easing: String,
    pub delay_ms: f64,
    /// `1.0` for once, `f64::INFINITY` for infinite. WAAPI also
    /// accepts integer counts (`2.0`).
    pub iterations: f64,
    /// `"normal"` | `"reverse"` | `"alternate"`.
    pub direction: String,
    /// `"none"` | `"forwards"` | `"backwards"` | `"both"`.
    pub fill: String,
}

impl Default for AnimationTiming {
    fn default() -> Self {
        Self {
            duration_ms: 200.0,
            easing: "ease-out".to_string(),
            delay_ms: 0.0,
            iterations: 1.0,
            direction: "normal".to_string(),
            fill: "both".to_string(),
        }
    }
}

/// What the compiler produces for a single `fx:` directive.
#[derive(Debug, Clone, PartialEq)]
pub struct BakedAnimation {
    pub keyframes: Vec<Keyframe>,
    pub timing: AnimationTiming,
}

/// Spring configuration. Re-exported at the module root so callers
/// don't have to import the submodule.
pub use spring::SpringConfig;

/// Tween (non-spring) configuration. Re-exported at the module root
/// for the same reason.
pub use keyframes::TweenConfig;

/// Discriminated union of all animation configs the compiler
/// accepts. The `fx:` IR lowers into this enum after token
/// resolution; [`bake`] dispatches on the variant.
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationConfig {
    Spring(SpringConfig),
    Tween(TweenConfig),
}

/// Errors that [`bake`] can produce. All variants are recoverable —
/// callers should surface them as `CompileError` with a span pointing
/// at the `fx:` directive in the user's `.sail` source.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum AnimationError {
    /// A numeric input was NaN (e.g. a token that didn't resolve to a
    /// number, or a divide-by-zero that produced NaN).
    #[error("animation config contained NaN: {field}")]
    NaN { field: &'static str },
    /// Spring stiffness was zero — the ODE degenerates into a free
    /// particle and never settles.
    #[error("spring stiffness must be > 0")]
    ZeroStiffness,
    /// A duration or other numeric param was negative or otherwise
    /// out of the accepted range.
    #[error("value out of range: {field} = {value}")]
    OutOfRange { field: &'static str, value: f64 },
}

/// Validate that a numeric value is finite. Returns
/// `AnimationError::NaN` on NaN or infinity, with the field name for
/// diagnostics.
fn check_finite(field: &'static str, value: f64) -> Result<(), AnimationError> {
    if !value.is_finite() {
        Err(AnimationError::NaN { field })
    } else {
        Ok(())
    }
}

/// Bake an [`AnimationConfig`] into a [`BakedAnimation`].
///
/// Dispatches to [`spring::bake`] for the `Spring` variant and
/// [`keyframes::bake`] for the `Tween` variant, after validating that
/// all numeric inputs are finite and non-negative where required.
pub fn bake(config: &AnimationConfig) -> Result<BakedAnimation, AnimationError> {
    match config {
        AnimationConfig::Spring(s) => {
            check_finite("stiffness", s.stiffness)?;
            check_finite("damping", s.damping)?;
            check_finite("mass", s.mass)?;
            check_finite("velocity", s.velocity)?;
            check_finite("rest_speed", s.rest_speed)?;
            check_finite("rest_delta", s.rest_delta)?;
            check_finite("max_duration_ms", s.max_duration_ms)?;
            if s.stiffness <= 0.0 {
                return Err(AnimationError::ZeroStiffness);
            }
            if s.max_duration_ms < 0.0 {
                return Err(AnimationError::OutOfRange {
                    field: "max_duration_ms",
                    value: s.max_duration_ms,
                });
            }
            if s.mass <= 0.0 {
                return Err(AnimationError::OutOfRange {
                    field: "mass",
                    value: s.mass,
                });
            }
            Ok(spring::bake(s))
        }
        AnimationConfig::Tween(t) => {
            check_finite("from", t.from)?;
            check_finite("to", t.to)?;
            check_finite("duration_ms", t.duration_ms)?;
            check_finite("delay_ms", t.delay_ms)?;
            check_finite("iterations", t.iterations)?;
            if t.duration_ms < 0.0 {
                return Err(AnimationError::OutOfRange {
                    field: "duration_ms",
                    value: t.duration_ms,
                });
            }
            if t.delay_ms < 0.0 {
                return Err(AnimationError::OutOfRange {
                    field: "delay_ms",
                    value: t.delay_ms,
                });
            }
            Ok(keyframes::bake(t))
        }
    }
}
