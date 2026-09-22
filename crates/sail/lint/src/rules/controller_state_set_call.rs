//! `sail-state-set-call` — signals are written via `.set(…)`, never plain
//! assignment.
//!
//! Controllers are now a single function: `controller((ctx) => { ... })`.
//! There is no more `setup:`/`derived:`/`mount`/`on:` section split and no
//! more MutableState writable proxy — every binding created via `signal(…)`
//! (or `derived(…)`) is a raw `Signal`/`Derived` from
//! `packages/sail/runtime/src/signal.ts`, used uniformly everywhere in the
//! controller body, including inside `mount(...)`/`onCleanup(...)` closures
//! and any handler the setup function defines and returns.
//!
//! A `Signal<T>` is a callable getter carrying a `.set(value | updater)`
//! writer (see `Signal<T>` in `signal.ts`) — `.set(...)` is the ONLY correct
//! way to write one, always. Plain assignment to a signal-bound identifier
//! (`count = 5`, `count += 1`, `count++`, …) does not touch the signal at
//! all: for a `const` binding it's a compile error; for anything looser it
//! silently reassigns the local binding and breaks reactivity, since nothing
//! downstream observes a plain variable reassignment.
//!
//! This rule tracks local identifiers bound to `signal(...)`/`derived(...)`
//! calls (`const x = signal(0)`, `let y = derived(...)`, …) within a
//! controller source, and flags any later plain assignment or update
//! expression (`AssignmentExpression`/`UpdateExpression`) that targets one
//! of those identifiers directly — as opposed to a property/member access,
//! or a call to `.set(...)` on it, both of which are left alone. This is a
//! lightweight AST pattern-matcher, not a scope-resolution pass: it doesn't
//! model block/function shadowing, matching the level of rigor the rest of
//! this lint crate uses for controller heuristics.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use std::collections::HashSet;

pub const RULE_ID: &str = "sail-state-set-call";

/// Lint a controller `.ts` source. Returns findings ordered by source
/// position. Parse errors are silently swallowed — the SAIL compiler
/// (and tsc) own parse-error reporting; we only care about clean,
/// well-formed sources.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    // Pass 1: collect every identifier bound to `signal(...)` or
    // `derived(...)` anywhere in the source.
    let mut collector = SignalBindingCollector {
        signals: HashSet::new(),
    };
    collector.visit_program(&ret.program);

    // Pass 2: flag plain assignment/update expressions targeting one of
    // those identifiers.
    let mut visitor = SetCallVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
        signals: &collector.signals,
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

/// Collects identifiers bound via `const/let X = signal(...)` or
/// `const/let X = derived(...)`.
struct SignalBindingCollector<'src> {
    signals: HashSet<&'src str>,
}

impl<'a, 'src> Visit<'a> for SignalBindingCollector<'src>
where
    'a: 'src,
{
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(id), Some(init)) = (&decl.id, &decl.init) {
            if is_signal_or_derived_call(init) {
                self.signals.insert(id.name.as_str());
            }
        }
        if let Some(init) = &decl.init {
            self.visit_expression(init);
        }
    }
}

/// Match `signal(...)` or `derived(...)` — a bare call to one of those two
/// identifiers, any argument shape.
fn is_signal_or_derived_call(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return false;
    };
    matches!(callee.name.as_str(), "signal" | "derived")
}

struct SetCallVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
    signals: &'src HashSet<&'src str>,
}

impl<'src> SetCallVisitor<'src> {
    fn report(&mut self, span: Span, key: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "`{key}` is a signal — plain assignment doesn't write through it, \
                 it just reassigns the local binding and breaks reactivity."
            ),
            suggest: Some(format!(
                "Use `{key}.set(...)` to write the signal, not `{key} = ...`."
            )),
        });
    }
}

impl<'a, 'src> Visit<'a> for SetCallVisitor<'src> {
    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = &expr.left {
            if self.signals.contains(id.name.as_str()) {
                self.report(expr.span, id.name.as_str());
            }
        }
        self.visit_assignment_target(&expr.left);
        self.visit_expression(&expr.right);
    }

    fn visit_update_expression(&mut self, expr: &UpdateExpression<'a>) {
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &expr.argument {
            if self.signals.contains(id.name.as_str()) {
                self.report(expr.span, id.name.as_str());
            }
        }
        self.visit_simple_assignment_target(&expr.argument);
    }
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

    #[test]
    fn flags_plain_assignment_to_signal() {
        let src = r#"
            export default controller((ctx) => {
              const count = signal(0)
              const incr = () => { count = count + 1 }
              return { count, incr }
            })
        "#;
        let findings = check(Path::new("foo.controller.ts"), src);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, RULE_ID);
        assert!(findings[0].message.contains("count"));
        assert!(
            findings[0]
                .suggest
                .as_ref()
                .unwrap()
                .contains("count.set(...)")
        );
    }

    #[test]
    fn flags_update_expression_on_signal() {
        let src = r#"
            export default controller((ctx) => {
              const count = signal(0)
              const incr = () => { count++ }
              return { count, incr }
            })
        "#;
        let findings = check(Path::new("foo.controller.ts"), src);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("count"));
    }

    #[test]
    fn does_not_flag_set_call() {
        // `.set(...)` is now the ONLY correct write form — never flagged.
        let src = r#"
            export default controller((ctx) => {
              const count = signal(0)
              const incr = () => { count.set(count() + 1) }
              return { count, incr }
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn does_not_flag_reassignment_of_non_signal_local() {
        let src = r#"
            export default controller((ctx) => {
              let attempts = 0
              const retry = () => { attempts += 1 }
              return { retry }
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn does_not_flag_updater_function_set() {
        let src = r#"
            export default controller((ctx) => {
              const count = signal(0)
              const incr = () => { count.set((prev) => prev + 1) }
              return { count, incr }
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn flags_plain_assignment_inside_mount_closure() {
        // Previously mount()/on: bodies were a distinct "proxy context";
        // now there is exactly one context, and the same signal, created in
        // the setup body, is written the same wrong way inside a mount()
        // closure the setup function defines.
        let src = r#"
            export default controller((ctx) => {
              const count = signal(0)
              mount((el) => {
                count = 5
              })
              return { count }
            })
        "#;
        let findings = check(Path::new("foo.controller.ts"), src);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("count"));
    }

    #[test]
    fn does_not_flag_set_call_inside_mount_closure() {
        let src = r#"
            export default controller((ctx) => {
              const count = signal(0)
              mount((el) => {
                count.set(5)
              })
              return { count }
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn flags_plain_assignment_to_derived_bound_local() {
        let src = r#"
            export default controller((ctx) => {
              let doubled = derived(() => 0)
              const reset = () => { doubled = derived(() => 1) }
              return { doubled, reset }
            })
        "#;
        let findings = check(Path::new("foo.controller.ts"), src);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("doubled"));
    }
}
