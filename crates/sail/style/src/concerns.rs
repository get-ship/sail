//! Concern setters — the Style setter table after SPEC-SPINNAKER-2 freeze.
//!
//! Named composite or slot object only. Absorbed property names do not resolve.
//! `kind` is a border family key. Slot-merge on `when:` / `match:` is per slot
//! (override emit writes only the named slots; CSS cascade keeps the rest).

use crate::channel::Channel;
use crate::ir::{PaintArg, PaintOp, StyleDiagnostic};
use crate::scales;
use crate::tokens::{self, CompositeToken};

/// One slot on a concern (`color` on `border`).
#[derive(Debug, Clone, Copy)]
pub struct ConcernSlot {
    pub name: &'static str,
    /// CSS longhand this slot writes for the default family (stroke / primary).
    pub css_prop: &'static str,
    pub scale: Option<&'static str>,
    pub keywords: &'static [&'static str],
}

/// One concern setter (`border`, `size`, `space`, `fill`, …).
#[derive(Debug, Clone, Copy)]
pub struct ConcernSpec {
    pub name: &'static str,
    pub slots: &'static [ConcernSlot],
    /// Property setters this concern replaces — compile error, not a warning.
    pub covered_setters: &'static [&'static str],
    pub channel: Channel,
}

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
    "avatarMedia",
    "glyphFill",
    "container",
    "containerType",
    "containerName",
];

