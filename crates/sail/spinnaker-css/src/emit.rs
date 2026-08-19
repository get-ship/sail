//! Style IR → atomic CSS (token vars, attr selectors, when pseudos).

use std::collections::{HashMap, HashSet};

use crate::lower::lower_op_css;
use spinnaker::ResolvedType;
use spinnaker::intent::{Intent, IntentNode, IntentValue, lower_to_intent};
use spinnaker::ir::{PaintArg, PaintOp, Style, StyleEnv};
use spinnaker::theme_env::{ThemeStyleEnv, current_styling_policy};

use crate::when::{WhenTarget, when_target};

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

/// Emit tag-default / leftover type through the one Style emitter.
///
/// One class per filled slot (`s-type-size-4xl`, `s-type-family-display`) so
/// size shares with authored `type({ size })` spends.
pub fn emit_resolved_type(
    ty: &ResolvedType,
    css: &mut String,
    seen: &mut HashSet<String>,
) -> String {
    let env: StyleEnv = HashMap::new();
    let mut classes = Vec::new();
    for (slot, value) in [
        ("family", ty.family.as_deref()),
        ("size", ty.size.as_deref()),
        ("leading", ty.leading.as_deref()),
    ] {
        let Some(value) = value else {
            continue;
        };
        let src = format!("style({{ type({{ {slot}: {value} }}) }})");
        let Ok(part) = lower_style_source(&src, &env, css, seen) else {
            continue;
        };
        if !part.is_empty() {
            classes.push(part);
        }
    }
    classes.join(" ")
}

fn append_rules(rules: &[CssRule], css: &mut String, seen: &mut HashSet<String>) -> String {
    let mut classes = Vec::new();
    for r in rules {
        if r.selector.is_empty() && r.declarations.is_empty() {
            if !classes.iter().any(|c| c == &r.class) {
                classes.push(r.class.clone());
            }
            continue;
        }
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
    classes.join(" ")
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
        let tag = if v == "true" || v == "*" || v.is_empty() {
            kebab_ident(k)
        } else {
            format!("{}-{}", kebab_ident(k), kebab_ident(v))
        };
        n.path_tags.push(tag);
        n
    }
    fn with_pseudo(&self, key: &str, sel: &str) -> Self {
        let mut n = self.clone();
        n.pseudo.push_str(sel);
        n.path_tags.push(kebab_ident(key));
        n
    }
    fn with_ancestor(&self, key: &str, anc: &str) -> Self {
        let mut n = self.clone();
        n.ancestor = format!("{}{anc}", n.ancestor);
        n.path_tags.push(kebab_ident(key));
        n
    }
    fn with_media(&self, key: &str, media: &str) -> Self {
        let mut n = self.clone();
        n.media = Some(media.into());
        n.path_tags.push(kebab_ident(key));
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
        "s-play-{}-{}",
        kebab_ident(&preset),
        kebab_ident(&duration_tok)
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

/// Author vocabulary, kebabed. CSS class fragment — no leading/trailing dash.
fn kebab_ident(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    let mut prev_dash = true;
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if !prev_dash && i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if c == '-' {
            if !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        } else if c == '%' {
            out.push_str("pct");
            prev_dash = false;
        } else if c == '.' {
            if !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn is_numeric_token(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '_')
}

fn arg_vocab(a: &PaintArg) -> String {
    match a {
        PaintArg::Token(t) => kebab_ident(t),
        PaintArg::Bind(n) => format!("bind-{}", kebab_ident(n)),
        PaintArg::MemberPath { root, path } => {
            let mut parts = vec![kebab_ident(root)];
            parts.extend(path.iter().map(|s| kebab_ident(s)));
            parts.join("-")
        }
        PaintArg::Slots(slots) => {
            // Bare `bg(muted)` / `pad(4)` lower to a single default slot.
            // Elide it so the class reads like the Style attr.
            if slots.len() == 1 {
                if let Some((k, v)) = slots.iter().next() {
                    if matches!(k.as_str(), "all" | "color") {
                        return arg_vocab(v);
                    }
                }
            }
            slots
                .iter()
                .map(|(k, v)| slot_vocab(k, v))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-")
        }
    }
}

fn slot_vocab(k: &str, v: &PaintArg) -> String {
    let key = kebab_ident(k);
    let val = arg_vocab(v);
    if key.is_empty() {
        return val;
    }
    if val.is_empty() {
        return key;
    }
    if is_numeric_token(&val) {
        format!("{key}{val}")
    } else {
        format!("{key}-{val}")
    }
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
    // `[variant=soft].s-variant-soft-bg-muted`; later elements re-emit that
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
    // One class per concern, kebabed after the author vocabulary.
    let setter = kebab_ident(&op.setter);
    let arg = op
        .args
        .iter()
        .map(arg_vocab)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let path = ctx
        .path_tags
        .iter()
        .filter(|t| !t.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("-");
    match (path.is_empty(), arg.is_empty()) {
        (true, true) => format!("s-{setter}"),
        (true, false) => format!("s-{setter}-{arg}"),
        (false, true) => format!("s-{path}-{setter}"),
        (false, false) => format!("s-{path}-{setter}-{arg}"),
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
    // retired (corpus codemod + block attachment). Remaining dual is `flat_chain` only.
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
    Ok((append_rules(&emit_style(&style, env), css, seen), motions))
}
