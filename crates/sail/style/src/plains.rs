//! Plains emit data — scale, keywords, slots, vocab head, negatable.
//!
//! Product names live on `vocab.rs`. This module is reached after
//! `find_vocab` says [`VocabTag::Plain`](crate::vocab::VocabTag::Plain).
//! Same relationship `concerns.rs` has to vocab: emit data, not a name atlas.

/// Emit fields for one vocab Plain. Looked up only after `find_vocab`.
#[derive(Debug, Clone, Copy)]
pub struct PlainSpec {
    pub name: &'static str,
    /// Utility head used for CSS resolve (`opacity`, `translate-x`, …).
    pub vocab: &'static str,
    /// Scale name for token check (`color`, `space`, `radius`, …), if any.
    pub scale: Option<&'static str>,
    /// Extra accepted keywords beyond the scale.
    pub keywords: &'static [&'static str],
    /// When true, token may be prefixed with `-` (negatable space/size).
    pub negatable: bool,
    /// CSS longhands this plain can write. Clash analysis keys on these.
    /// Empty only for `group` (host marker, no paint).
    pub slots: &'static [&'static str],
}

/// Skill / MCP surface line for one plain.
pub fn plain_doc_line(s: &PlainSpec) -> String {
    let scale = s.scale.unwrap_or("—");
    let kws = if s.keywords.is_empty() {
        String::new()
    } else {
        format!(" keywords: {}", s.keywords.join(", "))
    };
    format!("`{}(…)` → vocab `{}` scale `{scale}`{kws}", s.name, s.vocab)
}

