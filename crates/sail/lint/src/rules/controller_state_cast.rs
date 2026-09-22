//! `sail-controller-state-cast` — controllers read state through the typed
//! `MutableState` surface, not through a `Scope` cast.
//!
//! Bans, anywhere inside a `*.controller.ts` file, casting to the generated
//! `Scope` (or a `Scope & {…}` intersection / a `ScopeView` alias):
//! - `state as unknown as Scope`
//! - `state as Scope`
//! - `state as unknown as Scope & { … }`
//! - `state as unknown as ScopeView`
//!
//! These casts were load-bearing only while the read-side was untyped. The
//! controller's `state` is `MutableState<Scope & SC>`, which exposes:
//! - signal keys (props, factory signals, element capture signals) via
//!   `state.$signals.x` (call to read, `.set` to write) or the assignment
//!   proxy (`state.x` / `state.x = v`),
//! - factory functions / non-signal members via `state.x`.
//!
//! Casting to `Scope` re-introduces the lying type the rewrite (#121)
//! removed. Use the typed surface instead.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

pub const RULE_ID: &str = "sail-controller-state-cast";

/// Lint a controller `.ts` source for `Scope` casts. Parse errors are
/// swallowed — the compiler + tsc own parse-error reporting.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = StateCastVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct StateCastVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl<'a, 'src> Visit<'a> for StateCastVisitor<'src> {
    fn visit_ts_as_expression(&mut self, expr: &TSAsExpression<'a>) {
        if let Some(name) = scope_type_name(&expr.type_annotation) {
            let (line, col) = byte_offset_to_line_col(self.source, expr.span.start as usize);
            self.findings.push(Finding {
                file: self.file.clone(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message: format!(
                    "controllers do not cast to `{name}`. `state` is already \
                     MutableState<Scope & SC> — read signals via `state.$signals.x` \
                     (or `state.x`) and factory members via `state.x`."
                ),
                suggest: Some(
                    "Drop the cast: signal keys → state.$signals.x (call to read, \
                     .set to write); factory functions / non-signal members → state.x."
                        .to_string(),
                ),
            });
        }
        // Keep walking so nested casts (the `as unknown` inner, or casts in
        // sub-expressions) are still visited.
        self.visit_expression(&expr.expression);
    }
}

/// If `ty` is a reference to `Scope` / `ScopeView` — directly or as a member
/// of a `Scope & {…}` intersection — return the referenced name. Returns
/// `None` for any other type.
fn scope_type_name(ty: &TSType) -> Option<String> {
    match ty {
        TSType::TSTypeReference(r) => match &r.type_name {
            TSTypeName::IdentifierReference(id) if id.name == "Scope" || id.name == "ScopeView" => {
                Some(id.name.to_string())
            }
            _ => None,
        },
        TSType::TSIntersectionType(it) => it.types.iter().find_map(scope_type_name),
        _ => None,
    }
}

/// Convert a byte offset into a 1-based `(line, col)` pair.
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
