//! Concern setters — the Style setter table after SPEC-SPINNAKER-2 freeze.
//!
//! Named composite or slot object only. Absorbed property names do not resolve.
//! `kind` is a border family key. Slot-merge on `when:` / `match:` is per slot
//! (override emit writes only the named slots; the rest stay).

use std::collections::BTreeMap;

use crate::ir::{PaintArg, PaintOp, StyleDiagnostic};
use crate::scales;
use crate::tokens::{self, CompositeToken};

/// One slot on a concern (`color` on `border`).
#[derive(Debug, Clone, Copy)]
pub struct ConcernSlot {
    pub name: &'static str,
    pub scale: Option<&'static str>,
    pub keywords: &'static [&'static str],
}

/// One concern setter (`border`, `size`, `bg`, `pad`, …).
#[derive(Debug, Clone, Copy)]
pub struct ConcernSpec {
    pub name: &'static str,
    pub slots: &'static [ConcernSlot],
    /// Property setters this concern replaces — compile error, not a warning.
    pub covered_setters: &'static [&'static str],
}

/// Named gradient molecules. Same table on bg / fg / border.
pub const GRADIENT_MOLECULES: &[&str] = &["primary", "accent", "surface", "none"];

/// Markers stay verbs (SPEC-SPINNAKER-2 §4). Not concerns.
pub const MARKERS: &[&str] = &[
    "group",
    "isolate",
    "srOnly",
    "press",
    "controlFrame",
    "controlLift",
    "cursor",
    "select",
    "pointerEvents",
    "tapHighlight",
    "touch",
    "appearance",
    "tab",
    "content",
    "boxSizing",
    "objectFit",
    "objectPosition",
    "scrollbarThumb",
    "scrollbarTrack",
    "scrollbarThin",
    "container",
    "containerType",
    "containerName",
];

