//! Block parsers for `{#if}`, `{#for}`, `{#match}`, `{#while-loading}`,
//! `{#while-error}`. Mirrors `packages/sui/dom/src/template/compile-blocks.ts`.
//!
//! `for_header.rs` owns the gnarly header tokenization
//! (`X of items key={…}`) so this file stays under the line cap.

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;
use crate::parse::expr;

use super::tokenize::{Discriminant, Token, TokenKind};
use super::ParseResult;

mod async_block;
mod for_header;
pub(crate) use async_block::parse_async;
use for_header::parse_for_header;

/// Callback that re-enters the top-level parser. Avoids a circular
/// module dependency between blocks and the recursive parse driver.
pub(crate) struct ParseScope {
    #[allow(clippy::type_complexity)]
    pub body: fn(&[Token], usize, &[Discriminant]) -> Result<ParseResult, CompileError>,
}

pub(crate) struct InstrResult {
    pub instr: Instr,
    pub end: usize,
}

/// Multi-instr result for blocks that desugar to >1 top-level instr.
pub(crate) struct InstrsResult {
    pub instrs: Vec<Instr>,
    pub end: usize,
}

const WHILE_LOADING_TERMS: &[Discriminant] = &[Discriminant::CloseWhileLoading];
const WHILE_ERROR_TERMS: &[Discriminant] = &[Discriminant::CloseWhileError];
const ASYNC_TERMS: &[Discriminant] = &[
    Discriminant::AsyncLoading,
    Discriminant::AsyncError,
    Discriminant::AsyncOk,
    Discriminant::CloseAsync,
];

const IF_TERMS: &[Discriminant] = &[
    Discriminant::ElseIf,
    Discriminant::Else,
    Discriminant::CloseIf,
];
const FOR_TERMS: &[Discriminant] = &[Discriminant::Else, Discriminant::CloseFor];
const MATCH_TERMS: &[Discriminant] = &[
    Discriminant::Case,
    Discriminant::Default,
    Discriminant::CloseMatch,
];

pub(crate) fn parse_if(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenIf { src } => src.clone(),
        _ => unreachable!(),
    };
    expr::validate_at(&head_src, false, "{#if}", head_span)?;
    let mut branches: Vec<IfBranch> = Vec::new();
    let mut else_body: Option<Vec<Instr>> = None;
    let mut cur_cond = Expr::new(head_src, head_span, false);
    let mut i = start + 1;
    loop {
        let body = (scope.body)(tokens, i, IF_TERMS)?;
        branches.push(IfBranch { cond: cur_cond.clone(), body: body.instrs });
        i = body.end;
        let term = tokens
            .get(i)
            .ok_or_else(|| CompileError::new("missing {/if}").with_span(head_span))?;
        match &term.kind {
            TokenKind::CloseIf => {
                i += 1;
                break;
            }
            TokenKind::ElseIf { src } => {
                expr::validate_at(src, false, "{:else if}", term.span)?;
                cur_cond = Expr::new(src.clone(), term.span, false);
                i += 1;
            }
            TokenKind::Else => {
                i += 1;
                let eb = (scope.body)(tokens, i, &[Discriminant::CloseIf])?;
                else_body = Some(eb.instrs);
                i = eb.end;
                let closer = tokens.get(i).ok_or_else(|| {
                    CompileError::new("missing {/if}").with_span(head_span)
                })?;
                if !matches!(closer.kind, TokenKind::CloseIf) {
                    return Err(CompileError::new("missing {/if}").with_span(head_span));
                }
                i += 1;
                break;
            }
            _ => unreachable!(),
        }
    }
    Ok(InstrResult {
        instr: Instr::If(IfInstr { branches, else_body, span: head_span }),
        end: i,
    })
}

pub(crate) fn parse_for(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenFor { src } => src.clone(),
        _ => unreachable!(),
    };
    let header = parse_for_header(&head_src, head_span)?;
    let mut i = start + 1;
    let body = (scope.body)(tokens, i, FOR_TERMS)?;
    i = body.end;
    let mut else_body: Option<Vec<Instr>> = None;
    if let Some(t) = tokens.get(i)
        && matches!(t.kind, TokenKind::Else) {
            i += 1;
            let eb = (scope.body)(tokens, i, &[Discriminant::CloseFor])?;
            else_body = Some(eb.instrs);
            i = eb.end;
        }
    let closer = tokens
        .get(i)
        .ok_or_else(|| CompileError::new("missing {/for}").with_span(head_span))?;
    if !matches!(closer.kind, TokenKind::CloseFor) {
        return Err(CompileError::new("missing {/for}").with_span(head_span));
    }
    i += 1;
    Ok(InstrResult {
        instr: Instr::For(ForInstr {
            var_name: header.var_name,
            index_name: header.index_name,
            iterable: header.iterable,
            key: header.key,
            body: body.instrs,
            else_body,
            span: head_span,
        }),
        end: i,
    })
}

