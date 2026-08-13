//! Injection-query coverage for SPEC-LSP-1 highlighting.
//!
//! Asserts that `queries/injections.scm`:
//! - injects TypeScript into `<controller>` / `<scope>` text bodies
//! - keeps TypeScript injection on interpolation / brace_expression
//! - keeps CSS injection on inline `<style>` text bodies

use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

const INJECTIONS_SCM: &str = include_str!("../queries/injections.scm");

fn parse(src: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    let language = tree_sitter_sail::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("load sail language");
    parser.parse(src, None).expect("parse")
}

fn injections_query() -> Query {
    let language = tree_sitter_sail::LANGUAGE;
    Query::new(&language.into(), INJECTIONS_SCM).expect("injections.scm must be a valid query")
}

/// Collect (language, content_text) for every `@injection.content` match.
fn injection_hits(src: &str) -> Vec<(String, String)> {
    let tree = parse(src);
    let query = injections_query();
    let content_idx = query
        .capture_index_for_name("injection.content")
        .expect("@injection.content capture");

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), src.as_bytes());
    let mut out = Vec::new();
    while let Some(m) = matches.next() {
        let lang = query
            .property_settings(m.pattern_index)
            .iter()
            .find(|p| p.key.as_ref() == "injection.language")
            .and_then(|p| p.value.as_ref())
            .map(|v| v.as_ref().to_string())
            .unwrap_or_default();
        for cap in m.captures {
            if cap.index == content_idx {
                let text = cap.node.utf8_text(src.as_bytes()).unwrap_or("").to_string();
                out.push((lang.clone(), text));
            }
        }
    }
    out
}

#[test]
fn injections_query_parses() {
    let _ = injections_query();
}

#[test]
fn controller_body_injects_typescript() {
    let src = r#"
<controller>
export default controller(() => ({ count: 0 }))
</controller>
"#;
    let hits = injection_hits(src);
    let ts_hits: Vec<_> = hits
        .iter()
        .filter(|(lang, _)| lang == "typescript")
        .collect();
    assert!(
        ts_hits
            .iter()
            .any(|(_, text)| text.contains("export default controller")),
        "controller body text must inject as typescript; hits={hits:?}",
    );
}

#[test]
fn scope_body_injects_typescript() {
    let src = r#"
<scope>
  const card = style.pad(4)
</scope>
"#;
    let hits = injection_hits(src);
    assert!(
        hits.iter()
            .any(|(lang, text)| { lang == "typescript" && text.contains("const card") }),
        "scope body text must inject as typescript; hits={hits:?}",
    );
}

#[test]
fn interpolation_and_brace_still_inject_typescript() {
    let src = r#"
<p>{count()}</p>
<input value={name()} />
"#;
    let hits = injection_hits(src);
    let ts: Vec<_> = hits
        .iter()
        .filter(|(lang, _)| lang == "typescript")
        .map(|(_, t)| t.as_str())
        .collect();
    assert!(
        ts.iter().any(|t| t.contains("count")),
        "interpolation must still inject typescript; hits={hits:?}",
    );
    assert!(
        ts.iter().any(|t| t.contains("name")),
        "brace_expression must still inject typescript; hits={hits:?}",
    );
}

#[test]
fn style_body_still_injects_css() {
    let src = r#"
<style>
.foo color red
</style>
"#;
    let hits = injection_hits(src);
    assert!(
        hits.iter()
            .any(|(lang, text)| lang == "css" && text.contains(".foo")),
        "style body must still inject css; hits={hits:?}",
    );
}

#[test]
fn plain_element_text_does_not_inject_typescript() {
    let src = r#"
<div>
hello world
</div>
"#;
    let hits = injection_hits(src);
    assert!(
        !hits
            .iter()
            .any(|(lang, text)| { lang == "typescript" && text.contains("hello world") }),
        "plain element text must not inject as typescript; hits={hits:?}",
    );
}
