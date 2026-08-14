//! Apply a batch of edit ops to source (atomic, pre-validated by caller).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use super::fix::{self, FixPlan};
use super::node::{EditNode, build_node_index, model_revision};
use super::ops::EditOp;
use super::validate::{ValidateOutcome, validate_source};
use crate::LintOverride;
use crate::diagnostics::Diagnostic;

/// Result of applying an edit batch (in memory; write is caller's job).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditResult {
    pub applied: bool,
    pub model_revision: String,
    pub source: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DiagJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub auto_fixed: Vec<FixPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagJson {
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    pub message: String,
    pub line: usize,
    pub col: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_col: Option<usize>,
    /// Machine-readable rewrite (sail_edit op payload) when confluent.
    /// Surfaced by `sail check --fix suggest` / MCP `sail_check`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewrite: Option<serde_json::Value>,
}

impl From<&Diagnostic> for DiagJson {
    fn from(d: &Diagnostic) -> Self {
        Self {
            severity: match d.severity {
                crate::DiagSeverity::Error => "error".into(),
                crate::DiagSeverity::Warning => "warning".into(),
                crate::DiagSeverity::Fix => "fix".into(),
            },
            rule: d.rule.clone(),
            message: d.message.clone(),
            line: d.line,
            col: d.col,
            end_line: Some(d.end_line),
            end_col: Some(d.end_col),
            rewrite: None,
        }
    }
}

/// Attach rewrite payloads from FixPlans onto matching DiagJson rows.
pub fn attach_rewrites(diags: &mut [DiagJson], plans: &[FixPlan]) {
    for d in diags.iter_mut() {
        if d.rewrite.is_some() {
            continue;
        }
        if let Some(plan) = plans.iter().find(|p| {
            p.line == d.line && p.col == d.col && d.rule.as_deref() == Some(p.rule.as_str())
        }) {
            if let Ok(v) = serde_json::to_value(plan) {
                d.rewrite = Some(v);
            }
        }
    }
}

/// Apply ops to source without writing. Returns diagnostics if invalid.
pub fn apply_ops_to_source(
    source: &str,
    file: &str,
    ops: &[EditOp],
    path: Option<&Path>,
    autofix: AutofixMode,
    actor: Option<&str>,
) -> EditResult {
    apply_ops_inner(source, file, ops, path, autofix, actor, &[])
}

/// Autofix mode for edit/check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutofixMode {
    Off,
    Suggest,
    Apply,
}

impl AutofixMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "off" | "false" | "0" => Some(Self::Off),
            "suggest" | "hint" => Some(Self::Suggest),
            "apply" | "true" | "1" => Some(Self::Apply),
            _ => None,
        }
    }
}

/// Full apply with optional disk write by caller using returned source.
pub fn apply_ops(
    source: &str,
    file: &str,
    ops: &[EditOp],
    path: Option<&Path>,
    autofix: AutofixMode,
    actor: Option<&str>,
) -> EditResult {
    apply_ops_to_source(source, file, ops, path, autofix, actor)
}

