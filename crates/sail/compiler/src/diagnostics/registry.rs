//! DiagSpec registry skeleton (SPEC-LINTS-1 §4 Phase 1 / S3.3).
//!
//! Governs **new** diagnostics: id, severity, message template, allowed-list
//! source, fix text, docs anchor (+ jurisdiction for the compiler-error vs
//! lint law). Existing anonymous debt is **not** migrated here — that is
//! Phase 2 waves. The anonymous ratchet (S3.1) still blocks growth.
//!
//! # Intended full API
//!
//! | Symbol | Contract |
//! |--------|----------|
//! | [`DiagSpec`] | Static row: id + severity + message template + optional fix/docs/allowed-list |
//! | [`Jurisdiction`] | [`Jurisdiction::CompilerError`] vs [`Jurisdiction::Lint`] (SPEC-LINTS-1 §1) |
//! | [`registry`] | Full table slice (`&'static [DiagSpec]`) |
//! | [`lookup`] | `lookup(id) -> Option<&'static DiagSpec>` by stable `sail-*` id |
//!
//! # Phase 1 acceptance (gates)
//!
//! - Table may start small; **must** include known id'd product rules used as
//!   samples (`sail-layout`, `sail-unintended-overflow` at minimum).
//! - **New** diagnostics live in the table and emit with that `id`.
//! - At least one live emission path stamps a rule id present in the table
//!   (behavioral: emitted `Diagnostic.rule` is `lookup`-able).
//! - No mass migration of anonymous sites; no text goldens of full catalogs.

use super::DiagSeverity;

/// Whether a diagnostic is a non-negotiable compiler error or a
/// severity-configurable lint (SPEC-LINTS-1 §1 jurisdiction law).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jurisdiction {
    /// Compiler cannot produce correct output — never downgradable.
    CompilerError,
    /// Judgment on valid output — severity-configurable.
    Lint,
}

/// One governed diagnostic specification (SPEC-LINTS-1 §4 Phase 1).
///
/// Fields match the Phase 1 skeleton: id, severity, message template,
/// allowed-list source, fix text, docs anchor — plus [`Jurisdiction`] from
/// the jurisdiction law (SCOUT-S3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagSpec {
    /// Stable rule id (`sail-…`). Forever once shipped.
    pub id: &'static str,
    /// Compiler error vs lint (jurisdiction).
    pub jurisdiction: Jurisdiction,
    /// Default severity at emit (lint may still be overridden by config).
    pub default_severity: DiagSeverity,
    /// Message template (placeholders allowed; not a text-golden pin).
    pub message_template: &'static str,
    /// Where the allowed vocabulary / list for this rule comes from, if any
    /// (e.g. a live vocabulary module path). `None` when not list-gated.
    pub allowed_list_source: Option<&'static str>,
    /// Short fix guidance for authors/agents. `None` when no single fix.
    pub fix_text: Option<&'static str>,
    /// Docs / freeze-text anchor (path or fragment). `None` until linked.
    pub docs_anchor: Option<&'static str>,
}

/// Phase 1 sample / known product rows.
///
/// Sorted by `id` for stable lookup and human scan. Phase 2 waves expand
/// this table; mass anonymous migration is out of scope here.
///
/// Live emission for layout kinds is already wired in
/// [`crate::check::layout`] (`rule_id_for_kind` → `Diagnostic.rule`).
const DIAG_SPECS: &[DiagSpec] = &[
    DiagSpec {
        id: "sail-clipped-text",
        jurisdiction: Jurisdiction::Lint,
        default_severity: DiagSeverity::Warning,
        message_template: "text run is clipped without a truncate opt-in",
        allowed_list_source: None,
        fix_text: Some(
            "add `text: truncate()` if clipping is intentional, or widen the box / wrap the text",
        ),
        docs_anchor: Some("SPEC-LAYOUT-DIAG-1#clipped-text"),
    },
    DiagSpec {
        id: "sail-layout",
        jurisdiction: Jurisdiction::Lint,
        default_severity: DiagSeverity::Warning,
        message_template: "layout finding: {0}",
        allowed_list_source: Some("sail_lint::rules::layout::tag_props"),
        fix_text: Some(
            "fix axis values to the closed enum for the layout keyword, or correct the box geometry that taffy flagged",
        ),
        docs_anchor: Some("docs#sail-layout"),
    },
    DiagSpec {
        id: "sail-layout-primitive",
        jurisdiction: Jurisdiction::Lint,
        default_severity: DiagSeverity::Warning,
        message_template: "hand-rolled flex layout matches a language layout keyword",
        allowed_list_source: None,
        fix_text: Some("prefer `<row>` / `<column>` / `<grid>` / `<center>` / `<stack>` over raw flex in `<style>`"),
        docs_anchor: Some("docs#sail-layout-primitive"),
    },
    DiagSpec {
        id: "sail-unintended-overflow",
        jurisdiction: Jurisdiction::Lint,
        default_severity: DiagSeverity::Warning,
        message_template: "child overflows parent without an overflow opt-in",
        allowed_list_source: None,
        fix_text: Some(
            "add `overflowY(auto)` / `overflow(scroll)`, a `<scroll-area>`, or `truncate()` if the overflow is intentional",
        ),
        docs_anchor: Some("SPEC-LAYOUT-DIAG-1#unintended-overflow"),
    },
];

/// Full DiagSpec table.
///
/// New diagnostics **must** add a row here and emit with that stable `id`.
pub fn registry() -> &'static [DiagSpec] {
    DIAG_SPECS
}

/// Look up a [`DiagSpec`] by stable rule id.
pub fn lookup(id: &str) -> Option<&'static DiagSpec> {
    registry().iter().find(|spec| spec.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_rows_are_non_empty_and_lookupable() {
        let table = registry();
        assert!(!table.is_empty());
        assert!(lookup("sail-layout").is_some());
        assert!(lookup("sail-unintended-overflow").is_some());
        assert!(lookup("not-registered-zzzz").is_none());
    }

    #[test]
    fn registry_ids_are_unique() {
        let mut ids: Vec<&str> = registry().iter().map(|s| s.id).collect();
        let before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), before, "DiagSpec ids must be unique");
    }
}
