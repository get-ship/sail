//! Style IR → atomic CSS (token vars, attr selectors, when pseudos).

use std::collections::HashSet;

use crate::lower::lower_op_css;
use spinnaker::intent::{lower_to_intent, Intent, IntentNode, IntentValue};
use spinnaker::ir::{PaintArg, PaintOp, Style, StyleEnv};
use spinnaker::theme_env::{current_styling_policy, ThemeStyleEnv};

use crate::when::{when_target, WhenTarget};

/// One emitted CSS rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssRule {
    pub class: String,
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub media: Option<String>,
}

/// Product emit. The adapter never walks `Style` — only Intent.
pub fn emit_style(style: &Style, env: &StyleEnv) -> Vec<CssRule> {
    emit_intent(&lower_to_intent(style), env)
}

/// Realize a resolved Intent tree as CSS. This is the adapter entry.
pub fn emit_intent(intent: &Intent, env: &StyleEnv) -> Vec<CssRule> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    emit_intent_inner(
        intent,
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

fn arg_from_intent(v: &IntentValue) -> PaintArg {
    match v {
        IntentValue::Token(t) => PaintArg::Token(t.clone()),
        IntentValue::Bind(n) => PaintArg::Bind(n.clone()),
        IntentValue::MemberPath { root, path } => PaintArg::MemberPath {
            root: root.clone(),
            path: path.clone(),
        },
        IntentValue::Slots(slots) => PaintArg::Slots(
            slots
                .iter()
                .map(|(k, v)| (k.clone(), arg_from_intent(v)))
                .collect(),
        ),
    }
}

fn emit_intent_inner(
    intent: &Intent,
    env: &StyleEnv,
    ctx: &Ctx,
    out: &mut Vec<CssRule>,
    seen: &mut HashSet<String>,
    resolving: &mut HashSet<String>,
) {
    for node in &intent.nodes {
        match node {
            IntentNode::Set { setter, args } => {
                emit_set(setter, args, ctx, out, seen);
            }
            IntentNode::On { key, body } => {
                if let Some(target) = when_target(key) {
                    let next = match target {
                        WhenTarget::Pseudo(sel) => ctx.with_pseudo(key, sel.as_ref()),
                        WhenTarget::Ancestor(anc) => ctx.with_ancestor(key, anc.as_ref()),
                        WhenTarget::Media(m) => ctx.with_media(key, m.as_ref()),
                    };
                    emit_intent_inner(body, env, &next, out, seen, resolving);
                }
            }
            IntentNode::Match { scrutinee, arms } => {
                for (key, body) in arms {
                    let next = ctx.with_attr(scrutinee, key);
                    emit_intent_inner(body, env, &next, out, seen, resolving);
                }
            }
            IntentNode::Compound { when, body } => {
                let mut next = ctx.clone();
                for (k, v) in when {
                    next = next.with_attr(k, v);
                }
                emit_intent_inner(body, env, &next, out, seen, resolving);
            }
            IntentNode::Animate { trigger, config } => {
                emit_play(trigger, config, ctx, out, seen);
            }
            IntentNode::Ref(name) => {
                if !resolving.insert(name.clone()) {
                    continue;
                }
                if let Ok(s) = ThemeStyleEnv::from_registry().expand_style_ref(
                    name,
                    env,
                    current_styling_policy(),
                ) {
                    emit_intent_inner(&lower_to_intent(&s), env, ctx, out, seen, resolving);
                }
                resolving.remove(name);
            }
        }
    }
}

/// Play emit shape:
///
/// - keyframe preset (`play(fade)` on enter/exit/…): MotionAttach only
///   (`{ preset: fade }`) — runtime motion, no CSS atoms
/// - transition prop-set (`play(colors, fast)`): CSS
///   `transition-property` + `transition-duration: var(--duration-*)`
fn emit_play(
    trigger: &str,
    config: &str,
    ctx: &Ctx,
    out: &mut Vec<CssRule>,
    seen: &mut HashSet<String>,
) {
    if trigger != "transition" {
        return;
    }
    let Some(preset) = extract_play_field(config, "preset") else {
        return;
    };
    let Some(play) = ThemeStyleEnv::from_registry().resolve_play(&preset) else {
        return;
    };
    let is_transition = play.r#type.contains("play")
        || play.r#type.contains("transition")
        || play.value.contains_key("property")
        || play.value.contains_key("props");
    if !is_transition {
        return;
    }
    let property = play
        .value
        .get("property")
        .and_then(|v| v.as_str())
        .or_else(|| {
            play.value.get("props").and_then(|v| match v {
                serde_json::Value::String(s) => Some(s.as_str()),
                _ => None,
            })
        })
        .map(|s| s.to_string())
        .or_else(|| {
            play.value
                .get("props")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|i| i.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
        })
        .unwrap_or_else(|| "all".into());
    let duration_tok = extract_play_field(config, "duration").unwrap_or_else(|| "medium".into());
    let duration = if duration_tok.starts_with("var(") {
        duration_tok.clone()
    } else {
        format!("var(--duration-{duration_tok})")
    };
    let class = format!(
        "s-play{}{}",
        sanitize_ident(&preset),
        sanitize_ident(&duration_tok)
    );
    let selector = build_selector(&class, ctx);
    let decls = vec![
        ("transition-property".into(), property),
        ("transition-duration".into(), duration),
    ];
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

fn extract_play_field(config: &str, field: &str) -> Option<String> {
    let needle = format!("{field}:");
    let rest = config.split(&needle).nth(1)?;
    let token = rest
        .trim_start()
        .split(|c: char| c == ',' || c == '}' || c.is_whitespace())
        .find(|s| !s.is_empty())?;
    Some(token.to_string())
}

fn sanitize_ident(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// PaintOp is a private helper inside the adapter — Intent is the public input.
fn emit_set(
    setter: &str,
    args: &[IntentValue],
    ctx: &Ctx,
    out: &mut Vec<CssRule>,
    seen: &mut HashSet<String>,
) {
    let op = PaintOp {
        setter: setter.to_string(),
        args: args.iter().map(arg_from_intent).collect(),
        source: String::new(),
    };
    emit_op(&op, ctx, out, seen);
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
    // Tone-bound paint (`bg(tone)`, `fg(tone.foreground)`, mix on --tone)
    // must be `[tone]`-gated. Rest fills in the same match arm share
    // `[variant=soft].s-variantsoft-bgmuted`; later elements re-emit that
    // selector and crush the tone wash (badge/chip/avatar all go muted).
    let tone_ctx;
    let sel_ctx = if op_uses_tone(op_ref) && !ctx.attrs.iter().any(|(k, _)| k == "tone") {
        tone_ctx = ctx.with_attr("tone", "*");
        &tone_ctx
    } else {
        ctx
    };
    let selector = build_selector(&class, sel_ctx);
    let key = format!("{selector}|{decls:?}|{:?}", ctx.media);
    if seen.insert(key) {
        out.push(CssRule {
            class: class.clone(),
            selector,
            declarations: decls,
            media: ctx.media.clone(),
        });
    }
}

fn class_name(op: &PaintOp, ctx: &Ctx) -> String {
    // All args feed the class name — composites like border(1, border) vs
    // border(1, primary) must not share an atomic class.
    let mut arg = String::new();
    for a in &op.args {
        match a {
            PaintArg::Token(t) => arg.extend(t.chars().filter(|c| c.is_ascii_alphanumeric())),
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
            PaintArg::Slots(slots) => {
                for (k, v) in slots {
                    arg.extend(k.chars().filter(|c| c.is_ascii_alphanumeric()));
                    arg.extend(v.display().chars().filter(|c| c.is_ascii_alphanumeric()));
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

fn build_selector(class: &str, ctx: &Ctx) -> String {
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
        "{ancestor}{attrs}.{class}{pseudo}",
        ancestor = ctx.ancestor,
        pseudo = ctx.pseudo,
    )
}

fn op_uses_tone(op: &PaintOp) -> bool {
    fn arg_uses_tone(a: &PaintArg) -> bool {
        match a {
            PaintArg::Token(t) => t == "tone" || t.starts_with("tone."),
            PaintArg::MemberPath { root, .. } => root == "tone",
            PaintArg::Slots(slots) => slots.values().any(arg_uses_tone),
            PaintArg::Bind(_) => false,
        }
    }
    op.args.iter().any(arg_uses_tone)
}

fn op_has_surface_member(op: &PaintOp) -> bool {
    op.args.iter().any(|a| {
        matches!(a, PaintArg::MemberPath { root, .. }
            if spinnaker::component_surfaces::current_surface_pack().as_deref() == Some(root.as_str()))
    })
}

fn resolve_surface_args(op: &PaintOp) -> PaintOp {
    use spinnaker::component_surfaces::resolve_surface_author;
    let args = op
        .args
        .iter()
        .map(|a| match a {
            PaintArg::MemberPath { root, path }
                if spinnaker::component_surfaces::current_surface_pack().as_deref()
                    == Some(root.as_str()) =>
            {
                match resolve_surface_author(root, path) {
                    Ok(public) => PaintArg::Token(public.to_string()),
                    Err(_) => a.clone(),
                }
            }
            PaintArg::Slots(slots) => PaintArg::Slots(
                slots
                    .iter()
                    .map(|(k, v)| {
                        let resolved = match v {
                            PaintArg::MemberPath { root, path }
                                if spinnaker::component_surfaces::current_surface_pack()
                                    .as_deref()
                                    == Some(root.as_str()) =>
                            {
                                match resolve_surface_author(root, path) {
                                    Ok(public) => PaintArg::Token(public.to_string()),
                                    Err(_) => v.clone(),
                                }
                            }
                            other => other.clone(),
                        };
                        (k.clone(), resolved)
                    })
                    .collect(),
            ),
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
) -> Result<String, Vec<spinnaker::ir::StyleDiagnostic>> {
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
) -> Result<(String, Vec<(String, String)>), Vec<spinnaker::ir::StyleDiagnostic>> {
    // Product authoring: flat concern chain. Constructor `style.pad().bg()` is
    // retired (corpus codemod + block attachment). Migration dual is `flat_chain` only.
    // Early refuse matches product `parse` (no IR for flat chains).
    let t = src.trim_start();
    if t.starts_with("style.") {
        return Err(vec![spinnaker::ir::StyleDiagnostic::ParseError {
            message: "flat `style.method(…)` chains are retired — write concern calls: \
                      style({ size({ w: full }).pad(4).bg(surface), when: { hover: bg(muted) } })"
                .into(),
            source: t.to_string(),
        }]);
    }
    let (style, diags) = spinnaker::parse::parse(src);
    if diags
        .iter()
        .any(|d| matches!(d, spinnaker::ir::StyleDiagnostic::ResidualProduct { .. }))
    {
        return Err(diags);
    }
    // Clash
    let clashes = spinnaker::clash::detect_clashes(&style, env);
    if !clashes.is_empty() {
        let mut all = diags;
        all.extend(clashes);
        return Err(all);
    }
    if diags.iter().any(|d| {
        match d {
            spinnaker::ir::StyleDiagnostic::BadToken { .. }
            | spinnaker::ir::StyleDiagnostic::ThemeOwnedToken { .. }
            | spinnaker::ir::StyleDiagnostic::UnknownSetter { .. }
            | spinnaker::ir::StyleDiagnostic::UnknownWhenKey { .. }
            | spinnaker::ir::StyleDiagnostic::BadArity { .. }
            | spinnaker::ir::StyleDiagnostic::UnknownConcernSlot { .. }
            | spinnaker::ir::StyleDiagnostic::DuplicateSlot { .. }
            | spinnaker::ir::StyleDiagnostic::RetiredSetter { .. }
            | spinnaker::ir::StyleDiagnostic::UnknownVocabulary { .. }
            | spinnaker::ir::StyleDiagnostic::ThemedLocalStyle { .. }
            | spinnaker::ir::StyleDiagnostic::NotAStyleRef { .. } => true,
            // Hard-fail lifecycle-in-when, geometry-in-interaction-when, color alias.
            spinnaker::ir::StyleDiagnostic::ParseError { message, .. } => {
                message.contains("cannot appear inside when")
                    || message.contains("cannot appear inside interaction when")
                    || message.contains("state can't gate layout")
                    || message.contains("was renamed to `ink")
                    || message.contains("was renamed to `fg")
                    || message.contains("member access")
                    || message.contains("does not resolve at the use site")
            }
            _ => false,
        }
    }) {
        return Err(diags);
    }
    let ref_diags = spinnaker::parse::validate_style_refs(&style, env);
    if !ref_diags.is_empty() {
        let mut all = diags;
        all.extend(ref_diags);
        return Err(all);
    }
    // Resolve Style::Ref through ThemeStyleEnv + env so named style decls
    // (`style ambientDot = { …, animate: { loop: play(…) } }` + `style(ambientDot)`)
    // still inject `_sail_m_*` attrs. CSS emit already expands refs; motion
    // collection must not skip them or ambient/enter/exit on named styles die.
    let motions = spinnaker::parse::collect_motion_attaches_env(&style, env);
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
        // Dedupe CSS by selector (not class). Two rules can share a class
        // while targeting different selectors; class-keyed dedupe would drop one.
        let dedupe_key = if r.selector.is_empty() {
            r.class.clone()
        } else {
            r.selector.clone()
        };
        if seen.insert(dedupe_key) {
            css.push_str(&render_rule(r));
        }
        if !classes.iter().any(|c| c == &r.class) {
            classes.push(r.class.clone());
        }
    }
    Ok((classes.join(" "), motions))
}
