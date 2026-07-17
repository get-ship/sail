//! Handler-complexity check — condition (1) of the tier-1/tier-2 threshold.
//!
//! Emits a build-time error when an `on:event` handler body is a block
//! expression with more than one statement and the source has no controller
//! (the inline `<controller>` block, which owns complex handler logic).

use crate::diagnostics::CompileError;
use crate::ir::{AttrInstr, Instr, Program};

/// Entry point. Skipped entirely when the source has a controller (its inline
/// `<controller>` block) — the controller is responsible for complex handler
/// logic. `has_controller` is resolved in-memory; there is no paired-file probe.
pub fn check(program: &Program, has_controller: bool) -> Result<(), CompileError> {
    if has_controller {
        return Ok(());
    }
    check_instrs(&program.template)
}

fn check_instrs(instrs: &[Instr]) -> Result<(), CompileError> {
    for instr in instrs {
        check_instr(instr)?;
    }
    Ok(())
}

fn check_instr(instr: &Instr) -> Result<(), CompileError> {
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } | Instr::DynamicText { .. } => {}
        Instr::Element(el) => {
            for attr in &el.attrs {
                check_attr(attr)?;
            }
            check_instrs(&el.children)?;
        }
        Instr::If(i) => {
            for branch in &i.branches {
                check_instrs(&branch.body)?;
            }
            if let Some(eb) = &i.else_body {
                check_instrs(eb)?;
            }
        }
        Instr::For(f) => {
            check_instrs(&f.body)?;
            if let Some(eb) = &f.else_body {
                check_instrs(eb)?;
            }
        }
        Instr::Match(m) => {
            for case in &m.cases {
                check_instrs(&case.body)?;
            }
            if let Some(d) = &m.default {
                check_instrs(d)?;
            }
        }
        Instr::WhileLoading(wl) => check_instrs(&wl.body)?,
        Instr::WhileError(we) => check_instrs(&we.body)?,
    }
    Ok(())
}

fn check_attr(attr: &AttrInstr) -> Result<(), CompileError> {
    let AttrInstr::On { event, handler, .. } = attr else { return Ok(()); };
    if let Some(block_body) = arrow_block_body(&handler.source)
        && count_stmts(block_body) > 1 {
            return Err(CompileError::new(
                "multi-statement handler — move it into the `<controller>` block \
                 (name the handler there and reference it) for tier-2 work",
            )
            .with_span(handler.span)
            .with_category(format!("on:{event} handler")));
        }
    Ok(())
}

/// If `src` is an arrow function whose body is a `{ ... }` block, return
/// the text between the braces. Returns `None` for single-expression arrows.
fn arrow_block_body(src: &str) -> Option<&str> {
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;

    while i < bytes.len() {
        let c = bytes[i];
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
            b'(' | b'[' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' => {
                depth -= 1;
                i += 1;
            }
            b'=' if depth == 0 && i + 1 < bytes.len() && bytes[i + 1] == b'>' => {
                let after_arrow = src[i + 2..].trim_start();
                return if after_arrow.starts_with('{') {
                    extract_block(after_arrow)
                } else {
                    None // single-expression arrow
                };
            }
            _ => {
                i += 1;
            }
        }
    }
    None
}

/// Return the content between the outer `{` and its matching `}` of `s`.
fn extract_block(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    debug_assert_eq!(bytes[0], b'{');
    let mut depth = 1i32;
    let mut in_str: Option<u8> = None;
    let mut i = 1;
    while i < bytes.len() {
        let c = bytes[i];
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
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[1..i]);
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    None
}

/// Count top-level statements in block body text (content between `{` `}`).
///
/// Splits by `;` at depth 0 and counts non-empty segments. A single-statement
/// block has one non-empty segment; a trailing `;` produces an empty second
/// segment which is ignored.
fn count_stmts(body: &str) -> usize {
    let bytes = body.as_bytes();
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut count = 0usize;
    let mut seg_start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        let c = bytes[i];
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
            b'(' | b'[' | b'{' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' | b'}' => {
                depth -= 1;
                i += 1;
            }
            b';' if depth == 0 => {
                if !body[seg_start..i].trim().is_empty() {
                    count += 1;
                }
                seg_start = i + 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    if !body[seg_start..].trim().is_empty() {
        count += 1;
    }
    count
}
