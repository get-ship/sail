//! Machine-readable auto-fix plans for known lint rewrites.

use serde::{Deserialize, Serialize};

use super::node::{EditNode, build_node_index};
use super::ops::EditOp;
use std::collections::BTreeMap;

/// One suggested (or applied) fix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixPlan {
    pub rule: String,
    pub line: usize,
    pub col: usize,
    pub message: String,
    /// Ops that implement the rewrite (may be empty when only text patch).
    pub ops: Vec<EditOp>,
    /// Direct source rewrite when ops alone are awkward (prefer-primitive CSS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_patch: Option<TextPatch>,
    /// Actor attribution when applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPatch {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

/// Suggest fixes for known rules from diagnostics on `source`.
///
/// `source_path` resolves package policy (`autofix` / `compose_rhythm`) so
/// suggest/apply match plain `check`. When `None`, compose defaults to Fix
/// (inline snippets / unit tests without a package config).
pub fn suggest_fixes_for_source(
    source: &str,
    file: &str,
    source_path: Option<&std::path::Path>,
) -> Vec<FixPlan> {
    let diags = crate::check_all(source, source_path, &[], None);
    suggest_fixes(source, file, &diags)
}

/// Build fix plans from already-collected diagnostics.
pub fn suggest_fixes(
    source: &str,
    file: &str,
    diags: &[crate::diagnostics::Diagnostic],
) -> Vec<FixPlan> {
    let index = build_node_index(source, file).unwrap_or_default();
    let mut plans = Vec::new();
    for d in diags {
        if let Some(mut ps) = fix_off_scale_space(source, d) {
            plans.append(&mut ps);
            continue;
        }
        if let Some(mut ps) = fix_grow_outside_host(source, d) {
            plans.append(&mut ps);
            continue;
        }
        if let Some(mut ps) = fix_flex_trap(source, d) {
            plans.append(&mut ps);
            continue;
        }
        if let Some(mut ps) = fix_size_slot_full(source, d) {
            plans.append(&mut ps);
            continue;
        }
        if let Some(mut ps) = fix_gradient_molecule(source, d) {
            plans.append(&mut ps);
            continue;
        }
        if let Some(p) = fix_structured_unknown_op(source, d) {
            plans.push(p);
            continue;
        }
        if let Some(p) = fix_structured_dangling_ref(source, d) {
            plans.push(p);
            continue;
        }
        if let Some(p) = fix_structured_mix(source, d) {
            plans.push(p);
            continue;
        }
        if let Some(mut ps) = fix_unknown_when_key(source, d) {
            plans.append(&mut ps);
            continue;
        }
        if let Some(p) = fix_application_default(source, d) {
            plans.push(p);
            continue;
        }
        if let Some(p) = fix_delete_style_const_alias(source, d) {
            plans.push(p);
            continue;
        }
        if let Some(p) = fix_delete_dead_style_const(source, d) {
            plans.push(p);
            continue;
        }
        let rule = d.rule.as_deref().unwrap_or("");
        match rule {
            "sail-participation-style" => {
                if let Some(p) = fix_participation_style(source, &index, d) {
                    plans.push(p);
                }
            }
            "sail-display-on-layout" => {
                if let Some(p) = fix_display_on_layout(source, &index, d) {
                    plans.push(p);
                }
            }
            "sail-prefer-primitive" => {
                if let Some(p) = fix_prefer_primitive(source, d) {
                    plans.push(p);
                }
            }
            "sail-compose-rhythm" => {
                if d.severity == crate::diagnostics::DiagSeverity::Fix {
                    if let Some(mut ps) = fix_compose_rhythm(source, &index, d) {
                        plans.append(&mut ps);
                    }
                }
            }
            "sail-compose-contrast" => {
                if d.severity == crate::diagnostics::DiagSeverity::Fix {
                    if let Some(p) = fix_compose_contrast(source, &index, d) {
                        plans.push(p);
                    }
                }
            }
            "sail-compose-spacing" => {
                if d.severity == crate::diagnostics::DiagSeverity::Fix {
                    if let Some(mut ps) = fix_compose_rhythm(source, &index, d) {
                        for p in &mut ps {
                            p.rule = "sail-compose-spacing".into();
                            p.message = p.message.replacen(
                                "fixed rhythm violation",
                                "fixed off-scale spacing",
                                1,
                            );
                        }
                        plans.append(&mut ps);
                    }
                }
            }
            "sail-no-raw-container" => {
                if let Some(p) = fix_raw_container(source, &index, d) {
                    plans.push(p);
                }
            }
            "spinnaker" => {
                if let Some(p) = fix_unknown_sx_setter(source, &index, d) {
                    plans.push(p);
                }
            }
            "sail-scope-block" => {
                if let Some(p) = fix_unknown_style_const_row(source, d) {
                    plans.push(p);
                }
            }
            _ => {}
        }
    }
    plans
}

/// Confluent allow-list for `fix=apply`. Off-scale two-doors plans are
/// suggest-only — never auto-apply, never delete the slot, never pick a neighbor.
pub fn is_auto_apply_plan(plan: &FixPlan) -> bool {
    match plan.rule.as_str() {
        "sail-participation-style"
        | "sail-display-on-layout"
        | "sail-prefer-primitive"
        | "sail-compose-rhythm"
        | "sail-compose-contrast"
        | "sail-compose-spacing"
        | "sail-no-raw-container"
        | "sail-application-default"
        | "sail-const-eq-name"
        | "sail-dead-const" => true,
        "spinnaker" => plan.message.starts_with("rewrite unknown setter"),
        "sail-scope-block" => plan.message.starts_with("delete unknown style-const"),
        _ => false,
    }
}

