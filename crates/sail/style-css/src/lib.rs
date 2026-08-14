//! HTML/CSS realization of a Style program.
//!
//! Depends on `sail-style` (parse / IR / Intent). The domain crate must not
//! depend on this crate — not even as a `[dev-dependency]`.

pub mod conditions;
pub mod emit;
pub mod prefixes;
pub mod when;

pub use conditions::{CONDITION_PREFIXES, condition_media};
pub use emit::{
    CssRule, emit_intent, emit_style, lower_style, lower_style_source,
    lower_style_source_with_motion, render_rule,
};
pub use prefixes::{
    STATE_PREFIXES, SX_EXTRA_FORMS, group_attr_presence, is_prefix, pseudo_element_prefix,
    state_data, state_pseudo,
};
pub use when::{WhenTarget, when_selector, when_target};

#[cfg(test)]
mod adapter_lock;
#[cfg(test)]
mod flat_chain;
