//! `sail-no-node-import` — Node/Bun are test and tooling only.
//!
//! Product `src` (apps + packages) must not import `node:` / `bun:` specs or
//! free Node ambients (`process`, `Buffer`, …). Do not install `@types/node`
//! to silence product code — move the module under `test/` or `scripts/`.
//!
//! Severity: Error. Path-gated to product surfaces (not test/).

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-no-node-import";

const MESSAGE: &str = "Node/Bun is not allowed in product source — lives in test/ and tooling only";

const SUGGEST: &str = "Move Node-only code under test/ or scripts/. Do not add `@types/node` \
     to product Pure/Action surfaces.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if !is_product_scope(path) {
        return Vec::new();
    }

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = NodeImportVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

fn is_product_scope(path: &Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    if p.contains("/test/")
        || p.contains("/tests/")
        || p.contains("/__tests__/")
        || p.contains("/scripts/")
        || p.ends_with(".test.ts")
        || p.ends_with(".spec.ts")
    {
        return false;
    }
    // Apps + shipped packages (src only); crates/tooling out of scope.
    if p.contains("/crates/") {
        return false;
    }
    p.contains("/src/") || p.contains("/routes/")
}

fn is_node_or_bun_spec(module: &str) -> bool {
    let m = module.trim();
    m.starts_with("node:") || m.starts_with("bun:")
}

struct NodeImportVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl NodeImportVisitor<'_> {
    fn report(&mut self, span: Span, detail: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!("{MESSAGE} (got `{detail}`)"),
            suggest: Some(SUGGEST.to_string()),
        });
    }
}

impl<'a> Visit<'a> for NodeImportVisitor<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let module = decl.source.value.as_str();
        if is_node_or_bun_spec(module) {
            self.report(decl.span, module);
        }
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            let module = source.value.as_str();
            if is_node_or_bun_spec(module) {
                self.report(decl.span, module);
            }
        }
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let module = decl.source.value.as_str();
        if is_node_or_bun_spec(module) {
            self.report(decl.span, module);
        }
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &expr.source {
            let module = lit.value.as_str();
            if is_node_or_bun_spec(module) {
                self.report(expr.span, module);
            }
        }
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        // require('node:…')
        let is_require = match &expr.callee {
            Expression::Identifier(id) => id.name.as_str() == "require",
            _ => false,
        };
        if !is_require {
            return;
        }
        if let Some(arg) = expr.arguments.first() {
            if let Some(Expression::StringLiteral(lit)) = arg.as_expression() {
                let module = lit.value.as_str();
                if is_node_or_bun_spec(module) {
                    self.report(expr.span, module);
                }
            }
        }
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

    fn lint_at(path: &str, src: &str) -> Vec<Finding> {
        check(Path::new(path), src)
    }

    #[test]
    fn errors_on_node_fs_import() {
        let f = lint_at(
            "apps/demo/src/compositions/boot.ts",
            "import { readFileSync } from 'node:fs'\nexport const x = 1\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].message.contains("node:fs"), "{f:?}");
        assert!(
            f[0].suggest
                .as_deref()
                .is_some_and(|s| s.contains("test/") && !s.contains("npm i")),
            "{:?}",
            f[0].suggest
        );
    }

    #[test]
    fn errors_on_bun_test_import_in_src() {
        let f = lint_at(
            "packages/sail/runtime/src/oops.ts",
            "import { test } from 'bun:test'\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn allows_in_test_path() {
        let f = lint_at(
            "apps/demo/test/helpers/cli.ts",
            "import { spawn } from 'node:child_process'\n",
        );
        assert!(f.is_empty(), "test/ is allowed: {f:?}");
    }

    #[test]
    fn allows_ship_package() {
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import { signal } from '@ship/sail-runtime'\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }
}
