//! `sail-controller-matchmedia-layout` — layout breakpoints belong in Style
//! `when({ tablet: … })`, not in controller `matchMedia`.
//!
//! Flags `window.matchMedia(...)` / bare `matchMedia(...)` when:
//! 1. The query is **layout-flavored** (`min-width` / `max-width` / `orientation`), and
//! 2. The result's `.matches` **drives template-visible state** — it is stored into
//!    a `signal` / `.set(...)` / returned value (not merely used as a condition for a
//!    side effect like "close the drawer when the rail appears").
//!
//! **Not flagged** (legitimate imperative surface):
//! - `prefers-reduced-motion`, `prefers-color-scheme`, `print`, and other
//!   non-layout media queries
//! - Layout `matchMedia` used only as a condition (`if (mq.matches) …`) without
//!   writing `.matches` into reactive state
//!
//! Natural severity: **Warn** (surfaces in LSP / `sail-lint`; does not hard-fail
//! compile). Under `strict` profiles a host may promote via lint overrides —
//! same channel as other migration-Warn rules.
//!
//! Fix: Style `when({ tablet: … })` / `when({ mobile: … })` for sx; attr-level
//! responsive work lands under RESPONSIVE-1 (breakpoint-aware layout attrs).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-controller-matchmedia-layout";

/// Substrings that make a media query "layout-flavored". Case-insensitive.
const LAYOUT_FEATURES: &[&str] = &["min-width", "max-width", "orientation"];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = MatchMediaVisitor {
        source,
        file: path.to_path_buf(),
        // binding name → matchMedia call span (layout query only)
        layout_mqls: HashMap::new(),
        // binding name → originating matchMedia span (bound from .matches)
        layout_matches_vals: HashMap::new(),
        // matchMedia call spans already reported (by start offset)
        reported: HashSet::new(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor
        .findings
        .sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));
    visitor.findings
}

struct MatchMediaVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    layout_mqls: HashMap<String, Span>,
    layout_matches_vals: HashMap<String, Span>,
    reported: HashSet<u32>,
    findings: Vec<Finding>,
}

impl MatchMediaVisitor<'_> {
    fn report_at(&mut self, span: Span) {
        if !self.reported.insert(span.start) {
            return;
        }
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Warn,
            message: "layout breakpoints via `matchMedia` drive template-visible state \
                 — use Style `when({ tablet: … })` / `when({ mobile: … })` for sx \
                 instead. Breakpoint-aware attrs (RESPONSIVE-1) are the landing spot \
                 for attr-level responsive work. Non-layout media \
                 (`prefers-reduced-motion`, `prefers-color-scheme`, `print`) may stay \
                 imperative."
                .to_string(),
            suggest: Some(
                "For sx: `style.….when({ tablet: style.… })` (breakpoints are Style \
                 conditions, not controller signals). For layout track count / cols, \
                 prefer CSS `when` on the grid host once RESPONSIVE-1 attrs land — \
                 do not swap trees with `<if {isWide()}>` from matchMedia."
                    .to_string(),
            ),
        });
    }

    /// If `expr` is (or chains to) a layout-matches value, return the
    /// originating matchMedia span to report.
    fn layout_matches_origin(&self, expr: &Expression<'_>) -> Option<Span> {
        match expr {
            Expression::ParenthesizedExpression(p) => self.layout_matches_origin(&p.expression),
            Expression::TSAsExpression(a) => self.layout_matches_origin(&a.expression),
            Expression::TSSatisfiesExpression(s) => self.layout_matches_origin(&s.expression),
            Expression::TSNonNullExpression(n) => self.layout_matches_origin(&n.expression),
            Expression::TSTypeAssertion(t) => self.layout_matches_origin(&t.expression),
            Expression::LogicalExpression(l) => self
                .layout_matches_origin(&l.left)
                .or_else(|| self.layout_matches_origin(&l.right)),
            Expression::ConditionalExpression(c) => self
                .layout_matches_origin(&c.consequent)
                .or_else(|| self.layout_matches_origin(&c.alternate)),
            // `mq.matches` where mq is a layout MQL binding, or `matchMedia(...).matches`
            Expression::StaticMemberExpression(m) if m.property.name.as_str() == "matches" => {
                self.mql_object_span(&m.object)
            }
            // bare binding that holds a layout-matches boolean
            Expression::Identifier(id) => self.layout_matches_vals.get(id.name.as_str()).copied(),
            _ => None,
        }
    }

    /// `matchMedia(...)` call, or ident bound to a layout MQL.
    fn mql_object_span(&self, expr: &Expression<'_>) -> Option<Span> {
        match expr {
            Expression::ParenthesizedExpression(p) => self.mql_object_span(&p.expression),
            Expression::CallExpression(call) if is_match_media_callee(&call.callee) => {
                if layout_query_arg(call).is_some() {
                    Some(call.span)
                } else {
                    None
                }
            }
            Expression::Identifier(id) => self.layout_mqls.get(id.name.as_str()).copied(),
            _ => None,
        }
    }

    fn flag_if_layout_matches_value(&mut self, expr: &Expression<'_>) {
        if let Some(span) = self.layout_matches_origin(expr) {
            self.report_at(span);
        }
    }
}

