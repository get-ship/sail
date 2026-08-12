//! Table-driven Style setters → sx vocab atoms / CSS scales.

use std::borrow::Cow;

use crate::prefixes as sx_rules;
use crate::scales;
use crate::style_ir::{self, SxDecl, SxValue};
use crate::style_vocab::{self, Resolved};

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
    /// Paint slots: every CSS longhand this setter can write (via scale
    /// lowering, keyword maps, bare composites, and `bind` args). Clash
    /// analysis keys on these, so the list must be complete.
    pub slots: &'static [&'static str],
}

/// Closed product setter table (re-homes full vocab coverage via vocab heads).
pub fn setters() -> &'static [SetterSpec] {
    &[
        SetterSpec {
            name: "bg",
            vocab: "bg",
            scale: Some("color"),
            keywords: &["transparent", "current", "currentColor", "inherit", "none"],
            negatable: false,
            slots: &["background-color", "background-image"],
        },
        SetterSpec {
            name: "fg",
            vocab: "text",
            scale: Some("color"),
            keywords: &["transparent", "current", "currentColor", "inherit"],
            negatable: false,
            slots: &["color"],
        },
        // border disambiguates color / width / style by value via vocab.
        SetterSpec {
            name: "border",
            vocab: "border",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "solid",
                "dashed",
                "dotted",
                "none",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &["border-color", "border-width", "border-style"],
        },
        SetterSpec {
            name: "borderX",
            vocab: "border-x",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &[
                "border-left-color",
                "border-right-color",
                "border-left-width",
                "border-right-width",
                "border-left-style",
                "border-right-style",
            ],
        },
        SetterSpec {
            name: "borderY",
            vocab: "border-y",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &[
                "border-top-color",
                "border-bottom-color",
                "border-top-width",
                "border-bottom-width",
                "border-top-style",
                "border-bottom-style",
            ],
        },
        SetterSpec {
            name: "borderS",
            vocab: "border-s",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &[
                "border-inline-start-color",
                "border-inline-start-width",
                "border-inline-start-style",
            ],
        },
        SetterSpec {
            name: "borderE",
            vocab: "border-e",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &[
                "border-inline-end-color",
                "border-inline-end-width",
                "border-inline-end-style",
            ],
        },
        SetterSpec {
            name: "borderT",
            vocab: "border-t",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &["border-top-color", "border-top-width", "border-top-style"],
        },
        SetterSpec {
            name: "borderB",
            vocab: "border-b",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &[
                "border-bottom-color",
                "border-bottom-width",
                "border-bottom-style",
            ],
        },
        SetterSpec {
            name: "borderR",
            vocab: "border-r",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &[
                "border-right-color",
                "border-right-width",
                "border-right-style",
            ],
        },
        SetterSpec {
            name: "borderL",
            vocab: "border-l",
            scale: Some("color"),
            keywords: &[
                "transparent",
                "current",
                "currentColor",
                "0",
                "1",
                "2",
                "3",
                "4",
                "8",
            ],
            negatable: false,
            slots: &[
                "border-left-color",
                "border-left-width",
                "border-left-style",
            ],
        },
        SetterSpec {
            name: "borderWidth",
            vocab: "border",
            scale: None,
            keywords: &["0", "1", "2", "3", "4", "8", "control-stroke"],
            negatable: false,
            slots: &["border-width"],
        },
        SetterSpec {
            name: "borderBWidth",
            vocab: "border-b",
            scale: None,
            keywords: &["0", "1", "2", "3", "4", "8"],
            negatable: false,
            slots: &["border-bottom-width"],
        },
        SetterSpec {
            name: "borderTWidth",
            vocab: "border-t",
            scale: None,
            keywords: &["0", "1", "2", "3", "4", "8"],
            negatable: false,
            slots: &["border-top-width"],
        },
        SetterSpec {
            name: "borderEWidth",
            vocab: "border-e",
            scale: None,
            keywords: &["0", "1", "2", "3", "4", "8"],
            negatable: false,
            slots: &["border-inline-end-width"],
        },
        SetterSpec {
            name: "borderSWidth",
            vocab: "border-s",
            scale: None,
            keywords: &["0", "1", "2", "3", "4", "8"],
            negatable: false,
            slots: &["border-inline-start-width"],
        },
        SetterSpec {
            name: "borderRWidth",
            vocab: "border-r",
            scale: None,
            keywords: &["0", "1", "2", "3", "4", "8"],
            negatable: false,
            slots: &["border-right-width"],
        },
        SetterSpec {
            name: "borderLWidth",
            vocab: "border-l",
            scale: None,
            keywords: &["0", "1", "2", "3", "4", "8"],
            negatable: false,
            slots: &["border-left-width"],
        },
        SetterSpec {
            name: "radius",
            vocab: "rounded",
            scale: Some("radius"),
            keywords: &["none", "full"],
            negatable: false,
            slots: &["border-radius"],
        },
        SetterSpec {
            name: "radiusT",
            vocab: "rounded-t",
            scale: Some("radius"),
            keywords: &["none", "full"],
            negatable: false,
            slots: &["border-start-start-radius", "border-start-end-radius"],
        },
        SetterSpec {
            name: "radiusB",
            vocab: "rounded-b",
            scale: Some("radius"),
            keywords: &["none", "full"],
            negatable: false,
            slots: &["border-end-start-radius", "border-end-end-radius"],
        },
        SetterSpec {
            name: "radiusL",
            vocab: "rounded-l",
            scale: Some("radius"),
            keywords: &["none", "full"],
            negatable: false,
            slots: &["border-start-start-radius", "border-end-start-radius"],
        },
        SetterSpec {
            name: "radiusR",
            vocab: "rounded-r",
            scale: Some("radius"),
            keywords: &["none", "full"],
            negatable: false,
            slots: &["border-start-end-radius", "border-end-end-radius"],
        },
        SetterSpec {
            name: "pad",
            vocab: "p",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding"],
        },
        SetterSpec {
            name: "padX",
            vocab: "px",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding-inline"],
        },
        SetterSpec {
            name: "padY",
            vocab: "py",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding-block"],
        },
        SetterSpec {
            name: "padT",
            vocab: "pt",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding-top"],
        },
        SetterSpec {
            name: "padB",
            vocab: "pb",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding-bottom"],
        },
        SetterSpec {
            name: "padL",
            vocab: "pl",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding-left"],
        },
        SetterSpec {
            name: "padR",
            vocab: "pr",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding-right"],
        },
        SetterSpec {
            name: "padS",
            vocab: "ps",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding-inline-start"],
        },
        SetterSpec {
            name: "padE",
            vocab: "pe",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["padding-inline-end"],
        },
        SetterSpec {
            name: "m",
            vocab: "m",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin"],
        },
        SetterSpec {
            name: "mx",
            vocab: "mx",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin-inline"],
        },
        SetterSpec {
            name: "my",
            vocab: "my",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin-block"],
        },
        SetterSpec {
            name: "mt",
            vocab: "mt",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin-top"],
        },
        SetterSpec {
            name: "mb",
            vocab: "mb",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin-bottom"],
        },
        SetterSpec {
            name: "ms",
            vocab: "ms",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin-inline-start"],
        },
        SetterSpec {
            name: "me",
            vocab: "me",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin-inline-end"],
        },
        SetterSpec {
            name: "ml",
            vocab: "ml",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin-left"],
        },
        SetterSpec {
            name: "mr",
            vocab: "mr",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["margin-right"],
        },
        SetterSpec {
            name: "minH",
            vocab: "min-h",
            scale: Some("size"),
            keywords: &["0", "full", "screen"],
            negatable: false,
            slots: &["min-height"],
        },
        SetterSpec {
            name: "minW",
            vocab: "min-w",
            scale: Some("size"),
            keywords: &["0", "full", "panel-menu-min-w", "panel-rail-w"],
            negatable: false,
            slots: &["min-width"],
        },
        SetterSpec {
            name: "maxH",
            vocab: "max-h",
            scale: Some("size"),
            keywords: &["0", "full", "screen"],
            negatable: false,
            slots: &["max-height"],
        },
        SetterSpec {
            name: "maxW",
            vocab: "max-w",
            scale: Some("size"),
            keywords: &[
                "0",
                "full",
                "screen",
                "none",
                "panel-max-w-md",
                "panel-max-w-sm",
                "panel-max-w-lg",
                "panel-max-w-xs",
                "panel-ch-w-md",
                "panel-ch-w-lg",
            ],
            negatable: false,
            slots: &["max-width"],
        },
        SetterSpec {
            name: "w",
            vocab: "w",
            scale: Some("size"),
            keywords: &[
                "0",
                "full",
                "auto",
                "max",
                "border-width",
                "panel-toast-region-width",
                "panel-max-w-md",
                "panel-max-w-sm",
                "panel-max-w-lg",
                "panel-max-w-xs",
            ],
            negatable: false,
            slots: &["width"],
        },
        SetterSpec {
            name: "h",
            vocab: "h",
            scale: Some("size"),
            keywords: &["0", "full", "auto", "border-width"],
            negatable: false,
            slots: &["height"],
        },
        SetterSpec {
            name: "size",
            vocab: "size",
            scale: Some("size"),
            keywords: &["0", "full"],
            negatable: false,
            slots: &["width", "height"],
        },
        SetterSpec {
            name: "list",
            vocab: "list",
            scale: None,
            keywords: &["none", "disc", "decimal"],
            negatable: false,
            slots: &["list-style-type"],
        },
        SetterSpec {
            name: "fontSize",
            vocab: "text",
            scale: Some("text"),
            keywords: &[],
            negatable: false,
            slots: &["font-size"],
        },
        SetterSpec {
            name: "fontWeight",
            vocab: "font",
            scale: Some("weight"),
            keywords: &[],
            negatable: false,
            slots: &["font-weight"],
        },
        SetterSpec {
            name: "fontFamily",
            vocab: "font",
            scale: Some("font"),
            keywords: &["body", "heading", "mono", "display"],
            negatable: false,
            slots: &["font-family"],
        },
        SetterSpec {
            name: "leading",
            vocab: "leading",
            scale: Some("leading"),
            keywords: &["none"],
            negatable: false,
            slots: &["line-height"],
        },
        SetterSpec {
            name: "tracking",
            vocab: "tracking",
            scale: Some("tracking"),
            keywords: &[],
            negatable: false,
            slots: &["letter-spacing"],
        },
        SetterSpec {
            name: "shadow",
            vocab: "shadow",
            scale: Some("shadow"),
            keywords: &["none"],
            negatable: false,
            slots: &["box-shadow"],
        },
        SetterSpec {
            name: "opacity",
            vocab: "opacity",
            scale: None,
            keywords: &[
                "0", "5", "10", "20", "25", "30", "40", "50", "60", "70", "75", "80", "90", "95",
                "100",
            ],
            negatable: false,
            slots: &["opacity"],
        },
        SetterSpec {
            name: "whitespace",
            vocab: "whitespace",
            scale: None,
            keywords: &[
                "normal",
                "nowrap",
                "pre",
                "pre-line",
                "pre-wrap",
                "break-spaces",
            ],
            negatable: false,
            slots: &["white-space"],
        },
        SetterSpec {
            name: "align",
            vocab: "align",
            scale: None,
            keywords: &[
                "baseline",
                "top",
                "middle",
                "bottom",
                "text-top",
                "text-bottom",
            ],
            negatable: false,
            slots: &["vertical-align"],
        },
        SetterSpec {
            name: "textAlign",
            vocab: "text",
            scale: None,
            keywords: &["left", "center", "right", "justify", "start", "end"],
            negatable: false,
            slots: &["text-align"],
        },
        SetterSpec {
            name: "break",
            vocab: "break",
            scale: None,
            keywords: &["normal", "words", "all", "keep"],
            negatable: false,
            slots: &["overflow-wrap", "word-break"],
        },
        SetterSpec {
            name: "stroke",
            vocab: "stroke",
            scale: Some("color"),
            keywords: &["none", "current", "currentColor"],
            negatable: false,
            slots: &["stroke"],
        },
        SetterSpec {
            name: "fill",
            vocab: "fill",
            scale: Some("color"),
            keywords: &["none", "current", "currentColor"],
            negatable: false,
            slots: &["fill"],
        },
        SetterSpec {
            name: "ring",
            vocab: "outline",
            scale: Some("color"),
            keywords: &["none"],
            negatable: false,
            slots: &["outline-color", "outline-style"],
        },
        SetterSpec {
            name: "outline",
            vocab: "outline",
            scale: None,
            keywords: &[
                "none",
                "solid",
                "dashed",
                "1",
                "2",
                "4",
                "ring-width",
                "ring",
            ],
            negatable: false,
            slots: &["outline-color", "outline-width", "outline-style"],
        },
        SetterSpec {
            name: "outlineOffset",
            vocab: "outline-offset",
            scale: Some("space"),
            keywords: &["0", "ring-offset", "ring-inset"],
            negatable: false,
            slots: &["outline-offset"],
        },
        SetterSpec {
            name: "translateX",
            vocab: "translate-x",
            scale: Some("space"),
            keywords: &["0"],
            negatable: true,
            slots: &["translate"],
        },
        SetterSpec {
            name: "translateY",
            vocab: "translate-y",
            scale: Some("space"),
            keywords: &["0"],
            negatable: true,
            slots: &["translate"],
        },
        SetterSpec {
            name: "z",
            vocab: "z",
            scale: Some("z"),
            keywords: &["0", "1", "2", "auto"],
            negatable: false,
            slots: &["z-index"],
        },
        SetterSpec {
            name: "cursor",
            vocab: "cursor",
            scale: None,
            keywords: &[
                "auto",
                "default",
                "pointer",
                "not-allowed",
                "text",
                "crosshair",
                "grab",
                "grabbing",
                "move",
                "wait",
                "help",
                "none",
                "cell",
                "col-resize",
                "row-resize",
                "ew-resize",
                "ns-resize",
            ],
            negatable: false,
            slots: &["cursor"],
        },
        SetterSpec {
            name: "select",
            vocab: "select",
            scale: None,
            keywords: &["none", "text", "all", "auto"],
            negatable: false,
            slots: &["user-select"],
        },
        SetterSpec {
            name: "pointerEvents",
            vocab: "pointer-events",
            scale: None,
            keywords: &["none", "auto"],
            negatable: false,
            slots: &["pointer-events"],
        },
        SetterSpec {
            name: "overflow",
            vocab: "overflow",
            scale: None,
            keywords: &["auto", "hidden", "clip", "visible", "scroll"],
            negatable: false,
            slots: &["overflow"],
        },
        SetterSpec {
            name: "overflowX",
            vocab: "overflow-x",
            scale: None,
            keywords: &["auto", "hidden", "clip", "visible", "scroll"],
            negatable: false,
            slots: &["overflow-x"],
        },
        SetterSpec {
            name: "overflowY",
            vocab: "overflow-y",
            scale: None,
            keywords: &["auto", "hidden", "clip", "visible", "scroll"],
            negatable: false,
            slots: &["overflow-y"],
        },
        SetterSpec {
            name: "overscroll",
            vocab: "overscroll",
            scale: None,
            keywords: &["auto", "contain", "none"],
            negatable: false,
            slots: &["overscroll-behavior"],
        },
        SetterSpec {
            name: "tab",
            vocab: "tab",
            scale: None,
            keywords: &["1", "2", "4", "8"],
            negatable: false,
            slots: &["tab-size"],
        },
        SetterSpec {
            name: "underline",
            vocab: "underline",
            scale: None,
            keywords: &["true", "none", "line-through"],
            negatable: false,
            slots: &["text-decoration-line"],
        },
        SetterSpec {
            name: "scrollbarThumb",
            vocab: "scrollbar-thumb",
            scale: Some("color"),
            keywords: &["transparent"],
            negatable: false,
            slots: &["--sx-scrollbar-thumb"],
        },
        SetterSpec {
            name: "scrollbarTrack",
            vocab: "scrollbar-track",
            scale: Some("color"),
            keywords: &["transparent"],
            negatable: false,
            slots: &["--sx-scrollbar-track"],
        },
        SetterSpec {
            name: "display",
            vocab: "hidden",
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
            negatable: false,
            slots: &["display"],
        },
        SetterSpec {
            name: "position",
            vocab: "relative",
            scale: None,
            keywords: &["relative", "absolute", "sticky", "fixed", "static"],
            negatable: false,
            slots: &["position"],
        },
        SetterSpec {
            name: "top",
            vocab: "top",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["top"],
        },
        SetterSpec {
            name: "bottom",
            vocab: "bottom",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["bottom"],
        },
        SetterSpec {
            name: "left",
            vocab: "left",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["left"],
        },
        SetterSpec {
            name: "right",
            vocab: "right",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["right"],
        },
        SetterSpec {
            name: "inset",
            vocab: "inset",
            scale: Some("space"),
            keywords: &["0", "auto"],
            negatable: true,
            slots: &["inset"],
        },
        SetterSpec {
            name: "appearance",
            vocab: "appearance",
            scale: None,
            keywords: &["none", "auto"],
            negatable: false,
            slots: &["appearance"],
        },
        SetterSpec {
            name: "content",
            vocab: "content",
            scale: None,
            keywords: &["empty"],
            negatable: false,
            slots: &["content"],
        },
        SetterSpec {
            name: "shrink",
            vocab: "shrink",
            scale: None,
            keywords: &["0", "1"],
            negatable: false,
            slots: &["flex-shrink"],
        },
        SetterSpec {
            name: "grow",
            vocab: "grow",
            scale: None,
            keywords: &["0", "1"],
            negatable: false,
            slots: &["flex-grow"],
        },
        // flex-basis — size scale (0 / full / breakpoint-* / panel-* / space ramp).
        SetterSpec {
            name: "basis",
            vocab: "basis",
            scale: Some("size"),
            keywords: &["0", "auto", "full"],
            negatable: false,
            slots: &["flex-basis"],
        },
        SetterSpec {
            name: "self",
            vocab: "self",
            scale: None,
            keywords: &["auto", "start", "center", "end", "stretch", "baseline"],
            negatable: false,
            slots: &["align-self"],
        },
        SetterSpec {
            name: "boxSizing",
            vocab: "box",
            scale: None,
            keywords: &["border", "content"],
            negatable: false,
            slots: &["box-sizing"],
        },
        SetterSpec {
            name: "transition",
            vocab: "transition",
            scale: None,
            keywords: &["none", "all", "colors", "opacity", "shadow", "transform"],
            negatable: false,
            slots: &["transition-property"],
        },
        SetterSpec {
            name: "duration",
            vocab: "duration",
            scale: Some("duration"),
            keywords: &["fast", "medium", "slow", "instant", "deliberate", "glacial"],
            negatable: false,
            slots: &["transition-duration"],
        },
        SetterSpec {
            name: "ease",
            vocab: "ease",
            scale: Some("ease"),
            keywords: &["in", "out", "in-out", "out-soft", "spring"],
            negatable: false,
            slots: &["transition-timing-function"],
        },
        SetterSpec {
            name: "rotate",
            vocab: "rotate",
            scale: Some("angle"),
            keywords: &["0"],
            negatable: true,
            slots: &["rotate"],
        },
        // Transform scale (CSS `scale`) — percent steps + bind for timeline tracks.
        SetterSpec {
            name: "scale",
            vocab: "scale",
            scale: Some("percent"),
            keywords: &[
                "0", "50", "75", "90", "95", "100", "105", "110", "125", "150",
            ],
            negatable: false,
            slots: &["scale"],
        },
        SetterSpec {
            name: "tapHighlight",
            vocab: "tap-highlight",
            scale: None,
            keywords: &["none", "auto"],
            negatable: false,
            slots: &["-webkit-tap-highlight-color"],
        },
        SetterSpec {
            name: "saturate",
            vocab: "saturate",
            scale: None,
            keywords: &["0", "50", "60", "65", "75", "100"],
            negatable: false,
            slots: &["filter"],
        },
        // Phase 3 gap keys — clip / mask / filter suite / 3D / container queries.
        SetterSpec {
            name: "clipPath",
            vocab: "clip-path",
            scale: None,
            keywords: &["none", "circle", "ellipse", "inset-0", "border-box", "padding-box", "content-box", "fill-box", "stroke-box", "view-box", "margin-box"],
            negatable: false,
            slots: &["clip-path"],
        },
        SetterSpec {
            name: "mask",
            vocab: "mask",
            scale: None,
            keywords: &["none"],
            negatable: false,
            slots: &["mask-image"],
        },
        SetterSpec {
            name: "maskImage",
            vocab: "mask-image",
            scale: None,
            keywords: &["none"],
            negatable: false,
            slots: &["mask-image"],
        },
        SetterSpec {
            name: "blur",
            vocab: "blur",
            scale: None,
            keywords: &["0", "1", "2", "4", "8", "12", "16", "24"],
            negatable: false,
            slots: &["filter"],
        },
        SetterSpec {
            name: "brightness",
            vocab: "brightness",
            scale: None,
            keywords: &["0", "50", "75", "90", "95", "100", "105", "110", "125", "150", "200"],
            negatable: false,
            slots: &["filter"],
        },
        SetterSpec {
            name: "contrast",
            vocab: "contrast",
            scale: None,
            keywords: &["0", "50", "75", "90", "95", "100", "105", "110", "125", "150", "200"],
            negatable: false,
            slots: &["filter"],
        },
        SetterSpec {
            name: "grayscale",
            vocab: "grayscale",
            scale: None,
            keywords: &["0", "50", "100"],
            negatable: false,
            slots: &["filter"],
        },
        SetterSpec {
            name: "hueRotate",
            vocab: "hue-rotate",
            scale: None,
            keywords: &["0", "15", "30", "60", "90", "180"],
            negatable: true,
            slots: &["filter"],
        },
        SetterSpec {
            name: "invert",
            vocab: "invert",
            scale: None,
            keywords: &["0", "50", "100"],
            negatable: false,
            slots: &["filter"],
        },
        SetterSpec {
            name: "sepia",
            vocab: "sepia",
            scale: None,
            keywords: &["0", "50", "100"],
            negatable: false,
            slots: &["filter"],
        },
        SetterSpec {
            name: "perspective",
            vocab: "perspective",
            scale: None,
            keywords: &["none", "0", "250", "500", "750", "1000", "1500", "2000"],
            negatable: false,
            slots: &["perspective"],
        },
        SetterSpec {
            name: "rotateX",
            vocab: "rotate-x",
            scale: Some("angle"),
            keywords: &["0"],
            negatable: true,
            slots: &["rotate"],
        },
        SetterSpec {
            name: "rotateY",
            vocab: "rotate-y",
            scale: Some("angle"),
            keywords: &["0"],
            negatable: true,
            slots: &["rotate"],
        },
        SetterSpec {
            name: "rotateZ",
            vocab: "rotate-z",
            scale: Some("angle"),
            keywords: &["0"],
            negatable: true,
            slots: &["rotate"],
        },
        SetterSpec {
            name: "containerType",
            vocab: "container-type",
            scale: None,
            keywords: &["normal", "size", "inline-size"],
            negatable: false,
            slots: &["container-type"],
        },
        SetterSpec {
            name: "containerName",
            vocab: "container-name",
            scale: None,
            keywords: &["none"],
            negatable: false,
            slots: &["container-name"],
        },
        SetterSpec {
            name: "container",
            vocab: "container",
            scale: None,
            keywords: &["normal", "size", "inline-size"],
            negatable: false,
            slots: &["container-type"],
        },
        // Host marker for group-* descendant host Style — class `sail-grp`, no CSS decls.
        SetterSpec {
            name: "group",
            vocab: "group",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &[],
        },
        // Bare composite atoms
        SetterSpec {
            name: "animate",
            vocab: "animate",
            scale: None,
            keywords: &["spin"],
            negatable: false,
            slots: &["animation"],
        },
        SetterSpec {
            name: "bgImage",
            vocab: "bg-image",
            scale: None,
            keywords: &["none", "primary-accent"],
            negatable: false,
            slots: &["background-image"],
        },
        // Border-image corner treatment (component.card --corner-image).
        SetterSpec {
            name: "borderImage",
            vocab: "border-image",
            scale: None,
            keywords: &["none", "corner-image"],
            negatable: false,
            slots: &["border-image"],
        },
        // Backdrop filter (+ webkit prefix) via component tokens (card --card-backdrop-filter).
        SetterSpec {
            name: "backdrop",
            vocab: "backdrop",
            scale: None,
            keywords: &["none", "card-backdrop-filter"],
            negatable: false,
            slots: &["backdrop-filter", "-webkit-backdrop-filter"],
        },
        SetterSpec {
            name: "avatarMedia",
            vocab: "avatar-media",
            scale: None,
            keywords: &["true", "on"],
            negatable: false,
            slots: &[
                "display",
                "width",
                "height",
                "object-fit",
                "padding",
                "box-sizing",
            ],
        },
        // SVG child pack: display/block + fill host size (icon glyph box).
        SetterSpec {
            name: "glyphFill",
            vocab: "glyph-fill",
            scale: None,
            keywords: &["true", "on"],
            negatable: false,
            slots: &["display", "width", "height"],
        },
        // Standard scrollbar pack (width thin + color vars); webkit thumb chrome not on Style yet.
        SetterSpec {
            name: "scrollbarThin",
            vocab: "scrollbar-thin",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &["scrollbar-width", "scrollbar-color"],
        },
        // isolation: isolate (stacking root for overlays / theme swatches).
        SetterSpec {
            name: "isolate",
            vocab: "isolate",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &["isolation"],
        },
        SetterSpec {
            name: "mixBlend",
            vocab: "mix-blend",
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
            negatable: false,
            slots: &["mix-blend-mode"],
        },
        SetterSpec {
            name: "flexDir",
            vocab: "flex-dir",
            scale: None,
            keywords: &["row", "column", "row-reverse", "column-reverse"],
            negatable: false,
            slots: &["flex-direction"],
        },
        // Bare text/layout composites (utility-string product is dead; Style owns these).
        SetterSpec {
            name: "truncate",
            vocab: "truncate",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &["overflow", "text-overflow", "white-space"],
        },
        SetterSpec {
            name: "textCase",
            vocab: "uppercase",
            scale: None,
            keywords: &["uppercase", "lowercase", "capitalize", "normal"],
            negatable: false,
            slots: &["text-transform"],
        },
        SetterSpec {
            name: "nums",
            vocab: "tabular-nums",
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
            negatable: false,
            slots: &["font-variant-numeric"],
        },
        SetterSpec {
            name: "origin",
            vocab: "origin",
            scale: None,
            keywords: &[
                "center",
                "top",
                "top-right",
                "right",
                "bottom-right",
                "bottom",
                "bottom-left",
                "left",
                "top-left",
            ],
            negatable: false,
            slots: &["transform-origin"],
        },
        SetterSpec {
            name: "textGradient",
            vocab: "text-gradient",
            scale: None,
            keywords: &["primary", "accent"],
            negatable: false,
            slots: &[
                "background",
                "-webkit-background-clip",
                "background-clip",
                "color",
            ],
        },
        SetterSpec {
            name: "bgGlow",
            vocab: "bg-glow",
            scale: None,
            keywords: &["primary", "accent"],
            negatable: false,
            slots: &["position", "inset", "background", "pointer-events"],
        },
        // a11y clip pack (position absolute + 1px box + overflow clip).
        SetterSpec {
            name: "srOnly",
            vocab: "sr-only",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &[
                "position",
                "width",
                "height",
                "padding",
                "margin",
                "overflow",
                "clip-path",
                "white-space",
                "border-width",
            ],
        },
        // visibility: hidden | visible | collapse
        SetterSpec {
            name: "visibility",
            vocab: "visibility",
            scale: None,
            keywords: &["hidden", "visible", "collapse"],
            negatable: false,
            slots: &["visibility"],
        },
        // aspect-ratio map (square / video / auto)
        SetterSpec {
            name: "aspect",
            vocab: "aspect",
            scale: None,
            keywords: &["square", "video", "auto"],
            negatable: false,
            slots: &["aspect-ratio"],
        },
        // object-fit
        SetterSpec {
            name: "objectFit",
            vocab: "object",
            scale: None,
            keywords: &["contain", "cover", "fill", "none", "scale-down"],
            negatable: false,
            slots: &["object-fit"],
        },
        // CSS transform full value (canvas pan/zoom via bind)
        SetterSpec {
            name: "transform",
            vocab: "transform",
            scale: None,
            keywords: &["none"],
            negatable: false,
            slots: &["transform"],
        },
        // font-style
        SetterSpec {
            name: "italic",
            vocab: "italic",
            scale: None,
            keywords: &["true", "false", "on", "off"],
            negatable: false,
            slots: &["font-style"],
        },
        // Legacy-vocab long tail — longhands that had no Style setter yet.
        SetterSpec {
            name: "decoration",
            vocab: "decoration",
            scale: Some("color"),
            keywords: &["transparent", "current", "currentColor"],
            negatable: false,
            slots: &["text-decoration-color"],
        },
        SetterSpec {
            name: "indent",
            vocab: "indent",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["text-indent"],
        },
        SetterSpec {
            name: "underlineOffset",
            vocab: "underline-offset",
            scale: Some("space"),
            keywords: &["auto"],
            negatable: false,
            slots: &["text-underline-offset"],
        },
        SetterSpec {
            name: "antialiased",
            vocab: "antialiased",
            scale: None,
            keywords: &["true", "false"],
            negatable: false,
            slots: &["-webkit-font-smoothing"],
        },
        SetterSpec {
            name: "caret",
            vocab: "caret",
            scale: Some("color"),
            keywords: &["transparent", "current", "currentColor"],
            negatable: false,
            slots: &["caret-color"],
        },
        SetterSpec {
            name: "accent",
            vocab: "accent",
            scale: Some("color"),
            keywords: &["auto", "transparent", "current", "currentColor"],
            negatable: false,
            slots: &["accent-color"],
        },
        SetterSpec {
            name: "objectPosition",
            vocab: "object",
            scale: None,
            keywords: &["center", "top", "right", "bottom", "left"],
            negatable: false,
            slots: &["object-position"],
        },
        SetterSpec {
            name: "order",
            vocab: "order",
            scale: None,
            keywords: &["first", "last", "none"],
            negatable: false,
            slots: &["order"],
        },
        SetterSpec {
            name: "justifySelf",
            vocab: "justify-self",
            scale: None,
            keywords: &["auto", "start", "center", "end", "stretch"],
            negatable: false,
            slots: &["justify-self"],
        },
        SetterSpec {
            name: "bgClip",
            vocab: "bg-clip",
            scale: None,
            keywords: &["border", "padding", "content", "text"],
            negatable: false,
            slots: &["background-clip"],
        },
        SetterSpec {
            name: "touch",
            vocab: "touch",
            scale: None,
            keywords: &[
                "auto",
                "none",
                "pan-x",
                "pan-y",
                "pinch-zoom",
                "manipulation",
            ],
            negatable: false,
            slots: &["touch-action"],
        },
        SetterSpec {
            name: "resize",
            vocab: "resize",
            scale: None,
            keywords: &["none", "y", "x", "both"],
            negatable: false,
            slots: &["resize"],
        },
        SetterSpec {
            name: "scroll",
            vocab: "scroll",
            scale: None,
            keywords: &["auto", "smooth"],
            negatable: false,
            slots: &["scroll-behavior"],
        },
        SetterSpec {
            name: "willChange",
            vocab: "will-change",
            scale: None,
            keywords: &["auto", "scroll-position", "contents", "transform"],
            negatable: false,
            slots: &["will-change"],
        },
        SetterSpec {
            name: "contain",
            vocab: "contain",
            scale: None,
            keywords: &["none", "content", "strict", "size", "layout", "paint"],
            negatable: false,
            slots: &["contain"],
        },
        // Host-paint gap for flex/grid containers outside layout keywords (button base box, etc.).
        SetterSpec {
            name: "gap",
            vocab: "gap",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["gap"],
        },
        SetterSpec {
            name: "gapX",
            vocab: "gap-x",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["column-gap"],
        },
        SetterSpec {
            name: "gapY",
            vocab: "gap-y",
            scale: Some("space"),
            keywords: &["0"],
            negatable: false,
            slots: &["row-gap"],
        },
        // Hover-deepen primitive: color-mix toward foreground by percent.
        SetterSpec {
            name: "colorDeepen",
            vocab: "color-deepen",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &["background-color"],
        },
        // Soft wash primitive: color-mix hue into transparent by percent
        // (restores the deleted tone-*-bg ramp without minting design tokens).
        SetterSpec {
            name: "colorMix",
            vocab: "color-mix",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &["background-color"],
        },
        // Tier-2 control chrome channels.
        SetterSpec {
            name: "controlFrame",
            vocab: "control-frame",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &["box-shadow"],
        },
        SetterSpec {
            name: "controlLift",
            vocab: "control-lift",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &["translate"],
        },
        SetterSpec {
            name: "press",
            vocab: "press",
            scale: None,
            keywords: &[],
            negatable: false,
            slots: &["translate", "scale", "box-shadow"],
        },
    ]
}

