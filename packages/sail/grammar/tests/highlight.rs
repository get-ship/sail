//! Highlight SSOT gates.
//!
//! - every `@capture` in highlights.scm is in [`tree_sitter_sail::highlight::SCOPES`]
//! - known constructs emit the scopes Studio / LSP map

use std::collections::BTreeSet;

use tree_sitter_sail::{highlight, highlight::SCOPES};

const HIGHLIGHTS_SCM: &str = include_str!("../queries/highlights.scm");

fn scm_captures(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() {
                let c = bytes[j];
                if c.is_ascii_alphanumeric() || c == b'.' || c == b'_' {
                    j += 1;
                } else {
                    break;
                }
            }
            if j > start {
                let name = &src[start..j];
                if !name.starts_with('_') {
                    out.insert(name.to_string());
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

#[test]
fn highlight_scopes_cover_query_captures() {
    let from_scm = scm_captures(HIGHLIGHTS_SCM);
    let known: BTreeSet<&str> = SCOPES.iter().copied().collect();
    let missing: Vec<_> = from_scm
        .iter()
        .filter(|c| !known.contains(c.as_str()))
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "highlights.scm has captures not in tree_sitter_sail::highlight::SCOPES \
         (add the scope + an LSP / Studio map row): {missing:?}"
    );
}

#[test]
fn style_and_controller_paint() {
    let src = r#"
<scope>
  style card = { pad(4) }
</scope>
<controller>
export default controller(() => ({ n: 1 }))
</controller>
<div style(card)>{n()}</div>
"#;
    let spans = highlight(src);
    let scopes: BTreeSet<&str> = spans.iter().map(|s| s.scope).collect();
    assert!(
        scopes.contains("keyword") || scopes.contains("function"),
        "style() / style decl must paint; scopes={scopes:?}"
    );
    assert!(
        scopes.contains("keyword.directive"),
        "scope/controller tags must paint as keyword.directive; scopes={scopes:?}"
    );
    let painted: String = spans
        .iter()
        .filter(|s| s.scope == "function" || s.scope == "keyword")
        .map(|s| src.get(s.start..s.end).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("|");
    assert!(
        painted.contains("style") || painted.contains("card"),
        "expected style/card in painted text, got {painted:?}"
    );
}
