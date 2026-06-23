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
    OpenAsync { src: String },
    AsyncLoading,
    AsyncError { src: String },
    AsyncOk,
    ElseIf { src: String },
    Else,
    Case { src: String },
    Default,
    CloseIf,
    CloseFor,
    CloseMatch,
    CloseWhileLoading,
    CloseWhileError,
    CloseAsync,
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
    OpenAsync,
    AsyncLoading,
    AsyncError,
    AsyncOk,
    ElseIf,
    Else,
    Case,
    Default,
    CloseIf,
    CloseFor,
    CloseMatch,
    CloseWhileLoading,
    CloseWhileError,
    CloseAsync,
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
            TokenKind::OpenAsync { .. } => Discriminant::OpenAsync,
            TokenKind::AsyncLoading => Discriminant::AsyncLoading,
            TokenKind::AsyncError { .. } => Discriminant::AsyncError,
            TokenKind::AsyncOk => Discriminant::AsyncOk,
            TokenKind::ElseIf { .. } => Discriminant::ElseIf,
            TokenKind::Else => Discriminant::Else,
            TokenKind::Case { .. } => Discriminant::Case,
            TokenKind::Default => Discriminant::Default,
            TokenKind::CloseIf => Discriminant::CloseIf,
            TokenKind::CloseFor => Discriminant::CloseFor,
            TokenKind::CloseMatch => Discriminant::CloseMatch,
            TokenKind::CloseWhileLoading => Discriminant::CloseWhileLoading,
            TokenKind::CloseWhileError => Discriminant::CloseWhileError,
            TokenKind::CloseAsync => Discriminant::CloseAsync,
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
            Discriminant::OpenAsync => "{#async}",
            Discriminant::AsyncLoading => "{:loading}",
            Discriminant::AsyncError => "{:error}",
            Discriminant::AsyncOk => "{:ok}",
            Discriminant::ElseIf => "{:else if}",
            Discriminant::Else => "{:else}",
            Discriminant::Case => "{:case}",
            Discriminant::Default => "{:default}",
            Discriminant::CloseIf => "{/if}",
            Discriminant::CloseFor => "{/for}",
            Discriminant::CloseMatch => "{/match}",
            Discriminant::CloseWhileLoading => "{/while-loading}",
            Discriminant::CloseWhileError => "{/while-error}",
            Discriminant::CloseAsync => "{/async}",
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
                // Shadow DOM. Drop it when the parent has element children; keep it
                // when all siblings are text (e.g. <span> </span>) so intentional
                // inline whitespace is preserved.
                if has_element_sibling && t.data.chars().all(|c| c.is_ascii_whitespace()) {
                    continue;
                }
                let before = out.len();
                tokenize_text(&t.data, t.span, &mut out);
                // If this text node produced control-flow tokens (e.g. the raw source
                // is "\n  {#if ...}\n    "), strip the surrounding whitespace-only
                // text sub-tokens — they're structural indentation around the markers,
                // not content.
                let has_control = out[before..].iter().any(|tok| {
                    !matches!(tok.kind, TokenKind::Text(_))
                });
                if has_control {
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
            Node::Element(e) => out.push(Token {
                kind: TokenKind::Element(e.clone()),
                span: e.span,
            }),
        }
    }
    out
}

fn tokenize_text(text: &str, base_span: Span, out: &mut Vec<Token>) {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut buf = String::new();
    let mut buf_start = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{'
            && let Some(close) = find_matching_close(text, i) {
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
                    out.push(Token { kind: TokenKind::Text(decoded), span });
                }
                let span = Span::new(base_span.start + i, base_span.start + close + 1);
                out.push(Token { kind: classify_brace(&inner), span });
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
        out.push(Token { kind: TokenKind::Text(decoded), span });
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
    if let Some(rest) = inner.strip_prefix("#if") {
        return TokenKind::OpenIf { src: rest.trim().to_string() };
    }
    if let Some(rest) = inner.strip_prefix("#for") {
        return TokenKind::OpenFor { src: rest.trim().to_string() };
    }
    if let Some(rest) = inner.strip_prefix("#match") {
        return TokenKind::OpenMatch { src: rest.trim().to_string() };
    }
    if let Some(rest) = inner.strip_prefix("#while-loading") {
        return TokenKind::OpenWhileLoading { src: rest.trim().to_string() };
    }
    if let Some(rest) = inner.strip_prefix("#while-error") {
        return TokenKind::OpenWhileError { src: rest.trim().to_string() };
    }
    if let Some(rest) = inner.strip_prefix("#async") {
        return TokenKind::OpenAsync { src: rest.trim().to_string() };
    }
    if inner == ":loading" {
        return TokenKind::AsyncLoading;
    }
    if let Some(rest) = inner.strip_prefix(":error") {
        return TokenKind::AsyncError { src: rest.trim().to_string() };
    }
    if inner == ":ok" {
        return TokenKind::AsyncOk;
    }
    if let Some(rest) = inner.strip_prefix(":else if") {
        return TokenKind::ElseIf { src: rest.trim().to_string() };
    }
    if inner == ":else" {
        return TokenKind::Else;
    }
    if let Some(rest) = inner.strip_prefix(":case") {
        return TokenKind::Case { src: rest.trim().to_string() };
    }
    if inner == ":default" {
        return TokenKind::Default;
    }
    if inner == "/if" {
        return TokenKind::CloseIf;
    }
    if inner == "/for" {
        return TokenKind::CloseFor;
    }
    if inner == "/match" {
        return TokenKind::CloseMatch;
    }
    if inner == "/while-loading" {
        return TokenKind::CloseWhileLoading;
    }
    if inner == "/while-error" {
        return TokenKind::CloseWhileError;
    }
    if inner == "/async" {
        return TokenKind::CloseAsync;
    }
    TokenKind::Expr { src: inner.to_string() }
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
    fn classify_block_markers() {
        let mut out = Vec::new();
        tokenize_text("{#if a}b{/if}", Span::new(0, 13), &mut out);
        assert!(matches!(out[0].kind, TokenKind::OpenIf { .. }));
        assert!(matches!(out[2].kind, TokenKind::CloseIf));
    }
}
