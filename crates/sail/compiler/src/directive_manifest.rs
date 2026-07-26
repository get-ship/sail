//! The template-directive vocabulary — one source of truth for every
//! attribute-position directive the parser accepts (`on:`, `bind:`, `use:`,
//! `fx:`, slots, `mutation`), every `on:` event name, every `on:`
//! event modifier, every `bind:` target, and every retired directive with
//! the thing to use instead.
//!
//! A sibling of `block_manifest` / `action_manifest` / `runtime_ambient`:
//! the closed vocabulary is data, not a hand-listed paragraph on the docs
//! page. The parser (`parse::template::attrs`) is the enforcement point;
//! its unknown-event, unknown-modifier and unknown-bind errors render THESE
//! tables, and the tests below / in `parse::template::attrs` hold the
//! parser match arms in lockstep with the rows here.

use serde::Serialize;

/// One attribute-position directive: authored syntax and a one-line summary.
#[derive(Debug, Clone, Serialize)]
pub struct DirectiveEntry {
    /// The directive as authored, e.g. `on:<event>={handler}`.
    pub syntax: &'static str,
    /// One-line description of what it does.
    pub summary: &'static str,
}

/// Attribute-position directive prefixes that win the lexer race against
/// plain attribute names. The grammar generates a dedicated rule for
/// each prefix so `bind:value`, `on:click`, etc. parse as directives
/// rather than as ordinary `attr_name` tokens. Keep this in sync with
/// the `DIRECTIVES` entries whose syntax starts with `<prefix>:`.
pub const DIRECTIVE_PREFIXES: &[&str] = &["bind", "on", "use", "fx"];

/// Element names that are directive-like in the grammar: the parser treats
/// them as ordinary HTML elements, but the highlight query gives them a
/// directive keyword scope so they stand out in editors. Must be kept in
/// sync with the rows in `DIRECTIVES` that describe `<scope>` / `<slot>`.
pub const DIRECTIVE_ELEMENTS: &[&str] = &["scope", "slot"];

/// The directive registry, in the order the docs present them.
pub const DIRECTIVES: &[DirectiveEntry] = &[
    DirectiveEntry {
        syntax: "on:<event>|<modifier>…={handler}",
        summary: "attach a DOM event handler; chain modifiers with | (full modifier table below)",
    },
    DirectiveEntry {
        syntax: "bind:value / bind:checked / bind:this",
        summary: "two-way binding sugar; the three targets are the whole surface (table below)",
    },
    DirectiveEntry {
        syntax: "use:<action> / use:<action>={param}",
        summary: "attach behavior bound to the element's lifecycle; built-ins live in the actions reference, a scope value works too",
    },
    DirectiveEntry {
        syntax: "sx=\"atom atom …\"",
        summary: "static styling atoms from the closed sx vocabulary; sx={expr} is a compile error",
    },
    DirectiveEntry {
        syntax: "fx:<trigger>={…}",
        summary: "the one animation directive — triggers, properties, and config in the motion reference",
    },
    DirectiveEntry {
        syntax: "(--name) inside sx values",
        summary: "read a CSS custom property: any sx atom accepts (--name) and lowers to var(--name); typed actions write them",
    },
    DirectiveEntry {
        syntax: "slot=\"name\" (use site)",
        summary: "assign this subtree to the component's named slot; unassigned children fill the default slot",
    },
    DirectiveEntry {
        syntax: "<slot name=\"…\">fallback</slot>",
        summary: "declare a fill region; children of the <slot> render when the use site provides nothing",
    },
    DirectiveEntry {
        syntax: "<slot reflect-empty=\"data-attr\">",
        summary: "mirror the slot's emptiness onto a static data attribute, live — slot elements only",
    },
    DirectiveEntry {
        syntax: "mutation=\"ship.X.Y\" (on <form>)",
        summary: "wire a form's submit to a typed mutation",
    },
    DirectiveEntry {
        syntax: "&#123; and &#125; in text",
        summary: "a { with no matching } stays literal text; entity-escape a brace to force a literal one — there is no {{ escape",
    },
];

