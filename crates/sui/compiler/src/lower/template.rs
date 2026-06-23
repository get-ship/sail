//! Template-level helpers consumed by the emitter.
//!
//! Walks the IR collecting which special HTML tags are used so the emitter
//! can decide what runtime helpers to import.

use crate::ir::{Instr, ElementInstr};

/// Recursively true if any element under `instrs` is a custom
/// element (tag contains a hyphen). Used to flag whether the emitted
/// JS needs to wait for `customElements.whenDefined` at mount.
pub fn has_custom_element(instrs: &[Instr]) -> bool {
    instrs.iter().any(|i| match i {
        Instr::Element(e) => element_or_descendant_is_custom(e),
        Instr::If(b) => {
            b.branches.iter().any(|br| has_custom_element(&br.body))
                || b.else_body.as_ref().is_some_and(|e| has_custom_element(e))
        }
        Instr::For(f) => {
            has_custom_element(&f.body)
                || f.else_body.as_ref().is_some_and(|e| has_custom_element(e))
        }
        Instr::Match(m) => {
            m.cases.iter().any(|c| has_custom_element(&c.body))
                || m.default.as_ref().is_some_and(|d| has_custom_element(d))
        }
        Instr::WhileLoading(wl) => has_custom_element(&wl.body),
        Instr::WhileError(we) => has_custom_element(&we.body),
        _ => false,
    })
}

fn element_or_descendant_is_custom(el: &ElementInstr) -> bool {
    if el.tag.contains('-') {
        return true;
    }
    has_custom_element(&el.children)
}