impl<'a> Visit<'a> for MatchMediaVisitor<'_> {
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let Some(init) = &decl.init {
            // `const mq = window.matchMedia('(min-width: …)')`
            if let Some(span) = match_media_layout_span(init) {
                if let Some(name) = binding_name(&decl.id) {
                    self.layout_mqls.insert(name, span);
                }
            }
            // `const wide = mq.matches` / `const wide = matchMedia(...).matches`
            if let Some(span) = self.layout_matches_origin(init) {
                if let Some(name) = binding_name(&decl.id) {
                    self.layout_matches_vals.insert(name, span);
                }
            }
        }
        // Default walk: pattern + init (hits nested signal(...matchMedia...) too).
        oxc_ast_visit::walk::walk_variable_declarator(self, decl);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        // `signal(<layout-matches>)` or `foo.set(<layout-matches>)`
        if is_signal_callee(&expr.callee) || is_set_method(&expr.callee) {
            for arg in &expr.arguments {
                if let Some(e) = arg.as_expression() {
                    self.flag_if_layout_matches_value(e);
                }
            }
        }

        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e);
            }
        }
    }

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        if let Some(arg) = &stmt.argument {
            // `return { isWide }` shorthand or `return { isWide: …matches }`
            if let Expression::ObjectExpression(obj) = arg {
                for prop in &obj.properties {
                    if let ObjectPropertyKind::ObjectProperty(p) = prop {
                        self.flag_if_layout_matches_value(&p.value);
                    }
                }
            } else {
                self.flag_if_layout_matches_value(arg);
            }
            self.visit_expression(arg);
        }
    }
}

/// `matchMedia` or `window.matchMedia` / `globalThis.matchMedia` / `self.matchMedia`.
fn is_match_media_callee(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => id.name.as_str() == "matchMedia",
        Expression::StaticMemberExpression(m) => {
            m.property.name.as_str() == "matchMedia"
                && matches!(
                    &m.object,
                    Expression::Identifier(obj)
                        if matches!(obj.name.as_str(), "window" | "globalThis" | "self")
                )
        }
        Expression::ParenthesizedExpression(p) => is_match_media_callee(&p.expression),
        _ => false,
    }
}

fn is_signal_callee(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => id.name.as_str() == "signal",
        Expression::ParenthesizedExpression(p) => is_signal_callee(&p.expression),
        _ => false,
    }
}

/// `<expr>.set(...)` — signal writer.
fn is_set_method(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StaticMemberExpression(m) => m.property.name.as_str() == "set",
        Expression::ParenthesizedExpression(p) => is_set_method(&p.expression),
        _ => false,
    }
}

/// If `expr` is a layout `matchMedia(...)` call (possibly parenthesized /
/// as-cast), return its span.
fn match_media_layout_span(expr: &Expression<'_>) -> Option<Span> {
    match expr {
        Expression::ParenthesizedExpression(p) => match_media_layout_span(&p.expression),
        Expression::TSAsExpression(a) => match_media_layout_span(&a.expression),
        Expression::TSSatisfiesExpression(s) => match_media_layout_span(&s.expression),
        Expression::TSNonNullExpression(n) => match_media_layout_span(&n.expression),
        Expression::CallExpression(call) if is_match_media_callee(&call.callee) => {
            layout_query_arg(call).map(|_| call.span)
        }
        _ => None,
    }
}

/// First arg is a string literal containing a layout feature → Some(query).
fn layout_query_arg<'a>(call: &CallExpression<'a>) -> Option<&'a str> {
    let arg = call.arguments.first()?.as_expression()?;
    let q = string_literal(arg)?;
    if is_layout_query(q) {
        Some(q)
    } else {
        None
    }
}

fn string_literal<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::StringLiteral(s) => Some(s.value.as_str()),
        Expression::ParenthesizedExpression(p) => string_literal(&p.expression),
        Expression::TemplateLiteral(t) if t.expressions.is_empty() && t.quasis.len() == 1 => t
            .quasis[0]
            .value
            .cooked
            .as_ref()
            .map(|s| s.as_str())
            .or_else(|| Some(t.quasis[0].value.raw.as_str())),
        _ => None,
    }
}

