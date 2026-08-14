//! CSS adapter — HTML/CSS realization of Intent.
//!
//! This module may import domain. Domain modules must not import this module.

pub mod conditions;
pub mod emit;
pub mod prefixes;

pub use conditions::{CONDITION_PREFIXES, condition_media};
pub use emit::{
    CssRule, emit_style, lower_style, lower_style_source, lower_style_source_with_motion,
    render_rule,
};

#[cfg(test)]
mod flat_chain;
