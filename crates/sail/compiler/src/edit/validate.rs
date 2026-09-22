//! Pre-write validation: compile + lint the would-be source.

use crate::diagnostics::{DiagSeverity, Diagnostic};
use crate::{LintOverride, check_all};

#[derive(Debug, Clone)]
pub struct ValidateOutcome {
    pub diagnostics: Vec<Diagnostic>,
    pub has_errors: bool,
}

/// Validate `source` as if written at `path` (controller pairing / policy).
pub fn validate_source(
    source: &str,
    path: Option<&std::path::Path>,
    overrides: &[LintOverride],
) -> ValidateOutcome {
    let diagnostics = check_all(source, path, overrides, None);
    let has_errors = diagnostics
        .iter()
        .any(|d| matches!(d.severity, DiagSeverity::Error));
    ValidateOutcome {
        diagnostics,
        has_errors,
    }
}
