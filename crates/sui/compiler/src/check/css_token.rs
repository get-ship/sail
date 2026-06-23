//! CSS-token check — compile-time hard wall.
//!
//! Defers the rule definition to `sui-lint::rules::css_token`. The
//! same walker that drives editor diagnostics drives this build-time
//! check, so an unknown `var(--xyz)` looks identical in LSP and in
//! `cargo build` output.
//!
//! Mirrors `controller_purity.rs` (this directory) — same plumbing
//! (`sui_lint` finding → `CompileError`), only the source it lints is
//! the `.sui` itself rather than a paired `.controller.ts`.
//!
//! Skipped when `source` is empty, which only happens via internal
//! synthetic-fragment paths that bypass `compile()`.

use std::path::Path;

use crate::diagnostics::{CompileError, Span};

pub fn check(source: &str, source_path: Option<&Path>) -> Result<(), CompileError> {
    if source.is_empty() {
        return Ok(());
    }
    // For synthetic sources without a path we still lint, so typos in
    // unit-test fixtures are caught; we use a placeholder path that
    // won't surface elsewhere.
    let placeholder;
    let path = match source_path {
        Some(p) => p,
        None => {
            placeholder = Path::new("<synthetic>");
            placeholder
        }
    };

    // Only Severity::Error findings become compile errors. Warnings
    // (e.g. `sui-for-key`, `sui-event-name`) surface in LSP but don't
    // block the build — they catch silent-bug classes that the team
    // wants to fix but that existing views are allowed to have during
    // migration.
    let findings = sui_lint::lint_sui_template_source(path, source);
    let Some(first) = findings
        .iter()
        .find(|f| matches!(f.severity, sui_lint::Severity::Error))
    else {
        return Ok(());
    };

    let offset = line_col_to_offset(source, first.line, first.col);
    let span = Span::new(offset, offset + 2); // covers the `--` of the bad name

    let mut msg = format!("[{}] {}", first.rule, first.message);
    if let Some(s) = &first.suggest {
        msg.push_str("\n  suggest: ");
        msg.push_str(s);
    }

    let mut err = CompileError::new(msg)
        .with_span(span)
        .with_category(first.rule.to_string());
    if let Some(p) = source_path {
        err = err.with_path(p.to_path_buf());
    }
    Err(err)
}

/// 1-based `(line, col)` → byte offset into `source`. Mirrors the
/// helper in `controller_purity.rs` (this directory) — kept
/// duplicated for the same reason: each check owns its position
/// mapping and the shape may diverge.
fn line_col_to_offset(source: &str, target_line: usize, target_col: usize) -> usize {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let mut i = 0;
    while i < bytes.len() {
        if line == target_line && col == target_col {
            return i;
        }
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    bytes.len()
}
