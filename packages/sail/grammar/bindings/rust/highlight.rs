//! Highlight spans from `queries/highlights.scm`.
//!
//! One function, one query. `sail lsp` semantic tokens and the Studio
//! editor both consume this. Do not add a second capture table.

use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

/// One painted range. `scope` is the tree-sitter capture without `@`
/// (`tag`, `keyword.directive`, `function`, …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub scope: &'static str,
}

const HIGHLIGHTS_SCM: &str = include_str!("../../queries/highlights.scm");

/// Paint `src` with the committed highlight query.
///
/// Returns an empty vec on an empty document. Panics only if the
/// committed query cannot load against this parser (a generate drift).
pub fn highlight(src: &str) -> Vec<HighlightSpan> {
    if src.is_empty() {
        return Vec::new();
    }
    let mut parser = Parser::new();
    let language = crate::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("load sail language");
    let tree = parser.parse(src, None).expect("parse");
    let query = Query::new(&language.into(), HIGHLIGHTS_SCM)
        .expect("highlights.scm must be a valid query for this parser");
    let names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), src.as_bytes());
    let mut out = Vec::new();
    while let Some(m) = matches.next() {
        for cap in m.captures {
            let name = names[cap.index as usize];
            if name.starts_with('_') {
                continue;
            }
            // Capture names from the query live as long as HIGHLIGHTS_SCM.
            // Query::capture_names returns &str tied to the Query; leak is
            // avoided by intern_scope (static table of known scopes).
            let scope = intern_scope(name);
            out.push(HighlightSpan {
                start: cap.node.start_byte(),
                end: cap.node.end_byte(),
                scope,
            });
        }
    }
    out.sort_by_key(|s| (s.start, s.end, s.scope));
    out
}

/// Known highlight.scm captures. A new capture without a row fails
/// `tests/highlight.rs` (drift), so this table stays closed.
pub const SCOPES: &[&str] = &[
    "tag",
    "attribute",
    "string",
    "keyword",
    "keyword.directive",
    "function",
    "variable",
    "comment",
    "punctuation.bracket",
    "punctuation.delimiter",
];

fn intern_scope(name: &str) -> &'static str {
    for s in SCOPES {
        if *s == name {
            return s;
        }
    }
    "variable"
}
