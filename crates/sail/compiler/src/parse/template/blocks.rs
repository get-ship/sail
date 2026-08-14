//! Block parsers for `<if>`, `<for>`, `<match>`, `<when>`.
//!
//! Arms use brace form `{pattern as binding => body}` inside the block.
//! `for_header.rs` owns header tokenization (`X of items key={…}`).

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;
use crate::parse::expr;
use crate::parse::html;

use super::ParseResult;
use super::tokenize::{Discriminant, Token, TokenKind};

mod for_header;
use for_header::parse_for_header;

const RULE_ID: &str = "sail-template-block";
/// Callback that re-enters the top-level parser.
pub(crate) struct ParseScope {
    #[allow(clippy::type_complexity)]
    pub body: fn(&[Token], usize, &[Discriminant]) -> Result<ParseResult, CompileError>,
}

pub(crate) struct InstrResult {
    pub instr: Instr,
    pub end: usize,
}

const BODY_THEN_ARMS: &[Discriminant] = &[Discriminant::Arm, Discriminant::CloseIf];
const FOR_BODY_TERMS: &[Discriminant] = &[Discriminant::Arm, Discriminant::CloseFor];
const WHEN_BODY_TERMS: &[Discriminant] = &[Discriminant::Arm, Discriminant::CloseWhen];

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
    expr::validate_at(&head_src, false, "<if>", head_span)?;
    let mut branches: Vec<IfBranch> = Vec::new();
    let mut else_body: Option<Vec<Instr>> = None;

    // Free children before the first arm = then-body of the opening cond.
    let mut i = start + 1;
    let then_body = (scope.body)(tokens, i, BODY_THEN_ARMS)?;
    branches.push(IfBranch {
        cond: Expr::new(head_src, head_span, false),
        body: then_body.instrs,
    });
    i = then_body.end;

    while let Some(t) = tokens.get(i) {
        match &t.kind {
            TokenKind::CloseIf => {
                i += 1;
                break;
            }
            TokenKind::Text(s) if s.trim().is_empty() => i += 1,
            TokenKind::Arm { head, body_src } => {
                let head_trim = head.trim();
                if head_trim == "else" {
                    if else_body.is_some() {
                        return Err(CompileError::new("<if>: duplicate {else => …} arm")
                            .with_span(t.span)
                            .with_category(RULE_ID));
                    }
                    else_body = Some(parse_arm_body(body_src, t.span)?);
                    i += 1;
                    while let Some(n) = tokens.get(i) {
                        match &n.kind {
                            TokenKind::Text(s) if s.trim().is_empty() => i += 1,
                            TokenKind::CloseIf => {
                                i += 1;
                                return Ok(InstrResult {
                                    instr: Instr::If(IfInstr {
                                        branches,
                                        else_body,
                                        span: head_span,
                                    }),
                                    end: i,
                                });
                            }
                            _ => {
                                return Err(CompileError::new(
                                    "<if>: {else => …} must be the last arm before </if>",
                                )
                                .with_span(n.span)
                                .with_category(RULE_ID));
                            }
                        }
                    }
                    return Err(CompileError::new("missing </if>")
                        .with_span(head_span)
                        .with_category(RULE_ID));
                }
                if let Some(cond_src) = strip_else_if(head_trim) {
                    if else_body.is_some() {
                        return Err(CompileError::new("<if>: {else-if} cannot follow {else}")
                            .with_span(t.span)
                            .with_category(RULE_ID));
                    }
                    expr::validate_at(cond_src, false, "{else-if}", t.span)?;
                    branches.push(IfBranch {
                        cond: Expr::new(cond_src.to_string(), t.span, false),
                        body: parse_arm_body(body_src, t.span)?,
                    });
                    i += 1;
                    continue;
                }
                return Err(CompileError::new(format!(
                    "<if>: unknown arm {{{head_trim} => …}}; use {{else-if {{cond}} => …}} or {{else => …}}"
                ))
                .with_span(t.span).with_category(RULE_ID));
            }
            other => {
                return Err(CompileError::new(format!(
                    "<if>: unexpected {}; use {{else-if}}/{{else}} arms or </if>",
                    other.label()
                ))
                .with_span(t.span)
                .with_category(RULE_ID));
            }
        }
    }
    Ok(InstrResult {
        instr: Instr::If(IfInstr {
            branches,
            else_body,
            span: head_span,
        }),
        end: i,
    })
}