pub fn find_setter(name: &str) -> Option<&'static SetterSpec> {
    setters().iter().find(|s| s.name == name)
}

/// Unique closest product setter for typo correction (confluent only).
/// Returns `None` when no candidate is close enough or two candidates tie.
///
/// Ranking: lower Levenshtein first; then prefer names that are a prefix of the
/// typo (or vice versa) so `padd` → `pad` beats `padX` at the same distance;
/// then shorter absolute length delta. Exact ties remain non-confluent.
pub fn closest_unique_setter(typo: &str) -> Option<&'static str> {
    let typo_l = typo.to_ascii_lowercase();
    // score: (distance, prefix_penalty 0|1, len_delta)
    let mut best: Option<(usize, u8, usize, &'static str)> = None;
    let mut tied = false;
    for s in setters() {
        let name = s.name;
        let name_l = name.to_ascii_lowercase();
        let cap = std::cmp::max(2, name.len() / 2 + 1);
        let d = setter_levenshtein(&typo_l, &name_l, cap + 1);
        if d > cap || d == 0 {
            continue;
        }
        let prefix_ok = typo_l.starts_with(&name_l) || name_l.starts_with(&typo_l);
        let prefix_penalty: u8 = if prefix_ok { 0 } else { 1 };
        let len_delta = typo_l.len().abs_diff(name_l.len());
        let score = (d, prefix_penalty, len_delta, name);
        match best {
            None => {
                best = Some(score);
                tied = false;
            }
            Some(b) => {
                let b_key = (b.0, b.1, b.2);
                let s_key = (score.0, score.1, score.2);
                if s_key < b_key {
                    best = Some(score);
                    tied = false;
                } else if s_key == b_key {
                    tied = true;
                }
            }
        }
    }
    if tied {
        return None;
    }
    best.map(|(_, _, _, n)| n)
}