/// Closed concern atlas (SPEC-SPINNAKER-2 §3). This IS the setter table.
pub fn concerns() -> &'static [ConcernSpec] {
    &[
        ConcernSpec {
            name: "fill",
            slots: &[
                ConcernSlot {
                    name: "color",
                    css_prop: "background-color",
                    scale: Some("color"),
                    keywords: &["transparent", "current", "currentColor", "inherit", "none"],
                },
                ConcernSlot {
                    name: "image",
                    css_prop: "background-image",
                    scale: None,
                    keywords: &["none"],
                },
                ConcernSlot {
                    name: "clip",
                    css_prop: "background-clip",
                    scale: None,
                    keywords: &["border-box", "padding-box", "content-box", "text"],
                },
                ConcernSlot {
                    name: "glow",
                    css_prop: "box-shadow",
                    scale: Some("color"),
                    keywords: &["none", "primary", "accent"],
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
            channel: Channel::Paint,
        },
        ConcernSpec {
            name: "ink",
            slots: &[
                ConcernSlot {
                    name: "color",
                    css_prop: "color",
                    scale: Some("color"),
                    keywords: &["transparent", "current", "currentColor", "inherit"],
                },
                ConcernSlot {
                    name: "caret",
                    css_prop: "caret-color",
                    scale: Some("color"),
                    keywords: &["auto", "transparent", "current", "currentColor"],
                },
                ConcernSlot {
                    name: "accent",
                    css_prop: "accent-color",
                    scale: Some("color"),
                    keywords: &["auto", "transparent", "current", "currentColor"],
                },
            ],
            covered_setters: &["fg", "caret", "accent", "stroke", "textGradient", "color"],
            channel: Channel::Paint,
        },
        ConcernSpec {
            name: "border",
            slots: &[
                ConcernSlot {
                    name: "color",
                    css_prop: "border-color",
                    scale: Some("color"),
                    keywords: &["transparent", "current", "currentColor"],
                },
                ConcernSlot {
                    name: "width",
                    css_prop: "border-width",
                    scale: Some("border-width"),
                    keywords: &["0", "1", "2", "3", "4", "8", "control-stroke", "hairline"],
                },
                ConcernSlot {
                    name: "style",
                    css_prop: "border-style",
                    scale: None,
                    keywords: &["solid", "dashed", "dotted", "double", "hidden", "none"],
                },
                ConcernSlot {
                    name: "edge",
                    css_prop: "",
                    scale: None,
                    keywords: &["all", "x", "y", "t", "b", "l", "r", "s", "e"],
                },
                ConcernSlot {
                    name: "kind",
                    css_prop: "",
                    scale: None,
                    keywords: &["stroke", "outline", "ring", "image"],
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
            channel: Channel::Paint,
        },
        ConcernSpec {
            name: "radius",
            slots: &[
                ConcernSlot {
                    name: "all",
                    css_prop: "border-radius",
                    scale: Some("radius"),
                    keywords: &["none", "full", "sm", "md", "lg", "xl"],
                },
                ConcernSlot {
                    name: "t",
                    css_prop: "border-top-left-radius",
                    scale: Some("radius"),
                    keywords: &["none", "full"],
                },
                ConcernSlot {
                    name: "b",
                    css_prop: "border-bottom-left-radius",
                    scale: Some("radius"),
                    keywords: &["none", "full"],
                },
                ConcernSlot {
                    name: "l",
                    css_prop: "border-top-left-radius",
                    scale: Some("radius"),
                    keywords: &["none", "full"],
                },
                ConcernSlot {
                    name: "r",
                    css_prop: "border-top-right-radius",
                    scale: Some("radius"),
                    keywords: &["none", "full"],
                },
            ],
            covered_setters: &["radiusT", "radiusB", "radiusL", "radiusR"],
            channel: Channel::Paint,
        },
        ConcernSpec {
            name: "elevation",
            slots: &[ConcernSlot {
                name: "shadow",
                css_prop: "box-shadow",
                scale: Some("shadow"),
                keywords: &["none", "xs", "sm", "md", "lg", "xl"],
            }],
            covered_setters: &["shadow"],
            channel: Channel::Paint,
        },
        ConcernSpec {
            name: "filter",
            slots: &[
                ConcernSlot {
                    name: "blur",
                    css_prop: "filter",
                    scale: None,
                    keywords: &["0", "4", "8", "12", "16", "24"],
                },
                ConcernSlot {
                    name: "brightness",
                    css_prop: "filter",
                    scale: None,
                    keywords: &["0", "50", "75", "90", "95", "100", "105", "110", "125", "150"],
                },
                ConcernSlot {
                    name: "contrast",
                    css_prop: "filter",
                    scale: None,
                    keywords: &["0", "50", "75", "100", "125", "150", "200"],
                },
                ConcernSlot {
                    name: "grayscale",
                    css_prop: "filter",
                    scale: None,
                    keywords: &["0", "100"],
                },
                ConcernSlot {
                    name: "hue",
                    css_prop: "filter",
                    scale: None,
                    keywords: &["0", "15", "30", "60", "90", "180"],
                },
                ConcernSlot {
                    name: "invert",
                    css_prop: "filter",
                    scale: None,
                    keywords: &["0", "100"],
                },
                ConcernSlot {
                    name: "sepia",
                    css_prop: "filter",
                    scale: None,
                    keywords: &["0", "100"],
                },
                ConcernSlot {
                    name: "saturate",
                    css_prop: "filter",
                    scale: None,
                    keywords: &["0", "50", "100", "150", "200"],
                },
                ConcernSlot {
                    name: "backdrop",
                    css_prop: "backdrop-filter",
                    scale: None,
                    keywords: &["none", "card-backdrop-filter"],
                },
                ConcernSlot {
                    name: "blend",
                    css_prop: "mix-blend-mode",
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
                    css_prop: "opacity",
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
            channel: Channel::Paint,
        },
        ConcernSpec {
            name: "size",
            slots: &[
                ConcernSlot {
                    name: "w",
                    css_prop: "width",
                    scale: Some("size"),
                    keywords: &["0", "full", "auto", "fit", "min", "max", "screen"],
                },
                ConcernSlot {
                    name: "h",
                    css_prop: "height",
                    scale: Some("size"),
                    keywords: &["0", "full", "auto", "fit", "min", "max", "screen"],
                },
                ConcernSlot {
                    name: "min-w",
                    css_prop: "min-width",
                    scale: Some("size"),
                    keywords: &["0", "full", "screen"],
                },
                ConcernSlot {
                    name: "min-h",
                    css_prop: "min-height",
                    scale: Some("size"),
                    keywords: &["0", "full", "screen"],
                },
                ConcernSlot {
                    name: "max-w",
                    css_prop: "max-width",
                    scale: Some("size"),
                    keywords: &["0", "full", "screen", "none"],
                },
                ConcernSlot {
                    name: "max-h",
                    css_prop: "max-height",
                    scale: Some("size"),
                    keywords: &["0", "full", "screen", "none"],
                },
                ConcernSlot {
                    name: "aspect",
                    css_prop: "aspect-ratio",
                    scale: None,
                    keywords: &["square", "video", "auto"],
                },
            ],
            covered_setters: &["w", "h", "minW", "minH", "maxW", "maxH", "aspect", "basis"],
            channel: Channel::Box,
        },
        ConcernSpec {
            name: "space",
            slots: &[
                ConcernSlot {
                    name: "pad",
                    css_prop: "padding",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "pad-x",
                    css_prop: "padding-inline",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "pad-y",
                    css_prop: "padding-block",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "pad-t",
                    css_prop: "padding-top",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "pad-b",
                    css_prop: "padding-bottom",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "pad-l",
                    css_prop: "padding-left",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "pad-r",
                    css_prop: "padding-right",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "m",
                    css_prop: "margin",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "mx",
                    css_prop: "margin-inline",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "my",
                    css_prop: "margin-block",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "mt",
                    css_prop: "margin-top",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "mb",
                    css_prop: "margin-bottom",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "ml",
                    css_prop: "margin-left",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "mr",
                    css_prop: "margin-right",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "ms",
                    css_prop: "margin-inline-start",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "me",
                    css_prop: "margin-inline-end",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
            ],
            covered_setters: &[
                "pad", "padX", "padY", "padT", "padB", "padL", "padR", "padS", "padE", "m", "mx",
                "my", "mt", "mb", "ml", "mr", "ms", "me",
            ],
            channel: Channel::Box,
        },
        ConcernSpec {
            name: "place",
            slots: &[
                ConcernSlot {
                    name: "position",
                    css_prop: "position",
                    scale: None,
                    keywords: &["relative", "absolute", "sticky", "fixed", "static"],
                },
                ConcernSlot {
                    name: "inset",
                    css_prop: "inset",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "top",
                    css_prop: "top",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "bottom",
                    css_prop: "bottom",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "left",
                    css_prop: "left",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "right",
                    css_prop: "right",
                    scale: Some("space"),
                    keywords: &["0", "auto"],
                },
                ConcernSlot {
                    name: "z",
                    css_prop: "z-index",
                    scale: Some("z"),
                    keywords: &["0", "1", "2", "auto"],
                },
                ConcernSlot {
                    name: "order",
                    css_prop: "order",
                    scale: None,
                    keywords: &["0", "1", "2", "first", "last"],
                },
            ],
            covered_setters: &[
                "position", "inset", "top", "bottom", "left", "right", "z", "order",
            ],
            channel: Channel::Box,
        },
        ConcernSpec {
            name: "flow",
            slots: &[
                ConcernSlot {
                    name: "display",
                    css_prop: "display",
                    scale: None,
                    keywords: &[
                        "none",
                        "block",
                        "inline",
                        "inline-block",
                        "flex",
                        "inline-flex",
                        "contents",
                    ],
                },
                ConcernSlot {
                    name: "dir",
                    css_prop: "flex-direction",
                    scale: None,
                    keywords: &["row", "column", "row-reverse", "column-reverse"],
                },
                ConcernSlot {
                    name: "grow",
                    css_prop: "flex-grow",
                    scale: None,
                    keywords: &["0", "1"],
                },
                ConcernSlot {
                    name: "shrink",
                    css_prop: "flex-shrink",
                    scale: None,
                    keywords: &["0", "1"],
                },
                ConcernSlot {
                    name: "self",
                    css_prop: "align-self",
                    scale: None,
                    keywords: &["auto", "start", "end", "center", "stretch", "baseline"],
                },
            ],
            covered_setters: &["display", "flexDir", "grow", "shrink", "self", "justifySelf"],
            channel: Channel::Box,
        },
        ConcernSpec {
            name: "overflow",
            slots: &[
                ConcernSlot {
                    name: "all",
                    css_prop: "overflow",
                    scale: None,
                    keywords: &["auto", "hidden", "clip", "visible", "scroll"],
                },
                ConcernSlot {
                    name: "x",
                    css_prop: "overflow-x",
                    scale: None,
                    keywords: &["auto", "hidden", "clip", "visible", "scroll"],
                },
                ConcernSlot {
                    name: "y",
                    css_prop: "overflow-y",
                    scale: None,
                    keywords: &["auto", "hidden", "clip", "visible", "scroll"],
                },
                ConcernSlot {
                    name: "overscroll",
                    css_prop: "overscroll-behavior",
                    scale: None,
                    keywords: &["auto", "contain", "none"],
                },
                ConcernSlot {
                    name: "scroll",
                    css_prop: "scroll-behavior",
                    scale: None,
                    keywords: &["auto", "smooth"],
                },
            ],
            covered_setters: &[
                "overflowX",
                "overflowY",
                "overscroll",
                "scroll",
                "resize",
                "contain",
                "willChange",
                "visibility",
            ],
            channel: Channel::Box,
        },
        ConcernSpec {
            name: "type",
            slots: &[
                ConcernSlot {
                    name: "family",
                    css_prop: "font-family",
                    scale: Some("font"),
                    keywords: &["body", "heading", "mono", "display"],
                },
                ConcernSlot {
                    name: "size",
                    css_prop: "font-size",
                    scale: Some("text"),
                    keywords: &[],
                },
                ConcernSlot {
                    name: "weight",
                    css_prop: "font-weight",
                    scale: Some("weight"),
                    keywords: &[],
                },
                ConcernSlot {
                    name: "leading",
                    css_prop: "line-height",
                    scale: Some("leading"),
                    keywords: &["none"],
                },
                ConcernSlot {
                    name: "tracking",
                    css_prop: "letter-spacing",
                    scale: Some("tracking"),
                    keywords: &[],
                },
                ConcernSlot {
                    name: "align",
                    css_prop: "text-align",
                    scale: None,
                    keywords: &["left", "center", "right", "justify", "start", "end"],
                },
                ConcernSlot {
                    name: "case",
                    css_prop: "text-transform",
                    scale: None,
                    keywords: &["uppercase", "lowercase", "capitalize", "normal", "label-transform"],
                },
                ConcernSlot {
                    name: "wrap",
                    css_prop: "white-space",
                    scale: None,
                    keywords: &["normal", "nowrap", "truncate", "pretty"],
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
                "list",
            ],
            channel: Channel::Text,
        },
        ConcernSpec {
            name: "motion",
            slots: &[
                ConcernSlot {
                    name: "rotate",
                    css_prop: "rotate",
                    scale: None,
                    keywords: &["0", "1", "2", "3", "6", "12", "45", "90", "180"],
                },
                ConcernSlot {
                    name: "scale",
                    css_prop: "scale",
                    scale: None,
                    keywords: &["0", "50", "75", "90", "95", "100", "105", "110", "125"],
                },
                ConcernSlot {
                    name: "x",
                    css_prop: "translate",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "y",
                    css_prop: "translate",
                    scale: Some("space"),
                    keywords: &["0"],
                },
                ConcernSlot {
                    name: "origin",
                    css_prop: "transform-origin",
                    scale: None,
                    keywords: &[
                        "center", "top", "bottom", "left", "right", "top-left", "top-right",
                        "bottom-left", "bottom-right",
                    ],
                },
                ConcernSlot {
                    name: "ease",
                    css_prop: "transition-timing-function",
                    scale: Some("ease"),
                    keywords: &["linear", "in", "out", "in-out"],
                },
                ConcernSlot {
                    name: "duration",
                    css_prop: "transition-duration",
                    scale: Some("duration"),
                    keywords: &["0", "75", "100", "150", "200", "300", "500", "700", "1000"],
                },
            ],
            covered_setters: &[
                "rotate",
                "rotateX",
                "rotateY",
                "rotateZ",
                "scale",
                "translateX",
                "translateY",
                "transform",
                "origin",
                "perspective",
                "transition",
                "duration",
                "ease",
                "animate",
            ],
            channel: Channel::Paint,
        },
    ]
}

pub fn find_concern(name: &str) -> Option<&'static ConcernSpec> {
    concerns().iter().find(|c| c.name == name)
}

pub fn is_marker(name: &str) -> bool {
    MARKERS.contains(&name)
}

pub fn find_slot(spec: &ConcernSpec, name: &str) -> Option<&'static ConcernSlot> {
    spec.slots.iter().find(|s| s.name == name)
}

/// Teaching string for a retired property setter. `None` if `name` is live.
pub fn retired_replacement(name: &str) -> Option<(&'static str, String)> {
    if matches!(name, "gap" | "gapX" | "gapY") {
        return Some((
            "gap",
            "<row gap=\"4\"> (layout attr, not Style)".into(),
        ));
    }
    if name == "truncate" {
        return Some(("type", "type({ wrap: truncate })".into()));
    }
    if name == "color" {
        return Some(("ink", "ink({ color: … }) or ink(foreground)".into()));
    }
    for spec in concerns() {
        if spec.covered_setters.iter().any(|s| *s == name) {
            return Some((spec.name, concern_teaching(spec.name)));
        }
        // Same-name property form of a concern (overflow(hidden), size as longhand)
        // is handled in validate_concern_op, not here.
    }
    None
}

fn concern_teaching(concern: &str) -> String {
    match concern {
        "fill" => "fill(surface) or fill({ color: … })".into(),
        "ink" => "ink(foreground) or ink({ color: … })".into(),
        "border" => "border(name) or border({ color, width, style, kind, edge })".into(),
        "radius" => "radius(md) or radius({ all: md })".into(),
        "elevation" => "elevation(sm) or elevation({ shadow: sm })".into(),
        "filter" => "filter({ blur, opacity, … })".into(),
        "size" => "size(control-md) or size({ w, h, min-w, min-h, max-w, max-h, aspect })".into(),
        "space" => "space({ pad-x: 3, pad-y: 2 })".into(),
        "place" => "place({ position: fixed, inset: 0 })".into(),
        "flow" => "flow({ display: flex, dir: column })".into(),
        "overflow" => "overflow({ all: hidden })".into(),
        "type" => "type(body-sm) or type({ family, size, weight, wrap })".into(),
        "motion" => "motion({ rotate, scale, x, y, origin, ease, duration })".into(),
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
        line.push_str(" · pad is not a slot (use space)");
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
    if let Some(c) = tokens::find_composite(&scoped).filter(|c| c.r#type == concern) {
        return Some(c);
    }
    let component = format!("{raw}.{concern}");
    tokens::find_composite(&component).filter(|c| c.r#type == concern)
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

/// Clash keys: per-`(concern, kind)`. Two ring calls in one scope share `border:ring`.
pub fn concern_clash_keys(op: &PaintOp) -> Option<Vec<&'static str>> {
    let spec = find_concern(&op.setter)?;
    if op.args.len() != 1 {
        return None;
    }
    match op.first_arg() {
        PaintArg::Slots(_) | PaintArg::Token(_) | PaintArg::Bind(_) | PaintArg::MemberPath { .. } => {
            Some(vec![clash_key(spec.name, concern_kind(op))])
        }
    }
}

fn clash_key(concern: &str, kind: &str) -> &'static str {
    match (concern, kind) {
        ("border", "outline") => "border:outline",
        ("border", "ring") => "border:ring",
        ("border", "image") => "border:image",
        ("border", _) => "border:stroke",
        ("fill", _) => "fill",
        ("ink", _) => "ink",
        ("radius", _) => "radius",
        ("elevation", _) => "elevation",
        ("filter", _) => "filter",
        ("size", _) => "size",
        ("space", _) => "space",
        ("place", _) => "place",
        ("flow", _) => "flow",
        ("overflow", _) => "overflow",
        ("type", _) => "type",
        ("motion", _) => "motion",
        _ => "concern",
    }
}

/// CSS longhands this concern op writes (legacy helper; clash uses [`concern_clash_keys`]).
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
        PaintArg::Slots(slots) => validate_slot_object(spec, op, slots),
        PaintArg::Token(name) => {
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
            let teaching = if spec.name == "border" {
                format!("border({{ color: {name} }}) or border(subtle)")
            } else if spec.name == "fill" {
                format!("fill({{ color: {name} }}) or fill(surface)")
            } else if spec.name == "ink" {
                format!("ink({{ color: {name} }}) or ink(foreground)")
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

fn reject_size_pad_members(composite: &CompositeToken, op: &PaintOp) -> Result<(), StyleDiagnostic> {
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

fn is_named_scale_step(spec: &ConcernSpec, name: &str) -> bool {
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
            // `full` / control steps write geometry. `control-md` resolves as
            // composite when the catalog has it; otherwise accept size-scale names
            // that are not pad-like.
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
        if matches!(value, PaintArg::Slots(_)) {
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
        PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: spec.name.to_string(),
            value: value.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let bare = tok
                .strip_prefix('-')
                .filter(|r| !r.is_empty())
                .unwrap_or(tok);
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

pub fn lower_concern_op(spec: &ConcernSpec, op: &PaintOp) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    if op.args.len() != 1 {
        return Err(StyleDiagnostic::RetiredSetter {
            setter: spec.name.to_string(),
            replacement: concern_teaching(spec.name),
            source: op.source.clone(),
        });
    }
    match op.first_arg() {
        PaintArg::Slots(slots) => lower_slot_object(spec, op, slots),
        PaintArg::Token(name) => {
            if let Some(composite) = resolve_concern_name(spec, name) {
                if spec.name == "size" {
                    reject_size_pad_members(composite, op)?;
                }
                return Ok(lower_composite(spec, composite));
            }
            if is_named_scale_step(spec, name) {
                return lower_named_scale_step(spec, name);
            }
            Err(StyleDiagnostic::RetiredSetter {
                setter: format!("{}({name})", spec.name),
                replacement: concern_teaching(spec.name),
                source: op.source.clone(),
            })
        }
        PaintArg::Bind(name) => {
            let css = format!("var(--{})", css_name_for_bind(name));
            Ok(primary_decls(spec, css))
        }
        PaintArg::MemberPath { root, path } => {
            let css = crate::token_groups::bridge_var_ref(root, path);
            Ok(primary_decls(spec, css))
        }
    }
}

fn primary_decls(spec: &ConcernSpec, css: String) -> Vec<(String, String)> {
    match spec.name {
        "fill" => vec![("background-color".into(), css)],
        "ink" => vec![("color".into(), css)],
        "border" => vec![("border-color".into(), css)],
        "radius" => vec![("border-radius".into(), css)],
        "elevation" => vec![("box-shadow".into(), css)],
        "size" => vec![("width".into(), css.clone()), ("height".into(), css)],
        _ => spec
            .slots
            .iter()
            .find(|s| !s.css_prop.is_empty())
            .map(|s| vec![(s.css_prop.to_string(), css)])
            .unwrap_or_default(),
    }
}

fn lower_named_scale_step(spec: &ConcernSpec, name: &str) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    match spec.name {
        "radius" => {
            let slot = find_slot(spec, "all").unwrap();
            Ok(vec![lower_slot_value(slot, &PaintArg::Token(name.into()))?])
        }
        "elevation" => {
            let slot = find_slot(spec, "shadow").unwrap();
            Ok(vec![lower_slot_value(slot, &PaintArg::Token(name.into()))?])
        }
        "size" => {
            let w = find_slot(spec, "w").unwrap();
            let h = find_slot(spec, "h").unwrap();
            let tok = PaintArg::Token(name.into());
            Ok(vec![
                lower_slot_value(w, &tok)?,
                lower_slot_value(h, &tok)?,
            ])
        }
        _ => Ok(Vec::new()),
    }
}

/// Lower a resolved composite to the concern's CSS longhands (member vars).
pub fn lower_composite(spec: &ConcernSpec, composite: &CompositeToken) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for slot in spec.slots {
        if slot.css_prop.is_empty() {
            continue;
        }
        if let Some((_, css)) = composite
            .members
            .iter()
            .find(|(member, _)| *member == slot.name)
        {
            if spec.name == "radius" && slot.name == "all" {
                out.push((slot.css_prop.to_string(), format!("var(--{css})")));
            } else if spec.name == "radius" && matches!(slot.name, "t" | "b" | "l" | "r") {
                for prop in radius_side_props(slot.name) {
                    out.push((prop.to_string(), format!("var(--{css})")));
                }
            } else {
                out.push((slot.css_prop.to_string(), format!("var(--{css})")));
            }
        }
    }
    out
}

fn lower_slot_object(
    spec: &ConcernSpec,
    op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    if spec.name == "border" {
        return lower_border_slots(spec, op, slots);
    }
    if spec.name == "filter" {
        return lower_filter_slots(spec, op, slots);
    }
    if spec.name == "type" {
        return lower_type_slots(spec, op, slots);
    }
    if spec.name == "motion" {
        return lower_motion_slots(spec, op, slots);
    }
    if spec.name == "radius" {
        return lower_radius_slots(spec, op, slots);
    }
    let mut out = Vec::new();
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            return Err(StyleDiagnostic::UnknownConcernSlot {
                setter: spec.name.to_string(),
                slot: name.clone(),
                source: op.source.clone(),
            });
        };
        if slot.css_prop.is_empty() {
            continue;
        }
        out.push(lower_slot_value(slot, value)?);
    }
    Ok(out)
}

fn lower_radius_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut out = Vec::new();
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        let (_, css) = lower_slot_value(slot, value)?;
        match name.as_str() {
            "all" => out.push(("border-radius".into(), css)),
            "t" => {
                out.push(("border-top-left-radius".into(), css.clone()));
                out.push(("border-top-right-radius".into(), css));
            }
            "b" => {
                out.push(("border-bottom-left-radius".into(), css.clone()));
                out.push(("border-bottom-right-radius".into(), css));
            }
            "l" => {
                out.push(("border-top-left-radius".into(), css.clone()));
                out.push(("border-bottom-left-radius".into(), css));
            }
            "r" => {
                out.push(("border-top-right-radius".into(), css.clone()));
                out.push(("border-bottom-right-radius".into(), css));
            }
            _ => {}
        }
    }
    Ok(out)
}

fn radius_side_props(side: &str) -> &'static [&'static str] {
    match side {
        "t" => &["border-top-left-radius", "border-top-right-radius"],
        "b" => &["border-bottom-left-radius", "border-bottom-right-radius"],
        "l" => &["border-top-left-radius", "border-bottom-left-radius"],
        "r" => &["border-top-right-radius", "border-bottom-right-radius"],
        _ => &["border-radius"],
    }
}

fn lower_border_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let kind = match slots.get("kind") {
        Some(PaintArg::Token(k)) => k.as_str(),
        _ => "stroke",
    };
    let edge = match slots.get("edge") {
        Some(PaintArg::Token(e)) => e.as_str(),
        _ => "all",
    };
    let color = slots.get("color");
    let width = slots.get("width");
    let style = slots.get("style");
    match kind {
        "outline" => {
            let mut out = Vec::new();
            if let Some(v) = color {
                out.push(lower_named_css("outline-color", find_slot(spec, "color").unwrap(), v)?);
            }
            if let Some(v) = width {
                out.push(lower_named_css("outline-width", find_slot(spec, "width").unwrap(), v)?);
            }
            if let Some(v) = style {
                out.push(lower_named_css("outline-style", find_slot(spec, "style").unwrap(), v)?);
            }
            Ok(out)
        }
        "ring" => lower_ring(spec, color, width),
        "image" => {
            let mut out = Vec::new();
            if let Some(v) = color {
                out.push(lower_named_css("border-image-source", find_slot(spec, "color").unwrap(), v)?);
            }
            Ok(out)
        }
        _ => {
            let mut out = Vec::new();
            if let Some(v) = color {
                for prop in edge_props(edge, "color") {
                    out.push(lower_named_css(prop, find_slot(spec, "color").unwrap(), v)?);
                }
            }
            if let Some(v) = width {
                for prop in edge_props(edge, "width") {
                    out.push(lower_named_css(prop, find_slot(spec, "width").unwrap(), v)?);
                }
            }
            if let Some(v) = style {
                for prop in edge_props(edge, "style") {
                    out.push(lower_named_css(prop, find_slot(spec, "style").unwrap(), v)?);
                }
            }
            Ok(out)
        }
    }
}

fn edge_props(edge: &str, part: &str) -> Vec<&'static str> {
    match (edge, part) {
        ("all", "color") => vec!["border-color"],
        ("all", "width") => vec!["border-width"],
        ("all", "style") => vec!["border-style"],
        ("x", "color") => vec!["border-left-color", "border-right-color"],
        ("x", "width") => vec!["border-left-width", "border-right-width"],
        ("x", "style") => vec!["border-left-style", "border-right-style"],
        ("y", "color") => vec!["border-top-color", "border-bottom-color"],
        ("y", "width") => vec!["border-top-width", "border-bottom-width"],
        ("y", "style") => vec!["border-top-style", "border-bottom-style"],
        ("t", "color") => vec!["border-top-color"],
        ("t", "width") => vec!["border-top-width"],
        ("t", "style") => vec!["border-top-style"],
        ("b", "color") => vec!["border-bottom-color"],
        ("b", "width") => vec!["border-bottom-width"],
        ("b", "style") => vec!["border-bottom-style"],
        ("l", "color") => vec!["border-left-color"],
        ("l", "width") => vec!["border-left-width"],
        ("l", "style") => vec!["border-left-style"],
        ("r", "color") => vec!["border-right-color"],
        ("r", "width") => vec!["border-right-width"],
        ("r", "style") => vec!["border-right-style"],
        ("s", "color") => vec!["border-inline-start-color"],
        ("s", "width") => vec!["border-inline-start-width"],
        ("s", "style") => vec!["border-inline-start-style"],
        ("e", "color") => vec!["border-inline-end-color"],
        ("e", "width") => vec!["border-inline-end-width"],
        ("e", "style") => vec!["border-inline-end-style"],
        _ => vec!["border-color"],
    }
}

fn lower_ring(
    spec: &ConcernSpec,
    color: Option<&PaintArg>,
    width: Option<&PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let color_css = if let Some(v) = color {
        lower_slot_value(find_slot(spec, "color").unwrap(), v)?.1
    } else {
        "var(--ring)".into()
    };
    let width_css = if let Some(v) = width {
        lower_slot_value(find_slot(spec, "width").unwrap(), v)?.1
    } else {
        "var(--ring-width)".into()
    };
    // Existing ring path: double box-shadow (offset hole + ring).
    let shadow = format!(
        "0 0 0 var(--ring-offset) var(--background), 0 0 0 calc(var(--ring-offset) + {width_css}) {color_css}"
    );
    Ok(vec![("box-shadow".into(), shadow)])
}

fn lower_named_css(
    prop: &str,
    slot: &ConcernSlot,
    value: &PaintArg,
) -> Result<(String, String), StyleDiagnostic> {
    let (_, css) = lower_slot_value(slot, value)?;
    Ok((prop.to_string(), css))
}

fn lower_filter_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut filter_parts = Vec::new();
    let mut out = Vec::new();
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        let bare = match value {
            PaintArg::Token(t) => t
                .strip_prefix('-')
                .filter(|r| !r.is_empty())
                .unwrap_or(t)
                .to_string(),
            PaintArg::Bind(n) => format!("var(--{})", css_name_for_bind(n)),
            other => other.display(),
        };
        match name.as_str() {
            "blur" => filter_parts.push(format!("blur({bare}px)")),
            "brightness" => filter_parts.push(format!("brightness({}%)", bare)),
            "contrast" => filter_parts.push(format!("contrast({bare}%)")),
            "grayscale" => filter_parts.push(format!("grayscale({bare}%)")),
            "hue" => filter_parts.push(format!("hue-rotate({bare}deg)")),
            "invert" => filter_parts.push(format!("invert({bare}%)")),
            "sepia" => filter_parts.push(format!("sepia({bare}%)")),
            "saturate" => filter_parts.push(format!("saturate({bare}%)")),
            "backdrop" => {
                let css = if bare == "none" {
                    "none".into()
                } else {
                    format!("var(--{bare})")
                };
                out.push(("backdrop-filter".into(), css.clone()));
                out.push(("-webkit-backdrop-filter".into(), css));
            }
            "blend" => out.push(("mix-blend-mode".into(), bare)),
            "opacity" => {
                let css = if let Ok(n) = bare.parse::<u32>() {
                    format!("{}", n as f32 / 100.0)
                } else {
                    lower_slot_value(slot, value)?.1
                };
                out.push(("opacity".into(), css));
            }
            _ => {}
        }
    }
    if !filter_parts.is_empty() {
        out.push(("filter".into(), filter_parts.join(" ")));
    }
    Ok(out)
}

