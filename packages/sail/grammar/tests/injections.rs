//! Injection-query coverage for SPEC-LSP-1 highlighting.
//!
//! Asserts that `queries/injections.scm`:
//! - injects TypeScript into `<controller>` raw bodies
//! - injects TypeScript into `style name = {…}` / `props: {…}` braces
//! - keeps TypeScript injection on interpolation / brace_expression
//! - keeps CSS injection on inline `<style>` text bodies

use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

const INJECTIONS_SCM: &str = include_str!("../queries/injections.scm");
const HIGHLIGHTS_SCM: &str = include_str!("../queries/highlights.scm");

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
fn highlights_query_parses() {
    let language = tree_sitter_sail::LANGUAGE;
    Query::new(&language.into(), HIGHLIGHTS_SCM).expect("highlights.scm must be a valid query");
}

fn has_error(src: &str) -> bool {
    parse(src).root_node().has_error()
}

#[test]
fn controller_with_ts_generics_parses() {
    let src = r#"
<controller>
interface Lap { id: number }
const laps = signal<Lap[]>([])
export default controller(() => ({ laps }))
</controller>
"#;
    assert!(!has_error(src), "controller body must stay raw through `<` generics");
}

#[test]
fn scope_style_and_props_parse() {
    let src = r#"
<scope>
  type Row = { id: string }
  props: {
    label = 'Counter'
  }
  style card = { pad(4).bg(surface) }
  style quiet = host, { fg(muted-foreground) }
</scope>
"#;
    assert!(!has_error(src), "scope must parse style_decl + props + leftover type lines");
}

#[test]
fn style_attach_on_element_parses() {
    let src = r#"
<row nowrap gap="2" style(card, { grow(1) })>
  <strong style(name)>x</strong>
  <div style({ pad(4) })>y</div>
</row>
"#;
    assert!(!has_error(src), "tag-position style() must parse as style_attr");
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
        "controller_body must inject as typescript; hits={hits:?}",
    );
}

#[test]
fn scope_style_chain_injects_typescript() {
    let src = r#"
<scope>
  style card = { pad(4).bg(surface) }
</scope>
"#;
    let hits = injection_hits(src);
    assert!(
        hits.iter()
            .any(|(lang, text)| lang == "typescript" && text.contains("pad")),
        "style_decl brace body must inject as typescript; hits={hits:?}",
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