/// One `on:` event modifier. `canonical` is the long form the parser
/// documents; `alias` is the accepted short form when one exists; `arg`
/// names the optional `:arg` payload.
#[derive(Debug, Clone, Serialize)]
pub struct ModifierEntry {
    pub canonical: &'static str,
    pub alias: Option<&'static str>,
    pub arg: Option<&'static str>,
    pub summary: &'static str,
}

/// Every event modifier `on:` accepts, in documentation order. The parser's
/// `parse_modifier` match and this table are held in lockstep by the parity
/// test in `parse::template::attrs`.
pub const EVENT_MODIFIERS: &[ModifierEntry] = &[
    ModifierEntry {
        canonical: "preventDefault",
        alias: Some("prevent"),
        arg: None,
        summary: "call event.preventDefault() before the handler runs",
    },
    ModifierEntry {
        canonical: "stopPropagation",
        alias: Some("stop"),
        arg: None,
        summary: "call event.stopPropagation()",
    },
    ModifierEntry {
        canonical: "stopImmediatePropagation",
        alias: None,
        arg: None,
        summary: "call event.stopImmediatePropagation()",
    },
    ModifierEntry {
        canonical: "self",
        alias: None,
        arg: None,
        summary: "run only when the event's target is this element, not a descendant",
    },
    ModifierEntry {
        canonical: "trusted",
        alias: None,
        arg: None,
        summary: "run only for user-generated events (event.isTrusted)",
    },
    ModifierEntry {
        canonical: "key",
        alias: None,
        arg: Some("Name"),
        summary: "run only when the pressed key matches, e.g. on:keydown|key:Enter",
    },
    ModifierEntry {
        canonical: "debounce",
        alias: None,
        arg: Some("ms"),
        summary: "wait for ms of quiet before running the handler",
    },
    ModifierEntry {
        canonical: "throttle",
        alias: None,
        arg: Some("ms"),
        summary: "run the handler at most once per ms",
    },
];


/// One `on:` event name. Names are web DOM event names (or framework
/// custom events the runtime actually fires). The table is documentation
/// order — common UI events first, then the long tail by category.
#[derive(Debug, Clone, Serialize)]
pub struct EventEntry {
    pub name: &'static str,
    pub summary: &'static str,
}