pub(crate) fn parse_while_loading(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenWhileLoading { src } => src.clone(),
        _ => unreachable!(),
    };
    let bind = parse_of_binding(&head_src, head_span)?;
    let mut i = start + 1;
    let body = (scope.body)(tokens, i, WHILE_LOADING_TERMS)?;
    i = body.end;
    expect_close(tokens, i, TokenKind::CloseWhileLoading, "{/while-loading}", head_span)?;
    i += 1;
    Ok(InstrResult {
        instr: Instr::WhileLoading(WhileLoadingInstr { bind, body: body.instrs, span: head_span }),
        end: i,
    })
}

pub(crate) fn parse_while_error(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenWhileError { src } => src.clone(),
        _ => unreachable!(),
    };
    let (error_var, bind) = parse_error_of_binding(&head_src, head_span)?;
    let mut i = start + 1;
    let body = (scope.body)(tokens, i, WHILE_ERROR_TERMS)?;
    i = body.end;
    expect_close(tokens, i, TokenKind::CloseWhileError, "{/while-error}", head_span)?;
    i += 1;
    Ok(InstrResult {
        instr: Instr::WhileError(WhileErrorInstr { bind, error_var, body: body.instrs, span: head_span }),
        end: i,
    })
}

/// Parse `of <name>` from the header of `{#while-loading of <name>}`.
fn parse_of_binding(src: &str, span: Span) -> Result<String, CompileError> {
    let s = src.trim();
    let rest = s.strip_prefix("of").ok_or_else(|| {
        CompileError::new("{#while-loading} requires `of <signal>` (e.g. `{#while-loading of overview}`)")
            .with_span(span)
    })?;
    let name = rest.trim();
    if name.is_empty() || !is_identifier(name) {
        return Err(CompileError::new(
            format!("{{#while-loading}}: expected signal name after `of`, got {:?}", name)
        ).with_span(span));
    }
    Ok(name.to_string())
}

/// Parse `<err> of <name>` from the header of `{#while-error err of <name>}`.
fn parse_error_of_binding(src: &str, span: Span) -> Result<(String, String), CompileError> {
    let s = src.trim();
    let err_msg = || CompileError::new(
        "{#while-error} requires `<errVar> of <signal>` (e.g. `{#while-error err of overview}`)"
    ).with_span(span);
    let space = s.find(char::is_whitespace).ok_or_else(err_msg)?;
    let error_var = s[..space].trim().to_string();
    let rest = s[space..].trim();
    let name_part = rest.strip_prefix("of").ok_or_else(err_msg)?;
    let bind = name_part.trim();
    if error_var.is_empty() || !is_identifier(&error_var) {
        return Err(err_msg());
    }
    if bind.is_empty() || !is_identifier(bind) {
        return Err(err_msg());
    }
    Ok((error_var, bind.to_string()))
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn expect_close(
    tokens: &[Token],
    i: usize,
    expected: TokenKind,
    label: &'static str,
    head_span: Span,
) -> Result<(), CompileError> {
    let tok = tokens.get(i).ok_or_else(|| {
        CompileError::new(format!("missing {label}")).with_span(head_span)
    })?;
    if tok.kind.discriminant() != expected.discriminant() {
        return Err(CompileError::new(format!("missing {label}")).with_span(head_span));
    }
    Ok(())
}

pub(crate) fn parse_match(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenMatch { src } => src.clone(),
        _ => unreachable!(),
    };
    expr::validate_at(&head_src, false, "{#match}", head_span)?;
    let discriminant = Expr::new(head_src, head_span, false);
    let mut i = start + 1;
    while let Some(t) = tokens.get(i) {
        match &t.kind {
            TokenKind::Text(s) if s.trim().is_empty() => i += 1,
            _ => break,
        }
    }
    let mut cases: Vec<MatchCase> = Vec::new();
    let mut default: Option<Vec<Instr>> = None;
    while let Some(t) = tokens.get(i) {
        match &t.kind {
            TokenKind::CloseMatch => {
                i += 1;
                break;
            }
            TokenKind::Case { src } => {
                expr::validate_at(src, false, "{:case}", t.span)?;
                let test = Expr::new(src.clone(), t.span, false);
                i += 1;
                let body = (scope.body)(tokens, i, MATCH_TERMS)?;
                cases.push(MatchCase { test, body: body.instrs });
                i = body.end;
            }
            TokenKind::Default => {
                i += 1;
                let body = (scope.body)(tokens, i, &[Discriminant::CloseMatch])?;
                default = Some(body.instrs);
                i = body.end;
            }
            _ => {
                return Err(CompileError::new(format!(
                    "unexpected token in {{#match}}: {}",
                    t.kind.label()
                ))
                .with_span(t.span));
            }
        }
    }
    Ok(InstrResult {
        instr: Instr::Match(MatchInstr {
            discriminant,
            cases,
            default,
            span: head_span,
        }),
        end: i,
    })
}