/// Closed concern atlas (SPEC-SPINNAKER-2 §3). This IS the setter table.
pub fn concerns() -> &'static [ConcernSpec] {
    &[
        ConcernSpec {
            name: "bg",
            slots: &[
                ConcernSlot {
                    name: "color",
                    scale: Some("color"),
                    keywords: &[
                        "transparent",
                        "current",
                        "currentColor",
                        "inherit",
                        "none",
                        "CanvasText",
                        "Highlight",
                    ],
                },
                ConcernSlot {
                    name: "mix",
                    scale: None,
                    keywords: &[
                        "5", "10", "15", "20", "25", "30", "35", "40", "45", "50", "55", "60",
                        "65", "70", "75", "80", "85", "90", "95",
                    ],
                },
                ConcernSlot {
                    name: "image",
                    scale: None,
                    keywords: &["none", "primary-accent"],
                },
                ConcernSlot {
                    name: "clip",
                    scale: None,
                    keywords: &["border-box", "padding-box", "content-box", "text"],
                },
                ConcernSlot {
                    name: "glow",
                    scale: Some("color"),
                    keywords: &["none", "primary", "accent"],
                },
                ConcernSlot {
                    name: "gradient",
                    scale: None,
                    keywords: GRADIENT_MOLECULES,
                },
            ],
            covered_setters: &[
                "bg",
                "bgImage",
                "bgClip",
                "bgGlow",
                "colorMix",
                "colorDeepen",
            ],
        },
        ConcernSpec {
            name: "fg",
            slots: &[
                ConcernSlot {
                    name: "color",
                    scale: Some("color"),
                    keywords: &[
                        "transparent",
                        "current",
                        "currentColor",
                        "inherit",
                        "CanvasText",
                        "Highlight",
                    ],
                },
                ConcernSlot {
                    name: "caret",
                    scale: Some("color"),
                    keywords: &["auto", "transparent", "current", "currentColor"],
                },
                ConcernSlot {
                    name: "accent",
                    scale: Some("color"),
                    keywords: &["auto", "transparent", "current", "currentColor"],
                },
                ConcernSlot {
                    name: "gradient",
                    scale: None,
                    keywords: GRADIENT_MOLECULES,
                },
            ],
            covered_setters: &["fg", "caret", "accent", "stroke", "textGradient", "color"],
        },
        ConcernSpec {
            name: "border",
            slots: &[
                ConcernSlot {
                    name: "color",
                    scale: Some("color"),
                    keywords: &[
                        "transparent",
                        "current",
                        "currentColor",
                        "CanvasText",
                        "Highlight",
                    ],
                },
                ConcernSlot {
                    name: "width",
                    scale: Some("border-width"),
                    keywords: &[
                        "0",
                        "1",
                        "2",
                        "3",
                        "4",
                        "8",
                        "control-stroke",
                        "hairline",
                        "ring-width",
                    ],
                },
                ConcernSlot {
                    name: "style",
                    scale: None,
                    keywords: &["solid", "dashed", "dotted", "double", "hidden", "none"],
                },
                ConcernSlot {
                    name: "edge",
                    scale: None,
                    keywords: &["all", "x", "y", "t", "b", "l", "r", "s", "e"],
                },
                ConcernSlot {
                    name: "kind",
                    scale: None,
                    keywords: &["stroke", "outline", "ring", "image"],
                },
                ConcernSlot {
                    name: "offset",
                    scale: Some("space"),
                    keywords: &["0", "1", "2", "ring-offset", "ring-inset"],
                },
                ConcernSlot {
                    name: "gradient",
                    scale: None,
                    keywords: GRADIENT_MOLECULES,
                },
            ],
            covered_setters: &[
                "borderWidth",
                "borderStyle",
                "borderX",
                "borderY",
                "borderT",
                "borderB",
                "borderL",
                "borderR",
                "borderS",
                "borderE",
                "borderTWidth",
                "borderBWidth",
                "borderLWidth",
                "borderRWidth",
                "borderSWidth",
                "borderEWidth",
                "outline",
                "outlineOffset",
                "ring",
                "borderImage",
            ],
        },
        ConcernSpec {
            name: "radius",
            slots: &[
                ConcernSlot {
                    name: "all",
                    scale: Some("radius"),
                    keywords: &["none", "full", "sm", "md", "lg", "xl"],
                },
                ConcernSlot {
                    name: "t",
                    scale: Some("radius"),
                    keywords: &["none", "full"],
                },
                ConcernSlot {
                    name: "b",
                    scale: Some("radius"),
                    keywords: &["none", "full"],
                },
                ConcernSlot {
                    name: "l",
                    scale: Some("radius"),
                    keywords: &["none", "full"],
                },
                ConcernSlot {
                    name: "r",
                    scale: Some("radius"),
                    keywords: &["none", "full"],
                },
            ],
            covered_setters: &["radiusT", "radiusB", "radiusL", "radiusR"],
        },
        ConcernSpec {
            name: "elevation",
            slots: &[ConcernSlot {
                name: "shadow",
                scale: Some("shadow"),
                keywords: &["none", "xs", "sm", "md", "lg", "xl"],
            }],
            covered_setters: &["shadow"],
        },
        ConcernSpec {
            name: "filter",
            slots: &[
                ConcernSlot {
                    name: "blur",
                    scale: None,
                    keywords: &["0", "4", "8", "12", "16", "24"],
                },
                ConcernSlot {
                    name: "brightness",
                    scale: None,
                    keywords: &[
                        "0", "50", "75", "90", "95", "100", "105", "110", "125", "150",
                    ],
                },
                ConcernSlot {
                    name: "contrast",
                    scale: None,
                    keywords: &["0", "50", "75", "100", "125", "150", "200"],
                },
                ConcernSlot {
                    name: "grayscale",
                    scale: None,
                    keywords: &["0", "100"],
                },
                ConcernSlot {
                    name: "hue",
                    scale: None,
                    keywords: &["0", "15", "30", "60", "90", "180"],
                },
                ConcernSlot {
                    name: "invert",
                    scale: None,
                    keywords: &["0", "100"],
                },
                ConcernSlot {
                    name: "sepia",
                    scale: None,
                    keywords: &["0", "100"],
                },
                ConcernSlot {
                    name: "saturate",
                    scale: None,
                    keywords: &["0", "50", "60", "65", "100", "150", "200"],
                },
                ConcernSlot {
                    name: "backdrop",
                    scale: None,
                    keywords: &["none", "filter-backdrop"],
                },
                ConcernSlot {
                    name: "blend",
                    scale: None,
                    keywords: &[
                        "normal",
                        "multiply",
                        "screen",
                        "overlay",
                        "darken",
                        "lighten",
                        "difference",
                        "exclusion",
                        "luminosity",
                    ],
                },
                ConcernSlot {
                    name: "opacity",
                    scale: None,
                    keywords: &[
                        "0", "5", "10", "20", "25", "30", "40", "50", "60", "70", "75", "80", "90",
                        "95", "100",
                    ],
                },
            ],
            covered_setters: &[
                "blur",
                "brightness",
                "contrast",
                "grayscale",
                "hueRotate",
                "invert",
                "sepia",
                "saturate",
                "backdrop",
                "mixBlend",
                "opacity",
            ],
        },
        ConcernSpec {
            name: "size",
            slots: &[
                ConcernSlot {
                    name: "w",
                    scale: Some("size"),
                    keywords: &[
                        "0",
                        "auto",
                        "fit",
                        "min",
                        "max",
                        "screen",
                        "border-width",
                        "half",
                    ],
                },
                ConcernSlot {
                    name: "h",
                    scale: Some("size"),
                    keywords: &[
                        "0",
                        "auto",
                        "fit",
                        "min",
                        "max",
                        "screen",
                        "border-width",
                        "half",
                    ],
                },
                ConcernSlot {
                    name: "min-w",
                    scale: Some("size"),
                    keywords: &["0", "screen"],
                },
                ConcernSlot {
                    name: "min-h",
                    scale: Some("size"),
                    keywords: &["0", "screen"],
                },
                ConcernSlot {
                    name: "max-w",
                    scale: Some("size"),
                    keywords: &["0", "screen", "none"],
                },
                ConcernSlot {
                    name: "max-h",
                    scale: Some("size"),
                    keywords: &["0", "screen", "none"],
                },
                ConcernSlot {
                    name: "aspect",
                    scale: None,
                    keywords: &["square", "video", "auto"],
                },
                ConcernSlot {
                    name: "basis",
                    scale: Some("size"),
                    keywords: &["0", "auto"],
                },
            ],
            covered_setters: &["w", "h", "minW", "minH", "maxW", "maxH", "aspect", "basis"],
        },
        ConcernSpec {
            name: "pad",
            slots: &[
                ConcernSlot {
                    name: "all",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "x",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "y",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "t",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "b",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "l",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "r",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "s",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "e",
                    scale: Some("space"),
                    keywords: &["0"],
                },
            ],
            covered_setters: &[
                "padX", "padY", "padT", "padB", "padL", "padR", "padS", "padE",
            ],
        },
        ConcernSpec {
            name: "margin",
            slots: &[
                ConcernSlot {
                    name: "all",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "x",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "y",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "t",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "b",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "l",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "r",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "s",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "e",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
            ],
            covered_setters: &["m", "mx", "my", "mt", "mb", "ml", "mr", "ms", "me"],
        },
        ConcernSpec {
            name: "position",
            slots: &[
                ConcernSlot {
                    name: "scheme",
                    scale: None,
                    keywords: &["relative", "absolute", "sticky", "fixed", "static"],
                },
                ConcernSlot {
                    name: "inset",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "t",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "b",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "l",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "r",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "z",
                    scale: Some("z"),
                    keywords: &["0", "1", "2", "auto"],
                },
                ConcernSlot {
                    name: "order",
                    scale: None,
                    keywords: &["0", "1", "2", "first", "last"],
                },
            ],
            covered_setters: &[
                "position", "inset", "top", "bottom", "left", "right", "z", "order",
            ],
        },
        ConcernSpec {
            name: "overflow",
            slots: &[
                ConcernSlot {
                    name: "all",
                    scale: None,
                    keywords: &["auto", "hidden", "clip", "visible", "scroll"],
                },
                ConcernSlot {
                    name: "x",
                    scale: None,
                    keywords: &["auto", "hidden", "clip", "visible", "scroll"],
                },
                ConcernSlot {
                    name: "y",
                    scale: None,
                    keywords: &["auto", "hidden", "clip", "visible", "scroll"],
                },
                ConcernSlot {
                    name: "overscroll",
                    scale: None,
                    keywords: &["auto", "contain", "none"],
                },
                ConcernSlot {
                    name: "scroll",
                    scale: None,
                    keywords: &["auto", "smooth"],
                },
                ConcernSlot {
                    name: "snap",
                    scale: None,
                    keywords: &["none", "x", "y", "both", "block", "inline"],
                },
            ],
            covered_setters: &["overflowX", "overflowY", "overscroll", "scroll"],
        },
        ConcernSpec {
            name: "type",
            slots: &[
                ConcernSlot {
                    name: "family",
                    scale: Some("font"),
                    keywords: &["body", "heading", "mono", "display"],
                },
                ConcernSlot {
                    name: "size",
                    scale: Some("text"),
                    keywords: &[],
                },
                ConcernSlot {
                    name: "weight",
                    scale: Some("weight"),
                    keywords: &[],
                },
                ConcernSlot {
                    name: "leading",
                    scale: Some("leading"),
                    keywords: &["none"],
                },
                ConcernSlot {
                    name: "tracking",
                    scale: Some("tracking"),
                    keywords: &[],
                },
                ConcernSlot {
                    name: "align",
                    scale: None,
                    keywords: &["left", "center", "right", "justify", "start", "end"],
                },
                ConcernSlot {
                    name: "case",
                    scale: None,
                    keywords: &[
                        "uppercase",
                        "lowercase",
                        "capitalize",
                        "normal",
                        "label-transform",
                    ],
                },
                ConcernSlot {
                    name: "wrap",
                    scale: None,
                    keywords: &[
                        "normal", "nowrap", "truncate", "pretty", "words", "pre-wrap",
                    ],
                },
                ConcernSlot {
                    name: "list",
                    scale: None,
                    keywords: &["none", "disc", "decimal"],
                },
                ConcernSlot {
                    name: "decoration",
                    scale: None,
                    keywords: &["none", "underline", "line-through"],
                },
                ConcernSlot {
                    name: "italic",
                    scale: None,
                    keywords: &["true", "false", "on", "off", "italic", "normal"],
                },
                ConcernSlot {
                    name: "nums",
                    scale: None,
                    keywords: &[
                        "tabular",
                        "lining",
                        "oldstyle",
                        "proportional",
                        "ordinal",
                        "slashed-zero",
                        "normal",
                    ],
                },
                ConcernSlot {
                    name: "break",
                    scale: None,
                    keywords: &["words", "normal", "anywhere"],
                },
            ],
            covered_setters: &[
                "fontFamily",
                "fontSize",
                "fontWeight",
                "leading",
                "tracking",
                "textAlign",
                "textCase",
                "whitespace",
                "break",
                "truncate",
                "italic",
                "nums",
                "decoration",
                "underline",
                "underlineOffset",
                "indent",
                "antialiased",
            ],
        },
    ]
}