fn is_layout_query(query: &str) -> bool {
    let lower = query.to_ascii_lowercase();
    LAYOUT_FEATURES.iter().any(|f| lower.contains(f))
}

fn binding_name(pat: &BindingPattern<'_>) -> Option<String> {
    match pat {
        BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
        _ => None,
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

    fn lint(src: &str) -> Vec<Finding> {
        check(Path::new("page.controller.ts"), src)
    }

    fn msgs(src: &str) -> Vec<String> {
        lint(src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn flags_signal_init_from_window_matchmedia_min_width() {
        let src = r#"
export default controller(() => {
  const isWide = signal(
    typeof window !== 'undefined' && window.matchMedia('(min-width: 768px)').matches,
  )
  return { isWide }
})
"#;
        let f = lint(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert!(
            f[0].message.contains("when({ tablet"),
            "message must name the fix: {}",
            f[0].message
        );
        assert!(
            f[0].message.contains("RESPONSIVE-1")
                || f[0]
                    .suggest
                    .as_ref()
                    .is_some_and(|s| s.contains("RESPONSIVE-1")),
            "must name RESPONSIVE-1: msg={} suggest={:?}",
            f[0].message,
            f[0].suggest
        );
    }

    #[test]
    fn flags_set_from_bound_mql_matches() {
        let src = r#"
export default controller(() => {
  const isWide = signal(false)
  mount(() => {
    const mq = window.matchMedia('(min-width: 768px)')
    const onChange = (): void => isWide.set(mq.matches)
    onChange()
    mq.addEventListener('change', onChange)
    return () => mq.removeEventListener('change', onChange)
  })
  return { isWide }
})
"#;
        let f = lint(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
    }

    #[test]
    fn flags_max_width_and_orientation() {
        assert_eq!(
            lint("const x = signal(matchMedia('(max-width: 600px)').matches)").len(),
            1
        );
        assert_eq!(
            lint("const x = signal(matchMedia('(orientation: landscape)').matches)").len(),
            1
        );
    }

    #[test]
    fn silent_on_prefers_reduced_motion() {
        let src = r#"
export default controller(() => {
  const reduce = signal(
    typeof window !== 'undefined' &&
      window.matchMedia('(prefers-reduced-motion: reduce)').matches,
  )
  mount(() => {
    const mq = window.matchMedia('(prefers-reduced-motion: reduce)')
    const onChange = (): void => reduce.set(mq.matches)
    mq.addEventListener('change', onChange)
    return () => mq.removeEventListener('change', onChange)
  })
  return { reduce }
})
"#;
        assert!(lint(src).is_empty(), "{:?}", msgs(src));
    }

    #[test]
    fn silent_on_prefers_color_scheme_and_print() {
        assert!(
            lint("const d = signal(matchMedia('(prefers-color-scheme: dark)').matches)")
                .is_empty()
        );
        assert!(lint("const p = signal(matchMedia('print').matches)").is_empty());
    }

    #[test]
    fn silent_when_matches_only_gates_side_effect() {
        // Rails pattern: close drawer when viewport crosses tablet — matches
        // is a condition, not written into state as the layout value.
        let src = r#"
export default controller(() => {
  const navOpen = signal(false)
  mount(() => {
    if (typeof window === 'undefined') return
    const mq = window.matchMedia('(min-width: 768px)')
    const onChange = (): void => {
      if (mq.matches) navOpen.set(false)
    }
    mq.addEventListener('change', onChange)
    return () => mq.removeEventListener('change', onChange)
  })
  return { navOpen }
})
"#;
        assert!(
            lint(src).is_empty(),
            "condition-only must be silent: {:?}",
            msgs(src)
        );
    }

    #[test]
    fn landing_shaped_fixture_fires() {
        // Minimal mirror of apps/sail-site/routes/page.sail controller.
        let src = r#"
export default controller((_ctx) => {
  const isWide = signal(
    typeof window !== 'undefined' && window.matchMedia('(min-width: 768px)').matches,
  )
  mount(() => {
    if (typeof window === 'undefined') return
    const mq = window.matchMedia('(min-width: 768px)')
    const onChange = (): void => isWide.set(mq.matches)
    onChange()
    mq.addEventListener('change', onChange)
    return () => mq.removeEventListener('change', onChange)
  })
  return { isWide }
})
"#;
        let f = lint(src);
        assert!(
            f.len() >= 1,
            "expected at least one finding on landing fixture, got {f:?}"
        );
        assert!(f.iter().all(|x| x.rule == RULE_ID));
    }
}
