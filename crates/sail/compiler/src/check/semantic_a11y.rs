//! A11y IR checks: icon-only buttons require a `label` semantic prop.

use crate::diagnostics::CompileError;
use crate::ir::*;

/// Run a11y IR checks. Returns the first error.
pub fn check(program: &Program) -> Result<(), CompileError> {
    for instr in &program.template {
        check_instr(instr)?;
    }
    Ok(())
}

/// Collect all a11y IR errors (for check_all).
pub fn collect(program: &Program) -> Vec<CompileError> {
    let mut out = Vec::new();
    walk_collect(&program.template, &mut out);
    out
}

fn walk_collect(instrs: &[Instr], out: &mut Vec<CompileError>) {
    for instr in instrs {
        if let Instr::Element(el) = instr {
            if let Err(e) = check_element_self(el) {
                out.push(e);
            }
            walk_collect(&el.children, out);
        }
        match instr {
            Instr::If(i) => {
                for b in &i.branches {
                    walk_collect(&b.body, out);
                }
                if let Some(body) = &i.else_body {
                    walk_collect(body, out);
                }
            }
            Instr::For(f) => {
                walk_collect(&f.body, out);
                if let Some(body) = &f.else_body {
                    walk_collect(body, out);
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    walk_collect(&c.body, out);
                }
                if let Some(body) = &m.default {
                    walk_collect(body, out);
                }
            }
            Instr::When(w) => {
                walk_collect(&w.body, out);
                if let Some(body) = &w.else_body {
                    walk_collect(body, out);
                }
            }
            _ => {}
        }
    }
}

fn check_instr(instr: &Instr) -> Result<(), CompileError> {
    match instr {
        Instr::Element(el) => {
            check_element_self(el)?;
            for c in &el.children {
                check_instr(c)?;
            }
        }
        Instr::If(i) => {
            for b in &i.branches {
                for c in &b.body {
                    check_instr(c)?;
                }
            }
            if let Some(body) = &i.else_body {
                for c in body {
                    check_instr(c)?;
                }
            }
        }
        Instr::For(f) => {
            for c in &f.body {
                check_instr(c)?;
            }
            if let Some(body) = &f.else_body {
                for c in body {
                    check_instr(c)?;
                }
            }
        }
        Instr::Match(m) => {
            for arm in &m.cases {
                for c in &arm.body {
                    check_instr(c)?;
                }
            }
            if let Some(body) = &m.default {
                for c in body {
                    check_instr(c)?;
                }
            }
        }
        Instr::When(w) => {
            for c in &w.body {
                check_instr(c)?;
            }
            if let Some(body) = &w.else_body {
                for c in body {
                    check_instr(c)?;
                }
            }
        }
        Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => {}
    }
    Ok(())
}

fn check_element_self(el: &ElementInstr) -> Result<(), CompileError> {
    let tag = el.tag.to_ascii_lowercase();
    if tag == "button" && is_icon_only(el) && !has_label(el) {
        return Err(CompileError::new(
            "icon-only <button> requires semantic prop `label` \
             (accessible name) — authors never write aria-label",
        )
        .with_span(el.span)
        .with_category("semantic-a11y"));
    }
    Ok(())
}

fn has_label(el: &ElementInstr) -> bool {
    for a in &el.attrs {
        match a {
            AttrInstr::Static { name, value, .. } if name == "label" || name == "aria-label" => {
                if !value.trim().is_empty() {
                    return true;
                }
            }
            AttrInstr::Expr { name, .. } if name == "label" || name == "aria-label" => {
                return true;
            }
            // title= is a weak name fallback some icon buttons use
            AttrInstr::Static { name, value, .. }
                if name == "title" && !value.trim().is_empty() =>
            {
                return true;
            }
            AttrInstr::Expr { name, .. } if name == "title" => return true,
            _ => {}
        }
    }
    false
}

/// True when the element has no meaningful text content — only empty text,
/// comments, or decorative svg/icon children.
fn is_icon_only(el: &ElementInstr) -> bool {
    if el.children.is_empty() {
        return true;
    }
    el.children.iter().all(is_decorative_child)
}

fn is_decorative_child(instr: &Instr) -> bool {
    match instr {
        Instr::Comment { .. } => true,
        Instr::StaticText { text, .. } => text.trim().is_empty(),
        Instr::DynamicText { .. } => false,
        Instr::Element(child) => {
            let t = child.tag.to_ascii_lowercase();
            if matches!(
                t.as_str(),
                "svg"
                    | "icon"
                    | "img"
                    | "path"
                    | "circle"
                    | "rect"
                    | "line"
                    | "polyline"
                    | "polygon"
                    | "use"
                    | "g"
            ) {
                return true;
            }
            if matches!(t.as_str(), "span" | "row" | "stack" | "center" | "div") {
                return is_icon_only(child);
            }
            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    fn sp() -> Span {
        Span::new(0, 1)
    }

    fn btn(attrs: Vec<AttrInstr>, children: Vec<Instr>) -> ElementInstr {
        ElementInstr {
            tag: "button".into(),
            attrs,
            children,
            span: sp(),
        }
    }

    fn prog(el: ElementInstr) -> Program {
        Program {
            props: vec![],
            styles: String::new(),
            scope_types: String::new(),
            template: vec![Instr::Element(el)],
            source_len: 0,
            span: sp(),
        }
    }

    #[test]
    fn icon_only_without_label_errors() {
        let el = btn(
            vec![],
            vec![Instr::Element(ElementInstr {
                tag: "svg".into(),
                attrs: vec![],
                children: vec![],
                span: sp(),
            })],
        );
        assert!(check(&prog(el)).is_err());
    }

    #[test]
    fn icon_only_with_label_ok() {
        let el = btn(
            vec![AttrInstr::Static {
                name: "label".into(),
                value: "Close".into(),
                span: sp(),
            }],
            vec![Instr::Element(ElementInstr {
                tag: "svg".into(),
                attrs: vec![],
                children: vec![],
                span: sp(),
            })],
        );
        assert!(check(&prog(el)).is_ok());
    }

    #[test]
    fn text_button_ok() {
        let el = btn(
            vec![],
            vec![Instr::StaticText {
                text: "Save".into(),
                span: sp(),
            }],
        );
        assert!(check(&prog(el)).is_ok());
    }
}
