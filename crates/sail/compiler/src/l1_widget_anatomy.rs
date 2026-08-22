//! Extra L1 inner-part anatomy stamps (same door as `stamp_language_anatomy`).
//! Combobox roles live on the field + list, not the display host.

use crate::ir::{AttrInstr, Instr};
use crate::semantic_props::stamp_language_anatomy;

/// `tab` inner parts plus `<combobox>` field/listbox. Called from `compile()`.
pub fn stamp(program: &mut crate::ir::Program, tag: &str) {
    stamp_language_anatomy(program, tag);
    if tag != "combobox" {
        return;
    }
    stamp_first_tag(&mut program.template, "input", "role", "combobox");
    if let Some(Instr::Element(root)) = program
        .template
        .iter_mut()
        .find(|i| matches!(i, Instr::Element(_)))
    {
        stamp_first_tag(&mut root.children, "div", "role", "listbox");
    }
}

fn stamp_first_tag(instrs: &mut [Instr], tag: &str, name: &str, value: &str) {
    for i in instrs {
        match i {
            Instr::Element(el) => {
                if el.tag.eq_ignore_ascii_case(tag) {
                    stamp_attr(&mut el.attrs, name, value);
                    return;
                }
                stamp_first_tag(&mut el.children, tag, name, value);
            }
            Instr::If(v) => {
                for b in &mut v.branches {
                    stamp_first_tag(&mut b.body, tag, name, value);
                }
                if let Some(eb) = &mut v.else_body {
                    stamp_first_tag(eb, tag, name, value);
                }
            }
            Instr::For(v) => {
                stamp_first_tag(&mut v.body, tag, name, value);
                if let Some(eb) = &mut v.else_body {
                    stamp_first_tag(eb, tag, name, value);
                }
            }
            Instr::Match(v) => {
                for c in &mut v.cases {
                    stamp_first_tag(&mut c.body, tag, name, value);
                }
                if let Some(d) = &mut v.default {
                    stamp_first_tag(d, tag, name, value);
                }
            }
            Instr::When(v) => {
                stamp_first_tag(&mut v.body, tag, name, value);
                if let Some(e) = &mut v.else_body {
                    stamp_first_tag(e, tag, name, value);
                }
            }
            _ => {}
        }
    }
}

fn stamp_attr(attrs: &mut Vec<AttrInstr>, name: &str, value: &str) {
    if attrs.iter().any(|a| match a {
        AttrInstr::Static { name: n, .. } => n == name,
        _ => false,
    }) {
        return;
    }
    attrs.push(AttrInstr::Static {
        name: name.into(),
        value: value.into(),
        span: crate::diagnostics::Span::point(0),
    });
}