/// Every `on:` event the parser accepts by exact name, in documentation
/// order. Anything else is a compile error with a did-you-mean hint drawn
/// from this table.
///
/// Framework escape hatches (not every name is a row, still accepted):
/// - `sail-*` / `ship-*` prefixes — component/product custom events
/// - a `:` in the name — namespaced custom-element events (`tag:event`)
///
/// Those prefixes stay open so a new component event does not require a
/// compiler bump; this table is the honest web + known-framework set the
/// docs and completions surface. In-tree `sail-*` emits used by handlers
/// are listed so the reference page is not a bare DOM dump.
pub const EVENT_NAMES: &[EventEntry] = &[
    // ── pointer / mouse ──────────────────────────────────────────────
    EventEntry { name: "click", summary: "primary activation (mouse / pen / touch synthesized click)" },
    EventEntry { name: "dblclick", summary: "double-click" },
    EventEntry { name: "contextmenu", summary: "context-menu gesture (usually right-click)" },
    EventEntry { name: "mousedown", summary: "mouse button pressed" },
    EventEntry { name: "mouseup", summary: "mouse button released" },
    EventEntry { name: "mousemove", summary: "mouse moved" },
    EventEntry { name: "mouseenter", summary: "mouse entered the element (does not bubble)" },
    EventEntry { name: "mouseleave", summary: "mouse left the element (does not bubble)" },
    EventEntry { name: "mouseover", summary: "mouse entered the element or a descendant" },
    EventEntry { name: "mouseout", summary: "mouse left the element or a descendant" },
    EventEntry { name: "wheel", summary: "mouse wheel / trackpad scroll gesture" },
    EventEntry { name: "pointerdown", summary: "pointer (mouse/pen/touch) pressed" },
    EventEntry { name: "pointerup", summary: "pointer released" },
    EventEntry { name: "pointermove", summary: "pointer moved" },
    EventEntry { name: "pointerenter", summary: "pointer entered (does not bubble)" },
    EventEntry { name: "pointerleave", summary: "pointer left (does not bubble)" },
    EventEntry { name: "pointerover", summary: "pointer entered the element or a descendant" },
    EventEntry { name: "pointerout", summary: "pointer left the element or a descendant" },
    EventEntry { name: "pointercancel", summary: "pointer input cancelled by the UA" },
    EventEntry { name: "gotpointercapture", summary: "element received pointer capture" },
    EventEntry { name: "lostpointercapture", summary: "element lost pointer capture" },
    // ── touch ────────────────────────────────────────────────────────
    EventEntry { name: "touchstart", summary: "touch point placed on the surface" },
    EventEntry { name: "touchend", summary: "touch point removed" },
    EventEntry { name: "touchmove", summary: "touch point moved" },
    EventEntry { name: "touchcancel", summary: "touch point cancelled by the UA" },
    // ── keyboard ─────────────────────────────────────────────────────
    EventEntry { name: "keydown", summary: "key pressed (repeats while held)" },
    EventEntry { name: "keyup", summary: "key released" },
    EventEntry { name: "keypress", summary: "character key pressed (deprecated; prefer keydown)" },
    // ── focus ────────────────────────────────────────────────────────
    EventEntry { name: "focus", summary: "element received focus (does not bubble; prefer focusin)" },
    EventEntry { name: "blur", summary: "element lost focus (does not bubble; prefer focusout)" },
    EventEntry { name: "focusin", summary: "element or descendant received focus (bubbles)" },
    EventEntry { name: "focusout", summary: "element or descendant lost focus (bubbles)" },
    // ── form ─────────────────────────────────────────────────────────
    EventEntry { name: "input", summary: "value changed (fires on every edit)" },
    EventEntry { name: "beforeinput", summary: "value is about to change; cancelable" },
    EventEntry { name: "change", summary: "value committed (blur/select/commit)" },
    EventEntry { name: "submit", summary: "form submit attempted" },
    EventEntry { name: "reset", summary: "form reset" },
    EventEntry { name: "invalid", summary: "form control failed constraint validation" },
    EventEntry { name: "select", summary: "text selection changed inside an input/textarea" },
    EventEntry { name: "formdata", summary: "FormData is being constructed on submit" },
    // ── drag ─────────────────────────────────────────────────────────
    EventEntry { name: "dragstart", summary: "drag operation began" },
    EventEntry { name: "drag", summary: "drag source is being dragged" },
    EventEntry { name: "dragend", summary: "drag operation ended" },
    EventEntry { name: "dragenter", summary: "drag entered a drop target" },
    EventEntry { name: "dragleave", summary: "drag left a drop target" },
    EventEntry { name: "dragover", summary: "drag is over a drop target (call preventDefault to allow drop)" },
    EventEntry { name: "drop", summary: "drop completed on a target" },
    // ── clipboard ────────────────────────────────────────────────────
    EventEntry { name: "copy", summary: "copy to clipboard" },
    EventEntry { name: "cut", summary: "cut to clipboard" },
    EventEntry { name: "paste", summary: "paste from clipboard" },
    // ── composition (IME) ────────────────────────────────────────────
    EventEntry { name: "compositionstart", summary: "IME composition began" },
    EventEntry { name: "compositionupdate", summary: "IME composition updated" },
    EventEntry { name: "compositionend", summary: "IME composition finished" },
    // ── media ────────────────────────────────────────────────────────
    EventEntry { name: "play", summary: "media playback started" },
    EventEntry { name: "pause", summary: "media playback paused" },
    EventEntry { name: "ended", summary: "media reached the end" },
    EventEntry { name: "timeupdate", summary: "media currentTime changed" },
    EventEntry { name: "volumechange", summary: "media volume or mute changed" },
    EventEntry { name: "ratechange", summary: "media playbackRate changed" },
    EventEntry { name: "loadeddata", summary: "media frame data loaded at current position" },
    EventEntry { name: "loadedmetadata", summary: "media metadata loaded" },
    EventEntry { name: "canplay", summary: "media can start playing" },
    EventEntry { name: "canplaythrough", summary: "media can play through without buffering" },
    EventEntry { name: "stalled", summary: "media data unexpectedly not available" },
    EventEntry { name: "suspend", summary: "media data loading suspended" },
    EventEntry { name: "waiting", summary: "playback stopped pending data" },
    EventEntry { name: "seeking", summary: "seek operation began" },
    EventEntry { name: "seeked", summary: "seek operation completed" },
    EventEntry { name: "durationchange", summary: "media duration changed" },
    EventEntry { name: "emptied", summary: "media became empty (e.g. network error / reload)" },
    EventEntry { name: "abort", summary: "loading aborted" },
    EventEntry { name: "error", summary: "resource failed to load (media, img, …)" },
    // ── window / document lifecycle ──────────────────────────────────
    EventEntry { name: "load", summary: "resource finished loading" },
    EventEntry { name: "unload", summary: "document is unloading" },
    EventEntry { name: "beforeunload", summary: "document is about to unload (may prompt)" },
    EventEntry { name: "resize", summary: "viewport resized" },
    EventEntry { name: "scroll", summary: "element scrolled (does not bubble; listen on the scroller)" },
    EventEntry { name: "scrollend", summary: "scroll sequence finished" },
    EventEntry { name: "hashchange", summary: "location.hash changed" },
    EventEntry { name: "popstate", summary: "session history changed (back/forward)" },
    EventEntry { name: "pagehide", summary: "page is being hidden (bfcache-aware)" },
    EventEntry { name: "pageshow", summary: "page is being shown (bfcache-aware)" },
    EventEntry { name: "visibilitychange", summary: "document.visibilityState changed" },
    EventEntry { name: "online", summary: "browser went online" },
    EventEntry { name: "offline", summary: "browser went offline" },
    // ── animation / transition ───────────────────────────────────────
    EventEntry { name: "animationstart", summary: "CSS animation started" },
    EventEntry { name: "animationend", summary: "CSS animation finished" },
    EventEntry { name: "animationiteration", summary: "CSS animation completed one iteration" },
    EventEntry { name: "animationcancel", summary: "CSS animation cancelled" },
    EventEntry { name: "transitionstart", summary: "CSS transition started" },
    EventEntry { name: "transitionend", summary: "CSS transition finished" },
    EventEntry { name: "transitionrun", summary: "CSS transition created / running" },
    EventEntry { name: "transitioncancel", summary: "CSS transition cancelled" },
    // ── dialog / details ─────────────────────────────────────────────
    EventEntry { name: "toggle", summary: "details/dialog open state toggled" },
    EventEntry { name: "close", summary: "dialog closed" },
    EventEntry { name: "cancel", summary: "dialog cancel (Escape) or similar" },
    // ── pointer lock / misc ──────────────────────────────────────────
    EventEntry { name: "pointerlockchange", summary: "pointer lock acquired or released" },
    EventEntry { name: "pointerlockerror", summary: "pointer lock request failed" },
    EventEntry { name: "message", summary: "message received (window/worker/BroadcastChannel)" },
    EventEntry { name: "messageerror", summary: "message could not be deserialized" },
    EventEntry { name: "storage", summary: "another document changed localStorage/sessionStorage" },
    // ── framework custom (no sail-/ship- prefix) ──────────────────────
    EventEntry {
        name: "success",
        summary: "form mutation settled ok — fired by <form mutation=…> on its host",
    },
    EventEntry {
        name: "failure",
        summary: "form mutation settled with an error — fired by <form mutation=…> on its host",
    },
    // ── sail-* custom events used in-tree ────────────────────────────
    EventEntry { name: "sail-change", summary: "generic value change from a sail control" },
    EventEntry { name: "sail-combobox-select", summary: "combobox option selected" },
    EventEntry { name: "sail-tab-change", summary: "active tab changed" },
    EventEntry { name: "sail-page-change", summary: "pagination page changed" },
    EventEntry { name: "sail-variant-change", summary: "variant selection changed" },
    EventEntry { name: "sail-remove", summary: "item remove requested" },
    EventEntry { name: "sail-error-reset", summary: "error boundary reset requested" },
    EventEntry { name: "sail-toggle", summary: "toggle pressed" },
    EventEntry { name: "sail-toggle-change", summary: "toggle checked state changed" },
    EventEntry { name: "sail-select", summary: "selection changed" },
    EventEntry { name: "sail-menu-select", summary: "menu item selected" },
    EventEntry { name: "sail-dismiss", summary: "overlay/toast dismissed" },
    EventEntry { name: "sail-toast-dismiss", summary: "toast dismissed" },
    EventEntry { name: "sail-cancel", summary: "cancel action (e.g. composer)" },
    EventEntry { name: "sail-save", summary: "save action" },
    EventEntry { name: "sail-composer-submit", summary: "chat composer submitted" },
    EventEntry { name: "sail-citation-open", summary: "citation pill activated" },
    EventEntry { name: "sail-theme-change", summary: "theme selection changed" },
    EventEntry { name: "sail-date-change", summary: "date picker value changed" },
    EventEntry { name: "sail-range-change", summary: "range control value changed" },
    EventEntry { name: "sail-view-change", summary: "view mode changed" },
    EventEntry { name: "sail-error-caught", summary: "error boundary caught an error" },
    EventEntry { name: "sail-mermaid-error", summary: "mermaid diagram failed to render" },
    EventEntry { name: "sail-canvas-viewport", summary: "canvas pan/zoom viewport changed" },
    EventEntry { name: "sail-board-change", summary: "board model mutated" },
    EventEntry { name: "sail-node-select", summary: "board node selection changed" },
    EventEntry { name: "sail-edge-create", summary: "board edge created via port drag" },
    EventEntry { name: "sail-draw-change", summary: "draw scene mutated" },
    EventEntry { name: "sail-draw-select", summary: "draw selection changed" },
    EventEntry { name: "sail-annotation-staged", summary: "annotation staged for send" },
    EventEntry { name: "sail-annotation-sent", summary: "annotation POST succeeded" },
    EventEntry { name: "sail-annotation-error", summary: "annotation POST failed" },
    EventEntry { name: "sail-annotation-remove", summary: "annotation remove requested" },
    EventEntry { name: "sail-event-click", summary: "calendar event clicked" },
    EventEntry { name: "sail-event-select", summary: "calendar event selected" },
    EventEntry { name: "sail-event-move", summary: "calendar event moved" },
    EventEntry { name: "sail-event-create-intent", summary: "calendar empty-slot create intent" },
];

