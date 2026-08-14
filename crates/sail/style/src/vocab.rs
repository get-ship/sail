//! One vocabulary table, three row tags (SPEC-SPINNAKER-3 §1 / §4).
//!
//! Every author-facing style name is one row tagged `concern | axis | plain`.
//! Complete, clash, emit, LSP, skill, and MCP walk this table. `setters.rs`
//! remains the longhand emit / validate layer — not a second product atlas.
//!
//! Live author names are v3 (`bg`, `fg`, `position`, `pad`, `margin`, …).
//! Today's `fill` / `ink` / `place` / `space` stay as alias targets so the
//! v2 corpus still compiles; they are not offered as a dual. `z` is a
//! `position` slot only — not a row.

/// Vocabulary row tag. One table, three tags — not a second atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocabTag {
    Concern,
    Axis,
    Plain,
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
    /// Channel key (`box` / `paint` / `text` / `animate`) for complete filter.
    pub channel: &'static str,
    /// Offered by complete. Alias targets and retired longhands are false.
    pub offered: bool,
}

const fn row(
    name: &'static str,
    tag: VocabTag,
    emit: Option<&'static str>,
    channel: &'static str,
    offered: bool,
) -> VocabRow {
    VocabRow {
        name,
        tag,
        emit,
        channel,
        offered,
    }
}

const CONCERN: VocabTag = VocabTag::Concern;
const AXIS: VocabTag = VocabTag::Axis;
const PLAIN: VocabTag = VocabTag::Plain;

