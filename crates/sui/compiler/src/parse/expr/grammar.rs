//! Recursive-descent parser for the constrained `{expr}` subset.
//!
//! Grammar lives in `crates/sui/compiler/EXPRESSIONS.md`. This module
//! produces an [`Ast`] tree with byte-range spans pointing into the
//! original `.sui` source. The caller passes a `base_offset` so spans
//! are absolute in the file, not relative to the inner-of-brace slice.
//!
//! The parser does NOT enforce the renderable subset's "no calls with
//! args except allow-listed methods" rule — that's `expr_subset`'s
//! job. The parser's job is to produce structure; the post-parse
//! checks reject what the runtime won't accept.

use super::ast::{Arg, ArrowBody, Ast, AstKind, ObjEntry, ObjKey, TemplatePart};
use crate::diagnostics::Span;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

/// Parse `inner` (the brace-stripped expression source) starting at
/// absolute `base_offset` in the file. Arrows are rejected.
pub fn parse_expression(inner: &str, base_offset: usize) -> Result<Ast, ParseError> {
    parse_root(inner, base_offset, false)
}

/// Same as [`parse_expression`] but allows arrow-function bodies
/// (used for `on:event` handlers).
pub fn parse_expression_with_arrows(
    inner: &str,
    base_offset: usize,
) -> Result<Ast, ParseError> {
    parse_root(inner, base_offset, true)
}

fn parse_root(inner: &str, base_offset: usize, allow_arrow: bool) -> Result<Ast, ParseError> {
    let mut p = Parser::new(inner.as_bytes(), base_offset, allow_arrow);
    p.skip_ws();
    if p.eof() {
        return Err(p.err_at(p.i, "empty expression"));
    }
    let ast = p.parse_expr()?;
    p.skip_ws();
    if !p.eof() {
        return Err(p.err_at(p.i, "trailing input after expression"));
    }
    Ok(ast)
}

struct Parser<'a> {
    bytes: &'a [u8],
    base: usize,
    i: usize,
    allow_arrow: bool,
}

impl<'a> Parser<'a> {
    fn new(bytes: &'a [u8], base: usize, allow_arrow: bool) -> Self {
        Self { bytes, base, i: 0, allow_arrow }
    }

    fn eof(&self) -> bool {
        self.i >= self.bytes.len()
    }

    fn peek(&self) -> u8 {
        self.bytes.get(self.i).copied().unwrap_or(0)
    }

    fn peek_at(&self, off: usize) -> u8 {
        self.bytes.get(self.i + off).copied().unwrap_or(0)
    }

    fn starts_with(&self, s: &str) -> bool {
        self.i + s.len() <= self.bytes.len() && &self.bytes[self.i..self.i + s.len()] == s.as_bytes()
    }

