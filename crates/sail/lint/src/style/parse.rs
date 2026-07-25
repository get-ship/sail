//! Parser for the Style value language (chain + match / when / compound / bind).

use std::borrow::Cow;
use std::collections::BTreeMap;

use super::ir::{PaintArg, PaintOp, Style, StyleDiagnostic};
use super::setters::{find_setter, validate_op, when_target};

/// Parse a Style expression. Returns Style + diagnostics (fail-closed flags in diags).
pub fn parse(input: &str) -> (Style, Vec<StyleDiagnostic>) {
    let mut diags = Vec::new();
    // Authors reasonably write `//` / `/* */` comments inside long sx lists.
    // Strip them before tokenizing: the grammar has no division and no string
    // literals, so `/` can only start a comment. Without this, comment text
    // tokenizes into Words — silently injecting spurious Refs and truncating
    // list entries after the comment.
    let stripped = strip_comments(input);
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return (Style::Empty, diags);
    }
    // Residual utility-string product → hard error (not architecture).
    if looks_like_utility_string_product(trimmed) {
        diags.push(StyleDiagnostic::ResidualProduct {
            message: "utility-string sx is not the Style product — use style.bg(token).… chains, match, when({…}), compound"
                .into(),
            source: trimmed.chars().take(80).collect(),
        });
        return (Style::Empty, diags);
    }
    let tokens = tokenize(trimmed);
    let mut p = Parser::new(&tokens);
    let style = parse_style_expr(&mut p, &mut diags);
    if !p.at_end() {
        diags.push(StyleDiagnostic::ParseError {
            message: format!("unexpected token `{}`", p.peek_str()),
            source: p.remainder(),
        });
    }
    // Validate all paint ops.
    validate_style(&style, &mut diags);
    (style, diags)
}

/// True if input is the killed utility-string product (atoms / prop: value soup without style./match/when).
pub fn looks_like_utility_string_product(input: &str) -> bool {
    let t = input.trim();
    if t.starts_with("style.")
        || t.starts_with("match(")
        
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('[')
    {
        return false;
    }
    // Ident-only ref: `filled` — allowed as Style const ref
    if t.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return false;
    }
    // Space-separated utility soup or legacy atoms
    t.contains(':') || t.split_whitespace().any(|w| w.contains('-') && !w.starts_with("style"))
}

/// Remove `//` line and `/* */` block comments, replacing each with a single
/// space so words on either side can never merge. Unterminated block comments
/// consume to end of input (the truncated expression then fails loudly with a
/// normal parse diagnostic instead of silently dropping entries).
fn strip_comments(input: &str) -> Cow<'_, str> {
    let b = input.as_bytes();
    let mut out: Option<String> = None;
    let mut copied = 0; // input[..copied] already flushed into out
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'/' && (b[i + 1] == b'/' || b[i + 1] == b'*') {
            let s = out.get_or_insert_with(|| String::with_capacity(input.len()));
            s.push_str(&input[copied..i]);
            s.push(' ');
            if b[i + 1] == b'/' {
                let mut j = i + 2;
                while j < b.len() && b[j] != b'\n' {
                    j += 1;
                }
                i = j;
            } else {
                let mut j = i + 2;
                while j + 1 < b.len() && !(b[j] == b'*' && b[j + 1] == b'/') {
                    j += 1;
                }
                i = if j + 1 < b.len() { j + 2 } else { b.len() };
            }
            copied = i;
        } else {
            i += 1;
        }
    }
    match out {
        Some(mut s) => {
            s.push_str(&input[copied..]);
            Cow::Owned(s)
        }
        None => Cow::Borrowed(input),
    }
}

// ─── tokenize ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok<'a> {
    Word(&'a str),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrack,
    RBrack,
    Dot,
    Comma,
    Colon,
    Arrow, // =>
    Eq,
}

