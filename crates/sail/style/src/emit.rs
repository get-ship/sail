//! Style IR → atomic CSS (token vars, attr selectors, when pseudos).

use std::collections::{HashMap, HashSet};

use super::ir::{PaintArg, PaintOp, Style};
use super::setters::{WhenTarget, lower_op_css, when_target};

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

pub fn emit_style(style: &Style, env: &StyleEnv) -> Vec<CssRule> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    emit_inner(
        style,
        env,
        &Ctx::default(),
        &mut out,
        &mut seen,
        &mut HashSet::new(),
    );
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
    if r.selector.is_empty() || r.declarations.is_empty() {
        return String::new();
    }
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
    ancestor: String,
    media: Option<String>,
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
    fn with_ancestor(&self, key: &str, anc: &str) -> Self {
        let mut n = self.clone();
        n.ancestor = format!("{}{anc}", n.ancestor);
        n.path_tags.push(key.into());
        n
    }
    fn with_media(&self, key: &str, media: &str) -> Self {
        let mut n = self.clone();
        n.media = Some(media.into());
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
                if let Some(target) = when_target(key) {
                    let next = match target {
                        WhenTarget::Pseudo(sel) => ctx.with_pseudo(key, sel.as_ref()),
                        WhenTarget::Ancestor(anc) => ctx.with_ancestor(key, anc.as_ref()),
                        WhenTarget::Media(m) => ctx.with_media(key, m.as_ref()),
                    };
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
        // Lifecycle motion is attribute-lowered; CSS paint emit ignores these nodes.
        Style::MotionAttach { .. } => {}
    }
}

fn emit_op(op: &PaintOp, ctx: &Ctx, out: &mut Vec<CssRule>, seen: &mut HashSet<String>) {
    // Bound token-group paths (`bg(tone)`, `fg(tone.foreground)`) lower to
    // bridge vars (`var(--tone)`, `var(--tone-foreground)`). Host reflection
    // writes `--tone: var(--primary)` etc. — no `[tone='x']` selector tables.
    // group marker: sail-grp class, no CSS rule body required.
    if op.setter == "group" {
        let class = "sail-grp".to_string();
        let key = format!("marker|{class}");
        if seen.insert(key) {
            out.push(CssRule {
                class,
                selector: String::new(),
                declarations: Vec::new(),
                media: None,
            });
        }
        return;
    }
    // L1 host surface MemberPath → emit public name (identity stays in IR until here).
    let op_owned;
    let op_ref = if op_has_surface_member(op) {
        op_owned = resolve_surface_args(op);
        &op_owned
    } else {
        op
    };
    let Ok(decls) = lower_op_css(op_ref) else {
        return;
    };
    if decls.is_empty() {
        return;
    }
    let class = class_name(op_ref, ctx);
    let mut child = "";
    if op.setter == "avatarMedia" {
        child = " img";
    } else if op.setter == "glyphFill" {
        child = " svg";
    }
    let selector = build_selector(&class, ctx, child);
    let key = format!("{selector}|{decls:?}|{:?}", ctx.media);
    if seen.insert(key) {
        out.push(CssRule {
            class,
            selector,
            declarations: decls,
            media: ctx.media.clone(),
        });
    }
}

fn class_name(op: &PaintOp, ctx: &Ctx) -> String {
    if op.setter == "avatarMedia" {
        return "s-avatarmedia".into();
    }
    if op.setter == "glyphFill" {
        return "s-glyphfill".into();
    }
    // All args feed the class name — composites like border(1, border) vs
    // border(1, primary) must not share an atomic class.
    let mut arg = String::new();
    for a in &op.args {
        match a {
            PaintArg::Token(t) => {
                arg.extend(t.chars().filter(|c| c.is_ascii_alphanumeric()))
            }
            PaintArg::Bind(n) => {
                arg.push_str("bind");
                arg.extend(n.chars().filter(|c| c.is_ascii_alphanumeric()));
            }
            PaintArg::MemberPath { root, path } => {
                arg.extend(root.chars().filter(|c| c.is_ascii_alphanumeric()));
                for seg in path {
                    arg.extend(seg.chars().filter(|c| c.is_ascii_alphanumeric()));
                }
            }
        }
    }
    let setter: String = op
        .setter
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let path: String = ctx
        .path_tags
        .iter()
        .flat_map(|t| t.chars())
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if path.is_empty() {
        format!("s-{setter}{arg}")
    } else {
        format!("s-{path}-{setter}{arg}")
    }
}