pub fn find_concern(name: &str) -> Option<&'static ConcernSpec> {
    let key = crate::vocab::emit_target(name).unwrap_or(name);
    concerns().iter().find(|c| c.name == key)
}

/// Keep the author name. Wrap a bare token onto the concern's primary slot
/// unless it is a named composite.
///
/// `size(token)` is a square: both axes. `size({ w })` / `size({ h })` stay
/// one-axis. Folding a scale step onto `w` only flattened every spinner,
/// icon, thumb, and tone swatch.
pub fn prepare_concern_call(method: &str, args: Vec<PaintArg>) -> (String, Vec<PaintArg>) {
    let Some(spec) = find_concern(method) else {
        return (method.to_string(), args);
    };
    let primary = primary_slot(spec);
    let args = match args_one(args) {
        Some(PaintArg::Token(name)) if resolve_concern_name(spec, &name).is_some() => {
            vec![PaintArg::Token(name)]
        }
        Some(PaintArg::Token(name))
            if spec.name == "size" && matches!(name.as_str(), "hug" | "grow" | "max") =>
        {
            vec![PaintArg::Token(name)]
        }
        Some(PaintArg::Token(name))
            if spec.name == "size"
                && is_identity_name(&name)
                && resolve_concern_name(spec, &name).is_none()
                && !is_named_scale_step(spec, &name) =>
        {
            // Unknown identity (`size(wide)`). Do not fold onto w/h or the
            // diagnostic becomes `size:h(wide)` and the row looks like a
            // scale miss instead of a missing table identity.
            vec![PaintArg::Token(name)]
        }
        Some(PaintArg::Token(name)) if spec.name == "size" => {
            vec![PaintArg::Slots(BTreeMap::from([
                ("w".to_string(), PaintArg::Token(name.clone())),
                ("h".to_string(), PaintArg::Token(name)),
            ]))]
        }
        Some(PaintArg::Token(name)) => vec![PaintArg::Slots(BTreeMap::from([(
            primary.to_string(),
            PaintArg::Token(name),
        )]))],
        Some(PaintArg::Slots(slots)) => vec![PaintArg::Slots(slots)],
        Some(other) => vec![other],
        None => Vec::new(),
    };
    (method.to_string(), args)
}

pub fn primary_slot(spec: &ConcernSpec) -> &'static str {
    match spec.name {
        "bg" | "fg" => "color",
        "pad" | "margin" | "overflow" | "radius" => "all",
        "position" => "scheme",
        _ => spec.slots.first().map(|s| s.name).unwrap_or("all"),
    }
}

fn args_one(args: Vec<PaintArg>) -> Option<PaintArg> {
    if args.len() == 1 {
        args.into_iter().next()
    } else if args.is_empty() {
        None
    } else {
        args.into_iter().next()
    }
}

pub fn is_marker(name: &str) -> bool {
    MARKERS.contains(&name)
}

pub fn find_slot(spec: &ConcernSpec, name: &str) -> Option<&'static ConcernSlot> {
    spec.slots.iter().find(|s| s.name == name)
}

/// Teaching string for a retired property setter. `None` if `name` is live.
pub fn retired_replacement(name: &str) -> Option<(&'static str, String)> {
    // Vocab table is the author SSOT. Live rows are not retired. Unknown
    // names are unknown vocabulary, not a teaching alias.
    if crate::vocab::find_vocab(name).is_some() {
        return None;
    }
    if matches!(name, "gap" | "gapX" | "gapY") {
        return Some(("gap", "<row gap=\"4\"> (layout attr, not Style)".into()));
    }
    if name == "truncate" {
        return Some(("type", "type({ wrap: truncate })".into()));
    }
    if name == "color" {
        return Some(("fg", "fg({ color: … }) or fg(foreground)".into()));
    }
    for spec in concerns() {
        if spec.covered_setters.contains(&name) {
            return Some((spec.name, concern_teaching(spec.name)));
        }
        // Same-name property form of a concern (overflow(hidden), size as longhand)
        // is handled in validate_concern_op, not here.
    }
    None
}

pub fn concern_teaching(concern: &str) -> String {
    match concern {
        "bg" => "bg(surface) or bg({ color: … })".into(),
        "fg" => "fg(foreground) or fg({ color: … })".into(),
        "border" => "border(name) or border({ color, width, style, kind, edge })".into(),
        "radius" => "radius(md) or radius({ all: md })".into(),
        "elevation" => "elevation(sm) or elevation({ shadow: sm })".into(),
        "filter" => "filter({ blur, opacity, … })".into(),
        "size" => {
            "size(control-md) or size({ w, h, min-w, min-h, max-w, max-h, aspect, basis })".into()
        }
        "pad" => "pad(3) or pad({ x: 3, y: 2 })".into(),
        "margin" => "margin(auto) or margin({ x: auto })".into(),
        "position" => "position({ scheme: fixed, inset: 0 })".into(),
        "flow" => "display(flex).dir(column)".into(),
        "overflow" => "overflow({ all: hidden })".into(),
        "type" => "type(body-sm) or type({ family, size, weight, wrap })".into(),
        "motion" => "rotate(…) / scale(…) / translateX(…) / play(…)".into(),
        other => format!("{other}(name) or {other}({{ … }})"),
    }
}

