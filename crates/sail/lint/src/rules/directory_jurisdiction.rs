//! `sail-directory-jurisdiction` — Action-typed functions live in `src/actions/**`.
//!
//! `compositions/**` is Pure (DOM-floor types unnameable — the project type
//! gate enforces). This lint names the LAW when an `Action<…>` annotation
//! appears outside `src/actions/**`.
//!
//! Evasion classes (all ERROR):
//! - sidecar import — `import type { Action }` + Action-typed fn in compositions
//! - mislabeled Action — alias of `Action<…>` used to type a function
//! - wrong folder — Action-typed fn under stores / other non-actions homes
//!
//! Type-only re-exports, unused `import type`, comments, and strings do not fire.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-directory-jurisdiction";

/// LAW named by this lint. Project Pure gate is the teeth; this is the lesson.
pub const LAW: &str = "compositions wire state; the DOM floor lives in src/actions/";

const REMEDY: &str = "Move the Action to `src/actions/` (handed `node`, observation pack). \
     compositions wire state; the DOM floor lives in src/actions/.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if is_action_home(path) {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = Visitor {
        source,
        file: path.to_path_buf(),
        action_aliases: HashSet::new(),
        type_params: HashSet::new(),
        findings: Vec::new(),
    };
    visitor.collect_aliases(&ret.program);
    visitor.visit_program(&ret.program);
    visitor.findings.sort_by_key(|f| (f.line, f.col));
    visitor.findings
}

fn is_action_home(path: &Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    p.contains("/src/actions/") || p.starts_with("src/actions/")
}

struct Visitor<'src> {
    source: &'src str,
    file: PathBuf,
    action_aliases: HashSet<String>,
    type_params: HashSet<String>,
    findings: Vec<Finding>,
}

impl Visitor<'_> {
    fn collect_aliases(&mut self, program: &Program<'_>) {
        for stmt in &program.body {
            let Statement::VariableDeclaration(_) = stmt else {
                // Type aliases live as TSTypeAliasDeclaration.
                if let Statement::TSTypeAliasDeclaration(alias) = stmt {
                    if type_is_action(&alias.type_annotation, &HashSet::new(), &HashSet::new()) {
                        self.action_aliases
                            .insert(alias.id.name.as_str().to_string());
                    }
                }
                continue;
            };
        }
        // Also walk export-wrapped aliases.
        for stmt in &program.body {
            let Statement::ExportNamedDeclaration(exp) = stmt else {
                continue;
            };
            if let Some(Declaration::TSTypeAliasDeclaration(alias)) = &exp.declaration {
                if type_is_action(&alias.type_annotation, &HashSet::new(), &HashSet::new()) {
                    self.action_aliases
                        .insert(alias.id.name.as_str().to_string());
                }
            }
        }
    }

    fn report(&mut self, span: Span, name: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!("`{name}` is typed as Action outside `src/actions/**` — {LAW}"),
            suggest: Some(REMEDY.to_string()),
        });
    }

    fn check_type(&mut self, span: Span, name: &str, ty: &TSType<'_>) {
        if type_is_action(ty, &self.action_aliases, &self.type_params) {
            self.report(span, name);
        }
    }

    fn with_type_params<R>(
        &mut self,
        params: &HashSet<String>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let prev = std::mem::replace(&mut self.type_params, params.clone());
        let out = f(self);
        self.type_params = prev;
        out
    }

    fn check_ann(&mut self, span: Span, name: &str, ann: Option<&TSTypeAnnotation<'_>>) {
        if let Some(ann) = ann {
            self.check_type(span, name, &ann.type_annotation);
        }
    }
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(id) = &decl.id {
            self.check_ann(id.span, id.name.as_str(), decl.type_annotation.as_deref());
        }
        if let Some(init) = &decl.init {
            self.visit_expression(init);
        }
    }

    fn visit_function(&mut self, func: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {
        let params = type_param_names(func.type_parameters.as_deref());
        self.with_type_params(&params, |this| {
            let name = func
                .id
                .as_ref()
                .map(|i| i.name.as_str())
                .unwrap_or("anonymous");
            if let Some(ret) = &func.return_type {
                this.check_type(func.span, name, &ret.type_annotation);
            }
            for p in &func.params.items {
                if let BindingPattern::BindingIdentifier(id) = &p.pattern {
                    this.check_ann(id.span, id.name.as_str(), p.type_annotation.as_deref());
                }
            }
            if let Some(body) = &func.body {
                this.visit_function_body(body);
            }
        });
    }

    fn visit_arrow_function_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        if let Some(ret) = &expr.return_type {
            self.check_type(expr.span, "arrow", &ret.type_annotation);
        }
        for p in &expr.params.items {
            if let BindingPattern::BindingIdentifier(id) = &p.pattern {
                self.check_ann(id.span, id.name.as_str(), p.type_annotation.as_deref());
            }
        }
        self.visit_function_body(&expr.body);
    }

    fn visit_ts_as_expression(&mut self, expr: &TSAsExpression<'a>) {
        if type_is_action(
            &expr.type_annotation,
            &self.action_aliases,
            &self.type_params,
        ) {
            self.report(expr.span, "value");
        }
        self.visit_expression(&expr.expression);
    }

    fn visit_ts_satisfies_expression(&mut self, expr: &TSSatisfiesExpression<'a>) {
        if type_is_action(
            &expr.type_annotation,
            &self.action_aliases,
            &self.type_params,
        ) {
            self.report(expr.span, "value");
        }
        self.visit_expression(&expr.expression);
    }
}

