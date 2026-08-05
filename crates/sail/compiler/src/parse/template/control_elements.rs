//! Expand reserved control-flow **elements** into the flat token stream
//! used by `blocks.rs`. Author form (product + residual named attrs):
//!
//! ```sail
//! <if {open()}>…<else-if {x()}>…</else-if><else>…</else></if>
//! <for {items()} as="item" key={item.id}>…<else>…</else></for>
//! <match {kind()}>
//!   <case {'a'}>…</case>
//!   <default>…</default>
//! </match>
//! <while-loading {users}>…</while-loading>
//! <while-error as="err" of="users">…</while-error>
//! ```
//!
//! Named attrs still work: `cond=`, `on=`/`value=`, `of=`, `as=`, `key=`.
//! Brace-primary (`{expr}` right after the tag) is preferred.
//!
//! Brace blocks (`{#if}`, …) are rejected at classify time.

use crate::parse::html::{Element, Node, PRIMARY_BRACE_ATTR};

use super::tokenize::{Token, TokenKind};

/// Tags that are control-flow, never HTML elements.
pub fn is_control_tag(tag: &str) -> bool {
    matches!(
        tag,
        "if" | "else-if"
            | "else"
            | "for"
            | "match"
            | "when"
            | "case"
            | "default"
            | "while-loading"
            | "while-error"
    )
}

/// Expand a control element into Open/branch/Close tokens. Recurses for nested
/// control and normal elements via `flatten_nodes`.
pub fn expand_control_element(el: &Element, out: &mut Vec<Token>) {
    match el.tag.as_str() {
        "if" => expand_if(el, out),
        "for" => expand_for(el, out),
        "match" => expand_match(el, out),
        "when" => expand_when(el, out),
        "while-loading" => expand_while_loading(el, out),
        "while-error" => expand_while_error(el, out),
        // Stray clause tags at this level become errors later as unexpected tokens
        // if someone writes <else> alone — emit as markers with empty/malformed src.
        "else-if" => {
            out.push(Token {
                kind: TokenKind::ElseIf {
                    src: primary_or_named(el, &["cond"]).unwrap_or_default(),
                },
                span: el.span,
            });
            flatten_nodes(&el.children, out);
        }
        "else" => {
            out.push(Token {
                kind: TokenKind::Else,
                span: el.span,
            });
            flatten_nodes(&el.children, out);
        }
        "case" => {
            out.push(Token {
                kind: TokenKind::Case {
                    src: primary_or_named(el, &["of"]).unwrap_or_default(),
                },
                span: el.span,
            });
            flatten_nodes(&el.children, out);
        }
        "default" => {
            out.push(Token {
                kind: TokenKind::Default,
                span: el.span,
            });
            flatten_nodes(&el.children, out);
        }
        _ => unreachable!("is_control_tag guard"),
    }
}

fn expand_if(el: &Element, out: &mut Vec<Token>) {
    let cond = primary_or_named(el, &["cond"]).unwrap_or_default();
    out.push(Token {
        kind: TokenKind::OpenIf { src: cond },
        span: el.span,
    });
    flatten_if_children(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseIf,
        span: el.span,
    });
}

/// Body of `<if>`: normal nodes, then optional `<else-if>` / `<else>` elements
/// as **direct** children (not nested inside other elements).
fn flatten_if_children(nodes: &[Node], out: &mut Vec<Token>) {
    for n in nodes {
        match n {
            Node::Element(child) if child.tag == "else-if" => {
                out.push(Token {
                    kind: TokenKind::ElseIf {
                        src: primary_or_named(child, &["cond"]).unwrap_or_default(),
                    },
                    span: child.span,
                });
                flatten_nodes(&child.children, out);
            }
            Node::Element(child) if child.tag == "else" => {
                out.push(Token {
                    kind: TokenKind::Else,
                    span: child.span,
                });
                flatten_nodes(&child.children, out);
            }
            other => flatten_node(other, out),
        }
    }
}