/// RetiredSetter diagnostic for an absorbed property name.
pub fn retired_diag(name: &str, source: &str) -> Option<StyleDiagnostic> {
    let (concern, replacement) = retired_replacement(name)?;
    Some(StyleDiagnostic::RetiredSetter {
        setter: name.to_string(),
        replacement: if concern == "gap" {
            replacement
        } else {
            format!("{replacement} — `{name}` is the retired `{concern}` property")
        },
        source: source.to_string(),
    })
}

/// One skill / MCP surface line for a concern setter (slots, not property scale).
pub fn concern_doc_line(spec: &ConcernSpec) -> String {
    let slots: Vec<&str> = spec.slots.iter().map(|s| s.name).collect();
    let mut line = format!(
        "`style.{}(…)` → concern · slots `{}`",
        spec.name,
        slots.join("`/`")
    );
    if spec.name == "border" {
        line.push_str(" · radius is not a slot · kind is a family key");
    }
    if spec.name == "size" {
        line.push_str(" · pad is not a slot (use pad)");
    }
    line
}

/// Concern-scoped name: `{concern}.{name}` first, then `{name}.{concern}`.
///
/// `border(dialog.border)` is a color poke — it does not resolve. Use `border(dialog)`.
pub fn resolve_concern_name(spec: &ConcernSpec, raw: &str) -> Option<&'static CompositeToken> {
    let concern = spec.name;
    if raw.contains('.') {
        if raw.starts_with(concern) && raw.as_bytes().get(concern.len()) == Some(&b'.') {
            return tokens::find_composite(raw).filter(|c| c.r#type == concern);
        }
        return None;
    }
    let scoped = format!("{concern}.{raw}");
    if let Some(c) = find_typed_composite(concern, &scoped) {
        return Some(c);
    }
    // Registry keys are kebab (`fg-button`); paths are dotted (`fg.button`).
    if let Some(css) = tokens::path_to_css_name(&scoped) {
        if let Some(c) = find_typed_composite(concern, css) {
            return Some(c);
        }
    }
    if let Some(c) = find_typed_composite(concern, &format!("{concern}-{raw}")) {
        return Some(c);
    }
    let component = format!("{raw}.{concern}");
    find_typed_composite(concern, &component)
}

fn find_typed_composite(concern: &str, name: &str) -> Option<&'static CompositeToken> {
    tokens::find_composite(name).filter(|c| c.r#type == concern)
}

/// True when this op is the concern form (composite token, scale-step name, or slot object).
pub fn is_concern_form(op: &PaintOp) -> bool {
    let Some(spec) = find_concern(&op.setter) else {
        return false;
    };
    if op.args.len() != 1 {
        return false;
    }
    match op.first_arg() {
        PaintArg::Slots(_) => true,
        PaintArg::Token(name) => {
            resolve_concern_name(spec, name).is_some() || is_named_scale_step(spec, name)
        }
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => true,
    }
}

/// Family key for a border op. Default `stroke`. Empty for non-border concerns.
pub fn concern_kind(op: &PaintOp) -> &'static str {
    if op.setter != "border" {
        return "";
    }
    match op.first_arg() {
        PaintArg::Slots(slots) => match slots.get("kind") {
            Some(PaintArg::Token(k)) => match k.as_str() {
                "outline" => "outline",
                "ring" => "ring",
                "image" => "image",
                _ => "stroke",
            },
            _ => "stroke",
        },
        _ => "stroke",
    }
}

/// Clash keys: per-`(concern, kind)` and, for stroke borders, per-edge.
/// Different edges coexist (tooltip carets). `edge: all` overlaps every edge.
/// Size keys participation separately from geometry slots (`aspect`/`w`/`h`).
pub fn concern_clash_keys(op: &PaintOp) -> Option<Vec<&'static str>> {
    let spec = find_concern(&op.setter)?;
    if op.args.len() != 1 {
        return None;
    }
    match op.first_arg() {
        PaintArg::Slots(_)
        | PaintArg::Token(_)
        | PaintArg::Bind(_)
        | PaintArg::MemberPath { .. } => {
            if spec.name == "border" {
                Some(border_clash_keys(concern_kind(op), concern_edge(op)))
            } else if spec.name == "position" {
                Some(position_clash_keys(op))
            } else if spec.name == "size" {
                Some(size_clash_keys(op))
            } else {
                Some(vec![clash_key(spec.name, concern_kind(op))])
            }
        }
    }
}

fn concern_edge(op: &PaintOp) -> &'static str {
    match op.first_arg() {
        PaintArg::Slots(slots) => match slots.get("edge") {
            Some(PaintArg::Token(e)) => match e.as_str() {
                "x" => "x",
                "y" => "y",
                "t" => "t",
                "b" => "b",
                "l" => "l",
                "r" => "r",
                "s" => "s",
                "e" => "e",
                _ => "all",
            },
            _ => "all",
        },
        _ => "all",
    }
}

/// Complementary position slots coexist (`scheme`+`z`). Same slot twice clashes.
fn position_clash_keys(op: &PaintOp) -> Vec<&'static str> {
    match op.first_arg() {
        PaintArg::Slots(slots) => {
            let mut keys = Vec::new();
            for name in slots.keys() {
                match name.as_str() {
                    "scheme" => keys.push("position:scheme"),
                    "z" => keys.push("position:z"),
                    "order" => keys.push("position:order"),
                    "inset" => {
                        keys.push("position:t");
                        keys.push("position:b");
                        keys.push("position:l");
                        keys.push("position:r");
                    }
                    "t" => keys.push("position:t"),
                    "b" => keys.push("position:b"),
                    "l" => keys.push("position:l"),
                    "r" => keys.push("position:r"),
                    _ => keys.push("position"),
                }
            }
            if keys.is_empty() {
                vec!["position"]
            } else {
                keys
            }
        }
        // Scalar `position(relative)` is the scheme slot.
        PaintArg::Token(_) | PaintArg::Bind(_) | PaintArg::MemberPath { .. } => {
            vec!["position:scheme"]
        }
    }
}

/// Participation (`grow`/`hug`/`max`) is one write. Geometry slots
/// (`aspect`/`w`/`h`/min/max) are their own. Same slot twice clashes.
fn size_clash_keys(op: &PaintOp) -> Vec<&'static str> {
    match op.first_arg() {
        PaintArg::Slots(slots) => {
            let mut keys = Vec::new();
            for name in slots.keys() {
                keys.push(size_slot_clash_key(name));
            }
            if keys.is_empty() { vec!["size"] } else { keys }
        }
        PaintArg::Token(name) if matches!(name.as_str(), "hug" | "grow" | "max") => {
            vec!["size:participate"]
        }
        PaintArg::Token(name) => {
            if let Some(spec) = find_concern("size") {
                if let Some(c) = resolve_concern_name(spec, name) {
                    let keys: Vec<_> = c
                        .members
                        .iter()
                        .map(|(member, _)| size_slot_clash_key(member))
                        .collect();
                    if !keys.is_empty() {
                        return keys;
                    }
                }
            }
            vec!["size"]
        }
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => vec!["size"],
    }
}

