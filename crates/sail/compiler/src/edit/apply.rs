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
            let call = format_sx_call(setter, args);
            let next_open = set_sx_in_open(open, &call);
            Ok(replace_range(
                source,
                node.open_span.start,
                node.open_span.end,
                &next_open,
            ))
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
    for (atom, instead) in sail_lint::sx_vocab::refusals() {
        if name == atom || name.replace('_', "-") == atom {
            return Err(format!(
                "setSx `{name}` is off-vocab (refused): {instead}. Allowed setters: {}",
                allowed_setter_names().join(", ")
            ));
        }
    }
    // grow/shrink are participation — refuse on setSx product path
    if matches!(name, "grow" | "shrink") {
        return Err(format!(
            "setSx `{name}` is layout participation — use setAttr `{name}` instead. \
             Allowed setters: {}",
            allowed_setter_names().join(", ")
        ));
    }
    let known = sail_lint::style::setters::setters();
    let Some(spec) = known.iter().find(|s| s.name == name) else {
        return Err(format!(
            "setSx `{name}` is off-vocab. Allowed setters: {}",
            allowed_setter_names().join(", ")
        ));
    };
    // Optional scale/keyword check on first string/number arg
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
    sail_lint::style::setters::setters()
        .iter()
        .filter(|s| !matches!(s.name, "grow" | "shrink"))
        .map(|s| s.name)
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

fn format_sx_call(setter: &str, args: &[serde_json::Value]) -> String {
    let args_s: Vec<String> = args
        .iter()
        .map(|a| match a {
            serde_json::Value::String(s) => {
                // bare tokens unquoted when identifier-like
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
        .collect();
    format!("style.{setter}({})", args_s.join(", "))
}

fn set_sx_in_open(open: &str, call: &str) -> String {
    // If sx already present, append `.setter(…)` onto chain; else add style={call}.
    const STYLE_KEY: &str = "style={";
    if let Some(sx_at) = open.find(STYLE_KEY) {
        let expr_start = sx_at + STYLE_KEY.len();
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
        if depth == 0 {
            let expr = &open[expr_start..j];
            // call is `style.foo(…)` — if expr is `style.bar(…)` append `.foo(…)`
            let method_part = call.strip_prefix("style.").unwrap_or(call);
            let new_expr = if expr.trim().is_empty() {
                call.to_string()
            } else if expr.trim().starts_with("style.") {
                format!("{expr}.{method_part}")
            } else {
                // bare ref — wrap: style chain can't merge; replace
                call.to_string()
            };
            let mut out = String::new();
            out.push_str(&open[..expr_start]);
            out.push_str(&new_expr);
            out.push_str(&open[j..]);
            return out;
        }
    }
    // Insert sx= before closing >
    let insert = format!(" style={{{call}}}");
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
        let src = r#"<column gap="2"><div>x</div></column>"#;
        let id = root_id(src);
        let r = apply_ops_to_source(
            src,
            "t.sail",
            &[EditOp::SetAttr {
                node_id: id.clone(),
                name: "grow".into(),
                value: None,
            }],
            None,
            AutofixMode::Off,
            None,
        );
        assert!(r.applied, "{:?}", r.message);
        assert!(r.source.contains(" grow") || r.source.contains(" grow>"));

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
        assert!(r2.source.contains("style.pad(2)"));
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
