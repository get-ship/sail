//! HTML/CSS realization of a Style program.
//!
//! Depends on `spinnaker` (parse / IR / Intent). The domain crate must not
//! depend on this crate — not even as a `[dev-dependency]`.

pub mod concern_css;
pub mod conditions;
pub mod emit;
pub mod lower;
pub mod plains_css;
pub mod prefixes;
pub mod when;

pub use conditions::{
    CONDITION_PREFIXES, PREFERENCE_PREFIXES, condition_media, condition_prefixes,
};
pub use emit::{
    CssRule, emit_intent, emit_resolved_type, emit_style, lower_style, lower_style_source,
    lower_style_source_with_motion, render_rule,
};
pub use lower::lower_op_css;
pub use prefixes::{
    STATE_PREFIXES, SX_EXTRA_FORMS, group_attr_presence, is_prefix, pseudo_element_prefix,
    state_data, state_pseudo,
};
pub use when::{WhenTarget, when_selector, when_target};

#[cfg(test)]
mod adapter_lock;
#[cfg(test)]
mod flat_chain;
#[cfg(test)]
mod vocab_class;