fn strip_else_if(head: &str) -> Option<&str> {
    let h = head.trim();
    let rest = if let Some(r) = h.strip_prefix("else-if") {
        r
    } else {
        h.strip_prefix("else if")?
    };
    let rest = rest.trim();
    if rest.starts_with('{') && rest.ends_with('}') && rest.len() >= 2 {
        return Some(rest[1..rest.len() - 1].trim());
    }
    if !rest.is_empty() {
        return Some(rest);
    }
    None
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
    let body = (scope.body)(tokens, i, FOR_BODY_TERMS)?;
    i = body.end;
    let mut else_body: Option<Vec<Instr>> = None;
    loop {
        match tokens.get(i).map(|t| &t.kind) {
            Some(TokenKind::Text(s)) if s.trim().is_empty() => i += 1,
            Some(TokenKind::Arm { head, body_src }) => {
                if head.trim() != "else" {
                    return Err(CompileError::new(format!(
                        "<for>: only {{else => …}} arm is allowed, got {{{} => …}}",
                        head.trim()
                    ))
                    .with_span(tokens[i].span)
                    .with_category(RULE_ID));
                }
                else_body = Some(parse_arm_body(body_src, tokens[i].span)?);
                i += 1;
            }
            Some(TokenKind::CloseFor) => {
                i += 1;
                break;
            }
            Some(other) => {
                return Err(CompileError::new(format!(
                    "<for>: unexpected {} before </for>",
                    other.label()
                ))
                .with_span(tokens[i].span)
                .with_category(RULE_ID));
            }
            None => {
                return Err(CompileError::new("missing </for>")
                    .with_span(head_span)
                    .with_category(RULE_ID));
            }
        }
    }
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

pub(crate) fn parse_when(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenWhen { src } => src.clone(),
        _ => unreachable!(),
    };
    let token = head_src.trim();
    let media_query = resolve_when_token(token, head_span)?;
    let mut i = start + 1;
    let body = (scope.body)(tokens, i, WHEN_BODY_TERMS)?;
    i = body.end;
    let mut else_body: Option<Vec<Instr>> = None;
    loop {
        match tokens.get(i).map(|t| &t.kind) {
            Some(TokenKind::Text(s)) if s.trim().is_empty() => {
                i += 1;
                continue;
            }
            Some(TokenKind::Arm { head, body_src }) => {
                if head.trim() != "else" {
                    return Err(CompileError::new(format!(
                        "<when>: only {{else => …}} arm is allowed, got {{{} => …}}",
                        head.trim()
                    ))
                    .with_span(tokens[i].span)
                    .with_category(RULE_ID));
                }
                else_body = Some(parse_arm_body(body_src, tokens[i].span)?);
                i += 1;
                continue;
            }
            Some(TokenKind::CloseWhen) => {
                i += 1;
                break;
            }
            Some(other) => {
                return Err(CompileError::new(format!(
                    "<when>: unexpected {} before </when>",
                    other.label()
                ))
                .with_span(tokens[i].span)
                .with_category(RULE_ID));
            }
            None => {
                return Err(CompileError::new("missing </when>")
                    .with_span(head_span)
                    .with_category(RULE_ID));
            }
        }
    }
    Ok(InstrResult {
        instr: Instr::When(WhenInstr {
            token: token.to_string(),
            media_query: media_query.to_string(),
            body: body.instrs,
            else_body,
            span: head_span,
        }),
        end: i,
    })
}

