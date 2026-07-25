//! Paint-slot clash detection (fail closed).
//!
//! Detection keys on SEMANTIC PAINT SLOTS — the per-call slots
//! [`super::setters::op_slots`] narrows from each setter's declared
//! `SetterSpec::slots` union — never on lowered CSS declarations. Two
//! setters that touch the same slot (`border(1)` and `borderWidth` both
//! write `border-width`) clash under Chain even though their lowered decls
//! are produced by different code paths; slots are declared data, so
//! detection needs no lowering and cannot be blinded by lowering failures.
//!
//! Chain = unordered conjunction (duplicate slot = bug); List = ordered
//! composition (later shadows earlier, explicit precedence). `emit.rs`
//! (`emit_inner`) already walks List items in source order, so stylesheet
//! order implements the shadowing precedence — later rules win.

use std::collections::{HashMap, HashSet};

use super::emit::StyleEnv;
use super::ir::{Style, StyleDiagnostic};
use super::setters::op_slots;

/// Detect same paint slot written twice under the same guard path.
pub fn detect_clashes(style: &Style, env: &StyleEnv) -> Vec<StyleDiagnostic> {
    let mut writes: HashMap<(String, String), String> = HashMap::new();
    let mut diags = Vec::new();
    walk(
        style,
        env,
        "base",
        &mut writes,
        &mut HashSet::new(),
        &mut diags,
        &mut HashSet::new(),
    );
    diags
}

/// `writes` maps (paint slot, guard path) → source of the winning write.
/// `fresh` is the subset of `writes` keys written since the last List shadow
/// boundary: a hit on `fresh` is a real conjunction clash, while a hit only
/// on `writes` (inherited from an earlier List item) shadows silently.
fn walk(
    style: &Style,
    env: &StyleEnv,
    path: &str,
    writes: &mut HashMap<(String, String), String>,
    fresh: &mut HashSet<(String, String)>,
    diags: &mut Vec<StyleDiagnostic>,
    resolving: &mut HashSet<String>,
) {
    match style {
        Style::Empty => {}
        Style::Set(op) => {
            // Child-pack setters paint a descendant selector (`img` / `svg`), not
            // the host box — their width/height/display must not clash with host size.
            if matches!(op.setter.as_str(), "avatarMedia" | "glyphFill") {
                return;
            }
            // Key on the setter's per-call paint slots, not lowered decls:
            // cross-setter conflicts on one slot are caught without running
            // the lowering pipeline. Unknown setters produce no writes (the
            // resolve pass reports them separately).
            for slot in op_slots(op) {
                    let key = ((*slot).to_string(), path.to_string());
                    if fresh.contains(&key) {
                        diags.push(StyleDiagnostic::Clash {
                            slot: format!("{slot}@{path}"),
                            first: writes.get(&key).cloned().unwrap_or_default(),
                            second: op.source.clone(),
                        });
                    } else {
                        // First write — or a later List item shadowing an
                        // earlier one: overwrite, emit no diagnostic.
                        writes.insert(key.clone(), op.source.clone());
                        fresh.insert(key);
                    }
                }
        }
        Style::Chain(items) => {
            // Unordered conjunction: every item shares one shadow boundary, so
            // any duplicate (slot, guard path) inside the chain clashes.
            for i in items {
                walk(i, env, path, writes, fresh, diags, resolving);
            }
        }
        Style::List(items) => {
            // Ordered composition: each item restarts the fresh set from the
            // pre-List state, so duplicates WITHIN one item (its own Chain
            // semantics) still clash while duplicates ACROSS items shadow
            // silently. The shared `writes` map is overwritten by the later
            // item, matching emit.rs source-order emission.
            let base_fresh = fresh.clone();
            for i in items {
                let mut item_fresh = base_fresh.clone();
                walk(i, env, path, writes, &mut item_fresh, diags, resolving);
                fresh.extend(item_fresh);
            }
        }
        Style::Match { scrutinee, arms } => {
            for (k, body) in arms {
                // Exclusive arms: separate path per arm (no cross-arm clash).
                let p = format!("{path}/match:{scrutinee}={k}");
                let mut local = HashMap::new();
                walk(body, env, &p, &mut local, &mut HashSet::new(), diags, resolving);
            }
        }
        Style::Compound { when, body } => {
            let fp = when
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            let p = format!("{path}/compound:{fp}");
            walk(body, env, &p, writes, fresh, diags, resolving);
        }
        Style::When { entries } => {
            for (k, body) in entries {
                let p = format!("{path}/when:{k}");
                // Separate path per when key — no clash across hover vs focus.
                let prefix = format!("{path}/when:");
                let mut local = writes.clone();
                let mut local_fresh = fresh.clone();
                walk(body, env, &p, &mut local, &mut local_fresh, diags, resolving);
                for (k2, v) in local {
                    if k2.1.starts_with(&prefix) {
                        writes.insert(k2, v);
                    }
                }
                for k2 in local_fresh {
                    if k2.1.starts_with(&prefix) {
                        fresh.insert(k2);
                    }
                }
            }
        }
        Style::Ref(name) => {
            if !resolving.insert(name.clone()) {
                return;
            }
            if let Some(s) = env.get(name) {
                walk(s, env, path, writes, fresh, diags, resolving);
            }
            resolving.remove(name);
        }
    }
}
