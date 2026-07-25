//! Style IR → atomic CSS (token vars, attr selectors, when pseudos).

use std::collections::{HashMap, HashSet};

use super::ir::{PaintOp, Style};
use super::setters::{lower_op_css, when_selector};

/// One emitted CSS rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssRule {
    pub class: String,
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub media: Option<String>,
}

/// Fragment/const environment for resolving Style::Ref.
pub type StyleEnv = HashMap<String, Style>;

/// Emit Style to CSS rules. Unknown refs skipped (caller should resolve first).
pub fn emit_style(style: &Style, env: &StyleEnv) -> Vec<CssRule> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    emit_inner(style, env, &Ctx::default(), &mut out, &mut seen, &mut HashSet::new());
    out
}

/// Convenience: classes + stylesheet text.
pub fn lower_style(style: &Style, env: &StyleEnv) -> (String, String) {
    let rules = emit_style(style, env);
    let mut classes = Vec::new();
    let mut css = String::new();
    let mut seen_c = HashSet::new();
    for r in &rules {
        if seen_c.insert(r.class.clone()) {
            classes.push(r.class.clone());
        }
        css.push_str(&render_rule(r));
    }
    (classes.join(" "), css)
}

pub fn render_rule(r: &CssRule) -> String {
    let body = r
        .declarations
        .iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<_>>()
        .join(";");
    let rule = format!("{}{{{}}}", r.selector, body);
    match &r.media {
        Some(m) => format!("@media {m}{{{rule}}}"),
        None => rule,
    }
}

#[derive(Debug, Clone, Default)]
struct Ctx {
    attrs: Vec<(String, String)>,
    pseudo: String,
    path_tags: Vec<String>,
}

impl Ctx {
    fn with_attr(&self, k: &str, v: &str) -> Self {
        let mut n = self.clone();
        n.attrs.push((k.into(), v.into()));
        n.path_tags.push(format!("{k}{v}"));
        n
    }
    fn with_pseudo(&self, key: &str, sel: &str) -> Self {
        let mut n = self.clone();
        n.pseudo.push_str(sel);
        n.path_tags.push(key.into());
        n
    }
}

fn emit_inner(
    style: &Style,
    env: &StyleEnv,
    ctx: &Ctx,
    out: &mut Vec<CssRule>,
    seen: &mut HashSet<String>,
    resolving: &mut HashSet<String>,
) {
    match style {
        Style::Empty => {}
        Style::Set(op) => emit_op(op, ctx, out, seen),
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                emit_inner(i, env, ctx, out, seen, resolving);
            }
        }
        Style::Match { scrutinee, arms } => {
            for (key, body) in arms {
                let next = ctx.with_attr(scrutinee, key);
                emit_inner(body, env, &next, out, seen, resolving);
            }
        }
        Style::Compound { when, body } => {
            let mut next = ctx.clone();
            for (k, v) in when {
                next = next.with_attr(k, v);
            }
            emit_inner(body, env, &next, out, seen, resolving);
        }
        Style::When { entries } => {
            for (key, body) in entries {
                if let Some(sel) = when_selector(key) {
                    let next = ctx.with_pseudo(key, sel);
                    emit_inner(body, env, &next, out, seen, resolving);
                }
            }
        }
        Style::Ref(name) => {
            if !resolving.insert(name.clone()) {
                return;
            }
            if let Some(s) = env.get(name) {
                emit_inner(s, env, ctx, out, seen, resolving);
            }
            resolving.remove(name);
        }
    }
}

fn emit_op(op: &PaintOp, ctx: &Ctx, out: &mut Vec<CssRule>, seen: &mut HashSet<String>) {
    let Ok(decls) = lower_op_css(op) else {
        return;
    };
    let class = class_name(op, ctx);
    let selector = build_selector(&class, ctx);
    let key = format!("{selector}|{decls:?}");
    if seen.insert(key) {
        out.push(CssRule {
            class,
            selector,
            declarations: decls,
            media: None,
        });
    }
}

fn class_name(op: &PaintOp, ctx: &Ctx) -> String {
    let arg = match &op.arg {
        super::ir::PaintArg::Token(t) => t.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>(),
        super::ir::PaintArg::Bind(n) => {
            let mut s = String::from("bind");
            s.extend(n.chars().filter(|c| c.is_ascii_alphanumeric()));
            s
        }
    };
    let setter: String = op.setter.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let path: String = ctx
        .path_tags
        .iter()
        .flat_map(|t| t.chars())
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if path.is_empty() {
        format!("sx-{setter}{arg}")
    } else {
        format!("sx-{path}-{setter}{arg}")
    }
}

fn build_selector(class: &str, ctx: &Ctx) -> String {
    let mut attrs = String::new();
    for (k, v) in &ctx.attrs {
        if v == "true" {
            attrs.push_str(&format!("[{k}]"));
        } else if v == "false" {
            attrs.push_str(&format!(":not([{k}])"));
        } else {
            attrs.push_str(&format!("[{k}='{v}']"));
        }
    }
    format!("{attrs}.{class}{}", ctx.pseudo)
}

/// Product entry: parse Style expression → classes + CSS.
/// Residual utility strings return empty classes and push diagnostics via return flag.
pub fn lower_style_source(
    src: &str,
    env: &StyleEnv,
    css: &mut String,
    seen: &mut HashSet<String>,
) -> Result<String, Vec<super::ir::StyleDiagnostic>> {
    let (style, diags) = super::parse::parse(src);
    if diags.iter().any(|d| matches!(d, super::ir::StyleDiagnostic::ResidualProduct { .. })) {
        return Err(diags);
    }
    // Clash
    let clashes = super::clash::detect_clashes(&style, env);
    if !clashes.is_empty() {
        let mut all = diags;
        all.extend(clashes);
        return Err(all);
    }
    if diags.iter().any(|d| {
        matches!(
            d,
            super::ir::StyleDiagnostic::BadToken { .. }
                | super::ir::StyleDiagnostic::UnknownSetter { .. }
                | super::ir::StyleDiagnostic::UnknownWhenKey { .. }
        )
    }) {
        return Err(diags);
    }
    let rules = emit_style(&style, env);
    let mut classes = Vec::new();
    for r in &rules {
        if seen.insert(r.class.clone()) {
            css.push_str(&render_rule(r));
        }
        if !classes.iter().any(|c| c == &r.class) {
            classes.push(r.class.clone());
        }
    }
    Ok(classes.join(" "))
}
