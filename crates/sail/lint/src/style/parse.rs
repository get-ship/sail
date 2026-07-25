//! Parser for the Sail Style expression grammar.
//!
//! Author forms:
//!   style.bg(primary).fg(onPrimary)
//!   match(variant, { filled: style…, tonal: style… })
//!   when({ hover: style…, focus: style… })
//!   compound({ variant: filled, size: lg }, style…)
//!   bind(rowY)
//!   filled
//!
//! The parser is the sole legality authority. No dual string-sx path.

use std::collections::HashMap;

use crate::sx_ir::SxValue;

use super::ir::{PaintOp, Style, StyleDiagnostic};

#[cfg(test)]
pub(crate) fn debug_tokens(input: &str) -> Vec<String> {
    tokenize(input).iter().map(|t| t.as_str().to_string()).collect()
}

/// Parse a Style expression string. Returns the Style and any diagnostics.
pub fn parse(input: &str) -> (Style, Vec<StyleDiagnostic>) {
    let mut diags = Vec::new();
    let style = parse_impl(input.trim(), &mut diags);
    (style, diags)
}

fn parse_impl(input: &str, diags: &mut Vec<StyleDiagnostic>) -> Style {
    if input.is_empty() {
        return Style::Empty;
    }
    let tokens = tokenize(input);
    let mut p = Parser::new(&tokens);
    let style = parse_style_expr(&mut p, diags);
    if !p.at_end() {
        diags.push(StyleDiagnostic::ParseError {
            message: format!("unexpected token `{}`", p.peek().unwrap_or("<")),
            source: p.remainder(),
        });
    }
    style
}

// ─── Tokenizer ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok<'a> {
    Ident(&'a str),
    Str(&'a str),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Dot,
    Eq,
    Arrow,
}

fn tokenize(input: &str) -> Vec<Tok<'_>> {
    let mut out = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '(' => out.push(Tok::LParen),
            ')' => out.push(Tok::RParen),
            '{' => out.push(Tok::LBrace),
            '}' => out.push(Tok::RBrace),
            '[' => out.push(Tok::LBracket),
            ']' => out.push(Tok::RBracket),
            ',' => out.push(Tok::Comma),
            ':' => out.push(Tok::Colon),
            '.' => out.push(Tok::Dot),
            '=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                    out.push(Tok::Arrow);
                    i += 1;
                } else {
                    out.push(Tok::Eq);
                }
            }
            '"' | '\'' => {
                let quote = c;
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && bytes[j] as char != quote {
                    j += 1;
                }
                out.push(Tok::Str(&input[start..j]));
                i = j;
            }
            _ if c.is_ascii_alphanumeric() || c == '_' || c == '$' => {
                let start = i;
                let mut j = i;
                while j < bytes.len() {
                    let b = bytes[j] as char;
                    if b.is_ascii_alphanumeric() || b == '-' || b == '_' || b == '$' {
                        j += 1;
                    } else {
                        break;
                    }
                }
                out.push(Tok::Ident(&input[start..j]));
                i = j;
                continue;
            }
            _ => {
                out.push(Tok::Ident(&input[i..i + 1]));
            }
        }
        i += 1;
    }
    out
}

struct Parser<'a> {
    tokens: &'a [Tok<'a>],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Tok<'a>]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&'a str> {
        self.tokens.get(self.pos).map(|t| t.as_str())
    }

    fn peek_tok(&self) -> Option<&Tok<'a>> {
        self.tokens.get(self.pos)
    }

    fn bump(&mut self) -> Option<Tok<'a>> {
        let t = self.tokens.get(self.pos).cloned()?;
        self.pos += 1;
        Some(t)
    }

    fn expect(&mut self, expected: Tok<'static>, diags: &mut Vec<StyleDiagnostic>) -> bool {
        match self.bump() {
            Some(t) if t == expected => true,
            other => {
                let got = other.map(|t| t.as_str().to_string()).unwrap_or_default();
                diags.push(StyleDiagnostic::ParseError {
                    message: format!("expected `{}`, found `{}`", expected.as_str(), got),
                    source: self.remainder(),
                });
                false
            }
        }
    }

    fn remainder(&self) -> String {
        self.tokens[self.pos..]
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn slice_source(&self, start: usize, end: usize) -> String {
        self.tokens[start..end]
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join("")
    }
}

