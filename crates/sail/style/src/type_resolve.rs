//! Resolve type (family / size / leading) from Style IR and theme tokens.
//!
//! Layout and paint consume this resolution. They must not parse class
//! spelling (`s-fontSizexl`) to learn what Style already decided.

use crate::concerns::{find_concern, resolve_concern_name};
use crate::emit::StyleEnv;
use crate::ir::{PaintArg, PaintOp, Style};
use crate::theme_env::ThemeStyleEnv;
use crate::tokens;

/// Scale tokens spent by a Style on the type axis.
///
/// `size` is a text-scale step (`xl`, `2xl`), not a class name.
/// `family` is a font-scale token (`body`, `display`, `mono`, `heading`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedType {
    pub family: Option<String>,
    pub size: Option<String>,
    pub leading: Option<String>,
}

impl ResolvedType {
    pub fn is_empty(&self) -> bool {
        self.family.is_none() && self.size.is_none() && self.leading.is_none()
    }

    /// Overlay `other` onto self — set fields win.
    pub fn overlay(&mut self, other: &Self) {
        if other.family.is_some() {
            self.family.clone_from(&other.family);
        }
        if other.size.is_some() {
            self.size.clone_from(&other.size);
        }
        if other.leading.is_some() {
            self.leading.clone_from(&other.leading);
        }
    }
}

/// Walk Style IR and collect the resting type tokens (base layer, not when/hover).
pub fn resolve_type(style: &Style, env: &StyleEnv) -> ResolvedType {
    resolve_type_inner(style, env, &mut Vec::new())
}

/// Theme tag default (`h2` → `heading`) resolved through the type composite.
pub fn resolve_type_for_tag(tag: &str) -> ResolvedType {
    let Some(def) = ThemeStyleEnv::from_registry().resolve_tag_default(tag) else {
        return ResolvedType::default();
    };
    resolve_type_role(&def.type_role)
}

/// Type composite (`heading`, `body-sm`) → family / size / leading scale steps.
pub fn resolve_type_role(role: &str) -> ResolvedType {
    let Some(spec) = find_concern("type") else {
        return ResolvedType::default();
    };
    let Some(composite) = resolve_concern_name(spec, role) else {
        return ResolvedType::default();
    };
    let mut out = ResolvedType::default();
    for (slot, css) in composite.members {
        let Some(value) = tokens::token_value(css) else {
            continue;
        };
        match *slot {
            "size" => {
                if let Some(step) = size_step_from_css_value(value) {
                    out.size = Some(step);
                }
            }
            "family" => {
                if let Some(fam) = family_token_from_css_value(value) {
                    out.family = Some(fam);
                }
            }
            "leading" => {
                if let Some(lead) = leading_token_from_css_value(value) {
                    out.leading = Some(lead);
                }
            }
            _ => {}
        }
    }
    out
}

fn resolve_type_inner(style: &Style, env: &StyleEnv, resolving: &mut Vec<String>) -> ResolvedType {
    let mut out = ResolvedType::default();
    match style {
        Style::Empty | Style::MotionAttach { .. } => {}
        Style::Set(op) => out.overlay(&from_op(op)),
        Style::Chain(items) | Style::List(items) => {
            for item in items {
                out.overlay(&resolve_type_inner(item, env, resolving));
            }
        }
        Style::Compound { body, .. } => {
            out.overlay(&resolve_type_inner(body, env, resolving));
        }
        // Hover / media / attr branches are not the resting measure type.
        Style::When { .. } | Style::Match { .. } => {}
        Style::Ref(name) => {
            if resolving.iter().any(|n| n == name) {
                return out;
            }
            resolving.push(name.clone());
            if let Some(inner) = env.get(name) {
                out.overlay(&resolve_type_inner(inner, env, resolving));
            }
            resolving.pop();
        }
    }
    out
}

fn from_op(op: &PaintOp) -> ResolvedType {
    match op.setter.as_str() {
        "fontSize" => ResolvedType {
            size: token_arg(op.first_arg()),
            ..Default::default()
        },
        "fontFamily" => ResolvedType {
            family: token_arg(op.first_arg()),
            ..Default::default()
        },
        "leading" => ResolvedType {
            leading: token_arg(op.first_arg()),
            ..Default::default()
        },
        "type" => match op.first_arg() {
            PaintArg::Token(name) if !name.is_empty() => resolve_type_role(name),
            PaintArg::Slots(slots) => {
                let mut out = ResolvedType::default();
                if let Some(v) = slots.get("family").and_then(token_arg_ref) {
                    out.family = Some(v);
                }
                if let Some(v) = slots.get("size").and_then(token_arg_ref) {
                    out.size = Some(v);
                }
                if let Some(v) = slots.get("leading").and_then(token_arg_ref) {
                    out.leading = Some(v);
                }
                out
            }
            _ => ResolvedType::default(),
        },
        _ => ResolvedType::default(),
    }
}

