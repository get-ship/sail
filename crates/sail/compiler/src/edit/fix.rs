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
            "sail-sx" => {
                if let Some(p) = fix_unknown_sx_setter(source, &index, d) {
                    plans.push(p);
                }
            }
            _ => {}
        }
    }
    plans
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
    let box_call = format!("style({{ box: {setter}({arg}) }})");
    let replacement_open = if let Some((lo, hi, expr)) = find_tag_style_call(&stripped) {
        let merged = merge_box_setter_into_style(&expr, setter, &arg);
        format!("{}{}{}", &stripped[..lo], merged, &stripped[hi..])
    } else {
        inject_tag_style_call(&stripped, &box_call)
    };
    Some(FixPlan {
        rule: "sail-participation-style".into(),
        line: d.line,
        col: d.col,
        message: format!("move `{attr}` attr to box channel `{box_call}`"),
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
    // `display_call` is a full `style({ box: display(…) })` call.
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
        rule: "sail-sx".into(),
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
        } else if let Some(ch) = channel_key_for_setter_name(setter) {
            // Append into matching channel, or add a channel entry.
            merge_setter_into_channel_style(&expr, ch, setter, target)
        } else {
            // Unknown channel — append flat chain residual is not product; inject channel.
            format!("style({{ paint: {setter}({target}) }})")
        };
        return Some(format!("{}{}{}", &open[..lo], new_expr, &open[hi..]));
    }
    // No style call yet — inject tag-position style({ channel: setter(target) }).
    let ch = channel_key_for_setter_name(setter).unwrap_or("paint");
    let call = format!("style({{ {ch}: {setter}({target}) }})");
    Some(inject_tag_style_call(open, &call))
}

fn channel_key_for_setter_name(setter: &str) -> Option<&'static str> {
    match sail_lint::style::channel_for_setter(setter) {
        Some(sail_lint::style::Channel::Box) => Some("box"),
        Some(sail_lint::style::Channel::Text) => Some("text"),
        Some(sail_lint::style::Channel::Animate) => Some("animate"),
        Some(sail_lint::style::Channel::Paint) => Some("paint"),
        None => None,
    }
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
    if trimmed.ends_with("/>") {
        let head = &trimmed[..trimmed.len() - 2].trim_end();
        let trail = &open[trimmed.len()..];
        format!("{head} {style_call} />{trail}")
    } else if trimmed.ends_with('>') {
        let head = &trimmed[..trimmed.len() - 1].trim_end();
        let trail = &open[trimmed.len()..];
        format!("{head} {style_call}>{trail}")
    } else {
        format!("{open} {style_call}")
    }
}

fn merge_box_setter_into_style(expr: &str, setter: &str, arg: &str) -> String {
    merge_setter_into_channel_style(expr, "box", setter, arg)
}

fn merge_setter_into_channel_style(expr: &str, ch: &str, setter: &str, arg: &str) -> String {
    let t = expr.trim();
    let call = format!("{setter}({arg})");
    if let Some(rest) = t.strip_prefix("style(").and_then(|r| r.strip_suffix(')')) {
        // Only simple style({…}) without chain tails handled here; chains keep full expr prefix.
        let body = rest.trim();
        if body.starts_with('{') && body.ends_with('}') {
            let inner = body[1..body.len() - 1].trim();
            let key = format!("{ch}:");
            if let Some(idx) = find_channel_key_in(inner, ch) {
                let after_key = idx + key.len();
                let (val_start, val_end) = channel_value_span_in(inner, after_key);
                let val = inner[val_start..val_end].trim();
                let new_val = if val.is_empty() {
                    call.clone()
                } else if val.contains(&format!("{setter}(")) {
                    // Replace existing setter arg if present.
                    replace_method_arg(val, setter, arg).unwrap_or_else(|| format!("{val}.{call}"))
                } else {
                    format!("{val}.{call}")
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
            return format!("style({{ {inner}{sep}{ch}: {call} }})");
        }
    }
    // Non-object form (const ref / list) — leave existing and don't invent dual style.
    // Caller prefers inject only when no style present; here append is wrong.
    // Replace whole call with a channel spine that at least carries the fix.
    format!("style({{ {ch}: {call} }})")
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
    ordered.sort_by(|a, b| b.0.cmp(&a.0));
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

fn find_host_for_offset<'a>(
    index: &'a BTreeMap<String, EditNode>,
    offset: usize,
) -> Option<&'a EditNode> {
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
    // style({ box: display(none).pad(2) }) → peel display into outer wrap expr
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
    // Product wrap form always uses channel spine for the outer host.
    let display_expr = format!("style({{ box: display({display_arg}) }})");

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
    // Clean empty channel shells: style({ box:  }) / style({  })
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
            out.contains("style({ box: grow(1) })"),
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
        let open = r#"<div style({ box: pad(2) })>"#;
        let next = replace_sx_setter(open, "pad", "4").unwrap();
        assert_eq!(next, r#"<div style({ box: pad(4) })>"#);
        assert!(!next.contains("style={"), "{next}");
    }
}