impl<'a> Tok<'a> {
    fn as_str(&self) -> &'a str {
        match self {
            Tok::Ident(s) | Tok::Str(s) => s,
            Tok::LParen => "(",
            Tok::RParen => ")",
            Tok::LBrace => "{",
            Tok::RBrace => "}",
            Tok::LBracket => "[",
            Tok::RBracket => "]",
            Tok::Comma => ",",
            Tok::Colon => ":",
            Tok::Dot => ".",
            Tok::Eq => "=",
            Tok::Arrow => "=>",
        }
    }
}

// ─── Grammar ────────────────────────────────────────────────────────────────

/// Parse a Style expression (chain, guard, bind, or ref).
fn parse_style_expr(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    match p.peek_tok() {
        Some(Tok::Ident("match")) => parse_match(p, diags),
        Some(Tok::Ident("when")) => parse_when(p, diags),
        Some(Tok::Ident("compound")) => parse_compound(p, diags),
        Some(Tok::Ident("bind")) => parse_bind(p, diags),
        Some(Tok::Ident("style")) => parse_chain(p, diags),
        Some(Tok::Ident(name)) => {
            // Could be a scope const ref.
            let name = *name;
            p.bump();
            // If followed by a dot, it's a chain starting with a ref.
            if p.peek_tok() == Some(&Tok::Dot) {
                parse_chain_from_ref(p, name, diags)
            } else {
                Style::ref_(name.to_string())
            }
        }
        _ => {
            let source = p.remainder();
            diags.push(StyleDiagnostic::ParseError {
                message: format!("expected Style expression, found `{source}`"),
                source,
            });
            // Always advance to avoid infinite loops on malformed input.
            p.bump();
            Style::Empty
        }
    }
}

/// Parse `style.a(x).b(y).c(z)` chain.
fn parse_chain(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // style
    parse_chain_rest(p, Vec::new(), diags)
}

/// Continue parsing a chain after the initial `style` or ref head.
fn parse_chain_rest(
    p: &mut Parser<'_>,
    mut items: Vec<Style>,
    diags: &mut Vec<StyleDiagnostic>,
) -> Style {
    // Safety bound to prevent infinite loops on malformed input.
    let max_chain = 256;
    let mut count = 0;
    while p.peek_tok() == Some(&Tok::Dot) && count < max_chain {
        count += 1;
        p.bump(); // .
        let setter = match p.bump() {
            Some(Tok::Ident(s)) => s.to_string(),
            other => {
                let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
                diags.push(StyleDiagnostic::ParseError {
                    message: format!("expected setter name after `.`, found `{source}`"),
                    source: source.clone(),
                });
                source
            }
        };
        p.expect(Tok::LParen, diags);
        let arg = parse_arg(p, diags);
        p.expect(Tok::RParen, diags);
        let source = format!("{setter}({:?})", arg);
        items.push(Style::Set(PaintOp {
            name: setter,
            arg,
            source,
        }));
    }
    Style::chain(items)
}

fn parse_chain_from_ref(
    p: &mut Parser<'_>,
    ref_name: &str,
    diags: &mut Vec<StyleDiagnostic>,
) -> Style {
    let mut items = vec![Style::ref_(ref_name.to_string())];
    // Safety bound to prevent infinite loops on malformed input.
    let max_chain = 256;
    let mut count = 0;
    while p.peek_tok() == Some(&Tok::Dot) && count < max_chain {
        count += 1;
        p.bump(); // .
        let setter = match p.bump() {
            Some(Tok::Ident(s)) => s.to_string(),
            other => {
                let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
                diags.push(StyleDiagnostic::ParseError {
                    message: format!("expected setter name after `.`, found `{source}`"),
                    source: source.clone(),
                });
                source
            }
        };
        p.expect(Tok::LParen, diags);
        let arg = parse_arg(p, diags);
        p.expect(Tok::RParen, diags);
        let source = format!("{setter}({:?})", arg);
        items.push(Style::Set(PaintOp {
            name: setter,
            arg,
            source,
        }));
    }
    Style::chain(items)
}

