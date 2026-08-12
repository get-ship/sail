//! Token stream for template grammar processing.
//!
//! Walks element children and splits text nodes at brace markers.
//! Element children pass through as `Element` tokens for the
//! recursive compile pass. Brace arms (`{pattern as binding => body}`)
//! become `Arm` tokens whose body is re-parsed as a template fragment.

use crate::diagnostics::Span;
use crate::parse::html::{self, Node};

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    Text(String),
    Element(html::Element),
    Expr {
        src: String,
    },
    /// `{pattern as binding => body}` or `{else => body}` / `{else-if {c} => body}`.
    Arm {
        head: String,
        body_src: String,
    },
    OpenIf {
        src: String,
    },
    OpenFor {
        src: String,
    },
    OpenMatch {
        src: String,
    },
    OpenWhen {
        src: String,
    },
    CloseIf,
    CloseFor,
    CloseMatch,
    CloseWhen,
}

/// Lightweight discriminant used by the consumer's terminator list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Discriminant {
    Text,
    Element,
    Expr,
    Arm,
    OpenIf,
    OpenFor,
    OpenMatch,
    OpenWhen,
    CloseIf,
    CloseFor,
    CloseMatch,
    CloseWhen,
}

impl TokenKind {
    pub fn discriminant(&self) -> Discriminant {
        match self {
            TokenKind::Text(_) => Discriminant::Text,
            TokenKind::Element(_) => Discriminant::Element,
            TokenKind::Expr { .. } => Discriminant::Expr,
            TokenKind::Arm { .. } => Discriminant::Arm,
            TokenKind::OpenIf { .. } => Discriminant::OpenIf,
            TokenKind::OpenFor { .. } => Discriminant::OpenFor,
            TokenKind::OpenMatch { .. } => Discriminant::OpenMatch,
            TokenKind::OpenWhen { .. } => Discriminant::OpenWhen,
            TokenKind::CloseIf => Discriminant::CloseIf,
            TokenKind::CloseFor => Discriminant::CloseFor,
            TokenKind::CloseMatch => Discriminant::CloseMatch,
            TokenKind::CloseWhen => Discriminant::CloseWhen,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.discriminant() {
            Discriminant::Text => "text",
            Discriminant::Element => "element",
            Discriminant::Expr => "{expr}",
            Discriminant::Arm => "{arm => …}",
            Discriminant::OpenIf => "<if>",
            Discriminant::OpenFor => "<for>",
            Discriminant::OpenMatch => "<match>",
            Discriminant::OpenWhen => "<when>",
            Discriminant::CloseIf => "</if>",
            Discriminant::CloseFor => "</for>",
            Discriminant::CloseMatch => "</match>",
            Discriminant::CloseWhen => "</when>",
        }
    }
}

pub fn tokenize_children(nodes: &[Node]) -> Vec<Token> {
    let has_element_sibling = nodes.iter().any(|n| matches!(n, Node::Element(_)));
    let mut out = Vec::new();
    for n in nodes {
        match n {
            Node::Text(t) => {
                // Indentation/newline whitespace between elements is invisible in
                // light DOM. Drop it when the parent has element children; keep it
                // when all siblings are text (e.g. <span> </span>) so intentional
                // inline whitespace is preserved.
                if has_element_sibling && t.data.chars().all(|c| c.is_ascii_whitespace()) {
                    continue;
                }
                let before = out.len();
                tokenize_text(&t.data, t.span, &mut out);
                // Strip indentation whitespace around control markers / arms.
                let has_non_text = out[before..]
                    .iter()
                    .any(|tok| !matches!(tok.kind, TokenKind::Text(_)));
                if has_non_text {
                    let slice: Vec<Token> = out.drain(before..).collect();
                    for tok in slice {
                        if let TokenKind::Text(ref s) = tok.kind
                            && s.chars().all(|c| c.is_ascii_whitespace())
                        {
                            continue;
                        }
                        out.push(tok);
                    }
                }
            }
            Node::Comment(_) => {}
            Node::Element(e) if super::control_elements::is_control_tag(&e.tag) => {
                super::control_elements::expand_control_element(e, &mut out);
            }
            Node::Element(e) => out.push(Token {
                kind: TokenKind::Element(e.clone()),
                span: e.span,
            }),
        }
    }
    out
}

/// Public for control-element expansion of nested text.
pub(super) fn tokenize_text_public(text: &str, base_span: Span, out: &mut Vec<Token>) {
    tokenize_text(text, base_span, out);
}

fn tokenize_text(text: &str, base_span: Span, out: &mut Vec<Token>) {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut buf = String::new();
    let mut buf_start = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{'
            && let Some(close) = find_matching_close(text, i)
        {
            let inner = text[i + 1..close].trim().to_string();
            if !buf.is_empty() {
                let span = Span::new(base_span.start + buf_start, base_span.start + i);
                let decoded = super::super::html::decode_entities(&buf);
                buf.clear();
                out.push(Token {
                    kind: TokenKind::Text(decoded),
                    span,
                });
            }
            let span = Span::new(base_span.start + i, base_span.start + close + 1);
            out.push(Token {
                kind: classify_brace(&inner),
                span,
            });
            i = close + 1;
            buf_start = i;
            continue;
        }
        // Append a single UTF-8 char.
        let mut j = i + 1;
        while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
            j += 1;
        }
        buf.push_str(&text[i..j]);
        i = j;
    }
    if !buf.is_empty() {
        let span = Span::new(base_span.start + buf_start, base_span.start + text.len());
        let decoded = super::super::html::decode_entities(&buf);
        out.push(Token {
            kind: TokenKind::Text(decoded),
            span,
        });
    }
}

