use std::collections::BTreeSet;

use crate::ir::Instr;

/// Walk `instrs` recursively and return every distinct custom-element tag
/// the template uses — i.e. every tag containing a `-` (`sail-*` / `ship-*`).
///
/// The compiler deliberately knows NOTHING about islands. Which of these
/// tags are lazy separate bundles is derived downstream by the bundler's
/// per-route tree-shaking resolver; deck-core intersects this used-tag set
/// with that manifest to build `payload.islands`.
pub fn collect_used_tags(instrs: &[Instr]) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    visit(instrs, &mut found);
    found
}

fn visit(instrs: &[Instr], found: &mut BTreeSet<String>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                if el.tag.contains('-') {
                    found.insert(el.tag.clone());
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
            Instr::WhileLoading(wl) => visit(&wl.body, found),
            Instr::WhileError(we) => visit(&we.body, found),
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn used_tags(src: &str) -> BTreeSet<String> {
        let program = crate::parse::parse(src).expect("source should parse");
        collect_used_tags(&program.template)
    }

    #[test]
    fn collects_custom_element_tags_only() {
        let tags = used_tags("<div><ship-terminal></ship-terminal><span>hi</span><ship-foo /></div>");
        assert!(tags.contains("ship-terminal"));
        assert!(tags.contains("ship-foo"));
        assert!(!tags.contains("div"), "plain HTML tags excluded");
        assert!(!tags.contains("span"), "plain HTML tags excluded");
    }

    #[test]
    fn finds_tags_nested_in_control_flow() {
        // if / for / match / while-loading / while-error each carry a distinct tag.
        let src = r#"<div>
            {#if cond()}<sail-a></sail-a>{:else}<sail-b></sail-b>{/if}
            {#for x of items()}<sail-c></sail-c>{/for}
            {#match m()}{:case 1}<sail-d></sail-d>{:default}<sail-e></sail-e>{/match}
            {#while-loading of data}<sail-f></sail-f>{/while-loading}
            {#while-error err of data}<sail-g></sail-g>{/while-error}
        </div>"#;
        let tags = used_tags(src);
        for t in ["sail-a", "sail-b", "sail-c", "sail-d", "sail-e", "sail-f", "sail-g"] {
            assert!(tags.contains(t), "missing nested tag {t}; got {tags:?}");
        }
    }
}
