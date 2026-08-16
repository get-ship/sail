//! Compile-time collection of icon **roles** referenced by a template.
//! The bundler resolves roles through [`crate::icons::RoleRegistry`] and
//! emits only those glyphs — no runtime CDN, no full-pack embed.

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
/// Parse failure (standalone docs hosts, snippets) still scans `<icon name="…">`
/// so a broken file cannot 404 every glyph on the page.
pub fn collect_used_icons_from_source(source: &str) -> BTreeSet<String> {
    let mut found = match crate::parse::parse(source) {
        Ok(program) => collect_used_icons(&program.template),
        Err(_) => BTreeSet::new(),
    };
    for name in collect_used_icons_heuristic(source) {
        found.insert(name);
    }
    found
}

fn collect_used_icons_heuristic(source: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut rest = source;
    while let Some(pos) = rest.find("<icon") {
        let tag = &rest[pos..];
        if !icon_tag_boundary(&tag[5..]) {
            rest = &tag[5..];
            continue;
        }
        let end = tag.find('>').unwrap_or(tag.len());
        let open = &tag[..end];
        if let Some(name) = extract_static_name(open) {
            if is_plausible_icon_name(&name) {
                found.insert(name);
            }
        }
        rest = &tag[5..];
    }
    found
}

/// `<icon` then whitespace / `>` / `/` — not `<iconography` or `<icons>`.
fn icon_tag_boundary(after_icon: &str) -> bool {
    match after_icon.chars().next() {
        Some(c) => c.is_whitespace() || c == '>' || c == '/',
        None => false,
    }
}

fn extract_static_name(open_tag: &str) -> Option<String> {
    for key in ["name=\"", "name='"] {
        let Some(i) = open_tag.find(key) else {
            continue;
        };
        let after = &open_tag[i + key.len()..];
        let quote = key.chars().last()?;
        let end = after.find(quote)?;
        let v = after[..end].trim();
        if !v.is_empty() && !v.starts_with('{') {
            return Some(v.to_string());
        }
    }
    None
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
            <if {c()}><icon name="check"></icon>{else => <icon name="x"></icon>}</if>
            <for {items() as i}><icon name="circle"></icon></for>
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

    #[test]
    fn heuristic_recovers_icons_from_unparseable_source() {
        let got = icons(
            r#"<controller>not valid sail</controller>
            <icon name="sparkles"></icon>
            <icon name="inbox" size="sm"></icon>
            <icon name={dyn()}></icon>"#,
        );
        assert!(got.contains("sparkles"), "{got:?}");
        assert!(got.contains("inbox"), "{got:?}");
        assert!(!got.iter().any(|n| n.contains("dyn")), "{got:?}");
    }

    #[test]
    fn heuristic_unions_string_literal_names_when_parse_succeeds() {
        let got = icons(
            r#"<controller>
export default controller(() => ({
  sample: () => `<icon name="sparkles"></icon>`
}))
</controller>
<div>ok</div>"#,
        );
        assert!(got.contains("sparkles"), "{got:?}");
    }

    #[test]
    fn heuristic_skips_lookalike_tags_and_implausible_names() {
        let got = icons(
            r#"<controller>broken
            <iconography name="sparkles"></iconography>
            <icons name="inbox"></icons>
            <icon name=""></icon>
            <icon name="Not Valid"></icon>
            <icon name="way-too-long-to-be-a-role-name-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"></icon>
            <icon name='heart'></icon>
            </controller>"#,
        );
        assert!(!got.contains("sparkles"), "{got:?}");
        assert!(!got.contains("inbox"), "{got:?}");
        assert!(!got.contains("Not Valid"), "{got:?}");
        assert!(got.contains("heart"), "{got:?}");
        assert_eq!(got.len(), 1, "{got:?}");
    }
}
