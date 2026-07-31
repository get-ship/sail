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
    // Locate `.grow(` / `.shrink(` near the diagnostic offset.
    let offset = line_col_to_offset(source, d.line, d.col);
    let (method, arg) = find_style_call_at(source, offset)?;
    let attr = method; // grow / shrink
    let node = find_host_for_offset(index, offset)?;
    // Strip the chain segment and add attr.
    let open = &source[node.open_span.start..node.open_span.end];
    let new_open = strip_style_call_from_open(open, method, &arg)?;
    let attr_val = if arg == "1" || arg.is_empty() {
        None // boolean presence
    } else {
        Some(arg.clone())
    };
    let new_open = insert_attr(&new_open, attr, attr_val.as_deref());
    Some(FixPlan {
        rule: "sail-participation-style".into(),
        line: d.line,
        col: d.col,
        message: format!("move style.{method}(…) to `{attr}` attr"),
        ops: vec![],
        text_patch: Some(TextPatch {
            start: node.open_span.start,
            end: node.open_span.end,
            replacement: new_open,
        }),
        actor: None,
    })
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
    // Pull display(…) expression from sx if present; otherwise wrap with display(none).
    let (inner_without_display, display_expr) = split_display_from_element(&full, &node.tag)?;
    let wrap = format!("<div sx={{{display_expr}}}>{inner_without_display}</div>");
    Some(FixPlan {
        rule: "sail-display-on-layout".into(),
        line: d.line,
        col: d.col,
        message: format!("nest <{}> inside a plain host for style.display", node.tag),
        ops: vec![EditOp::Replace {
            node_id: node.id.clone(),
            fragment: wrap,
        }],
        text_patch: Some(TextPatch {
            start: node.full_span.start,
            end: node.full_span.end,
            replacement: format!("<div sx={{{display_expr}}}>{inner_without_display}</div>"),
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
    // If sx already contains `style.setter(...)`, replace the existing call.
    let sx_key = "sx={";
    let sx_at = open.find(sx_key)?;
    let expr_start = sx_at + sx_key.len();
    let mut depth = 1i32;
    let bytes = open.as_bytes();
    let mut j = expr_start;
    while j < bytes.len() {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => {
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
    let expr = &open[expr_start..j];
    let needle = format!(".{setter}(");
    if let Some(rel) = expr.find(&needle) {
        let call_start = rel;
        let arg_start = call_start + needle.len();
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
        let new_expr = format!("{}{target}{}", &expr[..arg_start], &expr[k..]);
        let mut out = String::new();
        out.push_str(&open[..expr_start]);
        out.push_str(&new_expr);
        out.push_str(&open[j..]);
        return Some(out);
    }
    // No existing setter — append it.
    let call = format!("style.{setter}({target})");
    let new_expr = if expr.trim().starts_with("style.") {
        format!("{expr}.{setter}({target})")
    } else {
        call
    };
    let mut out = String::new();
    out.push_str(&open[..expr_start]);
    out.push_str(&new_expr);
    out.push_str(&open[j..]);
    Some(out)
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

fn find_style_call_at(source: &str, offset: usize) -> Option<(&'static str, String)> {
    // Search around offset for `.grow(` or `.shrink(`.
    let window_start = offset.saturating_sub(8);
    let window = &source[window_start..source.len().min(offset + 48)];
    for (method, _) in [("grow", "grow"), ("shrink", "shrink")] {
        let needle = format!(".{method}(");
        if let Some(rel) = window.find(&needle) {
            let abs = window_start + rel + needle.len();
            let mut depth = 1i32;
            let mut j = abs;
            let bytes = source.as_bytes();
            while j < bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            let arg = source[abs..j].trim().to_string();
                            return Some((method, arg));
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
        }
    }
    None
}

fn strip_style_call_from_open(open: &str, method: &str, arg: &str) -> Option<String> {
    // Remove `.method(arg)` or `style.method(arg)` from sx= chain.
    let patterns = [
        format!(".{method}({arg})"),
        format!("style.{method}({arg})"),
        format!(".{method}( {arg} )"),
    ];
    for p in &patterns {
        if open.contains(p.as_str()) {
            let mut s = open.replacen(p.as_str(), "", 1);
            // Clean `style.` alone left empty chain edge cases.
            s = s.replace("sx={style}", "/*sx-cleared*/");
            s = s.replace("sx={style.}", "sx={style");
            // style.pad(2). → already fine; style. → style needs a method
            if let Some(idx) = s.find("sx={style}") {
                // empty style product — drop sx attr entirely if empty chain.
                let _ = idx;
            }
            // Collapse double dots
            while s.contains("..") {
                s = s.replace("..", ".");
            }
            s = s.replace("sx={style.}", "sx={style.");
            // sx={style} with nothing after
            if s.contains("sx={style}") || s.contains("sx={}") {
                s = remove_attr_from_open(&s, "sx");
            }
            // sx={style.pad(2).} trailing dot
            s = s.replace(".)}", ")}");
            s = s.replace(". }", " }");
            return Some(s);
        }
    }
    // Try regex-less: find `.method(` and matching paren.
    let needle = format!(".{method}(");
    if let Some(start) = open.find(&needle) {
        let mut depth = 1i32;
        let mut j = start + needle.len();
        let bytes = open.as_bytes();
        while j < bytes.len() {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        let mut s = String::new();
                        s.push_str(&open[..start]);
                        s.push_str(&open[j + 1..]);
                        while s.contains("..") {
                            s = s.replace("..", ".");
                        }
                        s = s.replace(".)}", ")}");
                        if s.contains("sx={style}") || s.contains("sx={}") {
                            s = remove_attr_from_open(&s, "sx");
                        }
                        return Some(s);
                    }
                }
                _ => {}
            }
            j += 1;
        }
    }
    None
}

fn insert_attr(open: &str, name: &str, value: Option<&str>) -> String {
    if open.contains(&format!("{name}=")) || open.split_whitespace().any(|t| t == name) {
        return open.to_string();
    }
    let insert = match value {
        None => format!(" {name}"),
        Some(v) => format!(" {name}=\"{v}\""),
    };
    if let Some(gt) = open.rfind('>') {
        let self_close = open[..gt].trim_end().ends_with('/');
        if self_close {
            let slash = open[..gt].rfind('/').unwrap_or(gt);
            format!("{}{}{}>", &open[..slash], insert, &open[slash..])
        } else {
            format!("{}{}{}", &open[..gt], insert, &open[gt..])
        }
    } else {
        format!("{open}{insert}")
    }
}

fn remove_attr_from_open(open: &str, name: &str) -> String {
    // Remove name="…" / name={…} / bare name
    let mut s = open.to_string();
    // name="..."
    if let Some(start) = s.find(&format!("{name}=\"")) {
        if let Some(rel) = s[start + name.len() + 2..].find('"') {
            let end = start + name.len() + 2 + rel + 1;
            s.replace_range(start..end, "");
            return collapse_ws_open(&s);
        }
    }
    // name={...}
    if let Some(start) = s.find(&format!("{name}={{")) {
        let mut depth = 0i32;
        let bytes = s.as_bytes();
        let mut j = start + name.len() + 1; // at '{'
        while j < bytes.len() {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        s.replace_range(start..j + 1, "");
                        return collapse_ws_open(&s);
                    }
                }
                _ => {}
            }
            j += 1;
        }
    }
    s
}

