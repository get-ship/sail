//! `sail-no-node-import` — Node/Bun are test and tooling only.
//!
//! Product `src` (apps + packages) must not import `node:` / `bun:` / bare
//! `process` specs or free Node ambients (`process`, …). Do not install
//! `@types/node` to silence product code — move the module under `test/` or
//! tooling.
//!
//! Severity: Error. Path-gated to product surfaces (not test/).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-no-node-import";

const MESSAGE: &str =
    "node/bun is the build toolchain, never a runtime dep — this belongs in test/ or tooling";

const SUGGEST: &str = "Move Node/Bun-only code under test/ or a tooling entry. \
     Do not add `@types/node` to product Pure/Action surfaces.";

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
        // `visit_program` enters the module scope via enter_scope.
        scopes: Vec::new(),
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
        || p.contains("/tooling/")
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
    m.starts_with("node:") || m.starts_with("bun:") || m == "process"
}

struct NodeImportVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
    /// Nested scopes of local bindings; free `process` only fires when unbound.
    scopes: Vec<HashSet<String>>,
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

    fn bind(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn is_bound(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.contains(name))
    }
}

impl<'a> Visit<'a> for NodeImportVisitor<'_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, _scope_id: &std::cell::Cell<Option<ScopeId>>) {
        self.scopes.push(HashSet::new());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let module = decl.source.value.as_str();
        if is_node_or_bun_spec(module) {
            self.report(decl.span, module);
        }
        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            let module = source.value.as_str();
            if is_node_or_bun_spec(module) {
                self.report(decl.span, module);
            }
        }
        // Walk declaration body so free `process` in `export const x = process…` fires.
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let module = decl.source.value.as_str();
        if is_node_or_bun_spec(module) {
            self.report(decl.span, module);
        }
        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &expr.source {
            let module = lit.value.as_str();
            if is_node_or_bun_spec(module) {
                self.report(expr.span, module);
            }
        }
        walk::walk_import_expression(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        // require('node:…') / require('process')
        let is_require = match &expr.callee {
            Expression::Identifier(id) => id.name.as_str() == "require",
            _ => false,
        };
        if is_require {
            if let Some(arg) = expr.arguments.first() {
                if let Some(Expression::StringLiteral(lit)) = arg.as_expression() {
                    let module = lit.value.as_str();
                    if is_node_or_bun_spec(module) {
                        self.report(expr.span, module);
                    }
                }
            }
        }
        walk::walk_call_expression(self, expr);
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        self.bind(ident.name.as_str());
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        // Free `process` ambient only — local bindings (const/param/import) are fine.
        if ident.name.as_str() == "process" && !self.is_bound("process") {
            self.report(ident.span, "process");
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
            f[0].message.contains("build toolchain") || f[0].message.contains("test/"),
            "teaching message missing: {}",
            f[0].message
        );
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
    fn errors_on_bare_process_module_import() {
        let f = lint_at(
            "apps/demo/src/compositions/env.ts",
            "import process from 'process'\nexport const e = process.env\n",
        );
        assert!(
            f.iter().any(|x| x.message.contains("process")),
            "bare process module must fire: {f:?}"
        );
    }

    #[test]
    fn errors_on_free_process_identifier() {
        // Bare + member forms (type-gate also rewrites free process via TS2304).
        for src in [
            "export const e = process.env.NODE_ENV\n",
            "const x = process\nexport { x }\n",
            "void process\n",
        ] {
            let f = lint_at("apps/demo/src/compositions/env.ts", src);
            assert!(
                f.iter().any(|x| x.message.contains("process")),
                "free process must fire for {src:?}: {f:?}"
            );
        }
    }

    #[test]
    fn allows_local_process_binding() {
        // Shadowed / local `process` is not the Node ambient.
        for src in [
            "const process = { env: {} }\nexport const e = process.env\n",
            "function f(process: { env: Record<string, string> }) { return process.env }\nexport const e = f({ env: {} })\n",
            "export function g(process: { x: number }) { return process.x }\n",
        ] {
            let f = lint_at("apps/demo/src/compositions/env.ts", src);
            assert!(
                f.is_empty(),
                "local process binding must not fire free-ambient: {src:?} → {f:?}"
            );
        }
    }

    #[test]
    fn allows_in_test_path() {
        let f = lint_at(
            "apps/demo/test/helpers/cli.ts",
            "import { spawn } from 'node:child_process'\nexport const x = process.env.X\n",
        );
        assert!(f.is_empty(), "test/ is allowed: {f:?}");
    }

    #[test]
    fn allows_in_tooling_path() {
        let f = lint_at(
            "apps/demo/tooling/gen.ts",
            "import { readFileSync } from 'node:fs'\n",
        );
        assert!(f.is_empty(), "tooling/ is allowed: {f:?}");
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
