//! Controller-purity check — compile-time hard wall.
//!
//! Defers the rule definition to `sui-lint`. The same walker that
//! drives editor diagnostics drives this build-time check, so a
//! violation looks identical to LSP and `cargo build`.
//!
//! Skipped when `source_path` is `None` (synthetic fragments in unit
//! tests), and when no `*.controller.ts` is paired with the source.

use std::path::Path;

use crate::diagnostics::{CompileError, Span};

pub fn check(
    source_path: Option<&Path>,
    overrides: &[crate::check::lint_overrides::LintOverride],
) -> Result<(), CompileError> {
    let Some(p) = source_path else { return Ok(()); };
    let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else { return Ok(()); };
    let dir = p.parent().unwrap_or(Path::new("."));
    let controller_path = if stem == "page" {
        dir.join("controller.ts")
    } else {
        dir.join(format!("{stem}.controller.ts"))
    };
    if !controller_path.exists() {
        return Ok(());
    }

    let source = match std::fs::read_to_string(&controller_path) {
        Ok(s) => s,
        Err(_) => return Ok(()), // unreadable — let other tooling surface it
    };

    // Apply the build host's lint overrides before the Error filter. The
    // rule's natural severity is preserved for everyone who consumes
    // the lint directly (the LSP, `sui_lint` CLI); the override only
    // affects this build-time gate.
    let findings = crate::check::lint_overrides::apply_overrides(
        sui_lint::lint_controller_source(&controller_path, &source),
        overrides,
    );
    // The build gate fails only on Error-severity findings. Warnings surface
    // through the LSP / `sui-lint` CLI (which call `lint_controller_source`
    // directly) but don't break compilation — so a rule can ship as a
    // surfaced-but-non-blocking warning before being promoted to a hard wall.
    let Some(first) = findings.iter().find(|f| f.severity == sui_lint::Severity::Error) else {
        return Ok(());
    };

    let span = Span::new(
        line_col_to_offset(&source, first.line, first.col),
        line_col_to_offset(&source, first.line, first.col + 1),
    );
    // `with_category` below carries the rule tag; embedding it here too
    // made `render()` print it twice.
    let mut msg = first.message.clone();
    if let Some(s) = &first.suggest {
        msg.push_str("\n  suggest: ");
        msg.push_str(s);
    }
    Err(CompileError::new(msg)
        .with_span(span)
        .with_path(controller_path)
        .with_category(first.rule.to_string()))
}

/// 1-based `(line, col)` → byte offset into `source`. Inverse of
/// `diagnostics::line_col`. Used to synthesise a `Span` from a
/// `lint::Finding` so the rest of the diagnostics machinery (which
/// expects `Span`) lights up identically to other compile errors.
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
