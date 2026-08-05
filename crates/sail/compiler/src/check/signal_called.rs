//! Type-strengthening #5: a reactive value referenced without calling it.
//!
//! ```text
//! <p>{p}</p>                     // renders `()=>(ne(t),t.value)` — the SOURCE
//! <progress value={p} max="100"> // coerces to 0
//! ```
//!
//! Both compiled clean and warned about nothing. A signal is a getter
//! function; a render hole binds `() => $scope.p`, so the DOM receives the
//! function itself and stringifies it. The author sees a garbled page, not a
//! diagnostic, and there is nothing in the output to grep for. `{p()}` is the
//! correct form, and the scope symbol table already knows `p` is a signal —
//! so this is a cheap lookup, not an inference.
//!
//! ## Where it applies
//!
//! Every RENDERING position — text holes, attribute values, dynamic attribute
//! names, attribute conditions, and the `<if>` / `{#each}` / `<match>`
//! control-flow expressions. In each, the value is consumed, so a bare
//! reference is always the mistake.
//!
//! ## Where it deliberately does NOT apply
//!
//! The positions that want the CELL, not its value — passing the reactive
//! handle onward rather than reading it:
//!
//!   - `bind:value={p}` / `bind:checked={p}` — two-way binding writes back.
//!   - (none for plain fields — `name={p}` is a render position; call the
//!     signal. Hand a cell via `use:` / two-way `bind:` when needed.)
//!   - `use:action={p}` — the action receives the reactive param.
//!   - `on:click={handler}` — a handler is called by the event, not by the
//!     template (`on_callable` polices that position).
//!   - `fx:` config fields — read inside the runtime's own effect.
//!
//! An arrow body is skipped too: its params shadow the scope, and its
//! contents run at call time, not render time.

use crate::check::controller_parse::ParsedController;
use crate::check::symbol_table::{ScopeKind, SymbolTable};
use crate::diagnostics::CompileError;
use crate::ir::{AttrInstr, Expr, Instr, Program};
use crate::parse::expr::{Arg, Ast, AstKind, ObjKey, TemplatePart, parse_expression};

pub const RULE: &str = "sail-signal-not-called";

pub fn check(program: &Program, controller: Option<&ParsedController>) -> Result<(), CompileError> {
    match collect(program, controller).into_iter().next() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Every uncalled reactive reference in the program, for the LSP / agent
/// multi-diagnostic surface. `check()` returns only the first.
pub fn collect(program: &Program, controller: Option<&ParsedController>) -> Vec<CompileError> {
    let table = SymbolTable::build(program, controller);
    if table.scope.is_empty() {
        // No controller scope resolved — nothing is known to be a signal, and
        // guessing from a bare name would false-positive on every free ident.
        return Vec::new();
    }
    let ctx_head = controller.map(|c| c.ctx_param.as_str()).unwrap_or("");
    let mut cx = Cx {
        table: &table,
        ctx_head,
        errs: Vec::new(),
    };
    cx.walk_instrs(&program.template);
    cx.errs
}

struct Cx<'a> {
    table: &'a SymbolTable,
    ctx_head: &'a str,
    errs: Vec<CompileError>,
}

