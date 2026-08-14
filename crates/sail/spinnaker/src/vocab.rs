//! One vocabulary table, three row tags (SPEC-SPINNAKER-3 §1 / §4).
//!
//! Every author-facing style name is one row tagged `concern | axis | plain`.
//! Complete, clash, emit, parse, LSP, skill, and MCP walk this table.
//! Plains emit lives on `plains.rs`. `setters.rs` keeps CSS lower helpers.
//!
//! Live author names are v3 (`bg`, `fg`, `position`, `pad`, `margin`, …).
//! Unknown names are unknown vocabulary. `z` is a `position` slot only.
//!
//! Capability flags (`geometry` / `textish` / `motion`) are the laws the
//! code already enforces. They are not author keys and not a Channel enum.

/// Vocabulary row tag. One table, three tags — not a second atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocabTag {
    Concern,
    Axis,
    Plain,
}

/// Laws a vocab row participates in. Not author keys (`box:` / `paint:` /
/// `text:` are dissolved). Complete filters `motion` for `animate:`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VocabCaps {
    /// Size / pad / margin / position / grow / shrink / dir / self / resize.
    pub geometry: bool,
    /// Type / typography setters. Host law lives in the compiler.
    pub textish: bool,
    /// Play / animate triggers.
    pub motion: bool,
}

impl VocabCaps {
    pub const NONE: Self = Self {
        geometry: false,
        textish: false,
        motion: false,
    };
    pub const GEOMETRY: Self = Self {
        geometry: true,
        textish: false,
        motion: false,
    };
    pub const TEXTISH: Self = Self {
        geometry: false,
        textish: true,
        motion: false,
    };
    pub const MOTION: Self = Self {
        geometry: false,
        textish: false,
        motion: true,
    };

    /// One-word label for complete / skill surfaces. Empty when unflagged.
    pub fn label(self) -> &'static str {
        if self.motion {
            "motion"
        } else if self.textish {
            "textish"
        } else if self.geometry {
            "geometry"
        } else {
            ""
        }
    }
}

/// One vocabulary row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VocabRow {
    /// Author (or alias-target) name.
    pub name: &'static str,
    pub tag: VocabTag,
    /// Emit-side concern name when this row lowers through a concern.
    /// `None` for plains. Alias targets point at themselves.
    pub emit: Option<&'static str>,
    /// Capability flags the code already enforces.
    pub caps: VocabCaps,
    /// Offered by complete. Alias targets and retired longhands are false.
    pub offered: bool,
}

const fn row(
    name: &'static str,
    tag: VocabTag,
    emit: Option<&'static str>,
    caps: VocabCaps,
    offered: bool,
) -> VocabRow {
    VocabRow {
        name,
        tag,
        emit,
        caps,
        offered,
    }
}

const CONCERN: VocabTag = VocabTag::Concern;
const AXIS: VocabTag = VocabTag::Axis;
const PLAIN: VocabTag = VocabTag::Plain;
const NONE: VocabCaps = VocabCaps::NONE;
const GEO: VocabCaps = VocabCaps::GEOMETRY;
const TEXT: VocabCaps = VocabCaps::TEXTISH;
const MOTION: VocabCaps = VocabCaps::MOTION;