fn setter_levenshtein(a: &str, b: &str, cap: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len().min(cap);
    }
    if b.is_empty() {
        return a.len().min(cap);
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        let mut row_min = curr[0];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            if curr[j] < row_min {
                row_min = curr[j];
            }
        }
        if row_min > cap {
            return cap;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()].min(cap)
}

/// Paint slots THIS op writes — narrowed from the setter's complete
/// [`SetterSpec::slots`] union by arg value. `border` / `outline` / `ring` /
/// `bg` disambiguate color vs width vs style (vs image) by value; every
/// other setter writes its full slot list regardless of arg. Clash analysis
/// keys on this, so `style.outline(solid).outline(2).outline(ring)` is three
/// disjoint writes, not a triple clash.
pub fn op_slots(op: &PaintOp) -> Vec<&'static str> {
    let Some(spec) = find_setter(&op.setter) else {
        return Vec::new();
    };
    // Composite border: position 0 = width slots, position 1 = color slots.
    if op.args.len() == 2 && is_border_composite(spec.name) {
        return slots_part(spec, "width")
            .into_iter()
            .chain(slots_part(spec, "color"))
            .collect();
    }
    if is_border_composite(spec.name) {
        return match op.first_arg() {
            // bind / bound group paths resolve through the color entry.
            PaintArg::Bind(_) | PaintArg::MemberPath { .. } => slots_part(spec, "color"),
            PaintArg::Token(tok) => {
                let bare = strip_neg(tok).1;
                if scales::contains("border-width", bare)
                    || (spec.name == "border" && bare == "control-stroke")
                {
                    slots_part(spec, "width")
                } else if matches!(
                    bare,
                    "solid" | "dashed" | "dotted" | "double" | "hidden" | "none"
                ) {
                    slots_part(spec, "style")
                } else {
                    slots_part(spec, "color")
                }
            }
        };
    }
    match spec.name {
        "outline" => {
            return match op.first_arg() {
                PaintArg::Bind(_) | PaintArg::MemberPath { .. } => vec!["outline-color"],
                PaintArg::Token(tok) => {
                    let bare = strip_neg(tok).1;
                    if bare == "ring-width" || scales::contains("border-width", bare) {
                        vec!["outline-width"]
                    } else if matches!(bare, "none" | "solid" | "dashed" | "dotted" | "double") {
                        vec!["outline-style"]
                    } else {
                        vec!["outline-color"]
                    }
                }
            };
        }
        "ring" => {
            return match op.first_arg() {
                PaintArg::Token(tok) if strip_neg(tok).1 == "none" => vec!["outline-style"],
                _ => vec!["outline-color"],
            };
        }
        "bg" => {
            return match op.first_arg() {
                PaintArg::Token(tok) if strip_neg(tok).1 == "none" => vec!["background-image"],
                _ => vec!["background-color"],
            };
        }
        _ => {}
    }
    spec.slots.to_vec()
}