fn expand_for(el: &Element, out: &mut Vec<Token>) {
    // Build the legacy header string parse_for_header already understands:
    //   `item of items()` or `(item, i) of items() key={…}`
    let as_name = attr_raw(el, "as").unwrap_or_else(|| "item".into());
    let index = attr_raw(el, "index");
    let iterable = primary_or_named(el, &["of"]).unwrap_or_default();
    let key = attr_expr(el, "key");

    let left = match index {
        Some(i) if !i.is_empty() => format!("({as_name}, {i})"),
        _ => as_name,
    };
    let mut header = format!("{left} of {iterable}");
    if let Some(k) = key {
        if !k.is_empty() {
            header.push_str(" key={");
            header.push_str(&k);
            header.push('}');
        }
    }

    out.push(Token {
        kind: TokenKind::OpenFor { src: header },
        span: el.span,
    });
    flatten_for_children(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseFor,
        span: el.span,
    });
}

fn flatten_for_children(nodes: &[Node], out: &mut Vec<Token>) {
    for n in nodes {
        match n {
            Node::Element(child) if child.tag == "else" => {
                out.push(Token {
                    kind: TokenKind::Else,
                    span: child.span,
                });
                flatten_nodes(&child.children, out);
            }
            other => flatten_node(other, out),
        }
    }
}

fn expand_match(el: &Element, out: &mut Vec<Token>) {
    let disc = primary_or_named(el, &["on", "value"]).unwrap_or_default();
    out.push(Token {
        kind: TokenKind::OpenMatch { src: disc },
        span: el.span,
    });
    for n in &el.children {
        match n {
            Node::Element(child) if child.tag == "case" => {
                out.push(Token {
                    kind: TokenKind::Case {
                        src: primary_or_named(child, &["of"]).unwrap_or_default(),
                    },
                    span: child.span,
                });
                flatten_nodes(&child.children, out);
            }
            Node::Element(child) if child.tag == "default" => {
                out.push(Token {
                    kind: TokenKind::Default,
                    span: child.span,
                });
                flatten_nodes(&child.children, out);
            }
            Node::Text(t) if t.data.chars().all(|c| c.is_ascii_whitespace()) => {}
            other => flatten_node(other, out),
        }
    }
    out.push(Token {
        kind: TokenKind::CloseMatch,
        span: el.span,
    });
}

fn expand_when(el: &Element, out: &mut Vec<Token>) {
    // Token is a theme breakpoint name, not an expression — same brace-primary
    // surface as other blocks (`<when {tablet}>`).
    let token = primary_or_named(el, &["bp", "breakpoint"]).unwrap_or_default();
    out.push(Token {
        kind: TokenKind::OpenWhen { src: token },
        span: el.span,
    });
    flatten_when_children(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseWhen,
        span: el.span,
    });
}

fn flatten_when_children(nodes: &[Node], out: &mut Vec<Token>) {
    for n in nodes {
        match n {
            Node::Element(child) if child.tag == "else" => {
                out.push(Token {
                    kind: TokenKind::Else,
                    span: child.span,
                });
                flatten_nodes(&child.children, out);
            }
            other => flatten_node(other, out),
        }
    }
}

fn expand_while_loading(el: &Element, out: &mut Vec<Token>) {
    // Header: `of <signalName>` — signal is a bare identifier.
    // Prefer brace-primary `<while-loading {users}>`, then `of=`.
    let name = primary_or_named(el, &["of"])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| attr_ident(el, "of"))
        .unwrap_or_default();
    out.push(Token {
        kind: TokenKind::OpenWhileLoading {
            src: format!("of {name}"),
        },
        span: el.span,
    });
    flatten_nodes(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseWhileLoading,
        span: el.span,
    });
}

fn expand_while_error(el: &Element, out: &mut Vec<Token>) {
    // Header: `<errVar> of <signalName>`
    let err = attr_raw(el, "as").unwrap_or_else(|| "err".into());
    let name = primary_or_named(el, &["of"])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| attr_ident(el, "of"))
        .unwrap_or_default();
    out.push(Token {
        kind: TokenKind::OpenWhileError {
            src: format!("{err} of {name}"),
        },
        span: el.span,
    });
    flatten_nodes(&el.children, out);
    out.push(Token {
        kind: TokenKind::CloseWhileError,
        span: el.span,
    });
}

