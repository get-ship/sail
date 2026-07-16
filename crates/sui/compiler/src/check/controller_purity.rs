//! The controller lint gate — a compile-time hard wall.
//!
//! Defers the rules to `sui-lint` (the same walker that drives the LSP), so a
//! violation looks identical in the editor and in `cargo build`.
//!
//! The source is the `<controller>` BLOCK. It used to hunt for a sibling
//! `<stem>.controller.ts` on disk — a file that is a compile error now
//! (`sui-paired-controller`) and therefore never exists. So this gate read
//! nothing and returned Ok for every SFC in the repo, and the whole controller
//! rule family — purity, ambient authority, set-value, state-cast, and the
//! imperative-styling floor — was silently dead.


use crate::check::controller_parse::ParsedController;
use crate::diagnostics::{CompileError, Span};

pub fn check(
    controller: Option<&ParsedController>,
    overrides: &[crate::check::lint_overrides::LintOverride],
) -> Result<(), CompileError> {
    let Some(c) = controller else { return Ok(()); };
    let controller_path = c.path.clone();
    let source = c.source.clone();

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
