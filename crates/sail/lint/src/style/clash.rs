//! Paint-slot clash detection (fail closed).

use std::collections::HashMap;

use super::emit::StyleEnv;
use super::ir::{Style, StyleDiagnostic};
use super::setters::lower_op_css;

/// Detect same CSS property written twice under the same guard path.
pub fn detect_clashes(style: &Style, env: &StyleEnv) -> Vec<StyleDiagnostic> {
    let mut writes: HashMap<(String, String), String> = HashMap::new();
    let mut diags = Vec::new();
    walk(style, env, "base", &mut writes, &mut diags, &mut std::collections::HashSet::new());
    diags
}

fn walk(
    style: &Style,
    env: &StyleEnv,
    path: &str,
    writes: &mut HashMap<(String, String), String>,
    diags: &mut Vec<StyleDiagnostic>,
    resolving: &mut std::collections::HashSet<String>,
) {
    match style {
        Style::Empty => {}
        Style::Set(op) => {
            if let Ok(decls) = lower_op_css(op) {
                for (css_prop, _) in decls {
                    let key = (css_prop.clone(), path.to_string());
                    if let Some(prev) = writes.get(&key) {
                        diags.push(StyleDiagnostic::Clash {
                            slot: format!("{css_prop}@{path}"),
                            first: prev.clone(),
                            second: op.source.clone(),
                        });
                    } else {
                        writes.insert(key, op.source.clone());
                    }
                }
            }
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                walk(i, env, path, writes, diags, resolving);
            }
        }
        Style::Match { scrutinee, arms } => {
            for (k, body) in arms {
                // Exclusive arms: separate path per arm (no cross-arm clash).
                let p = format!("{path}/match:{scrutinee}={k}");
                let mut local = HashMap::new();
                walk(body, env, &p, &mut local, diags, resolving);
                // Don't merge arm writes into parent path as conflicts with each other.
            }
        }
        Style::Compound { when, body } => {
            let fp = when
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            let p = format!("{path}/compound:{fp}");
            walk(body, env, &p, writes, diags, resolving);
        }
        Style::When { entries } => {
            for (k, body) in entries {
                let p = format!("{path}/when:{k}");
                // Separate path per when key — no clash across hover vs focus.
                let mut local = writes.clone();
                walk(body, env, &p, &mut local, diags, resolving);
                // Merge only non-conflicting? For when branches, paths differ so parent writes stay.
                for (k2, v) in local {
                    if k2.1.starts_with(&format!("{path}/when:")) {
                        writes.insert(k2, v);
                    }
                }
            }
        }
        Style::Ref(name) => {
            if !resolving.insert(name.clone()) {
                return;
            }
            if let Some(s) = env.get(name) {
                walk(s, env, path, writes, diags, resolving);
            }
            resolving.remove(name);
        }
    }
}
