//! Cross-check symbol table built from a paired controller.
//!
//! The byte-walking scope-bound check answers "is this name declared?"
//! but stops short of "what kind of binding is it?". The
//! type-strengthening checks need the second answer: an arrow-shaped
//! scope action behaves differently from a ref-shaped scope handle,
//! and a derived getter is only a callable when invoked with `()`.
//!
//! This module owns the flat symbol table each downstream check
//! (`scope_member`, `bind_this_target`, `on_callable`) reads instead of
//! re-walking the controller source. The controller's setup function
//! returns ONE flat scope object — there is no more `derived:`/`on:`/
//! `use:` section to separate; everything the setup function returns
//! lives in `SymbolTable::scope`, classified by its value's shape (see
//! `controller_parse::ScopeKind`).

use std::collections::BTreeMap;

use crate::check::controller_parse::ParsedController;
pub use crate::check::controller_parse::ScopeKind;
use crate::ir::Program;

/// Symbol table merged across the program's `<prop>` directives and the
/// paired controller's returned scope members.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub props: BTreeMap<String, KeyEntry>,
    /// Setup-function return keys, with kinds.
    pub scope: BTreeMap<String, ScopeEntry>,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    /// Byte offset into the controller source (or 0 if the key is
    /// declared in the `.sui`).
    pub offset: usize,
    /// Source kind label — surfaced in hover.
    pub kind_label: &'static str,
}

#[derive(Debug, Clone)]
pub struct ScopeEntry {
    pub offset: usize,
    pub kind: ScopeKind,
}

impl SymbolTable {
    pub fn build(program: &Program, controller: Option<&ParsedController>) -> Self {
        let mut t = Self::default();
        for p in &program.props {
            let cm = crate::emit::util::camel_of(&p.name);
            t.props.insert(cm, KeyEntry { offset: 0, kind_label: "prop" });
        }
        let Some(c) = controller else {
            return t;
        };
        for member in &c.returned {
            t.scope.insert(
                member.name.clone(),
                ScopeEntry { offset: member.key_span.start, kind: member.kind },
            );
        }
        t
    }

    pub fn kind_of_scope(&self, name: &str) -> Option<ScopeKind> {
        self.scope.get(name).map(|e| e.kind)
    }

    /// Iterator of every (name, label) pair for completion.
    pub fn all_root_names(&self) -> Vec<(String, &'static str)> {
        let mut out: Vec<(String, &'static str)> = Vec::new();
        out.extend(self.props.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.scope.iter().map(|(k, v)| (k.clone(), v.kind.label())));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use std::path::PathBuf;

    fn ctrl(src: &str) -> ParsedController {
        parse(PathBuf::from("test.controller.ts"), src.to_string()).expect("parsed")
    }

    fn empty_program() -> Program {
        Program {
            props: vec![],
            styles: String::new(),
            scope_types: String::new(),
            template: vec![],
            source_len: 0,
            span: crate::diagnostics::Span::point(0),
        }
    }

    #[test]
    fn classifies_scope_kinds() {
        let c = ctrl(
            r#"export default controller((ctx) => ({
                el: ref(null),
                n: signal(0),
                doubled: derived(() => 1),
                handle: (e) => 1,
                plain: 42,
            }))"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("el"), Some(ScopeKind::Ref));
        assert_eq!(t.kind_of_scope("n"), Some(ScopeKind::Signal));
        assert_eq!(t.kind_of_scope("doubled"), Some(ScopeKind::Derived));
        assert_eq!(t.kind_of_scope("handle"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("plain"), Some(ScopeKind::Other));
    }

    #[test]
    fn classifies_async_and_function_expression_values() {
        // Pre-fix, `toggleStream: function toggleStream(): void {…}` and
        // `openModal: async (): Promise<void> => {…}` both classified as
        // Other, so `on:click={toggleStream}` was rejected as "not
        // callable", forcing arrow wrap. Real-world idioms (named
        // function expressions for stack-trace readability, async
        // handlers) should round-trip the on:-callable check.
        let c = ctrl(
            r#"export default controller((ctx) => ({
                asyncArrow: async (): Promise<void> => {},
                asyncShort: async e => e,
                fnExpr: function foo(): void {},
                fnAnon: function () { return 1 },
                asyncFn: async function bar() {},
                gen: function* () { yield 1 },
            }))"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("asyncArrow"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("asyncShort"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("fnExpr"),     Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("fnAnon"),     Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("asyncFn"),    Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("gen"),        Some(ScopeKind::Action));
    }

    #[test]
    fn classifies_shorthand_arrow_const() {
        let c = ctrl(
            r#"export default controller((ctx) => {
                const inc = (e) => 1;
                const typed = (e: KeyboardEvent): void => { return };
                function dec() { return 2 }
                const n = signal(0);
                return { inc, typed, dec, n };
            })"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("inc"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("typed"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("dec"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("n"), Some(ScopeKind::Signal));
    }

    #[test]
    fn classifies_ref_with_ts_generic() {
        let c = ctrl(
            r#"export default controller((ctx) => ({
                btn: ref<HTMLButtonElement>(null),
                list: signal<string[]>([]),
            }))"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("btn"), Some(ScopeKind::Ref));
        assert_eq!(t.kind_of_scope("list"), Some(ScopeKind::Signal));
    }

    #[test]
    fn classifies_derived_and_action_keys() {
        let c = ctrl(
            r#"export default controller((ctx) => ({
                total: derived(() => 1),
                increment: () => {},
            }))"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("total"), Some(ScopeKind::Derived));
        assert_eq!(t.kind_of_scope("increment"), Some(ScopeKind::Action));
    }
}
