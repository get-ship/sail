//! Token stream for template grammar processing.
//!
//! Walks element children and splits text nodes at brace markers.
//! Element children pass through as `Element` tokens for the
//! recursive compile pass.

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
    Expr { src: String },
    OpenIf { src: String },
    OpenFor { src: String },
    OpenMatch { src: String },
    OpenWhileLoading { src: String },
    OpenWhileError { src: String },
    ElseIf { src: String },
    Else,
    Case { src: String },
    Default,
    CloseIf,
    CloseFor,
    CloseMatch,
    CloseWhileLoading,
    CloseWhileError,
}

/// Lightweight discriminant used by the consumer's terminator list.
/// Two `Discriminant`s compare equal if they match the same variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Discriminant {
    Text,
    Element,
    Expr,
    OpenIf,
    OpenFor,
    OpenMatch,
    OpenWhileLoading,
    OpenWhileError,
    ElseIf,
    Else,
    Case,
    Default,
    CloseIf,
    CloseFor,
    CloseMatch,
    CloseWhileLoading,
    CloseWhileError,
}

impl TokenKind {
    pub fn discriminant(&self) -> Discriminant {
        match self {
            TokenKind::Text(_) => Discriminant::Text,
            TokenKind::Element(_) => Discriminant::Element,
            TokenKind::Expr { .. } => Discriminant::Expr,
            TokenKind::OpenIf { .. } => Discriminant::OpenIf,
            TokenKind::OpenFor { .. } => Discriminant::OpenFor,
            TokenKind::OpenMatch { .. } => Discriminant::OpenMatch,
            TokenKind::OpenWhileLoading { .. } => Discriminant::OpenWhileLoading,
            TokenKind::OpenWhileError { .. } => Discriminant::OpenWhileError,
            TokenKind::ElseIf { .. } => Discriminant::ElseIf,
            TokenKind::Else => Discriminant::Else,
            TokenKind::Case { .. } => Discriminant::Case,
            TokenKind::Default => Discriminant::Default,
            TokenKind::CloseIf => Discriminant::CloseIf,
            TokenKind::CloseFor => Discriminant::CloseFor,
            TokenKind::CloseMatch => Discriminant::CloseMatch,
            TokenKind::CloseWhileLoading => Discriminant::CloseWhileLoading,
            TokenKind::CloseWhileError => Discriminant::CloseWhileError,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.discriminant() {
            Discriminant::Text => "text",
            Discriminant::Element => "element",
            Discriminant::Expr => "{expr}",
            Discriminant::OpenIf => "{#if}",
            Discriminant::OpenFor => "{#for}",
            Discriminant::OpenMatch => "{#match}",
            Discriminant::OpenWhileLoading => "{#while-loading}",
            Discriminant::OpenWhileError => "{#while-error}",
            Discriminant::ElseIf => "{:else if}",
            Discriminant::Else => "{:else}",
            Discriminant::Case => "{:case}",
            Discriminant::Default => "{:default}",
            Discriminant::CloseIf => "{/if}",
            Discriminant::CloseFor => "{/for}",
            Discriminant::CloseMatch => "{/match}",
            Discriminant::CloseWhileLoading => "{/while-loading}",
            Discriminant::CloseWhileError => "{/while-error}",
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
                // Strip indentation whitespace around control markers / retired-block errors.
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
            // Expression body stays RAW — never entity-decoded. JS
            // source goes through the expression tokenizer / checker
            // which assumes the bytes match the source positions.
            let inner = text[i + 1..close].trim().to_string();
            if !buf.is_empty() {
                let span = Span::new(base_span.start + buf_start, base_span.start + i);
                // Text segment around the expression CAN be decoded —
                // it becomes a DOM text node and entity references like
                // `&deg;` are author shorthand for the resulting char.
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
    // Brace control-flow is retired. Emit a DynamicText expr that the
    // expression validator will reject with a migration message — or we
    // hard-error at Open* if anything still produces those tokens.
    // Prefer an immediate Expr with a sentinel the validate step catches.
    let retired = |kind: &str| -> TokenKind {
        TokenKind::Expr {
            src: format!(
                "__sail_retired_block__(\"{kind}\") /* use <{kind}>…</{kind}> elements */"
            ),
        }
    };
    if inner.starts_with("#if") {
        return retired("if");
    }
    if inner.starts_with("#for") {
        return retired("for");
    }
    if inner.starts_with("#match") {
        return retired("match");
    }
    if inner.starts_with("#while-loading") {
        return retired("while-loading");
    }
    if inner.starts_with("#while-error") {
        return retired("while-error");
    }
    if inner.starts_with(":else if")
        || inner == ":else"
        || inner.starts_with(":case")
        || inner == ":default"
        || inner == "/if"
        || inner == "/for"
        || inner == "/match"
        || inner == "/while-loading"
        || inner == "/while-error"
    {
        return TokenKind::Expr {
            src: "__sail_retired_block__(\"clause\") /* use <else-if>/<else>/<case>/<default> inside <if>/<for>/<match> */"
                .into(),
        };
    }
    TokenKind::Expr {
        src: inner.to_string(),
    }
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
    fn brace_blocks_are_retired_not_open_if() {
        let mut out = Vec::new();
        tokenize_text("{#if a}b{/if}", Span::new(0, 13), &mut out);
        assert!(matches!(&out[0].kind, TokenKind::Expr { src } if src.contains("retired")));
    }
}
