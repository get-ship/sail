//! `{#async X as v}…{:loading}…{:error err}…{:ok}…{/async}`.
//!
//! Desugars at parse-time into the three existing constructs:
//!
//! ```text
//! {#while-loading of X}…{/while-loading}
//! {#while-error err of X}…{/while-error}
//! {#if X().kind === 'ok'}
//!   {let v = X().value in …}
//! {/if}
//! ```
//!
//! `{:ok}` is required. `{:loading}` and `{:error}` are optional and
//! fall back to `<sui-spinner size="md" centered>` and
//! `<sui-error-state>{err.message}</sui-error-state>`. `as <var>` is
//! optional and emits an `Instr::Let` snapshot binding around the
//! `{:ok}` body.

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;

use super::super::tokenize::{Token, TokenKind};
use super::{InstrsResult, ParseScope, ASYNC_TERMS};

pub(crate) fn parse_async(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrsResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenAsync { src } => src.clone(),
        _ => unreachable!(),
    };
    let header = parse_async_header(&head_src, head_span)?;

    let mut i = start + 1;
    let mut loading_body: Option<Vec<Instr>> = None;
    let mut error_body: Option<Vec<Instr>> = None;
    let mut error_var = String::from("error");
    let mut ok_body: Option<Vec<Instr>> = None;

    // Skip leading whitespace before the first branch marker.
    while let Some(t) = tokens.get(i) {
        if let TokenKind::Text(s) = &t.kind
            && s.trim().is_empty()
        {
            i += 1;
            continue;
        }
        break;
    }

    while let Some(t) = tokens.get(i) {
        match &t.kind {
            TokenKind::CloseAsync => {
                i += 1;
                break;
            }
            TokenKind::AsyncLoading => {
                i += 1;
                let body = (scope.body)(tokens, i, ASYNC_TERMS)?;
                loading_body = Some(body.instrs);
                i = body.end;
            }
            TokenKind::AsyncError { src } => {
                let v = src.trim();
                if !v.is_empty() {
                    if !is_identifier(v) {
                        return Err(CompileError::new(format!(
                            "{{:error}}: bad variable name {v:?}"
                        ))
                        .with_span(t.span));
                    }
                    error_var = v.to_string();
                }
                i += 1;
                let body = (scope.body)(tokens, i, ASYNC_TERMS)?;
                error_body = Some(body.instrs);
                i = body.end;
            }
            TokenKind::AsyncOk => {
                i += 1;
                let body = (scope.body)(tokens, i, ASYNC_TERMS)?;
                ok_body = Some(body.instrs);
                i = body.end;
            }
            _ => {
                return Err(CompileError::new(format!(
                    "unexpected token in {{#async}}: {}",
                    t.kind.label()
                ))
                .with_span(t.span));
            }
        }
    }

    let ok_body = ok_body.ok_or_else(|| {
        CompileError::new("{#async} requires {:ok} branch").with_span(head_span)
    })?;

    // Wrap the ok body in a `let` instr if `as v` was given.
    let ok_wrapped: Vec<Instr> = if let Some(name) = header.as_var.clone() {
        let value_src = format!("{}().value", header.signal);
        let value_expr = Expr::new(value_src, head_span, false);
        vec![Instr::Let(LetInstr {
            name,
            value: value_expr,
            body: ok_body,
            span: head_span,
        })]
    } else {
        ok_body
    };

    let if_kind_eq_ok_src = format!("{}().kind === 'ok'", header.signal);
    let if_branch = IfBranch {
        cond: Expr::new(if_kind_eq_ok_src, head_span, false),
        body: ok_wrapped,
    };

    let out: Vec<Instr> = vec![
        Instr::WhileLoading(WhileLoadingInstr {
            bind: header.signal.clone(),
            body: loading_body.unwrap_or_else(default_loading_body),
            span: head_span,
        }),
        Instr::WhileError(WhileErrorInstr {
            bind: header.signal.clone(),
            error_var: error_var.clone(),
            body: error_body.unwrap_or_else(|| default_error_body(&error_var, head_span)),
            span: head_span,
        }),
        Instr::If(IfInstr {
            branches: vec![if_branch],
            else_body: None,
            span: head_span,
        }),
    ];

    Ok(InstrsResult { instrs: out, end: i })
}

struct AsyncHeader {
    signal: String,
    as_var: Option<String>,
}

fn parse_async_header(src: &str, span: Span) -> Result<AsyncHeader, CompileError> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return Err(CompileError::new(
            "{#async} requires a signal name (e.g. `{#async overview as v}`)",
        )
        .with_span(span));
    }
    let (signal, as_var) = match find_as_keyword(trimmed) {
        Some(idx) => {
            let signal = trimmed[..idx].trim().to_string();
            let var = trimmed[idx + 4..].trim().to_string();
            if !is_identifier(&var) {
                return Err(CompileError::new(format!(
                    "{{#async}}: bad `as` variable: {var:?}"
                ))
                .with_span(span));
            }
            (signal, Some(var))
        }
        None => (trimmed.to_string(), None),
    };
    if !is_identifier(&signal) {
        return Err(CompileError::new(format!(
            "{{#async}}: expected bare signal name, got {signal:?}"
        ))
        .with_span(span));
    }
    Ok(AsyncHeader { signal, as_var })
}

/// Locate the ` as ` keyword (surrounded by ASCII whitespace) in a header.
/// Skips occurrences inside identifiers (e.g. `cas_var` shouldn't match).
fn find_as_keyword(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if &bytes[i..i + 4] == b" as " {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn default_loading_body() -> Vec<Instr> {
    vec![Instr::Element(ElementInstr {
        tag: "sui-spinner".to_string(),
        attrs: vec![
            AttrInstr::Static {
                name: "size".to_string(),
                value: "md".to_string(),
                span: Span::new(0, 0),
            },
            AttrInstr::Static {
                name: "centered".to_string(),
                value: "".to_string(),
                span: Span::new(0, 0),
            },
        ],
        children: vec![],
        span: Span::new(0, 0),
    })]
}

fn default_error_body(error_var: &str, span: Span) -> Vec<Instr> {
    vec![Instr::Element(ElementInstr {
        tag: "sui-error-state".to_string(),
        attrs: vec![],
        children: vec![Instr::DynamicText {
            expr: Expr::new(format!("{error_var}.message"), span, false),
        }],
        span,
    })]
}

