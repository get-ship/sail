//! SUI authoring lint engine.
//!
//! Lints `*.controller.ts` (paired SUI controllers), `*.sui` templates,
//! and the `.ts` composition modules under a component pack's `src/`
//! (the seam the floor closes: every `.ts` file a component ships, not
//! just the `<controller>` block). Rule IDs are prefixed `sail-<descriptor>`.

pub mod config;
pub mod prop_registry;
pub mod scales;
pub mod report;
pub mod rules;
pub mod source;
pub mod sx_vocab;
pub mod tokens;

#[cfg(test)]
mod tests;

pub use report::{Category, Finding, Severity};

/// All rule IDs defined by this crate. Generated from the registry.
pub fn all_rule_ids() -> Vec<&'static str> {
    rules::CONTROLLER_RULES
        .iter()
        .chain(rules::SUI_TEMPLATE_RULES.iter())
        .chain(rules::PACK_SOURCE_RULES.iter())
        .map(|(id, _)| *id)
        .collect()
}

/// True iff `path` is a paired SUI controller (`<stem>.controller.ts`).
///
/// The `.controller.ts` suffix is the SUI convention for paired
/// controllers. Plain `.ts` files (helpers, types, tests) are never
/// linted as controllers regardless of co-location.
pub fn is_controller_path(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".controller.ts"))
        .unwrap_or(false)
}

/// True iff `path` is a SUI template (`*.sui`).
pub fn is_sui_template_path(path: &std::path::Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("sui")
}

/// True iff `path` is a pack source `.ts` (a composition module a component ships).
///
/// The set is "any `.ts` file a component pack ships, not under the framework
/// runtime". The framework (the actions themselves) is EXEMPT — the
/// `.style` writes inside `<controller>` blocks are already gated by
/// `lint_controller_source`; this third category closes the hatch a
/// plain `.ts` helper in a pack would otherwise leave open.
///
/// Detection walks the path's segments: a path under `packages/components/sail/**/src/`
/// (or the repo-local equivalent) is a pack source. The `*.controller.ts`
/// files are matched by `is_controller_path` first, so this never overlaps.
pub fn is_pack_source_ts(path: &std::path::Path) -> bool {
    if path.extension().and_then(|e| e.to_str()) != Some("ts") {
        return false;
    }
    // Controllers go through  first in ; the
    // caller that bypasses the dispatch and asks this directly won't see a
    // controller file as a pack source.
    if is_controller_path(path) {
        return false;
    }
    // The framework layer — exempt: the actions live here, and .style writes
    // inside an action are the seam, not an authoring hatch.
    // We detect by a `packages/sail/runtime/src/` parent path; both the
    // public barrel (`runtime/src/index.ts`) and the actions themselves
    // match.
    let p = path.to_string_lossy();
    let components_hit = p.contains("/packages/components/sail/")
        && (p.contains("/src/") || p.ends_with("/src"));
    if !components_hit {
        return false;
    }
    // Skip test files (they re-implement focus traps for assertion convenience
    // and the lint surface is irrelevant). The pack's own test directory
    // (often `test/` next to `src/`) is also out of scope.
    p.contains("/src/")
        || p.ends_with("/src")
}
/// Lint a SUI controller from its source. Returns findings sorted by
/// `(line, col)`.
///
/// Caller-supplied `path` is used in findings' `file` field; reads
/// happen elsewhere. No filesystem access.
pub fn lint_controller_source(path: &std::path::Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (_id, check) in rules::CONTROLLER_RULES {
        findings.extend(check(path, source));
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a pack source `.ts` file from its source. Returns findings
/// sorted by `(line, col)`. No filesystem access.
///
/// The composition modules a component ships — helpers, glue code, the
/// packed controllers in their on-disk form (when not inline — the SFC
/// path uses `lint_controller_source` instead). The framework layer
/// (`packages/sail/runtime/src/**`) is exempt; this dispatches only on
/// paths under `packages/components/sail/**/src/**`.
pub fn lint_pack_source_source(path: &std::path::Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (_id, check) in rules::PACK_SOURCE_RULES {
        findings.extend(check(path, source));
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a SUI template (`*.sui`) from its source. Returns findings
/// sorted by `(line, col)`. No filesystem access.
pub fn lint_sui_template_source(path: &std::path::Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (_id, check) in rules::SUI_TEMPLATE_RULES {
        findings.extend(check(path, source));
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a single file path. Reads from disk, dispatches by extension.
/// Returns empty `Vec` for files outside SUI's authoring scope.
pub fn lint_path(path: &std::path::Path) -> Vec<Finding> {
    if is_controller_path(path) {
        let Ok(source) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return lint_controller_source(path, &source);
    }
    if is_sui_template_path(path) {
        let Ok(source) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return lint_sui_template_source(path, &source);
    }
    if is_pack_source_ts(path) {
        let Ok(source) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return lint_pack_source_source(path, &source);
    }
    Vec::new()
}
