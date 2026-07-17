//! Hand-rolled HTML parser scoped to `.sail` files.
//!
//! After `preprocess::preprocess` has run, the source is well-formed
//! enough that we don't need a full HTML5 implementation:
//!  - All non-void self-closes have been expanded.
//!  - All brace attributes are quoted.
//!
//! Span-tracking is the reason we don't reach for `html5ever`: every
//! IR node, every attribute, every text run carries a byte range so
//! diagnostics point at the user's source.
//!
//! The Parser implementation is split into `parser.rs` to keep both
//! files under the 300-line cap. This file owns the public surface
//! (types + `parse_fragment`).

use crate::diagnostics::{CompileError, Span};

mod parser;

pub(super) const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "source", "track", "wbr",
];

/// Tags whose body is treated as raw text (no nested elements,
/// closing handled by the literal `</tag>` match).
pub(super) const RAW_TEXT_TAGS: &[&str] = &["script", "style", "controller", "scope"];

/// Parse a `.sail` source fragment into a flat list of top-level
/// nodes. The fragment is parsed at "body" level — no document/html
/// wrapping, no doctype handling.
pub fn parse_fragment(source: &str) -> Result<Vec<Node>, CompileError> {
    let mut p = parser::Parser::new(source);
    p.parse_children(None)
}

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(Text),
    Comment(Comment),
}

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub attributes: Vec<Attr>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub value: String,
    pub span: Span,
    /// Span of just the value (between quotes when quoted, or the
    /// raw value byte range when unquoted/boolean).
    pub value_span: Span,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub data: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub data: String,
    pub span: Span,
}

impl Element {
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.clone())
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|a| a.name == name)
    }
}

/// Decode HTML entities found in `.sail` text content + attribute values.
///
/// Supports:
///   * The five XML core entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`,
///     `&apos;`).
///   * A curated list of common typographic / symbol entities authors
///     reach for (`&middot;`, `&hellip;`, `&copy;`, etc.). Without
///     this, templates using `&middot;` rendered the literal text
///     `&middot;` to the page (visible bug in `views/git/page.sail`
///     where every `<a> &middot; <b>` pair surfaced as undecoded HTML).
///   * Numeric character references — both decimal `&#NNN;` and
///     hexadecimal `&#xHEX;` / `&#XHEX;`.
///
/// Anything unrecognized passes through verbatim. The decoder is
/// linear-time over the input plus the named-entity replace passes.
pub fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = s
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
        .replace("&nbsp;", "\u{a0}");
    for (named, repl) in NAMED_ENTITIES {
        if out.contains(named) {
            out = out.replace(named, repl);
        }
    }
    decode_numeric_refs(&out)
}

const NAMED_ENTITIES: &[(&str, &str)] = &[
    ("&middot;", "·"),
    ("&hellip;", "…"),
    ("&copy;", "©"),
    ("&reg;", "®"),
    ("&trade;", "™"),
    ("&mdash;", "—"),
    ("&ndash;", "–"),
    ("&laquo;", "«"),
    ("&raquo;", "»"),
    ("&lsquo;", "\u{2018}"),
    ("&rsquo;", "\u{2019}"),
    ("&ldquo;", "\u{201C}"),
    ("&rdquo;", "\u{201D}"),
    ("&bull;", "•"),
    ("&deg;", "°"),
    ("&plusmn;", "±"),
    ("&times;", "×"),
    ("&divide;", "÷"),
    ("&euro;", "€"),
    ("&pound;", "£"),
    ("&yen;", "¥"),
    ("&sect;", "§"),
    ("&para;", "¶"),
    ("&dagger;", "†"),
    ("&Dagger;", "‡"),
    ("&permil;", "‰"),
    ("&prime;", "′"),
    ("&Prime;", "″"),
    ("&larr;", "←"),
    ("&uarr;", "↑"),
    ("&rarr;", "→"),
    ("&darr;", "↓"),
    ("&harr;", "↔"),
    ("&crarr;", "↵"),
];

/// Decode `&#NNN;` and `&#xHEX;` numeric character references in place.
/// Unrecognized refs (out-of-range, bad digits, missing semicolon) pass
/// through verbatim. Operates on byte indices but only branches on the
/// ASCII `&#…;` pattern so multi-byte UTF-8 is forwarded safely.
fn decode_numeric_refs(s: &str) -> String {
    if !s.contains("&#") {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' && i + 1 < bytes.len() && bytes[i + 1] == b'#' {
            let body_start = i + 2;
            let mut end = body_start;
            while end < bytes.len() && end - body_start < 8 && bytes[end] != b';' {
                end += 1;
            }
            if end < bytes.len() && bytes[end] == b';' {
                let body = &s[body_start..end];
                let codepoint = if let Some(hex) = body.strip_prefix(['x', 'X']) {
                    u32::from_str_radix(hex, 16).ok()
                } else {
                    body.parse::<u32>().ok()
                };
                if let Some(cp) = codepoint
                    && let Some(c) = char::from_u32(cp)
                {
                    out.push(c);
                    i = end + 1;
                    continue;
                }
            }
        }
        // Forward one UTF-8 character at a time. `s.is_char_boundary(i)`
        // is true whenever `i` sits at the start of a UTF-8 sequence
        // (or at end-of-string).
        let next_boundary = (i + 1..=bytes.len())
            .find(|&j| s.is_char_boundary(j))
            .unwrap_or(bytes.len());
        out.push_str(&s[i..next_boundary]);
        i = next_boundary;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_element_and_text() {
        let nodes = parse_fragment("<h1>Hi</h1>").unwrap();
        assert_eq!(nodes.len(), 1);
        let Node::Element(el) = &nodes[0] else {
            panic!()
        };
        assert_eq!(el.tag, "h1");
        assert_eq!(el.children.len(), 1);
    }

    #[test]
    fn parses_attrs_quoted_and_brace() {
        let nodes = parse_fragment(r#"<x a="1" b="{count()}" c />"#).unwrap();
        let Node::Element(el) = &nodes[0] else {
            panic!()
        };
        assert_eq!(el.attributes.len(), 3);
        assert_eq!(el.get_attribute("a"), Some("1".into()));
        assert_eq!(el.get_attribute("b"), Some("{count()}".into()));
        assert!(el.has_attribute("c"));
    }

    #[test]
    fn parses_self_closing_void_tag() {
        let nodes = parse_fragment("<br/>after").unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn diagnoses_mismatched_close() {
        let err = parse_fragment("<a></b>").unwrap_err();
        assert!(err.message.contains("</b>"));
    }

    #[test]
    fn comment_round_trip() {
        let nodes = parse_fragment("<!-- hi --> ").unwrap();
        assert!(matches!(nodes[0], Node::Comment(_)));
    }
}
