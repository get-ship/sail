//! Template grammar scan.
//!
//! Mirrors `packages/sail/dom/src/template/{tokenize,compile,
//! compile-blocks}.ts`. The flow:
//!
//!   parse children of <root> → flat token list
//!     → recursive parse into `Instr` tree
//!     → element children recurse
//!     → attribute classification (see `attrs::compile_attrs`)

use crate::diagnostics::CompileError;
use crate::ir::*;
use crate::parse::{expr, html, html::Node};

mod attrs;
mod blocks;
mod control_elements;
mod tokenize;

pub use attrs::compile_attrs;
use blocks::ParseScope;
pub(crate) use tokenize::{Discriminant, Token, TokenKind, tokenize_children};

/// Compile a flat list of top-level template nodes.
pub fn compile_children(parent_children: &[Node]) -> Result<Vec<Instr>, CompileError> {
    let tokens = tokenize_children(parent_children);
    let result = parse_tokens(&tokens, 0, &[])?;
    Ok(result.instrs)
}

pub(crate) struct ParseResult {
    pub instrs: Vec<Instr>,
    pub end: usize,
}

pub(crate) fn parse_tokens(
    tokens: &[Token],
    start: usize,
    terminators: &[Discriminant],
) -> Result<ParseResult, CompileError> {
    let mut instrs = Vec::new();
    let mut i = start;
    while i < tokens.len() {
        let tok = &tokens[i];
        if terminators.contains(&tok.kind.discriminant()) {
            return Ok(ParseResult { instrs, end: i });
        }
        match &tok.kind {
            TokenKind::Text(s) => {
                if !s.is_empty() {
                    instrs.push(Instr::StaticText {
                        text: s.clone(),
                        span: tok.span,
                    });
                }
                i += 1;
            }
            TokenKind::Expr { src } => {
                expr::validate_at(src, false, "{expr}", tok.span)?;
                instrs.push(Instr::DynamicText {
                    expr: Expr::new(src.clone(), tok.span, false),
                });
                i += 1;
            }
            TokenKind::Element(node) => {
                instrs.push(Instr::Element(compile_element(node)?));
                i += 1;
            }
            TokenKind::OpenIf { .. } => {
                let scope = make_scope();
                let r = blocks::parse_if(tokens, i, &scope)?;
                instrs.push(r.instr);
                i = r.end;
            }
            TokenKind::OpenFor { .. } => {
                let scope = make_scope();
                let r = blocks::parse_for(tokens, i, &scope)?;
                instrs.push(r.instr);
                i = r.end;
            }
            TokenKind::OpenMatch { .. } => {
                let scope = make_scope();
                let r = blocks::parse_match(tokens, i, &scope)?;
                instrs.push(r.instr);
                i = r.end;
            }
            TokenKind::OpenWhileLoading { .. } => {
                let scope = make_scope();
                let r = blocks::parse_while_loading(tokens, i, &scope)?;
                instrs.push(r.instr);
                i = r.end;
            }
            TokenKind::OpenWhileError { .. } => {
                let scope = make_scope();
                let r = blocks::parse_while_error(tokens, i, &scope)?;
                instrs.push(r.instr);
                i = r.end;
            }
            TokenKind::ElseIf { .. }
            | TokenKind::Else
            | TokenKind::Case { .. }
            | TokenKind::Default
            | TokenKind::CloseIf
            | TokenKind::CloseFor
            | TokenKind::CloseMatch
            | TokenKind::CloseWhileLoading
            | TokenKind::CloseWhileError => {
                return Err(
                    CompileError::new(format!("unexpected token: {}", tok.kind.label()))
                        .with_span(tok.span),
                );
            }
        }
    }
    Ok(ParseResult { instrs, end: i })
}

fn make_scope() -> ParseScope {
    ParseScope {
        body: |tokens, start, terminators| parse_tokens(tokens, start, terminators),
    }
}

fn compile_element(el: &html::Element) -> Result<ElementInstr, CompileError> {
    let tag_lower = el.tag.as_str();
    let loc_prefix = format!("<{tag_lower}>");
    let attrs = compile_attrs(el, &loc_prefix)?;
    let children = compile_children(&el.children)?;
    Ok(ElementInstr {
        tag: el.tag.clone(),
        attrs,
        children,
        span: el.span,
    })
}
