//! Setter capability flags (geometry / textish / motion).
//!
//! Flags live on [`crate::vocab::VocabRow`]. This module walks the Style
//! tree and answers the laws those flags encode. Tag / host allowlists
//! do not live here — the compiler owns text-host law.

use crate::ir::{PaintArg, PaintOp, Style};
use crate::vocab::{VocabCaps, find_vocab};

/// Motion triggers that live under `animate:` (brace-config form).
pub fn is_animate_trigger(name: &str) -> bool {
    matches!(
        name,
        "enter"
            | "exit"
            | "loop"
            | "reveal"
            | "scroll"
            | "hover"
            | "press"
            | "focus"
            | "focus-visible"
    )
}

/// Capability flags for a product setter name.
///
/// Walks `vocab.rs` then `find_concern` / `covered_setters`. Gap keys are
/// layout attrs, not Style — still geometry so retirement checks hold.
pub fn setter_caps(name: &str) -> Option<VocabCaps> {
    if let Some(row) = find_vocab(name) {
        return Some(row.caps);
    }
    if let Some(c) = crate::concerns::find_concern(name) {
        return find_vocab(c.name).map(|r| r.caps);
    }
    for spec in crate::concerns::concerns() {
        if spec.covered_setters.contains(&name) {
            return find_vocab(spec.name).map(|r| r.caps);
        }
    }
    match name {
        "gap" | "gapX" | "gapY" | "basis" => Some(VocabCaps::GEOMETRY),
        _ => None,
    }
}

/// True when `style` spends any textish setter (typography).
///
/// Does not resolve `Style::Ref` — callers that have a [`crate::StyleEnv`]
/// should expand refs first, or use [`style_is_textish_env`].
pub fn style_is_textish(style: &Style) -> bool {
    style_is_textish_env(style, &std::collections::HashMap::new())
}

/// Alias of [`style_is_textish`].
pub fn style_uses_text(style: &Style) -> bool {
    style_is_textish(style)
}

/// Like [`style_is_textish`], resolving scope refs via `env`.
pub fn style_is_textish_env(style: &Style, env: &std::collections::HashMap<String, Style>) -> bool {
    style_is_textish_inner(style, env, &mut std::collections::HashSet::new())
}

fn style_is_textish_inner(
    style: &Style,
    env: &std::collections::HashMap<String, Style>,
    resolving: &mut std::collections::HashSet<String>,
) -> bool {
    match style {
        Style::Empty | Style::MotionAttach { .. } => false,
        Style::Set(op) => setter_caps(&op.setter).is_some_and(|c| c.textish),
        Style::Chain(items) | Style::List(items) => items
            .iter()
            .any(|i| style_is_textish_inner(i, env, resolving)),
        Style::Match { arms, .. } => arms
            .iter()
            .any(|(_, b)| style_is_textish_inner(b, env, resolving)),
        Style::Compound { body, .. } => style_is_textish_inner(body, env, resolving),
        Style::When { entries } => entries
            .iter()
            .any(|(_, b)| style_is_textish_inner(b, env, resolving)),
        Style::Ref(name) => {
            if !resolving.insert(name.clone()) {
                return false;
            }
            let hit = crate::ThemeStyleEnv::from_registry()
                .expand_style_ref(name, env, crate::current_styling_policy())
                .ok()
                .is_some_and(|s| style_is_textish_inner(&s, env, resolving));
            resolving.remove(name);
            hit
        }
    }
}