fn lower_type_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut out = Vec::new();
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        if name == "wrap" {
            let bare = match value {
                PaintArg::Token(t) => t.as_str(),
                _ => {
                    out.push(lower_slot_value(slot, value)?);
                    continue;
                }
            };
            match bare {
                "truncate" => {
                    out.push(("overflow".into(), "hidden".into()));
                    out.push(("text-overflow".into(), "ellipsis".into()));
                    out.push(("white-space".into(), "nowrap".into()));
                }
                "pretty" => {
                    out.push(("text-wrap".into(), "pretty".into()));
                    out.push(("white-space".into(), "normal".into()));
                }
                "nowrap" => out.push(("white-space".into(), "nowrap".into())),
                "normal" => out.push(("white-space".into(), "normal".into())),
                _ => out.push(lower_slot_value(slot, value)?),
            }
            continue;
        }
        if name == "case" {
            if let PaintArg::Token(t) = value {
                let css = match t.as_str() {
                    "normal" => "none",
                    "label-transform" => "var(--label-transform)",
                    other => other,
                };
                out.push(("text-transform".into(), css.into()));
                continue;
            }
        }
        out.push(lower_slot_value(slot, value)?);
    }
    Ok(out)
}

fn lower_motion_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut out = Vec::new();
    let mut tx: Option<String> = None;
    let mut ty: Option<String> = None;
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        match name.as_str() {
            "x" => tx = Some(lower_slot_value(slot, value)?.1),
            "y" => ty = Some(lower_slot_value(slot, value)?.1),
            "scale" => {
                let (_, css) = lower_slot_value(slot, value)?;
                let scaled = if let PaintArg::Token(t) = value {
                    if let Ok(n) = t.parse::<f32>() {
                        format!("{}", n / 100.0)
                    } else {
                        css
                    }
                } else {
                    css
                };
                out.push(("scale".into(), scaled));
            }
            _ => out.push(lower_slot_value(slot, value)?),
        }
    }
    match (tx, ty) {
        (Some(x), Some(y)) => out.push(("translate".into(), format!("{x} {y}"))),
        (Some(x), None) => out.push(("translate".into(), format!("{x} 0"))),
        (None, Some(y)) => out.push(("translate".into(), format!("0 {y}"))),
        (None, None) => {}
    }
    Ok(out)
}