fn size_slot_clash_key(name: &str) -> &'static str {
    match name {
        "w" => "size:w",
        "h" => "size:h",
        "min-w" | "minW" => "size:min-w",
        "min-h" | "minH" => "size:min-h",
        "max-w" | "maxW" => "size:max-w",
        "max-h" | "maxH" => "size:max-h",
        "aspect" => "size:aspect",
        "basis" => "size:basis",
        _ => "size",
    }
}

fn border_clash_keys(kind: &str, edge: &str) -> Vec<&'static str> {
    match kind {
        "outline" => vec!["border:outline"],
        "ring" => vec!["border:ring"],
        "image" => vec!["border:image"],
        _ => match edge {
            // Per-edge keys only — opposite edges coexist (tooltip carets).
            // `all` / `x` / `y` overlap the edges they write.
            "t" => vec!["border:stroke:t"],
            "b" => vec!["border:stroke:b"],
            "l" => vec!["border:stroke:l"],
            "r" => vec!["border:stroke:r"],
            "s" => vec!["border:stroke:s"],
            "e" => vec!["border:stroke:e"],
            "x" => vec![
                "border:stroke:l",
                "border:stroke:r",
                "border:stroke:s",
                "border:stroke:e",
            ],
            "y" => vec!["border:stroke:t", "border:stroke:b"],
            _ => vec![
                "border:stroke:t",
                "border:stroke:b",
                "border:stroke:l",
                "border:stroke:r",
                "border:stroke:s",
                "border:stroke:e",
            ],
        },
    }
}

/// pad and margin both emit through `space`. They are different axes —
fn clash_key(concern: &str, kind: &str) -> &'static str {
    match (concern, kind) {
        ("border", "outline") => "border:outline",
        ("border", "ring") => "border:ring",
        ("border", "image") => "border:image",
        ("border", _) => "border:stroke",
        ("bg", _) => "bg",
        ("fg", _) => "fg",
        ("radius", _) => "radius",
        ("elevation", _) => "elevation",
        ("filter", _) => "filter",
        ("size", _) => "size",
        ("pad", _) => "pad",
        ("margin", _) => "margin",
        ("position", _) => "position",
        ("overflow", _) => "overflow",
        ("type", _) => "type",
        _ => "concern",
    }
}

/// CSS longhands this concern op writes (clash uses [`concern_clash_keys`]).
pub fn concern_op_slots(op: &PaintOp) -> Option<Vec<&'static str>> {
    concern_clash_keys(op)
}

pub fn validate_concern_op(spec: &ConcernSpec, op: &PaintOp) -> Result<(), StyleDiagnostic> {
    if op.args.len() != 1 {
        return Err(StyleDiagnostic::RetiredSetter {
            setter: spec.name.to_string(),
            replacement: concern_teaching(spec.name),
            source: op.source.clone(),
        });
    }
    match op.first_arg() {
        PaintArg::Slots(slots) if is_structured_value(slots) => {
            validate_structured(slots, &op.source)
        }
        PaintArg::Slots(slots) => validate_slot_object(spec, op, slots),
        PaintArg::Token(name) => {
            if crate::ir::raw_inner(name).is_some() {
                return Ok(());
            }
            if resolve_concern_name(spec, name).is_some() {
                if spec.name == "size" {
                    if let Some(c) = resolve_concern_name(spec, name) {
                        reject_size_pad_members(c, op)?;
                    }
                }
                return Ok(());
            }
            if is_named_scale_step(spec, name) {
                return Ok(());
            }
            if name.contains('.') {
                let head = name.split('.').next().unwrap_or(name);
                return Err(StyleDiagnostic::RetiredSetter {
                    setter: format!("{}({name})", spec.name),
                    replacement: format!("{}({})", spec.name, head),
                    source: op.source.clone(),
                });
            }
            if spec.name == "size"
                && is_identity_name(name)
                && !matches!(name.as_str(), "hug" | "grow" | "max")
            {
                return Err(StyleDiagnostic::UnknownVocabulary {
                    name: format!("size.{name}"),
                    source: op.source.clone(),
                });
            }
            let teaching = if spec.name == "border" {
                format!("border({{ color: {name} }}) or border(subtle)")
            } else if spec.name == "bg" {
                format!("bg({{ color: {name} }}) or bg(surface)")
            } else if spec.name == "fg" {
                format!("fg({{ color: {name} }}) or fg(foreground)")
            } else {
                concern_teaching(spec.name)
            };
            Err(StyleDiagnostic::RetiredSetter {
                setter: format!("{}({name})", spec.name),
                replacement: teaching,
                source: op.source.clone(),
            })
        }
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => Ok(()),
    }
}

pub fn reject_size_pad_members(
    composite: &CompositeToken,
    op: &PaintOp,
) -> Result<(), StyleDiagnostic> {
    for (member, _) in composite.members {
        if member.starts_with("pad") || *member == "m" || member.starts_with("m-") {
            return Err(StyleDiagnostic::UnknownConcernSlot {
                setter: "size".into(),
                slot: (*member).to_string(),
                source: op.source.clone(),
            });
        }
    }
    Ok(())
}

/// Bare concern arg that looks like a table identity (`wide`, `page`,
/// `control-md`), not a numeric / scale-step token (`4`, `1_5`).
fn is_identity_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        }
        _ => false,
    }
}

pub fn is_named_scale_step(spec: &ConcernSpec, name: &str) -> bool {
    match spec.name {
        "radius" => {
            let slot = find_slot(spec, "all").unwrap();
            slot.keywords.contains(&name) || slot.scale.is_some_and(|s| scales::contains(s, name))
        }
        "elevation" => {
            let slot = find_slot(spec, "shadow").unwrap();
            slot.keywords.contains(&name) || slot.scale.is_some_and(|s| scales::contains(s, name))
        }
        "size" => {
            if matches!(name, "hug" | "grow" | "max") {
                return true;
            }
            // Participation is hug/grow/max. Control steps write geometry
            // through the catalog composite; otherwise accept size-scale names
            // that are not pad-like. `full` is not a size-slot keyword.
            let slot = find_slot(spec, "w").unwrap();
            slot.keywords.contains(&name) || slot.scale.is_some_and(|s| scales::contains(s, name))
        }
        "type" => {
            // Named type steps are composites (`type.body`). Scale-only names
            // are not a type identity.
            false
        }
        _ => false,
    }
}