fn resolve_when_token(token: &str, span: Span) -> Result<&'static str, CompileError> {
    let list = crate::breakpoints::breakpoint_token_list();
    if token.is_empty() {
        return Err(
            CompileError::new(format!("<when> requires a theme breakpoint token: {list}"))
                .with_span(span)
                .with_category(RULE_ID),
        );
    }
    if token.chars().any(|c| c.is_ascii_digit())
        || token.contains('(')
        || token.contains(':')
        || token.contains(' ')
    {
        return Err(CompileError::new(format!(
            "<when> rejects raw media queries and pixel values; use a theme breakpoint token: {list} (got {token:?})"
        ))
        .with_span(span).with_category(RULE_ID));
    }
    crate::breakpoints::breakpoint_media(token).ok_or_else(|| {
        CompileError::new(format!(
            "unknown breakpoint token {token:?} in <when>; theme scale is: {list}"
        ))
        .with_span(span)
        .with_category(RULE_ID)
    })
}

pub(crate) fn parse_match(
    tokens: &[Token],
    start: usize,
    _scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenMatch { src } => src.clone(),
        _ => unreachable!(),
    };
    expr::validate_at(&head_src, false, "<match>", head_span)?;
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
            TokenKind::Arm { head, body_src } => {
                let head_trim = head.trim();
                if head_trim == "else" {
                    if default.is_some() {
                        return Err(CompileError::new("<match>: duplicate {else => …} arm")
                            .with_span(t.span)
                            .with_category(RULE_ID));
                    }
                    default = Some(parse_arm_body(body_src, t.span)?);
                    i += 1;
                    continue;
                }
                let (test, binding) = parse_match_arm_head(head_trim, t.span)?;
                let body = parse_arm_body(body_src, t.span)?;
                cases.push(MatchCase {
                    test,
                    binding,
                    body,
                });
                i += 1;
            }
            TokenKind::Text(s) if s.trim().is_empty() => i += 1,
            other => {
                return Err(CompileError::new(format!(
                    "unexpected token in <match>: {}; arms are {{pattern as binding => …}}",
                    other.label()
                ))
                .with_span(t.span)
                .with_category(RULE_ID));
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

fn parse_match_arm_head(
    head: &str,
    span: Span,
) -> Result<(MatchTest, Option<String>), CompileError> {
    let head = head.trim();
    if head.is_empty() {
        return Err(CompileError::new("<match> arm needs a pattern before =>")
            .with_span(span)
            .with_category(RULE_ID));
    }
    if let Some(as_at) = find_top_level_as(head) {
        let pat = head[..as_at].trim();
        let binding = head[as_at + 4..].trim();
        if !is_identifier(binding) {
            return Err(CompileError::new(format!(
                "<match> arm binding must be an identifier, got {binding:?}"
            ))
            .with_span(span)
            .with_category(RULE_ID));
        }
        return Ok((pattern_to_test(pat, span)?, Some(binding.to_string())));
    }
    Ok((pattern_to_test(head, span)?, None))
}

fn pattern_to_test(pat: &str, span: Span) -> Result<MatchTest, CompileError> {
    let pat = pat.trim();
    if is_identifier(pat) {
        return Ok(MatchTest::Kind(pat.to_string()));
    }
    expr::validate_at(pat, false, "<match> arm pattern", span)?;
    Ok(MatchTest::Expr(Expr::new(pat.to_string(), span, false)))
}

fn find_top_level_as(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    let mut i = 0usize;
    while i + 4 <= bytes.len() {
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
        if depth == 0 && &s[i..i + 4] == " as " {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn parse_arm_body(body_src: &str, span: Span) -> Result<Vec<Instr>, CompileError> {
    let src = body_src.trim();
    if src.is_empty() {
        return Ok(Vec::new());
    }
    let nodes = html::parse_fragment(src).map_err(|e| {
        CompileError::new(format!("arm body parse error: {}", e.message))
            .with_span(span)
            .with_category(RULE_ID)
    })?;
    super::compile_children(&nodes).map_err(|e| {
        let mut err = CompileError::new(e.message.clone())
            .with_category(RULE_ID)
            .with_category(e.category.clone().unwrap_or_else(|| RULE_ID.to_string()));
        if let Some(s) = e.span {
            err = err.with_span(s);
        } else {
            err = err.with_span(span);
        }
        err
    })
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}