fn apply_ops_inner(
    source: &str,
    file: &str,
    ops: &[EditOp],
    path: Option<&Path>,
    autofix: AutofixMode,
    actor: Option<&str>,
    overrides: &[LintOverride],
) -> EditResult {
    let mut auto_fixed: Vec<FixPlan> = Vec::new();
    let mut working = source.to_string();

    // Optional auto-fix first (suggest collects without write; apply mutates).
    if matches!(autofix, AutofixMode::Suggest | AutofixMode::Apply) {
        let plans = fix::suggest_fixes_for_source(&working, file, path);
        if autofix == AutofixMode::Apply {
            let patches: Vec<_> = plans
                .iter()
                .filter_map(|p| {
                    p.text_patch
                        .as_ref()
                        .map(|t| (t.start, t.end, t.replacement.clone()))
                })
                .collect();
            if !patches.is_empty() {
                let (next, skipped) = fix::apply_text_patches(&working, &patches);
                if skipped > 0 {
                    return EditResult {
                        applied: false,
                        model_revision: model_revision(source),
                        source: source.to_string(),
                        diagnostics: vec![],
                        auto_fixed: vec![],
                        message: Some(format!(
                            "autofix aborted: {skipped} text patch(es) had invalid spans"
                        )),
                    };
                }
                working = next;
            }
            for mut p in plans {
                p.actor = actor.map(|a| a.to_string());
                auto_fixed.push(p);
            }
        } else {
            // suggest: attach plans without mutating
            auto_fixed.extend(plans);
        }
    }

    // Apply user ops in order; rebuild index after each for stable addressing.
    for op in ops {
        match apply_one(&working, file, op) {
            Ok(next) => working = next,
            Err(msg) => {
                return EditResult {
                    applied: false,
                    model_revision: model_revision(source),
                    source: source.to_string(),
                    diagnostics: vec![],
                    auto_fixed: vec![],
                    message: Some(msg),
                };
            }
        }
    }

    // Pre-write validation
    let outcome: ValidateOutcome = validate_source(&working, path, overrides);
    if outcome.has_errors {
        let diags: Vec<DiagJson> = outcome.diagnostics.iter().map(DiagJson::from).collect();
        return EditResult {
            applied: false,
            model_revision: model_revision(source),
            source: source.to_string(),
            diagnostics: diags,
            auto_fixed: vec![],
            message: Some("edit rejected: would produce uncompilable Sail (file untouched)".into()),
        };
    }

    EditResult {
        applied: true,
        model_revision: model_revision(&working),
        source: working,
        diagnostics: outcome.diagnostics.iter().map(DiagJson::from).collect(),
        auto_fixed,
        message: None,
    }
}