fn collapse_ws_open(s: &str) -> String {
    let mut out = s.to_string();
    while out.contains("  ") {
        out = out.replace("  ", " ");
    }
    out = out.replace(" >", ">");
    out
}

fn split_display_from_element(full: &str, tag: &str) -> Option<(String, String)> {
    // Find sx={…} on the open tag and peel display(…) out of the chain.
    let gt = full.find('>')?;
    let open = &full[..gt + 1];
    let rest = &full[gt + 1..];
    let sx_key = "sx={";
    let sx_start = open.find(sx_key)?;
    let expr_start = sx_start + sx_key.len();
    let mut depth = 1i32;
    let bytes = open.as_bytes();
    let mut j = expr_start;
    while j < bytes.len() {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => {
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
    let expr = &open[expr_start..j];
    let (new_expr, display_expr) = peel_display(expr)?;
    let mut new_open = String::new();
    new_open.push_str(&open[..sx_start]);
    if new_expr.trim().is_empty() || new_expr.trim() == "style" {
        // drop sx
        new_open.push_str(&open[j + 1..]);
    } else {
        new_open.push_str("sx={");
        new_open.push_str(&new_expr);
        new_open.push('}');
        new_open.push_str(&open[j + 1..]);
    }
    new_open = collapse_ws_open(&new_open);
    let inner = format!("{new_open}{rest}");
    // Ensure close tag still matches
    let _ = tag;
    Some((inner, display_expr))
}

fn peel_display(expr: &str) -> Option<(String, String)> {
    // style.display(none).pad(2) → (style.pad(2), style.display(none))
    // style.pad(2).display(none) → same
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
    // include leading `style.` if this is the only call, or `.` before display
    let call_start = if disp >= 6 && &expr[disp - 6..disp] == "style." {
        disp - 6
    } else if disp > 0 && expr.as_bytes()[disp - 1] == b'.' {
        disp - 1
    } else {
        disp
    };
    let call_end = j + 1; // after ')'
    let call = format!("style.{}", &expr[disp..call_end]);
    let mut new_expr = String::new();
    new_expr.push_str(&expr[..call_start]);
    new_expr.push_str(&expr[call_end..]);
    while new_expr.contains("..") {
        new_expr = new_expr.replace("..", ".");
    }
    if new_expr.starts_with('.') {
        new_expr = format!("style{new_expr}");
    }
    if new_expr.trim().is_empty() {
        new_expr.clear();
    }
    Some((new_expr, call))
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
    fn participation_strips_grow() {
        let open = r#"<div sx={style.pad(2).grow(1).bg(card)}>"#;
        let next = strip_style_call_from_open(open, "grow", "1").unwrap();
        assert!(!next.contains("grow"));
        assert!(next.contains("pad(2)"));
        let with_attr = insert_attr(&next, "grow", None);
        assert!(with_attr.contains(" grow") || with_attr.contains(" grow>"));
    }
}