pub fn lower_slot_value(
    slot: &ConcernSlot,
    value: &PaintArg,
) -> Result<(String, String), StyleDiagnostic> {
    let css = match value {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::MemberPath { root, path } => crate::token_groups::bridge_var_ref(root, path),
        PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: slot.name.to_string(),
                value: value.display(),
                source: value.display(),
            });
        }
        PaintArg::Token(tok) => {
            let bare = tok
                .strip_prefix('-')
                .filter(|r| !r.is_empty())
                .unwrap_or(tok);
            if bare == "hairline" && slot.name == "width" {
                "var(--border-width)".into()
            } else if bare == "control-stroke" && slot.name == "width" {
                "var(--control-stroke)".into()
            } else if slot.name == "aspect" {
                match bare {
                    "square" => "1 / 1".into(),
                    "video" => "16 / 9".into(),
                    "auto" => "auto".into(),
                    other => other.to_string(),
                }
            } else if slot.name == "order" && bare == "first" {
                "-9999".into()
            } else if slot.name == "order" && bare == "last" {
                "9999".into()
            } else if slot.keywords.contains(&bare) && slot.scale.is_none() {
                match bare {
                    "current" => "currentColor".into(),
                    other => other.to_string(),
                }
            } else if let Some(scale) = slot.scale {
                if let Some(lowered) = scales::lower(scale, bare) {
                    lowered
                } else if matches!(bare, "transparent" | "current" | "currentColor") {
                    if bare == "current" {
                        "currentColor".into()
                    } else {
                        bare.into()
                    }
                } else if slot.keywords.contains(&bare) {
                    match bare {
                        "current" => "currentColor".into(),
                        other => other.to_string(),
                    }
                } else if slot.name == "width" && bare.parse::<u32>().is_ok() {
                    format!("{bare}px")
                } else {
                    format!("var(--{bare})")
                }
            } else if slot.name == "width" && bare.parse::<u32>().is_ok() {
                format!("{bare}px")
            } else {
                match bare {
                    "current" => "currentColor".into(),
                    other => other.to_string(),
                }
            }
        }
    };
    Ok((slot.css_prop.to_string(), css))
}