fn tokenize(input: &str) -> Vec<Tok<'_>> {
    let mut out = Vec::new();
    let b = input.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '=' && i + 1 < b.len() && b[i + 1] == b'>' {
            out.push(Tok::Arrow);
            i += 2;
            continue;
        }
        match c {
            '(' => out.push(Tok::LParen),
            ')' => out.push(Tok::RParen),
            '{' => out.push(Tok::LBrace),
            '}' => out.push(Tok::RBrace),
            '[' => out.push(Tok::LBrack),
            ']' => out.push(Tok::RBrack),
            '.' => out.push(Tok::Dot),
            ',' => out.push(Tok::Comma),
            ':' => out.push(Tok::Colon),
            '=' => out.push(Tok::Eq),
            // Words: identifiers, tokens, and leading-minus tokens (`-px`).
            _ if c.is_ascii_alphanumeric() || c == '_' || c == '-' => {
                let start = i;
                let mut j = i + 1;
                while j < b.len() {
                    let ch = b[j] as char;
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                        j += 1;
                    } else {
                        break;
                    }
                }
                // Bare `-` is not a word — leave it for the skip path.
                if &input[start..j] == "-" {
                    i = j;
                    continue;
                }
                out.push(Tok::Word(&input[start..j]));
                i = j;
                continue;
            }
            _ => {
                // skip unknown one char
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
    fn peek(&self) -> Option<&Tok<'a>> {
        self.tokens.get(self.pos)
    }
    fn peek_str(&self) -> &str {
        match self.peek() {
            Some(Tok::Word(w)) => w,
            Some(Tok::LParen) => "(",
            Some(Tok::RParen) => ")",
            Some(Tok::LBrace) => "{",
            Some(Tok::RBrace) => "}",
            Some(Tok::LBrack) => "[",
            Some(Tok::RBrack) => "]",
            Some(Tok::Dot) => ".",
            Some(Tok::Comma) => ",",
            Some(Tok::Colon) => ":",
            Some(Tok::Arrow) => "=>",
            Some(Tok::Eq) => "=",
            None => "<eof>",
        }
    }
    fn bump(&mut self) -> Option<Tok<'a>> {
        let t = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(t)
    }
    fn eat(&mut self, t: &Tok<'static>) -> bool {
        if self.peek() == Some(t) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, t: Tok<'static>, diags: &mut Vec<StyleDiagnostic>) -> bool {
        if self.eat(&t) {
            true
        } else {
            diags.push(StyleDiagnostic::ParseError {
                message: format!("expected `{}`, found `{}`", tok_str(&t), self.peek_str()),
                source: self.remainder(),
            });
            false
        }
    }
    fn remainder(&self) -> String {
        self.tokens[self.pos..]
            .iter()
            .map(tok_str_ref)
            .collect::<Vec<_>>()
            .join("")
    }

    fn force_progress(&mut self) {
        if !self.at_end() {
            self.pos += 1;
        }
    }
}

/// Cap map/list/chain iterations so a stuck parser cannot run unbounded.
const MAX_PARSE_ITERS: usize = 10_000;

fn tok_str<'a>(t: &'a Tok<'a>) -> &'a str {
    match t {
        Tok::Word(w) => w,
        Tok::LParen => "(",
        Tok::RParen => ")",
        Tok::LBrace => "{",
        Tok::RBrace => "}",
        Tok::LBrack => "[",
        Tok::RBrack => "]",
        Tok::Dot => ".",
        Tok::Comma => ",",
        Tok::Colon => ":",
        Tok::Arrow => "=>",
        Tok::Eq => "=",
    }
}
fn tok_str_ref<'a>(t: &'a Tok<'a>) -> &'a str {
    tok_str(t)
}

// ─── grammar ────────────────────────────────────────────────────────────────

fn parse_style_expr(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    match p.peek() {
        Some(Tok::LBrack) => parse_list(p, diags),
        Some(Tok::Word("match")) => parse_match(p, diags),
        Some(Tok::Word("when")) => parse_when_call(p, diags),
        Some(Tok::Word("compound")) => parse_compound(p, diags),
        Some(Tok::Word("style")) => parse_style_chain(p, diags),
        Some(Tok::Word(_)) => {
            // Style const ref or start of something unexpected
            let name = match p.bump() {
                Some(Tok::Word(w)) => w.to_string(),
                _ => return Style::Empty,
            };
            // `filled.when({…})`
            if p.peek() == Some(&Tok::Dot) {
                let mut base = Style::Ref(name);
                while p.eat(&Tok::Dot) {
                    if p.peek() == Some(&Tok::Word("when")) {
                        p.bump();
                        let entries = parse_when_map_body(p, diags);
                        base = Style::chain(vec![base, Style::When { entries }]);
                    } else {
                        diags.push(StyleDiagnostic::ParseError {
                            message: "only `.when({…})` may follow a Style ref".into(),
                            source: p.remainder(),
                        });
                        break;
                    }
                }
                base
            } else {
                Style::Ref(name)
            }
        }
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected Style expression (style.…, match, when, compound, name, or […])"
                    .into(),
                source: p.remainder(),
            });
            Style::Empty
        }
    }
}

