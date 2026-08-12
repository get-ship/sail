//! Expand reserved control-flow **elements** into the flat token stream
//! used by `blocks.rs`. Author form:
//!
//! ```sail
//! <if {open()}>…{else-if {x()} => …}{else => …}</if>
//! <for {items as item} key={item.id}>…{else => …}</for>
//! <match {state()}>
//!   {loading => …}
//!   {error as err => …}
//!   {ok as value => …}
//! </match>
//! <when {tablet}>…{else => …}</when>
//! ```
//!
//! Four blocks only: if / for / match / when. Binding lives in-brace
//! (`items as item`, `error as err`); `as=` attributes and sibling
//! `<else>` / `<case>` / `<default>` / `<while-*>` elements are gone.

use crate::parse::html::{Element, Node, PRIMARY_BRACE_ATTR};

use super::tokenize::{Token, TokenKind};

/// Tags that are control-flow, never HTML elements.
pub fn is_control_tag(tag: &str) -> bool {
    matches!(tag, "if" | "for" | "match" | "when")
}

/// Expand a control element into Open/Close tokens. Nested content
/// (including arm braces) is flattened via `flatten_nodes`.
pub fn expand_control_element(el: &Element, out: &mut Vec<Token>) {
    match el.tag.as_str() {
        "if" => expand_if(el, out),
        "for" => expand_for(el, out),
        "match" => expand_match(el, out),
        "when" => expand_when(el, out),
        _ => unreachable!("is_control_tag guard"),
    }
}

fn expand_if(el: &Element, out: &mut Vec<Token>) {
    let cond = brace_primary(el).unwrap_or_default();
    out.push(Token {
        kind: TokenKind::OpenIf { src: cond },
        span: el.span,
    });
    flatten_nodes(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseIf,
        span: el.span,
    });
}

fn expand_for(el: &Element, out: &mut Vec<Token>) {
    // Header form understood by parse_for_header:
    //   `item of items()` / `(item, i) of items() key={…}`
    // Author form is brace-primary `items as item` (+ optional `, i`) with
    // optional `key=` attr. Legacy `as=` is rejected at blocks parse.
    let primary = brace_primary(el).unwrap_or_default();
    let key = attr_expr(el, "key");
    let header = build_for_header(&primary, key.as_deref());

    out.push(Token {
        kind: TokenKind::OpenFor { src: header },
        span: el.span,
    });
    flatten_nodes(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseFor,
        span: el.span,
    });
}

/// Convert author `{iterable as item[, idx]}` (+ optional key attr) into the
/// legacy `item of iterable key={…}` header string.
fn build_for_header(primary: &str, key: Option<&str>) -> String {
    let primary = primary.trim();
    let (iterable, binding) = split_as_binding(primary).unwrap_or((primary, "item"));
    let left = binding.trim();
    let mut header = if left.starts_with('(') {
        format!("{left} of {iterable}")
    } else if let Some((item, idx)) = split_top_comma(left) {
        format!("({item}, {idx}) of {iterable}")
    } else {
        format!("{left} of {iterable}")
    };
    if let Some(k) = key {
        if !k.is_empty() {
            header.push_str(" key={");
            header.push_str(k);
            header.push('}');
        }
    }
    header
}

/// Split `iterable as binding` on top-level ` as `.
///
/// Accepts a bare primary (`markers() as m`) or a still-braced group
/// (`{markers() as m}`) — `brace_primary` normally strips braces, but
/// defense-in-depth here keeps a residual brace group from collapsing
/// into the default `item` binder (which leaves `as`/`m` free idents).
fn split_as_binding(src: &str) -> Option<(&str, &str)> {
    let src = {
        let s = src.trim();
        if s.starts_with('{') && s.ends_with('}') && s.len() >= 2 {
            s[1..s.len() - 1].trim()
        } else {
            s
        }
    };
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    let mut i = 0usize;
    while i + 4 <= bytes.len() {
        // Stay on UTF-8 char boundaries — multi-byte glyphs must not be
        // sliced mid-codepoint when probing for ` as `.
        if !src.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let c = bytes[i];
        if let Some(q) = str_quote {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                str_quote = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            str_quote = Some(c);
            i += 1;
            continue;
        }
        if matches!(c, b'(' | b'[' | b'{') {
            depth += 1;
            i += 1;
            continue;
        }
        if matches!(c, b')' | b']' | b'}') {
            depth -= 1;
            i += 1;
            continue;
        }
        if depth == 0 && src.is_char_boundary(i + 4) && &src[i..i + 4] == " as " {
            let left = src[..i].trim();
            let right = src[i + 4..].trim();
            if !left.is_empty() && !right.is_empty() {
                return Some((left, right));
            }
            return None;
        }
        i += 1;
    }
    None
}

fn split_top_comma(src: &str) -> Option<(&str, &str)> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    for (i, &c) in bytes.iter().enumerate() {
        if let Some(q) = str_quote {
            if c == q {
                str_quote = None;
            }
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            str_quote = Some(c);
            continue;
        }
        if matches!(c, b'(' | b'[' | b'{') {
            depth += 1;
        } else if matches!(c, b')' | b']' | b'}') {
            depth -= 1;
        } else if c == b',' && depth == 0 {
            let a = src[..i].trim();
            let b = src[i + 1..].trim();
            if !a.is_empty() && !b.is_empty() {
                return Some((a, b));
            }
            return None;
        }
    }
    None
}

fn expand_match(el: &Element, out: &mut Vec<Token>) {
    let disc = brace_primary(el).unwrap_or_default();
    out.push(Token {
        kind: TokenKind::OpenMatch { src: disc },
        span: el.span,
    });
    flatten_nodes(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseMatch,
        span: el.span,
    });
}

