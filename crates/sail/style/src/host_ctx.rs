//! L1 host unit context for Style surface vocabulary.
//!
//! When the compiler lowers an L1 host unit (`compile_element`), the host is
//! known by construction. Surface author spelling (`chip`, `chip.foreground`)
//! is legal only in that unit. App / L2 Style has no surface identities.

use std::cell::RefCell;

thread_local! {
    static HOST: RefCell<Option<String>> = const { RefCell::new(None) };
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