fn type_param_names(params: Option<&TSTypeParameterDeclaration<'_>>) -> HashSet<String> {
    let mut out = HashSet::new();
    let Some(params) = params else {
        return out;
    };
    for p in &params.params {
        out.insert(p.name.name.as_str().to_string());
    }
    out
}

fn type_is_action(
    ty: &TSType<'_>,
    aliases: &HashSet<String>,
    type_params: &HashSet<String>,
) -> bool {
    match ty {
        TSType::TSTypeReference(r) => match &r.type_name {
            TSTypeName::IdentifierReference(id) => {
                if type_params.contains(id.name.as_str()) {
                    return false;
                }
                id.name == "Action" || aliases.contains(id.name.as_str())
            }
            TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => false,
        },
        TSType::TSUnionType(u) => u
            .types
            .iter()
            .any(|t| type_is_action(t, aliases, type_params)),
        TSType::TSIntersectionType(i) => i
            .types
            .iter()
            .any(|t| type_is_action(t, aliases, type_params)),
        TSType::TSParenthesizedType(p) => type_is_action(&p.type_annotation, aliases, type_params),
        _ => false,
    }
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run_at(path: &str, src: &str) -> Vec<Finding> {
        check(Path::new(path), src)
    }

    /// Red: sidecar import — Action implemented in compositions/.
    #[test]
    fn fixture_sidecar_import_fails() {
        let src = r#"
import type { Action } from '@ship/sail-runtime'
export const currentNav: Action<undefined> = (node) => {
  return { destroy() {} }
}
"#;
        let f = run_at("apps/site/src/compositions/current-nav.ts", src);
        assert!(!f.is_empty(), "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].message.contains(LAW), "{}", f[0].message);
        assert!(f[0].message.contains("currentNav"), "{}", f[0].message);
    }

    /// Red: mislabeled Action — alias hides the type name.
    #[test]
    fn fixture_mislabeled_action_fails() {
        let src = r#"
import type { Action } from '@ship/sail-runtime'
type ClickHandler = Action<void>
export const onClick: ClickHandler = (node) => {
  return { destroy() {} }
}
"#;
        let f = run_at("apps/site/src/compositions/click.ts", src);
        assert!(f.iter().any(|x| x.message.contains("onClick")), "{f:?}");
        assert!(f.iter().any(|x| x.message.contains(LAW)), "{f:?}");
    }

    /// Red: wrong folder — Action under stores/.
    #[test]
    fn fixture_wrong_folder_fails() {
        let src = r#"
import type { Action } from '@ship/sail-runtime'
export function mark(node: Element): Action<void> {
  return (_n) => {}
}
"#;
        let f = run_at("apps/site/src/stores/mark.ts", src);
        assert!(!f.is_empty(), "{f:?}");
        assert!(f.iter().any(|x| x.message.contains(LAW)), "{f:?}");
    }

    /// Green: sanctioned Action in src/actions/.
    #[test]
    fn fixture_sanctioned_action_home_passes() {
        let src = r#"
import type { Action } from '@ship/sail-runtime'
export const autofocus: Action<void> = (node) => {
  return { destroy() {} }
}
"#;
        let f = run_at("apps/site/src/actions/autofocus.ts", src);
        assert!(f.is_empty(), "{f:?}");
    }

    /// Green: composition wires state, no Action type.
    #[test]
    fn fixture_sanctioned_composition_passes() {
        let src = r#"
import { signal } from '@ship/sail-runtime'
export function listModel() {
  const items = signal<string[]>([])
  return { items }
}
"#;
        let f = run_at("apps/site/src/compositions/list-model.ts", src);
        assert!(f.is_empty(), "{f:?}");
    }

    /// Exemption: unused type-only import does not fire.
    #[test]
    fn exemption_unused_type_import() {
        let src = r#"
import type { Action } from '@ship/sail-runtime'
export const n = 1
"#;
        let f = run_at("apps/site/src/compositions/idle.ts", src);
        assert!(f.is_empty(), "{f:?}");
    }

    /// Exemption: Action mentioned only in a string / comment.
    #[test]
    fn exemption_string_and_comment() {
        let src = r#"
// export const x: Action<void> = (node) => {}
export const code = "export const autofocus: Action<void> = (node) => {}"
"#;
        let f = run_at("apps/site/src/compositions/snippets.ts", src);
        assert!(f.is_empty(), "{f:?}");
    }

    /// Exemption: type-only re-export of Action is not a function.
    #[test]
    fn exemption_type_only_reexport() {
        let src = "export type { Action } from '@ship/sail-runtime'\n";
        let f = run_at("apps/site/src/compositions/types.ts", src);
        assert!(f.is_empty(), "{f:?}");
    }

    /// Exemption: type parameter named Action is not the Action type.
    #[test]
    fn exemption_type_param_named_action() {
        let src = "export function wrap<Action>(x: Action): Action { return x }\n";
        let f = run_at("apps/site/src/compositions/wrap.ts", src);
        assert!(f.is_empty(), "{f:?}");
    }

    /// Exemption: function named Action with no Action type.
    #[test]
    fn exemption_value_named_action() {
        let src = "export function Action() { return 1 }\n";
        let f = run_at("apps/site/src/compositions/named.ts", src);
        assert!(f.is_empty(), "{f:?}");
    }
}
