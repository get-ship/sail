//! `sail-prefer-query` — fetch-in-store (async load + signal `.set`) should
//! use `query()` / `mutation()` instead.
//!
//! Warning only — never an error this cycle. Pedagogical: teach the
//! platform data primitive before ambient network doors close harder.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-prefer-query";

const MESSAGE: &str =
    "async load + signal `.set` is a fetch-in-store pattern — prefer `query()` / `mutation()`";

/// Controllers + app sources under apps/*/src|routes.
fn path_in_scope(path: &Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    if p.contains("/test/")
        || p.contains("/tests/")
        || p.contains("/qa/")
        || p.contains("/scripts/")
        || p.contains(".gen.")
        || p.contains(".test.")
        || p.contains(".spec.")
        || p.contains("/__tests__/")
        || p.contains("/packages/")
        || p.starts_with("packages/")
        || p.contains("/node_modules/")
    {
        return false;
    }
    if p.ends_with(".controller.ts") || p.ends_with("/controller.ts") || p == "controller.ts" {
        return true;
    }
    let under_apps = p.contains("/apps/") || p.starts_with("apps/");
    under_apps && (p.contains("/src/") || p.contains("/routes/"))
}

fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn is_query_or_mutation_call(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => {
            matches!(id.name.as_str(), "query" | "queryKeyed" | "mutation")
        }
        Expression::StaticMemberExpression(m) => {
            matches!(
                m.property.name.as_str(),
                "query" | "queryKeyed" | "mutation"
            )
        }
        _ => false,
    }
}

fn is_set_call(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StaticMemberExpression(m) if m.property.name.as_str() == "set" => true,
        Expression::ComputedMemberExpression(m) => {
            matches!(&m.expression, Expression::StringLiteral(s) if s.value.as_str() == "set")
        }
        _ => false,
    }
}

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if !path_in_scope(path) {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut v = Visitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
        async_depth: 0,
        saw_await_call: false,
        saw_signal_set: false,
        saw_query_api: false,
        await_span_start: 0,
    };
    v.visit_program(&ret.program);
    v.findings
}

struct Visitor<'s> {
    source: &'s str,
    file: PathBuf,
    findings: Vec<Finding>,
    async_depth: u32,
    saw_await_call: bool,
    saw_signal_set: bool,
    saw_query_api: bool,
    await_span_start: u32,
}

impl Visitor<'_> {
    fn enter_async(&mut self) {
        self.async_depth += 1;
        if self.async_depth == 1 {
            self.saw_await_call = false;
            self.saw_signal_set = false;
            self.saw_query_api = false;
            self.await_span_start = 0;
        }
    }

    fn leave_async(&mut self) {
        if self.async_depth == 1
            && self.saw_await_call
            && self.saw_signal_set
            && !self.saw_query_api
        {
            let (line, col) = line_col(self.source, self.await_span_start as usize);
            self.findings.push(Finding {
                file: self.file.clone(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Warn,
                message: MESSAGE.to_string(),
                suggest: Some(
                    "const data = query(() => api.load()); // or mutation() for writes".into(),
                ),
            });
        }
        self.async_depth = self.async_depth.saturating_sub(1);
    }
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_function(&mut self, func: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if func.r#async {
            self.enter_async();
        }
        walk::walk_function(self, func, flags);
        if func.r#async {
            self.leave_async();
        }
    }

    fn visit_arrow_function_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        if expr.r#async {
            self.enter_async();
        }
        walk::walk_arrow_function_expression(self, expr);
        if expr.r#async {
            self.leave_async();
        }
    }

    fn visit_await_expression(&mut self, expr: &AwaitExpression<'a>) {
        if self.async_depth > 0 {
            if let Expression::CallExpression(call) = &expr.argument {
                self.saw_await_call = true;
                if self.await_span_start == 0 {
                    self.await_span_start = expr.span.start;
                }
                if is_query_or_mutation_call(&call.callee) {
                    self.saw_query_api = true;
                }
            }
        }
        walk::walk_await_expression(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if is_query_or_mutation_call(&expr.callee) {
            self.saw_query_api = true;
        }
        if self.async_depth > 0 && is_set_call(&expr.callee) {
            self.saw_signal_set = true;
        }
        walk::walk_call_expression(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_fetch_in_store() {
        let src = r#"
            const data = signal(null)
            async function load() {
              const r = await fetch('/api')
              data.set(r)
            }
        "#;
        let f = check(Path::new("apps/studio/src/store.ts"), src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(matches!(f[0].severity, Severity::Warn));
    }

    #[test]
    fn silent_on_query() {
        let src = r#"
            export default controller(() => {
              const users = query(() => api.users.list())
              return { users }
            })
        "#;
        let f = check(Path::new("apps/studio/src/x.controller.ts"), src);
        assert!(
            f.iter().all(|x| x.rule != RULE_ID),
            "query() should be silent: {f:?}"
        );
    }

    #[test]
    fn silent_when_query_present_in_async() {
        let src = r#"
            async function load() {
              const users = query(() => api.users.list())
              const r = await other()
              users.set(r)
            }
        "#;
        let f = check(Path::new("apps/studio/src/x.ts"), src);
        assert!(f.is_empty(), "{f:?}");
    }
}
