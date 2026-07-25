//! Emit CSS from the Sail Style IR.
//!
//! Style lowers to atomic classes + CSS rules. Token values emit as
//! `var(--token-name)`. No JS runtime.

use std::collections::HashMap;

use crate::rules::sx as sx_rules;
use crate::sx_ir::SxValue;

use super::ir::{PaintOp, Style};

/// One emitted CSS rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssRule {
    pub class: String,
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub media: Option<String>,
}

/// Lower a Style value to CSS rules. `consts` resolves `Style::Ref` to scope
/// const definitions.
pub fn emit_style(style: &Style, consts: &HashMap<String, Style>) -> Vec<CssRule> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    emit_style_inner(style, consts, &SelectorCtx::default(), &mut out, &mut seen);
    out
}

#[derive(Debug, Clone, Default)]
struct SelectorCtx {
    attrs: Vec<(String, String)>,
    pseudo: String,
    media: Option<String>,
}

impl SelectorCtx {
    fn with_attr(&self, key: &str, value: &str) -> Self {
        let mut next = self.clone();
        next.attrs.push((key.to_string(), value.to_string()));
        next
    }

    fn with_pseudo(&self, pseudo: &str) -> Self {
        let mut next = self.clone();
        next.pseudo.push_str(pseudo);
        next
    }

    fn with_media(&self, media: &str) -> Self {
        let mut next = self.clone();
        next.media = Some(media.to_string());
        next
    }

    fn build_selector(&self, class: &str) -> String {
        let attr_part = self
            .attrs
            .iter()
            .map(|(k, v)| format!("[{k}='{v}']"))
            .collect::<Vec<_>>()
            .join("");
        let base = format!("{attr_part} .{class}");
        if self.pseudo.is_empty() {
            base
        } else {
            format!("{base}{}", self.pseudo)
        }
    }
}

fn emit_style_inner(
    style: &Style,
    consts: &HashMap<String, Style>,
    ctx: &SelectorCtx,
    out: &mut Vec<CssRule>,
    seen: &mut std::collections::HashSet<String>,
) {
    match style {
        Style::Empty => {}
        Style::Set(op) => emit_paint_op(op, ctx, out, seen),
        Style::Chain(items) => {
            for item in items {
                emit_style_inner(item, consts, ctx, out, seen);
            }
        }
        Style::Match { scrutinee, arms } => {
            for (value, body) in arms {
                let next = ctx.with_attr(scrutinee, value);
                emit_style_inner(body, consts, &next, out, seen);
            }
        }
        Style::Compound { conditions, body } => {
            let mut next = ctx.clone();
            for (k, v) in conditions {
                next.attrs.push((k.clone(), v.clone()));
            }
            emit_style_inner(body, consts, &next, out, seen);
        }
        Style::When { map } => {
            for (key, body) in map {
                let next = ctx_for_when_key(ctx, key);
                emit_style_inner(body, consts, &next, out, seen);
            }
        }
        Style::Bind { name } => {
            // Reactive paint is emitted as a custom property write.
            // The class is a marker; the actual var assignment is done by the
            // compiler's reflect path.
            let class = format!("sx-bind-{}", sanitize_class(name));
            if seen.insert(class.clone()) {
                out.push(CssRule {
                    class,
                    selector: format!(".sx-bind-{}", sanitize_class(name)),
                    declarations: vec![(format!("--{name}"), "initial".to_string())],
                    media: ctx.media.clone(),
                });
            }
        }
        Style::Ref { name } => {
            if let Some(resolved) = consts.get(name) {
                emit_style_inner(resolved, consts, ctx, out, seen);
            }
        }
    }
}

fn emit_paint_op(
    op: &PaintOp,
    ctx: &SelectorCtx,
    out: &mut Vec<CssRule>,
    seen: &mut std::collections::HashSet<String>,
) {
    let Some(decls) = lower_paint_op(op) else {
        return;
    };
    let class = class_name(op);
    let selector = ctx.build_selector(&class);
    let key = format!(
        "{}|{}|{:?}",
        ctx.media.as_deref().unwrap_or(""),
        selector,
        decls
    );
    if seen.insert(key) {
        out.push(CssRule {
            class,
            selector,
            declarations: decls,
            media: ctx.media.clone(),
        });
    }
}

fn ctx_for_when_key(ctx: &SelectorCtx, key: &str) -> SelectorCtx {
    if let Some(media) = sx_rules::condition_media(key) {
        return ctx.with_media(media);
    }
    if let Some(pseudo) = sx_rules::state_pseudo(key) {
        return ctx.with_pseudo(pseudo);
    }
    if let Some(pe) = sx_rules::pseudo_element_prefix(key) {
        return ctx.with_pseudo(pe);
    }
    if let Some(data) = sx_rules::state_data(key) {
        return ctx.with_pseudo(&data);
    }
    if let Some(attr) = sx_rules::group_attr_presence(key) {
        return ctx.with_pseudo(&format!(".sail-grp[{attr}] &"));
    }
    if key == "group" {
        return ctx.with_pseudo(".sail-grp &");
    }
    // Unknown key: pass through as a pseudo (will likely not match).
    ctx.with_pseudo(&format!(":{key}"))
}