/// True when a `when` key is interaction / component-state (not media/viewport).
///
/// Used for “state can't gate geometry”. Media keys may still gate geometry.
/// Structural pseudo-elements (`before`/`after`) and structural slot keys
/// (`empty`, `first`/`last`) are not interaction — they define pseudo boxes
/// and slot chrome, not pointer/ARIA thrash.
pub fn is_interaction_when_key(key: &str) -> bool {
    match key {
        "hover" | "active" | "focus" | "focusWithin" | "focus-within" | "disabled" | "selected"
        | "expanded" | "checked" | "pressed" | "open" | "current" | "busy" | "sortable" => true,
        // Structural / pseudo-element — not interaction thrash.
        "empty" | "before" | "after" | "first" | "first-child" | "last" | "last-child" => false,
        other => {
            let kebab = when_key_kebab(other);
            // Media / preference conditions are not interaction.
            if crate::conditions::condition_media(other).is_some()
                || crate::conditions::condition_media(&kebab).is_some()
            {
                return false;
            }
            // state-*, group-* (component / ancestor state) are interaction.
            if kebab.starts_with("state-") || kebab.starts_with("group-") {
                return true;
            }
            crate::prefixes::state_pseudo(other).is_some()
                || crate::prefixes::state_pseudo(&kebab).is_some()
                || crate::prefixes::state_data(other).is_some()
                || crate::prefixes::state_data(&kebab).is_some()
        }
    }
}

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

/// True when the setter is geometry-class for the “state can't gate geometry”
/// law: dimensional ops (size / pad / margin / position / grow / shrink /
/// dir / self / resize / basis).
///
/// Keys off the vocab `geometry` flag. `display` and `overflow` stay
/// unflagged so show/hide under interaction when does not trip pad law.
pub fn is_geometry_setter(name: &str) -> bool {
    match name {
        "display" | "overflow" | "overflowX" | "overflowY" | "flow" => false,
        other => setter_caps(other).is_some_and(|c| c.geometry),
    }
}

fn place_gates_layout(op: &PaintOp) -> bool {
    match op.first_arg() {
        PaintArg::Slots(slots) => slots.keys().any(|k| k != "z"),
        _ => true,
    }
}

/// True when `style` contains dimensional geometry that interaction state
/// must not gate. Nested structural when keys (`before`/`after`/`empty`/…)
/// own their own pseudo boxes — geometry there is not counted as gated by
/// an outer interaction key (switch thumb under `checked → before`).
pub fn style_contains_geometry(style: &Style) -> bool {
    match style {
        Style::Empty | Style::MotionAttach { .. } | Style::Ref(_) => false,
        Style::Set(op) => {
            // `position({ z })` is stacking, not box geometry — z lived on
            // paint before SPEC-2 and must still be legal under interaction when().
            if op.setter == "position" {
                place_gates_layout(op)
            } else {
                is_geometry_setter(&op.setter)
            }
        }
        Style::Chain(items) | Style::List(items) => items.iter().any(style_contains_geometry),
        Style::Match { arms, .. } => arms.iter().any(|(_, b)| style_contains_geometry(b)),
        Style::Compound { body, .. } => style_contains_geometry(body),
        Style::When { entries } => entries.iter().any(|(k, b)| {
            if is_structural_when_key(k) {
                false
            } else {
                style_contains_geometry(b)
            }
        }),
    }
}

