//! AST for the constrained `{expr}` subset.
//!
//! Spans are byte ranges into the original `.sail` source — the parser
//! takes a `base_offset` so caller-side spans line up with diagnostics
//! emitted from anywhere else in the pipeline.

use crate::diagnostics::Span;

#[derive(Debug, Clone)]
pub struct Ast {
    pub kind: AstKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstKind {
    /// Numeric literal source text (preserved verbatim).
    Number(String),
    /// String literal — already unescaped.
    String(String),
    /// Template literal with interpolated parts.
    Template(Vec<TemplatePart>),
    Bool(bool),
    Null,
    Undefined,
    Ident(String),
    /// `obj.prop` / `obj?.prop`.
    Member { obj: Box<Ast>, prop: String, optional: bool },
    /// `obj[idx]` / `obj?.[idx]`.
    Index { obj: Box<Ast>, idx: Box<Ast>, optional: bool },
    /// `callee(args…)` / `callee?.(args…)`.
    Call { callee: Box<Ast>, args: Vec<Arg>, optional: bool },
    /// Prefix unary: `!`, `-`, `+`.
    Unary { op: String, arg: Box<Ast> },
    /// Arithmetic / comparison / nullish — anything that isn't `&&` / `||`.
    Binary { op: String, lhs: Box<Ast>, rhs: Box<Ast> },
    /// Short-circuit logical: `&&`, `||`.
    Logical { op: String, lhs: Box<Ast>, rhs: Box<Ast> },
    Ternary { cond: Box<Ast>, then: Box<Ast>, otherwise: Box<Ast> },
    Array(Vec<Arg>),
    Object(Vec<ObjEntry>),
    Arrow { params: Vec<String>, body: ArrowBody },
}

#[derive(Debug, Clone)]
pub enum ArrowBody {
    /// Expression body: `(e) => x()`.
    Expr(Box<Ast>),
    /// Block body — list of statement expressions: `(e) => { x.set(1); y.set(2) }`.
    /// The block is opaque past parse — tests still go through the legacy
    /// validator + handler-complexity check for now.
    Block(String),
}

#[derive(Debug, Clone)]
pub enum Arg {
    Expr(Ast),
    Spread(Ast),
}

#[derive(Debug, Clone)]
pub enum TemplatePart {
    Static(String),
    Expr(Ast),
}

#[derive(Debug, Clone)]
pub struct ObjEntry {
    pub key: ObjKey,
    /// `None` when the entry is shorthand or a spread (key carries the value).
    pub value: Option<Ast>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ObjKey {
    Ident(String),
    String(String),
    Spread(Ast),
}

impl Ast {
    pub fn new(kind: AstKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// True when this expression is a simple `Ident` reference.
    pub fn as_ident(&self) -> Option<&str> {
        match &self.kind {
            AstKind::Ident(name) => Some(name),
            _ => None,
        }
    }

    /// Walk `obj.a.b.c` chains — return the head ident plus every
    /// dotted segment in order. Returns `None` if the chain is not a
    /// pure `Ident.x.y…` shape (e.g. there's a call or index in the
    /// middle).
    pub fn member_chain(&self) -> Option<(&str, Vec<&str>)> {
        let mut tail: Vec<&str> = Vec::new();
        let mut cur = self;
        loop {
            match &cur.kind {
                AstKind::Member { obj, prop, .. } => {
                    tail.push(prop);
                    cur = obj;
                }
                AstKind::Ident(name) => {
                    tail.reverse();
                    return Some((name, tail));
                }
                _ => return None,
            }
        }
    }

    /// Visit every node in pre-order. The callback returns `false` to
    /// stop traversal early (used for cursor-position lookups in LSP).
    pub fn walk<F: FnMut(&Ast) -> bool>(&self, f: &mut F) -> bool {
        if !f(self) {
            return false;
        }
        match &self.kind {
            AstKind::Number(_)
            | AstKind::String(_)
            | AstKind::Bool(_)
            | AstKind::Null
            | AstKind::Undefined
            | AstKind::Ident(_) => true,
            AstKind::Template(parts) => parts.iter().all(|p| match p {
                TemplatePart::Static(_) => true,
                TemplatePart::Expr(e) => e.walk(f),
            }),
            AstKind::Member { obj, .. } => obj.walk(f),
            AstKind::Index { obj, idx, .. } => obj.walk(f) && idx.walk(f),
            AstKind::Call { callee, args, .. } => {
                if !callee.walk(f) {
                    return false;
                }
                args.iter().all(|a| match a {
                    Arg::Expr(e) | Arg::Spread(e) => e.walk(f),
                })
            }
            AstKind::Unary { arg, .. } => arg.walk(f),
            AstKind::Binary { lhs, rhs, .. } | AstKind::Logical { lhs, rhs, .. } => {
                lhs.walk(f) && rhs.walk(f)
            }
            AstKind::Ternary { cond, then, otherwise } => {
                cond.walk(f) && then.walk(f) && otherwise.walk(f)
            }
            AstKind::Array(items) => items.iter().all(|a| match a {
                Arg::Expr(e) | Arg::Spread(e) => e.walk(f),
            }),
            AstKind::Object(entries) => entries.iter().all(|entry| {
                let key_ok = match &entry.key {
                    ObjKey::Spread(e) => e.walk(f),
                    _ => true,
                };
                key_ok && entry.value.as_ref().map(|v| v.walk(f)).unwrap_or(true)
            }),
            AstKind::Arrow { body, .. } => match body {
                ArrowBody::Expr(e) => e.walk(f),
                ArrowBody::Block(_) => true,
            },
        }
    }

    /// Innermost node whose span contains `byte_offset`. None if the
    /// whole AST sits outside.
    pub fn innermost_at(&self, byte_offset: usize) -> Option<&Ast> {
        if byte_offset < self.span.start || byte_offset > self.span.end {
            return None;
        }
        // Search children first; fall back to self.
        let child_hit = match &self.kind {
            AstKind::Number(_)
            | AstKind::String(_)
            | AstKind::Bool(_)
            | AstKind::Null
            | AstKind::Undefined
            | AstKind::Ident(_) => None,
            AstKind::Template(parts) => parts.iter().find_map(|p| match p {
                TemplatePart::Static(_) => None,
                TemplatePart::Expr(e) => e.innermost_at(byte_offset),
            }),
            AstKind::Member { obj, .. } => obj.innermost_at(byte_offset),
            AstKind::Index { obj, idx, .. } => obj
                .innermost_at(byte_offset)
                .or_else(|| idx.innermost_at(byte_offset)),
            AstKind::Call { callee, args, .. } => callee
                .innermost_at(byte_offset)
                .or_else(|| {
                    args.iter().find_map(|a| match a {
                        Arg::Expr(e) | Arg::Spread(e) => e.innermost_at(byte_offset),
                    })
                }),
            AstKind::Unary { arg, .. } => arg.innermost_at(byte_offset),
            AstKind::Binary { lhs, rhs, .. } | AstKind::Logical { lhs, rhs, .. } => lhs
                .innermost_at(byte_offset)
                .or_else(|| rhs.innermost_at(byte_offset)),
            AstKind::Ternary { cond, then, otherwise } => cond
                .innermost_at(byte_offset)
                .or_else(|| then.innermost_at(byte_offset))
                .or_else(|| otherwise.innermost_at(byte_offset)),
            AstKind::Array(items) => items.iter().find_map(|a| match a {
                Arg::Expr(e) | Arg::Spread(e) => e.innermost_at(byte_offset),
            }),
            AstKind::Object(entries) => entries.iter().find_map(|entry| {
                let from_key = match &entry.key {
                    ObjKey::Spread(e) => e.innermost_at(byte_offset),
                    _ => None,
                };
                from_key.or_else(|| {
                    entry.value.as_ref().and_then(|v| v.innermost_at(byte_offset))
                })
            }),
            AstKind::Arrow { body, .. } => match body {
                ArrowBody::Expr(e) => e.innermost_at(byte_offset),
                ArrowBody::Block(_) => None,
            },
        };
        Some(child_hit.unwrap_or(self))
    }
}