/// The one registry. Live v3 names first; alias targets and resolve-only
/// plains follow. No `z` row — `z` rides `position`.
pub static VOCAB: &[VocabRow] = &[
    // ── live concerns (v3 author names) ────────────────────────────────
    row("bg", CONCERN, Some("fill"), "paint", true),
    row("fg", CONCERN, Some("ink"), "paint", true),
    row("border", CONCERN, Some("border"), "paint", true),
    row("size", CONCERN, Some("size"), "box", true),
    row("type", CONCERN, Some("type"), "text", true),
    row("elevation", CONCERN, Some("elevation"), "paint", true),
    row("position", CONCERN, Some("place"), "box", true),
    row("play", CONCERN, None, "animate", true),
    // ── live axis ──────────────────────────────────────────────────────
    row("pad", AXIS, Some("space"), "box", true),
    row("margin", AXIS, Some("space"), "box", true),
    row("overflow", AXIS, Some("overflow"), "box", true),
    row("radius", AXIS, Some("radius"), "paint", true),
    // ── live plains: markers (verbs) ───────────────────────────────────
    row("group", PLAIN, None, "paint", true),
    row("isolate", PLAIN, None, "paint", true),
    row("srOnly", PLAIN, None, "paint", true),
    row("press", PLAIN, None, "paint", true),
    row("controlFrame", PLAIN, None, "paint", true),
    row("controlLift", PLAIN, None, "paint", true),
    row("cursor", PLAIN, None, "paint", true),
    row("select", PLAIN, None, "paint", true),
    row("pointerEvents", PLAIN, None, "paint", true),
    row("tapHighlight", PLAIN, None, "paint", true),
    row("touch", PLAIN, None, "paint", true),
    row("appearance", PLAIN, None, "paint", true),
    row("tab", PLAIN, None, "text", true),
    row("content", PLAIN, None, "paint", true),
    row("boxSizing", PLAIN, None, "paint", true),
    row("objectFit", PLAIN, None, "paint", true),
    row("objectPosition", PLAIN, None, "paint", true),
    row("scrollbarThumb", PLAIN, None, "paint", true),
    row("scrollbarTrack", PLAIN, None, "paint", true),
    row("scrollbarThin", PLAIN, None, "paint", true),
    row("avatarMedia", PLAIN, None, "paint", true),
    row("glyphFill", PLAIN, None, "paint", true),
    row("container", PLAIN, None, "paint", true),
    row("containerType", PLAIN, None, "paint", true),
    row("containerName", PLAIN, None, "paint", true),
    // ── alias targets (v2 corpus; not offered) ─────────────────────────
    row("fill", CONCERN, Some("fill"), "paint", false),
    row("ink", CONCERN, Some("ink"), "paint", false),
    row("place", CONCERN, Some("place"), "box", false),
    row("space", AXIS, Some("space"), "box", false),
    row("flow", CONCERN, Some("flow"), "box", false),
    row("filter", CONCERN, Some("filter"), "paint", false),
    row("motion", CONCERN, Some("motion"), "paint", false),
    // ── spec plains, resolve-only (emit still via longhand / flow) ─────
    row("display", PLAIN, None, "box", false),
    row("grow", PLAIN, None, "box", false),
    row("shrink", PLAIN, None, "box", false),
    row("self", PLAIN, None, "box", false),
    row("dir", PLAIN, None, "box", false),
    row("opacity", PLAIN, None, "paint", false),
    row("rotate", PLAIN, None, "paint", false),
    row("rotateX", PLAIN, None, "paint", false),
    row("rotateY", PLAIN, None, "paint", false),
    row("rotateZ", PLAIN, None, "paint", false),
    row("scale", PLAIN, None, "paint", false),
    row("translateX", PLAIN, None, "paint", false),
    row("translateY", PLAIN, None, "paint", false),
    row("transform", PLAIN, None, "paint", false),
    row("blur", PLAIN, None, "paint", false),
    row("brightness", PLAIN, None, "paint", false),
    row("contrast", PLAIN, None, "paint", false),
    row("grayscale", PLAIN, None, "paint", false),
    row("hueRotate", PLAIN, None, "paint", false),
    row("invert", PLAIN, None, "paint", false),
    row("sepia", PLAIN, None, "paint", false),
    row("saturate", PLAIN, None, "paint", false),
    row("backdrop", PLAIN, None, "paint", false),
    row("mixBlend", PLAIN, None, "paint", false),
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
        assert_eq!(resolve_vocab("pad"), Some(VocabTag::Axis));
        assert_eq!(resolve_vocab("margin"), Some(VocabTag::Axis));
        assert_eq!(resolve_vocab("overflow"), Some(VocabTag::Axis));
        assert_eq!(resolve_vocab("radius"), Some(VocabTag::Axis));
        assert_eq!(resolve_vocab("display"), Some(VocabTag::Plain));
        assert_eq!(resolve_vocab("grow"), Some(VocabTag::Plain));
        assert_eq!(resolve_vocab("cursor"), Some(VocabTag::Plain));
        assert_eq!(resolve_vocab("srOnly"), Some(VocabTag::Plain));
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
        let place = find_concern("place").expect("place emit target");
        assert!(
            find_slot(place, "z").is_some(),
            "z lives on the position/place concern"
        );
        let via_author = find_concern("position").expect("position → place");
        assert!(find_slot(via_author, "z").is_some());
    }

    #[test]
    fn alias_targets_resolve_and_are_not_offered() {
        for (alias, tag, emit) in [
            ("fill", VocabTag::Concern, "fill"),
            ("ink", VocabTag::Concern, "ink"),
            ("place", VocabTag::Concern, "place"),
            ("space", VocabTag::Axis, "space"),
        ] {
            let row = find_vocab(alias).expect(alias);
            assert_eq!(row.tag, tag, "{alias}");
            assert_eq!(row.emit, Some(emit), "{alias}");
            assert!(!row.offered, "{alias} must not dual-offer with the v3 name");
        }
        assert_eq!(emit_target("bg"), Some("fill"));
        assert_eq!(emit_target("fg"), Some("ink"));
        assert_eq!(emit_target("position"), Some("place"));
        assert_eq!(emit_target("pad"), Some("space"));
        assert_eq!(emit_target("margin"), Some("space"));
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
        assert_eq!(find_concern("bg").map(|c| c.name), Some("fill"));
        assert_eq!(find_concern("fg").map(|c| c.name), Some("ink"));
        assert_eq!(find_concern("position").map(|c| c.name), Some("place"));
        assert_eq!(find_concern("pad").map(|c| c.name), Some("space"));
        assert_eq!(find_concern("fill").map(|c| c.name), Some("fill"));
        assert!(find_concern("display").is_none());
        assert!(find_concern("z").is_none());
    }
}