fn apply_one(source: &str, file: &str, op: &EditOp) -> Result<String, String> {
    let index = build_node_index(source, file)?;
    match op {
        EditOp::Replace { node_id, fragment } => {
            let node = require_node(&index, node_id)?;
            validate_fragment(fragment)?;
            Ok(replace_range(
                source,
                node.full_span.start,
                node.full_span.end,
                fragment,
            ))
        }
        EditOp::Insert {
            parent_node_id,
            position,
            fragment,
        } => {
            let parent = require_node(&index, parent_node_id)?;
            validate_fragment(fragment)?;
            let pos = position.resolve(parent.children.len())?;
            let insert_at = child_insert_offset(source, parent, &index, pos)?;
            Ok(replace_range(source, insert_at, insert_at, fragment))
        }
        EditOp::Wrap {
            node_id,
            tag,
            attrs,
        } => {
            let node = require_node(&index, node_id)?;
            validate_wrap_tag(tag)?;
            let body = source
                .get(node.full_span.start..node.full_span.end)
                .ok_or_else(|| "node span out of range".to_string())?;
            let open_attrs = attrs_to_open_suffix(attrs.as_ref());
            let wrapped = format!("<{tag}{open_attrs}>{body}</{tag}>");
            Ok(replace_range(
                source,
                node.full_span.start,
                node.full_span.end,
                &wrapped,
            ))
        }
        EditOp::SetAttr {
            node_id,
            name,
            value,
        } => {
            let node = require_node(&index, node_id)?;
            let open = source
                .get(node.open_span.start..node.open_span.end)
                .ok_or_else(|| "open span out of range".to_string())?;
            let next_open = set_attr_in_open(open, name, value.as_deref());
            Ok(replace_range(
                source,
                node.open_span.start,
                node.open_span.end,
                &next_open,
            ))
        }
        EditOp::RemoveAttr { node_id, name } => {
            let node = require_node(&index, node_id)?;
            let open = source
                .get(node.open_span.start..node.open_span.end)
                .ok_or_else(|| "open span out of range".to_string())?;
            let next_open = remove_attr_in_open(open, name);
            Ok(replace_range(
                source,
                node.open_span.start,
                node.open_span.end,
                &next_open,
            ))
        }
        EditOp::SetSx {
            node_id,
            setter,
            args,
        } => {
            validate_sx_setter(setter, args)?;
            let node = require_node(&index, node_id)?;
            let open = source
                .get(node.open_span.start..node.open_span.end)
                .ok_or_else(|| "open span out of range".to_string())?;
            // Product Style is a tag-position call on the open tag.
            if let Some((rel_lo, rel_hi, expr)) = try_tag_style_in_open(open) {
                let merged = merge_style_setter(&expr, setter, args)?;
                let abs_lo = node.open_span.start + rel_lo;
                let abs_hi = node.open_span.start + rel_hi;
                Ok(replace_range(source, abs_lo, abs_hi, &merged))
            } else {
                let insert = format!(" {}", style_stmt(setter, args));
                let open_inner = open.trim_end();
                let cut = if open_inner.ends_with("/>") {
                    open.len() - (open.len() - open_inner.len()) - 2
                } else if open_inner.ends_with('>') {
                    open.len() - (open.len() - open_inner.len()) - 1
                } else {
                    return Err("setSx: open tag has no closing `>`".into());
                };
                let abs = node.open_span.start + cut;
                Ok(replace_range(source, abs, abs, &insert))
            }
        }
        EditOp::Move {
            node_id,
            new_parent_id,
            position,
        } => {
            if node_id == new_parent_id {
                return Err("cannot move a node under itself".into());
            }
            let node = require_node(&index, node_id)?.clone();
            let parent = require_node(&index, new_parent_id)?;
            // Participation: refuse moving a node into its own descendant.
            if is_descendant(&index, node_id, new_parent_id) {
                return Err("move would create a cycle (new parent is a descendant)".into());
            }
            let body = source
                .get(node.full_span.start..node.full_span.end)
                .ok_or_else(|| "node span out of range".to_string())?
                .to_string();
            // Remove first (from original source), then insert into resulting text.
            // Adjust insert position if parent is after the removed node.
            let without = replace_range(source, node.full_span.start, node.full_span.end, "");
            // Rebuild index on without to find parent.
            let index2 = build_node_index(&without, file)?;
            // Parent id may change after removal if content hash differs — locate by path-ish:
            // Prefer same tag+static identity: find node whose full span still contains children.
            let parent2 = index2
                .get(new_parent_id)
                .or_else(|| {
                    // After removal, node ids rehash. Match by original open span shift.
                    let removed_before = node.full_span.end;
                    let shift = node.full_span.end - node.full_span.start;
                    index2.values().find(|n| {
                        n.tag == parent.tag
                            && n.open_span.start
                                == if parent.open_span.start >= removed_before {
                                    parent.open_span.start - shift
                                } else {
                                    parent.open_span.start
                                }
                    })
                })
                .ok_or_else(|| format!("new parent `{new_parent_id}` lost after cut"))?;
            let pos = position.resolve(parent2.children.len())?;
            let insert_at = child_insert_offset(&without, parent2, &index2, pos)?;
            Ok(replace_range(&without, insert_at, insert_at, &body))
        }
    }
}

fn require_node<'a>(
    index: &'a BTreeMap<String, EditNode>,
    id: &str,
) -> Result<&'a EditNode, String> {
    index
        .get(id)
        .ok_or_else(|| format!("unknown nodeId `{id}`"))
}

fn is_descendant(index: &BTreeMap<String, EditNode>, ancestor: &str, maybe_desc: &str) -> bool {
    let mut cur = Some(maybe_desc.to_string());
    while let Some(id) = cur {
        if id == ancestor {
            return true;
        }
        cur = index.get(&id).and_then(|n| n.parent_id.clone());
    }
    false
}

fn replace_range(source: &str, start: usize, end: usize, rep: &str) -> String {
    let mut out = String::with_capacity(source.len() + rep.len());
    out.push_str(&source[..start.min(source.len())]);
    out.push_str(rep);
    out.push_str(&source[end.min(source.len())..]);
    out
}