pub fn validate_slot_object(
    spec: &ConcernSpec,
    op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<(), StyleDiagnostic> {
    if spec.name == "size" {
        for name in slots.keys() {
            if name.starts_with("pad") || name == "text" || name == "m" {
                return Err(StyleDiagnostic::UnknownConcernSlot {
                    setter: spec.name.to_string(),
                    slot: name.clone(),
                    source: op.source.clone(),
                });
            }
        }
    }
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            return Err(StyleDiagnostic::UnknownConcernSlot {
                setter: spec.name.to_string(),
                slot: name.clone(),
                source: op.source.clone(),
            });
        };
        if let PaintArg::Slots(inner) = value {
            if is_structured_value(inner) {
                validate_structured(inner, &op.source)?;
                continue;
            }
            if slot.name == "gradient" {
                return Err(StyleDiagnostic::BadToken {
                    setter: format!("{}:gradient", spec.name),
                    value: value.display(),
                    source: op.source.clone(),
                });
            }
            return Err(StyleDiagnostic::BadToken {
                setter: spec.name.to_string(),
                value: value.display(),
                source: op.source.clone(),
            });
        }
        validate_slot_value(spec, slot, op, value)?;
    }
    Ok(())
}

pub fn validate_slot_value(
    spec: &ConcernSpec,
    slot: &ConcernSlot,
    op: &PaintOp,
    value: &PaintArg,
) -> Result<(), StyleDiagnostic> {
    match value {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => Ok(()),
        PaintArg::Slots(inner) if is_structured_value(inner) => {
            validate_structured(inner, &op.source)
        }
        PaintArg::Slots(_) if slot.name == "gradient" => Err(StyleDiagnostic::BadToken {
            setter: format!("{}:gradient", spec.name),
            value: value.display(),
            source: op.source.clone(),
        }),
        PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: spec.name.to_string(),
            value: value.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            // `raw(V)` is a marker anyone may spend — skip scale membership.
            if crate::ir::raw_inner(tok).is_some() {
                return Ok(());
            }
            let bare = tok
                .strip_prefix('-')
                .filter(|r| !r.is_empty())
                .unwrap_or(tok);
            // Slot `full` is not a size keyword. Bare `size(full)` stays the
            // flex-trap. Free geometry is `100%`; leftover is grow/max.
            if spec.name == "size" && bare == "full" {
                return Err(StyleDiagnostic::BadToken {
                    setter: format!("size:{}", slot.name),
                    value: "full".into(),
                    source: op.source.clone(),
                });
            }
            if slot.scale == Some("color")
                && crate::component_surfaces::is_component_public_name(bare)
            {
                return Err(StyleDiagnostic::ThemeOwnedToken {
                    name: tok.clone(),
                    source: op.source.clone(),
                });
            }
            if slot.keywords.contains(&bare) {
                return Ok(());
            }
            if spec.name == "size" && is_free_size_value(bare) {
                return Ok(());
            }
            if let Some(scale) = slot.scale {
                if scales::contains(scale, bare) {
                    return Ok(());
                }
            }
            // Literal domains declared on the atlas row (pct int, px int, z int).
            if matches!(slot.name, "opacity") && bare.parse::<u32>().is_ok() {
                return Ok(());
            }
            if matches!(slot.name, "z" | "order") && (bare == "auto" || bare.parse::<i32>().is_ok())
            {
                return Ok(());
            }
            if matches!(slot.name, "width") && bare.parse::<u32>().is_ok() {
                return Ok(());
            }
            Err(StyleDiagnostic::BadToken {
                setter: format!("{}:{slot}", spec.name, slot = slot.name),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Theme `{ref}` / `{op}` object, opened as a Style value.
pub fn is_structured_value(slots: &BTreeMap<String, PaintArg>) -> bool {
    slots.contains_key("ref") || slots.contains_key("op")
}

fn is_structured_ref_target(name: &str) -> bool {
    crate::scales::is_keyword_color(name)
        || name == "current"
        || crate::tokens::registry().contains(name)
        || crate::scales::contains("color", name)
}

pub fn validate_structured(
    slots: &BTreeMap<String, PaintArg>,
    source: &str,
) -> Result<(), StyleDiagnostic> {
    if let Some(r) = slots.get("ref") {
        let name = match r {
            PaintArg::Token(t) => t.as_str(),
            other => {
                return Err(StyleDiagnostic::BadToken {
                    setter: "structured:ref".into(),
                    value: other.display(),
                    source: source.to_string(),
                });
            }
        };
        if !is_structured_ref_target(name) {
            return Err(StyleDiagnostic::BadToken {
                setter: "structured:ref".into(),
                value: name.to_string(),
                source: source.to_string(),
            });
        }
        return Ok(());
    }
    if let Some(op) = slots.get("op") {
        let name = match op {
            PaintArg::Token(t) => t.as_str(),
            other => {
                return Err(StyleDiagnostic::BadToken {
                    setter: "structured:op".into(),
                    value: other.display(),
                    source: source.to_string(),
                });
            }
        };
        if name != "mix" {
            return Err(StyleDiagnostic::BadToken {
                setter: "structured:op".into(),
                value: name.to_string(),
                source: source.to_string(),
            });
        }
        let from = slots.get("from").or_else(|| slots.get("color"));
        let onto = slots.get("onto").or_else(|| slots.get("with"));
        if from.is_none() {
            return Err(StyleDiagnostic::BadToken {
                setter: "structured:mix".into(),
                value: "from".into(),
                source: source.to_string(),
            });
        }
        if onto.is_none() {
            return Err(StyleDiagnostic::BadToken {
                setter: "structured:mix".into(),
                value: "onto".into(),
                source: source.to_string(),
            });
        }
        validate_structured_operand(from.unwrap(), source)?;
        validate_structured_operand(onto.unwrap(), source)?;
        return Ok(());
    }
    Err(StyleDiagnostic::BadToken {
        setter: "structured".into(),
        value: "ref / op".into(),
        source: source.to_string(),
    })
}

fn validate_structured_operand(value: &PaintArg, source: &str) -> Result<(), StyleDiagnostic> {
    match value {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => Ok(()),
        PaintArg::Slots(inner) => validate_structured(inner, source),
        PaintArg::Token(t) => {
            if is_structured_ref_target(t) {
                Ok(())
            } else {
                Err(StyleDiagnostic::BadToken {
                    setter: "structured:ref".into(),
                    value: t.clone(),
                    source: source.to_string(),
                })
            }
        }
    }
}

/// Use-site free geometry after a named size token is evicted (px/rem/%/calc).
pub fn is_free_size_value(raw: &str) -> bool {
    let s = raw.trim();
    let s = s
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .or_else(|| s.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
        .unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    if s.starts_with("min(")
        || s.starts_with("max(")
        || s.starts_with("clamp(")
        || s.starts_with("calc(")
    {
        return s.ends_with(')');
    }
    if let Some(num) = s.strip_suffix('%') {
        return !num.is_empty() && num.parse::<f64>().is_ok();
    }
    for unit in [
        "rem", "px", "em", "ch", "vh", "vw", "dvh", "dvw", "svh", "lvh",
    ] {
        if let Some(num) = s.strip_suffix(unit) {
            return !num.is_empty() && num.parse::<f64>().is_ok();
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use std::collections::HashMap;

    #[test]
    fn atlas_lists_full_concern_set_not_markers() {
        let names: Vec<&str> = concerns().iter().map(|c| c.name).collect();
        for want in [
            "bg",
            "fg",
            "border",
            "radius",
            "elevation",
            "filter",
            "size",
            "pad",
            "margin",
            "position",
            "overflow",
            "type",
        ] {
            assert!(names.contains(&want), "missing concern {want} in {names:?}");
        }
        assert_eq!(names.len(), 12, "{names:?}");
        for m in MARKERS {
            assert!(!names.contains(m), "marker {m} must not be a concern");
        }
    }

    #[test]
    fn unknown_size_identity_is_named_at_the_token() {
        let (_, diags) = parse("style({ size(wide) })");
        assert!(
            diags.iter().any(|d| matches!(
                d,
                StyleDiagnostic::UnknownVocabulary { name, .. } if name == "size.wide"
            )),
            "size(wide) must name the identity, not fold onto size:h: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        assert!(
            !diags.iter().any(|d| d.message().contains("size:h")),
            "must not rewrite the identity into a scale slot: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn size_has_no_pad_or_text_slots() {
        let spec = find_concern("size").unwrap();
        for s in spec.slots {
            assert!(!s.name.starts_with("pad"), "{}", s.name);
            assert_ne!(s.name, "text");
        }
    }

    #[test]
    fn resolve_subtle_in_concern_namespace() {
        let spec = find_concern("border").unwrap();
        let c = resolve_concern_name(spec, "subtle").expect("border.subtle");
        assert_eq!(c.name, "border.subtle");
        assert_eq!(c.r#type, "border");
    }

    #[test]
    fn resolve_default_as_border_composite() {
        let spec = find_concern("border").unwrap();
        let c = resolve_concern_name(spec, "default").expect("border.default");
        assert_eq!(c.name, "border.default");
    }

    #[test]
    fn pack_border_composites_are_gone() {
        let spec = find_concern("border").unwrap();
        assert!(resolve_concern_name(spec, "card").is_none());
        assert!(resolve_concern_name(spec, "dialog").is_none());
    }

    #[test]
    fn qualified_concern_name_resolves() {
        let spec = find_concern("border").unwrap();
        assert!(resolve_concern_name(spec, "border.subtle").is_some());
    }

    #[test]
    fn component_dotted_name_is_color_poke() {
        let spec = find_concern("border").unwrap();
        assert!(
            resolve_concern_name(spec, "dialog.border").is_none(),
            "dialog.border is the deleted color poke"
        );
    }

    #[test]
    fn radius_is_not_a_border_slot() {
        let (_, diags) = parse("style({ border({ radius: md }) })");
        assert!(
            diags.iter().any(|d| {
                matches!(d, StyleDiagnostic::UnknownConcernSlot { slot, .. } if slot == "radius")
                    || d.message().contains("radius")
            }),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn border_primary_is_the_color_slot() {
        let (style, diags) = parse("style({ border(primary) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        assert!(!matches!(style, crate::ir::Style::Empty));
    }

    #[test]
    fn duplicate_slot_in_object_names_the_slot() {
        let (_, diags) = parse("style({ border({ color: muted, color: primary }) })");
        let msg = diags
            .iter()
            .find(|d| matches!(d, StyleDiagnostic::DuplicateSlot { slot, .. } if slot == "color"))
            .map(|d| d.message())
            .unwrap_or_else(|| {
                panic!(
                    "{:?}",
                    diags.iter().map(|d| d.message()).collect::<Vec<_>>()
                )
            });
        assert!(
            msg.contains("style `border` assigns `color` twice"),
            "{msg}"
        );
    }

    #[test]
    fn slot_color_unknown_pack_name_is_bad_token() {
        let (_, diags) = parse("style({ border({ color: dialog-border }) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { value, .. } if value == "dialog-border")
                    || matches!(d, StyleDiagnostic::ThemeOwnedToken { name, .. } if name == "dialog-border")),
            "gone pack names are unknown, not theme-owned: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn same_kind_twice_clashes() {
        let (style, parse_diags) = parse("style({ border(subtle).border({ color: primary }) })");
        assert!(
            parse_diags.is_empty(),
            "{:?}",
            parse_diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let diags = crate::clash::detect_clashes(&style, &HashMap::new());
        let clash = diags
            .iter()
            .find(|d| matches!(d, StyleDiagnostic::Clash { .. }))
            .expect("expected clash");
        let msg = clash.message();
        assert!(msg.contains("border(subtle)"), "{msg}");
        assert!(
            msg.contains("border({ color: primary })") || msg.contains("color: primary"),
            "{msg}"
        );
    }

    #[test]
    fn stroke_plus_ring_do_not_clash() {
        let (style, diags) =
            parse("style({ border(subtle).border({ kind: ring, color: primary }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(clashes.is_empty(), "{clashes:?}");
    }

    #[test]
    fn when_override_does_not_clash() {
        let (style, diags) =
            parse("style({ border(subtle), when: { hover: { border({ color: primary }) } } })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(clashes.is_empty(), "{clashes:?}");
    }

    #[test]
    fn size_and_space_do_not_clash() {
        let (style, diags) = parse("style({ size({ w: '100%' }).pad({ x: 3 }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(clashes.is_empty(), "{clashes:?}");
    }

    #[test]
    fn size_slot_full_is_retired() {
        for src in [
            "style({ size({ w: full }) })",
            "style({ size({ h: full }) })",
            "style({ size({ min-w: full }) })",
            "style({ size({ max-w: full }) })",
            "style({ size({ basis: full }) })",
        ] {
            let (_, diags) = parse(src);
            assert!(
                diags.iter().any(|d| d
                    .message()
                    .contains("size.full retired — use size(grow)|size(max) or free 100%")),
                "{src}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn radius_full_stays() {
        let (_, diags) = parse("style({ radius(full) })");
        assert!(
            diags.is_empty(),
            "radius(full) is a pill: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn fg_button_is_the_named_composite() {
        let (_style, diags) = parse("style({ fg(foreground) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn bg_surface_is_the_named_composite() {
        let (_style, diags) = parse("style({ bg(surface) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn position_scheme_and_z_do_not_clash() {
        let (style, diags) = parse(
            "style({ position({ scheme: absolute, t: 0, b: 0, l: 0, r: 0 }).position({ z: dropdown }) })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(clashes.is_empty(), "{clashes:?}");
    }

    #[test]
    fn position_same_slot_twice_clashes() {
        let (style, parse_diags) =
            parse("style({ position({ z: dropdown }).position({ z: sticky }) })");
        assert!(
            parse_diags.is_empty(),
            "{:?}",
            parse_diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(
            clashes
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "{clashes:?}"
        );
    }

    #[test]
    fn size_grow_and_aspect_do_not_clash() {
        let (style, diags) = parse("style({ size(grow), size({ aspect: square }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(clashes.is_empty(), "{clashes:?}");
    }

    #[test]
    fn size_grow_twice_clashes() {
        // Same token text is an idempotent rewrite. Two participate
        // tokens (`grow` + `hug`) still write the same key.
        let (style, parse_diags) = parse("style({ size(grow), size(hug) })");
        assert!(
            parse_diags.is_empty(),
            "{:?}",
            parse_diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(
            clashes
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "{clashes:?}"
        );
    }

    #[test]
    fn size_aspect_twice_clashes() {
        let (style, parse_diags) =
            parse("style({ size({ aspect: square }), size({ aspect: video }) })");
        assert!(
            parse_diags.is_empty(),
            "{:?}",
            parse_diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(
            clashes
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "{clashes:?}"
        );
    }

    #[test]
    fn pad_and_margin_do_not_clash() {
        let (style, diags) = parse("style({ pad(0).margin(auto) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes = crate::clash::detect_clashes(&style, &HashMap::new());
        assert!(clashes.is_empty(), "{clashes:?}");
    }

    #[test]
    fn concern_doc_line_names_border_slots_not_radius() {
        let spec = find_concern("border").expect("border");
        let line = concern_doc_line(spec);
        assert!(line.contains("`color`"), "{line}");
        assert!(line.contains("radius is not a slot"), "{line}");
        assert!(line.contains("concern"), "{line}");
    }

    #[test]
    fn live_v3_names_are_not_retired() {
        assert!(retired_replacement("bg").is_none());
        assert!(retired_replacement("fg").is_none());
        assert!(retired_replacement("pad").is_none());
        assert!(retired_replacement("display").is_none());
        assert_eq!(retired_replacement("padX").unwrap().0, "pad");
        assert_eq!(retired_replacement("w").unwrap().0, "size");
        assert_eq!(retired_replacement("shadow").unwrap().0, "elevation");
        assert_eq!(retired_replacement("gap").unwrap().0, "gap");
    }

    #[test]
    fn size_bare_token_is_a_square() {
        let (name, args) =
            prepare_concern_call("size", vec![PaintArg::Token("icon-size-md".into())]);
        assert_eq!(name, "size");
        match &args[0] {
            PaintArg::Slots(slots) => {
                assert_eq!(
                    slots.get("w"),
                    Some(&PaintArg::Token("icon-size-md".into()))
                );
                assert_eq!(
                    slots.get("h"),
                    Some(&PaintArg::Token("icon-size-md".into()))
                );
            }
            other => panic!("expected w+h slots, got {other:?}"),
        }
    }

    #[test]
    fn alias_bg_surface_stays_named_or_color() {
        let (name, args) = prepare_concern_call("bg", vec![PaintArg::Token("surface".into())]);
        assert_eq!(name, "bg");
        assert!(
            args == vec![PaintArg::Token("surface".into())]
                || matches!(
                    &args[0],
                    PaintArg::Slots(slots) if slots.get("color") == Some(&PaintArg::Token("surface".into()))
                ),
            "{args:?}"
        );
    }

    #[test]
    fn alias_fg_color_token_wraps_color_slot() {
        let (name, args) =
            prepare_concern_call("fg", vec![PaintArg::Token("muted-foreground".into())]);
        assert_eq!(name, "fg");
        match &args[0] {
            PaintArg::Slots(slots) => {
                assert_eq!(
                    slots.get("color"),
                    Some(&PaintArg::Token("muted-foreground".into()))
                );
            }
            other => panic!("expected color slot, got {other:?}"),
        }
    }

    #[test]
    fn pad_axis_object_keeps_author_slots() {
        let slots = BTreeMap::from([
            ("x".into(), PaintArg::Token("5".into())),
            ("y".into(), PaintArg::Token("4".into())),
        ]);
        let (name, args) = prepare_concern_call("pad", vec![PaintArg::Slots(slots)]);
        assert_eq!(name, "pad");
        match &args[0] {
            PaintArg::Slots(slots) => {
                assert_eq!(slots.get("x"), Some(&PaintArg::Token("5".into())));
                assert_eq!(slots.get("y"), Some(&PaintArg::Token("4".into())));
            }
            other => panic!("expected pad slots, got {other:?}"),
        }
    }

    #[test]
    fn margin_off_scale_names_space_steps() {
        let (_, diags) = parse("style({ margin({ t: 3.5 }) })");
        let scale: Vec<String> = diags
            .iter()
            .map(|d| d.message())
            .filter(|m| m.contains("3.5 is not a space step"))
            .collect();
        assert_eq!(scale.len(), 1, "{scale:?}");
        assert!(
            scale[0].contains(
                "3.5 is not a space step: did you mean 3 | 4 — or is this an exception: raw(3.5)?"
            ),
            "{scale:?}"
        );
    }

    #[test]
    fn pad_off_scale_names_space_steps() {
        let (_, diags) = parse("style({ pad({ x: 3.5 }) })");
        let scale: Vec<String> = diags
            .iter()
            .map(|d| d.message())
            .filter(|m| m.contains("3.5 is not a space step"))
            .collect();
        assert_eq!(scale.len(), 1, "{scale:?}");
        assert!(
            scale[0].contains(
                "3.5 is not a space step: did you mean 3 | 4 — or is this an exception: raw(3.5)?"
            ),
            "{scale:?}"
        );
        assert!(
            !scale[0].contains("3.5 is not a space step: 3 | 4;"),
            "scale sentence must appear once: {scale:?}"
        );
    }

    #[test]
    fn pad_raw_compiles_silently_and_counts() {
        let (style, diags) = parse("style({ pad({ x: raw(3.5) }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        assert_eq!(style.count_raw_uses(), 1);
        assert_eq!(crate::count_raw_uses_in_source("pad({ x: raw(3.5) })"), 1);
    }

    #[test]
    fn pad_raw_two_uses_count_two() {
        let (style, diags) = parse("style({ pad({ x: raw(3.5), y: raw(2.5) }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        assert_eq!(style.count_raw_uses(), 2);
        assert_eq!(
            crate::count_raw_uses_in_source("pad({ x: raw(3.5), y: raw(2.5) })"),
            2
        );
    }

    #[test]
    fn pad_unwrapped_off_scale_still_errors() {
        let (_, diags) = parse("style({ pad({ x: 3.5 }) })");
        assert!(
            diags
                .iter()
                .any(|d| d.message().contains("3.5 is not a space step")),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn pad_on_scale_slots_are_silent() {
        let (_, diags) = parse("style({ pad({ x: 4 }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }
}