/// The subset of a setter's slots matching one part (`width` / `color` /
/// `style`) — slot longhands carry the part name (`border-left-color`).
fn slots_part(spec: &SetterSpec, part: &str) -> Vec<&'static str> {
    spec.slots
        .iter()
        .copied()
        .filter(|s| s.contains(part))
        .collect()
}

/// Resolved `when({ key })` target: pseudo/attr on self, or ancestor prefix.
///
/// `Cow` so closed static keys stay zero-alloc while dynamic `state-*` /
/// `group-*` families (same grammar as string-sx prefixes) can own selectors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhenTarget {
    /// Appended to the element selector (`:hover`, `[aria-selected='true']`, …).
    Pseudo(Cow<'static, str>),
    /// Prepended ancestor context (`.sail-grp[required] `).
    Ancestor(Cow<'static, str>),
    /// Media condition wrapping the rule.
    Media(Cow<'static, str>),
}

fn pseudo_static(s: &'static str) -> WhenTarget {
    WhenTarget::Pseudo(Cow::Borrowed(s))
}
fn ancestor_static(s: &'static str) -> WhenTarget {
    WhenTarget::Ancestor(Cow::Borrowed(s))
}
fn media_static(s: &'static str) -> WhenTarget {
    WhenTarget::Media(Cow::Borrowed(s))
}

/// Normalize camelCase `when` keys to the kebab names used by string-sx
/// condition prefixes (`motionReduce` → `motion-reduce`).
fn when_key_kebab(key: &str) -> String {
    let mut out = String::with_capacity(key.len() + 4);
    for (i, c) in key.chars().enumerate() {
        if c.is_ascii_uppercase() {
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

/// Closed `when({ key: Style })` browser/framework-state and media keys.
///
/// Breakpoints and preference media come from [`super::conditions`] — the
/// Style product table, locked to design-token `breakpoint-*` lengths.
///
/// Dynamic families mirror string-sx prefixes: `state-<value>`, `group-<state>`,
/// plus ARIA keys (`busy`, `sortable`) and camelCase product spellings.
pub fn when_target(key: &str) -> Option<WhenTarget> {
    match key {
        // Hover/active/focus never fire while disabled — otherwise disabled
        // buttons still lift, wash, and press-scale on mouseover.
        "hover" => Some(pseudo_static(
            ":hover:not(:disabled):not([aria-disabled='true'])",
        )),
        "focus" => Some(pseudo_static(
            ":focus-visible:not(:disabled):not([aria-disabled='true'])",
        )),
        "focusWithin" | "focus-within" => Some(pseudo_static(":focus-within")),
        "active" => Some(pseudo_static(
            ":active:not(:disabled):not([aria-disabled='true'])",
        )),
        "disabled" => Some(pseudo_static(":is(:disabled,[aria-disabled='true'])")),
        "empty" => Some(pseudo_static(":empty")),
        "before" => Some(pseudo_static("::before")),
        "after" => Some(pseudo_static("::after")),
        "first" | "first-child" => Some(pseudo_static(":first-child")),
        "last" | "last-child" => Some(pseudo_static(":last-child")),
        "selected" => Some(pseudo_static("[aria-selected='true']")),
        "expanded" => Some(pseudo_static("[aria-expanded='true']")),
        // Native checkbox/switch + ARIA — both forms appear in host skins.
        "checked" => Some(pseudo_static(":is(:checked,[aria-checked='true'])")),
        "pressed" => Some(pseudo_static("[aria-pressed='true']")),
        "open" => Some(pseudo_static("[open]")),
        "current" => Some(pseudo_static("[aria-current]:not([aria-current='false'])")),
        "busy" => Some(pseudo_static("[aria-busy='true']")),
        "sortable" => Some(pseudo_static("[aria-sort]")),
        // Component data-state tokens (`data-state="live"`) — self, not group host.
        "state-live" | "stateLive" => Some(pseudo_static("[data-state~='live']")),
        "group-current" | "groupCurrent" => Some(ancestor_static(
            ".sail-grp[aria-current]:not([aria-current='false']) ",
        )),
        "group-required" | "groupRequired" => Some(ancestor_static(".sail-grp[required] ")),
        "group-invalid" | "groupInvalid" => Some(ancestor_static(".sail-grp[invalid] ")),
        "group-disabled" | "groupDisabled" => Some(ancestor_static(
            ".sail-grp:is(:disabled,[aria-disabled='true'],[disabled]) ",
        )),
        "group-hover" | "groupHover" => Some(ancestor_static(".sail-grp:hover ")),
        "group-focus-within" | "groupFocusWithin" => {
            Some(ancestor_static(".sail-grp:focus-within "))
        }
        "group-dismissible" | "groupDismissible" => {
            Some(ancestor_static(".sail-grp[dismissible] "))
        }
        "group-removable" | "groupRemovable" => Some(ancestor_static(".sail-grp[removable] ")),
        "group-open" | "groupOpen" => Some(ancestor_static(".sail-grp[open] ")),
        "group-state-has-details" | "groupStateHasDetails" => {
            Some(ancestor_static(".sail-grp[data-state~='has-details'] "))
        }
        "group-state-open" | "groupStateOpen" => {
            Some(ancestor_static(".sail-grp[data-state~='open'] "))
        }
        // error-boundary catchRender stamps data-caught on the group host.
        "group-caught" | "groupCaught" => Some(ancestor_static(".sail-grp[data-caught] ")),
        // theme-picker `inline` presence attr on the group host.
        "group-inline" | "groupInline" => Some(ancestor_static(".sail-grp[inline] ")),
        // <file variant="drop"> — hide the native picker, show the hint.
        "group-drop" | "groupDrop" => Some(ancestor_static(".sail-grp[variant='drop'] ")),
        other => {
            let kebab = when_key_kebab(other);
            if let Some(m) = super::conditions::condition_media(other)
                .or_else(|| super::conditions::condition_media(&kebab))
            {
                return Some(media_static(m));
            }
            // ARIA / pseudo from the closed string-sx STATE_PREFIXES table.
            if let Some(p) =
                sx_rules::state_pseudo(other).or_else(|| sx_rules::state_pseudo(&kebab))
            {
                return Some(WhenTarget::Pseudo(Cow::Borrowed(p)));
            }
            // Component data-state: state-weekend / stateWeekend → [data-state~='weekend']
            if let Some(sel) = sx_rules::state_data(other).or_else(|| sx_rules::state_data(&kebab))
            {
                return Some(WhenTarget::Pseudo(Cow::Owned(sel)));
            }
            // group-* ancestor family (group-selected, group-state-user, groupSelected, …)
            let kebab_full = when_key_kebab(other);
            if let Some(base) = kebab_full.strip_prefix("group-") {
                if let Some(p) = sx_rules::state_pseudo(base) {
                    return Some(WhenTarget::Ancestor(Cow::Owned(format!(".sail-grp{p} "))));
                }
                if let Some(p) = sx_rules::state_data(base) {
                    return Some(WhenTarget::Ancestor(Cow::Owned(format!(".sail-grp{p} "))));
                }
                // bare data-state value on group host: group-user → data-state~=user
                if let Some(p) = sx_rules::state_data(&format!("state-{base}")) {
                    return Some(WhenTarget::Ancestor(Cow::Owned(format!(".sail-grp{p} "))));
                }
                if let Some(n) = sx_rules::group_attr_presence(base) {
                    return Some(WhenTarget::Ancestor(Cow::Owned(format!(".sail-grp[{n}] "))));
                }
            }
            None
        }
    }
}

/// Back-compat helper used by emit for simple pseudo append.
pub fn when_selector(key: &str) -> Option<String> {
    match when_target(key)? {
        WhenTarget::Pseudo(s) => Some(s.into_owned()),
        _ => None,
    }
}

/// Validate paint arg against setter table; fail closed.
pub fn validate_op(op: &PaintOp) -> Result<(), StyleDiagnostic> {
    let Some(spec) = find_setter(&op.setter) else {
        return Err(StyleDiagnostic::UnknownSetter {
            setter: op.setter.clone(),
            source: op.source.clone(),
            suggestion: closest_unique_setter(&op.setter).map(|s| s.to_string()),
        });
    };
    // Marker setters with empty arg (group, scrollbarThin, isolate, truncate, srOnly,
    // control chrome, press).
    if matches!(
        spec.name,
        "group"
            | "scrollbarThin"
            | "isolate"
            | "truncate"
            | "srOnly"
            | "controlFrame"
            | "controlLift"
            | "press"
    ) {
        return Ok(());
    }
    // Composite positional setters: style.border(1, border) = width + color.
    if op.args.len() > 1 {
        return validate_composite_op(spec, op);
    }
    match op.first_arg() {
        PaintArg::Bind(_) => Ok(()),
        // Bound paths validated at parse; emit expands before lower.
        PaintArg::MemberPath { .. } => Ok(()),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg && !spec.negatable {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
            // SPEC-D1: component surface public names are emit labels only —
            // not Style author Tokens. Host units use dotted MemberPath.
            if spec.scale == Some("color")
                && crate::component_surfaces::is_component_public_name(bare)
            {
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
                if scale == "color"
                    || scale == "font"
                    || scale == "weight"
                    || scale == "text"
                    || scale == "duration"
                    || scale == "ease"
                    || scale == "space"
                    || scale == "size"
                    || scale == "z"
                    || scale == "shadow"
                    || scale == "leading"
                    || scale == "tracking"
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
            suggestion: closest_unique_setter(&op.setter).map(|s| s.to_string()),
        });
    };
    // group: host marker only — no CSS declarations.
    if spec.name == "group" {
        return Ok(Vec::new());
    }
    // Phase 3 gap keys — special-case CSS (filter longhands, clip/mask, 3D, cq).
    if let Some(pairs) = lower_gap_key_css(spec.name, op) {
        return pairs;
    }
    // isolation stacking root.
    if spec.name == "isolate" {
        return Ok(vec![("isolation".into(), "isolate".into())]);
    }
    // scrollbar-thin bare composite (width + color vars).
    if spec.name == "scrollbarThin" {
        if let Resolved::Ok(pairs) = style_vocab::resolve("scrollbar-thin") {
            return Ok(pairs);
        }
        return Ok(vec![
            ("scrollbar-width".into(), "thin".into()),
            (
                "scrollbar-color".into(),
                "var(--sx-scrollbar-thumb, var(--border)) var(--sx-scrollbar-track, transparent)"
                    .into(),
            ),
        ]);
    }
    // truncate bare composite.
    if spec.name == "truncate" {
        if let Resolved::Ok(pairs) = style_vocab::resolve("truncate") {
            return Ok(pairs);
        }
        return Ok(vec![
            ("overflow".into(), "hidden".into()),
            ("text-overflow".into(), "ellipsis".into()),
            ("white-space".into(), "nowrap".into()),
        ]);
    }
    // sr-only a11y clip pack.
    if spec.name == "srOnly" {
        if let Resolved::Ok(pairs) = style_vocab::resolve("sr-only") {
            return Ok(pairs);
        }
        return Ok(vec![
            ("position".into(), "absolute".into()),
            ("width".into(), "1px".into()),
            ("height".into(), "1px".into()),
            ("padding".into(), "0".into()),
            ("margin".into(), "-1px".into()),
            ("overflow".into(), "hidden".into()),
            ("clip-path".into(), "inset(50%)".into()),
            ("white-space".into(), "nowrap".into()),
            ("border-width".into(), "0".into()),
        ]);
    }
    // Composite border: width + color in one call (style.border(1, border)).
    if op.args.len() > 1 {
        if op.args.len() != 2 {
            return Err(StyleDiagnostic::BadArity {
                setter: op.setter.clone(),
                got: op.args.len(),
                source: op.source.clone(),
            });
        }
        if is_border_composite(spec.name) {
            let mut out = lower_border_width_arg(spec, op, &op.args[0])?;
            out.extend(lower_border_color_arg(spec, op, &op.args[1])?);
            return Ok(out);
        }
        if spec.name == "colorDeepen" {
            return lower_color_deepen_arg(spec, op, &op.args[0], &op.args[1]);
        }
        if spec.name == "colorMix" {
            return lower_color_mix_wash_arg(spec, op, &op.args[0], &op.args[1]);
        }
        return Err(StyleDiagnostic::BadArity {
            setter: op.setter.clone(),
            got: op.args.len(),
            source: op.source.clone(),
        });
    }
    let arg = op.first_arg();
    // visibility keywords.
    if spec.name == "visibility" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css = match bare_t {
                "hidden" => "hidden",
                "visible" => "visible",
                "collapse" => "collapse",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            return Ok(vec![("visibility".into(), css.into())]);
        }
    }
    // aspect-ratio map.
    if spec.name == "aspect" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let atom = format!("aspect-{bare_t}");
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            let css = match bare_t {
                "square" => "1 / 1",
                "video" => "16 / 9",
                "auto" => "auto",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            return Ok(vec![("aspect-ratio".into(), css.into())]);
        }
    }
    // object-fit.
    if spec.name == "objectFit" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if !spec.keywords.iter().any(|k| *k == bare_t) {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare_t.into(),
                    source: op.source.clone(),
                });
            }
            let atom = format!("object-{bare_t}");
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            return Ok(vec![("object-fit".into(), bare_t.into())]);
        }
    }
    // transform full value (token none or bind → var).
    if spec.name == "transform" {
        match arg {
            PaintArg::Token(t) => {
                let bare_t = strip_neg(t).1;
                if bare_t == "none" {
                    return Ok(vec![("transform".into(), "none".into())]);
                }
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare_t.into(),
                    source: op.source.clone(),
                });
            }
            PaintArg::Bind(name) => {
                let css = css_name_for_bind(name);
                return Ok(vec![("transform".into(), format!("var(--{css})"))]);
            }
            PaintArg::MemberPath { .. } => {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: arg.display(),
                    source: op.source.clone(),
                });
            }
        }
    }
    // italic / not-italic.
    if spec.name == "italic" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css = match bare_t {
                "true" | "on" => "italic",
                "false" | "off" => "normal",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            return Ok(vec![("font-style".into(), css.into())]);
        }
    }
    // text-transform via textCase(uppercase|…)
    if spec.name == "textCase" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css = match bare_t {
                "uppercase" => "uppercase",
                "lowercase" => "lowercase",
                "capitalize" => "capitalize",
                "normal" => "none",
                // Casing channel: label-style text reads --label-transform so
                // themes govern casing (a literal silently beats the theme).
                "label-transform" => "var(--label-transform)",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            return Ok(vec![("text-transform".into(), css.into())]);
        }
    }
    // font-variant-numeric via nums(tabular|…)
    if spec.name == "nums" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let atom = match bare_t {
                "tabular" => "tabular-nums",
                "lining" => "lining-nums",
                "oldstyle" => "oldstyle-nums",
                "proportional" => "proportional-nums",
                "ordinal" => "ordinal",
                "slashed-zero" => "slashed-zero",
                "normal" => "normal-nums",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            if let Resolved::Ok(pairs) = style_vocab::resolve(atom) {
                return Ok(pairs);
            }
            let css_val = match bare_t {
                "tabular" => "tabular-nums",
                "lining" => "lining-nums",
                "oldstyle" => "oldstyle-nums",
                "proportional" => "proportional-nums",
                "ordinal" => "ordinal",
                "slashed-zero" => "slashed-zero",
                "normal" => "normal",
                _ => bare_t,
            };
            return Ok(vec![("font-variant-numeric".into(), css_val.into())]);
        }
    }
    // transform-origin via origin(bottom|…)
    if spec.name == "origin" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let atom = format!("origin-{bare_t}");
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            let css_val = bare_t.replace('-', " ");
            return Ok(vec![("transform-origin".into(), css_val.into())]);
        }
    }
    // text-gradient-* composite
    if spec.name == "textGradient" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let atom = format!("text-gradient-{bare_t}");
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            if matches!(bare_t, "primary" | "accent") {
                return Ok(vec![
                    ("background".into(), format!("var(--gradient-{bare_t})")),
                    ("-webkit-background-clip".into(), "text".into()),
                    ("background-clip".into(), "text".into()),
                    ("color".into(), "transparent".into()),
                ]);
            }
        }
    }
    // bg-glow-* composite
    if spec.name == "bgGlow" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let atom = format!("bg-glow-{bare_t}");
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            if matches!(bare_t, "primary" | "accent") {
                return Ok(vec![
                    ("position".into(), "absolute".into()),
                    ("inset".into(), "-4rem 0 0".into()),
                    ("background".into(), format!("var(--glow-{bare_t})")),
                    ("pointer-events".into(), "none".into()),
                ]);
            }
        }
    }
    // Host-paint gap props for flex/grid containers outside layout keywords.
    if matches!(spec.name, "gap" | "gapX" | "gapY") {
        let value = match arg {
            PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
            PaintArg::Token(t) => {
                let bare = strip_neg(t).1;
                if bare == "0" {
                    "0".into()
                } else {
                    scales::lower("space", bare).ok_or_else(|| StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: t.clone(),
                        source: op.source.clone(),
                    })?
                }
            }
            PaintArg::MemberPath { .. } => {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: arg.display(),
                    source: op.source.clone(),
                });
            }
        };
        let prop = match spec.name {
            "gap" => "gap",
            "gapX" => "column-gap",
            "gapY" => "row-gap",
            _ => unreachable!(),
        };
        return Ok(vec![(prop.into(), value)]);
    }
    // Tier-2 control chrome / press marker channels.
    if spec.name == "controlFrame" {
        return Ok(vec![("box-shadow".into(), "var(--control-frame)".into())]);
    }
    if spec.name == "controlLift" {
        return Ok(vec![("translate".into(), "var(--control-lift)".into())]);
    }
    if spec.name == "press" {
        return Ok(vec![
            ("translate".into(), "var(--press-translate)".into()),
            ("scale".into(), "var(--press-scale)".into()),
            ("box-shadow".into(), "var(--press-shadow)".into()),
        ]);
    }
    let (value, bare, negative) = match arg {
        PaintArg::Token(t) => {
            let (neg, bare) = strip_neg(t);
            (Some(SxValue::Token(bare.to_string())), bare.is_empty(), neg)
        }
        PaintArg::Bind(name) => {
            let css = css_name_for_bind(name);
            return Ok(bind_css_props(spec, &css));
        }
        // Bound group paths (`tone`, `tone.foreground`) → bridge vars.
        // Host reflects `--tone: var(--primary)` from the color-group typed prop.
        PaintArg::MemberPath { root, path } => {
            let css = crate::token_groups::bridge_var_name(root, path);
            return Ok(bind_css_props(spec, &css));
        }
    };
    // Axis overflow / overscroll / text-align / tab-size / underline / list keywords
    if matches!(
        spec.name,
        "overflowX"
            | "overflowY"
            | "overscroll"
            | "textAlign"
            | "tab"
            | "underline"
            | "list"
            | "mixBlend"
    ) {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if !spec.keywords.iter().any(|k| *k == bare_t) {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare_t.into(),
                    source: op.source.clone(),
                });
            }
            let atom = format!("{}-{}", spec.vocab, bare_t);
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            // tab map is numeric; underline map may use bare prop+value.
            if let Some(pairs) = style_ir::lower_decl(&SxDecl {
                prefixes: Vec::new(),
                prop: spec.vocab.to_string(),
                value: Some(SxValue::Token(bare_t.into())),
                negative: false,
                bare: false,
                source: op.source.clone(),
            }) {
                return Ok(pairs);
            }
            let css_prop = match spec.name {
                "overflowX" => "overflow-x",
                "overflowY" => "overflow-y",
                "overscroll" => "overscroll-behavior",
                "textAlign" => "text-align",
                "tab" => "tab-size",
                "underline" => "text-decoration-line",
                "list" => "list-style-type",
                "mixBlend" => "mix-blend-mode",
                _ => spec.vocab,
            };
            let css_val = match (spec.name, bare_t) {
                ("underline", "true") => "underline",
                ("underline", "none") => "none",
                ("underline", "line-through") => "line-through",
                _ => bare_t,
            };
            return Ok(vec![(css_prop.into(), css_val.into())]);
        }
    }
    // Special-case display
    if spec.name == "display" {
        let tok = match arg {
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
                });
            }
        };
        return Ok(css);
    }
    // Special-case position bare keywords
    if spec.name == "position" {
        let tok = match arg {
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
        if let PaintArg::Token(t) = arg {
            if strip_neg(t).1 == "none" {
                return Ok(vec![("background-image".into(), "none".into())]);
            }
        }
    }
    // outlineOffset special tokens
    if spec.name == "outlineOffset" {
        if let PaintArg::Token(t) = arg {
            match strip_neg(t).1 {
                "ring-offset" => {
                    return Ok(vec![("outline-offset".into(), "var(--ring-offset)".into())]);
                }
                "ring-inset" => {
                    return Ok(vec![(
                        "outline-offset".into(),
                        "calc(-1 * var(--ring-width))".into(),
                    )]);
                }
                _ => {}
            }
        }
    }
    // outline color tokens (primary, tone, ring, tone-ring, …) vs width/style keywords
    if spec.name == "outline" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if bare_t == "ring-width" {
                return Ok(vec![("outline-width".into(), "var(--ring-width)".into())]);
            }
            if !matches!(
                bare_t,
                "none" | "solid" | "dashed" | "1" | "2" | "4" | "0" | "3" | "8"
            ) {
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
                    let atom = style_ir::decl_atom(&d);
                    if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                        return Ok(pairs);
                    }
                }
            }
        }
    }
    // Dimension / composite special cases
    if let PaintArg::Token(t) = arg {
        let bare_t = strip_neg(t).1;
        match (spec.name, bare_t) {
            ("borderWidth", "control-stroke") => {
                return Ok(vec![(
                    "border-width".into(),
                    "var(--control-stroke)".into(),
                )]);
            }
            // The hairline channel: control hairlines spend --border-width,
            // never a literal px (channel-discipline ratchet, baseline 0).
            ("borderWidth", "border-width") => {
                return Ok(vec![("border-width".into(), "var(--border-width)".into())]);
            }
            ("w" | "h", "border-width") => {
                let prop = if spec.name == "w" { "width" } else { "height" };
                return Ok(vec![(prop.into(), "var(--border-width)".into())]);
            }
            ("w", "panel-toast-region-width") => {
                return Ok(vec![("width".into(), "var(--panel-toast-region-width)".into())]);
            }
            // Closed product keyframe catalog (packages/sail/runtime
            // keyframes-catalog.ts). Style.animate(name) → CSS host Style var;
            // ambient @keyframes live in the generated contracts sheet,
            // not free CSS and not fx:loop. Keywords on the animate
            // SetterSpec must stay in lockstep with styleAnimate:true rows.
            ("animate", name) if matches!(name, "spin") => {
                return Ok(vec![("animation".into(), format!("var(--animate-{name})"))]);
            }
            ("bgImage", "none") => {
                return Ok(vec![("background-image".into(), "none".into())]);
            }
            ("borderImage", "none") => {
                return Ok(vec![("border-image".into(), "none".into())]);
            }
            ("borderImage", "corner-image") => {
                return Ok(vec![("border-image".into(), "var(--corner-image)".into())]);
            }
            ("backdrop", "none") => {
                return Ok(vec![
                    ("backdrop-filter".into(), "none".into()),
                    ("-webkit-backdrop-filter".into(), "none".into()),
                ]);
            }
            ("backdrop", "card-backdrop-filter") => {
                return Ok(vec![
                    (
                        "backdrop-filter".into(),
                        "var(--card-backdrop-filter)".into(),
                    ),
                    (
                        "-webkit-backdrop-filter".into(),
                        "var(--card-backdrop-filter)".into(),
                    ),
                ]);
            }
            ("bgImage", "primary-accent") => {
                return Ok(vec![(
                    "background-image".into(),
                    "linear-gradient(135deg, var(--primary), var(--accent))".into(),
                )]);
            }
            // Photo path default: cover fill (no letterbox). Icon-slug path is
            // a second rule in emit (`[icon-slug] img` → contain + pad).
            ("avatarMedia", "true" | "on") => {
                return Ok(vec![
                    ("display".into(), "block".into()),
                    ("width".into(), "100%".into()),
                    ("height".into(), "100%".into()),
                    ("object-fit".into(), "cover".into()),
                    ("box-sizing".into(), "border-box".into()),
                ]);
            }
            ("glyphFill", "true" | "on") => {
                return Ok(vec![
                    ("display".into(), "block".into()),
                    ("width".into(), "100%".into()),
                    ("height".into(), "100%".into()),
                ]);
            }
            ("w", "max") => {
                return Ok(vec![("width".into(), "max-content".into())]);
            }
            ("w", "panel-max-w-md") => {
                return Ok(vec![("width".into(), "var(--panel-max-w-md)".into())]);
            }
            ("w", "panel-max-w-sm") => {
                return Ok(vec![("width".into(), "var(--panel-max-w-sm)".into())]);
            }
            ("w", "panel-max-w-lg") => {
                return Ok(vec![("width".into(), "var(--panel-max-w-lg)".into())]);
            }
            ("w", "panel-max-w-xs") => {
                return Ok(vec![("width".into(), "var(--panel-max-w-xs)".into())]);
            }
            ("maxW", "panel-max-w-md") => {
                return Ok(vec![("max-width".into(), "var(--panel-max-w-md)".into())]);
            }
            ("maxW", "panel-max-w-sm") => {
                return Ok(vec![("max-width".into(), "var(--panel-max-w-sm)".into())]);
            }
            ("maxW", "panel-max-w-lg") => {
                return Ok(vec![("max-width".into(), "var(--panel-max-w-lg)".into())]);
            }
            ("maxW", "panel-max-w-xs") => {
                return Ok(vec![("max-width".into(), "var(--panel-max-w-xs)".into())]);
            }
            ("maxW", "panel-ch-w-md") => {
                return Ok(vec![("max-width".into(), "var(--panel-ch-w-md)".into())]);
            }
            ("maxW", "panel-ch-w-lg") => {
                return Ok(vec![("max-width".into(), "var(--panel-ch-w-lg)".into())]);
            }
            ("flexDir", dir @ ("row" | "column" | "row-reverse" | "column-reverse")) => {
                return Ok(vec![("flex-direction".into(), dir.into())]);
            }
            ("minW", "panel-menu-min-w") => {
                return Ok(vec![("min-width".into(), "var(--panel-menu-min-w)".into())]);
            }
            ("minW", "panel-rail-w") => {
                return Ok(vec![("min-width".into(), "var(--panel-rail-w)".into())]);
            }
            ("textAlign", align @ ("left" | "center" | "right" | "justify" | "start" | "end")) => {
                return Ok(vec![("text-align".into(), align.into())]);
            }
            ("break", "words") => {
                return Ok(vec![
                    ("overflow-wrap".into(), "anywhere".into()),
                    ("word-break".into(), "normal".into()),
                ]);
            }
            ("break", "all") => {
                return Ok(vec![
                    ("overflow-wrap".into(), "normal".into()),
                    ("word-break".into(), "break-all".into()),
                ]);
            }
            ("break", "keep") => {
                return Ok(vec![
                    ("overflow-wrap".into(), "normal".into()),
                    ("word-break".into(), "keep-all".into()),
                ]);
            }
            ("break", "normal") => {
                return Ok(vec![
                    ("overflow-wrap".into(), "normal".into()),
                    ("word-break".into(), "normal".into()),
                ]);
            }
            ("stroke", "current" | "currentColor") => {
                return Ok(vec![("stroke".into(), "currentColor".into())]);
            }
            ("stroke", "none") => {
                return Ok(vec![("stroke".into(), "none".into())]);
            }
            ("fill", "current" | "currentColor") => {
                return Ok(vec![("fill".into(), "currentColor".into())]);
            }
            ("fill", "none") => {
                return Ok(vec![("fill".into(), "none".into())]);
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
        if let PaintArg::Token(t) = arg {
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
                        if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                            return Ok(pairs);
                        }
                    }
                    if let Resolved::Ok(pairs) = style_vocab::resolve(bare_atom) {
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
            let atom = style_ir::decl_atom(&d);
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            let atom2 = if negative {
                format!("-{}-{}", spec.vocab, cand)
            } else {
                format!("{}-{}", spec.vocab, cand)
            };
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom2) {
                return Ok(pairs);
            }
        }
    }
    match style_ir::lower_decl(&decl) {
        Some(pairs) => Ok(pairs),
        None => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
    }
}

