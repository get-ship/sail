//! Table-driven Style setters → sx vocab atoms / CSS scales.

use crate::scales;
use crate::sx_ir::{self, SxDecl, SxValue};
use crate::sx_vocab::{self, Resolved};

use super::ir::{PaintArg, PaintOp, StyleDiagnostic};

/// One product setter: `style.bg(token)`.
#[derive(Debug, Clone, Copy)]
pub struct SetterSpec {
    pub name: &'static str,
    /// Vocab utility head used for resolve (`bg`, `px`, `rounded`, …).
    pub vocab: &'static str,
    /// Scale name for token check (`color`, `space`, `radius`, …), if any.
    pub scale: Option<&'static str>,
    /// Extra accepted keywords beyond the scale.
    pub keywords: &'static [&'static str],
}

/// Closed product setter table (re-homes full vocab coverage via vocab heads).
pub fn setters() -> &'static [SetterSpec] {
    &[
        SetterSpec { name: "bg", vocab: "bg", scale: Some("color"), keywords: &["transparent", "current", "inherit"] },
        SetterSpec { name: "fg", vocab: "text", scale: Some("color"), keywords: &["transparent", "current", "inherit"] },
        SetterSpec { name: "color", vocab: "text", scale: Some("color"), keywords: &["transparent", "current"] },
        SetterSpec { name: "border", vocab: "border", scale: Some("color"), keywords: &["transparent", "current"] },
        SetterSpec { name: "borderX", vocab: "border-x", scale: Some("color"), keywords: &[] },
        SetterSpec { name: "borderY", vocab: "border-y", scale: Some("color"), keywords: &[] },
        SetterSpec { name: "borderS", vocab: "border-s", scale: Some("color"), keywords: &[] },
        SetterSpec { name: "borderWidth", vocab: "border", scale: None, keywords: &["0", "1", "2", "3", "4", "8"] },
        SetterSpec { name: "radius", vocab: "rounded", scale: Some("radius"), keywords: &["none", "full"] },
        SetterSpec { name: "pad", vocab: "p", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "padX", vocab: "px", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "padY", vocab: "py", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "padT", vocab: "pt", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "padB", vocab: "pb", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "padL", vocab: "pl", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "padR", vocab: "pr", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "m", vocab: "m", scale: Some("space"), keywords: &["0", "auto"] },
        SetterSpec { name: "mx", vocab: "mx", scale: Some("space"), keywords: &["0", "auto"] },
        SetterSpec { name: "my", vocab: "my", scale: Some("space"), keywords: &["0", "auto"] },
        SetterSpec { name: "minH", vocab: "min-h", scale: Some("size"), keywords: &["0", "full", "screen"] },
        SetterSpec { name: "minW", vocab: "min-w", scale: Some("size"), keywords: &["0", "full"] },
        SetterSpec { name: "w", vocab: "w", scale: Some("size"), keywords: &["0", "full", "auto"] },
        SetterSpec { name: "h", vocab: "h", scale: Some("size"), keywords: &["0", "full", "auto"] },
        SetterSpec { name: "size", vocab: "size", scale: Some("size"), keywords: &["0", "full"] },
        SetterSpec { name: "fontSize", vocab: "text", scale: Some("text"), keywords: &[] },
        SetterSpec { name: "fontWeight", vocab: "font", scale: Some("weight"), keywords: &[] },
        SetterSpec { name: "fontFamily", vocab: "font", scale: Some("font"), keywords: &["body", "heading", "mono", "display"] },
        SetterSpec { name: "leading", vocab: "leading", scale: Some("leading"), keywords: &["none"] },
        SetterSpec { name: "tracking", vocab: "tracking", scale: Some("tracking"), keywords: &[] },
        SetterSpec { name: "shadow", vocab: "shadow", scale: Some("shadow"), keywords: &["none"] },
        SetterSpec { name: "opacity", vocab: "opacity", scale: None, keywords: &["0", "5", "10", "20", "25", "30", "40", "50", "60", "70", "75", "80", "90", "95", "100"] },
        SetterSpec { name: "whitespace", vocab: "whitespace", scale: None, keywords: &["normal", "nowrap", "pre", "pre-line", "pre-wrap"] },
        SetterSpec { name: "align", vocab: "align", scale: None, keywords: &["baseline", "top", "middle", "bottom", "text-top", "text-bottom"] },
        SetterSpec { name: "ring", vocab: "outline", scale: Some("color"), keywords: &["none"] },
        SetterSpec { name: "outline", vocab: "outline", scale: None, keywords: &["none", "solid", "dashed", "1", "2", "4"] },
        SetterSpec { name: "outlineOffset", vocab: "outline-offset", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "translateX", vocab: "translate-x", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "translateY", vocab: "translate-y", scale: Some("space"), keywords: &["0"] },
        SetterSpec { name: "z", vocab: "z", scale: Some("z"), keywords: &["0", "1", "2", "auto"] },
        SetterSpec { name: "cursor", vocab: "cursor", scale: None, keywords: &["auto", "default", "pointer", "not-allowed", "text"] },
        SetterSpec { name: "select", vocab: "select", scale: None, keywords: &["none", "text", "all", "auto"] },
        SetterSpec { name: "pointerEvents", vocab: "pointer-events", scale: None, keywords: &["none", "auto"] },
        SetterSpec { name: "overflow", vocab: "overflow", scale: None, keywords: &["auto", "hidden", "clip", "visible", "scroll"] },
        SetterSpec { name: "display", vocab: "hidden", scale: None, keywords: &["none", "block", "inline", "flex", "inline-flex"] },
    ]
}