fn flatten_nodes(nodes: &[Node], out: &mut Vec<Token>) {
    // Re-enter the same whitespace rules as tokenize_children for this sublist.
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

/// Expression attribute: `cond={open()}` → `open()` after preprocess quoting.
fn attr_expr(el: &Element, name: &str) -> Option<String> {
    let raw = el.get_attribute(name)?;
    Some(strip_attr_expr(&raw).to_string())
}

fn attr_raw(el: &Element, name: &str) -> Option<String> {
    el.get_attribute(name).map(|s| s.trim().to_string())
}

/// Identifier attribute (`of="users"` or `of={users}`).
fn attr_ident(el: &Element, name: &str) -> Option<String> {
    let s = attr_expr(el, name).or_else(|| attr_raw(el, name))?;
    let s = s.trim().to_string();
    if s.is_empty() {
        return None;
    }
    Some(s)
}

/// Product form first: brace-primary `{expr}` on the open tag, then named
/// attrs (`cond` / `on` / `of` / …) for residual compatibility.
fn primary_or_named(el: &Element, named: &[&str]) -> Option<String> {
    if let Some(raw) = el.get_attribute(PRIMARY_BRACE_ATTR) {
        let inner = strip_attr_expr(&raw).to_string();
        if !inner.is_empty() {
            return Some(inner);
        }
    }
    for name in named {
        if let Some(v) = attr_expr(el, name) {
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

fn strip_attr_expr(raw: &str) -> &str {
    let s = raw.trim();
    // preprocess may leave `"…"` quotes on the attr value already stripped by parser
    // into Attr.value without outer quotes. Values are often `{expr}` or plain.
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
    fn expands_if_else() {
        let el = parse_el("<if cond={a()}><p>y</p><else><p>n</p></else></if>");
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        assert!(matches!(out[0].kind, TokenKind::OpenIf { .. }));
        assert!(out.iter().any(|t| matches!(t.kind, TokenKind::Else)));
        assert!(matches!(out.last().unwrap().kind, TokenKind::CloseIf));
    }

    #[test]
    fn expands_if_brace_primary() {
        let el = parse_el(
            "<if {a()}><p>y</p><else-if {b()}><p>m</p></else-if><else><p>n</p></else></if>",
        );
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        match &out[0].kind {
            TokenKind::OpenIf { src } => assert_eq!(src, "a()"),
            other => panic!("expected OpenIf, got {other:?}"),
        }
        let else_if = out
            .iter()
            .find(|t| matches!(t.kind, TokenKind::ElseIf { .. }))
            .expect("else-if");
        match &else_if.kind {
            TokenKind::ElseIf { src } => assert_eq!(src, "b()"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn expands_match_case_brace_primary() {
        let el = parse_el(
            "<match {kind()}><case {'ok'}><p>y</p></case><default><p>n</p></default></match>",
        );
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        match &out[0].kind {
            TokenKind::OpenMatch { src } => assert_eq!(src, "kind()"),
            other => panic!("expected OpenMatch, got {other:?}"),
        }
        let case = out
            .iter()
            .find(|t| matches!(t.kind, TokenKind::Case { .. }))
            .expect("case");
        match &case.kind {
            TokenKind::Case { src } => assert_eq!(src, "'ok'"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn expands_for_brace_primary_iterable() {
        let el = parse_el(r#"<for {items()} as="row" key={row.id}><p>{row.id}</p></for>"#);
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
    fn brace_primary_with_spaces_inside() {
        let el = parse_el("<if { open() && ready() }><p>y</p></if>");
        let mut out = Vec::new();
        expand_control_element(&el, &mut out);
        match &out[0].kind {
            TokenKind::OpenIf { src } => assert_eq!(src, "open() && ready()"),
            other => panic!("expected OpenIf, got {other:?}"),
        }
    }
}
