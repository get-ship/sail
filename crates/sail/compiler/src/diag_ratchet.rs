//! Anonymous diagnostic ratchet (SPEC-LINTS-1 Phase 0).
//!
//! Prevents **growth** of anonymous compiler diagnostic throw sites.
//! Existing debt may remain until migration waves; new anonymous sites fail CI.
//!
//! # Intended full API (implementer)
//!
//! | Symbol | Contract |
//! |--------|----------|
//! | [`is_stable_rule_id`] | `true` iff `id` matches `^sail-[a-z0-9-]+$` |
//! | [`classify_snippet`] | Classify a throw-site source window as id'd or anonymous |
//! | [`count_anonymous_sites`] | Scan live `crates/sail/compiler/src/**/*.rs` throw sites |
//! | [`anonymous_baseline`] | Deliberate pin from a real tip remeasure (not a guess) |
//!
//! A construction site is **id'd** iff its emitted rule/category resolves to a
//! stable id matching `^sail-[a-z0-9-]+$` via `.with_category("sail-…")`,
//! `rule: Some("sail-…")`, or a same-file `const` that is that string.
//!
//! Everything else (missing category, free-form location strings, `expression`,
//! `manifest`, `for-iterable`, bare messages) is **anonymous**.
//!
//! Throw construction patterns to count:
//! - `CompileError::new(`
//! - `Diagnostic {` message-bearing product emissions (not the type definition)
//!
//! # Baseline policy
//!
//! Assert `count_anonymous_sites() <= anonymous_baseline()`. Drops pass without
//! pin update (debt reduction is free); only growth fails. Implementer must
//! remeasure tip and set the pin — do not hardcode inventory 181 without scan.
//!
//! # Status
//!
//! Stub only — test-writer red gate. Implementer replaces bodies; must not
//! weaken the anonymous/id'd definitions above.

/// Classification of one diagnostic throw construction site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteClass {
    /// Rule/category resolves to `^sail-[a-z0-9-]+$`.
    Idd,
    /// Missing, free-form, or non-`sail-*` category/rule.
    Anonymous,
}

/// Return `true` when `id` is a stable sail rule id (`^sail-[a-z0-9-]+$`).
pub fn is_stable_rule_id(_id: &str) -> bool {
    todo!("diag_ratchet: implement is_stable_rule_id (^sail-[a-z0-9-]+$)")
}

/// Classify a throw-site source window (fixture snippet or live file slice).
///
/// Detects id attachment in the snippet:
/// - `.with_category("sail-…")` string literal
/// - `.with_category(CONST)` resolving to `sail-…` via same-file const
/// - `rule: Some("sail-…")` or `rule: Some(CONST)` resolving to `sail-…`
///
/// Missing category / free-form categories → [`SiteClass::Anonymous`].
pub fn classify_snippet(_source: &str) -> SiteClass {
    todo!("diag_ratchet: implement classify_snippet for throw-site windows")
}

/// Count anonymous throw sites under live `crates/sail/compiler/src/**/*.rs`.
pub fn count_anonymous_sites() -> usize {
    todo!("diag_ratchet: scan compiler src; count anonymous throw sites")
}

/// Pinned anonymous baseline. Implementer sets after a real tip remeasure.
///
/// Gate: `count_anonymous_sites() <= anonymous_baseline()`. Prefer `<=` so
/// debt reduction does not require a pin update in the same PR.
pub fn anonymous_baseline() -> usize {
    todo!("diag_ratchet: pin ANONYMOUS_BASELINE from measured tip scan")
}