fn lower_paint_op(op: &PaintOp) -> Option<Vec<(String, String)>> {
    // Reuse the existing sx_ir lowering by building a synthetic decl.
    // The setter name maps to the vocabulary utility name.
    let atom = setter_atom(&op.name, &op.arg);
    match sx_vocab_resolve(&atom) {
        Some(decls) => Some(decls),
        None => None,
    }
}

fn setter_atom(name: &str, arg: &SxValue) -> String {
    let arg_str = match arg {
        SxValue::Token(v) | SxValue::Keyword(v) => v.clone(),
        SxValue::Raw(v) => format!("raw:{v}"),
        SxValue::VarRef(v) => format!("(--{v})"),
    };
    // Map camelCase setter names to kebab-case vocabulary names.
    let kebab = camel_to_kebab(name);
    format!("{kebab}-{arg_str}")
}

fn camel_to_kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

fn sx_vocab_resolve(atom: &str) -> Option<Vec<(String, String)>> {
    match crate::sx_vocab::resolve(atom) {
        crate::sx_vocab::Resolved::Ok(decls) => Some(decls),
        _ => None,
    }
}

fn class_name(op: &PaintOp) -> String {
    let kebab = camel_to_kebab(&op.name);
    let sanitized: String = kebab
        .chars()
        .chain(std::iter::once('-'))
        .chain(
            match &op.arg {
                SxValue::Token(v) | SxValue::Keyword(v) => v.chars().collect::<Vec<_>>().into_iter(),
                SxValue::Raw(v) => format!("raw{v}").chars().collect::<Vec<_>>().into_iter(),
                SxValue::VarRef(v) => format!("bind{v}").chars().collect::<Vec<_>>().into_iter(),
            },
        )
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    format!("sx-{sanitized}")
}

fn sanitize_class(name: &str) -> String {
    name.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// Render a list of CSS rules to a stylesheet string.
pub fn render_rules(rules: &[CssRule]) -> String {
    let mut out = String::new();
    let mut by_media: HashMap<Option<String>, Vec<&CssRule>> = HashMap::new();
    for rule in rules {
        by_media.entry(rule.media.clone()).or_default().push(rule);
    }
    let mut media_keys: Vec<_> = by_media.keys().cloned().collect();
    media_keys.sort_by_key(|k| k.is_none());
    for media in media_keys {
        let rs = by_media.get(&media).unwrap();
        let body = rs
            .iter()
            .map(|r| {
                let decls = r
                    .declarations
                    .iter()
                    .map(|(k, v)| format!("{k}:{v}"))
                    .collect::<Vec<_>>()
                    .join(";");
                format!("{} {{{}}} ", r.selector, decls)
            })
            .collect::<String>();
        match media {
            Some(m) => out.push_str(&format!("@media {} {{ {} }}", m, body)),
            None => out.push_str(&body),
        }
    }
    out
}

/// Convenience: lower a Style to `(class_list, css_text)`.
pub fn lower_style(style: &Style, consts: &HashMap<String, Style>) -> (String, String) {
    let rules = emit_style(style, consts);
    let classes = rules
        .iter()
        .map(|r| r.class.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(" ");
    (classes, render_rules(&rules))
}

/// Lower a `sx={…}` or `sx={[…]}` attribute value into classes and CSS.
pub fn lower_sx_attr(
    val: &str,
    consts: &HashMap<String, Style>,
    css: &mut String,
    seen: &mut std::collections::HashSet<String>,
) -> String {
    let (style, _diags) = super::parse::parse(val);
    let rules = emit_style(&style, consts);
    let mut classes = Vec::new();
    for rule in &rules {
        if seen.insert(rule.class.clone()) {
            css.push_str(&render_rules(std::slice::from_ref(rule)));
        }
        classes.push(rule.class.clone());
    }
    classes.join(" ")
}

/// Lower a `sx={[…]}` list into classes and CSS.
pub fn lower_sx_list_attr(
    val: &str,
    consts: &HashMap<String, Style>,
    css: &mut String,
    seen: &mut std::collections::HashSet<String>,
) -> String {
    let (items, _diags) = super::parse::parse_style_list(val);
    let style = Style::chain(items);
    let rules = emit_style(&style, consts);
    let mut classes = Vec::new();
    for rule in &rules {
        if seen.insert(rule.class.clone()) {
            css.push_str(&render_rules(std::slice::from_ref(rule)));
        }
        classes.push(rule.class.clone());
    }
    classes.join(" ")
}
