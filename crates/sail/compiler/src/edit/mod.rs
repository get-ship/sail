//! Structured Sail edit ops — IR-addressed, validate-before-write.
//!
//! Ops address nodes by the same identity scheme as SailInspect (`file#path#hash`).
//! Application is span-stable source surgery guided by the author IR tree; the
//! would-be result is always re-checked before any disk write (callers decide
//! whether to write).

mod apply;
mod fix;
mod node;
mod ops;
mod validate;

pub use apply::{
    AutofixMode, DiagJson, EditResult, apply_ops, apply_ops_to_source, attach_rewrites,
    validate_sx_setter,
};
pub use fix::{
    FixPlan, TextPatch, apply_text_patches, is_auto_apply_plan, suggest_fixes,
    suggest_fixes_for_source,
};
pub use node::{EditNode, build_node_index, model_revision};
pub use ops::{EditOp, Position, parse_ops};
pub use validate::{ValidateOutcome, validate_source};
