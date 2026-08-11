//! DiagSpec registry skeleton (SPEC-LINTS-1 §4 Phase 1 / S3.3).
//!
//! Governs **new** diagnostics: id, severity, message template, allowed-list
//! source, fix text, docs anchor (+ jurisdiction for the compiler-error vs
//! lint law). Existing anonymous debt is **not** migrated here — that is
//! Phase 2 waves. The anonymous ratchet (S3.1) still blocks growth.
//!
//! # Intended full API (implementer)
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
//!
//! # Status
//!
//! Stub only — test-writer red gate. Implementer populates the table and
//! wires at least one emission path; must not weaken the field contracts.

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

/// Full DiagSpec table.
///
/// Implementer: populate with at least the sample / known id'd rules the
/// gate tests require (`sail-layout`, `sail-unintended-overflow`, …).
/// New diagnostics **must** add a row here.
pub fn registry() -> &'static [DiagSpec] {
    // Stub: empty table — implementer fills Phase 1 sample rows.
    &[]
}

/// Look up a [`DiagSpec`] by stable rule id.
pub fn lookup(id: &str) -> Option<&'static DiagSpec> {
    registry().iter().find(|spec| spec.id == id)
}