fn token_arg(arg: &PaintArg) -> Option<String> {
    token_arg_ref(arg)
}

fn token_arg_ref(arg: &PaintArg) -> Option<String> {
    match arg {
        PaintArg::Token(t) if !t.is_empty() => Some(t.clone()),
        _ => None,
    }
}

fn parse_css_var(value: &str) -> Option<&str> {
    let rest = value.trim().strip_prefix("var(--")?;
    let name = rest.split([')', ',']).next()?.trim();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn size_step_from_css_value(value: &str) -> Option<String> {
    let name = parse_css_var(value)?;
    if let Some(step) = name.strip_prefix("text-") {
        return Some(step.to_string());
    }
    tokens::token_value(name).and_then(size_step_from_css_value)
}

fn family_token_from_css_value(value: &str) -> Option<String> {
    let name = parse_css_var(value)?;
    if let Some(fam) = name.strip_suffix("-font") {
        if fam != "heading" && !fam.is_empty() {
            return Some(fam.to_string());
        }
        // `heading-font` aliases the display face — follow one more hop
        // so the measure key is the font scale token, not the role name.
        if fam == "heading" {
            if let Some(next) = tokens::token_value(name) {
                if let Some(resolved) = family_token_from_css_value(next) {
                    return Some(resolved);
                }
            }
            return Some("heading".into());
        }
    }
    if let Some(fam) = name.strip_prefix("font-") {
        return Some(fam.to_string());
    }
    tokens::token_value(name).and_then(family_token_from_css_value)
}

fn leading_token_from_css_value(value: &str) -> Option<String> {
    let name = parse_css_var(value)?;
    if let Some(step) = name.strip_prefix("leading-") {
        return Some(step.to_string());
    }
    tokens::token_value(name).and_then(leading_token_from_css_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use std::collections::HashMap;

    fn resolved(src: &str) -> ResolvedType {
        let (style, diags) = parse(src);
        assert!(
            diags.is_empty(),
            "{src}: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        resolve_type(&style, &HashMap::new())
    }

    #[test]
    fn type_size_slot_is_the_scale_step() {
        let t = resolved("style({ text: type({ size: xl }) })");
        assert_eq!(t.size.as_deref(), Some("xl"));
        assert!(t.family.is_none());
    }

    #[test]
    fn type_size_step_change_is_visible() {
        let xl = resolved("style({ text: type({ size: xl }) })");
        let xxl = resolved("style({ text: type({ size: 2xl }) })");
        assert_eq!(xl.size.as_deref(), Some("xl"));
        assert_eq!(xxl.size.as_deref(), Some("2xl"));
        assert_ne!(xl.size, xxl.size);
    }

    #[test]
    fn type_role_heading_spends_theme_size_token() {
        let t = resolve_type_role("heading");
        let expected = size_step_from_css_value(
            tokens::token_value("type-heading-size").expect("type-heading-size"),
        );
        assert_eq!(t.size, expected);
        assert_eq!(t.size.as_deref(), Some("xl"));
        assert_eq!(t.family.as_deref(), Some("display"));
    }

    #[test]
    fn tag_h2_uses_heading_role() {
        let t = resolve_type_for_tag("h2");
        let heading = resolve_type_role("heading");
        assert_eq!(t, heading);
        assert_eq!(t.size.as_deref(), Some("xl"));
    }

    #[test]
    fn tag_p_uses_body_role() {
        let t = resolve_type_for_tag("p");
        assert_eq!(t.size.as_deref(), Some("base"));
        assert_eq!(t.family.as_deref(), Some("body"));
    }

    #[test]
    fn tag_without_default_is_empty() {
        let t = resolve_type_for_tag("span");
        assert!(t.is_empty());
    }

    #[test]
    fn type_family_and_size_slots() {
        let t = resolved("style({ text: type({ family: mono, size: sm }) })");
        assert_eq!(t.family.as_deref(), Some("mono"));
        assert_eq!(t.size.as_deref(), Some("sm"));
    }

    #[test]
    fn font_size_setter_still_resolves() {
        // Retired in product language; IR still carries the setter for resolve.
        let (style, _) = parse("style({ text: fontSize(2xl) })");
        let t = resolve_type(&style, &HashMap::new());
        assert_eq!(t.size.as_deref(), Some("2xl"));
    }

    #[test]
    fn when_hover_does_not_replace_resting_type() {
        let t = resolved(
            "style({ text: type({ size: sm }), when: { hover: { text: type({ size: xl }) } } })",
        );
        assert_eq!(t.size.as_deref(), Some("sm"));
    }

    #[test]
    fn ref_follows_env() {
        let (named, _) = parse("style({ text: type({ size: xl }) })");
        let mut env = HashMap::new();
        env.insert("title".into(), named);
        let (style, diags) = parse("style(title)");
        assert!(diags.is_empty(), "{diags:?}");
        let t = resolve_type(&style, &env);
        assert_eq!(t.size.as_deref(), Some("xl"));
    }
}