fn parse_list(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.expect(Tok::LBrack, diags);
    let mut items = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrack) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "style list too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        items.push(parse_style_expr(p, diags));
        if p.pos == before {
            // Malformed item — skip one token so we cannot spin forever.
            p.force_progress();
            continue;
        }
        if !p.eat(&Tok::Comma) {
            break;
        }
    }
    p.expect(Tok::RBrack, diags);
    Style::List(items)
}

fn parse_style_chain(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.expect(Tok::Word("style"), diags);
    let mut ops = Vec::new();
    let mut when_entries = None;
    let mut iters = 0usize;
    loop {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "style chain too many setters".into(),
                source: p.remainder(),
            });
            break;
        }
        if !p.eat(&Tok::Dot) {
            break;
        }
        let before = p.pos;
        let method = match p.bump() {
            Some(Tok::Word(w)) => w,
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "expected setter name after `.`".into(),
                    source: p.remainder(),
                });
                break;
            }
        };
        if method == "when" {
            when_entries = Some(parse_when_map_body(p, diags));
            break;
        }
        p.expect(Tok::LParen, diags);
        // Empty-arg markers (style.group()) or comma-separated paint args
        // (`style.border(1, border)` — width + color in one call).
        let mut args = Vec::new();
        if p.peek() == Some(&Tok::RParen) {
            args.push(PaintArg::Token(String::new()));
        } else {
            args.push(parse_paint_arg(p, diags));
            while p.eat(&Tok::Comma) {
                args.push(parse_paint_arg(p, diags));
            }
        }
        p.expect(Tok::RParen, diags);
        if p.pos == before {
            p.force_progress();
            continue;
        }
        let source = format!("style.{method}(…)");
        if find_setter(method).is_none() {
            diags.push(StyleDiagnostic::UnknownSetter {
                setter: method.to_string(),
                source: source.clone(),
            });
        } else {
            let op = PaintOp {
                setter: method.to_string(),
                args: args.clone(),
                source: source.clone(),
            };
            if let Err(d) = validate_op(&op) {
                diags.push(d);
            }
        }
        ops.push(Style::Set(PaintOp {
            setter: method.to_string(),
            args,
            source,
        }));
    }
    let chain = if ops.is_empty() {
        Style::Empty
    } else {
        Style::chain(ops)
    };
    if let Some(entries) = when_entries {
        Style::chain(vec![chain, Style::When { entries }])
    } else {
        chain
    }
}

fn parse_paint_arg(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> PaintArg {
    match p.bump() {
        Some(Tok::Word("bind")) => {
            p.expect(Tok::LParen, diags);
            let name = match p.bump() {
                Some(Tok::Word(w)) => w.to_string(),
                _ => {
                    diags.push(StyleDiagnostic::ParseError {
                        message: "bind(name) expects an identifier".into(),
                        source: p.remainder(),
                    });
                    String::new()
                }
            };
            p.expect(Tok::RParen, diags);
            PaintArg::Bind(name)
        }
        // Negatable token: style.ms(-px) tokenizes as Word("-px") after we allow leading '-'.
        Some(Tok::Word(w)) => PaintArg::Token(w.to_string()),
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected token or bind(name)".into(),
                source: p.remainder(),
            });
            PaintArg::Token(String::new())
        }
    }
}

fn parse_match(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    // match(scrutinee, { arm: Style, … })
    p.bump();
    p.expect(Tok::LParen, diags);
    let scrutinee = parse_scrutinee(p, diags);
    p.expect(Tok::Comma, diags);
    p.expect(Tok::LBrace, diags);
    let mut arms = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "match arms too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let key = match p.bump() {
            Some(Tok::Word(w)) => w.to_string(),
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "expected match arm key".into(),
                    source: p.remainder(),
                });
                p.force_progress();
                break;
            }
        };
        p.expect(Tok::Colon, diags);
        let body = parse_style_expr(p, diags);
        arms.push((key, body));
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::RParen, diags);
    Style::Match { scrutinee, arms }
}