fn css_name_for_bind(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use crate::setters::lower_op_css;
    use std::collections::HashMap;

    fn only_op(src: &str) -> PaintOp {
        let (style, diags) = parse(src);
        assert!(
            diags.is_empty(),
            "{src}: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        match style {
            crate::ir::Style::Set(op) => op,
            other => panic!("{src}: expected Set, got {other:?}"),
        }
    }

    #[test]
    fn atlas_lists_full_concern_set_not_markers() {
        let names: Vec<&str> = concerns().iter().map(|c| c.name).collect();
        for want in [
            "fill", "ink", "border", "radius", "elevation", "filter", "size", "space", "place",
            "flow", "overflow", "type", "motion",
        ] {
            assert!(names.contains(&want), "missing concern {want} in {names:?}");
        }
        assert_eq!(names.len(), 13, "{names:?}");
        for m in MARKERS {
            assert!(!names.contains(m), "marker {m} must not be a concern");
        }
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
    fn resolve_card_as_component_composite() {
        let spec = find_concern("border").unwrap();
        let c = resolve_concern_name(spec, "card").expect("card.border");
        assert_eq!(c.name, "card.border");
    }

    #[test]
    fn resolve_dialog_as_component_composite() {
        let spec = find_concern("border").unwrap();
        let c = resolve_concern_name(spec, "dialog").expect("dialog.border");
        assert_eq!(c.name, "dialog.border");
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
    fn border_subtle_emits_member_vars() {
        let op = only_op("style({ paint: border(subtle) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-color".into(), "var(--border-subtle-color)".into())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-width".into(), "var(--border-subtle-width)".into())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-style".into(), "var(--border-subtle-style)".into())),
            "{css:?}"
        );
    }

    #[test]
    fn border_card_emits_component_member_vars() {
        let op = only_op("style({ paint: border(card) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-color".into(), "var(--card-border)".into())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-width".into(), "var(--card-border-width)".into())),
            "{css:?}"
        );
    }

    #[test]
    fn border_slot_object_emits_only_named_slots() {
        let op = only_op("style({ paint: border({ color: muted, width: hairline }) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-color".into(), "var(--muted)".into())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-width".into(), "var(--border-width)".into())),
            "{css:?}"
        );
        assert_eq!(css.len(), 2, "only named slots: {css:?}");
    }

    #[test]
    fn radius_is_not_a_border_slot() {
        let (_, diags) = parse("style({ paint: border({ radius: md }) })");
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
    fn color_only_border_is_retired() {
        let (_, diags) = parse("style({ paint: border(primary) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::RetiredSetter { .. })),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn duplicate_slot_in_object_names_the_slot() {
        let (_, diags) = parse("style({ paint: border({ color: muted, color: primary }) })");
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
    fn slot_color_refuses_theme_owned_public_name() {
        let (_, diags) = parse("style({ paint: border({ color: dialog-border }) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::ThemeOwnedToken { name, .. } if name == "dialog-border")),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn same_kind_twice_clashes() {
        let (style, parse_diags) =
            parse("style({ paint: border(subtle).border({ color: primary }) })");
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
        assert!(msg.contains("border({ color: primary })") || msg.contains("color: primary"), "{msg}");
    }

    #[test]
    fn stroke_plus_ring_do_not_clash() {
        let (style, diags) = parse(
            "style({ paint: border(subtle).border({ kind: ring, color: primary }) })",
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
    fn when_override_does_not_clash() {
        let (style, diags) = parse(
            "style({ paint: border(subtle), when: { hover: { paint: border({ color: primary }) } } })",
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
    fn size_and_space_do_not_clash() {
        let (style, diags) = parse("style({ box: size({ w: full }).space({ pad-x: 3 }) })");
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
    fn size_slot_object_lowers() {
        let op = only_op("style({ box: size({ w: full, h: full }) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.iter().any(|(p, v)| p == "width" && v.contains("100%")),
            "{css:?}"
        );
        assert!(
            css.iter().any(|(p, v)| p == "height" && v.contains("100%")),
            "{css:?}"
        );
    }

    #[test]
    fn retired_bg_names_fill() {
        assert_eq!(retired_replacement("bg").unwrap().0, "fill");
        assert_eq!(retired_replacement("fg").unwrap().0, "ink");
        assert_eq!(retired_replacement("padX").unwrap().0, "space");
        assert_eq!(retired_replacement("w").unwrap().0, "size");
        assert_eq!(retired_replacement("shadow").unwrap().0, "elevation");
        assert_eq!(retired_replacement("gap").unwrap().0, "gap");
    }
}