/// Plains emit table. Names here are vocab Plains; walk `vocab.rs` for names.
const PLAINS: &[PlainSpec] = &[
    PlainSpec {
        name: "group",
        vocab: "group",
        scale: None,
        keywords: &[],
        negatable: false,
        slots: &[],
    },
    PlainSpec {
        name: "isolate",
        vocab: "isolate",
        scale: None,
        keywords: &[],
        negatable: false,
        slots: &["isolation"],
    },
    PlainSpec {
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
    PlainSpec {
        name: "press",
        vocab: "press",
        scale: None,
        keywords: &[],
        negatable: false,
        slots: &["translate", "scale", "box-shadow"],
    },
    PlainSpec {
        name: "controlFrame",
        vocab: "control-frame",
        scale: None,
        keywords: &[],
        negatable: false,
        slots: &["box-shadow"],
    },
    PlainSpec {
        name: "controlLift",
        vocab: "control-lift",
        scale: None,
        keywords: &[],
        negatable: false,
        slots: &["translate"],
    },
    PlainSpec {
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
    PlainSpec {
        name: "select",
        vocab: "select",
        scale: None,
        keywords: &["none", "text", "all", "auto"],
        negatable: false,
        slots: &["user-select"],
    },
    PlainSpec {
        name: "pointerEvents",
        vocab: "pointer-events",
        scale: None,
        keywords: &["none", "auto"],
        negatable: false,
        slots: &["pointer-events"],
    },
    PlainSpec {
        name: "tapHighlight",
        vocab: "tap-highlight",
        scale: None,
        keywords: &["none", "auto"],
        negatable: false,
        slots: &["-webkit-tap-highlight-color"],
    },
    PlainSpec {
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
    PlainSpec {
        name: "appearance",
        vocab: "appearance",
        scale: None,
        keywords: &["none", "auto"],
        negatable: false,
        slots: &["appearance"],
    },
    PlainSpec {
        name: "tab",
        vocab: "tab",
        scale: None,
        keywords: &["1", "2", "4", "8"],
        negatable: false,
        slots: &["tab-size"],
    },
    PlainSpec {
        name: "content",
        vocab: "content",
        scale: None,
        keywords: &["empty"],
        negatable: false,
        slots: &["content"],
    },
    PlainSpec {
        name: "boxSizing",
        vocab: "box",
        scale: None,
        keywords: &["border", "content"],
        negatable: false,
        slots: &["box-sizing"],
    },
    PlainSpec {
        name: "objectFit",
        vocab: "object",
        scale: None,
        keywords: &["contain", "cover", "fill", "none", "scale-down"],
        negatable: false,
        slots: &["object-fit"],
    },
    PlainSpec {
        name: "objectPosition",
        vocab: "object",
        scale: None,
        keywords: &["center", "top", "right", "bottom", "left"],
        negatable: false,
        slots: &["object-position"],
    },
    PlainSpec {
        name: "scrollbarThumb",
        vocab: "scrollbar-thumb",
        scale: Some("color"),
        keywords: &["transparent"],
        negatable: false,
        slots: &["--sx-scrollbar-thumb"],
    },
    PlainSpec {
        name: "scrollbarTrack",
        vocab: "scrollbar-track",
        scale: Some("color"),
        keywords: &["transparent"],
        negatable: false,
        slots: &["--sx-scrollbar-track"],
    },
    PlainSpec {
        name: "scrollbarThin",
        vocab: "scrollbar-thin",
        scale: None,
        keywords: &[],
        negatable: false,
        slots: &["scrollbar-width", "scrollbar-color"],
    },
    PlainSpec {
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
    PlainSpec {
        name: "glyphFill",
        vocab: "glyph-fill",
        scale: None,
        keywords: &["true", "on"],
        negatable: false,
        slots: &["display", "width", "height"],
    },
    PlainSpec {
        name: "container",
        vocab: "container",
        scale: None,
        keywords: &["normal", "size", "inline-size"],
        negatable: false,
        slots: &["container-type"],
    },
    PlainSpec {
        name: "containerType",
        vocab: "container-type",
        scale: None,
        keywords: &["normal", "size", "inline-size"],
        negatable: false,
        slots: &["container-type"],
    },
    PlainSpec {
        name: "containerName",
        vocab: "container-name",
        scale: None,
        keywords: &["none"],
        negatable: false,
        slots: &["container-name"],
    },
    PlainSpec {
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
    PlainSpec {
        name: "grow",
        vocab: "grow",
        scale: None,
        keywords: &["0", "1"],
        negatable: false,
        slots: &["flex-grow"],
    },
    PlainSpec {
        name: "shrink",
        vocab: "shrink",
        scale: None,
        keywords: &["0", "1"],
        negatable: false,
        slots: &["flex-shrink"],
    },
    PlainSpec {
        name: "self",
        vocab: "self",
        scale: None,
        keywords: &["auto", "start", "center", "end", "stretch", "baseline"],
        negatable: false,
        slots: &["align-self"],
    },
    PlainSpec {
        name: "dir",
        vocab: "flex-dir",
        scale: None,
        keywords: &["row", "column", "row-reverse", "column-reverse"],
        negatable: false,
        slots: &["flex-direction"],
    },
    PlainSpec {
        name: "opacity",
        vocab: "opacity",
        scale: None,
        keywords: &[
            "0", "5", "10", "20", "25", "30", "40", "50", "60", "70", "75", "80", "90", "95", "100",
        ],
        negatable: false,
        slots: &["opacity"],
    },
    PlainSpec {
        name: "rotate",
        vocab: "rotate",
        scale: Some("angle"),
        keywords: &["0"],
        negatable: true,
        slots: &["rotate"],
    },
    PlainSpec {
        name: "rotateX",
        vocab: "rotate-x",
        scale: Some("angle"),
        keywords: &["0"],
        negatable: true,
        slots: &["rotate"],
    },
    PlainSpec {
        name: "rotateY",
        vocab: "rotate-y",
        scale: Some("angle"),
        keywords: &["0"],
        negatable: true,
        slots: &["rotate"],
    },
    PlainSpec {
        name: "rotateZ",
        vocab: "rotate-z",
        scale: Some("angle"),
        keywords: &["0"],
        negatable: true,
        slots: &["rotate"],
    },
    PlainSpec {
        name: "scale",
        vocab: "scale",
        scale: Some("percent"),
        keywords: &[
            "0", "50", "75", "90", "95", "100", "105", "110", "125", "150",
        ],
        negatable: false,
        slots: &["scale"],
    },
    PlainSpec {
        name: "translateX",
        vocab: "translate-x",
        scale: Some("space"),
        keywords: &["0"],
        negatable: true,
        slots: &["translate"],
    },
    PlainSpec {
        name: "translateY",
        vocab: "translate-y",
        scale: Some("space"),
        keywords: &["0"],
        negatable: true,
        slots: &["translate"],
    },
    PlainSpec {
        name: "transform",
        vocab: "transform",
        scale: None,
        keywords: &["none"],
        negatable: false,
        slots: &["transform"],
    },
    PlainSpec {
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
    PlainSpec {
        name: "animate",
        vocab: "animate",
        scale: None,
        keywords: &["spin"],
        negatable: false,
        slots: &["animation"],
    },
    PlainSpec {
        name: "transition",
        vocab: "transition",
        scale: None,
        keywords: &["none", "all", "colors", "opacity", "shadow", "transform"],
        negatable: false,
        slots: &["transition-property"],
    },
    PlainSpec {
        name: "duration",
        vocab: "duration",
        scale: Some("duration"),
        keywords: &["fast", "medium", "slow", "instant", "deliberate", "glacial"],
        negatable: false,
        slots: &["transition-duration"],
    },
    PlainSpec {
        name: "ease",
        vocab: "ease",
        scale: Some("ease"),
        keywords: &["in", "out", "in-out", "out-soft", "spring"],
        negatable: false,
        slots: &["transition-timing-function"],
    },
    PlainSpec {
        name: "blur",
        vocab: "blur",
        scale: None,
        keywords: &["0", "1", "2", "4", "8", "12", "16", "24"],
        negatable: false,
        slots: &["filter"],
    },
    PlainSpec {
        name: "brightness",
        vocab: "brightness",
        scale: None,
        keywords: &[
            "0", "50", "75", "90", "95", "100", "105", "110", "125", "150", "200",
        ],
        negatable: false,
        slots: &["filter"],
    },
    PlainSpec {
        name: "contrast",
        vocab: "contrast",
        scale: None,
        keywords: &[
            "0", "50", "75", "90", "95", "100", "105", "110", "125", "150", "200",
        ],
        negatable: false,
        slots: &["filter"],
    },
    PlainSpec {
        name: "grayscale",
        vocab: "grayscale",
        scale: None,
        keywords: &["0", "50", "100"],
        negatable: false,
        slots: &["filter"],
    },
    PlainSpec {
        name: "hueRotate",
        vocab: "hue-rotate",
        scale: None,
        keywords: &["0", "15", "30", "60", "90", "180"],
        negatable: true,
        slots: &["filter"],
    },
    PlainSpec {
        name: "invert",
        vocab: "invert",
        scale: None,
        keywords: &["0", "50", "100"],
        negatable: false,
        slots: &["filter"],
    },
    PlainSpec {
        name: "sepia",
        vocab: "sepia",
        scale: None,
        keywords: &["0", "50", "100"],
        negatable: false,
        slots: &["filter"],
    },
    PlainSpec {
        name: "saturate",
        vocab: "saturate",
        scale: None,
        keywords: &["0", "50", "60", "65", "75", "100"],
        negatable: false,
        slots: &["filter"],
    },
    PlainSpec {
        name: "backdrop",
        vocab: "backdrop",
        scale: None,
        keywords: &["none", "card-backdrop-filter"],
        negatable: false,
        slots: &["backdrop-filter", "-webkit-backdrop-filter"],
    },
    PlainSpec {
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
];

/// Emit data for a vocab Plain. Call after `find_vocab` says Plain.
pub fn find_plain(name: &str) -> Option<&'static PlainSpec> {
    PLAINS.iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab::{VocabTag, vocab_rows};

    #[test]
    fn vocab_plains_have_emit_slots() {
        for row in vocab_rows() {
            if row.tag != VocabTag::Plain {
                continue;
            }
            let s = find_plain(row.name)
                .unwrap_or_else(|| panic!("plain `{}` has no emit data", row.name));
            if s.name == "group" {
                assert!(
                    s.slots.is_empty(),
                    "group is a host marker — no paint slots"
                );
            } else {
                assert!(!s.slots.is_empty(), "plain {} has no slots", s.name);
            }
        }
    }

    #[test]
    fn plains_emit_names_are_vocab_plains() {
        for p in PLAINS {
            let row = crate::vocab::find_vocab(p.name)
                .unwrap_or_else(|| panic!("plain emit `{}` is not vocabulary", p.name));
            assert_eq!(row.tag, VocabTag::Plain, "{}", p.name);
        }
    }

    #[test]
    fn dir_emits_flex_direction() {
        let p = find_plain("dir").expect("dir");
        assert_eq!(p.slots, &["flex-direction"]);
        assert!(p.keywords.contains(&"column"));
    }
}