/// Clip / filter / 3D / container-query product setters → real CSS.
fn lower_gap_key_css(
    name: &str,
    op: &PaintOp,
) -> Option<Result<Vec<(String, String)>, StyleDiagnostic>> {
    let arg = op.first_arg();
    let (neg, bare) = match arg {
        PaintArg::Token(t) => strip_neg(t),
        PaintArg::Bind(n) => {
            let var = format!("var(--{})", n.replace('_', "-"));
            return Some(Ok(match name {
                "clipPath" => vec![("clip-path".into(), var)],
                "mask" | "maskImage" => vec![("mask-image".into(), var)],
                "blur" | "brightness" | "contrast" | "grayscale" | "hueRotate" | "invert"
                | "sepia" | "saturate" => {
                    let fn_name = match name {
                        "hueRotate" => "hue-rotate",
                        other => other,
                    };
                    vec![("filter".into(), format!("{fn_name}({var})"))]
                }
                "perspective" => vec![("perspective".into(), var)],
                "rotateX" => vec![("transform".into(), format!("rotateX({var})"))],
                "rotateY" => vec![("transform".into(), format!("rotateY({var})"))],
                "rotateZ" => vec![("transform".into(), format!("rotateZ({var})"))],
                "containerType" | "container" => vec![("container-type".into(), var)],
                "containerName" => vec![("container-name".into(), var)],
                _ => return None,
            }));
        }
        PaintArg::MemberPath { root, path } => {
            // Only gap-key setters consume this arm; color setters fall through
            // to bridge-var lowering in lower_op_css.
            let gap_keys = matches!(
                name,
                "clipPath"
                    | "mask"
                    | "maskImage"
                    | "blur"
                    | "brightness"
                    | "contrast"
                    | "grayscale"
                    | "hueRotate"
                    | "invert"
                    | "sepia"
                    | "saturate"
                    | "perspective"
                    | "rotateX"
                    | "rotateY"
                    | "rotateZ"
                    | "containerType"
                    | "container"
                    | "containerName"
            );
            if !gap_keys {
                return None;
            }
            let var = crate::token_groups::bridge_var_ref(root, path);
            return Some(Ok(match name {
                "clipPath" => vec![("clip-path".into(), var)],
                "mask" | "maskImage" => vec![("mask-image".into(), var)],
                "blur" | "brightness" | "contrast" | "grayscale" | "hueRotate" | "invert"
                | "sepia" | "saturate" => {
                    let fn_name = match name {
                        "hueRotate" => "hue-rotate",
                        other => other,
                    };
                    vec![("filter".into(), format!("{fn_name}({var})"))]
                }
                "perspective" => vec![("perspective".into(), var)],
                "rotateX" => vec![("transform".into(), format!("rotateX({var})"))],
                "rotateY" => vec![("transform".into(), format!("rotateY({var})"))],
                "rotateZ" => vec![("transform".into(), format!("rotateZ({var})"))],
                "containerType" | "container" => vec![("container-type".into(), var)],
                "containerName" => vec![("container-name".into(), var)],
                _ => return None,
            }));
        }
    };
    let sign = if neg { "-" } else { "" };
    Some(Ok(match name {
        "clipPath" => {
            let css = match bare {
                "none" => "none".into(),
                "circle" => "circle()".into(),
                "ellipse" => "ellipse()".into(),
                "inset-0" => "inset(0)".into(),
                "border-box" | "padding-box" | "content-box" | "fill-box" | "stroke-box"
                | "view-box" | "margin-box" => bare.into(),
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("clip-path".into(), css)]
        }
        "mask" | "maskImage" => {
            let css = match bare {
                "none" => "none".into(),
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("mask-image".into(), css)]
        }
        "blur" => {
            let px = match bare {
                "0" | "1" | "2" | "4" | "8" | "12" | "16" | "24" => bare,
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("filter".into(), format!("blur({px}px)"))]
        }
        "brightness" | "contrast" | "saturate" => {
            let pct = match bare {
                "0" | "50" | "60" | "65" | "75" | "90" | "95" | "100" | "105" | "110" | "125"
                | "150" | "200" => bare,
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("filter".into(), format!("{name}({pct}%)"))]
        }
        "grayscale" | "invert" | "sepia" => {
            let pct = match bare {
                "0" | "50" | "100" => bare,
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("filter".into(), format!("{name}({pct}%)"))]
        }
        "hueRotate" => {
            let deg = match bare {
                "0" | "15" | "30" | "60" | "90" | "180" => bare,
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("filter".into(), format!("hue-rotate({sign}{deg}deg)"))]
        }
        "perspective" => {
            let css = match bare {
                "none" => "none".into(),
                "0" => "0px".into(),
                "250" | "500" | "750" | "1000" | "1500" | "2000" => format!("{bare}px"),
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("perspective".into(), css)]
        }
        "rotateX" | "rotateY" | "rotateZ" => {
            let axis = match name {
                "rotateX" => "X",
                "rotateY" => "Y",
                _ => "Z",
            };
            // Angle scale tokens (e.g. 45 → 45deg) or 0.
            let deg = if bare == "0" {
                "0deg".into()
            } else if bare.chars().all(|c| c.is_ascii_digit()) {
                format!("{sign}{bare}deg")
            } else if let Some(v) = scales::lower("angle", bare) {
                if neg {
                    format!("calc(-1 * {v})")
                } else {
                    v
                }
            } else {
                return Some(Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare.into(),
                    source: op.source.clone(),
                }));
            };
            vec![("transform".into(), format!("rotate{axis}({deg})"))]
        }
        "containerType" | "container" => {
            let css = match bare {
                "normal" | "size" | "inline-size" => bare.into(),
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("container-type".into(), css)]
        }
        "containerName" => {
            let css = match bare {
                "none" => "none".into(),
                // Ident-like custom names (closed enough: [a-z0-9-]+).
                t if t
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                    && !t.is_empty() =>
                {
                    t.into()
                }
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("container-name".into(), css)]
        }
        _ => return None,
    }))
}

