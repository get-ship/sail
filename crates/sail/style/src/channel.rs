//! Setter → channel assignment (INVENTORY-CHANNELS-1 §3 / job-spec spine).
//!
//! Phase 3 also owns text-bearing host law and geometry-class interaction gating.

use crate::ir::{PaintArg, PaintOp, Style};

/// Author channel for a Style setter or motion trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Box,
    Paint,
    Text,
    Animate,
}

impl Channel {
    pub fn key(self) -> &'static str {
        match self {
            Channel::Box => "box",
            Channel::Paint => "paint",
            Channel::Text => "text",
            Channel::Animate => "animate",
        }
    }
}

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

/// Elements that may host the `text:` channel (typography overrides).
///
/// SPEC list (p, h1–h6, span, a, label, code, summary, legend) plus interactive
/// and prose hosts used by the corpus. Layout keywords, pure structure (`div`,
/// `svg`, `scroll-area`), and non-text chrome are **not** text-bearing.
///
/// Scope Style consts are **not** host-checked at definition (no tag); the
/// host check runs when a Style expression attaches to an element.
pub const TEXT_BEARING_ELEMENTS: &[&str] = &[
    // SPEC core
    "p",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "span",
    "a",
    "label",
    "code",
    "summary",
    "legend",
    // Interactive text hosts
    "button",
    "input",
    "textarea",
    "option",
    // Prose / inline / table text
    "pre",
    "small",
    "strong",
    "em",
    "b",
    "i",
    "li",
    "th",
    "td",
    "dt",
    "dd",
    "figcaption",
    "caption",
    "cite",
    "time",
    "mark",
    "abbr",
    "sub",
    "sup",
    "blockquote",
    // Optional custom text host
    "text",
];

/// True when `tag` may carry the text channel.
pub fn is_text_bearing(tag: &str) -> bool {
    let t = tag.trim();
    if t.is_empty() {
        return false;
    }
    // Case-insensitive HTML tags; custom elements stay lowercase in corpus.
    let lower = t.to_ascii_lowercase();
    TEXT_BEARING_ELEMENTS.iter().any(|e| *e == lower)
}

/// True when `style` uses any text-channel setter (typography).
///
/// Does not resolve `Style::Ref` — callers that have a [`StyleEnv`] should
/// expand refs first, or use [`style_uses_text_channel_env`].
pub fn style_uses_text_channel(style: &Style) -> bool {
    style_uses_text_channel_env(style, &std::collections::HashMap::new())
}

/// Like [`style_uses_text_channel`], resolving scope refs via `env`.
pub fn style_uses_text_channel_env(
    style: &Style,
    env: &std::collections::HashMap<String, Style>,
) -> bool {
    style_uses_text_channel_inner(style, env, &mut std::collections::HashSet::new())
}

fn style_uses_text_channel_inner(
    style: &Style,
    env: &std::collections::HashMap<String, Style>,
    resolving: &mut std::collections::HashSet<String>,
) -> bool {
    match style {
        Style::Empty | Style::MotionAttach { .. } => false,
        Style::Set(op) => channel_for_setter(&op.setter) == Some(Channel::Text),
        Style::Chain(items) | Style::List(items) => items
            .iter()
            .any(|i| style_uses_text_channel_inner(i, env, resolving)),
        Style::Match { arms, .. } => arms
            .iter()
            .any(|(_, b)| style_uses_text_channel_inner(b, env, resolving)),
        Style::Compound { body, .. } => style_uses_text_channel_inner(body, env, resolving),
        Style::When { entries } => entries
            .iter()
            .any(|(_, b)| style_uses_text_channel_inner(b, env, resolving)),
        Style::Ref(name) => {
            if !resolving.insert(name.clone()) {
                return false;
            }
            let hit = crate::ThemeStyleEnv::from_registry()
                .expand_style_ref(name, env, crate::current_styling_policy())
                .ok()
                .is_some_and(|s| style_uses_text_channel_inner(&s, env, resolving));
            resolving.remove(name);
            hit
        }
    }
}

/// Diagnostic when text channel attaches to a non-text-bearing host.
pub fn text_channel_host_error(tag: &str) -> String {
    format!(
        "text channel is only legal on text-bearing elements (got `<{tag}>`) — \
         hosts: p, h1–h6, span, a, label, code, summary, legend, button, input, \
         textarea, option, pre, small, strong, em, b, i, li, th, td, and related \
         prose tags. Move typography onto a child text host (span/p/…), not layout \
         keywords or pure structure (row/column/div/svg/…)."
    )
}