fn expand_when(el: &Element, out: &mut Vec<Token>) {
    let token = brace_primary(el).unwrap_or_default();
    out.push(Token {
        kind: TokenKind::OpenWhen { src: token },
        span: el.span,
    });
    flatten_nodes(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseWhen,
        span: el.span,
    });
}

fn flatten_nodes(nodes: &[Node], out: &mut Vec<Token>) {
    let has_element_sibling = nodes.iter().any(|n| matches!(n, Node::Element(_)));
    for n in nodes {
        match n {
            Node::Text(t)
                if has_element_sibling && t.data.chars().all(|c| c.is_ascii_whitespace()) => {}
            other => flatten_node(other, out),
        }
    }
}

fn flatten_node(n: &Node, out: &mut Vec<Token>) {
    match n {
        Node::Text(t) => {
            super::tokenize::tokenize_text_public(&t.data, t.span, out);
        }
        Node::Comment(_) => {}
        Node::Element(e) if is_control_tag(&e.tag) => {
            expand_control_element(e, out);
        }
        Node::Element(e) => {
            out.push(Token {
                kind: TokenKind::Element(e.clone()),
                span: e.span,
            });
        }
    }
}

/// Expression attribute: `key={item.id}` → `item.id` after preprocess quoting.
fn attr_expr(el: &Element, name: &str) -> Option<String> {
    let raw = el.get_attribute(name)?;
    Some(strip_attr_expr(&raw).to_string())
}

/// Brace-primary `{expr}` on the open tag — the only primary-expression form.
fn brace_primary(el: &Element) -> Option<String> {
    let raw = el.get_attribute(PRIMARY_BRACE_ATTR)?;
    let inner = strip_attr_expr(&raw).to_string();
    if inner.is_empty() { None } else { Some(inner) }
}

fn strip_attr_expr(raw: &str) -> &str {
    let s = raw.trim();
    if s.starts_with('{') && s.ends_with('}') && s.len() >= 2 {
        return s[1..s.len() - 1].trim();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::html;

    fn parse_el(src: &str) -> Element {
        let nodes = html::parse_fragment(src).expect("parse");
        match nodes.into_iter().next().unwrap() {
            Node::Element(e) => e,
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn expands_if_with_else_arm() {
        let el = parse_el("<if {a()}><p>y</p>{else => <p>n</p>}</if>");
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        assert!(matches!(out[0].kind, TokenKind::OpenIf { .. }));
        assert!(out.iter().any(|t| matches!(t.kind, TokenKind::Arm { .. })));
        assert!(matches!(out.last().unwrap().kind, TokenKind::CloseIf));
    }

    #[test]
    fn expands_for_in_brace_binding() {
        let el = parse_el(r#"<for {items() as row} key={row.id}><p>{row.id}</p></for>"#);
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        match &out[0].kind {
            TokenKind::OpenFor { src } => {
                assert!(src.contains("row of items()"), "got {src}");
                assert!(src.contains("key={row.id}"), "got {src}");
            }
            other => panic!("expected OpenFor, got {other:?}"),
        }
    }

    #[test]
    fn expands_annotation_layer_for_as_m() {
        // Exact pack shape that failed under pre-blocks-1 binaries:
        // `Identifier 'as' is not declared` when the primary was left as
        // the iterable expression instead of `m of markers()`.
        let el = parse_el(r#"<for {markers() as m} key={m.id}><span/></for>"#);
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        match &out[0].kind {
            TokenKind::OpenFor { src } => {
                assert_eq!(
                    src, "m of markers() key={m.id}",
                    "for-as must convert to legacy header, got {src}"
                );
            }
            other => panic!("expected OpenFor, got {other:?}"),
        }
    }

    #[test]
    fn split_as_binding_strips_residual_braces() {
        let (it, b) = super::split_as_binding("{markers() as m}").expect("split");
        assert_eq!(it, "markers()");
        assert_eq!(b, "m");
    }

    #[test]
    fn expands_match_arms() {
        let el = parse_el(
            "<match {kind()}>{ok as v => <p>y</p>}{error as e => <p>n</p>}{loading => <p>…</p>}</match>",
        );
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        match &out[0].kind {
            TokenKind::OpenMatch { src } => assert_eq!(src, "kind()"),
            other => panic!("expected OpenMatch, got {other:?}"),
        }
        let arms: Vec<_> = out
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Arm { .. }))
            .collect();
        assert_eq!(arms.len(), 3, "three arms: {out:?}");
    }

    #[test]
    fn residual_cond_named_attr_is_ignored() {
        let el = parse_el("<if cond={a()}><p>y</p></if>");
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        match &out[0].kind {
            TokenKind::OpenIf { src } => {
                assert!(src.is_empty(), "cond= must not supply primary, got {src}")
            }
            other => panic!("expected OpenIf, got {other:?}"),
        }
    }

    #[test]
    fn expands_when_multiline_else_arm() {
        let el = parse_el(
            "<when {tablet}><p>wide</p>{else =>\n  <if {open()}><span>n</span></if>\n}</when>",
        );
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        eprintln!(
            "tokens: {:?}",
            out.iter().map(|t| t.kind.label()).collect::<Vec<_>>()
        );
        for t in &out {
            if let TokenKind::Arm { head, body_src } = &t.kind {
                eprintln!("ARM head={head:?} body={body_src:?}");
            }
        }
        assert!(out.iter().any(|t| matches!(t.kind, TokenKind::Arm { .. })));
        assert!(matches!(out.last().unwrap().kind, TokenKind::CloseWhen));
    }
}
