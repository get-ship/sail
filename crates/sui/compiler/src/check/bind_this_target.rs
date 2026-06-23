//! Type-strengthening #3: `bind:this={X}` target.
//!
//! `bind:this` writes the rendered DOM element into a slot the
//! controller declared. The runtime expects that slot to be a
//! `ref(...)` — refs are write-once-from-runtime, signal-shaped on
//! read, and the only kind of scope key the runtime actually rebinds.
//!
//! Without this check, `bind:this={mySignal}` typechecks at the
//! parser level (the ident exists), then the runtime silently
//! overwrites the signal slot with an Element handle, breaking every
//! later read.
//!
//! Rule: `bind:this={X}` requires `X` to be a `scope:` key declared
//! as `X: ref(...)` in the paired controller.

use crate::check::controller_parse::ParsedController;
use crate::check::symbol_table::{ScopeKind, SymbolTable};
use crate::diagnostics::CompileError;
use crate::ir::{AttrInstr, Instr, Program};

pub fn check(
    program: &Program,
    controller: Option<&ParsedController>,
) -> Result<(), CompileError> {
    let table = SymbolTable::build(program, controller);
    walk(&program.template, &table)
}

fn walk(instrs: &[Instr], table: &SymbolTable) -> Result<(), CompileError> {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                for a in &el.attrs {
                    if let AttrInstr::BindThis { var, span } = a {
                        validate(var, *span, table)?;
                    }
                }
                walk(&el.children, table)?;
            }
            Instr::If(i) => {
                for b in &i.branches {
                    walk(&b.body, table)?;
                }
                if let Some(eb) = &i.else_body {
                    walk(eb, table)?;
                }
            }
            Instr::For(f) => {
                walk(&f.body, table)?;
                if let Some(eb) = &f.else_body {
                    walk(eb, table)?;
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    walk(&c.body, table)?;
                }
                if let Some(d) = &m.default {
                    walk(d, table)?;
                }
            }
            Instr::WhileLoading(wl) => walk(&wl.body, table)?,
            Instr::WhileError(we) => walk(&we.body, table)?,
            _ => {}
        }
    }
    Ok(())
}

fn validate(
    var: &str,
    span: crate::diagnostics::Span,
    table: &SymbolTable,
) -> Result<(), CompileError> {
    let Some(kind) = table.kind_of_scope(var) else {
        // The scope-bound check already reports unresolved idents; if
        // we get here it means the var wasn't classified as a scope
        // key — the controller might be missing or this name lives in
        // a different section. Either way, surface a focused error
        // pointing at the runtime contract.
        if !table.scope.is_empty() {
            return Err(CompileError::new(format!(
                "bind:this={{{var}}} expects `{var}` to be declared as `{var}: ref(...)` in the controller's scope: factory"
            ))
            .with_span(span)
            .with_category("bind:this".to_string()));
        }
        return Ok(());
    };
    if kind != ScopeKind::Ref {
        return Err(CompileError::new(format!(
            "bind:this={{{var}}} requires a `ref(...)` scope key, but `{var}` is declared as a {} — change to `{var}: ref(null)` in the controller's scope: factory",
            kind.label()
        ))
        .with_span(span)
        .with_category("bind:this".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use crate::diagnostics::Span;
    use crate::ir::{ElementInstr, Instr};
    use std::path::PathBuf;

    fn ctrl(src: &str) -> ParsedController {
        parse(PathBuf::from("x.controller.ts"), src.to_string()).unwrap()
    }

    fn program_with_bind_this(var: &str) -> Program {
        let attr = AttrInstr::BindThis {
            var: var.to_string(),
            span: Span::point(0),
        };
        let el = ElementInstr {
            tag: "div".into(),
            attrs: vec![attr],
            children: vec![],
            span: Span::point(0),
        };
        Program {
            data: vec![],
            props: vec![],
            clients: vec![],
            styles: String::new(),
            template: vec![Instr::Element(el)],
            source_len: 0,
            span: Span::point(0),
        }
    }

    #[test]
    fn allows_bind_this_to_ref() {
        let c = ctrl("export default controller({ scope: () => ({ el: ref(null) }) });");
        let p = program_with_bind_this("el");
        check(&p, Some(&c)).unwrap();
    }

    #[test]
    fn rejects_bind_this_to_signal() {
        let c = ctrl("export default controller({ scope: () => ({ el: signal(0) }) });");
        let p = program_with_bind_this("el");
        let err = check(&p, Some(&c)).unwrap_err();
        assert!(err.message.contains("`ref(...)`"), "{}", err.message);
        assert!(err.message.contains("signal"), "{}", err.message);
    }

    #[test]
    fn rejects_bind_this_to_unknown() {
        let c = ctrl("export default controller({ scope: () => ({ el: ref(null) }) });");
        let p = program_with_bind_this("foo");
        let err = check(&p, Some(&c)).unwrap_err();
        assert!(err.message.contains("scope: factory"), "{}", err.message);
    }
}