/// True when a `when` key is interaction / component-state (not media/viewport).
///
/// Used for “state can't gate geometry”. Media keys may still gate box.
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
/// law: dimensional box ops (pad/gap/w/h/grow/position axes/…).
///
/// `display` / `overflow*` stay box-channel for taffy but may appear under
/// interaction when for show/hide (empty slots, group-caught swap) without
/// pointer thrash. Dimensional geometry under hover/group-state still fails.
pub fn is_geometry_setter(name: &str) -> bool {
    match name {
        "display" | "overflow" | "overflowX" | "overflowY" | "flow" => false,
        "size" | "space" | "place" => true,
        other => channel_for_setter(other) == Some(Channel::Box),
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
            // `place({ z })` is stacking, not box geometry — z lived on paint
            // before SPEC-2 and must still be legal under interaction when().
            if op.setter == "place" {
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

/// Channel for a product setter name. `None` when unknown / not a product setter.
///
/// Assignment follows job-spec CHANNELS spine + inventory table. OPEN items
/// home only when the job-spec lists them (objectFit/Position → paint;
/// vertical-align `align` → paint as presentation).
pub fn channel_for_setter(name: &str) -> Option<Channel> {
    if let Some(c) = crate::concerns::find_concern(name) {
        return Some(c.channel);
    }
    Some(match name {
        // box (absorbed names kept so retirement still channel-checks)
        "pad" | "padX" | "padY" | "padT" | "padB" | "padL" | "padR" | "padS" | "padE" => {
            Channel::Box
        }
        "m" | "mx" | "my" | "mt" | "mb" | "ms" | "me" | "ml" | "mr" => Channel::Box,
        "minH" | "minW" | "maxH" | "maxW" | "w" | "h" => Channel::Box,
        "overflowX" | "overflowY" => Channel::Box,
        "display" | "position" | "top" | "bottom" | "left" | "right" | "inset" => Channel::Box,
        "basis" | "self" | "flexDir" | "aspect" | "order" | "justifySelf" | "resize" => {
            Channel::Box
        }
        "gap" | "gapX" | "gapY" => Channel::Box,
        "grow" | "shrink" => Channel::Box,

        // text
        "fontSize" | "fontWeight" | "fontFamily" | "leading" | "tracking" | "type" => Channel::Text,
        "whitespace" | "textAlign" | "break" | "tab" | "underline" | "truncate" => Channel::Text,
        "textCase" | "nums" | "italic" | "decoration" | "indent" | "underlineOffset"
        | "antialiased" => Channel::Text,

        // paint (includes gap keys: clip/filter/3D/cq + presentation)
        "bg" | "fg" | "list" | "shadow" | "opacity" => Channel::Paint,
        "border" | "borderX" | "borderY" | "borderS" | "borderE" | "borderT" | "borderB"
        | "borderR" | "borderL" => Channel::Paint,
        "borderWidth" | "borderBWidth" | "borderTWidth" | "borderEWidth" | "borderSWidth"
        | "borderRWidth" | "borderLWidth" => Channel::Paint,
        "radius" | "radiusT" | "radiusB" | "radiusL" | "radiusR" => Channel::Paint,
        "stroke" | "fill" | "ring" | "outline" | "outlineOffset" => Channel::Paint,
        "translateX" | "translateY" | "z" | "cursor" | "select" | "pointerEvents" => Channel::Paint,
        "overscroll" | "appearance" | "content" | "transition" | "duration" | "ease" => {
            Channel::Paint
        }
        "rotate" | "scale" | "tapHighlight" | "saturate" | "group" | "bgImage" | "borderImage"
        | "backdrop" => Channel::Paint,
        "avatarMedia" | "glyphFill" | "scrollbarThumb" | "scrollbarTrack" | "scrollbarThin" => {
            Channel::Paint
        }
        "isolate" | "mixBlend" | "origin" | "textGradient" | "bgGlow" | "srOnly" | "visibility" => {
            Channel::Paint
        }
        "objectFit" | "objectPosition" | "transform" | "caret" | "accent" | "touch" | "scroll" => {
            Channel::Paint
        }
        "willChange" | "contain" | "colorMix" | "colorDeepen" | "controlFrame" | "controlLift"
        | "press" | "boxSizing" | "bgClip" | "animate" | "align" | "elevation" => Channel::Paint,
        // Phase 3 gap keys
        "clipPath" | "mask" | "maskImage" => Channel::Paint,
        "blur" | "brightness" | "contrast" | "grayscale" | "hueRotate" | "invert" | "sepia" => {
            Channel::Paint
        }
        "perspective" | "rotateX" | "rotateY" | "rotateZ" => Channel::Paint,
        "containerType" | "containerName" | "container" => Channel::Paint,

        // Retired product alias — still recognized so parse can emit a pointer.
        "color" => Channel::Paint,

        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn text_bearing_list_covers_spec_and_prose() {
        for t in [
            "p", "h1", "span", "a", "button", "input", "li", "th", "pre", "text",
        ] {
            assert!(is_text_bearing(t), "{t}");
        }
        for t in [
            "row",
            "column",
            "grid",
            "center",
            "stack",
            "div",
            "svg",
            "scroll-area",
            "main",
        ] {
            assert!(!is_text_bearing(t), "{t}");
        }
        assert!(is_text_bearing("BUTTON"));
    }

    #[test]
    fn style_uses_text_detects_channel_setters() {
        let (s, _) = parse("style({ text: type({ size: sm }) })");
        assert!(style_uses_text_channel(&s));
        let (s, _) = parse("style({ pad(2), bg(primary) })");
        assert!(!style_uses_text_channel(&s));
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
    fn geometry_setters_are_dimensional_box() {
        for s in ["size", "space", "place"] {
            assert!(is_geometry_setter(s), "{s}");
        }
        assert!(!is_geometry_setter("display"));
        assert!(!is_geometry_setter("overflow"));
        assert!(!is_geometry_setter("flow"));
        assert!(!is_geometry_setter("fill"));
        assert!(!is_geometry_setter("type"));
    }

    #[test]
    fn geometry_in_interaction_when_refused() {
        let (_, diags) =
            parse("style({ pad(2), when: { hover: { pad(4) } } })");
        assert!(
            diags
                .iter()
                .any(|d| d.message().contains("state can't gate layout")),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, diags) = parse(
            "style({ pad(2), when: { tablet: { pad(4) } } })",
        );
        assert!(
            !diags
                .iter()
                .any(|d| d.message().contains("state can't gate layout")),
            "media may gate box: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, diags) = parse(
            "style({ pad(2), when: { hover: { bg(muted) } } })",
        );
        assert!(
            !diags
                .iter()
                .any(|d| d.message().contains("state can't gate layout")),
            "paint under hover ok: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    /// Geometry-class × interaction matrix (SPEC: state can't gate dimensional box).
    /// Asserts only current product law — display/overflow are non-dimensional box.
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
            let src = format!(
                "style({{ pad(2), when: {{ {key}: {{ pad(4) }} }} }})"
            );
            assert!(gate_err(&src), "pad under {key} must refuse: {src}");
            let src = format!(
                "style({{ box: size({{ w: full }}), when: {{ {key}: {{ box: size({{ h: full }}) }} }} }})"
            );
            assert!(gate_err(&src), "size under {key} must refuse: {src}");
        }

        // Media / preference may still gate box geometry.
        for key in ["tablet", "dark"] {
            let src = format!(
                "style({{ pad(2), when: {{ {key}: {{ pad(4) }} }} }})"
            );
            assert!(!gate_err(&src), "pad under media {key} must allow: {src}");
        }

        // Paint under interaction is legal.
        assert!(!gate_err(
            "style({ bg(primary), when: { hover: { bg(muted) } } })"
        ));
        assert!(!gate_err(
            "style({ pad(2), bg(primary), when: { hover: { bg(muted) } } })"
        ));

        // Non-dimensional box (flow/overflow) under interaction — currently legal
        // (is_geometry_setter excludes them so show/hide slots don't thrash pad law).
        assert!(
            !is_geometry_setter("flow") && !is_geometry_setter("overflow"),
            "flow/overflow stay non-geometry for interaction gate"
        );
        assert!(!gate_err(
            "style({ display(none), when: { hover: { display(block) } } })"
        ));
        assert!(!gate_err(
            "style({ box: overflow({ all: hidden }), when: { group-hover: { box: overflow({ all: auto }) } } })"
        ));
    }

    #[test]
    fn color_setter_refused_with_fg_pointer() {
        let (_, diags) = parse("style({ paint: color(primary) })");
        assert!(
            diags.iter().any(|d| d.message().contains("fg")),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn filter_and_markers_compile_and_lower() {
        let src = "style({ \
            paint: filter({ blur: 4, brightness: 100, contrast: 100, grayscale: 0, \
                hue: 0, invert: 0, sepia: 0, saturate: 100 }).containerType(inline-size).containerName(main), \
            pad(2) \
        })";
        let (style, diags) = parse(src);
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = crate::emit::lower_style(&style, &std::collections::HashMap::new());
        for needle in ["filter", "blur(", "container-type", "container-name"] {
            assert!(css.contains(needle), "missing {needle} in:\n{css}");
        }
    }
}