pub fn find_setter(name: &str) -> Option<&'static SetterSpec> {
    setters().iter().find(|s| s.name == name)
}

/// Closed `when({ key: Style })` browser-state keys → CSS selector suffix.
pub fn when_selector(key: &str) -> Option<&'static str> {
    match key {
        "hover" => Some(":hover"),
        "focus" => Some(":focus-visible"),
        "focusWithin" | "focus-within" => Some(":focus-within"),
        "active" => Some(":active"),
        "disabled" => Some(":is(:disabled,[aria-disabled='true'])"),
        "empty" => Some(":empty"),
        "before" => Some("::before"),
        "after" => Some("::after"),
        "selected" => Some("[aria-selected='true']"),
        "expanded" => Some("[aria-expanded='true']"),
        "checked" => Some("[aria-checked='true']"),
        "pressed" => Some("[aria-pressed='true']"),
        "open" => Some("[open]"),
        "current" => Some("[aria-current]:not([aria-current='false'])"),
        _ => None,
    }
}

/// Validate paint arg against setter table; fail closed.
pub fn validate_op(op: &PaintOp) -> Result<(), StyleDiagnostic> {
    let Some(spec) = find_setter(&op.setter) else {
        return Err(StyleDiagnostic::UnknownSetter {
            setter: op.setter.clone(),
            source: op.source.clone(),
        });
    };
    match &op.arg {
        PaintArg::Bind(_) => Ok(()), // name resolution is compiler/scope concern
        PaintArg::Token(tok) => {
            if spec.keywords.iter().any(|k| *k == tok.as_str()) {
                return Ok(());
            }
            // Prefer scale membership.
            if let Some(scale) = spec.scale {
                if scales::contains(scale, tok) {
                    return Ok(());
                }
                // Color tokens often use bare css names (primary, muted-foreground).
                if scale == "color" || scale == "font" || scale == "weight" || scale == "text" {
                    // Probe via vocab resolve on the atom.
                    if lower_op_css(op).is_ok() {
                        return Ok(());
                    }
                }
            } else if lower_op_css(op).is_ok() {
                return Ok(());
            }
            // Last resort: vocab accepts the atom.
            if lower_op_css(op).is_ok() {
                return Ok(());
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Lower a PaintOp to CSS declarations via sx vocab (table-driven).
pub fn lower_op_css(op: &PaintOp) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let Some(spec) = find_setter(&op.setter) else {
        return Err(StyleDiagnostic::UnknownSetter {
            setter: op.setter.clone(),
            source: op.source.clone(),
        });
    };
    let (value, bare) = match &op.arg {
        PaintArg::Token(t) => (Some(SxValue::Token(t.clone())), false),
        PaintArg::Bind(name) => {
            let css = css_name_for_bind(name);
            return Ok(bind_css_props(spec, &css));
        }
    };
    // Special-case display/hidden
    if spec.name == "display" {
        let tok = match &op.arg {
            PaintArg::Token(t) => t.as_str(),
            _ => "none",
        };
        let css = match tok {
            "none" => vec![("display".into(), "none".into())],
            "block" => vec![("display".into(), "block".into())],
            "inline" => vec![("display".into(), "inline".into())],
            "flex" => vec![("display".into(), "flex".into())],
            "inline-flex" => vec![("display".into(), "inline-flex".into())],
            _ => return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.into(),
                source: op.source.clone(),
            }),
        };
        return Ok(css);
    }
    // fontWeight / fontFamily share vocab head `font` — disambiguate via atom value.
    let decl = SxDecl {
        prefixes: Vec::new(),
        prop: spec.vocab.to_string(),
        value,
        negative: false,
        bare,
        source: op.source.clone(),
    };
    // Try candidates for color tokens (onPrimary → on-primary, etc.)
    if let Some(SxValue::Token(ref t)) = decl.value {
        for cand in token_candidates(t) {
            let mut d = decl.clone();
            d.value = Some(SxValue::Token(cand.clone()));
            let atom = sx_ir::decl_atom(&d);
            if let Resolved::Ok(pairs) = sx_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            // Also try raw prop-cand atom
            let atom2 = format!("{}-{}", spec.vocab, cand);
            if let Resolved::Ok(pairs) = sx_vocab::resolve(&atom2) {
                return Ok(pairs);
            }
        }
    }
    match sx_ir::lower_decl(&decl) {
        Some(pairs) => Ok(pairs),
        None => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: match &op.arg {
                PaintArg::Token(t) => t.clone(),
                PaintArg::Bind(n) => format!("bind({n})"),
            },
            source: op.source.clone(),
        }),
    }
}