fn build_selector(class: &str, ctx: &Ctx, child: &str) -> String {
    let mut attrs = String::new();
    for (k, v) in &ctx.attrs {
        // `true` (bool prop) and `*` (any-set wildcard) → presence gate [k].
        if v == "true" || v == "*" {
            attrs.push_str(&format!("[{k}]"));
        } else if v == "false" {
            attrs.push_str(&format!(":not([{k}])"));
        } else {
            attrs.push_str(&format!("[{k}='{v}']"));
        }
    }
    format!(
        "{ancestor}{attrs}.{class}{pseudo}{child}",
        ancestor = ctx.ancestor,
        pseudo = ctx.pseudo,
        child = child
    )
}



fn op_has_surface_member(op: &PaintOp) -> bool {
    op.args.iter().any(|a| {
        matches!(a, PaintArg::MemberPath { root, .. }
            if super::component_surfaces::current_surface_pack() == Some(root.as_str()))
    })
}

fn resolve_surface_args(op: &PaintOp) -> PaintOp {
    use super::component_surfaces::resolve_surface_author;
    let args = op
        .args
        .iter()
        .map(|a| match a {
            PaintArg::MemberPath { root, path }
                if super::component_surfaces::current_surface_pack() == Some(root.as_str()) =>
            {
                match resolve_surface_author(root, path) {
                    Ok(public) => PaintArg::Token(public.to_string()),
                    Err(_) => a.clone(),
                }
            }
            other => other.clone(),
        })
        .collect();
    PaintOp {
        setter: op.setter.clone(),
        args,
        source: op.source.clone(),
    }
}

/// Product entry: parse Style expression → classes + CSS.
/// Residual utility strings return empty classes and push diagnostics via return flag.
pub fn lower_style_source(
    src: &str,
    env: &StyleEnv,
    css: &mut String,
    seen: &mut HashSet<String>,
) -> Result<String, Vec<super::ir::StyleDiagnostic>> {
    lower_style_source_with_motion(src, env, css, seen).map(|(classes, _)| classes)
}

/// Like [`lower_style_source`], but also returns motion attaches extracted from
/// `.enter` / `.exit` / `.loop` / `.reveal` / `.scroll({…})` chain methods.
///
/// Preprocess injects these as compiler-private `_sail_m_<trigger>="{{…}}"`
/// attributes so the existing `AttrInstr::Motion` emit path handles them.
pub fn lower_style_source_with_motion(
    src: &str,
    env: &StyleEnv,
    css: &mut String,
    seen: &mut HashSet<String>,
) -> Result<(String, Vec<(String, String)>), Vec<super::ir::StyleDiagnostic>> {
    // Product authoring: channel spine only. Flat `style.pad().bg()` is retired
    // (corpus codemod + block attachment). Migration dual is `flat_chain` only.
    // Early refuse matches product `parse` (no IR for flat chains).
    let t = src.trim_start();
    if t.starts_with("style.") {
        return Err(vec![super::ir::StyleDiagnostic::ParseError {
            message: "flat `style.method(…)` chains are retired — use the channel spine: \
                      style({ box: pad(4).gap(2), paint: bg(primary), when: { hover: { paint: bg(muted) } } })"
                .into(),
            source: t.to_string(),
        }]);
    }
    let (style, diags) = super::parse::parse(src);
    if diags
        .iter()
        .any(|d| matches!(d, super::ir::StyleDiagnostic::ResidualProduct { .. }))
    {
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
        match d {
            super::ir::StyleDiagnostic::BadToken { .. }
            | super::ir::StyleDiagnostic::UnknownSetter { .. }
            | super::ir::StyleDiagnostic::UnknownWhenKey { .. }
            | super::ir::StyleDiagnostic::BadArity { .. } => true,
            // Hard-fail lifecycle-in-when, geometry-in-interaction-when, color alias.
            super::ir::StyleDiagnostic::ParseError { message, .. } => {
                message.contains("cannot appear inside when")
                    || message.contains("cannot appear inside interaction when")
                    || message.contains("state can't gate layout")
                    || message.contains("was renamed to `fg")
                    || message.contains("text channel is only legal")
            }
            _ => false,
        }
    }) {
        return Err(diags);
    }
    // Resolve Style::Ref through env so named style decls
    // (`style ambientDot = { …, animate: loop({…}) }` + `style(ambientDot)`)
    // still inject `_sail_m_*` attrs. CSS emit already expands refs; motion
    // collection must not skip them or ambient/enter/exit on named styles die.
    let motions = super::parse::collect_motion_attaches_env(&style, env);
    let rules = emit_style(&style, env);
    let mut classes = Vec::new();
    for r in &rules {
        // Marker-only rules (group) contribute a class with empty selector.
        if r.selector.is_empty() && r.declarations.is_empty() {
            if !classes.iter().any(|c| c == &r.class) {
                classes.push(r.class.clone());
            }
            continue;
        }
        if seen.insert(r.class.clone()) {
            css.push_str(&render_rule(r));
        }
        if !classes.iter().any(|c| c == &r.class) {
            classes.push(r.class.clone());
        }
    }
    Ok((classes.join(" "), motions))
}