impl Cx<'_> {
    fn walk_instrs(&mut self, instrs: &[Instr]) {
        for instr in instrs {
            match instr {
                Instr::DynamicText { expr } => self.rendered(expr),
                Instr::Element(el) => {
                    for a in &el.attrs {
                        self.attr(a);
                    }
                    self.walk_instrs(&el.children);
                }
                Instr::If(i) => {
                    for b in &i.branches {
                        self.rendered(&b.cond);
                        self.walk_instrs(&b.body);
                    }
                    if let Some(eb) = &i.else_body {
                        self.walk_instrs(eb);
                    }
                }
                Instr::For(f) => {
                    self.rendered(&f.iterable);
                    if let Some(k) = &f.key {
                        self.rendered(k);
                    }
                    self.walk_instrs(&f.body);
                    if let Some(eb) = &f.else_body {
                        self.walk_instrs(eb);
                    }
                }
                Instr::Match(m) => {
                    self.rendered(&m.discriminant);
                    for c in &m.cases {
                        self.rendered(&c.test);
                        self.walk_instrs(&c.body);
                    }
                    if let Some(d) = &m.default {
                        self.walk_instrs(d);
                    }
                }
                Instr::When(w) => {
                    self.walk_instrs(&w.body);
                    if let Some(e) = &w.else_body {
                        self.walk_instrs(e);
                    }
                }
                                Instr::WhileLoading(wl) => self.walk_instrs(&wl.body),
                Instr::WhileError(we) => self.walk_instrs(&we.body),
                Instr::StaticText { .. } | Instr::Comment { .. } => {}
            }
        }
    }

    fn attr(&mut self, attr: &AttrInstr) {
        match attr {
            AttrInstr::Expr { expr, .. } => self.rendered(expr),
            // Handle-passing positions — see the module docs.
            AttrInstr::BindValue { .. }
            | AttrInstr::BindChecked { .. }
            | AttrInstr::Use { .. }
            | AttrInstr::On { .. }
            | AttrInstr::Motion { .. }
            | AttrInstr::Mutation { .. }
            | AttrInstr::SlotReflectEmpty { .. }
            | AttrInstr::Static { .. } => {}
        }
    }

    /// One expression in a rendering position.
    fn rendered(&mut self, expr: &Expr) {
        let Ok(ast) = parse_expression(&expr.source, expr.span.start) else {
            // Shape the grammar parser can't model — the lenient validators
            // still police it; a guess here would be a false accusation.
            return;
        };
        self.value(&ast);
    }

    /// Visit `node` in VALUE position — where a bare reactive reference is the
    /// mistake this pass exists to name.
    fn value(&mut self, node: &Ast) {
        match &node.kind {
            AstKind::Ident(name) => self.flag(name, name, node),
            AstKind::Member { obj, prop, .. } => {
                if self.is_ctx_head(obj) {
                    self.flag(prop, &format!("{}.{prop}", self.ctx_head), node);
                } else {
                    self.value(obj);
                }
            }
            // The callee of a call is the ONE position where a bare name is
            // right: `p()` is what we're asking for. Its receiver still isn't —
            // `items.map(…)` on a signal is `items().map(…)`.
            AstKind::Call { callee, args, .. } => {
                self.callee(callee);
                for a in args {
                    let (Arg::Expr(e) | Arg::Spread(e)) = a;
                    self.value(e);
                }
            }
            AstKind::Index { obj, idx, .. } => {
                self.value(obj);
                self.value(idx);
            }
            AstKind::Unary { arg, .. } => self.value(arg),
            AstKind::Binary { lhs, rhs, .. } | AstKind::Logical { lhs, rhs, .. } => {
                self.value(lhs);
                self.value(rhs);
            }
            AstKind::Ternary {
                cond,
                then,
                otherwise,
            } => {
                self.value(cond);
                self.value(then);
                self.value(otherwise);
            }
            AstKind::Template(parts) => {
                for p in parts {
                    if let TemplatePart::Expr(e) = p {
                        self.value(e);
                    }
                }
            }
            AstKind::Array(items) => {
                for a in items {
                    let (Arg::Expr(e) | Arg::Spread(e)) = a;
                    self.value(e);
                }
            }
            AstKind::Object(entries) => {
                for entry in entries {
                    if let ObjKey::Spread(e) = &entry.key {
                        self.value(e);
                    }
                    if let Some(v) = &entry.value {
                        self.value(v);
                    }
                }
            }
            // An arrow's params shadow the scope and its body runs at call
            // time, not render time.
            AstKind::Arrow { .. }
            | AstKind::Number(_)
            | AstKind::String(_)
            | AstKind::Bool(_)
            | AstKind::Null
            | AstKind::Undefined => {}
        }
    }

    /// Visit the callee of a call: the name itself is being invoked (correct),
    /// but anything it is reached THROUGH is still a value.
    fn callee(&mut self, node: &Ast) {
        match &node.kind {
            AstKind::Ident(_) => {}
            AstKind::Member { obj, .. } if self.is_ctx_head(obj) => {}
            _ => self.value(node),
        }
    }

    fn is_ctx_head(&self, node: &Ast) -> bool {
        !self.ctx_head.is_empty() && node.as_ident() == Some(self.ctx_head)
    }

    /// `name` is the scope key to classify; `shown` is how the author wrote it.
    fn flag(&mut self, name: &str, shown: &str, node: &Ast) {
        let Some(kind) = self.table.kind_of_scope(name) else {
            return;
        };
        if !reads_by_call(kind) {
            return;
        }
        self.errs.push(
            CompileError::new(format!(
                "`{{{shown}}}` is a {} referenced without calling it — did you mean `{{{shown}()}}`? \
                 A {} is a getter: rendered bare, the DOM receives the function itself.",
                kind.label(),
                kind.label()
            ))
            .with_span(node.span)
            .with_category(RULE),
        );
    }
}

