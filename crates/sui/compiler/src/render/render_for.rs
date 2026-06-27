//! `{#for}` rendering — kept separate from `{#if}` / `{#match}` because
//! the body-iteration plus snapshot/restore logic is large enough to
//! push `control_flow.rs` past the file cap.
//!
//! The walker's snapshot/restore makes every iteration share the same
//! `b<N>` key range inside the body. Phase E-prep adds the same
//! treatment for the registry counter, so the registry's index advances
//! exactly once across the body regardless of iteration count.

use serde_json::Value;

use crate::ir::{ForInstr, Instr};

use crate::escape::escape_attr_value;
use super::eval::{eval, stringify};
use super::template_helpers::{
    close_active, close_for_item, empty_template, open_active, open_for_item,
};
use super::walker::Walker;

pub(super) fn render_for(walker: &mut Walker<'_>, instr: &ForInstr) {
    let key = walker.next_key();
    let value_result = eval(&instr.iterable.source, &walker.scope);

    let array = match &value_result {
        Ok(Value::Array(a)) if !a.is_empty() => Some(a.clone()),
        _ => None,
    };

    if let Some(array) = array {
        open_active(walker, "for", &key);
        let body_start = walker.keys.snapshot();
        let body_start_idx = walker.reg_idx;
        let mut body_end = body_start;
        let mut body_end_idx = body_start_idx;
        for (idx, item) in array.iter().enumerate() {
            if idx > 0 {
                walker.keys.restore(body_start);
                walker.reg_idx = body_start_idx;
            }
            let prior_var = walker
                .scope
                .insert(instr.var_name.clone(), item.clone());
            let prior_idx = instr.index_name.as_ref().map(|n| {
                (
                    n.clone(),
                    walker.scope.insert(n.clone(), Value::from(idx as i64)),
                )
            });

            let key_attr_value = instr.key.as_ref().map(|k| {
                eval(&k.source, &walker.scope)
                    .map(|v| stringify_key(&v))
                    .unwrap_or_default()
            });

            walk_iteration_body(walker, &instr.body, key_attr_value.as_deref());

            body_end = walker.keys.snapshot();
            body_end_idx = walker.reg_idx;

            restore_scope(walker, &instr.var_name, prior_var, prior_idx);
        }

        walker.keys.restore(body_end);
        walker.reg_idx = body_end_idx;
        close_active(walker, "for");

        if let Some(else_body) = &instr.else_body {
            walker.walk_block_silent(else_body);
        }
        return;
    }

    walker.walk_block_silent(&instr.body);

    let render_else = matches!(&value_result, Ok(Value::Array(a)) if a.is_empty())
        && instr.else_body.is_some();

    if render_else {
        let Some(else_body) = instr.else_body.as_ref() else {
            return;
        };
        open_active(walker, "for", &key);
        walker.walk_block(else_body);
        close_active(walker, "for");
    } else {
        if let Some(else_body) = &instr.else_body {
            walker.walk_block_silent(else_body);
        }
        empty_template(walker, "for", &key);
    }
}

fn restore_scope(
    walker: &mut Walker<'_>,
    var_name: &str,
    prior_var: Option<Value>,
    prior_idx: Option<(String, Option<Value>)>,
) {
    match prior_var {
        Some(v) => {
            walker.scope.insert(var_name.to_string(), v);
        }
        None => {
            walker.scope.remove(var_name);
        }
    }
    if let Some((name, prior)) = prior_idx {
        match prior {
            Some(v) => {
                walker.scope.insert(name, v);
            }
            None => {
                walker.scope.remove(&name);
            }
        }
    }
}

/// Render a `{#for}` iteration body, optionally injecting the
/// `key` attribute. When the body has a single element root, the
/// attribute attaches to that element; otherwise we wrap the iteration
/// in a `<div key="…" style="display:contents">` so
/// reconciliation has a per-iteration anchor (and the body still
/// renders — see `template_helpers.rs` for why `<template>` is wrong
/// for active branches).
fn walk_iteration_body(walker: &mut Walker<'_>, body: &[Instr], key_attr_value: Option<&str>) {
    let Some(key_val) = key_attr_value else {
        walker.walk_block(body);
        return;
    };

    let element_positions: Vec<usize> = body
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(i, Instr::Element(_)))
        .map(|(i, _)| i)
        .collect();

    if element_positions.len() == 1 {
        let target = element_positions[0];
        for (i, instr) in body.iter().enumerate() {
            if i == target {
                if let Instr::Element(el) = instr {
                    walker.walk_element_with_extra(el, &[("key", key_val)]);
                }
            } else {
                walker.walk_instr(instr);
            }
        }
        return;
    }

    open_for_item(walker, &escape_attr_value(key_val));
    walker.walk_block(body);
    close_for_item(walker);
}

/// Stringify a value for use as a `key` attribute. JSON
/// scalars use their evaluator-string form (`42` → `"42"`, `"alice"` →
/// `"alice"`, `true` → `"true"`, `null` → `"null"`); objects/arrays
/// fall back to a stable JSON dump so reconciliation has *something* to
/// key on (the authoring smell is real but the framework allows it).
fn stringify_key(v: &Value) -> String {
    match v {
        Value::Object(_) | Value::Array(_) => {
            serde_json::to_string(v).unwrap_or_default()
        }
        _ => stringify(v),
    }
}
