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
    /// When true, token may be prefixed with `-` (negatable space/size).
    pub negatable: bool,
}

/// Closed product setter table (re-homes full vocab coverage via vocab heads).
pub fn setters() -> &'static [SetterSpec] {
    &[
        SetterSpec { name: "bg", vocab: "bg", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "inherit", "none"], negatable: false },
        SetterSpec { name: "fg", vocab: "text", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "inherit"], negatable: false },
        SetterSpec { name: "color", vocab: "text", scale: Some("color"), keywords: &["transparent", "current", "currentColor"], negatable: false },
        // border disambiguates color / width / style by value via vocab.
        SetterSpec {
            name: "border",
            vocab: "border",
            scale: Some("color"),
            keywords: &["transparent", "current", "currentColor", "solid", "dashed", "dotted", "none", "0", "1", "2", "3", "4", "8"],
            negatable: false,
        },
        SetterSpec { name: "borderX", vocab: "border-x", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderY", vocab: "border-y", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderS", vocab: "border-s", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderE", vocab: "border-e", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderT", vocab: "border-t", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderB", vocab: "border-b", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderR", vocab: "border-r", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderL", vocab: "border-l", scale: Some("color"), keywords: &["transparent", "current", "currentColor", "0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderWidth", vocab: "border", scale: None, keywords: &["0", "1", "2", "3", "4", "8", "control-stroke"], negatable: false },
        SetterSpec { name: "borderBWidth", vocab: "border-b", scale: None, keywords: &["0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderTWidth", vocab: "border-t", scale: None, keywords: &["0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderEWidth", vocab: "border-e", scale: None, keywords: &["0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderSWidth", vocab: "border-s", scale: None, keywords: &["0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderRWidth", vocab: "border-r", scale: None, keywords: &["0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "borderLWidth", vocab: "border-l", scale: None, keywords: &["0", "1", "2", "3", "4", "8"], negatable: false },
        SetterSpec { name: "radius", vocab: "rounded", scale: Some("radius"), keywords: &["none", "full"], negatable: false },
        SetterSpec { name: "radiusT", vocab: "rounded-t", scale: Some("radius"), keywords: &["none", "full"], negatable: false },
        SetterSpec { name: "radiusB", vocab: "rounded-b", scale: Some("radius"), keywords: &["none", "full"], negatable: false },
        SetterSpec { name: "radiusL", vocab: "rounded-l", scale: Some("radius"), keywords: &["none", "full"], negatable: false },
        SetterSpec { name: "radiusR", vocab: "rounded-r", scale: Some("radius"), keywords: &["none", "full"], negatable: false },
        SetterSpec { name: "pad", vocab: "p", scale: Some("space"), keywords: &["0"], negatable: false },
        SetterSpec { name: "padX", vocab: "px", scale: Some("space"), keywords: &["0"], negatable: false },
        SetterSpec { name: "padY", vocab: "py", scale: Some("space"), keywords: &["0"], negatable: false },
        SetterSpec { name: "padT", vocab: "pt", scale: Some("space"), keywords: &["0"], negatable: false },
        SetterSpec { name: "padB", vocab: "pb", scale: Some("space"), keywords: &["0"], negatable: false },
        SetterSpec { name: "padL", vocab: "pl", scale: Some("space"), keywords: &["0"], negatable: false },
        SetterSpec { name: "padR", vocab: "pr", scale: Some("space"), keywords: &["0"], negatable: false },
        SetterSpec { name: "m", vocab: "m", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "mx", vocab: "mx", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "my", vocab: "my", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "mt", vocab: "mt", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "mb", vocab: "mb", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "ms", vocab: "ms", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "me", vocab: "me", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "minH", vocab: "min-h", scale: Some("size"), keywords: &["0", "full", "screen"], negatable: false },
        SetterSpec { name: "minW", vocab: "min-w", scale: Some("size"), keywords: &["0", "full"], negatable: false },
        SetterSpec {
            name: "w",
            vocab: "w",
            scale: Some("size"),
            keywords: &["0", "full", "auto", "border-width", "toast-region-width"],
            negatable: false,
        },
        SetterSpec {
            name: "h",
            vocab: "h",
            scale: Some("size"),
            keywords: &["0", "full", "auto", "border-width"],
            negatable: false,
        },
        SetterSpec { name: "size", vocab: "size", scale: Some("size"), keywords: &["0", "full"], negatable: false },
        SetterSpec { name: "fontSize", vocab: "text", scale: Some("text"), keywords: &[], negatable: false },
        SetterSpec { name: "fontWeight", vocab: "font", scale: Some("weight"), keywords: &[], negatable: false },
        SetterSpec { name: "fontFamily", vocab: "font", scale: Some("font"), keywords: &["body", "heading", "mono", "display"], negatable: false },
        SetterSpec { name: "leading", vocab: "leading", scale: Some("leading"), keywords: &["none"], negatable: false },
        SetterSpec { name: "tracking", vocab: "tracking", scale: Some("tracking"), keywords: &[], negatable: false },
        SetterSpec { name: "shadow", vocab: "shadow", scale: Some("shadow"), keywords: &["none"], negatable: false },
        SetterSpec { name: "opacity", vocab: "opacity", scale: None, keywords: &["0", "5", "10", "20", "25", "30", "40", "50", "60", "70", "75", "80", "90", "95", "100"], negatable: false },
        SetterSpec { name: "whitespace", vocab: "whitespace", scale: None, keywords: &["normal", "nowrap", "pre", "pre-line", "pre-wrap"], negatable: false },
        SetterSpec { name: "align", vocab: "align", scale: None, keywords: &["baseline", "top", "middle", "bottom", "text-top", "text-bottom"], negatable: false },
        SetterSpec { name: "ring", vocab: "outline", scale: Some("color"), keywords: &["none"], negatable: false },
        SetterSpec { name: "outline", vocab: "outline", scale: None, keywords: &["none", "solid", "dashed", "1", "2", "4", "ring-width", "ring"], negatable: false },
        SetterSpec { name: "outlineOffset", vocab: "outline-offset", scale: Some("space"), keywords: &["0", "ring-offset", "ring-inset"], negatable: false },
        SetterSpec { name: "translateX", vocab: "translate-x", scale: Some("space"), keywords: &["0"], negatable: true },
        SetterSpec { name: "translateY", vocab: "translate-y", scale: Some("space"), keywords: &["0"], negatable: true },
        SetterSpec {
            name: "z",
            vocab: "z",
            scale: Some("z"),
            keywords: &["0", "1", "2", "auto", "toast-region-z"],
            negatable: false,
        },
        SetterSpec { name: "cursor", vocab: "cursor", scale: None, keywords: &["auto", "default", "pointer", "not-allowed", "text"], negatable: false },
        SetterSpec { name: "select", vocab: "select", scale: None, keywords: &["none", "text", "all", "auto"], negatable: false },
        SetterSpec { name: "pointerEvents", vocab: "pointer-events", scale: None, keywords: &["none", "auto"], negatable: false },
        SetterSpec { name: "overflow", vocab: "overflow", scale: None, keywords: &["auto", "hidden", "clip", "visible", "scroll"], negatable: false },
        SetterSpec { name: "display", vocab: "hidden", scale: None, keywords: &["none", "block", "inline", "inline-block", "flex", "inline-flex", "contents"], negatable: false },
        SetterSpec { name: "position", vocab: "relative", scale: None, keywords: &["relative", "absolute", "sticky", "fixed", "static"], negatable: false },
        SetterSpec { name: "top", vocab: "top", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "bottom", vocab: "bottom", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "left", vocab: "left", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "right", vocab: "right", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "inset", vocab: "inset", scale: Some("space"), keywords: &["0", "auto"], negatable: true },
        SetterSpec { name: "appearance", vocab: "appearance", scale: None, keywords: &["none", "auto"], negatable: false },
        SetterSpec { name: "content", vocab: "content", scale: None, keywords: &["empty"], negatable: false },
        SetterSpec { name: "shrink", vocab: "shrink", scale: None, keywords: &["0", "1"], negatable: false },
        SetterSpec { name: "grow", vocab: "grow", scale: None, keywords: &["0", "1"], negatable: false },
        SetterSpec {
            name: "self",
            vocab: "self",
            scale: None,
            keywords: &["auto", "start", "center", "end", "stretch", "baseline"],
            negatable: false,
        },
        SetterSpec { name: "box", vocab: "box", scale: None, keywords: &["border", "content"], negatable: false },
        SetterSpec { name: "transition", vocab: "transition", scale: None, keywords: &["none", "all", "colors", "opacity", "shadow", "transform"], negatable: false },
        SetterSpec { name: "duration", vocab: "duration", scale: Some("duration"), keywords: &["fast", "medium", "slow", "instant", "deliberate", "glacial"], negatable: false },
        SetterSpec { name: "ease", vocab: "ease", scale: Some("ease"), keywords: &["in", "out", "in-out", "out-soft", "spring"], negatable: false },
        SetterSpec { name: "rotate", vocab: "rotate", scale: Some("angle"), keywords: &["0"], negatable: true },
        SetterSpec { name: "tapHighlight", vocab: "tap-highlight", scale: None, keywords: &["none", "auto"], negatable: false },
        SetterSpec { name: "saturate", vocab: "saturate", scale: None, keywords: &["0", "50", "60", "65", "75", "100"], negatable: false },
        // Host marker for group-* descendant PE — class `sail-grp`, no CSS decls.
        SetterSpec { name: "group", vocab: "group", scale: None, keywords: &[], negatable: false },
        // Bare composite atoms
        SetterSpec { name: "animate", vocab: "animate", scale: None, keywords: &["spin"], negatable: false },
        SetterSpec { name: "bgImage", vocab: "bg-image", scale: None, keywords: &["none", "primary-accent"], negatable: false },
        SetterSpec { name: "avatarMedia", vocab: "avatar-media", scale: None, keywords: &["true", "on"], negatable: false },
        SetterSpec {
            name: "flexDir",
            vocab: "flex-dir",
            scale: None,
            keywords: &["row", "column", "row-reverse", "column-reverse"],
            negatable: false,
        },
    ]
}

pub fn find_setter(name: &str) -> Option<&'static SetterSpec> {
    setters().iter().find(|s| s.name == name)
}

/// Resolved `when({ key })` target: pseudo/attr on self, or ancestor prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhenTarget {
    /// Appended to the element selector (`:hover`, `[aria-selected='true']`, …).
    Pseudo(&'static str),
    /// Prepended ancestor context (`.sail-grp[required] `).
    Ancestor(&'static str),
    /// Media condition wrapping the rule.
    Media(&'static str),
}

/// Closed `when({ key: Style })` browser/framework-state keys.
pub fn when_target(key: &str) -> Option<WhenTarget> {
    match key {
        "hover" => Some(WhenTarget::Pseudo(":hover")),
        "focus" => Some(WhenTarget::Pseudo(":focus-visible")),
        "focusWithin" | "focus-within" => Some(WhenTarget::Pseudo(":focus-within")),
        "active" => Some(WhenTarget::Pseudo(":active")),
        "disabled" => Some(WhenTarget::Pseudo(":is(:disabled,[aria-disabled='true'])")),
        "empty" => Some(WhenTarget::Pseudo(":empty")),
        "before" => Some(WhenTarget::Pseudo("::before")),
        "after" => Some(WhenTarget::Pseudo("::after")),
        "first" | "first-child" => Some(WhenTarget::Pseudo(":first-child")),
        "last" | "last-child" => Some(WhenTarget::Pseudo(":last-child")),
        "selected" => Some(WhenTarget::Pseudo("[aria-selected='true']")),
        "expanded" => Some(WhenTarget::Pseudo("[aria-expanded='true']")),
        // Native checkbox/switch + ARIA — both forms appear in host skins.
        "checked" => Some(WhenTarget::Pseudo(":is(:checked,[aria-checked='true'])")),
        "pressed" => Some(WhenTarget::Pseudo("[aria-pressed='true']")),
        "open" => Some(WhenTarget::Pseudo("[open]")),
        "current" => Some(WhenTarget::Pseudo("[aria-current]:not([aria-current='false'])")),
        "group-required" | "groupRequired" => Some(WhenTarget::Ancestor(".sail-grp[required] ")),
        "group-invalid" | "groupInvalid" => Some(WhenTarget::Ancestor(".sail-grp[invalid] ")),
        "group-disabled" | "groupDisabled" => {
            Some(WhenTarget::Ancestor(".sail-grp:is(:disabled,[aria-disabled='true'],[disabled]) "))
        }
        "group-hover" | "groupHover" => Some(WhenTarget::Ancestor(".sail-grp:hover ")),
        "group-focus-within" | "groupFocusWithin" => {
            Some(WhenTarget::Ancestor(".sail-grp:focus-within "))
        }
        "motion-reduce" | "motionReduce" => {
            Some(WhenTarget::Media("(prefers-reduced-motion: reduce)"))
        }
        "forced-colors" | "forcedColors" => Some(WhenTarget::Media("(forced-colors: active)")),
        _ => None,
    }
}

/// Back-compat helper used by emit for simple pseudo append.
pub fn when_selector(key: &str) -> Option<&'static str> {
    match when_target(key)? {
        WhenTarget::Pseudo(s) => Some(s),
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
    // Marker setters with empty arg (group).
    if spec.name == "group" {
        return Ok(());
    }
    match &op.arg {
        PaintArg::Bind(_) => Ok(()),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg && !spec.negatable {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
            if spec.keywords.iter().any(|k| *k == bare) {
                return Ok(());
            }
            if let Some(scale) = spec.scale {
                if scales::contains(scale, bare) {
                    return Ok(());
                }
                if scale == "color" || scale == "font" || scale == "weight" || scale == "text"
                    || scale == "duration" || scale == "ease" || scale == "space" || scale == "size"
                    || scale == "z" || scale == "shadow" || scale == "leading" || scale == "tracking"
                    || scale == "radius"
                {
                    if lower_op_css(op).is_ok() {
                        return Ok(());
                    }
                }
            } else if lower_op_css(op).is_ok() {
                return Ok(());
            }
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
    // group: host marker only — no CSS declarations.
    if spec.name == "group" {
        return Ok(Vec::new());
    }
    let (value, bare, negative) = match &op.arg {
        PaintArg::Token(t) => {
            let (neg, bare) = strip_neg(t);
            (Some(SxValue::Token(bare.to_string())), bare.is_empty(), neg)
        }
        PaintArg::Bind(name) => {
            let css = css_name_for_bind(name);
            return Ok(bind_css_props(spec, &css));
        }
    };
    // Special-case display
    if spec.name == "display" {
        let tok = match &op.arg {
            PaintArg::Token(t) => strip_neg(t).1,
            _ => "none",
        };
        let css = match tok {
            "none" => vec![("display".into(), "none".into())],
            "block" => vec![("display".into(), "block".into())],
            "inline" => vec![("display".into(), "inline".into())],
            "inline-block" => vec![("display".into(), "inline-block".into())],
            "flex" => vec![("display".into(), "flex".into())],
            "inline-flex" => vec![("display".into(), "inline-flex".into())],
            "contents" => vec![("display".into(), "contents".into())],
            _ => {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.into(),
                    source: op.source.clone(),
                })
            }
        };
        return Ok(css);
    }
    // Special-case position bare keywords
    if spec.name == "position" {
        let tok = match &op.arg {
            PaintArg::Token(t) => strip_neg(t).1,
            _ => "relative",
        };
        let ok = matches!(tok, "relative" | "absolute" | "sticky" | "fixed" | "static");
        if !ok {
            return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.into(),
                source: op.source.clone(),
            });
        }
        return Ok(vec![("position".into(), tok.into())]);
    }
    // bg(none) → background-image:none (checkbox restyle clear)
    if spec.name == "bg" {
        if let PaintArg::Token(t) = &op.arg {
            if strip_neg(t).1 == "none" {
                return Ok(vec![("background-image".into(), "none".into())]);
            }
        }
    }
    // outlineOffset special tokens
    if spec.name == "outlineOffset" {
        if let PaintArg::Token(t) = &op.arg {
            match strip_neg(t).1 {
                "ring-offset" => {
                    return Ok(vec![(
                        "outline-offset".into(),
                        "var(--ring-offset)".into(),
                    )]);
                }
                "ring-inset" => {
                    return Ok(vec![("outline-offset".into(), "calc(-1 * var(--ring-width))".into())]);
                }
                _ => {}
            }
        }
    }
    // outline color tokens (primary, tone, ring, tone-ring, …) vs width/style keywords
    if spec.name == "outline" {
        if let PaintArg::Token(t) = &op.arg {
            let bare_t = strip_neg(t).1;
            if bare_t == "ring-width" {
                return Ok(vec![("outline-width".into(), "var(--ring-width)".into())]);
            }
            if !matches!(bare_t, "none" | "solid" | "dashed" | "1" | "2" | "4" | "0" | "3" | "8") {
                // Treat as outline-color via outline-<token>
                let mut d = SxDecl {
                    prefixes: Vec::new(),
                    prop: "outline".into(),
                    value: Some(SxValue::Token(bare_t.into())),
                    negative: false,
                    bare: false,
                    source: op.source.clone(),
                };
                for cand in token_candidates(bare_t) {
                    d.value = Some(SxValue::Token(cand.clone()));
                    let atom = sx_ir::decl_atom(&d);
                    if let Resolved::Ok(pairs) = sx_vocab::resolve(&atom) {
                        return Ok(pairs);
                    }
                }
            }
        }
    }
    // Dimension / composite special cases
    if let PaintArg::Token(t) = &op.arg {
        let bare_t = strip_neg(t).1;
        match (spec.name, bare_t) {
            ("borderWidth", "control-stroke") => {
                return Ok(vec![("border-width".into(), "var(--control-stroke)".into())]);
            }
            ("w" | "h", "border-width") => {
                let prop = if spec.name == "w" { "width" } else { "height" };
                return Ok(vec![(prop.into(), "var(--border-width)".into())]);
            }
            ("w", "toast-region-width") => {
                return Ok(vec![("width".into(), "var(--toast-region-width)".into())]);
            }
            ("z", "toast-region-z") => {
                return Ok(vec![("z-index".into(), "var(--toast-region-z)".into())]);
            }
            ("animate", "spin") => {
                return Ok(vec![("animation".into(), "var(--animate-spin)".into())]);
            }
            ("bgImage", "none") => {
                return Ok(vec![("background-image".into(), "none".into())]);
            }
            ("bgImage", "primary-accent") => {
                return Ok(vec![(
                    "background-image".into(),
                    "linear-gradient(135deg, var(--primary), var(--accent))".into(),
                )]);
            }
            ("avatarMedia", "true" | "on") => {
                return Ok(vec![
                    ("display".into(), "block".into()),
                    ("width".into(), "100%".into()),
                    ("height".into(), "100%".into()),
                    ("object-fit".into(), "contain".into()),
                    ("padding".into(), "10%".into()),
                    ("box-sizing".into(), "border-box".into()),
                ]);
            }
            ("flexDir", dir @ ("row" | "column" | "row-reverse" | "column-reverse")) => {
                return Ok(vec![("flex-direction".into(), dir.into())]);
            }
            _ => {}
        }
    }
    // Bare border width atoms (border, border-b, … without value → 1px solid)
    if matches!(
        spec.name,
        "borderWidth"
            | "borderBWidth"
            | "borderTWidth"
            | "borderEWidth"
            | "borderSWidth"
            | "borderRWidth"
            | "borderLWidth"
    ) {
        if let PaintArg::Token(t) = &op.arg {
            let bare_t = strip_neg(t).1;
            if bare_t == "1" || bare_t.is_empty() {
                // Prefer bare border atom when width is default 1.
                if bare_t == "1" || bare_t.is_empty() {
                    let bare_atom = match spec.name {
                        "borderWidth" => "border",
                        "borderBWidth" => "border-b",
                        "borderTWidth" => "border-t",
                        "borderEWidth" => "border-e",
                        "borderSWidth" => "border-s",
                        "borderRWidth" => "border-r",
                        "borderLWidth" => "border-l",
                        _ => "border",
                    };
                    if bare_t == "1" {
                        // Also try name-1 first for explicit width token
                        let atom = format!("{bare_atom}-1");
                        if let Resolved::Ok(pairs) = sx_vocab::resolve(&atom) {
                            return Ok(pairs);
                        }
                    }
                    if let Resolved::Ok(pairs) = sx_vocab::resolve(bare_atom) {
                        return Ok(pairs);
                    }
                }
            }
        }
    }
    let decl = SxDecl {
        prefixes: Vec::new(),
        prop: spec.vocab.to_string(),
        value,
        negative,
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
            let atom2 = if negative {
                format!("-{}-{}", spec.vocab, cand)
            } else {
                format!("{}-{}", spec.vocab, cand)
            };
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

fn strip_neg(tok: &str) -> (bool, &str) {
    if let Some(rest) = tok.strip_prefix('-') {
        if !rest.is_empty() {
            return (true, rest);
        }
    }
    (false, tok)
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
    out.push(tok.replace('_', "-"));
    out
}

fn css_name_for_bind(name: &str) -> String {
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
    match spec.name {
        "top" => return vec![("top".into(), var)],
        "right" => return vec![("right".into(), var)],
        "bottom" => return vec![("bottom".into(), var)],
        "left" => return vec![("left".into(), var)],
        "inset" => return vec![("inset".into(), var)],
        "ms" => return vec![("margin-inline-start".into(), var)],
        "me" => return vec![("margin-inline-end".into(), var)],
        "mt" => return vec![("margin-top".into(), var)],
        "mb" => return vec![("margin-bottom".into(), var)],
        "w" => return vec![("width".into(), var)],
        "h" => return vec![("height".into(), var)],
        "z" => return vec![("z-index".into(), var)],
        "translateY" => {
            return vec![("translate".into(), format!("var(--sx-translate-x, 0) {var}"))];
        }
        "translateX" => {
            return vec![("translate".into(), format!("{var} var(--sx-translate-y, 0)"))];
        }
        _ => {}
    }
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
    let prop = match spec.name {
        "bg" => "background-color",
        "fg" | "color" => "color",
        "padX" => "padding-inline",
        "padY" => "padding-block",
        "radius" => "border-radius",
        "fontWeight" => "font-weight",
        "fontFamily" => "font-family",
        other => other,
    };
    vec![(prop.into(), var)]
}
