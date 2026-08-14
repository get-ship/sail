//! L1 host unit context for Style surface vocabulary + bound group props.
//!
//! When the compiler lowers an L1 host unit (`compile_element`), it installs
//! the pack name it owns (L1 manifest). Surface author spelling (`chip`,
//! `chip.foreground`) is legal only because the compiler said so. App / L2
//! Style has no surface identities.
//!
//! Bound color-group props are type-keyed: the compiler installs the prop
//! names whose type is the color-group union (`Tone`) so `bg(shade)` parses
//! as a bridge path. Prop name is irrelevant.

use std::cell::RefCell;

thread_local! {
    static HOST: RefCell<Option<String>> = const { RefCell::new(None) };
    static BOUND_GROUP_PROPS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Run `f` with optional L1 host surface pack in scope for Style parse/validate.
pub fn with_host_surface<T>(host: Option<&str>, f: impl FnOnce() -> T) -> T {
    HOST.with(|cell| {
        let prev = cell.replace(host.map(|s| s.to_string()));
        let out = f();
        *cell.borrow_mut() = prev;
        out
    })
}

/// Current host surface pack name, if compiling an L1 host unit.
pub fn current_host_surface() -> Option<String> {
    HOST.with(|cell| cell.borrow().clone())
}

/// Run `f` with color-group typed prop names recognized as bound paint roots.
///
/// Nested calls replace (not merge) the set. The product ambient name `tone`
/// (common color-group prop) is always recognized in
/// [`crate::token_groups::looks_like_bound_group_root`] even when this set is
/// empty (parse/complete without a component).
pub fn with_bound_group_props<T>(props: &[String], f: impl FnOnce() -> T) -> T {
    BOUND_GROUP_PROPS.with(|cell| {
        let prev = cell.replace(props.to_vec());
        let out = f();
        *cell.borrow_mut() = prev;
        out
    })
}

/// Prop names currently recognized as color-group typed bound paint roots.
pub fn bound_group_prop_names() -> Vec<String> {
    BOUND_GROUP_PROPS.with(|cell| cell.borrow().clone())
}

/// True if `name` is a bound color-group prop in the current context
/// (or product ambient `tone`).
pub fn is_bound_group_prop(name: &str) -> bool {
    if name == "tone" {
        return true;
    }
    BOUND_GROUP_PROPS.with(|cell| cell.borrow().iter().any(|p| p == name))
}
