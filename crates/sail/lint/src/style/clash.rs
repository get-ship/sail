//! Clash detection for Sail Style.
//!
//! A clash is two writes to the same CSS property under the same guard path.
//! Fail closed: emit a diagnostic. No silent merge.

use std::collections::HashMap;

use super::ir::{PaintOp, Style, StyleDiagnostic};

/// A write site used for clash reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSite {
    pub setter: String,
    pub css_property: String,
    pub guard_path: String,
    pub source: String,
}

/// Detect clashes within a Style.
pub fn detect_clashes(style: &Style) -> Vec<StyleDiagnostic> {
    let writes = collect_writes(style, "");
    let mut diags = Vec::new();
    let mut groups: HashMap<(String, String), Vec<WriteSite>> = HashMap::new();
    for w in writes {
        groups
            .entry((w.guard_path.clone(), w.css_property.clone()))
            .or_default()
            .push(w);
    }
    for ((path, css_prop), sites) in groups {
        if sites.len() < 2 {
            continue;
        }
        // Match arms are mutually exclusive by scrutinee value.
        let base_count = sites.iter().filter(|s| !s.guard_path.contains('[')).count();
        let mut reported: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for (i, a) in sites.iter().enumerate() {
            for (j, b) in sites.iter().enumerate() {
                if i >= j {
                    continue;
                }
                if same_match_arm(&a.guard_path, &b.guard_path)
                    && a.guard_path.contains('[')
                    && base_count == 0
                {
                    continue;
                }
                if reported.insert(i) || reported.insert(j) {
                    diags.push(StyleDiagnostic::Clash {
                        prop: css_prop.clone(),
                        first: format!("{path} -> {}", a.source),
                        second: format!("{path} -> {}", b.source),
                    });
                }
            }
        }
    }
    diags
}

fn collect_writes(style: &Style, path: &str) -> Vec<WriteSite> {
    let mut out = Vec::new();
    collect(style, path, &mut out);
    out
}

fn collect(style: &Style, path: &str, out: &mut Vec<WriteSite>) {
    match style {
        Style::Empty => {}
        Style::Set(op) => {
            for css_prop in paint_op_css_properties(op) {
                out.push(WriteSite {
                    setter: op.name.clone(),
                    css_property: css_prop,
                    guard_path: path.to_string(),
                    source: op.source.clone(),
                });
            }
        }
        Style::Chain(items) => {
            for item in items {
                collect(item, path, out);
            }
        }
        Style::Match { scrutinee, arms } => {
            for (value, body) in arms {
                let next = format!("{path}[{scrutinee}={value}]");
                collect(body, &next, out);
            }
        }
        Style::Compound { conditions, body } => {
            let conds = conditions
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(",");
            let next = format!("{path}({conds})");
            collect(body, &next, out);
        }
        Style::When { map } => {
            for (key, body) in map {
                let next = format!("{path}:{key}");
                collect(body, &next, out);
            }
        }
        Style::Bind { .. } | Style::Ref { .. } => {}
    }
}

fn paint_op_css_properties(op: &PaintOp) -> Vec<String> {
    let arg_str = match &op.arg {
        crate::sx_ir::SxValue::Token(v) | crate::sx_ir::SxValue::Keyword(v) => v.clone(),
        crate::sx_ir::SxValue::Raw(v) => format!("raw:{v}"),
        crate::sx_ir::SxValue::VarRef(v) => format!("(--{v})"),
    };
    let atom = format!("{}-{}", op.name, arg_str);
    match crate::sx_vocab::resolve(&atom) {
        crate::sx_vocab::Resolved::Ok(decls) => decls.into_iter().map(|(k, _)| k).collect(),
        _ => Vec::new(),
    }
}

fn same_match_arm(a: &str, b: &str) -> bool {
    let last_a = a.rsplit_once('[').map(|(_, rest)| rest);
    let last_b = b.rsplit_once('[').map(|(_, rest)| rest);
    last_a.is_some() && last_a == last_b
}