fn child_insert_offset(
    source: &str,
    parent: &EditNode,
    index: &BTreeMap<String, EditNode>,
    pos: usize,
) -> Result<usize, String> {
    if pos == 0 {
        // After open tag
        return Ok(parent.open_span.end);
    }
    if pos > parent.children.len() {
        return Err("position past children".into());
    }
    if pos == parent.children.len() {
        // Before close tag
        let close_start = parent.full_span.end.saturating_sub(
            // </tag>
            parent.tag.len() + 3,
        );
        // Scan back for `</`
        let full = source
            .get(parent.full_span.start..parent.full_span.end)
            .ok_or_else(|| "parent span out of range".to_string())?;
        if let Some(rel) = full.rfind(&format!("</{}", parent.tag)) {
            return Ok(parent.full_span.start + rel);
        }
        return Ok(close_start);
    }
    let child_id = &parent.children[pos.saturating_sub(1)];
    let child = index
        .get(child_id)
        .ok_or_else(|| format!("missing child {child_id}"))?;
    Ok(child.full_span.end)
}

fn validate_fragment(fragment: &str) -> Result<(), String> {
    let trimmed = fragment.trim();
    if trimmed.is_empty() {
        return Err("fragment must be non-empty".into());
    }
    // Parse as author template snippet wrapped in a synthetic root if needed.
    let wrapped = format!("<_edit_root>{trimmed}</_edit_root>");
    match crate::parse::parse_author(&wrapped) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("fragment parse failed: {e}")),
    }
}

fn validate_wrap_tag(tag: &str) -> Result<(), String> {
    let t = tag.trim();
    if t.is_empty() {
        return Err("wrap tag must be non-empty".into());
    }
    // Layout keywords or HTML/custom element names.
    let ok = sail_lint::rules::layout::is_layout_tag(t)
        || t.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !ok {
        return Err(format!("invalid wrap tag `{t}`"));
    }
    Ok(())
}

fn attrs_to_open_suffix(attrs: Option<&serde_json::Value>) -> String {
    let Some(v) = attrs else {
        return String::new();
    };
    let Some(obj) = v.as_object() else {
        return String::new();
    };
    let mut s = String::new();
    for (k, val) in obj {
        if val.is_null() {
            s.push(' ');
            s.push_str(k);
            continue;
        }
        if let Some(b) = val.as_bool() {
            if b {
                s.push(' ');
                s.push_str(k);
            }
            continue;
        }
        let vs = match val {
            serde_json::Value::String(x) => x.clone(),
            other => other.to_string(),
        };
        s.push(' ');
        s.push_str(k);
        s.push_str("=\"");
        s.push_str(&vs.replace('"', "&quot;"));
        s.push('"');
    }
    s
}

/// Closed-vocab check for setSx.
pub fn validate_sx_setter(setter: &str, args: &[serde_json::Value]) -> Result<(), String> {
    let name = setter.trim();
    if name.is_empty() {
        return Err("setSx requires non-empty `setter`".into());
    }
    // Refusals (layout-as-sx)
    for (atom, instead) in sail_lint::style_vocab::refusals() {
        if name == atom || name.replace('_', "-") == atom {
            return Err(format!(
                "setSx `{name}` is off-vocab (refused): {instead}. Allowed setters: {}",
                allowed_setter_names().join(", ")
            ));
        }
    }
    // grow/shrink are Style participation — allowed on setSx / Style.
    if let Some((concern, teaching)) = sail_lint::style::retired_replacement(name) {
        return Err(format!(
            "setSx `{name}` is retired — use `{teaching}` (`{concern}`). Allowed setters: {}",
            allowed_setter_names().join(", ")
        ));
    }
    if sail_lint::style::find_vocab(name).is_none() {
        return Err(format!(
            "setSx `{name}` is off-vocab. Allowed setters: {}",
            allowed_setter_names().join(", ")
        ));
    }
    // Optional plains emit scale/keyword check on first string/number arg.
    let Some(spec) = sail_lint::style::plains::find_plain(name) else {
        return Ok(());
    };
    if let Some(scale) = spec.scale {
        if let Some(arg0) = args.first() {
            let tok = arg_to_token(arg0);
            if let Some(tok) = tok {
                let allowed = sail_lint::scales::values(scale);
                let kw_ok = spec.keywords.iter().any(|k| *k == tok.as_str());
                let scale_ok = allowed.iter().any(|v| v == &tok)
                    || (spec.negatable
                        && tok.starts_with('-')
                        && allowed.iter().any(|v| v.as_str() == &tok[1..]));
                if !kw_ok && !scale_ok && !allowed.is_empty() {
                    let mut v: Vec<String> =
                        spec.keywords.iter().map(|k| (*k).to_string()).collect();
                    v.extend(allowed);
                    return Err(format!(
                        "setSx `{name}({tok})` value not in scale `{scale}`. Allowed: {}",
                        v.join(", ")
                    ));
                }
            }
        }
    }
    Ok(())
}