fn parse_scrutinee(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> String {
    // variant | props.variant | ($) => $.variant
    if p.peek() == Some(&Tok::LParen) {
        // static lambda: ( $ ) => $ . path
        p.bump();
        let _param = match p.bump() {
            Some(Tok::Word(w)) => w.to_string(),
            _ => {
                diags.push(StyleDiagnostic::BadScrutinee {
                    message: "match lambda must be static path: ($) => $.prop".into(),
                    source: p.remainder(),
                });
                return String::new();
            }
        };
        p.expect(Tok::RParen, diags);
        if !p.eat(&Tok::Arrow) {
            diags.push(StyleDiagnostic::BadScrutinee {
                message: "match lambda requires `=>`".into(),
                source: p.remainder(),
            });
            return String::new();
        }
        // $.prop or $ . prop
        let mut path = Vec::new();
        if let Some(Tok::Word(w)) = p.bump() {
            // might be $ 
            if w != "$" && w != _param {
                // allow bare prop after =>
                return w.to_string();
            }
        }
        while p.eat(&Tok::Dot) {
            if let Some(Tok::Word(seg)) = p.bump() {
                path.push(seg.to_string());
            }
        }
        if path.is_empty() {
            diags.push(StyleDiagnostic::BadScrutinee {
                message: "match lambda must select a prop path".into(),
                source: String::new(),
            });
            String::new()
        } else {
            path.join(".")
        }
    } else {
        // variant or props.variant
        let mut parts = Vec::new();
        if let Some(Tok::Word(w)) = p.bump() {
            parts.push(w.to_string());
        } else {
            diags.push(StyleDiagnostic::BadScrutinee {
                message: "match scrutinee must be a prop identifier".into(),
                source: p.remainder(),
            });
            return String::new();
        }
        while p.eat(&Tok::Dot) {
            if let Some(Tok::Word(w)) = p.bump() {
                parts.push(w.to_string());
            }
        }
        // props.variant → variant for attr emit
        if parts.len() > 1 && parts[0] == "props" {
            parts[1..].join(".")
        } else {
            parts.join(".")
        }
    }
}

fn parse_when_call(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // when
    let entries = parse_when_map_body(p, diags);
    Style::When { entries }
}

fn parse_when_map_body(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Vec<(String, Style)> {
    p.expect(Tok::LParen, diags);
    p.expect(Tok::LBrace, diags);
    let mut entries = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "when map too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let key = match p.bump() {
            Some(Tok::Word(w)) => w.to_string(),
            _ => {
                p.force_progress();
                break;
            }
        };
        if when_target(&key).is_none() {
            diags.push(StyleDiagnostic::UnknownWhenKey {
                key: key.clone(),
                source: key.clone(),
            });
        }
        p.expect(Tok::Colon, diags);
        let body = parse_style_expr(p, diags);
        entries.push((key, body));
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::RParen, diags);
    entries
}

fn parse_compound(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // compound
    p.expect(Tok::LParen, diags);
    p.expect(Tok::LBrace, diags);
    let mut when = BTreeMap::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "compound map too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let k = match p.bump() {
            Some(Tok::Word(w)) => w.to_string(),
            _ => {
                p.force_progress();
                break;
            }
        };
        p.expect(Tok::Colon, diags);
        let v = match p.bump() {
            Some(Tok::Word(w)) => w.to_string(),
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "compound condition value must be a literal".into(),
                    source: p.remainder(),
                });
                String::new()
            }
        };
        when.insert(k, v);
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::Comma, diags);
    let body = parse_style_expr(p, diags);
    p.expect(Tok::RParen, diags);
    Style::Compound {
        when,
        body: Box::new(body),
    }
}

fn validate_style(style: &Style, diags: &mut Vec<StyleDiagnostic>) {
    match style {
        Style::Empty | Style::Ref(_) => {}
        Style::Set(op) => {
            if let Err(e) = validate_op(op) {
                diags.push(e);
            }
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                validate_style(i, diags);
            }
        }
        Style::Match { arms, .. } => {
            for (_, s) in arms {
                validate_style(s, diags);
            }
        }
        Style::Compound { body, .. } => validate_style(body, diags),
        Style::When { entries } => {
            for (_, s) in entries {
                validate_style(s, diags);
            }
        }
    }
}

/// Parse `const name = <style-expr>` (scope Style const line, without the `const` keyword optional).
pub fn parse_style_const_rhs(input: &str) -> (Style, Vec<StyleDiagnostic>) {
    parse(input)
}