/// True when `name` is an accepted `on:` event: an `EVENT_NAMES` row, a
/// `sail-*` / `ship-*` custom event, or a namespaced `tag:event` name.
pub fn is_known_event(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if EVENT_NAMES.iter().any(|e| e.name == name) {
        return true;
    }
    // Framework / product custom-event prefixes — open so component authors
    // do not need a compiler bump for every new emit.
    if name.starts_with("sail-") || name.starts_with("ship-") {
        return name.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b':' || b == b'.'
        });
    }
    // Namespaced custom-element event (`sail-rail:toggle`, `dialog:open`).
    if name.contains(':') {
        return name.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b':' || b == b'.'
        });
    }
    false
}

/// Closest `EVENT_NAMES` entry to `name` by Levenshtein distance, or
/// `None` when nothing is close enough to be a plausible typo.
pub fn suggest_event(name: &str) -> Option<&'static str> {
    let name_l = name.to_ascii_lowercase();
    let mut best: Option<(usize, &'static str)> = None;
    for e in EVENT_NAMES {
        let d = levenshtein(&name_l, e.name);
        let max_d = (e.name.len().max(name_l.len()) / 3).max(1);
        if d <= max_d && best.is_none_or(|(b, _)| d < b) {
            best = Some((d, e.name));
        }
    }
    best.map(|(_, s)| s)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// One `bind:` target.
#[derive(Debug, Clone, Serialize)]
pub struct BindTargetEntry {
    pub target: &'static str,
    pub summary: &'static str,
}

/// Every `bind:` target. Anything else is a compile error that renders
/// this list.
pub const BIND_TARGETS: &[BindTargetEntry] = &[
    BindTargetEntry { target: "value", summary: "two-way binding to the element's value" },
    BindTargetEntry {
        target: "checked",
        summary: "two-way binding to a checkbox or radio checked state",
    },
    BindTargetEntry {
        target: "this",
        summary: "capture the element handle into a bare scope variable — write-once, not derivable",
    },
];

/// One retired directive and its replacement. The parser rejects these
/// with an error naming the replacement; this table is the docs' copy of
/// the same redirect.
#[derive(Debug, Clone, Serialize)]
pub struct RetiredEntry {
    pub syntax: &'static str,
    pub instead: &'static str,
}

/// Every directive the parser refuses by name, with what to use instead.
pub const RETIRED_DIRECTIVES: &[RetiredEntry] = &[
    RetiredEntry {
        syntax: "prop:<name>",
        instead: "write plain name={expr}; the compiler/runtime lowers field → property or attribute from contracts + value shape — no author attr/property split",
    },
    RetiredEntry {
        syntax: "attr:<name> / attr:{nameExpr}",
        instead: "write plain name={expr}; dynamic field names are not an author directive — fixed-name fields only",
    },
    RetiredEntry {
        syntax: "class:<name>",
        instead: "styling is sx (or a named use: action writing a custom property sx reads via (--name)); state is aria-*/closed sx state prefixes/layout — not a global CSS class toggle. Bare class= is also banned",
    },
    RetiredEntry {
        syntax: "style:<prop>",
        instead: "runtime geometry is owned by named, typed actions (use:rect, use:pin, use:virtualRow, …) — sx reads the variables they write via (--name)",
    },
    RetiredEntry {
        syntax: "part=\"…\"",
        instead: "style with sx on the element itself",
    },
    RetiredEntry {
        syntax: "transition: / in: / out:",
        instead: "fx:enter / fx:exit",
    },
];

/// One value form the `fx:` config grammar accepts.
#[derive(Debug, Clone, Serialize)]
pub struct MotionValueForm {
    pub form: &'static str,
    pub summary: &'static str,
}

/// The value grammar inside a `fx:` config object — what each shape of
/// value means. Enforced by `parse::template::attrs::parse_motion_value`.
pub const MOTION_VALUE_FORMS: &[MotionValueForm] = &[
    MotionValueForm {
        form: "'quoted'",
        summary: "a literal: a preset name, an easing keyword, or a unit-bearing magnitude like '120px'",
    },
    MotionValueForm {
        form: "bare number",
        summary: "milliseconds for time keys (duration, delay, stagger); a native magnitude for properties",
    },
    MotionValueForm {
        form: "[x1, y1, x2, y2]",
        summary: "an inline cubic-bezier easing (x values in 0..1)",
    },
    MotionValueForm {
        form: "unquoted expression",
        summary: "a reactive expression — how a signal drives motion; unit-bearing literals must be quoted",
    },
    MotionValueForm {
        form: "{ from, to, preset, duration, delay, easing, iterations, direction }",
        summary: "per-channel config object; preset supplies defaults, explicit keys override it",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_unique() {
        let mut names: Vec<&str> = EVENT_MODIFIERS.iter().map(|m| m.canonical).collect();
        names.extend(EVENT_MODIFIERS.iter().filter_map(|m| m.alias));
        let before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), before, "modifier names and aliases must be unique");

        let mut targets: Vec<&str> = BIND_TARGETS.iter().map(|b| b.target).collect();
        targets.sort_unstable();
        targets.dedup();
        assert_eq!(targets.len(), BIND_TARGETS.len(), "bind targets must be unique");

        let mut events: Vec<&str> = EVENT_NAMES.iter().map(|e| e.name).collect();
        let before = events.len();
        events.sort_unstable();
        events.dedup();
        assert_eq!(events.len(), before, "event names must be unique");

        let mut syntaxes: Vec<&str> = DIRECTIVES.iter().map(|d| d.syntax).collect();
        syntaxes.sort_unstable();
        syntaxes.dedup();
        assert_eq!(syntaxes.len(), DIRECTIVES.len(), "directive rows must be unique");
    }

    #[test]
    fn known_events_are_accepted_and_typos_suggest() {
        assert!(is_known_event("click"));
        assert!(is_known_event("sail-change"));
        assert!(is_known_event("sail-rail:toggle"));
        assert!(is_known_event("ship-thing:done"));
        assert!(is_known_event("success"));
        assert!(!is_known_event("clik"));
        assert!(!is_known_event("Click"));
        assert_eq!(suggest_event("clik"), Some("click"));
        assert_eq!(suggest_event("Click"), Some("click"));
    }
}