/// Kinds whose value is only reachable through a call. `Action` is excluded —
/// a handler is passed, not read — and so is `Other`, which we can't classify.
fn reads_by_call(kind: ScopeKind) -> bool {
    matches!(
        kind,
        ScopeKind::Signal | ScopeKind::Derived | ScopeKind::Timeline
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use std::path::PathBuf;

    fn compiled(template: &str, ctrl_body: &str) -> Result<(), CompileError> {
        let src = format!("{template}");
        let program = crate::parse::parse(&src).expect("template parses");
        let c = parse(
            PathBuf::from("x.sail"),
            format!("export default controller((ctx) => ({{ {ctrl_body} }}))"),
        )
        .expect("controller parses");
        check(&program, Some(&c))
    }

    #[test]
    fn flags_bare_signal_text_hole() {
        let err = compiled("<p>{p}</p>", "p: signal(0)").unwrap_err();
        assert_eq!(err.category.as_deref(), Some(RULE));
        assert!(err.message.contains("{p()}"), "{}", err.message);
    }

    #[test]
    fn flags_bare_signal_attribute() {
        let err = compiled(r#"<progress value={p}></progress>"#, "p: signal(0)").unwrap_err();
        assert_eq!(err.category.as_deref(), Some(RULE));
    }

    #[test]
    fn flags_derived_and_ref_too() {
        assert!(compiled("<p>{d}</p>", "d: derived(() => 1)").is_err());
        assert!(compiled("<p>{el}</p>", "el: ref(null)").is_err());
    }

    #[test]
    fn flags_ctx_qualified_reference() {
        let err = compiled("<p>{ctx.p}</p>", "p: signal(0)").unwrap_err();
        assert!(err.message.contains("ctx.p"), "{}", err.message);
    }

    #[test]
    fn flags_member_access_on_an_uncalled_signal() {
        // `items.length` is the same silent bug wearing a suffix.
        let err = compiled("<p>{items.length}</p>", "items: signal([])").unwrap_err();
        assert!(err.message.contains("items"), "{}", err.message);
    }

    #[test]
    fn allows_the_called_forms() {
        compiled("<p>{p()}</p>", "p: signal(0)").unwrap();
        compiled("<p>{ctx.p()}</p>", "p: signal(0)").unwrap();
        compiled("<p>{items().length}</p>", "items: signal([])").unwrap();
        compiled("<p>{p() + 1}</p>", "p: signal(0)").unwrap();
    }

    #[test]
    fn allows_non_reactive_and_action_names() {
        compiled("<p>{plain}</p>", "plain: 42").unwrap();
        compiled("<p>{title}</p>", "title: 'hi'").unwrap();
        // An action referenced bare is `on_callable`'s business, not ours.
        compiled("<p>{inc}</p>", "inc: () => 1").unwrap();
    }

    #[test]
    fn ignores_unknown_names() {
        // Undeclared names are `scope_bound`'s error, reported before this pass.
        compiled("<p>{whatever}</p>", "p: signal(0)").unwrap();
    }
}