fn allowed_setter_names() -> Vec<&'static str> {
    sail_lint::style::vocab_rows()
        .iter()
        .map(|r| r.name)
        .collect()
}

fn arg_to_token(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn format_sx_args(args: &[serde_json::Value]) -> String {
    args.iter()
        .map(|a| match a {
            serde_json::Value::String(s) => {
                if s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    s.clone()
                } else {
                    format!("'{s}'")
                }
            }
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
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

fn style_stmt(setter: &str, args: &[serde_json::Value]) -> String {
    let args_s = format_sx_args(args);
    if is_animate_trigger(setter) {
        format!("style({{ animate: {setter}({args_s}) }})")
    } else {
        format!("style({{ {setter}({args_s}) }})")
    }
}

/// Merge a setter into an existing `style({…})` expression (flat concern chain).
fn merge_style_setter(
    expr: &str,
    setter: &str,
    args: &[serde_json::Value],
) -> Result<String, String> {
    let t = expr.trim();
    let args_s = format_sx_args(args);
    if let Some(rest) = t.strip_prefix("style(").and_then(|r| r.strip_suffix(')')) {
        let body = rest.trim();
        if body.starts_with('{') && body.ends_with('}') {
            let inner = body[1..body.len() - 1].trim();
            if is_animate_trigger(setter) {
                return Ok(merge_into_animate(inner, setter, &args_s));
            }
            return Ok(merge_into_flat_chain(inner, setter, &args_s));
        }
    }
    Ok(style_stmt(setter, args))
}

const STYLE_OBJECT_KEYS: &[&str] = &["when", "match", "compound", "animate"];

fn merge_into_flat_chain(inner: &str, setter: &str, args_s: &str) -> String {
    let call = format!("{setter}({args_s})");
    let key_at = first_object_key_at(inner);
    let (chain, keys) = if let Some(idx) = key_at {
        (
            inner[..idx].trim().trim_end_matches(',').trim(),
            inner[idx..].trim(),
        )
    } else {
        (inner.trim(), "")
    };
    let new_chain = if chain.is_empty() {
        call
    } else {
        format!("{chain}.{call}")
    };
    if keys.is_empty() {
        format!("style({{ {new_chain} }})")
    } else {
        format!("style({{ {new_chain}, {keys} }})")
    }
}

fn merge_into_animate(inner: &str, setter: &str, args_s: &str) -> String {
    let call = format!("{setter}({args_s})");
    if let Some(idx) = find_object_key(inner, "animate") {
        let after_key = idx + "animate:".len();
        let (val_start, val_end) = object_value_span(inner, after_key);
        let val = inner[val_start..val_end].trim();
        let new_val = if val.is_empty() {
            call
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
            if !new_val.is_empty() {
                out.push(' ');
            }
            out.push_str(tail);
        }
        out.push_str(" })");
        out
    } else {
        let sep = if inner.is_empty() { "" } else { ", " };
        format!("style({{ {inner}{sep}animate: {call} }})")
    }
}

fn first_object_key_at(inner: &str) -> Option<usize> {
    STYLE_OBJECT_KEYS
        .iter()
        .filter_map(|k| find_object_key(inner, k))
        .min()
}

fn find_object_key(inner: &str, ch: &str) -> Option<usize> {
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
            // word boundary before key
            if i == 0 || bytes[i - 1].is_ascii_whitespace() || bytes[i - 1] == b',' {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn object_value_span(inner: &str, after_key: usize) -> (usize, usize) {
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

/// Locate tag-position `style(…)` call inside an open tag string.
fn try_tag_style_in_open(open: &str) -> Option<(usize, usize, String)> {
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
                            return try_block_style_in_body(&open[j..])
                                .map(|(lo, hi, e)| (j + lo, j + hi, e));
                        }
                    }
                }
                j += 1;
            }
        }
    }
    None
}