    fn skip_ws(&mut self) {
        while self.i < self.bytes.len() && self.bytes[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn span_from(&self, start: usize) -> Span {
        Span::new(self.base + start, self.base + self.i)
    }

    fn err_at(&self, byte: usize, msg: impl Into<String>) -> ParseError {
        ParseError {
            message: msg.into(),
            span: Span::point(self.base + byte),
        }
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        std::str::from_utf8(&self.bytes[start..end]).unwrap_or("")
    }

    fn eat(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.i += s.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, s: &str) -> Result<(), ParseError> {
        if self.eat(s) {
            Ok(())
        } else {
            Err(self.err_at(self.i, format!("expected `{s}`")))
        }
    }

    // ── Expression precedence ladder ────────────────────────────────

    fn parse_expr(&mut self) -> Result<Ast, ParseError> {
        // Detect arrow at the top so `(a, b) => …` doesn't eat `,` as
        // a comma operator (we don't have one anyway).
        if self.allow_arrow && self.is_arrow_start() {
            return self.parse_arrow();
        }
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Ast, ParseError> {
        let start = self.i;
        let cond = self.parse_coalesce()?;
        self.skip_ws();
        if self.eat("?") {
            // Disambiguate from `?.`.
            if self.peek() == b'.' {
                self.i -= 1;
                return Ok(cond);
            }
            self.skip_ws();
            let then = self.parse_expr()?;
            self.skip_ws();
            self.expect(":")?;
            self.skip_ws();
            let otherwise = self.parse_expr()?;
            return Ok(Ast::new(
                AstKind::Ternary {
                    cond: Box::new(cond),
                    then: Box::new(then),
                    otherwise: Box::new(otherwise),
                },
                self.span_from(start),
            ));
        }
        Ok(cond)
    }

    fn parse_coalesce(&mut self) -> Result<Ast, ParseError> {
        self.parse_left_assoc(&[("??", false)], &Self::parse_logical_or)
    }
    fn parse_logical_or(&mut self) -> Result<Ast, ParseError> {
        self.parse_left_assoc(&[("||", true)], &Self::parse_logical_and)
    }
    fn parse_logical_and(&mut self) -> Result<Ast, ParseError> {
        self.parse_left_assoc(&[("&&", true)], &Self::parse_equality)
    }
    fn parse_equality(&mut self) -> Result<Ast, ParseError> {
        self.parse_left_assoc(
            &[("===", false), ("!==", false), ("==", false), ("!=", false)],
            &Self::parse_relational,
        )
    }
    fn parse_relational(&mut self) -> Result<Ast, ParseError> {
        self.parse_left_assoc(
            &[("<=", false), (">=", false), ("<", false), (">", false)],
            &Self::parse_additive,
        )
    }
    fn parse_additive(&mut self) -> Result<Ast, ParseError> {
        self.parse_left_assoc(&[("+", false), ("-", false)], &Self::parse_multiplicative)
    }
    fn parse_multiplicative(&mut self) -> Result<Ast, ParseError> {
        self.parse_left_assoc(
            &[("*", false), ("/", false), ("%", false)],
            &Self::parse_unary,
        )
    }

    fn parse_left_assoc(
        &mut self,
        ops: &[(&str, bool)],
        next: &dyn Fn(&mut Self) -> Result<Ast, ParseError>,
    ) -> Result<Ast, ParseError> {
        let start = self.i;
        let mut lhs = next(self)?;
        loop {
            self.skip_ws();
            let mut matched: Option<(&str, bool)> = None;
            for &(op, logical) in ops {
                if self.starts_with(op) {
                    // Reject `<=` matching `<` early — longest-match.
                    matched = Some((op, logical));
                    break;
                }
            }
            let Some((op, is_logical)) = matched else {
                break;
            };
            // Disambiguate `+` / `-` consuming a unary (handled by parse_unary).
            self.i += op.len();
            self.skip_ws();
            let rhs = next(self)?;
            let kind = if is_logical {
                AstKind::Logical { op: op.to_string(), lhs: Box::new(lhs), rhs: Box::new(rhs) }
            } else {
                AstKind::Binary { op: op.to_string(), lhs: Box::new(lhs), rhs: Box::new(rhs) }
            };
            lhs = Ast::new(kind, self.span_from(start));
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Ast, ParseError> {
        self.skip_ws();
        let start = self.i;
        if self.peek() == b'!' && self.peek_at(1) != b'=' {
            self.i += 1;
            self.skip_ws();
            let arg = self.parse_unary()?;
            return Ok(Ast::new(
                AstKind::Unary { op: "!".into(), arg: Box::new(arg) },
                self.span_from(start),
            ));
        }
        if self.peek() == b'-' || self.peek() == b'+' {
            let op = self.peek() as char;
            // Don't eat as unary if followed by another `+`/`-` (compound op
            // surfaces from the binary-ladder caller, not here).
            if self.peek_at(1) != self.peek() {
                self.i += 1;
                self.skip_ws();
                let arg = self.parse_unary()?;
                return Ok(Ast::new(
                    AstKind::Unary { op: op.to_string(), arg: Box::new(arg) },
                    self.span_from(start),
                ));
            }
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Ast, ParseError> {
        let start = self.i;
        let mut node = self.parse_primary()?;
        loop {
            self.skip_ws();
            if self.eat("?.") {
                self.skip_ws();
                if self.peek() == b'(' {
                    let args = self.parse_call_args()?;
                    node = Ast::new(
                        AstKind::Call { callee: Box::new(node), args, optional: true },
                        self.span_from(start),
                    );
                } else if self.peek() == b'[' {
                    self.i += 1;
                    self.skip_ws();
                    let idx = self.parse_expr()?;
                    self.skip_ws();
                    self.expect("]")?;
                    node = Ast::new(
                        AstKind::Index { obj: Box::new(node), idx: Box::new(idx), optional: true },
                        self.span_from(start),
                    );
                } else {
                    let prop = self.parse_ident()?;
                    node = Ast::new(
                        AstKind::Member { obj: Box::new(node), prop, optional: true },
                        self.span_from(start),
                    );
                }
            } else if self.peek() == b'.' {
                self.i += 1;
                self.skip_ws();
                let prop = self.parse_ident()?;
                node = Ast::new(
                    AstKind::Member { obj: Box::new(node), prop, optional: false },
                    self.span_from(start),
                );
            } else if self.peek() == b'[' {
                self.i += 1;
                self.skip_ws();
                let idx = self.parse_expr()?;
                self.skip_ws();
                self.expect("]")?;
                node = Ast::new(
                    AstKind::Index { obj: Box::new(node), idx: Box::new(idx), optional: false },
                    self.span_from(start),
                );
            } else if self.peek() == b'(' {
                let args = self.parse_call_args()?;
                node = Ast::new(
                    AstKind::Call { callee: Box::new(node), args, optional: false },
                    self.span_from(start),
                );
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Arg>, ParseError> {
        self.expect("(")?;
        let mut out = Vec::new();
        self.skip_ws();
        if self.peek() == b')' {
            self.i += 1;
            return Ok(out);
        }
        loop {
            self.skip_ws();
            if self.eat("...") {
                self.skip_ws();
                out.push(Arg::Spread(self.parse_expr()?));
            } else {
                out.push(Arg::Expr(self.parse_expr()?));
            }
            self.skip_ws();
            if self.eat(",") {
                continue;
            }
            break;
        }
        self.skip_ws();
        self.expect(")")?;
        Ok(out)
    }

    fn parse_primary(&mut self) -> Result<Ast, ParseError> {
        self.skip_ws();
        let start = self.i;
        if self.eof() {
            return Err(self.err_at(start, "unexpected end of expression"));
        }
        let c = self.peek();
        if c == b'(' {
            self.i += 1;
            self.skip_ws();
            let inner = self.parse_expr()?;
            self.skip_ws();
            self.expect(")")?;
            // Re-span the parenthesised group so the AST node covers the
            // outer parens, not just the inner expression — useful for
            // hover / cursor lookups.
            return Ok(Ast::new(inner.kind, self.span_from(start)));
        }
        if c == b'[' {
            return self.parse_array(start);
        }
        if c == b'{' {
            return self.parse_object(start);
        }
        if c == b'`' {
            return self.parse_template(start);
        }
        if c == b'"' || c == b'\'' {
            return self.parse_string(start, c);
        }
        if c.is_ascii_digit() || (c == b'.' && self.peek_at(1).is_ascii_digit()) {
            return self.parse_number(start);
        }
        if is_ident_start(c) {
            let name = self.parse_ident()?;
            let span = self.span_from(start);
            let kind = match name.as_str() {
                "true" => AstKind::Bool(true),
                "false" => AstKind::Bool(false),
                "null" => AstKind::Null,
                "undefined" => AstKind::Undefined,
                _ => AstKind::Ident(name),
            };
            return Ok(Ast::new(kind, span));
        }
        Err(self.err_at(start, format!("unexpected `{}`", c as char)))
    }

    fn parse_ident(&mut self) -> Result<String, ParseError> {
        let start = self.i;
        if !is_ident_start(self.peek()) {
            return Err(self.err_at(start, "expected identifier"));
        }
        self.i += 1;
        while !self.eof() && is_ident_cont(self.peek()) {
            self.i += 1;
        }
        Ok(self.slice(start, self.i).to_string())
    }

    fn parse_number(&mut self, start: usize) -> Result<Ast, ParseError> {
        while !self.eof() {
            let c = self.peek();
            if c.is_ascii_digit() || c == b'.' || c == b'_' {
                self.i += 1;
                continue;
            }
            if (c == b'e' || c == b'E') && (self.peek_at(1) == b'+' || self.peek_at(1) == b'-' || self.peek_at(1).is_ascii_digit()) {
                self.i += 1;
                if self.peek() == b'+' || self.peek() == b'-' {
                    self.i += 1;
                }
                continue;
            }
            break;
        }
        let raw = self.slice(start, self.i).to_string();
        Ok(Ast::new(AstKind::Number(raw), self.span_from(start)))
    }

    fn parse_string(&mut self, start: usize, quote: u8) -> Result<Ast, ParseError> {
        self.i += 1;
        let mut buf = String::new();
        while !self.eof() {
            let c = self.peek();
            if c == b'\\' {
                let esc = self.peek_at(1);
                buf.push(match esc {
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    b'\\' => '\\',
                    b'\'' => '\'',
                    b'"' => '"',
                    b'`' => '`',
                    0 => return Err(self.err_at(self.i, "unterminated escape")),
                    other => other as char,
                });
                self.i += 2;
                continue;
            }
            if c == quote {
                self.i += 1;
                return Ok(Ast::new(AstKind::String(buf), self.span_from(start)));
            }
            buf.push(c as char);
            self.i += 1;
        }
        Err(self.err_at(start, "unterminated string literal"))
    }

    fn parse_template(&mut self, start: usize) -> Result<Ast, ParseError> {
        self.i += 1;
        let mut parts = Vec::new();
        let mut buf = String::new();
        while !self.eof() {
            let c = self.peek();
            if c == b'`' {
                self.i += 1;
                if !buf.is_empty() {
                    parts.push(TemplatePart::Static(std::mem::take(&mut buf)));
                }
                return Ok(Ast::new(AstKind::Template(parts), self.span_from(start)));
            }
            if c == b'\\' && self.peek_at(1) != 0 {
                buf.push(self.peek_at(1) as char);
                self.i += 2;
                continue;
            }
            if c == b'$' && self.peek_at(1) == b'{' {
                if !buf.is_empty() {
                    parts.push(TemplatePart::Static(std::mem::take(&mut buf)));
                }
                self.i += 2;
                self.skip_ws();
                let inner = self.parse_expr()?;
                self.skip_ws();
                self.expect("}")?;
                parts.push(TemplatePart::Expr(inner));
                continue;
            }
            buf.push(c as char);
            self.i += 1;
        }
        Err(self.err_at(start, "unterminated template literal"))
    }

    fn parse_array(&mut self, start: usize) -> Result<Ast, ParseError> {
        self.i += 1;
        let mut out = Vec::new();
        self.skip_ws();
        if self.eat("]") {
            return Ok(Ast::new(AstKind::Array(out), self.span_from(start)));
        }
        loop {
            self.skip_ws();
            if self.eat("...") {
                self.skip_ws();
                out.push(Arg::Spread(self.parse_expr()?));
            } else {
                out.push(Arg::Expr(self.parse_expr()?));
            }
            self.skip_ws();
            if self.eat(",") {
                self.skip_ws();
                if self.peek() == b']' {
                    break;
                }
                continue;
            }
            break;
        }
        self.skip_ws();
        self.expect("]")?;
        Ok(Ast::new(AstKind::Array(out), self.span_from(start)))
    }

    fn parse_object(&mut self, start: usize) -> Result<Ast, ParseError> {
        self.i += 1;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.eat("}") {
            return Ok(Ast::new(AstKind::Object(entries), self.span_from(start)));
        }
        loop {
            self.skip_ws();
            let entry_start = self.i;
            if self.eat("...") {
                self.skip_ws();
                let value = self.parse_expr()?;
                let span = self.span_from(entry_start);
                entries.push(ObjEntry { key: ObjKey::Spread(value), value: None, span });
            } else if self.peek() == b'"' || self.peek() == b'\'' {
                let q = self.peek();
                let s = self.parse_string(self.i, q)?;
                let key_str = match s.kind {
                    AstKind::String(s) => s,
                    _ => unreachable!(),
                };
                self.skip_ws();
                self.expect(":")?;
                self.skip_ws();
                let value = self.parse_expr()?;
                let span = self.span_from(entry_start);
                entries.push(ObjEntry {
                    key: ObjKey::String(key_str),
                    value: Some(value),
                    span,
                });
            } else if is_ident_start(self.peek()) {
                let name = self.parse_ident()?;
                self.skip_ws();
                let value = if self.eat(":") {
                    self.skip_ws();
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                let span = self.span_from(entry_start);
                entries.push(ObjEntry { key: ObjKey::Ident(name), value, span });
            } else {
                return Err(self.err_at(self.i, "expected object key"));
            }
            self.skip_ws();
            if self.eat(",") {
                self.skip_ws();
                if self.peek() == b'}' {
                    break;
                }
                continue;
            }
            break;
        }
        self.skip_ws();
        self.expect("}")?;
        Ok(Ast::new(AstKind::Object(entries), self.span_from(start)))
    }

    // ── Arrow detection / parsing (handler regions only) ────────────

    fn is_arrow_start(&self) -> bool {
        // `() => …`, `(a, b) => …`, `x => …`, `(a): T => …`.
        let mut k = self.i;
        while k < self.bytes.len() && self.bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= self.bytes.len() {
            return false;
        }
        let c = self.bytes[k];
        if c == b'(' {
            // Skim to matching ')' — return true only if `=>` follows.
            let close = match_paren(self.bytes, k);
            let mut after = match close {
                Some(p) => p + 1,
                None => return false,
            };
            while after < self.bytes.len() && self.bytes[after].is_ascii_whitespace() {
                after += 1;
            }
            // Optional `: T` return-type annotation — skim to `=>` at depth 0.
            if after < self.bytes.len() && self.bytes[after] == b':' {
                let mut depth = 0i32;
                let mut j = after + 1;
                while j + 1 < self.bytes.len() {
                    match self.bytes[j] {
                        b'(' | b'[' | b'<' | b'{' => depth += 1,
                        b')' | b']' | b'>' | b'}' => depth -= 1,
                        b'=' if depth == 0 && self.bytes[j + 1] == b'>' => return true,
                        _ => {}
                    }
                    j += 1;
                }
                return false;
            }
            return after + 1 < self.bytes.len()
                && self.bytes[after] == b'='
                && self.bytes[after + 1] == b'>';
        }
        if is_ident_start(c) {
            let mut j = k + 1;
            while j < self.bytes.len() && is_ident_cont(self.bytes[j]) {
                j += 1;
            }
            let mut after = j;
            while after < self.bytes.len() && self.bytes[after].is_ascii_whitespace() {
                after += 1;
            }
            return after + 1 < self.bytes.len()
                && self.bytes[after] == b'='
                && self.bytes[after + 1] == b'>';
        }
        false
    }

    fn parse_arrow(&mut self) -> Result<Ast, ParseError> {
        let start = self.i;
        let params = if self.peek() == b'(' {
            let close = match_paren(self.bytes, self.i)
                .ok_or_else(|| self.err_at(self.i, "unterminated parameter list"))?;
            let inner = &self.bytes[self.i + 1..close];
            self.i = close + 1;
            harvest_param_idents(inner)
        } else {
            vec![self.parse_ident()?]
        };
        self.skip_ws();
        // Optional TS return type annotation `: T`.
        if self.peek() == b':' {
            // Skim until top-level `=>`.
            let mut depth = 0i32;
            self.i += 1;
            while self.i + 1 < self.bytes.len() {
                let c = self.peek();
                match c {
                    b'(' | b'[' | b'<' | b'{' => depth += 1,
                    b')' | b']' | b'>' | b'}' => depth -= 1,
                    b'=' if depth == 0 && self.peek_at(1) == b'>' => break,
                    _ => {}
                }
                self.i += 1;
            }
        }
        self.expect("=>")?;
        self.skip_ws();
        let body = if self.peek() == b'{' {
            // Block body — capture verbatim so handler-complexity check
            // can still walk it byte-by-byte.
            let close = match_paren(self.bytes, self.i)
                .ok_or_else(|| self.err_at(self.i, "unterminated arrow block"))?;
            let body_src = self.slice(self.i, close + 1).to_string();
            self.i = close + 1;
            ArrowBody::Block(body_src)
        } else {
            ArrowBody::Expr(Box::new(self.parse_expr()?))
        };
        Ok(Ast::new(
            AstKind::Arrow { params, body },
            self.span_from(start),
        ))
    }
}

use crate::scan::{is_ident_cont, is_ident_start};

/// Match the opener at `bytes[pos]` to its closer. Returns the offset
/// of the closer, or `None` on imbalance / EOF. Tracks string and
/// template-literal depth so quoted closers don't fool the count.
pub(crate) fn match_paren(bytes: &[u8], pos: usize) -> Option<usize> {
    let (open, close) = match bytes.get(pos)? {
        b'(' => (b'(', b')'),
        b'{' => (b'{', b'}'),
        b'[' => (b'[', b']'),
        _ => return None,
    };
    let mut depth = 0i32;
    let mut i = pos;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = quote {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                quote = Some(c);
                i += 1;
            }
            c if c == open => {
                depth += 1;
                i += 1;
            }
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

fn harvest_param_idents(inner: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut skip = false;
    while i < inner.len() {
        let c = inner[i];
        if let Some(q) = in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
                i += 1;
            }
            b'(' | b'[' | b'{' | b'<' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' | b'}' | b'>' => {
                depth -= 1;
                i += 1;
            }
            b':' | b'=' if depth == 0 => {
                skip = true;
                i += 1;
            }
            b',' if depth == 0 => {
                skip = false;
                i += 1;
            }
            _ if !skip && is_ident_start(c) => {
                let s = i;
                while i < inner.len() && is_ident_cont(inner[i]) {
                    i += 1;
                }
                let w = std::str::from_utf8(&inner[s..i]).unwrap_or("");
                if !w.is_empty() && !is_keyword(w) {
                    out.push(w.to_string());
                }
                skip = true;
            }
            _ => i += 1,
        }
    }
    out
}

fn is_keyword(w: &str) -> bool {
    matches!(
        w,
        "if" | "else"
            | "return"
            | "let"
            | "const"
            | "var"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "typeof"
            | "instanceof"
            | "in"
            | "of"
            | "new"
            | "true"
            | "false"
            | "null"
            | "undefined"
            | "async"
            | "await"
            | "yield"
            | "function"
            | "class"
            | "this"
            | "super"
            | "void"
            | "delete"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Ast {
        parse_expression(s, 0).unwrap_or_else(|e| panic!("{} ({s})", e.message))
    }

    #[test]
    fn parses_ident() {
        let a = parse("count");
        assert!(matches!(a.kind, AstKind::Ident(ref s) if s == "count"));
    }

    #[test]
    fn parses_member_chain() {
        let a = parse("scope.foo.bar");
        let (head, tail) = a.member_chain().unwrap();
        assert_eq!(head, "scope");
        assert_eq!(tail, vec!["foo", "bar"]);
    }

    #[test]
    fn parses_call() {
        let a = parse("count()");
        match a.kind {
            AstKind::Call { args, .. } => assert!(args.is_empty()),
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn parses_compare_with_lt() {
        // The original OOM trigger.
        let a = parse("x < y");
        match a.kind {
            AstKind::Binary { op, .. } => assert_eq!(op, "<"),
            _ => panic!("expected Binary"),
        }
    }

    #[test]
    fn parses_ternary() {
        let a = parse("a ? b : c");
        assert!(matches!(a.kind, AstKind::Ternary { .. }));
    }

    #[test]
    fn parses_template_literal() {
        let a = parse("`hi ${name}`");
        match a.kind {
            AstKind::Template(parts) => {
                assert_eq!(parts.len(), 2);
                assert!(matches!(parts[0], TemplatePart::Static(_)));
                assert!(matches!(parts[1], TemplatePart::Expr(_)));
            }
            _ => panic!("expected Template"),
        }
    }

    #[test]
    fn parses_array_with_spread() {
        let a = parse("[1, ...rest, 2]");
        match a.kind {
            AstKind::Array(items) => assert_eq!(items.len(), 3),
            _ => panic!("expected Array"),
        }
    }

    #[test]
    fn parses_object_shorthand_and_spread() {
        let a = parse("{ x, y: 2, ...z }");
        match a.kind {
            AstKind::Object(entries) => assert_eq!(entries.len(), 3),
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn parses_optional_chain_call() {
        let a = parse("a?.b?.()");
        // outer node should be a Call with optional=true
        match a.kind {
            AstKind::Call { optional, .. } => assert!(optional),
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn parses_arrow_when_allowed() {
        let a = parse_expression_with_arrows("(e) => x.set(1)", 0).unwrap();
        match a.kind {
            AstKind::Arrow { params, body } => {
                assert_eq!(params, vec!["e"]);
                assert!(matches!(body, ArrowBody::Expr(_)));
            }
            _ => panic!("expected Arrow"),
        }
    }

    #[test]
    fn parses_arrow_block_body() {
        let a = parse_expression_with_arrows("() => { x.set(1); y.set(2) }", 0).unwrap();
        match a.kind {
            AstKind::Arrow { body, .. } => assert!(matches!(body, ArrowBody::Block(_))),
            _ => panic!("expected Arrow"),
        }
    }

    #[test]
    fn rejects_arrow_when_disallowed() {
        let err = parse_expression("(e) => x()", 0).unwrap_err();
        // The parser without arrow support reports "trailing input"
        // (the `=>` doesn't match any postfix continuation).
        assert!(err.message.contains("trailing") || err.message.contains("expected"));
    }

    #[test]
    fn rejects_empty() {
        let err = parse_expression("   ", 0).unwrap_err();
        assert!(err.message.contains("empty"));
    }

    #[test]
    fn span_is_absolute_with_offset() {
        let a = parse_expression("foo", 100).unwrap();
        assert_eq!(a.span.start, 100);
        assert_eq!(a.span.end, 103);
    }

    #[test]
    fn innermost_at_finds_ident() {
        let a = parse("scope.foo");
        // Cursor on `foo` (offset 6 within "scope.foo").
        let inner = a.innermost_at(6).unwrap();
        match &inner.kind {
            AstKind::Member { prop, .. } => assert_eq!(prop, "foo"),
            _ => panic!("expected Member, got {:?}", inner.kind),
        }
    }
}