fn find_matching_close(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    let mut i = start;
    while i < bytes.len() {
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
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn classify_brace(inner: &str) -> TokenKind {
    // Retired brace-block forms — never Open* tokens. Expression validator
    // rejects the sentinel with a normal unknown-call error (DEEMBALM: no
    // permanent tombstone pointer).
    if inner.starts_with('#')
        || inner.starts_with(':')
        || inner.starts_with('/')
        || inner.starts_with("while-")
    {
        return TokenKind::Expr {
            src: format!("__sail_unknown_block__({inner:?})"),
        };
    }
    // Arm: `{head => body}` — top-level `=>` (not inside nested braces/parens).
    if let Some((head, body)) = split_arm(inner) {
        return TokenKind::Arm {
            head: head.to_string(),
            body_src: body.to_string(),
        };
    }
    TokenKind::Expr {
        src: inner.to_string(),
    }
}

/// Split `head => body` on the first top-level `=>`.
fn split_arm(inner: &str) -> Option<(&str, &str)> {
    let bytes = inner.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    let mut i = 0usize;
    while i + 1 < bytes.len() {
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
        if depth == 0 && c == b'=' && bytes[i + 1] == b'>' {
            let head = inner[..i].trim();
            let body = inner[i + 2..].trim();
            if head.is_empty() {
                return None;
            }
            return Some((head, body));
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_text_at_brace() {
        let mut out = Vec::new();
        tokenize_text("hi {x()} there", Span::new(0, 14), &mut out);
        assert_eq!(out.len(), 3);
        assert!(matches!(out[1].kind, TokenKind::Expr { .. }));
    }

    #[test]
    fn brace_blocks_are_not_open_if() {
        let mut out = Vec::new();
        tokenize_text("{#if a}b{/if}", Span::new(0, 13), &mut out);
        assert!(matches!(&out[0].kind, TokenKind::Expr { src } if src.contains("unknown_block")));
    }

    #[test]
    fn classifies_arm_with_binding() {
        let mut out = Vec::new();
        tokenize_text(
            "{ok as names => <p>{names}</p>}",
            Span::new(0, 31),
            &mut out,
        );
        match &out[0].kind {
            TokenKind::Arm { head, body_src } => {
                assert_eq!(head, "ok as names");
                assert!(body_src.contains("<p>"), "body={body_src}");
            }
            other => panic!("expected Arm, got {other:?}"),
        }
    }

    #[test]
    fn classifies_else_arm() {
        let mut out = Vec::new();
        tokenize_text("{else => <p>n</p>}", Span::new(0, 18), &mut out);
        match &out[0].kind {
            TokenKind::Arm { head, .. } => assert_eq!(head, "else"),
            other => panic!("expected Arm, got {other:?}"),
        }
    }
}