/// Setters that take two positional args: (width-token?, color-token).
/// One arg stays the current color/width/style-by-value behavior.
fn is_border_composite(name: &str) -> bool {
    matches!(
        name,
        "border"
            | "borderX"
            | "borderY"
            | "borderT"
            | "borderB"
            | "borderL"
            | "borderR"
            | "borderS"
            | "borderE"
    )
}

/// Validate a composite (two-arg) paint op positionally.
fn validate_composite_op(spec: &SetterSpec, op: &PaintOp) -> Result<(), StyleDiagnostic> {
    if op.args.len() != 2 {
        return Err(StyleDiagnostic::BadArity {
            setter: op.setter.clone(),
            got: op.args.len(),
            source: op.source.clone(),
        });
    }
    if is_border_composite(spec.name) {
        validate_border_width_arg(spec, op, &op.args[0])?;
        validate_border_color_arg(spec, op, &op.args[1])?;
        return Ok(());
    }
    if spec.name == "colorDeepen" || spec.name == "colorMix" {
        // Same arity/token rules: color token + closed percent ramp.
        return validate_color_deepen_args(spec, op);
    }
    Err(StyleDiagnostic::BadArity {
        setter: op.setter.clone(),
        got: op.args.len(),
        source: op.source.clone(),
    })
}