fn is_structural_when_key(key: &str) -> bool {
    matches!(
        key,
        "before" | "after" | "empty" | "first" | "first-child" | "last" | "last-child"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn style_is_textish_detects_type_not_pad() {
        let (s, _) = parse("style({ type({ size: sm }) })");
        assert!(style_is_textish(&s));
        assert!(style_uses_text(&s));
        let (s, _) = parse("style({ pad(2), bg(primary) })");
        assert!(!style_is_textish(&s));
    }

    #[test]
    fn interaction_when_keys_vs_media() {
        assert!(is_interaction_when_key("hover"));
        assert!(is_interaction_when_key("focusWithin"));
        assert!(is_interaction_when_key("group-hover"));
        assert!(is_interaction_when_key("state-open"));
        assert!(is_interaction_when_key("selected"));
        assert!(!is_interaction_when_key("tablet"));
        assert!(!is_interaction_when_key("desktop"));
        assert!(!is_interaction_when_key("dark"));
        assert!(!is_interaction_when_key("motionReduce"));
        assert!(!is_interaction_when_key("motion-reduce"));
        assert!(!is_interaction_when_key("empty"));
        assert!(!is_interaction_when_key("before"));
        assert!(!is_interaction_when_key("after"));
    }

    #[test]
    fn geometry_setters_are_dimensional() {
        for s in [
            "size", "position", "pad", "margin", "grow", "shrink", "dir", "self", "resize", "basis",
        ] {
            assert!(is_geometry_setter(s), "{s}");
        }
        assert!(setter_caps("dir").unwrap().geometry);
        assert!(!setter_caps("display").unwrap().geometry);
        assert!(!is_geometry_setter("display"));
        assert!(!is_geometry_setter("overflow"));
        assert!(!is_geometry_setter("type"));
        assert!(!setter_caps("align").unwrap().geometry);
        assert!(setter_caps("resize").unwrap().geometry);
    }

    #[test]
    fn geometry_in_interaction_when_refused() {
        let (_, diags) = parse("style({ pad(2), when: { hover: { pad(4) } } })");
        assert!(
            diags
                .iter()
                .any(|d| d.message().contains("state can't gate layout")),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, diags) = parse("style({ pad(2), when: { tablet: { pad(4) } } })");
        assert!(
            !diags
                .iter()
                .any(|d| d.message().contains("state can't gate layout")),
            "media may gate geometry: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, diags) = parse("style({ pad(2), when: { hover: { bg(muted) } } })");
        assert!(
            !diags
                .iter()
                .any(|d| d.message().contains("state can't gate layout")),
            "paint under hover ok: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    /// Geometry-class × interaction matrix (SPEC: state can't gate dimensional box).
    /// Asserts only current product law — display/overflow are non-geometry.
    #[test]
    fn geometry_interaction_matrix() {
        fn gate_err(src: &str) -> bool {
            let (_, diags) = parse(src);
            diags
                .iter()
                .any(|d| d.message().contains("state can't gate layout"))
        }

        // Dimensional geometry under interaction keys — refuse.
        for key in ["hover", "group-hover", "state-open", "focusWithin"] {
            let src = format!("style({{ pad(2), when: {{ {key}: {{ pad(4) }} }} }})");
            assert!(gate_err(&src), "pad under {key} must refuse: {src}");
            let src = format!(
                "style({{ size({{ w: full }}), when: {{ {key}: {{ size({{ h: full }}) }} }} }})"
            );
            assert!(gate_err(&src), "size under {key} must refuse: {src}");
        }

        // Media / preference may still gate geometry.
        for key in ["tablet", "dark"] {
            let src = format!("style({{ pad(2), when: {{ {key}: {{ pad(4) }} }} }})");
            assert!(!gate_err(&src), "pad under media {key} must allow: {src}");
        }

        // Paint under interaction is legal.
        assert!(!gate_err(
            "style({ bg(primary), when: { hover: { bg(muted) } } })"
        ));
        assert!(!gate_err(
            "style({ pad(2), bg(primary), when: { hover: { bg(muted) } } })"
        ));

        // Non-geometry flow/overflow under interaction — currently legal
        // (is_geometry_setter excludes them so show/hide slots don't thrash pad law).
        assert!(
            !is_geometry_setter("flow") && !is_geometry_setter("overflow"),
            "flow/overflow stay non-geometry for interaction gate"
        );
        assert!(!gate_err(
            "style({ display(none), when: { hover: { display(block) } } })"
        ));
        assert!(!gate_err(
            "style({ overflow({ all: hidden }), when: { group-hover: { overflow({ all: auto }) } } })"
        ));
    }

    #[test]
    fn color_setter_refused_with_fg_pointer() {
        let (_, diags) = parse("style({ color(primary) })");
        assert!(
            diags.iter().any(|d| d.message().contains("fg")),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn filter_and_markers_compile_and_lower() {
        let src = "style({ \
            filter({ blur: 4, brightness: 100, contrast: 100, grayscale: 0, \
                hue: 0, invert: 0, sepia: 0, saturate: 100 }).containerType(inline-size).containerName(main), \
            pad(2) \
        })";
        let (_style, diags) = parse(src);
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }
}