/// The one registry. Live author names. No `z` row — `z` rides `position`.
pub static VOCAB: &[VocabRow] = &[
    // ── live concerns (v3 author names) ────────────────────────────────
    row("bg", CONCERN, Some("bg"), NONE, true),
    row("fg", CONCERN, Some("fg"), NONE, true),
    row("border", CONCERN, Some("border"), NONE, true),
    row("size", CONCERN, Some("size"), GEO, true),
    row("type", CONCERN, Some("type"), TEXT, true),
    row("elevation", CONCERN, Some("elevation"), NONE, true),
    row("position", CONCERN, Some("position"), GEO, true),
    row("play", CONCERN, None, MOTION, true),
    row("filter", CONCERN, Some("filter"), NONE, true),
    // ── live axis ──────────────────────────────────────────────────────
    row("pad", AXIS, Some("pad"), GEO, true),
    row("margin", AXIS, Some("margin"), GEO, true),
    row("overflow", AXIS, Some("overflow"), NONE, true),
    row("radius", AXIS, Some("radius"), NONE, true),
    // ── live plains: markers (verbs) ───────────────────────────────────
    row("group", PLAIN, None, NONE, true),
    row("isolate", PLAIN, None, NONE, true),
    row("srOnly", PLAIN, None, NONE, true),
    row("press", PLAIN, None, NONE, true),
    row("controlFrame", PLAIN, None, NONE, true),
    row("controlLift", PLAIN, None, NONE, true),
    row("cursor", PLAIN, None, NONE, true),
    row("select", PLAIN, None, NONE, true),
    row("pointerEvents", PLAIN, None, NONE, true),
    row("tapHighlight", PLAIN, None, NONE, true),
    row("touch", PLAIN, None, NONE, true),
    row("appearance", PLAIN, None, NONE, true),
    row("tab", PLAIN, None, TEXT, true),
    row("content", PLAIN, None, NONE, true),
    row("boxSizing", PLAIN, None, NONE, true),
    row("objectFit", PLAIN, None, NONE, true),
    row("objectPosition", PLAIN, None, NONE, true),
    row("scrollbarThumb", PLAIN, None, NONE, true),
    row("scrollbarTrack", PLAIN, None, NONE, true),
    row("scrollbarThin", PLAIN, None, NONE, true),
    row("container", PLAIN, None, NONE, true),
    row("containerType", PLAIN, None, NONE, true),
    row("containerName", PLAIN, None, NONE, true),
    // ── live spec plains ──────────────────────────────────────────────
    row("display", PLAIN, None, NONE, true),
    row("grow", PLAIN, None, GEO, true),
    row("shrink", PLAIN, None, GEO, true),
    row("self", PLAIN, None, GEO, true),
    row("dir", PLAIN, None, GEO, true),
    row("opacity", PLAIN, None, NONE, true),
    row("rotate", PLAIN, None, NONE, true),
    row("rotateX", PLAIN, None, NONE, true),
    row("rotateY", PLAIN, None, NONE, true),
    row("rotateZ", PLAIN, None, NONE, true),
    row("scale", PLAIN, None, NONE, true),
    row("translateX", PLAIN, None, NONE, true),
    row("translateY", PLAIN, None, NONE, true),
    row("transform", PLAIN, None, NONE, true),
    row("origin", PLAIN, None, NONE, true),
    row("animate", PLAIN, None, NONE, true),
    row("transition", PLAIN, None, NONE, true),
    row("duration", PLAIN, None, NONE, true),
    row("ease", PLAIN, None, NONE, true),
    row("blur", PLAIN, None, NONE, true),
    row("brightness", PLAIN, None, NONE, true),
    row("contrast", PLAIN, None, NONE, true),
    row("grayscale", PLAIN, None, NONE, true),
    row("hueRotate", PLAIN, None, NONE, true),
    row("invert", PLAIN, None, NONE, true),
    row("sepia", PLAIN, None, NONE, true),
    row("saturate", PLAIN, None, NONE, true),
    row("backdrop", PLAIN, None, NONE, true),
    row("mixBlend", PLAIN, None, NONE, true),
    // ── SPEC §8 restored plains ───────────────────────────────────────
    row("align", PLAIN, None, NONE, true),
    row("list", PLAIN, None, NONE, true),
    row("resize", PLAIN, None, GEO, true),
    row("contain", PLAIN, None, NONE, true),
    row("willChange", PLAIN, None, NONE, true),
    row("visibility", PLAIN, None, NONE, true),
];

/// The table. Complete / resolve / find walk this.
pub fn vocab_rows() -> &'static [VocabRow] {
    VOCAB
}

/// Live author rows offered by complete.
pub fn offered_rows() -> impl Iterator<Item = &'static VocabRow> {
    VOCAB.iter().filter(|r| r.offered)
}

/// Look up a name in the one table.
pub fn find_vocab(name: &str) -> Option<&'static VocabRow> {
    VOCAB.iter().find(|r| r.name == name)
}

/// Tag for `name`, or `None` when it is not vocabulary (`z` is not a row).
pub fn resolve_vocab(name: &str) -> Option<VocabTag> {
    find_vocab(name).map(|r| r.tag)
}

/// Emit-side concern name when `name` lowers through a concern / axis.
pub fn emit_target(name: &str) -> Option<&'static str> {
    find_vocab(name).and_then(|r| r.emit)
}