/// Position 0 of a composite border call: a width token on the closed
/// border-width ramp (or `control-stroke` on plain `border`, mirroring
/// `borderWidth`).
fn validate_border_width_arg(
    spec: &SetterSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<(), StyleDiagnostic> {
    match arg {
        PaintArg::Bind(_) => Ok(()),
        PaintArg::MemberPath { .. } => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if !neg
                && (scales::contains("border-width", bare)
                    || (spec.name == "border" && bare == "control-stroke"))
            {
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

/// Position 1 of a composite border call: a color token / keyword. The
/// resolved declarations must all be `*-color` (a width or style token in
/// this position is a positional error, fail closed).
fn validate_border_color_arg(
    spec: &SetterSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<(), StyleDiagnostic> {
    match arg {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => Ok(()),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if !neg
                && (matches!(bare, "transparent" | "current" | "currentColor")
                    || scales::contains("color", bare))
            {
                return Ok(());
            }
            if !neg {
                for cand in token_candidates(bare) {
                    let atom = format!("{}-{}", spec.vocab, cand);
                    if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                        if pairs.iter().all(|(p, _)| p.contains("color")) {
                            return Ok(());
                        }
                    }
                }
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// CSS longhands a border-family setter writes for one part (`width` /
/// `color` / `style`).
fn border_props(name: &str, part: &str) -> Vec<String> {
    let bases: &[&str] = match name {
        "borderX" => &["border-left", "border-right"],
        "borderY" => &["border-top", "border-bottom"],
        "borderT" => &["border-top"],
        "borderB" => &["border-bottom"],
        "borderL" => &["border-left"],
        "borderR" => &["border-right"],
        "borderS" => &["border-inline-start"],
        "borderE" => &["border-inline-end"],
        _ => &["border"],
    };
    bases.iter().map(|b| format!("{b}-{part}")).collect()
}

/// Lower composite position 0 (width) to declarations.
fn lower_border_width_arg(
    spec: &SetterSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    match arg {
        PaintArg::Bind(name) => {
            let var = format!("var(--{})", css_name_for_bind(name));
            Ok(border_props(spec.name, "width")
                .into_iter()
                .map(|p| (p, var.clone()))
                .collect())
        }
        PaintArg::MemberPath { .. } => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let bare = strip_neg(tok).1;
            if spec.name == "border" && bare == "control-stroke" {
                return Ok(vec![(
                    "border-width".into(),
                    "var(--control-stroke)".into(),
                )]);
            }
            let atom = format!("{}-{}", spec.vocab, bare);
            if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                return Ok(pairs);
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Lower composite position 1 (color) to declarations.
fn lower_border_color_arg(
    spec: &SetterSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    match arg {
        PaintArg::Bind(name) => {
            let var = format!("var(--{})", css_name_for_bind(name));
            Ok(border_props(spec.name, "color")
                .into_iter()
                .map(|p| (p, var.clone()))
                .collect())
        }
        PaintArg::MemberPath { root, path } => {
            let var = crate::token_groups::bridge_var_ref(root, path);
            Ok(border_props(spec.name, "color")
                .into_iter()
                .map(|p| (p, var.clone()))
                .collect())
        }
        PaintArg::Token(tok) => {
            let bare = strip_neg(tok).1;
            for cand in token_candidates(bare) {
                let atom = format!("{}-{}", spec.vocab, cand);
                if let Resolved::Ok(pairs) = style_vocab::resolve(&atom) {
                    if pairs.iter().all(|(p, _)| p.contains("color")) {
                        return Ok(pairs);
                    }
                }
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Validate `colorDeepen(token, pct)`: token on color scale, pct in 5..95 step 5.
fn validate_color_deepen_args(_spec: &SetterSpec, op: &PaintOp) -> Result<(), StyleDiagnostic> {
    // Position 0: color token or bind.
    match &op.args[0] {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => {}
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg
                || (!scales::contains("color", bare)
                    && !matches!(bare, "transparent" | "current" | "currentColor"))
            {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
        }
    }
    // Position 1: closed percent ramp 5..95 step 5, or bind.
    match &op.args[1] {
        PaintArg::Bind(_) => {}
        PaintArg::MemberPath { .. } => {
            return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: op.args[1].display(),
                source: op.source.clone(),
            });
        }
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
            let ok = bare
                .parse::<i32>()
                .map_or(false, |n| n >= 5 && n <= 95 && n % 5 == 0);
            if !ok {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Lower `colorDeepen(token, pct)` to a color-mix background-color.
fn lower_color_deepen_arg(
    _spec: &SetterSpec,
    _op: &PaintOp,
    token_arg: &PaintArg,
    pct_arg: &PaintArg,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let token = match token_arg {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::Token(t) => {
            let bare = strip_neg(t).1;
            match bare {
                "transparent" | "current" | "currentColor" => bare.into(),
                _ => format!("var(--{bare})"),
            }
        }
        PaintArg::MemberPath { root, path } => {
            crate::token_groups::bridge_var_ref(root, path)
        }
    };
    let pct = match pct_arg {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::Token(t) => t.clone(),
        PaintArg::MemberPath { .. } => {
            return Err(StyleDiagnostic::BadToken {
                setter: "colorDeepen".into(),
                value: pct_arg.display(),
                source: _op.source.clone(),
            });
        }
    };
    Ok(vec![(
        "background-color".into(),
        format!("color-mix(in oklch, {token}, var(--foreground) {pct}%)"),
    )])
}

/// Lower `colorMix(token, pct)` to a transparent hue wash background-color.
/// Emits `color-mix(in oklch, {token} {pct}%, transparent)` — the former
/// tone-*-bg shape, without reintroducing tone-* design tokens.
fn lower_color_mix_wash_arg(
    _spec: &SetterSpec,
    _op: &PaintOp,
    token_arg: &PaintArg,
    pct_arg: &PaintArg,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let token = match token_arg {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::Token(t) => {
            let bare = strip_neg(t).1;
            match bare {
                "transparent" | "current" | "currentColor" => bare.into(),
                _ => format!("var(--{bare})"),
            }
        }
        PaintArg::MemberPath { root, path } => {
            crate::token_groups::bridge_var_ref(root, path)
        }
    };
    let pct = match pct_arg {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::Token(t) => t.clone(),
        PaintArg::MemberPath { .. } => {
            return Err(StyleDiagnostic::BadToken {
                setter: "colorMix".into(),
                value: pct_arg.display(),
                source: _op.source.clone(),
            });
        }
    };
    Ok(vec![(
        "background-color".into(),
        format!("color-mix(in oklch, {token} {pct}%, transparent)"),
    )])
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
        "ml" => return vec![("margin-left".into(), var)],
        "mr" => return vec![("margin-right".into(), var)],
        "mt" => return vec![("margin-top".into(), var)],
        "mb" => return vec![("margin-bottom".into(), var)],
        "pad" => return vec![("padding".into(), var)],
        "padX" => return vec![("padding-inline".into(), var)],
        "padY" => return vec![("padding-block".into(), var)],
        "padT" => return vec![("padding-top".into(), var)],
        "padB" => return vec![("padding-bottom".into(), var)],
        "padL" => return vec![("padding-left".into(), var)],
        "padR" => return vec![("padding-right".into(), var)],
        "padS" => return vec![("padding-inline-start".into(), var)],
        "padE" => return vec![("padding-inline-end".into(), var)],
        "w" => return vec![("width".into(), var)],
        "h" => return vec![("height".into(), var)],
        "minW" => return vec![("min-width".into(), var)],
        "minH" => return vec![("min-height".into(), var)],
        "maxW" => return vec![("max-width".into(), var)],
        "maxH" => return vec![("max-height".into(), var)],
        "z" => return vec![("z-index".into(), var)],
        "opacity" => return vec![("opacity".into(), var)],
        "mixBlend" => return vec![("mix-blend-mode".into(), var)],
        "fontSize" => return vec![("font-size".into(), var)],
        "fontWeight" => return vec![("font-weight".into(), var)],
        "fontFamily" => return vec![("font-family".into(), var)],
        "tracking" => return vec![("letter-spacing".into(), var)],
        "bg" => return vec![("background-color".into(), var)],
        "fg" | "color" => return vec![("color".into(), var)],
        "transform" => return vec![("transform".into(), var)],
        "translateY" => {
            return vec![(
                "translate".into(),
                format!("var(--sx-translate-x, 0) {var}"),
            )];
        }
        "translateX" => {
            return vec![(
                "translate".into(),
                format!("{var} var(--sx-translate-y, 0)"),
            )];
        }
        "rotate" => return vec![("rotate".into(), var)],
        "scale" => return vec![("scale".into(), var)],
        "basis" => return vec![("flex-basis".into(), var)],
        "decoration" => return vec![("text-decoration-color".into(), var)],
        "caret" => return vec![("caret-color".into(), var)],
        "accent" => return vec![("accent-color".into(), var)],
        "indent" => return vec![("text-indent".into(), var)],
        "underlineOffset" => return vec![("text-underline-offset".into(), var)],
        "antialiased" => return vec![("-webkit-font-smoothing".into(), var)],
        "objectPosition" => return vec![("object-position".into(), var)],
        "order" => return vec![("order".into(), var)],
        "justifySelf" => return vec![("justify-self".into(), var)],
        "bgClip" => return vec![("background-clip".into(), var)],
        "touch" => return vec![("touch-action".into(), var)],
        "resize" => return vec![("resize".into(), var)],
        "scroll" => return vec![("scroll-behavior".into(), var)],
        "willChange" => return vec![("will-change".into(), var)],
        "contain" => return vec![("contain".into(), var)],
        "gap" => return vec![("gap".into(), var)],
        "gapX" => return vec![("column-gap".into(), var)],
        "gapY" => return vec![("row-gap".into(), var)],
        "borderImage" => return vec![("border-image".into(), var)],
        "backdrop" => {
            return vec![
                ("backdrop-filter".into(), var.clone()),
                ("-webkit-backdrop-filter".into(), var),
            ];
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
    if let Some(pairs) = style_ir::lower_decl(&probe) {
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

#[cfg(test)]
mod tests {
    use super::super::ir::{Style, StyleDiagnostic};
    use super::*;

    /// Product parse (channel-only).
    fn parse(src: &str) -> (Style, Vec<StyleDiagnostic>) {
        super::super::parse::parse(src)
    }

    fn only_op(src: &str) -> PaintOp {
        let (style, diags) = parse(src);
        assert!(
            diags.is_empty(),
            "{src}: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        match style {
            Style::Set(op) => op,
            other => panic!("{src}: expected single Set, got {other:?}"),
        }
    }

    #[test]
    fn every_setter_declares_slots() {
        for s in setters() {
            if s.name == "group" {
                assert!(
                    s.slots.is_empty(),
                    "group is a host marker — no paint slots"
                );
            } else {
                assert!(!s.slots.is_empty(), "setter {} has no slots", s.name);
            }
        }
    }

    #[test]
    fn border_single_arg_unchanged() {
        let op = only_op("style({ paint: border(border) })");
        assert_eq!(op.args.len(), 1);
        let css = lower_op_css(&op).unwrap();
        assert_eq!(
            css,
            vec![("border-color".to_string(), "var(--border)".to_string())]
        );
        // width and style by value still work on one arg
        let op = only_op("style({ paint: border(2) })");
        assert!(
            lower_op_css(&op)
                .unwrap()
                .iter()
                .any(|(p, _)| p == "border-width")
        );
        let op = only_op("style({ paint: border(dashed) })");
        assert!(
            lower_op_css(&op)
                .unwrap()
                .iter()
                .any(|(p, _)| p == "border-style")
        );
    }

    #[test]
    fn border_composite_width_and_color() {
        let op = only_op("style({ paint: border(1, border) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-width".to_string(), "1px".to_string())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-color".to_string(), "var(--border)".to_string())),
            "{css:?}"
        );
    }

    #[test]
    fn border_axis_composites() {
        let op = only_op("style({ paint: borderX(2, primary) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-left-width".to_string(), "2px".to_string())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-right-width".to_string(), "2px".to_string())),
            "{css:?}"
        );
        assert!(
            css.contains(&(
                "border-left-color".to_string(),
                "var(--primary)".to_string()
            )),
            "{css:?}"
        );
        let op = only_op("style({ paint: borderT(1, muted) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-top-width".to_string(), "1px".to_string())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-top-color".to_string(), "var(--muted)".to_string())),
            "{css:?}"
        );
        let op = only_op("style({ paint: borderS(1, border) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-inline-start-width".to_string(), "1px".to_string())),
            "{css:?}"
        );
        assert!(
            css.contains(&(
                "border-inline-start-color".to_string(),
                "var(--border)".to_string()
            )),
            "{css:?}"
        );
    }

    #[test]
    fn border_composite_positions_enforced() {
        // width token in the color position fails closed
        let (_, diags) = parse("style({ paint: border(1, 2) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "{diags:?}"
        );
        // color token in the width position fails closed
        let (_, diags) = parse("style({ paint: border(primary, border) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "{diags:?}"
        );
        // non-border setters stay single-arg
        let (_, diags) = parse("style({ paint: bg(primary, muted) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadArity { .. })),
            "{diags:?}"
        );
    }

    #[test]
    fn border_composite_binds() {
        let op = only_op("style({ paint: border(bind(strokeW), bind(toneBorder)) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-width".to_string(), "var(--stroke-w)".to_string())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-color".to_string(), "var(--tone-border)".to_string())),
            "{css:?}"
        );
    }

    #[test]
    fn long_tail_setters_lower_like_their_vocab_utilities() {
        let cases: &[(&str, &str, &str)] = &[
            (
                "style({ text: decoration(accent) })",
                "text-decoration-color",
                "var(--accent)",
            ),
            ("style({ text: indent(2) })", "text-indent", "var(--space-2)"),
            (
                "style({ text: underlineOffset(auto) })",
                "text-underline-offset",
                "auto",
            ),
            (
                "style({ text: antialiased(true) })",
                "-webkit-font-smoothing",
                "antialiased",
            ),
            ("style({ paint: caret(primary) })", "caret-color", "var(--primary)"),
            ("style({ paint: accent(accent) })", "accent-color", "var(--accent)"),
            ("style({ paint: accent(auto) })", "accent-color", "auto"),
            ("style({ paint: objectPosition(top) })", "object-position", "top"),
            ("style({ box: order(first) })", "order", "-9999"),
            ("style({ box: justifySelf(center) })", "justify-self", "center"),
            ("style({ paint: bgClip(text) })", "background-clip", "text"),
            ("style({ paint: touch(pan-x) })", "touch-action", "pan-x"),
            ("style({ box: resize(y) })", "resize", "vertical"),
            ("style({ paint: scroll(smooth) })", "scroll-behavior", "smooth"),
            ("style({ paint: willChange(transform) })", "will-change", "transform"),
            ("style({ paint: contain(layout) })", "contain", "layout"),
        ];
        for (src, prop, val) in cases {
            let op = only_op(src);
            let css = lower_op_css(&op).unwrap();
            assert!(
                css.contains(&(prop.to_string(), val.to_string())),
                "{src}: expected {prop}: {val} in {css:?}"
            );
        }
    }

    #[test]
    fn op_slots_narrow_by_arg_value() {
        let op = only_op("style({ paint: outline(solid) })");
        assert_eq!(op_slots(&op), vec!["outline-style"]);
        let op = only_op("style({ paint: outline(2) })");
        assert_eq!(op_slots(&op), vec!["outline-width"]);
        let op = only_op("style({ paint: outline(ring) })");
        assert_eq!(op_slots(&op), vec!["outline-color"]);
        let op = only_op("style({ paint: ring(none) })");
        assert_eq!(op_slots(&op), vec!["outline-style"]);
        let op = only_op("style({ paint: ring(primary) })");
        assert_eq!(op_slots(&op), vec!["outline-color"]);
        let op = only_op("style({ paint: bg(none) })");
        assert_eq!(op_slots(&op), vec!["background-image"]);
        let op = only_op("style({ paint: bg(primary) })");
        assert_eq!(op_slots(&op), vec!["background-color"]);
        let op = only_op("style({ paint: border(2) })");
        assert_eq!(op_slots(&op), vec!["border-width"]);
        let op = only_op("style({ paint: border(dashed) })");
        assert_eq!(op_slots(&op), vec!["border-style"]);
        let op = only_op("style({ paint: border(primary) })");
        assert_eq!(op_slots(&op), vec!["border-color"]);
        let op = only_op("style({ paint: border(1, border) })");
        assert_eq!(op_slots(&op), vec!["border-width", "border-color"]);
        let op = only_op("style({ paint: borderX(2, primary) })");
        assert_eq!(
            op_slots(&op),
            vec![
                "border-left-width",
                "border-right-width",
                "border-left-color",
                "border-right-color"
            ]
        );
        // single-value setters keep their full static slot list
        let op = only_op("style({ text: truncate() })");
        assert_eq!(
            op_slots(&op),
            vec!["overflow", "text-overflow", "white-space"]
        );
    }

    #[test]
    fn outline_style_width_color_triple_write_does_not_clash() {
        let (style, diags) =
            parse("style({ when: { focus: { paint: outline(solid).outline(2).outline(ring) } } })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let clashes =
            super::super::clash::detect_clashes(&style, &std::collections::HashMap::new());
        assert!(clashes.is_empty(), "{clashes:?}");
    }

    #[test]
    fn long_tail_setters_reject_off_scale() {
        let (_, diags) = parse("style({ paint: contain(banana) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "{diags:?}"
        );
    }

    #[test]
    fn border_image_and_backdrop_lower_component_tokens() {
        let op = only_op("style({ paint: borderImage(corner-image) })");
        let css = lower_op_css(&op).unwrap();
        assert_eq!(
            css,
            vec![(
                "border-image".to_string(),
                "var(--corner-image)".to_string()
            )]
        );
        let op = only_op("style({ paint: borderImage(none) })");
        assert_eq!(
            lower_op_css(&op).unwrap(),
            vec![("border-image".to_string(), "none".to_string())]
        );
        let op = only_op("style({ paint: backdrop(card-backdrop-filter) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&(
                "backdrop-filter".to_string(),
                "var(--card-backdrop-filter)".to_string()
            )),
            "{css:?}"
        );
        assert!(
            css.contains(&(
                "-webkit-backdrop-filter".to_string(),
                "var(--card-backdrop-filter)".to_string()
            )),
            "{css:?}"
        );
        // off-table tokens fail closed
        let (_, diags) = parse("style({ paint: backdrop(banana) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "{diags:?}"
        );
        let (_, diags) = parse("style({ paint: borderImage(banana) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "{diags:?}"
        );
        // bind lowers to the same longhands
        let op = only_op("style({ paint: backdrop(bind(cardBlur)) })");
        let css = lower_op_css(&op).unwrap();
        assert!(
            css.contains(&(
                "backdrop-filter".to_string(),
                "var(--card-blur)".to_string()
            )),
            "{css:?}"
        );
        assert!(
            css.contains(&(
                "-webkit-backdrop-filter".to_string(),
                "var(--card-blur)".to_string()
            )),
            "{css:?}"
        );
    }
}