fn parse_arg(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> SxValue {
    match p.bump() {
        Some(Tok::Str(s)) => SxValue::Token(s.to_string()),
        Some(Tok::Ident("bind")) => {
            p.expect(Tok::LParen, diags);
            let name = match p.bump() {
                Some(Tok::Ident(s)) => s.to_string(),
                other => {
                    let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
                    diags.push(StyleDiagnostic::ParseError {
                        message: format!("expected bind name, found `{source}`"),
                        source: source.clone(),
                    });
                    source
                }
            };
            p.expect(Tok::RParen, diags);
            SxValue::VarRef(name)
        }
        Some(Tok::Ident(s)) => SxValue::Token(s.to_string()),
        other => {
            let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
            diags.push(StyleDiagnostic::ParseError {
                message: format!("expected token argument, found `{source}`"),
                source: source.clone(),
            });
            SxValue::Token(source)
        }
    }
}

/// Parse `match(scrutinee, { key: style, … })`.
fn parse_match(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // match
    p.expect(Tok::LParen, diags);
    let scrutinee = match p.bump() {
        Some(Tok::Ident(s)) => s.to_string(),
        other => {
            let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
            diags.push(StyleDiagnostic::IllegalScrutinee {
                scrutinee: source.clone(),
                source,
            });
            String::new()
        }
    };
    p.expect(Tok::Comma, diags);
    p.expect(Tok::LBrace, diags);
    let mut arms = Vec::new();
    let max_arms = 64;
    let mut arm_count = 0;
    while !p.at_end() && p.peek_tok() != Some(&Tok::RBrace) && arm_count < max_arms {
        arm_count += 1;
        let key = match p.bump() {
            Some(Tok::Ident(s)) => s.to_string(),
            Some(Tok::Str(s)) => s.to_string(),
            other => {
                let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
                diags.push(StyleDiagnostic::ParseError {
                    message: format!("expected match arm key, found `{source}`"),
                    source: source.clone(),
                });
                source
            }
        };
        p.expect(Tok::Colon, diags);
        let body = parse_style_expr(p, diags);
        arms.push((key, body));
        if p.peek_tok() == Some(&Tok::Comma) {
            p.bump();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::RParen, diags);
    Style::match_(scrutinee, arms)
}

/// Parse `when({ key: style, … })`.
fn parse_when(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // when
    p.expect(Tok::LParen, diags);
    p.expect(Tok::LBrace, diags);
    let mut map = Vec::new();
    let max_keys = 64;
    let mut key_count = 0;
    while !p.at_end() && p.peek_tok() != Some(&Tok::RBrace) && key_count < max_keys {
        key_count += 1;
        let key = match p.bump() {
            Some(Tok::Ident(s)) => s.to_string(),
            other => {
                let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
                diags.push(StyleDiagnostic::IllegalWhenKey {
                    key: source.clone(),
                    source: source.clone(),
                });
                source
            }
        };
        p.expect(Tok::Colon, diags);
        let body = parse_style_expr(p, diags);
        map.push((key, body));
        if p.peek_tok() == Some(&Tok::Comma) {
            p.bump();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::RParen, diags);
    Style::when(map)
}

/// Parse `compound({ k: v, … }, style)`.
fn parse_compound(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // compound
    p.expect(Tok::LParen, diags);
    p.expect(Tok::LBrace, diags);
    let mut conditions = Vec::new();
    let max_conditions = 64;
    let mut cond_count = 0;
    while !p.at_end() && p.peek_tok() != Some(&Tok::RBrace) && cond_count < max_conditions {
        cond_count += 1;
        let key = match p.bump() {
            Some(Tok::Ident(s)) => s.to_string(),
            other => {
                let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
                diags.push(StyleDiagnostic::ParseError {
                    message: format!("expected compound condition key, found `{source}`"),
                    source: source.clone(),
                });
                source
            }
        };
        p.expect(Tok::Colon, diags);
        let value = match p.bump() {
            Some(Tok::Ident(s)) => s.to_string(),
            Some(Tok::Str(s)) => s.to_string(),
            other => {
                let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
                diags.push(StyleDiagnostic::ParseError {
                    message: format!("expected compound condition value, found `{source}`"),
                    source: source.clone(),
                });
                source
            }
        };
        conditions.push((key, value));
        if p.peek_tok() == Some(&Tok::Comma) {
            p.bump();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::Comma, diags);
    let body = parse_style_expr(p, diags);
    p.expect(Tok::RParen, diags);
    Style::compound(conditions, body)
}

/// Parse `bind(name)`.
fn parse_bind(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // bind
    p.expect(Tok::LParen, diags);
    let name = match p.bump() {
        Some(Tok::Ident(s)) => s.to_string(),
        other => {
            let source = other.map(|t| t.as_str().to_string()).unwrap_or_default();
            diags.push(StyleDiagnostic::IllegalBind {
                name: source.clone(),
                source: source.clone(),
            });
                source
        }
    };
    p.expect(Tok::RParen, diags);
    Style::bind(name)
}

/// Parse a Style list `[style, style, …]` (for `sx={[…]}`).
pub fn parse_style_list(input: &str) -> (Vec<Style>, Vec<StyleDiagnostic>) {
    let mut diags = Vec::new();
    let tokens = tokenize(input.trim());
    let mut p = Parser::new(&tokens);
    let mut items = Vec::new();
    if p.peek_tok() == Some(&Tok::LBracket) {
        p.bump();
        let max_items = 64;
        let mut item_count = 0;
        while !p.at_end() && p.peek_tok() != Some(&Tok::RBracket) && item_count < max_items {
            item_count += 1;
            items.push(parse_style_expr(&mut p, &mut diags));
            if p.peek_tok() == Some(&Tok::Comma) {
                p.bump();
            }
        }
        p.expect(Tok::RBracket, &mut diags);
    } else {
        diags.push(StyleDiagnostic::ParseError {
            message: "expected `[…]` style list".to_string(),
            source: input.to_string(),
        });
    }
    (items, diags)
}

/// Parse scope `const name = style…` declarations.
///
/// Input is the raw body of a `<scope>` block. Returns a map of const names
/// to their Style values.
pub fn parse_scope_consts(input: &str) -> (HashMap<String, Style>, Vec<StyleDiagnostic>) {
    let mut consts = HashMap::new();
    let mut diags = Vec::new();

    // Split on top-level newlines/commas, then parse each `const name = style…`.
    for entry in split_top_level(input, '\n') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        if !entry.starts_with("const ") {
            continue;
        }
        let Some(rest) = entry.strip_prefix("const ") else {
            continue;
        };
        let Some((name, expr)) = rest.split_once('=') else {
            continue;
        };
        let name = name.trim();
        let expr = expr.trim().trim_end_matches(';');
        let (style, mut d) = parse(expr);
        diags.append(&mut d);
        consts.insert(name.to_string(), style);
    }

    (consts, diags)
}

fn split_top_level(s: &str, delim: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut in_str = None::<char>;
    for c in s.chars() {
        if let Some(q) = in_str {
            cur.push(c);
            if c == q {
                in_str = None;
            }
            continue;
        }
        match c {
            '\'' | '"' => {
                in_str = Some(c);
                cur.push(c);
            }
            '{' | '(' | '[' => {
                depth += 1;
                cur.push(c);
            }
            '}' | ')' | ']' => {
                depth -= 1;
                cur.push(c);
            }
            c if c == delim && depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// Returns true if the input looks like the new Style expression grammar.
pub fn uses_style_syntax(input: &str) -> bool {
    input.contains("style.")
        || input.contains("match(")
        || input.contains("when({")
        || input.contains("compound(")
        || input.contains("bind(")
}
