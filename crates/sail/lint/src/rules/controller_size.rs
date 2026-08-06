//! `sail-controller-size` — thin-controller doctrine: warn when the setup
//! body grows past ~50 non-empty, non-comment lines.
//!
//! CHANNELS-1 WI10 / SAIL-SYNTAX-FREEZE §1: controllers orchestrate — signals,
//! effects, data calls, action wiring — they do not host implementation.
//! Crossing the threshold is a **policy-layer Warning** (default on). It never
//! hard-fails the build; silence via `policy.lint.severity.sail-controller-size
//! = "off"` if a host truly needs a thick controller during migration.
//!
//! ## Measurement
//!
//! 1. Parse the controller source (oxc TS).
//! 2. Find `export default controller(<setup>)` (or bare `controller(<setup>)`).
//! 3. Take the setup function body:
//!    - block body → union of each top-level statement's span (braces and
//!      inter-statement blanks/comments do not count by themselves);
//!    - expression-arrow body → the expression span.
//! 4. Count lines in that text that are neither blank nor comment-only
//!    (`//…`, `/*…*/` full-line, or interior of a multi-line block comment).
//! 5. If count **≥ [`THRESHOLD`]** (50), emit one Warning at the `controller`
//!    call site.
//!
//! If parse fails or no `controller(…)` call is found, the rule is silent
//! (other gates own those cases).

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-controller-size";

/// Non-empty, non-comment lines of the setup body that trigger the warning.
pub const THRESHOLD: usize = 50;

const MESSAGE_HEAD: &str = "controller body is";
const MESSAGE_TAIL: &str = "lines (thin-controller threshold 50) — keep orchestration \
     here; move implementation into `src/actions/`, `src/compositions/`, or a helper \
     module";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = SizeVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct SizeVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl SizeVisitor<'_> {
    fn report(&mut self, span: Span, loc: usize) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Warn,
            message: format!("{MESSAGE_HEAD} {loc} {MESSAGE_TAIL}"),
            suggest: Some(
                "Extract pure logic into `src/compositions/` or an action under \
                 `src/actions/`; leave the controller as wiring (signals, effects, \
                 returns)."
                    .to_string(),
            ),
        });
    }
}

impl<'a> Visit<'a> for SizeVisitor<'_> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if is_controller_callee(&expr.callee)
            && let Some(setup) = expr.arguments.first().and_then(|a| a.as_expression())
        {
            let loc = count_setup_loc(self.source, setup);
            if loc >= THRESHOLD {
                self.report(expr.span, loc);
            }
        }
        // Walk children so nested `controller(...)` (unusual) is still checked.
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e);
            }
        }
    }
}

fn is_controller_callee(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => id.name.as_str() == "controller",
        Expression::TSInstantiationExpression(te) => is_controller_callee(&te.expression),
        Expression::ParenthesizedExpression(p) => is_controller_callee(&p.expression),
        _ => false,
    }
}

/// Count non-blank, non-comment-only lines of the setup function body.
fn count_setup_loc(source: &str, expr: &Expression<'_>) -> usize {
    match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                // `() => <expr>` — single expression statement's span.
                arrow
                    .body
                    .statements
                    .first()
                    .map(|s| count_span_loc(source, s.span()))
                    .unwrap_or(0)
            } else {
                count_block_statements_loc(source, &arrow.body.statements)
            }
        }
        Expression::FunctionExpression(fun) => fun
            .body
            .as_ref()
            .map(|b| count_block_statements_loc(source, &b.statements))
            .unwrap_or(0),
        Expression::ParenthesizedExpression(p) => count_setup_loc(source, &p.expression),
        _ => 0,
    }
}

/// Sum LOC over each top-level statement (braces / blank gaps ignored).
fn count_block_statements_loc(source: &str, stmts: &[Statement<'_>]) -> usize {
    stmts.iter().map(|s| count_span_loc(source, s.span())).sum()
}

/// Count non-blank, non-comment-only lines inside `span`.
pub fn count_span_loc(source: &str, span: Span) -> usize {
    let start = (span.start as usize).min(source.len());
    let end = (span.end as usize).min(source.len());
    if start >= end {
        return 0;
    }
    count_loc_in_slice(&source[start..end])
}

fn count_loc_in_slice(slice: &str) -> usize {
    let mut loc = 0usize;
    let mut in_block = false;
    for line in slice.lines() {
        let s = line.trim();
        if s.is_empty() {
            continue;
        }
        // Full-line // comments do not count.
        if !in_block && s.starts_with("//") {
            continue;
        }
        // Block-comment state machine across lines.
        let mut i = 0;
        let bytes = s.as_bytes();
        let mut has_code = false;
        while i < bytes.len() {
            if in_block {
                if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    in_block = false;
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                in_block = true;
                i += 2;
                continue;
            }
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
                // Rest of line is line comment.
                break;
            }
            // Non-comment character.
            has_code = true;
            break;
        }
        if has_code {
            loc += 1;
        }
    }
    loc
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn lint(src: &str) -> Vec<Finding> {
        check(Path::new("page.sail"), src)
    }

    fn setup_with_n_statements(n: usize) -> String {
        let mut body = String::from("export default controller(() => {\n");
        for i in 0..n {
            body.push_str(&format!("  const x{i} = {i}\n"));
        }
        body.push_str("  return { ok: true }\n})\n");
        body
    }

    #[test]
    fn under_threshold_is_clean() {
        // 49 consts + return = 50 would fire; 48 + return = 49 clean.
        let src = setup_with_n_statements(48);
        let f = lint(&src);
        assert!(f.is_empty(), "expected clean under threshold, got {f:?}");
    }

    #[test]
    fn at_threshold_warns() {
        // 49 consts + `return { ok: true }` = 50 lines.
        let src = setup_with_n_statements(49);
        let f = lint(&src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert!(
            f[0].message.contains("50") || f[0].message.contains("lines"),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn over_threshold_warns_once() {
        let src = setup_with_n_statements(60);
        let f = lint(&src);
        assert_eq!(f.len(), 1, "one finding, not per-line: {f:?}");
        assert_eq!(f[0].severity, Severity::Warn);
    }

    #[test]
    fn blank_and_comment_lines_do_not_count() {
        let mut body = String::from("export default controller(() => {\n");
        for i in 0..40 {
            body.push_str(&format!("  const x{i} = {i}\n"));
            body.push_str("\n"); // blank
            body.push_str("  // commentary\n");
            body.push_str("  /* block\n     comment */\n");
        }
        body.push_str("  return {}\n})\n");
        // 40 consts + return = 41 < 50
        let f = lint(&body);
        assert!(f.is_empty(), "comments/blanks must not inflate: {f:?}");
    }

    #[test]
    fn expression_arrow_body_counted() {
        // Multi-line expression body under threshold.
        let src = "export default controller(() => ({\n  a: 1,\n  b: 2,\n  c: 3,\n}))\n";
        let f = lint(src);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn threshold_constant_is_fifty() {
        assert_eq!(THRESHOLD, 50);
    }

    #[test]
    fn count_loc_skips_full_line_comments() {
        let slice = "const a = 1\n// skip\nconst b = 2\n/* skip */\nconst c = 3\n";
        assert_eq!(count_loc_in_slice(slice), 3);
    }
}