/// Consume a Style product expression starting at `body` (used for tag-call
/// tails after reserved `style`, and residual body scans).
fn try_block_style_in_body(body: &str) -> Option<(usize, usize, String)> {
    let bytes = body.as_bytes();
    let mut start = 0;
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    if start >= bytes.len() {
        return None;
    }
    if bytes[start] == b'<' || bytes[start] == b'{' {
        return None;
    }
    let rest = &body[start..];
    let t = rest.trim_start();
    if !(t.starts_with("style(") || t.starts_with("style.") || t.starts_with("match(")) {
        return None;
    }
    // balanced consume
    let mut i = 0;
    let mut paren = 0i32;
    let mut brace = 0i32;
    let mut bracket = 0i32;
    let b = rest.as_bytes();
    while i < b.len() {
        match b[i] {
            b'(' => paren += 1,
            b')' => {
                paren -= 1;
                if paren == 0 && brace == 0 && bracket == 0 {
                    i += 1;
                    // allow .when / motion chain
                    while i < b.len() && b[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    if i < b.len() && b[i] == b'.' {
                        i += 1;
                        continue;
                    }
                    break;
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
    if paren != 0 || brace != 0 || bracket != 0 {
        return None;
    }
    let expr = rest[..i].trim_end().to_string();
    Some((start, start + i, expr))
}

fn set_attr_in_open(open: &str, name: &str, value: Option<&str>) -> String {
    let without = remove_attr_in_open(open, name);
    let piece = match value {
        None => format!(" {name}"),
        Some(v) => format!(" {name}=\"{}\"", v.replace('"', "&quot;")),
    };
    if let Some(gt) = without.rfind('>') {
        let self_close = without[..gt].trim_end().ends_with('/');
        if self_close {
            let slash = without[..gt].rfind('/').unwrap_or(gt);
            format!("{}{}{}>", &without[..slash], piece, &without[slash..])
        } else {
            format!("{}{}{}", &without[..gt], piece, &without[gt..])
        }
    } else {
        format!("{without}{piece}")
    }
}

fn remove_attr_in_open(open: &str, name: &str) -> String {
    let mut s = open.to_string();
    // name="..."
    let key_eq_q = format!("{name}=\"");
    if let Some((ws_start, key_start)) = find_attr_key(&s, &key_eq_q) {
        let value_start = key_start + key_eq_q.len();
        if let Some(rel) = s[value_start..].find('"') {
            let end = value_start + rel + 1;
            s.replace_range(ws_start..end, "");
            return collapse_ws(&s);
        }
    }
    // name={...}
    let key_eq_b = format!("{name}={{");
    if let Some((ws_start, key_start)) = find_attr_key(&s, &key_eq_b) {
        let mut depth = 0i32;
        let bytes = s.as_bytes();
        let mut j = key_start + name.len() + 1; // at '{'
        while j < bytes.len() {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        s.replace_range(ws_start..j + 1, "");
                        return collapse_ws(&s);
                    }
                }
                _ => {}
            }
            j += 1;
        }
    }
    // bare boolean attr
    if let Some((ws_start, key_start)) = find_attr_key(&s, name) {
        let after = key_start + name.len();
        let after_ok = after >= s.len()
            || matches!(
                s.as_bytes()[after],
                b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/'
            );
        // bare only when not followed by '='
        if after_ok && s.as_bytes().get(after) != Some(&b'=') {
            s.replace_range(ws_start..after, "");
            return collapse_ws(&s);
        }
    }
    s
}

/// Find attribute key `key` in open tag. Returns `(whitespace_start, key_start)`.
fn find_attr_key(open: &str, key: &str) -> Option<(usize, usize)> {
    let mut from = 0;
    while let Some(rel) = open[from..].find(key) {
        let abs = from + rel;
        let before_ok = abs == 0
            || open.as_bytes()[abs - 1].is_ascii_whitespace()
            || open.as_bytes()[abs - 1] == b'<';
        if before_ok {
            let ws_start = if abs > 0 && open.as_bytes()[abs - 1].is_ascii_whitespace() {
                abs - 1
            } else {
                abs
            };
            return Some((ws_start, abs));
        }
        from = abs + 1;
    }
    None
}

fn collapse_ws(s: &str) -> String {
    let mut out = s.to_string();
    while out.contains("  ") {
        out = out.replace("  ", " ");
    }
    out = out.replace(" >", ">");
    out = out.replace(" />", "/>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn root_id(src: &str) -> String {
        let idx = build_node_index(src, "t.sail").unwrap();
        idx.values()
            .find(|n| n.parent_id.is_none())
            .unwrap()
            .id
            .clone()
    }

    #[test]
    fn set_attr_and_set_sx() {
        // Participation is Style (not bare attr). SetSx grows + pads.
        let src = r#"<column gap="2"><div>x</div></column>"#;
        let id = root_id(src);
        let r = apply_ops_to_source(
            src,
            "t.sail",
            &[EditOp::SetSx {
                node_id: id.clone(),
                setter: "grow".into(),
                args: vec![json!(1)],
            }],
            None,
            AutofixMode::Off,
            None,
        );
        assert!(r.applied, "{:?}", r.message);
        assert!(
            r.source.contains("style({") && r.source.contains("grow(1)"),
            "expected grow Style: {}",
            r.source
        );

        let r2 = apply_ops_to_source(
            &r.source,
            "t.sail",
            &[EditOp::SetSx {
                node_id: root_id(&r.source),
                setter: "pad".into(),
                args: vec![json!(2)],
            }],
            None,
            AutofixMode::Off,
            None,
        );
        assert!(r2.applied, "{:?}", r2.message);
        assert!(
            r2.source.contains("pad(2)") && r2.source.contains("style({"),
            "expected pad merge: {}",
            r2.source
        );
    }

    #[test]
    fn off_vocab_setsx_rejected() {
        let src = r#"<div>x</div>"#;
        let r = apply_ops_to_source(
            src,
            "t.sail",
            &[EditOp::SetSx {
                node_id: root_id(src),
                setter: "notARealSetter".into(),
                args: vec![json!("x")],
            }],
            None,
            AutofixMode::Off,
            None,
        );
        assert!(!r.applied);
        assert!(
            r.message.as_deref().unwrap_or("").contains("off-vocab")
                || r.message
                    .as_deref()
                    .unwrap_or("")
                    .contains("Allowed setters"),
            "{:?}",
            r.message
        );
    }

    #[test]
    fn replace_preserves_validity() {
        let src = r#"<column><div>old</div></column>"#;
        let idx = build_node_index(src, "t.sail").unwrap();
        let div = idx.values().find(|n| n.tag == "div").unwrap();
        let r = apply_ops_to_source(
            src,
            "t.sail",
            &[EditOp::Replace {
                node_id: div.id.clone(),
                fragment: r#"<span>new</span>"#.into(),
            }],
            None,
            AutofixMode::Off,
            None,
        );
        assert!(r.applied, "{:?}", r.message);
        assert!(r.source.contains("<span>new</span>"));
        assert!(!r.source.contains("old"));
    }
}
