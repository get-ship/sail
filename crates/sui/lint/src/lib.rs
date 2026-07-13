//! SUI authoring lint engine.
//!
//! Lints `*.controller.ts` (paired SUI controllers) and `*.sui` templates.
//! Rule IDs are prefixed `sui-<descriptor>`.

pub mod prop_registry;
pub mod report;
pub mod rules;
pub mod source;
pub mod tokens;

#[cfg(test)]
mod tests;

pub use report::{Category, Finding, Severity};

/// All rule IDs defined by this crate. Generated from the registry.
pub fn all_rule_ids() -> Vec<&'static str> {
    rules::CONTROLLER_RULES
        .iter()
        .chain(rules::SUI_TEMPLATE_RULES.iter())
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
    Vec::new()
}
