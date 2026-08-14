//! HTML/CSS realization of a Style program.
//!
//! Depends on `sail-style` (parse / IR / Intent). The domain crate must not
//! depend on this crate except as a `[dev-dependency]` for adapter tests.

pub mod emit;

pub use emit::{
    CssRule, emit_intent, emit_style, lower_style, lower_style_source,
    lower_style_source_with_motion, render_rule,
};

#[cfg(test)]
mod adapter_lock;
#[cfg(test)]
mod flat_chain;