fn token_candidates(tok: &str) -> Vec<String> {
    let mut out = vec![tok.to_string()];
    // camelCase → kebab
    let mut kebab = String::new();
    for (i, c) in tok.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                kebab.push('-');
            }
            kebab.extend(c.to_lowercase());
        } else {
            kebab.push(c);
        }
    }
    if kebab != tok {
        out.push(kebab.clone());
    }
    // on-primary style already
    out.push(tok.replace('_', "-"));
    out
}

fn css_name_for_bind(name: &str) -> String {
    // camelCase prop/signal → kebab css custom property
    let mut kebab = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                kebab.push('-');
            }
            kebab.extend(c.to_lowercase());
        } else {
            kebab.push(c);
        }
    }
    kebab
}

fn bind_css_props(spec: &SetterSpec, css_name: &str) -> Vec<(String, String)> {
    let var = format!("var(--{css_name})");
    // Probe CSS property list from vocab.
    let probe = SxDecl {
        prefixes: Vec::new(),
        prop: spec.vocab.to_string(),
        value: Some(SxValue::Token("primary".into())),
        negative: false,
        bare: false,
        source: String::new(),
    };
    if let Some(pairs) = sx_ir::lower_decl(&probe) {
        return pairs.into_iter().map(|(p, _)| (p, var.clone())).collect();
    }
    // Fallbacks by setter name
    let prop = match spec.name {
        "bg" => "background-color",
        "fg" | "color" => "color",
        "translateY" => "translate",
        "translateX" => "translate",
        "padX" => "padding-inline",
        "padY" => "padding-block",
        "radius" => "border-radius",
        other => other,
    };
    if spec.name == "translateY" {
        return vec![("translate".into(), format!("var(--sx-translate-x, 0) {var}"))];
    }
    if spec.name == "translateX" {
        return vec![("translate".into(), format!("{var} var(--sx-translate-y, 0)"))];
    }
    vec![(prop.into(), var)]
}