/// One skill / MCP surface line. Author name, never an emit-target alias.
pub fn vocab_surface_line(row: &VocabRow) -> String {
    let kind = match row.tag {
        VocabTag::Concern => "concern",
        VocabTag::Axis => "axis",
        VocabTag::Plain => "plain",
    };
    let cap = row.caps.label();
    let cap_bit = if cap.is_empty() {
        String::new()
    } else {
        format!(" · {cap}")
    };
    match row.tag {
        VocabTag::Concern | VocabTag::Axis => {
            let slots = row
                .emit
                .and_then(crate::concerns::find_concern)
                .map(|c| {
                    c.slots
                        .iter()
                        .map(|s| s.name)
                        .collect::<Vec<_>>()
                        .join("`/`")
                })
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "…".into());
            let mut line = format!("`{}(…)` · {kind}{cap_bit} · slots `{slots}`", row.name);
            if row.name == "border" {
                line.push_str(" · radius is not a slot · kind is a family key");
            }
            if row.name == "size" {
                line.push_str(" · pad is not a slot (use `pad`)");
            }
            if row.name == "play" {
                line.push_str(" · `animate:` only");
            }
            line
        }
        VocabTag::Plain => format!("`{}(…)` · {kind}{cap_bit}", row.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concerns::{find_concern, find_slot};
    use crate::theme_env::ThemeStyleEnv;

    #[test]
    fn live_rows_are_tagged() {
        assert_eq!(resolve_vocab("bg"), Some(VocabTag::Concern));
        assert_eq!(resolve_vocab("fg"), Some(VocabTag::Concern));
        assert_eq!(resolve_vocab("position"), Some(VocabTag::Concern));
        assert_eq!(resolve_vocab("play"), Some(VocabTag::Concern));
        assert_eq!(resolve_vocab("filter"), Some(VocabTag::Concern));
        assert_eq!(resolve_vocab("pad"), Some(VocabTag::Axis));
        assert_eq!(resolve_vocab("margin"), Some(VocabTag::Axis));
        assert_eq!(resolve_vocab("overflow"), Some(VocabTag::Axis));
        assert_eq!(resolve_vocab("radius"), Some(VocabTag::Axis));
        assert_eq!(resolve_vocab("display"), Some(VocabTag::Plain));
        assert_eq!(resolve_vocab("grow"), Some(VocabTag::Plain));
        assert_eq!(resolve_vocab("cursor"), Some(VocabTag::Plain));
        assert_eq!(resolve_vocab("srOnly"), Some(VocabTag::Plain));
        assert_eq!(resolve_vocab("origin"), Some(VocabTag::Plain));
        for name in [
            "display",
            "grow",
            "dir",
            "opacity",
            "rotate",
            "scale",
            "blur",
            "mixBlend",
            "align",
            "list",
            "resize",
            "contain",
            "willChange",
            "visibility",
        ] {
            let row = find_vocab(name).unwrap_or_else(|| panic!("{name}"));
            assert!(row.offered, "{name} is a live spec plain");
            assert_eq!(row.tag, VocabTag::Plain);
        }
    }

    #[test]
    fn capability_flags_match_laws() {
        assert!(find_vocab("size").unwrap().caps.geometry);
        assert!(find_vocab("pad").unwrap().caps.geometry);
        assert!(find_vocab("grow").unwrap().caps.geometry);
        assert!(find_vocab("resize").unwrap().caps.geometry);
        assert!(!find_vocab("display").unwrap().caps.geometry);
        assert!(!find_vocab("overflow").unwrap().caps.geometry);
        assert!(find_vocab("type").unwrap().caps.textish);
        assert!(find_vocab("tab").unwrap().caps.textish);
        assert!(!find_vocab("bg").unwrap().caps.textish);
        assert!(find_vocab("play").unwrap().caps.motion);
        assert!(!find_vocab("animate").unwrap().caps.motion);
        assert!(!find_vocab("align").unwrap().caps.geometry);
        assert!(!find_vocab("align").unwrap().caps.textish);
    }

    #[test]
    fn theme_env_resolve_vocab_walks_the_table() {
        let env = ThemeStyleEnv::from_registry();
        assert_eq!(env.resolve_vocab("bg"), Some(VocabTag::Concern));
        assert_eq!(env.resolve_vocab("pad"), Some(VocabTag::Axis));
        assert_eq!(env.resolve_vocab("display"), Some(VocabTag::Plain));
        assert_eq!(env.resolve_vocab("z"), None);
        assert_eq!(env.resolve_vocab("notARealSetterZzz"), None);
    }

    #[test]
    fn z_is_position_slot_not_a_row() {
        assert!(find_vocab("z").is_none());
        assert_eq!(resolve_vocab("z"), None);
        let pos = find_concern("position").expect("position concern");
        assert!(
            find_slot(pos, "z").is_some(),
            "z lives on the position concern"
        );
    }

    #[test]
    fn emit_targets_are_the_author_concerns() {
        assert_eq!(emit_target("bg"), Some("bg"));
        assert_eq!(emit_target("fg"), Some("fg"));
        assert_eq!(emit_target("position"), Some("position"));
        assert_eq!(emit_target("pad"), Some("pad"));
        assert_eq!(emit_target("margin"), Some("margin"));
    }

    #[test]
    fn names_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for row in VOCAB {
            assert!(
                seen.insert(row.name),
                "duplicate vocabulary row `{}`",
                row.name
            );
        }
    }

    #[test]
    fn find_concern_reads_author_names() {
        assert_eq!(find_concern("bg").map(|c| c.name), Some("bg"));
        assert_eq!(find_concern("fg").map(|c| c.name), Some("fg"));
        assert_eq!(find_concern("position").map(|c| c.name), Some("position"));
        assert_eq!(find_concern("pad").map(|c| c.name), Some("pad"));
        assert!(find_concern("display").is_none());
        assert!(find_concern("z").is_none());
    }

    #[test]
    fn surface_line_uses_author_names() {
        let bg = find_vocab("bg").expect("bg");
        let line = vocab_surface_line(bg);
        assert!(line.contains("`bg(…)`"), "{line}");
        assert!(!line.contains("paint"), "{line}");
        let pad = find_vocab("pad").expect("pad");
        let line = vocab_surface_line(pad);
        assert!(line.contains("`pad(…)`"), "{line}");
        assert!(line.contains("geometry"), "{line}");
        let ty = find_vocab("type").expect("type");
        let line = vocab_surface_line(ty);
        assert!(line.contains("textish"), "{line}");
        let play = find_vocab("play").expect("play");
        let line = vocab_surface_line(play);
        assert!(line.contains("animate:"), "{line}");
        assert!(!line.contains("channel"), "{line}");
    }
}
