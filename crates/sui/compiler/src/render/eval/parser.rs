//! Recursive-descent parser for the renderable subset.
//!
//! Produces a small AST consumed by `exec`. Mirrors the grammar
//! enforced by `check/expr_subset.rs` — everything that passes the
//! check should parse here, and anything that doesn't parse here is
//! returned as `EvalError::ParseError` (caller falls back to marker-
//! only).

use super::EvalError;
use super::lex::{Tok, tokenize};

pub use super::ast::{BinOp, LogicalOp, Node, TplPart, UnaryOp};

pub fn parse_expr(src: &str) -> Result<Node, EvalError> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return Err(EvalError::ParseError("empty expression".into()));
    }
    let tokens = tokenize(trimmed)?;
    let mut p = Parser { toks: tokens, pos: 0 };
    let node = p.parse_ternary()?;
    if p.pos != p.toks.len() {
        return Err(EvalError::ParseError(format!(
            "trailing tokens after position {}",
            p.pos
        )));
    }
    Ok(node)
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn eat_punct(&mut self, p: &str) -> bool {
        if let Some(Tok::Punct(s)) = self.peek()
            && *s == p {
                self.pos += 1;
                return true;
            }
        false
    }

    fn expect_punct(&mut self, p: &str) -> Result<(), EvalError> {
        if self.eat_punct(p) {
            Ok(())
        } else {
            Err(EvalError::ParseError(format!(
                "expected `{p}` at token {}",
                self.pos
            )))
        }
    }

    fn parse_ternary(&mut self) -> Result<Node, EvalError> {
        let cond = self.parse_coalesce()?;
        if self.eat_punct("?") {
            let then = self.parse_ternary()?;
            self.expect_punct(":")?;
            let els = self.parse_ternary()?;
            return Ok(Node::Conditional {
                cond: Box::new(cond),
                then: Box::new(then),
                els: Box::new(els),
            });
        }
        Ok(cond)
    }

    fn parse_coalesce(&mut self) -> Result<Node, EvalError> {
        let mut l = self.parse_or()?;
        while self.eat_punct("??") {
            let r = self.parse_or()?;
            l = Node::Logical { op: LogicalOp::Coalesce, l: Box::new(l), r: Box::new(r) };
        }
        Ok(l)
    }

    fn parse_or(&mut self) -> Result<Node, EvalError> {
        let mut l = self.parse_and()?;
        while self.eat_punct("||") {
            let r = self.parse_and()?;
            l = Node::Logical { op: LogicalOp::Or, l: Box::new(l), r: Box::new(r) };
        }
        Ok(l)
    }

    fn parse_and(&mut self) -> Result<Node, EvalError> {
        let mut l = self.parse_eq()?;
        while self.eat_punct("&&") {
            let r = self.parse_eq()?;
            l = Node::Logical { op: LogicalOp::And, l: Box::new(l), r: Box::new(r) };
        }
        Ok(l)
    }

    fn parse_eq(&mut self) -> Result<Node, EvalError> {
        let mut l = self.parse_cmp()?;
        loop {
            let op = if self.eat_punct("===") { BinOp::EqStrict }
                else if self.eat_punct("!==") { BinOp::NeqStrict }
                else if self.eat_punct("==") { BinOp::Eq }
                else if self.eat_punct("!=") { BinOp::Neq }
                else { break };
            let r = self.parse_cmp()?;
            l = Node::Binary { op, l: Box::new(l), r: Box::new(r) };
        }
        Ok(l)
    }

    fn parse_cmp(&mut self) -> Result<Node, EvalError> {
        let mut l = self.parse_add()?;
        loop {
            let op = if self.eat_punct("<=") { BinOp::Le }
                else if self.eat_punct(">=") { BinOp::Ge }
                else if self.eat_punct("<") { BinOp::Lt }
                else if self.eat_punct(">") { BinOp::Gt }
                else { break };
            let r = self.parse_add()?;
            l = Node::Binary { op, l: Box::new(l), r: Box::new(r) };
        }
        Ok(l)
    }

    fn parse_add(&mut self) -> Result<Node, EvalError> {
        let mut l = self.parse_mul()?;
        loop {
            let op = if self.eat_punct("+") { BinOp::Add }
                else if self.eat_punct("-") { BinOp::Sub }
                else { break };
            let r = self.parse_mul()?;
            l = Node::Binary { op, l: Box::new(l), r: Box::new(r) };
        }
        Ok(l)
    }

    fn parse_mul(&mut self) -> Result<Node, EvalError> {
        let mut l = self.parse_unary()?;
        loop {
            let op = if self.eat_punct("*") { BinOp::Mul }
                else if self.eat_punct("/") { BinOp::Div }
                else if self.eat_punct("%") { BinOp::Mod }
                else { break };
            let r = self.parse_unary()?;
            l = Node::Binary { op, l: Box::new(l), r: Box::new(r) };
        }
        Ok(l)
    }

    fn parse_unary(&mut self) -> Result<Node, EvalError> {
        if self.eat_punct("!") {
            let e = self.parse_unary()?;
            return Ok(Node::Unary { op: UnaryOp::Not, expr: Box::new(e) });
        }
        if self.eat_punct("-") {
            let e = self.parse_unary()?;
            return Ok(Node::Unary { op: UnaryOp::Neg, expr: Box::new(e) });
        }
        if self.eat_punct("+") {
            let e = self.parse_unary()?;
            return Ok(Node::Unary { op: UnaryOp::Pos, expr: Box::new(e) });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Node, EvalError> {
        let mut node = self.parse_primary()?;
        loop {
            if self.eat_punct(".") {
                let prop = self.expect_ident()?;
                node = Node::Member { obj: Box::new(node), prop, optional: false };
            } else if self.eat_punct("?.") {
                node = self.parse_optional_chain(node)?;
            } else if self.eat_punct("[") {
                let idx = self.parse_ternary()?;
                self.expect_punct("]")?;
                node = Node::Index { obj: Box::new(node), idx: Box::new(idx), optional: false };
            } else if self.eat_punct("(") {
                let args = self.parse_args()?;
                node = Node::Call { callee: Box::new(node), args };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_optional_chain(&mut self, node: Node) -> Result<Node, EvalError> {
        if self.eat_punct("(") {
            let args = self.parse_args()?;
            return Ok(Node::Call { callee: Box::new(node), args });
        }
        if self.eat_punct("[") {
            let idx = self.parse_ternary()?;
            self.expect_punct("]")?;
            return Ok(Node::Index { obj: Box::new(node), idx: Box::new(idx), optional: true });
        }
        let prop = self.expect_ident()?;
        Ok(Node::Member { obj: Box::new(node), prop, optional: true })
    }

    fn parse_args(&mut self) -> Result<Vec<Node>, EvalError> {
        let mut args = Vec::new();
        if self.eat_punct(")") { return Ok(args); }
        loop {
            args.push(self.parse_ternary()?);
            if self.eat_punct(",") { continue; }
            self.expect_punct(")")?;
            break;
        }
        Ok(args)
    }

    fn expect_ident(&mut self) -> Result<String, EvalError> {
        match self.toks.get(self.pos).cloned() {
            Some(Tok::Ident(s)) => { self.pos += 1; Ok(s) }
            _ => Err(EvalError::ParseError(format!(
                "expected identifier at token {}", self.pos
            ))),
        }
    }

    fn parse_primary(&mut self) -> Result<Node, EvalError> {
        let tok = self.toks.get(self.pos).cloned()
            .ok_or_else(|| EvalError::ParseError("unexpected end of input".into()))?;
        match tok {
            Tok::Num(n) => { self.pos += 1; Ok(Node::Num(n)) }
            Tok::Str(s) => { self.pos += 1; Ok(Node::Str(s)) }
            Tok::Template(parts) => { self.pos += 1; Ok(Node::Template(parts)) }
            Tok::Ident(name) => {
                self.pos += 1;
                Ok(match name.as_str() {
                    "true" => Node::Bool(true),
                    "false" => Node::Bool(false),
                    "null" => Node::Null,
                    "undefined" => Node::Undefined,
                    _ => Node::Ident(name),
                })
            }
            Tok::Punct("(") => {
                self.pos += 1;
                let e = self.parse_ternary()?;
                self.expect_punct(")")?;
                Ok(e)
            }
            Tok::Punct("[") => { self.pos += 1; self.parse_array_lit() }
            Tok::Punct("{") => { self.pos += 1; self.parse_object_lit() }
            other => Err(EvalError::ParseError(format!(
                "unexpected token {other:?} at position {}", self.pos
            ))),
        }
    }

    fn parse_array_lit(&mut self) -> Result<Node, EvalError> {
        let mut items = Vec::new();
        if self.eat_punct("]") { return Ok(Node::Array(items)); }
        loop {
            items.push(self.parse_ternary()?);
            if self.eat_punct(",") { continue; }
            self.expect_punct("]")?;
            break;
        }
        Ok(Node::Array(items))
    }

    fn parse_object_lit(&mut self) -> Result<Node, EvalError> {
        let mut props: Vec<(String, Node)> = Vec::new();
        if self.eat_punct("}") { return Ok(Node::Object(props)); }
        loop {
            let key = self.parse_object_key()?;
            self.expect_punct(":")?;
            let val = self.parse_ternary()?;
            props.push((key, val));
            if self.eat_punct(",") { continue; }
            self.expect_punct("}")?;
            break;
        }
        Ok(Node::Object(props))
    }

    fn parse_object_key(&mut self) -> Result<String, EvalError> {
        match self.toks.get(self.pos).cloned() {
            Some(Tok::Ident(s)) => { self.pos += 1; Ok(s) }
            Some(Tok::Str(s)) => { self.pos += 1; Ok(s) }
            Some(Tok::Num(n)) => { self.pos += 1; Ok(n.to_string()) }
            _ => Err(EvalError::ParseError(format!(
                "expected object key at token {}", self.pos
            ))),
        }
    }
}