/// Omit a Style spend that restates the tag default (`type(heading)` on `<h1>`).
fn fix_application_default(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<FixPlan> {
    if d.rule.as_deref() != Some("sail-application-default") {
        return None;
    }
    let offset = line_col_to_offset(source, d.line, d.col);
    let (start, end) = restated_style_call_span(source, offset)?;
    Some(FixPlan {
        rule: "sail-application-default".into(),
        line: d.line,
        col: d.col,
        message: "omit restated tag-default Style".into(),
        ops: vec![],
        text_patch: Some(TextPatch {
            start,
            end,
            replacement: String::new(),
        }),
        actor: None,
    })
}

/// Delete `style heading = { type(heading) }` — exact alias of an existing name.
fn fix_delete_style_const_alias(
    source: &str,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    if d.rule.as_deref() != Some("sail-const-eq-name") {
        return None;
    }
    delete_backtick_style_const(source, d, "sail-const-eq-name", "delete alias style const")
}

/// Delete an unused `style name =`.
fn fix_delete_dead_style_const(
    source: &str,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    if d.rule.as_deref() != Some("sail-dead-const") {
        return None;
    }
    delete_backtick_style_const(source, d, "sail-dead-const", "delete unused style const")
}

fn delete_backtick_style_const(
    source: &str,
    d: &crate::diagnostics::Diagnostic,
    rule: &str,
    message: &str,
) -> Option<FixPlan> {
    let name = first_backtick_name(&d.message)?;
    let offset = line_col_to_offset(source, d.line, d.col);
    let at = style_const_at_offset(source, offset, name)?;
    let (start, end) = expand_style_const_row(source, at)?;
    Some(FixPlan {
        rule: rule.into(),
        line: d.line,
        col: d.col,
        message: message.into(),
        ops: vec![],
        text_patch: Some(TextPatch {
            start,
            end,
            replacement: String::new(),
        }),
        actor: None,
    })
}

fn first_backtick_name(message: &str) -> Option<&str> {
    let rest = message.strip_prefix('`')?;
    let name = rest.split('`').next()?;
    if name.is_empty() { None } else { Some(name) }
}

/// Whole `style(…)` call on the flagged tag, including the leading space.
fn restated_style_call_span(source: &str, offset: usize) -> Option<(usize, usize)> {
    let tags = sail_lint::rules::layout::scan_any_open_tags(source);
    let tag = tags
        .iter()
        .filter(|t| t.start <= offset && offset < t.end)
        .min_by_key(|t| t.end - t.start)
        .or_else(|| {
            tags.iter()
                .filter(|t| t.start <= offset)
                .max_by_key(|t| t.start)
        })?;
    let sx = tag.attrs.iter().find(|(n, _)| n == "style")?.1.as_str();
    let open = source.get(tag.start..tag.end)?;
    let rel = open.find(sx)?;
    let mut start = tag.start + rel;
    while start > tag.start {
        let prev = source.as_bytes()[start - 1];
        if !prev.is_ascii_whitespace() || prev == b'\n' {
            break;
        }
        start -= 1;
    }
    Some((start, tag.start + rel + sx.len()))
}

/// Three suggest-only plans for `3.5 is not a space step` (neighbors + `raw()`).
fn fix_off_scale_space(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<Vec<FixPlan>> {
    if !d.message.contains("is not a space step") || !d.message.contains("raw(") {
        return None;
    }
    let value = off_scale_space_value(&d.message)?;
    let neighbors = spinnaker::scales::space_neighbors(value)?;
    let offset = line_col_to_offset(source, d.line, d.col);
    let (start, end) = find_token_span_from(source, offset, value)?;
    let rule = d.rule.clone().unwrap_or_else(|| "spinnaker".into());
    let mut plans = Vec::new();
    for neighbor in neighbors
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        plans.push(FixPlan {
            rule: rule.clone(),
            line: d.line,
            col: d.col,
            message: format!("replace `{value}` with `{neighbor}`"),
            ops: vec![],
            text_patch: Some(TextPatch {
                start,
                end,
                replacement: neighbor.to_string(),
            }),
            actor: None,
        });
    }
    plans.push(FixPlan {
        rule,
        line: d.line,
        col: d.col,
        message: format!("wrap as `raw({value})`"),
        ops: vec![],
        text_patch: Some(TextPatch {
            start,
            end,
            replacement: format!("raw({value})"),
        }),
        actor: None,
    });
    if plans.len() < 2 {
        return None;
    }
    Some(plans)
}

/// Suggest-only: unknown gradient molecule → table names. Nothing auto-applies.
fn fix_gradient_molecule(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<Vec<FixPlan>> {
    if !d.message.contains("is not a gradient molecule:") {
        return None;
    }
    let head = d
        .message
        .split(" is not a gradient molecule:")
        .next()?
        .trim();
    let value = head.strip_prefix("Style error:").unwrap_or(head).trim();
    if value.is_empty() {
        return None;
    }
    let offset = line_col_to_offset(source, d.line, d.col);
    let (start, end) = find_token_span_from(source, offset, value)
        .or_else(|| find_token_span_from(source, 0, value))?;
    let rule = d.rule.clone().unwrap_or_else(|| "spinnaker".into());
    let plans = spinnaker::GRADIENT_MOLECULES
        .iter()
        .map(|name| FixPlan {
            rule: rule.clone(),
            line: d.line,
            col: d.col,
            message: format!("replace `{value}` with `{name}`"),
            ops: vec![],
            text_patch: Some(TextPatch {
                start,
                end,
                replacement: (*name).to_string(),
            }),
            actor: None,
        })
        .collect::<Vec<_>>();
    if plans.is_empty() { None } else { Some(plans) }
}

/// Suggest-only: unknown structured op → `mix`. Nothing auto-applies.
fn fix_structured_unknown_op(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<FixPlan> {
    if !d.message.contains("unknown op `") || !d.message.contains("expected {op: mix}") {
        return None;
    }
    let value = d
        .message
        .split("unknown op `")
        .nth(1)?
        .split('`')
        .next()
        .filter(|s| !s.is_empty())?;
    let offset = line_col_to_offset(source, d.line, d.col);
    let (start, end) = find_token_span_from(source, offset, value)?;
    Some(FixPlan {
        rule: d.rule.clone().unwrap_or_else(|| "spinnaker".into()),
        line: d.line,
        col: d.col,
        message: format!("replace `{value}` with `mix`"),
        ops: vec![],
        text_patch: Some(TextPatch {
            start,
            end,
            replacement: "mix".into(),
        }),
        actor: None,
    })
}

/// Suggest-only: dangling `{ref}` → `primary`. Nothing auto-applies.
fn fix_structured_dangling_ref(
    source: &str,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    if !d.message.contains("dangling ref `") {
        return None;
    }
    let inner = d
        .message
        .split("dangling ref `")
        .nth(1)?
        .split('`')
        .next()
        .filter(|s| !s.is_empty())?;
    let value = inner.trim_start_matches('{').trim_end_matches('}');
    if value.is_empty() {
        return None;
    }
    let offset = line_col_to_offset(source, d.line, d.col);
    let (start, end) = find_token_span_from(source, offset, value)?;
    Some(FixPlan {
        rule: d.rule.clone().unwrap_or_else(|| "spinnaker".into()),
        line: d.line,
        col: d.col,
        message: format!("replace `{value}` with `primary`"),
        ops: vec![],
        text_patch: Some(TextPatch {
            start,
            end,
            replacement: "primary".into(),
        }),
        actor: None,
    })
}

/// Suggest-only: mix missing `from` / `onto` → insert the key. Nothing auto-applies.
fn fix_structured_mix(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<FixPlan> {
    if !d.message.contains("mix op requires `") {
        return None;
    }
    let key = d
        .message
        .split("mix op requires `")
        .nth(1)?
        .split('`')
        .next()
        .filter(|s| !s.is_empty())?;
    let insert = match key {
        "from" => "from: primary",
        "onto" => "onto: accent",
        _ => return None,
    };
    let offset = line_col_to_offset(source, d.line, d.col);
    let from = offset.min(source.len());
    let rel = source[from..]
        .find('}')
        .or_else(|| source.find('}').map(|i| i.saturating_sub(from)))?;
    let at = from + rel;
    Some(FixPlan {
        rule: d.rule.clone().unwrap_or_else(|| "spinnaker".into()),
        line: d.line,
        col: d.col,
        message: format!("insert `{insert}`"),
        ops: vec![],
        text_patch: Some(TextPatch {
            start: at,
            end: at,
            replacement: format!(", {insert}"),
        }),
        actor: None,
    })
}

/// Suggest-only: unknown `when:` key → theme breakpoints + `hover`.
fn fix_unknown_when_key(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<Vec<FixPlan>> {
    if !d.message.contains("is not in the theme breakpoint table") {
        return None;
    }
    let key = d
        .message
        .split("style when key `")
        .nth(1)?
        .split('`')
        .next()
        .filter(|s| !s.is_empty())?;
    let offset = line_col_to_offset(source, d.line, d.col);
    let (start, end) = find_token_span_from(source, offset, key)
        .or_else(|| find_token_span_from(source, 0, key))?;
    let rule = d.rule.clone().unwrap_or_else(|| "spinnaker".into());
    let mut names: Vec<String> = spinnaker::tokens::breakpoint_table()
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    if !names.iter().any(|n| n == "hover") {
        names.insert(0, "hover".into());
    }
    let plans = names
        .into_iter()
        .map(|name| FixPlan {
            rule: rule.clone(),
            line: d.line,
            col: d.col,
            message: format!("replace `{key}` with `{name}`"),
            ops: vec![],
            text_patch: Some(TextPatch {
                start,
                end,
                replacement: name,
            }),
            actor: None,
        })
        .collect::<Vec<_>>();
    if plans.is_empty() { None } else { Some(plans) }
}

/// Suggest-only: three doors for retired size-slot `full`. Nothing auto-applies.
fn fix_size_slot_full(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<Vec<FixPlan>> {
    if !d.message.contains("size.full retired") {
        return None;
    }
    let offset = line_col_to_offset(source, d.line, d.col);
    let (tok_start, tok_end) = find_token_span_from(source, offset, "full")
        .or_else(|| find_token_span_from(source, 0, "full"))?;
    // Bare `size(full)` is the flex-trap, not a slot spend.
    if source.get(tok_start.saturating_sub(5)..tok_start) == Some("size(") {
        return None;
    }
    let rule = d.rule.clone().unwrap_or_else(|| "spinnaker".into());
    let mut plans = vec![FixPlan {
        rule: rule.clone(),
        line: d.line,
        col: d.col,
        message: "rewrite `full` to `100%`".into(),
        ops: vec![],
        text_patch: Some(TextPatch {
            start: tok_start,
            end: tok_end,
            replacement: "100%".into(),
        }),
        actor: None,
    }];
    if let Some((call_start, call_end)) = find_size_slot_call_span(source, tok_start) {
        for (rep, msg) in [
            ("size(grow)", "rewrite slot `full` to `size(grow)`"),
            ("size(max)", "rewrite slot `full` to `size(max)`"),
        ] {
            plans.push(FixPlan {
                rule: rule.clone(),
                line: d.line,
                col: d.col,
                message: msg.into(),
                ops: vec![],
                text_patch: Some(TextPatch {
                    start: call_start,
                    end: call_end,
                    replacement: rep.into(),
                }),
                actor: None,
            });
        }
    }
    Some(plans)
}

fn find_size_slot_call_span(source: &str, inside: usize) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut i = inside;
    while i > 0 {
        if source.get(i..).is_some_and(|s| s.starts_with("size("))
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
        {
            let end = scan_paren_close(source, i + 4)?;
            return Some((i, end));
        }
        i -= 1;
    }
    None
}

fn scan_paren_close(source: &str, open_paren: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(open_paren) != Some(&b'(') {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open_paren;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Suggest-only: point `size(full)` at `size(grow)` and `size(max)`. Nothing auto-applies.
fn fix_flex_trap(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<Vec<FixPlan>> {
    if d.rule.as_deref() != Some("sail-flex-trap")
        && !d.message.contains("`size(full)` is a DTCG CSS trap")
    {
        return None;
    }
    let offset = line_col_to_offset(source, d.line, d.col);
    let (start, end) = find_token_span_from(source, offset, "size(full)")?;
    Some(vec![
        FixPlan {
            rule: "sail-flex-trap".into(),
            line: d.line,
            col: d.col,
            message: "rewrite `size(full)` to `size(grow)`".into(),
            ops: vec![],
            text_patch: Some(TextPatch {
                start,
                end,
                replacement: "size(grow)".into(),
            }),
            actor: None,
        },
        FixPlan {
            rule: "sail-flex-trap".into(),
            line: d.line,
            col: d.col,
            message: "rewrite `size(full)` to `size(max)`".into(),
            ops: vec![],
            text_patch: Some(TextPatch {
                start,
                end,
                replacement: "size(max)".into(),
            }),
            actor: None,
        },
    ])
}

/// Suggest-only: rewrite `size(grow)` to `size(hug)` when grow sits outside a host.
fn fix_grow_outside_host(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<Vec<FixPlan>> {
    if d.rule.as_deref() != Some("sail-grow-outside-host") {
        return None;
    }
    let offset = line_col_to_offset(source, d.line, d.col);
    let text_patch = grow_outside_host_span(source, offset).map(|(start, end)| TextPatch {
        start,
        end,
        replacement: "size(hug)".into(),
    });
    Some(vec![FixPlan {
        rule: "sail-grow-outside-host".into(),
        line: d.line,
        col: d.col,
        message: "rewrite `size(grow)` to `size(hug)`".into(),
        ops: vec![],
        text_patch,
        actor: None,
    }])
}

/// Inline `size(grow)` on the flagged tag, else that tag's named-const RHS.
/// Never a later sibling's `size(grow)`.
fn grow_outside_host_span(source: &str, offset: usize) -> Option<(usize, usize)> {
    let tags = sail_lint::rules::layout::scan_any_open_tags(source);
    let tag = tags
        .iter()
        .filter(|t| t.start <= offset && offset < t.end)
        .min_by_key(|t| t.end - t.start)
        .or_else(|| {
            tags.iter()
                .filter(|t| t.start <= offset)
                .max_by_key(|t| t.start)
        })?;
    if let Some(rel) = source.get(tag.start..tag.end)?.find("size(grow)") {
        let start = tag.start + rel;
        return Some((start, start + "size(grow)".len()));
    }
    let sx = tag.attrs.iter().find(|(n, _)| n == "style")?.1.as_str();
    let name = style_const_ref(sx)?;
    const_rhs_size_grow_span(source, &name)
}

fn style_const_ref(sx: &str) -> Option<String> {
    let mut s = sx.trim();
    if let Some(rest) = s.strip_prefix("style") {
        let rest = rest.trim_start();
        if let Some(inner) = rest.strip_prefix('(').and_then(|t| t.strip_suffix(')')) {
            s = inner.trim();
        } else {
            return None;
        }
    } else {
        return None;
    }
    if s.is_empty()
        || !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        || s.chars().next()?.is_ascii_digit()
    {
        return None;
    }
    Some(s.to_string())
}

fn const_rhs_size_grow_span(source: &str, name: &str) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if source[i..].starts_with("style") && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric()) {
            let mut j = i + "style".len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if source[j..].starts_with(name) {
                let after = j + name.len();
                if after < bytes.len()
                    && (bytes[after].is_ascii_alphanumeric()
                        || bytes[after] == b'_'
                        || bytes[after] == b'$')
                {
                    i += 1;
                    continue;
                }
                let mut k = after;
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if bytes.get(k) == Some(&b'=') {
                    k += 1;
                    let init_start = k;
                    while k < bytes.len() && bytes[k] != b'\n' {
                        if (bytes[k] & 0xC0) != 0x80 && source[k..].starts_with("</scope>") {
                            break;
                        }
                        k += 1;
                    }
                    if let Some(rel) = source[init_start..k].find("size(grow)") {
                        let start = init_start + rel;
                        return Some((start, start + "size(grow)".len()));
                    }
                    return None;
                }
            }
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    None
}

fn off_scale_space_value(message: &str) -> Option<&str> {
    let head = message.split(" is not a space step").next()?;
    let value = head.rsplit(": ").next().unwrap_or(head).trim();
    if value.is_empty() { None } else { Some(value) }
}

fn find_token_span_from(source: &str, offset: usize, value: &str) -> Option<(usize, usize)> {
    let from = offset.min(source.len());
    let rel = source[from..].find(value)?;
    let start = from + rel;
    Some((start, start + value.len()))
}

fn fix_participation_style(
    source: &str,
    index: &BTreeMap<String, EditNode>,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    // Bare attr → box channel. Diagnostic points at the attr name on the open tag.
    let offset = line_col_to_offset(source, d.line, d.col);
    let node = find_host_for_offset(index, offset)?;
    let open = &source[node.open_span.start..node.open_span.end];
    let (attr, value) = find_participation_attr_at(open, source, offset)?;
    let setter = match attr.as_str() {
        "grow" => "grow",
        "shrink" => "shrink",
        "w" => "w",
        "min-w" => "minW",
        "max-w" => "maxW",
        _ => return None,
    };
    let arg = match (attr.as_str(), value.as_str()) {
        ("grow" | "shrink", "") | ("grow" | "shrink", "1") | ("grow" | "shrink", "true") => {
            "1".into()
        }
        ("grow" | "shrink", "0") | ("grow" | "shrink", "false") => "0".into(),
        (_, v) if !v.is_empty() => v.to_string(),
        _ => return None,
    };
    let stripped = strip_attr_from_open(open, &attr)?;
    // Product Style attaches as a tag-position call on the open tag (not body).
    let style_call = participation_style_call(setter, &arg);
    let replacement_open = if let Some((lo, hi, expr)) = find_tag_style_call(&stripped) {
        let merged = merge_participation_into_style(&expr, setter, &arg);
        format!("{}{}{}", &stripped[..lo], merged, &stripped[hi..])
    } else {
        inject_tag_style_call(&stripped, &style_call)
    };
    Some(FixPlan {
        rule: "sail-participation-style".into(),
        line: d.line,
        col: d.col,
        message: format!("move `{attr}` attr to Style `{style_call}`"),
        ops: vec![],
        text_patch: Some(TextPatch {
            start: node.open_span.start,
            end: node.open_span.end,
            replacement: replacement_open,
        }),
        actor: None,
    })
}

fn find_participation_attr_at(
    open: &str,
    _source: &str,
    _abs_offset: usize,
) -> Option<(String, String)> {
    // Scan attrs on the open tag fragment.
    let bytes = open.as_bytes();
    let mut i = 0;
    // skip <tag
    while i < bytes.len() && bytes[i] != b' ' && bytes[i] != b'>' && bytes[i] != b'/' {
        i += 1;
    }
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'>' || bytes[i] == b'/' {
            break;
        }
        let ns = i;
        while i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_')
        {
            i += 1;
        }
        let name = open[ns..i].to_string();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut val = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                val = open[vs..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
            }
        }
        if matches!(name.as_str(), "grow" | "shrink" | "w" | "min-w" | "max-w") {
            return Some((name, val));
        }
    }
    None
}

fn strip_attr_from_open(open: &str, attr: &str) -> Option<String> {
    // Remove `attr` or `attr="…"` from the open tag string (byte scan).
    let bytes = open.as_bytes();
    let mut i = 0;
    let mut out = String::new();
    // copy <tag
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
        out.push(bytes[i] as char);
        i += 1;
    }
    while i < bytes.len() {
        let ws_start = i;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'>' || (bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'>') {
            out.push_str(&open[ws_start..]);
            break;
        }
        let ns = i;
        while i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_')
        {
            i += 1;
        }
        let name = &open[ns..i];
        let name_end = i;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            } else if i < bytes.len() && bytes[i] == b'{' {
                let mut d = 0i32;
                while i < bytes.len() {
                    if bytes[i] == b'{' {
                        d += 1;
                    } else if bytes[i] == b'}' {
                        d -= 1;
                        if d == 0 {
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
            } else {
                while i < bytes.len()
                    && !bytes[i].is_ascii_whitespace()
                    && bytes[i] != b'>'
                    && bytes[i] != b'/'
                {
                    i += 1;
                }
            }
        }
        if name == attr {
            let _ = (ws_start, name_end);
            continue;
        }
        let fragment = &open[ws_start..i];
        if !out.ends_with(|c: char| c.is_whitespace()) && !fragment.starts_with(char::is_whitespace)
        {
            out.push(' ');
        }
        out.push_str(fragment);
    }
    if !out.contains('>') {
        out.push('>');
    }
    Some(out)
}

fn fix_display_on_layout(
    source: &str,
    index: &BTreeMap<String, EditNode>,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    let offset = line_col_to_offset(source, d.line, d.col);
    let node = find_host_for_offset(index, offset)?;
    let full = source
        .get(node.full_span.start..node.full_span.end)?
        .to_string();
    // Pull display(…) expression from style if present; otherwise wrap with display(none).
    let (inner_without_display, display_call) = split_display_from_element(&full, &node.tag)?;
    // Tag-position product form only — never emit style={…}.
    // `display_call` is a full `style({ display(…) })` call.
    let wrap = format!("<div {display_call}>{inner_without_display}</div>");
    Some(FixPlan {
        rule: "sail-display-on-layout".into(),
        line: d.line,
        col: d.col,
        message: format!("nest <{}> inside a plain host for style.display", node.tag),
        ops: vec![EditOp::Replace {
            node_id: node.id.clone(),
            fragment: wrap.clone(),
        }],
        text_patch: Some(TextPatch {
            start: node.full_span.start,
            end: node.full_span.end,
            replacement: wrap,
        }),
        actor: None,
    })
}

fn fix_prefer_primitive(source: &str, d: &crate::diagnostics::Diagnostic) -> Option<FixPlan> {
    // Drop the `display: flex|grid` declaration at the diagnostic site.
    let offset = line_col_to_offset(source, d.line, d.col);
    let (start, end) = find_display_decl_span(source, offset)?;
    Some(FixPlan {
        rule: "sail-prefer-primitive".into(),
        line: d.line,
        col: d.col,
        message: "remove hand-rolled display:flex|grid (prefer layout keywords)".into(),
        ops: vec![],
        text_patch: Some(TextPatch {
            start,
            end,
            replacement: String::new(),
        }),
        actor: None,
    })
}

fn fix_compose_rhythm(
    source: &str,
    index: &BTreeMap<String, EditNode>,
    d: &crate::diagnostics::Diagnostic,
) -> Option<Vec<FixPlan>> {
    let offset = line_col_to_offset(source, d.line, d.col);
    let node = find_host_for_offset(index, offset)?;

    // Parent-gap repairs are not emitted by compose yet; margin path only.
    if let Some((setter, target)) = parse_compose_rhythm_margin(&d.message) {
        // The diagnostic points at one child; fix that child only.
        let open = &source[node.open_span.start..node.open_span.end];
        let new_open = replace_sx_setter(open, setter.as_str(), &target)?;
        let (line, col) = crate::diagnostics::line_col(source, node.open_span.start);
        return Some(vec![FixPlan {
            rule: "sail-compose-rhythm".into(),
            line,
            col,
            message: format!("fixed rhythm violation: set {setter} to {target}"),
            ops: vec![super::ops::EditOp::SetSx {
                node_id: node.id.clone(),
                setter: setter.clone(),
                args: vec![serde_json::Value::String(target.clone())],
            }],
            text_patch: Some(TextPatch {
                start: node.open_span.start,
                end: node.open_span.end,
                replacement: new_open,
            }),
            actor: None,
        }]);
    }

    None
}

fn parse_compose_rhythm_margin(message: &str) -> Option<(String, String)> {
    let start = message.find("[sail:fix:margin=")? + "[sail:fix:margin=".len();
    let end = message[start..].find(']')?;
    let inner = &message[start..start + end];
    let (setter, target) = inner.split_once(':')?;
    match setter {
        "mr" | "mb" | "mt" | "ml" | "ms" | "me" | "mx" | "my" | "m" | "pad" | "padX" | "padY"
        | "padT" | "padB" | "padL" | "padR" | "gap" | "gapX" | "gapY" => {
            Some((setter.to_string(), target.to_string()))
        }
        _ => None,
    }
}

fn fix_compose_contrast(
    source: &str,
    index: &BTreeMap<String, EditNode>,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    let target = parse_fix_payload(d.message.as_str(), "fg=")?;
    let offset = line_col_to_offset(source, d.line, d.col);
    let node = find_host_for_offset(index, offset)?;
    let open = &source[node.open_span.start..node.open_span.end];
    let new_open = replace_sx_setter(open, "fg", &target)?;
    Some(FixPlan {
        rule: "sail-compose-contrast".into(),
        line: d.line,
        col: d.col,
        message: format!("fixed contrast: set fg to {target}"),
        ops: vec![super::ops::EditOp::SetSx {
            node_id: node.id.clone(),
            setter: "fg".into(),
            args: vec![serde_json::Value::String(target.clone())],
        }],
        text_patch: Some(TextPatch {
            start: node.open_span.start,
            end: node.open_span.end,
            replacement: new_open,
        }),
        actor: None,
    })
}

fn parse_fix_payload(message: &str, prefix: &str) -> Option<String> {
    let key = format!("[sail:fix:{prefix}");
    let start = message.find(&key)? + key.len();
    let end = message[start..].find(']')?;
    Some(message[start..start + end].to_string())
}

/// Confluent raw-container rewrite: bare `<div>` → `<column>` (or `<row>` when
/// sx expresses horizontal margin rhythm via `mr` / explicit flex row hints).
fn fix_raw_container(
    source: &str,
    index: &BTreeMap<String, EditNode>,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    let offset = line_col_to_offset(source, d.line, d.col);
    let node = find_host_for_offset(index, offset)?;
    let tag = node.tag.as_str();
    if tag != "div" && tag != "span" {
        return None;
    }
    let open = &source[node.open_span.start..node.open_span.end];
    let full = source.get(node.full_span.start..node.full_span.end)?;
    // Prefer row when sx uses mr (horizontal sibling spacing) or tag is span.
    let wants_row = tag == "span"
        || open.contains(".mr(")
        || open.contains("flex-direction:row")
        || open.contains("style.row");
    let replacement_tag = if wants_row { "row" } else { "column" };
    // Only rewrite when there is no class/id that would change meaning beyond layout.
    if open.contains(" class=") || open.contains(" id=") {
        return None;
    }
    let new_full = rewrite_tag_name(full, tag, replacement_tag)?;
    Some(FixPlan {
        rule: "sail-no-raw-container".into(),
        line: d.line,
        col: d.col,
        message: format!("rewrite raw <{tag}> to <{replacement_tag}>"),
        ops: vec![super::ops::EditOp::Replace {
            node_id: node.id.clone(),
            fragment: new_full.clone(),
        }],
        text_patch: Some(TextPatch {
            start: node.full_span.start,
            end: node.full_span.end,
            replacement: new_full,
        }),
        actor: None,
    })
}

fn rewrite_tag_name(fragment: &str, from: &str, to: &str) -> Option<String> {
    let open = format!("<{from}");
    let close = format!("</{from}>");
    if !fragment.starts_with(&open) {
        return None;
    }
    let mut out = String::with_capacity(fragment.len() + 4);
    out.push('<');
    out.push_str(to);
    out.push_str(&fragment[open.len()..]);
    if let Some(idx) = out.rfind(&close) {
        out.replace_range(idx..idx + close.len(), &format!("</{to}>"));
    }
    Some(out)
}

/// Unknown sx setter → closest valid setter (typo correction) when unique.
fn fix_unknown_sx_setter(
    source: &str,
    index: &BTreeMap<String, EditNode>,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    // Prefer machine payload; fall back to "did you mean `foo`" text.
    let target =
        parse_fix_payload(&d.message, "setter=").or_else(|| extract_did_you_mean(&d.message))?;
    let bad = extract_unknown_setter(&d.message)?;
    if bad == target {
        return None;
    }
    let offset = line_col_to_offset(source, d.line, d.col);
    let node = find_host_for_offset(index, offset)?;
    let open = &source[node.open_span.start..node.open_span.end];
    let new_open = open.replacen(&format!(".{bad}("), &format!(".{target}("), 1);
    if new_open == open {
        return None;
    }
    Some(FixPlan {
        rule: "spinnaker".into(),
        line: d.line,
        col: d.col,
        message: format!("rewrite unknown setter {bad} → {target}"),
        ops: vec![],
        text_patch: Some(TextPatch {
            start: node.open_span.start,
            end: node.open_span.end,
            replacement: new_open,
        }),
        actor: None,
    })
}

/// Delete a value-row unknown identity (`Style \`name\`: unknown vocabulary`).
/// Structural scope errors stay unfixed. `wide` is not rewritten to a size name.
fn fix_unknown_style_const_row(
    source: &str,
    d: &crate::diagnostics::Diagnostic,
) -> Option<FixPlan> {
    if !d.message.starts_with("Style `") || !d.message.contains("unknown vocabulary") {
        return None;
    }
    let (start, end) = style_const_row_span(source, d)?;
    Some(FixPlan {
        rule: "sail-scope-block".into(),
        line: d.line,
        col: d.col,
        message: "delete unknown style-const row".into(),
        ops: vec![],
        text_patch: Some(TextPatch {
            start,
            end,
            replacement: String::new(),
        }),
        actor: None,
    })
}

fn style_const_row_span(
    source: &str,
    d: &crate::diagnostics::Diagnostic,
) -> Option<(usize, usize)> {
    let name = d
        .message
        .strip_prefix("Style `")?
        .split('`')
        .next()
        .filter(|n| !n.is_empty())?;
    let offset = line_col_to_offset(source, d.line, d.col);
    let at = style_const_at_offset(source, offset, name)?;
    expand_style_const_row(source, at)
}

/// Bindings are found from the diagnostic line, not byte 0. Quoted
/// `style wide =` in `<!-- … -->` must not steal the live row.
fn style_const_at_offset(source: &str, offset: usize, name: &str) -> Option<usize> {
    let needle = format!("style {name}");
    let bytes = source.as_bytes();
    let mut line_start = offset.min(source.len());
    while line_start > 0 && bytes[line_start - 1] != b'\n' {
        line_start -= 1;
    }
    let mut i = line_start;
    while let Some(rel) = source[i..].find(&needle) {
        let at = i + rel;
        let after = at + needle.len();
        if in_html_comment(source, at) {
            i = after;
            continue;
        }
        if source.get(after..)?.trim_start().starts_with('=') {
            return Some(at);
        }
        i = after;
    }
    None
}

fn in_html_comment(source: &str, at: usize) -> bool {
    let before = &source[..at];
    match (before.rfind("<!--"), before.rfind("-->")) {
        (Some(open), Some(close)) => open > close,
        (Some(_), None) => true,
        _ => false,
    }
}

fn expand_style_const_row(source: &str, at: usize) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut start = at.min(bytes.len());
    while start > 0 && bytes[start - 1] != b'\n' && bytes[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    let eq = source[start..].find('=')?;
    let mut i = start + eq + 1;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i = scan_style_rhs_end(source, i);
    if i < bytes.len() && bytes[i] == b';' {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'\n' {
        i += 1;
    }
    Some((start, i))
}

fn scan_style_rhs_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut depth = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => {
                if depth == 0 {
                    return i;
                }
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            b'\n' | b';' if depth == 0 => return i,
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

fn extract_did_you_mean(message: &str) -> Option<String> {
    // "did you mean `pad`?" / "did you mean \"pad\""
    for (a, b) in [("did you mean `", "`"), ("did you mean \"", "\"")] {
        if let Some(start) = message.find(a) {
            let rest = &message[start + a.len()..];
            if let Some(end) = rest.find(b) {
                let s = rest[..end].trim();
                if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}

fn extract_unknown_setter(message: &str) -> Option<String> {
    // "unknown style setter `padd`" / "unknown setter padd"
    for (a, b) in [
        ("setter `", "`"),
        ("setter \"", "\""),
        ("unknown style.", " "),
        ("Unknown setter ", " "),
    ] {
        if let Some(start) = message.find(a) {
            let rest = &message[start + a.len()..];
            let end = rest
                .find(b)
                .or_else(|| rest.find(|c: char| c.is_whitespace() || c == ')' || c == ','))
                .unwrap_or(rest.len());
            let s = rest[..end].trim().trim_matches(|c| c == '`' || c == '"');
            if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn replace_sx_setter(open: &str, setter: &str, target: &str) -> Option<String> {
    // Tag-position `style(…)` — replace an existing setter call, or inject one.
    if let Some((lo, hi, expr)) = find_tag_style_call(open) {
        let needle = format!(".{setter}(");
        let new_expr = if let Some(rel) = expr.find(&needle) {
            let arg_start = rel + needle.len();
            let mut d = 1i32;
            let mut k = arg_start;
            while k < expr.len() {
                match expr.as_bytes()[k] {
                    b'(' => d += 1,
                    b')' => {
                        d -= 1;
                        if d == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                k += 1;
            }
            if d != 0 {
                return None;
            }
            format!("{}{target}{}", &expr[..arg_start], &expr[k..])
        } else if is_animate_trigger(setter) {
            merge_animate_into_style(&expr, setter, target)
        } else {
            merge_setter_into_flat_style(&expr, setter, target)
        };
        return Some(format!("{}{}{}", &open[..lo], new_expr, &open[hi..]));
    }
    // No style call yet — inject tag-position style({ setter(target) }).
    let call = if is_animate_trigger(setter) {
        format!("style({{ animate: {{ {setter}: play({target}) }} }})")
    } else {
        format!("style({{ {setter}({target}) }})")
    };
    Some(inject_tag_style_call(open, &call))
}

/// Locate tag-position `style(…)` (+ `.when`/chain tails) inside an open tag.
fn find_tag_style_call(open: &str) -> Option<(usize, usize, String)> {
    let bytes = open.as_bytes();
    let mut j = 0usize;
    let mut quote: Option<u8> = None;
    while j < bytes.len() {
        match (quote, bytes[j]) {
            (Some(q), b) if b == q => {
                quote = None;
                j += 1;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[j]);
                j += 1;
            }
            (Some(_), _) => j += 1,
            (None, _) => {
                if open[j..].starts_with("style") {
                    let after = j + 5;
                    let ok_boundary = after >= bytes.len()
                        || (!bytes[after].is_ascii_alphanumeric()
                            && bytes[after] != b'_'
                            && bytes[after] != b'$'
                            && bytes[after] != b'-'
                            && bytes[after] != b':');
                    if ok_boundary {
                        let mut k = after;
                        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                        if k < bytes.len() && bytes[k] == b'(' {
                            let end = consume_style_call_len(open, j)?;
                            let expr = open[j..end].to_string();
                            return Some((j, end, expr));
                        }
                    }
                }
                j += 1;
            }
        }
    }
    None
}

/// Absolute end index of `style(…)` (+ `.method(` tails) starting at `start`.
fn consume_style_call_len(src: &str, start: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if !src[start..].starts_with("style") {
        return None;
    }
    let mut i = start + 5;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'(' {
        return None;
    }
    let mut paren = 0i32;
    let mut brace = 0i32;
    let mut bracket = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => paren += 1,
            b')' => {
                paren -= 1;
                if paren == 0 && brace == 0 && bracket == 0 {
                    i += 1;
                    // chain tails: .when(…) / .method(…)
                    loop {
                        let mut k = i;
                        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                        if k >= bytes.len() || bytes[k] != b'.' {
                            return Some(i);
                        }
                        k += 1;
                        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                        let name_start = k;
                        while k < bytes.len()
                            && (bytes[k].is_ascii_alphanumeric()
                                || bytes[k] == b'_'
                                || bytes[k] == b'-')
                        {
                            k += 1;
                        }
                        if k == name_start {
                            return Some(i);
                        }
                        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                        if k >= bytes.len() || bytes[k] != b'(' {
                            return Some(i);
                        }
                        // consume balanced call
                        let mut p = 0i32;
                        let mut b = 0i32;
                        let mut br = 0i32;
                        while k < bytes.len() {
                            match bytes[k] {
                                b'(' => p += 1,
                                b')' => {
                                    p -= 1;
                                    if p == 0 && b == 0 && br == 0 {
                                        k += 1;
                                        break;
                                    }
                                }
                                b'{' => b += 1,
                                b'}' => b -= 1,
                                b'[' => br += 1,
                                b']' => br -= 1,
                                _ => {}
                            }
                            k += 1;
                        }
                        i = k;
                    }
                }
            }
            b'{' => brace += 1,
            b'}' => brace -= 1,
            b'[' => bracket += 1,
            b']' => bracket -= 1,
            _ => {}
        }
        i += 1;
    }
    None
}

fn inject_tag_style_call(open: &str, style_call: &str) -> String {
    let trimmed = open.trim_end();
    if let Some(stripped) = trimmed.strip_suffix("/>") {
        let head = stripped.trim_end();
        let trail = &open[trimmed.len()..];
        format!("{head} {style_call} />{trail}")
    } else if let Some(stripped) = trimmed.strip_suffix('>') {
        let head = stripped.trim_end();
        let trail = &open[trimmed.len()..];
        format!("{head} {style_call}>{trail}")
    } else {
        format!("{open} {style_call}")
    }
}

fn is_animate_trigger(name: &str) -> bool {
    matches!(
        name,
        "enter"
            | "exit"
            | "loop"
            | "reveal"
            | "scroll"
            | "hover"
            | "press"
            | "focus"
            | "focus-visible"
    )
}

fn participation_style_call(setter: &str, arg: &str) -> String {
    match setter {
        "w" => format!("style({{ size({{ w: {arg} }}) }})"),
        "minW" => format!("style({{ size({{ min-w: {arg} }}) }})"),
        "maxW" => format!("style({{ size({{ max-w: {arg} }}) }})"),
        other => format!("style({{ {other}({arg}) }})"),
    }
}

fn merge_participation_into_style(expr: &str, setter: &str, arg: &str) -> String {
    match setter {
        "w" => merge_size_slot_into_style(expr, "w", arg),
        "minW" => merge_size_slot_into_style(expr, "min-w", arg),
        "maxW" => merge_size_slot_into_style(expr, "max-w", arg),
        _ => merge_setter_into_flat_style(expr, setter, arg),
    }
}

fn merge_setter_into_flat_style(expr: &str, setter: &str, arg: &str) -> String {
    let t = expr.trim();
    let call = format!("{setter}({arg})");
    if let Some(rest) = t.strip_prefix("style(").and_then(|r| r.strip_suffix(')')) {
        let body = rest.trim();
        if body.starts_with('{') && body.ends_with('}') {
            let inner = body[1..body.len() - 1].trim();
            return merge_call_into_inner(inner, setter, arg, &call);
        }
    }
    format!("style({{ {call} }})")
}

fn merge_animate_into_style(expr: &str, setter: &str, arg: &str) -> String {
    let t = expr.trim();
    let entry = format!("{setter}: play({arg})");
    if let Some(rest) = t.strip_prefix("style(").and_then(|r| r.strip_suffix(')')) {
        let body = rest.trim();
        if body.starts_with('{') && body.ends_with('}') {
            let inner = body[1..body.len() - 1].trim();
            if let Some(idx) = find_channel_key_in(inner, "animate") {
                let after_key = idx + "animate:".len();
                let (val_start, val_end) = channel_value_span_in(inner, after_key);
                let val = inner[val_start..val_end].trim();
                let new_val = if val.trim().starts_with('{') {
                    merge_play_entry_into_map(val, setter, &entry)
                } else {
                    format!("{{ {entry} }}")
                };
                let mut out = String::from("style({ ");
                out.push_str(inner[..val_start].trim_end());
                if !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(&new_val);
                let tail = inner[val_end..].trim_start();
                if !tail.is_empty() {
                    out.push(' ');
                    out.push_str(tail);
                }
                out.push_str(" })");
                return out;
            }
            let sep = if inner.is_empty() { "" } else { ", " };
            return format!("style({{ {inner}{sep}animate: {{ {entry} }} }})");
        }
    }
    format!("style({{ animate: {{ {entry} }} }})")
}

fn merge_play_entry_into_map(val: &str, setter: &str, entry: &str) -> String {
    let t = val.trim();
    if !(t.starts_with('{') && t.ends_with('}')) {
        return format!("{{ {entry} }}");
    }
    let inner = t[1..t.len() - 1].trim();
    if inner.is_empty() {
        return format!("{{ {entry} }}");
    }
    let key = format!("{setter}:");
    if inner.contains(&key) {
        return format!("{{ {inner} }}");
    }
    format!("{{ {inner}, {entry} }}")
}

fn merge_size_slot_into_style(expr: &str, slot: &str, arg: &str) -> String {
    let t = expr.trim();
    if let Some(rest) = t.strip_prefix("style(").and_then(|r| r.strip_suffix(')')) {
        let body = rest.trim();
        if body.starts_with('{') && body.ends_with('}') {
            let inner = body[1..body.len() - 1].trim();
            let key_at = first_style_object_key_at(inner);
            let (chain, keys) = split_chain_and_keys(inner, key_at);
            let new_chain = merge_size_slot_into_chain(chain, slot, arg);
            return reassemble_style(&new_chain, keys);
        }
    }
    format!("style({{ size({{ {slot}: {arg} }}) }})")
}

fn merge_call_into_inner(inner: &str, setter: &str, arg: &str, call: &str) -> String {
    let key_at = first_style_object_key_at(inner);
    let (chain, keys) = split_chain_and_keys(inner, key_at);
    let new_chain = if chain.is_empty() {
        call.to_string()
    } else if chain.contains(&format!("{setter}(")) {
        replace_method_arg(chain, setter, arg).unwrap_or_else(|| format!("{chain}.{call}"))
    } else {
        format!("{chain}.{call}")
    };
    reassemble_style(&new_chain, keys)
}

fn split_chain_and_keys(inner: &str, key_at: Option<usize>) -> (&str, &str) {
    if let Some(idx) = key_at {
        (
            inner[..idx].trim().trim_end_matches(',').trim(),
            inner[idx..].trim(),
        )
    } else {
        (inner.trim(), "")
    }
}

fn reassemble_style(chain: &str, keys: &str) -> String {
    if keys.is_empty() {
        format!("style({{ {chain} }})")
    } else if chain.is_empty() {
        format!("style({{ {keys} }})")
    } else {
        format!("style({{ {chain}, {keys} }})")
    }
}

fn first_style_object_key_at(inner: &str) -> Option<usize> {
    ["when", "match", "compound", "animate"]
        .iter()
        .filter_map(|k| find_channel_key_in(inner, k))
        .min()
}

fn merge_size_slot_into_chain(chain: &str, slot: &str, arg: &str) -> String {
    if let Some(rel) = chain.find("size(") {
        let arg_start = rel + "size(".len();
        let mut d = 1i32;
        let mut k = arg_start;
        let bytes = chain.as_bytes();
        while k < bytes.len() {
            match bytes[k] {
                b'(' => d += 1,
                b')' => {
                    d -= 1;
                    if d == 0 {
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        if d == 0 {
            let size_arg = chain[arg_start..k].trim();
            if size_arg.starts_with('{') && size_arg.ends_with('}') {
                let obj = size_arg[1..size_arg.len() - 1].trim();
                let new_obj = if let Some(idx) = find_channel_key_in(obj, slot) {
                    let after = idx + slot.len() + 1;
                    let (vs, ve) = channel_value_span_in(obj, after);
                    format!("{}{}{}", &obj[..vs], arg, &obj[ve..])
                } else {
                    let sep = if obj.is_empty() { "" } else { ", " };
                    format!("{obj}{sep}{slot}: {arg}")
                };
                return format!("{}size({{ {new_obj} }}){}", &chain[..rel], &chain[k + 1..]);
            }
        }
    }
    let call = format!("size({{ {slot}: {arg} }})");
    if chain.is_empty() {
        call
    } else {
        format!("{chain}.{call}")
    }
}

fn find_channel_key_in(inner: &str, ch: &str) -> Option<usize> {
    let key = format!("{ch}:");
    let bytes = inner.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            _ => {}
        }
        if depth == 0 && inner[i..].starts_with(&key) {
            if i == 0 || bytes[i - 1].is_ascii_whitespace() || bytes[i - 1] == b',' {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn channel_value_span_in(inner: &str, after_key: usize) -> (usize, usize) {
    let bytes = inner.as_bytes();
    let mut i = after_key;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let start = i;
    let mut paren = 0i32;
    let mut brace = 0i32;
    let mut bracket = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => paren += 1,
            b')' => paren -= 1,
            b'{' => brace += 1,
            b'}' => brace -= 1,
            b'[' => bracket += 1,
            b']' => bracket -= 1,
            b',' if paren == 0 && brace == 0 && bracket == 0 => break,
            _ => {}
        }
        i += 1;
    }
    (start, i)
}

fn replace_method_arg(chain: &str, setter: &str, arg: &str) -> Option<String> {
    let needle = format!("{setter}(");
    let rel = chain.find(&needle)?;
    let arg_start = rel + needle.len();
    let mut d = 1i32;
    let mut k = arg_start;
    while k < chain.len() {
        match chain.as_bytes()[k] {
            b'(' => d += 1,
            b')' => {
                d -= 1;
                if d == 0 {
                    return Some(format!("{}{arg}{}", &chain[..arg_start], &chain[k..]));
                }
            }
            _ => {}
        }
        k += 1;
    }
    None
}

/// Apply all text patches (highest offset first) + return new source.
/// Apply text patches (end-first). Returns `(result, skipped)` where `skipped`
/// counts patches with out-of-range or inverted spans that were not applied.
pub fn apply_text_patches(source: &str, patches: &[(usize, usize, String)]) -> (String, usize) {
    let mut ordered: Vec<_> = patches.to_vec();
    ordered.sort_by_key(|b| std::cmp::Reverse(b.0));
    let mut out = source.to_string();
    let mut skipped = 0usize;
    for (start, end, rep) in ordered {
        if start > out.len() || end > out.len() || start > end {
            skipped += 1;
            continue;
        }
        out.replace_range(start..end, &rep);
    }
    (out, skipped)
}

fn line_col_to_offset(source: &str, line: usize, col: usize) -> usize {
    let mut cur_line = 1usize;
    let mut cur_col = 1usize;
    for (i, ch) in source.char_indices() {
        if cur_line == line && cur_col == col {
            return i;
        }
        if ch == '\n' {
            cur_line += 1;
            cur_col = 1;
        } else {
            cur_col += 1;
        }
    }
    source.len()
}

fn find_host_for_offset(index: &BTreeMap<String, EditNode>, offset: usize) -> Option<&EditNode> {
    // Prefer the tightest open-tag that covers offset, else full_span.
    index
        .values()
        .filter(|n| n.open_span.start <= offset && offset < n.open_span.end)
        .min_by_key(|n| n.open_span.end - n.open_span.start)
        .or_else(|| {
            index
                .values()
                .filter(|n| n.full_span.start <= offset && offset < n.full_span.end)
                .min_by_key(|n| n.full_span.end - n.full_span.start)
        })
}

fn collapse_ws_open(s: &str) -> String {
    let mut out = s.to_string();
    while out.contains("  ") {
        out = out.replace("  ", " ");
    }
    out = out.replace(" >", ">");
    out = out.replace(" />", "/>");
    out
}

fn split_display_from_element(full: &str, tag: &str) -> Option<(String, String)> {
    // Find tag-position style(…) on the open tag and peel display(…) out.
    let gt = full.find('>')?;
    let open = &full[..gt + 1];
    let rest = &full[gt + 1..];
    let (sx_start, sx_end, expr) = find_tag_style_call(open)?;
    let (new_expr, display_expr) = peel_display(&expr)?;
    let mut new_open = String::new();
    new_open.push_str(&open[..sx_start]);
    if new_expr.trim().is_empty() {
        // drop style call entirely
        new_open.push_str(&open[sx_end..]);
    } else {
        new_open.push_str(&new_expr);
        new_open.push_str(&open[sx_end..]);
    }
    new_open = collapse_ws_open(&new_open);
    let inner = format!("{new_open}{rest}");
    let _ = tag;
    Some((inner, display_expr))
}

fn peel_display(expr: &str) -> Option<(String, String)> {
    // style({ display(none).pad(2) }) → peel display into outer wrap expr
    // Also residual flat: style.display(none).pad(2)
    let disp = expr.find("display(")?;
    let arg_start = disp + "display(".len();
    let mut depth = 1i32;
    let bytes = expr.as_bytes();
    let mut j = arg_start;
    while j < bytes.len() {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        j += 1;
    }
    if depth != 0 {
        return None;
    }
    let call_end = j + 1; // after ')'
    let display_arg = expr[arg_start..j].trim();
    // Product wrap form uses a concern-chain style for the outer host.
    let display_expr = format!("style({{ display({display_arg}) }})");

    // Remove `display(…)` / `.display(…)` from the original chain.
    let call_start = if disp > 0 && bytes[disp - 1] == b'.' {
        disp - 1
    } else {
        disp
    };
    let mut new_expr = String::new();
    new_expr.push_str(&expr[..call_start]);
    new_expr.push_str(&expr[call_end..]);
    while new_expr.contains("..") {
        new_expr = new_expr.replace("..", ".");
    }
    if new_expr.starts_with('.') {
        new_expr = format!("style{new_expr}");
    }
    // Clean empty Style shells (including leftover channel empties).
    let trimmed = new_expr.trim();
    if trimmed.is_empty()
        || trimmed == "style({})"
        || trimmed == "style({ })"
        || trimmed == "style({ box: })"
        || trimmed == "style({ box:  })"
    {
        new_expr.clear();
    }
    // Outer wrap host gets a full tag-position style call.
    Some((new_expr, display_expr))
}

fn find_display_decl_span(source: &str, offset: usize) -> Option<(usize, usize)> {
    // Walk back to start of `display` property, forward through value to `;` or `}`.
    let bytes = source.as_bytes();
    let mut start = offset.min(bytes.len());
    while start > 0 && bytes[start - 1] != b';' && bytes[start - 1] != b'\n' {
        start -= 1;
    }
    // include leading whitespace after previous ;
    let mut s = start;
    while s < bytes.len() && bytes[s].is_ascii_whitespace() {
        s += 1;
    }
    if !source[s..].starts_with("display") {
        // try scanning around
        let window_start = offset.saturating_sub(20);
        if let Some(rel) = source[window_start..].find("display") {
            s = window_start + rel;
        } else {
            return None;
        }
    }
    let mut end = s;
    while end < bytes.len() && bytes[end] != b';' && bytes[end] != b'}' {
        end += 1;
    }
    if end < bytes.len() && bytes[end] == b';' {
        end += 1;
    }
    // also swallow following newline
    if end < bytes.len() && bytes[end] == b'\n' {
        end += 1;
    }
    Some((s, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn participation_strips_bare_grow_attr() {
        let open = r#"<div grow min-w="0" class="x">"#;
        let next = strip_attr_from_open(open, "grow").unwrap();
        assert!(!next.contains("grow"), "{next}");
        assert!(next.contains("min-w"), "{next}");
        let next2 = strip_attr_from_open(&next, "min-w").unwrap();
        assert!(!next2.contains("min-w"), "{next2}");
        assert!(next2.contains("class"), "{next2}");
    }

    #[test]
    fn participation_fix_injects_tag_style_not_body() {
        let src = r#"<div grow>x</div>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let p = plans
            .iter()
            .find(|p| p.rule == "sail-participation-style")
            .expect("participation fix");
        let patch = p.text_patch.as_ref().expect("text patch");
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(
            out.contains("style({ grow(1) })"),
            "expected tag style call: {out}"
        );
        assert!(
            !out.contains(">\n  style(") && !out.contains(">\nstyle("),
            "must not inject body Style: {out}"
        );
        assert!(!out.contains("style={"), "must not emit style={{: {out}");
        // Fixed source must preprocess cleanly (tag form accepted).
        crate::parse::preprocess::preprocess(&out).expect(&format!("preprocess failed: {out}"));
    }

    #[test]
    fn replace_sx_setter_targets_tag_style() {
        let open = r#"<div style({ pad(2) })>"#;
        let next = replace_sx_setter(open, "pad", "4").unwrap();
        assert_eq!(next, r#"<div style({ pad(4) })>"#);
        assert!(!next.contains("style={"), "{next}");
    }

    #[test]
    fn scope_block_fix_deletes_unknown_style_const_row() {
        let src = r#"
<!--
        style wide = { size(wide) }
-->
<scope>
  style base = { pad(page) }
  style wide = { size(wide) }
</scope>
<column style(base)>x</column>
"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let p = plans
            .iter()
            .find(|p| p.rule == "sail-scope-block")
            .expect("sail-scope-block fix");
        assert_eq!(
            plans
                .iter()
                .filter(|p| p.rule == "sail-scope-block")
                .count(),
            1
        );
        let patch = p.text_patch.as_ref().expect("text patch");
        let comment_end = src.find("-->").expect("quoted trap comment");
        assert!(
            patch.start > comment_end,
            "must delete the live row, not the quoted trap: start={} comment_end={comment_end}",
            patch.start
        );
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(
            !out.contains("\n  style wide ="),
            "failed row must be gone: {out}"
        );
        assert!(
            out.contains("        style wide = { size(wide) }"),
            "quoted trap in the comment must stay: {out}"
        );
        assert!(
            out.contains("style base = { pad(page) }"),
            "sibling const must stay: {out}"
        );
        let diags = crate::check_all(&out, None, &[], None);
        let joined = diags
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !joined.contains("unknown vocabulary") && !joined.contains("wide"),
            "fixed source must not diagnose wide: {joined}"
        );
        let lowered =
            crate::parse::preprocess::preprocess(&out).expect("fixed source must preprocess");
        assert!(
            lowered.contains("s-padpage") || lowered.contains("--pad-page"),
            "style(base) must still lower: {lowered}"
        );
    }

    #[test]
    fn flex_trap_suggests_grow_and_max() {
        let src = r#"<row><column style({ size(full) })>x</column></row>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let flex: Vec<_> = plans
            .iter()
            .filter(|p| p.rule == "sail-flex-trap")
            .collect();
        assert_eq!(flex.len(), 2, "{plans:?}");
        let reps: Vec<&str> = flex
            .iter()
            .filter_map(|p| p.text_patch.as_ref().map(|t| t.replacement.as_str()))
            .collect();
        assert!(reps.contains(&"size(grow)"), "{reps:?}");
        assert!(reps.contains(&"size(max)"), "{reps:?}");
        for p in &flex {
            assert!(!is_auto_apply_plan(p), "suggest-only: {p:?}");
        }
    }

    #[test]
    fn flex_trap_apply_does_not_consume() {
        let src = r#"<row><column style({ size(full) })>x</column></row>"#;
        let result = crate::edit::apply_ops_to_source(
            src,
            "t.sail",
            &[],
            None,
            crate::edit::AutofixMode::Apply,
            Some("test"),
        );
        assert!(
            result.auto_fixed.is_empty(),
            "fix=apply must not rewrite flex-trap: {:?}",
            result.auto_fixed
        );
        assert!(
            result.source.contains("size(full)"),
            "source must stay full: {}",
            result.source
        );
    }

    #[test]
    fn size_slot_full_suggests_three_doors() {
        let src = r#"<div style({ size({ w: full }) })>x</div>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let retired: Vec<_> = plans
            .iter()
            .filter(|p| {
                p.message.contains("100%")
                    || p.message.contains("size(grow)")
                    || p.message.contains("size(max)")
            })
            .collect();
        assert_eq!(retired.len(), 3, "{plans:?}");
        let reps: Vec<&str> = retired
            .iter()
            .filter_map(|p| p.text_patch.as_ref().map(|t| t.replacement.as_str()))
            .collect();
        assert!(reps.contains(&"100%"), "{reps:?}");
        assert!(reps.contains(&"size(grow)"), "{reps:?}");
        assert!(reps.contains(&"size(max)"), "{reps:?}");
        for p in &retired {
            assert!(!is_auto_apply_plan(p), "suggest-only: {p:?}");
        }
    }

    #[test]
    fn size_slot_full_apply_does_not_consume() {
        let src = r#"<div style({ size({ w: full }) })>x</div>"#;
        let result = crate::edit::apply_ops_to_source(
            src,
            "t.sail",
            &[],
            None,
            crate::edit::AutofixMode::Apply,
            Some("test"),
        );
        assert!(
            result.auto_fixed.is_empty(),
            "fix=apply must not rewrite slot full: {:?}",
            result.auto_fixed
        );
        assert!(
            result.source.contains("w: full"),
            "source must stay full: {}",
            result.source
        );
    }

    #[test]
    fn grow_outside_host_const_ref_rewrites_rhs_not_later_sibling() {
        let src = r#"
<scope>
  style leftover = { size(grow), bg(accent) }
</scope>
<column style(leftover)>x</column>
<row>
  <column style({ size(grow) })>legal</column>
</row>
"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let grow: Vec<_> = plans
            .iter()
            .filter(|p| p.rule == "sail-grow-outside-host")
            .collect();
        assert_eq!(grow.len(), 1, "{plans:?}");
        assert!(!is_auto_apply_plan(grow[0]), "suggest-only: {:?}", grow[0]);
        let patch = grow[0].text_patch.as_ref().expect("const RHS patch");
        assert_eq!(patch.replacement, "size(hug)");
        let sliced = &src[patch.start..patch.end];
        assert_eq!(sliced, "size(grow)");
        assert!(
            patch.start < src.find("<column style(leftover)>").unwrap(),
            "must rewrite the const RHS, not a later use: start={}",
            patch.start
        );
        let legal = src.find("<column style({ size(grow) })>").unwrap();
        assert!(
            patch.end <= legal,
            "must not rewrite the later legal size(grow): start={} legal={legal}",
            patch.start
        );
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(
            out.contains("style leftover = { size(hug), bg(accent) }"),
            "{out}"
        );
        assert!(
            out.contains("<column style({ size(grow) })>legal</column>"),
            "{out}"
        );
    }

    #[test]
    fn grow_outside_host_suggests_hug_only() {
        let src = r#"<column style({ size(grow) })>x</column>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let grow: Vec<_> = plans
            .iter()
            .filter(|p| p.rule == "sail-grow-outside-host")
            .collect();
        assert_eq!(grow.len(), 1, "{plans:?}");
        assert!(!is_auto_apply_plan(grow[0]), "suggest-only: {:?}", grow[0]);
        let patch = grow[0].text_patch.as_ref().expect("text patch");
        assert_eq!(patch.replacement, "size(hug)");
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(out.contains("size(hug)"), "{out}");
        let diags = crate::check_all(&out, None, &[], None);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule.as_deref() == Some("sail-grow-outside-host")),
            "hug rewrite must clear the diagnostic: {diags:?}"
        );
    }

    #[test]
    fn grow_outside_host_apply_does_not_consume() {
        let src = r#"<column style({ size(grow) })>x</column>"#;
        let result = crate::edit::apply_ops_to_source(
            src,
            "t.sail",
            &[],
            None,
            crate::edit::AutofixMode::Apply,
            Some("test"),
        );
        assert!(
            result.auto_fixed.is_empty(),
            "fix=apply must not rewrite grow-outside-host: {:?}",
            result.auto_fixed
        );
        assert!(
            result.source.contains("size(grow)"),
            "source must stay grow: {}",
            result.source
        );
    }

    #[test]
    fn off_scale_pad_suggests_three_plans() {
        let src = r#"<div style({ pad({ x: 3.5 }) })>x</div>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let space: Vec<_> = plans
            .iter()
            .filter(|p| {
                p.rule == "spinnaker"
                    || p.rule == "sail-scope-block"
                    || p.message.contains("raw(3.5)")
                    || p.message.contains("replace `3.5`")
            })
            .collect();
        assert_eq!(space.len(), 3, "{plans:?}");
        let reps: Vec<&str> = space
            .iter()
            .filter_map(|p| p.text_patch.as_ref().map(|t| t.replacement.as_str()))
            .collect();
        assert!(reps.contains(&"3"), "{reps:?}");
        assert!(reps.contains(&"4"), "{reps:?}");
        assert!(reps.contains(&"raw(3.5)"), "{reps:?}");
        for p in &space {
            assert!(!is_auto_apply_plan(p), "suggest-only: {p:?}");
        }
        for p in &space {
            let patch = p.text_patch.as_ref().expect("text patch");
            let (out, skipped) =
                apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
            assert_eq!(skipped, 0, "{out}");
            crate::parse::preprocess::preprocess(&out)
                .unwrap_or_else(|e| panic!("applying {:?} must compile: {e:?}\n{out}", p.message));
        }
    }

    #[test]
    fn gradient_unknown_suggests_table() {
        let src = r#"<div style({ bg({ gradient: rainbow }) })>x</div>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let g: Vec<_> = plans
            .iter()
            .filter(|p| p.message.starts_with("replace `") && p.text_patch.is_some())
            .filter(|p| {
                matches!(
                    p.text_patch.as_ref().map(|t| t.replacement.as_str()),
                    Some("primary" | "accent" | "surface" | "none")
                )
            })
            .collect();
        let reps: Vec<&str> = g
            .iter()
            .filter_map(|p| p.text_patch.as_ref().map(|t| t.replacement.as_str()))
            .collect();
        assert!(reps.contains(&"primary"), "{plans:?}");
        assert!(reps.contains(&"accent"), "{reps:?}");
        assert!(reps.contains(&"surface"), "{reps:?}");
        assert!(reps.contains(&"none"), "{reps:?}");
        for p in &g {
            assert!(!is_auto_apply_plan(p), "suggest-only: {p:?}");
        }
        let primary = g
            .iter()
            .find(|p| p.text_patch.as_ref().map(|t| t.replacement.as_str()) == Some("primary"))
            .unwrap();
        let patch = primary.text_patch.as_ref().unwrap();
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(out.contains("gradient: primary"), "{out}");
        crate::parse::preprocess::preprocess(&out)
            .unwrap_or_else(|e| panic!("fixed source must compile: {e:?}\n{out}"));
    }

    #[test]
    fn structured_unknown_op_suggests_mix() {
        let src = r#"<div style({ bg({ op: blur, from: primary, onto: accent }) })>x</div>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let op: Vec<_> = plans
            .iter()
            .filter(|p| p.message.contains("`mix`"))
            .collect();
        assert_eq!(op.len(), 1, "{plans:?}");
        assert!(!is_auto_apply_plan(op[0]), "suggest-only: {:?}", op[0]);
        let patch = op[0].text_patch.as_ref().unwrap();
        assert_eq!(patch.replacement, "mix");
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(out.contains("op: mix"), "{out}");
        crate::parse::preprocess::preprocess(&out)
            .unwrap_or_else(|e| panic!("fixed source must compile: {e:?}\n{out}"));
    }

    #[test]
    fn structured_mix_missing_from_inserts_key() {
        let src = r#"<div style({ bg({ op: mix, onto: accent }) })>x</div>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let mix: Vec<_> = plans
            .iter()
            .filter(|p| p.message.contains("from: primary"))
            .collect();
        assert_eq!(mix.len(), 1, "{plans:?}");
        assert!(!is_auto_apply_plan(mix[0]), "suggest-only: {:?}", mix[0]);
        let patch = mix[0].text_patch.as_ref().unwrap();
        assert_eq!(patch.replacement, ", from: primary");
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(out.contains("from: primary"), "{out}");
        crate::parse::preprocess::preprocess(&out)
            .unwrap_or_else(|e| panic!("fixed source must compile: {e:?}\n{out}"));
    }

    #[test]
    fn structured_dangling_ref_suggests_primary() {
        let src = r#"<div style({ bg({ ref: no-such }) })>x</div>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let r: Vec<_> = plans
            .iter()
            .filter(|p| p.message.contains("`primary`"))
            .collect();
        assert_eq!(r.len(), 1, "{plans:?}");
        assert!(!is_auto_apply_plan(r[0]), "suggest-only: {:?}", r[0]);
        let patch = r[0].text_patch.as_ref().unwrap();
        assert_eq!(patch.replacement, "primary");
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(out.contains("ref: primary"), "{out}");
        crate::parse::preprocess::preprocess(&out)
            .unwrap_or_else(|e| panic!("fixed source must compile: {e:?}\n{out}"));
    }

    #[test]
    fn off_scale_pad_apply_does_not_consume() {
        let src = r#"<div style({ pad({ x: 3.5 }) })>x</div>"#;
        let result = crate::edit::apply_ops_to_source(
            src,
            "t.sail",
            &[],
            None,
            crate::edit::AutofixMode::Apply,
            Some("test"),
        );
        assert!(
            result.auto_fixed.is_empty(),
            "fix=apply must not rewrite two-doors plans: {:?}",
            result.auto_fixed
        );
        assert!(
            result.source.contains("3.5"),
            "source must stay unwrapped: {}",
            result.source
        );
    }

    #[test]
    fn unknown_when_key_suggests_theme_breakpoints() {
        let src = r#"<column style({ bg(primary), when: { kiosk: bg(accent) } })>x</column>"#;
        let plans = suggest_fixes_for_source(src, "t.sail", None);
        let when_plans: Vec<_> = plans
            .iter()
            .filter(|p| p.message.contains("replace `kiosk`"))
            .collect();
        assert!(
            !when_plans.is_empty(),
            "expected FixPlan for undeclared when key: {plans:?}"
        );
        assert!(
            when_plans.iter().any(|p| p.message.contains("`hover`")),
            "must offer a language state: {when_plans:?}"
        );
        for name in ["mobile", "tablet", "desktop"] {
            assert!(
                when_plans
                    .iter()
                    .any(|p| p.message.contains(&format!("`{name}`"))),
                "must offer theme bp `{name}`: {when_plans:?}"
            );
        }
        assert!(
            when_plans.iter().all(|p| !is_auto_apply_plan(p)),
            "suggest-only: {when_plans:?}"
        );
        let hover = when_plans
            .iter()
            .find(|p| p.text_patch.as_ref().map(|t| t.replacement.as_str()) == Some("hover"))
            .unwrap();
        let patch = hover.text_patch.as_ref().unwrap();
        let (out, skipped) =
            apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
        assert_eq!(skipped, 0);
        assert!(out.contains("when: { hover:"), "{out}");
        crate::parse::preprocess::preprocess(&out)
            .unwrap_or_else(|e| panic!("fixed source must compile: {e:?}\n{out}"));
    }
}
