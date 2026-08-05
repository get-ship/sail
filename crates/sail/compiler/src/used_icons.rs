//! Compile-time collection of lucide (icon pack) glyph names referenced by a
//! template. Used by the bundler to emit only the glyphs an app needs as
//! static `/_deck/icons/{name}.svg` assets — no runtime CDN.

use std::collections::BTreeSet;

use crate::ir::{AttrInstr, Instr};

/// Walk `instrs` and return every distinct static `name="…"` on an icon host.
///
/// After stdlib lowering, `<icon name="search">` is a `<span use:icon name="search">`.
/// Pre-lower / raw sources may still carry the `icon` tag. Both shapes are collected.
/// Dynamic `name={expr}` is intentionally skipped — only compile-known glyphs ship.
pub fn collect_used_icons(instrs: &[Instr]) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    visit(instrs, &mut found);
    found
}

/// Parse source and collect used icon names (full preprocess + parse).
pub fn collect_used_icons_from_source(source: &str) -> BTreeSet<String> {
    match crate::parse::parse(source) {
        Ok(program) => collect_used_icons(&program.template),
        Err(_) => BTreeSet::new(),
    }
}

fn visit(instrs: &[Instr], found: &mut BTreeSet<String>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                if is_icon_host(el) {
                    if let Some(name) = static_name_attr(&el.attrs) {
                        if is_plausible_icon_name(&name) {
                            found.insert(name);
                        }
                    }
                }
                visit(&el.children, found);
            }
            Instr::If(i) => {
                for branch in &i.branches {
                    visit(&branch.body, found);
                }
                if let Some(else_body) = &i.else_body {
                    visit(else_body, found);
                }
            }
            Instr::For(f) => {
                visit(&f.body, found);
                if let Some(else_body) = &f.else_body {
                    visit(else_body, found);
                }
            }
            Instr::Match(m) => {
                for case in &m.cases {
                    visit(&case.body, found);
                }
                if let Some(default) = &m.default {
                    visit(default, found);
                }
            }
            Instr::When(w) => {
                visit(&w.body, found);
                if let Some(e) = &w.else_body {
                    visit(e, found);
                }
            }
                        Instr::WhileLoading(wl) => visit(&wl.body, found),
            Instr::WhileError(we) => visit(&we.body, found),
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => {}
        }
    }
}

fn is_icon_host(el: &crate::ir::ElementInstr) -> bool {
    let tag = el.tag.as_str();
    if tag == "icon" || tag == "sail-icon" {
        return true;
    }
    el.attrs.iter().any(|a| {
        matches!(
            a,
            AttrInstr::Use {
                action,
                ..
            } if action == "icon"
        )
    })
}

fn static_name_attr(attrs: &[AttrInstr]) -> Option<String> {
    for a in attrs {
        if let AttrInstr::Static { name, value, .. } = a {
            if name == "name" {
                let v = value.trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

fn is_plausible_icon_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icons(src: &str) -> BTreeSet<String> {
        collect_used_icons_from_source(src)
    }

    #[test]
    fn collects_static_icon_names() {
        let got = icons(
            r#"<div>
            <icon name="search"></icon>
            <icon name="git-branch" size="lg"></icon>
            <icon name={dynamic()}></icon>
          </div>"#,
        );
        assert!(got.contains("search"), "{got:?}");
        assert!(got.contains("git-branch"), "{got:?}");
        assert!(!got.iter().any(|n| n.contains("dynamic")), "{got:?}");
        assert_eq!(got.len(), 2, "{got:?}");
    }

    #[test]
    fn nested_control_flow() {
        let got = icons(
            r#"<div>
            <if {c()}><icon name="check"></icon><else><icon name="x"></icon></else></if>
            <for {items()} as="i"><icon name="circle"></icon></for>
          </div>"#,
        );
        assert_eq!(
            got,
            BTreeSet::from(["check".into(), "x".into(), "circle".into()])
        );
    }

    #[test]
    fn ignores_non_icon_name_attrs() {
        let got = icons(r#"<input name="email" /><button name="submit"></button>"#);
        assert!(got.is_empty(), "{got:?}");
    }
}
