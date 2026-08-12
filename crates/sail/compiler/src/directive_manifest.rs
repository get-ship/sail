//! The template-directive vocabulary — one source of truth for every
//! attribute-position directive the parser accepts (`on:`, `bind:`, `use:`,
//! slots, `mutation`), every `on:` event name, every `on:`
//! event modifier, every `bind:` target, and permanent foreign-idiom
//! teaching (HTML/CSS/JS habits → sail way).
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
/// the *active* `DIRECTIVES` entries whose syntax starts with `<prefix>:`.
/// Unknown colon-prefixes fail with the generic unknown-vocabulary error
/// that lists these; foreign HTML/CSS habits live in [`FOREIGN_IDIOMS`].
pub const DIRECTIVE_PREFIXES: &[&str] = &["bind", "on", "use"];

/// Element names that are directive-like in the grammar: the parser treats
/// them as ordinary HTML elements, but the highlight query gives them a
/// directive keyword scope so they stand out in editors. Must be kept in
/// sync with the rows in `DIRECTIVES` that describe `<scope>` / `<slot>` /
/// `<partial>`.
pub const DIRECTIVE_ELEMENTS: &[&str] = &["scope", "slot", "partial"];

/// The directive registry, in the order the docs present them.
pub const DIRECTIVES: &[DirectiveEntry] = &[
    DirectiveEntry {
        syntax: "on:<event>|<modifier>…={handler}",
        summary: "attach a DOM event handler; chain modifiers with | (full modifier table below)",
    },
    DirectiveEntry {
        syntax: "bind:value / bind:checked",
        summary: "two-way binding sugar; value and checked are the whole surface (table below)",
    },
    DirectiveEntry {
        syntax: "use:<action> / use:<action>={param}",
        summary: "attach behavior bound to the element's lifecycle; built-ins live in the actions reference, a scope value works too",
    },
    DirectiveEntry {
        syntax: "style({…}) / style(name, {…}) / style(partial, …) tag call",
        summary: "Style product paint on the open tag — style({…}).when(…) call form, not an HTML style= attribute",
    },
    DirectiveEntry {
        syntax: "(--name) inside Style values",
        summary: "read a CSS custom property as any Style setter argument; lowers to var(--name); typed actions write them",
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
        syntax: "<partial kebab-name props: Type>…</partial>",
        summary: "in-file functional component (same file only); invoke as <kebab-name …>; expanded at compile time; params are a plain scope type read as bare {title}",
    },
    DirectiveEntry {
        syntax: "<slot reflect-empty=\"data-attr\">",
        summary: "mirror the slot's emptiness onto a static data attribute, live — slot elements only",
    },
    DirectiveEntry {
        syntax: "mutation=\"client.method\" (on <form>)",
        summary: "wire a form's submit to a injected client method",
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
/// **DOM events only.** Component moments are typed callback props
/// (`on-select={fn}`), not CustomEvents. Form mutation moments are
/// `on-success` / `on-failure` callback props on `<form mutation=…>`.
/// There is no author-facing custom-event bus.
pub const EVENT_NAMES: &[EventEntry] = &[
    // ── pointer / mouse ──────────────────────────────────────────────
    EventEntry {
        name: "click",
        summary: "primary activation (mouse / pen / touch synthesized click)",
    },
    EventEntry {
        name: "dblclick",
        summary: "double-click",
    },
    EventEntry {
        name: "contextmenu",
        summary: "context-menu gesture (usually right-click)",
    },
    EventEntry {
        name: "mousedown",
        summary: "mouse button pressed",
    },
    EventEntry {
        name: "mouseup",
        summary: "mouse button released",
    },
    EventEntry {
        name: "mousemove",
        summary: "mouse moved",
    },
    EventEntry {
        name: "mouseenter",
        summary: "mouse entered the element (does not bubble)",
    },
    EventEntry {
        name: "mouseleave",
        summary: "mouse left the element (does not bubble)",
    },
    EventEntry {
        name: "mouseover",
        summary: "mouse entered the element or a descendant",
    },
    EventEntry {
        name: "mouseout",
        summary: "mouse left the element or a descendant",
    },
    EventEntry {
        name: "wheel",
        summary: "mouse wheel / trackpad scroll gesture",
    },
    EventEntry {
        name: "pointerdown",
        summary: "pointer (mouse/pen/touch) pressed",
    },
    EventEntry {
        name: "pointerup",
        summary: "pointer released",
    },
    EventEntry {
        name: "pointermove",
        summary: "pointer moved",
    },
    EventEntry {
        name: "pointerenter",
        summary: "pointer entered (does not bubble)",
    },
    EventEntry {
        name: "pointerleave",
        summary: "pointer left (does not bubble)",
    },
    EventEntry {
        name: "pointerover",
        summary: "pointer entered the element or a descendant",
    },
    EventEntry {
        name: "pointerout",
        summary: "pointer left the element or a descendant",
    },
    EventEntry {
        name: "pointercancel",
        summary: "pointer input cancelled by the UA",
    },
    EventEntry {
        name: "gotpointercapture",
        summary: "element received pointer capture",
    },
    EventEntry {
        name: "lostpointercapture",
        summary: "element lost pointer capture",
    },
    // ── touch ────────────────────────────────────────────────────────
    EventEntry {
        name: "touchstart",
        summary: "touch point placed on the surface",
    },
    EventEntry {
        name: "touchend",
        summary: "touch point removed",
    },
    EventEntry {
        name: "touchmove",
        summary: "touch point moved",
    },
    EventEntry {
        name: "touchcancel",
        summary: "touch point cancelled by the UA",
    },
    // ── keyboard ─────────────────────────────────────────────────────
    EventEntry {
        name: "keydown",
        summary: "key pressed (repeats while held)",
    },
    EventEntry {
        name: "keyup",
        summary: "key released",
    },
    EventEntry {
        name: "keypress",
        summary: "character key pressed (deprecated; prefer keydown)",
    },
    // ── focus ────────────────────────────────────────────────────────
    EventEntry {
        name: "focus",
        summary: "element received focus (does not bubble; prefer focusin)",
    },
    EventEntry {
        name: "blur",
        summary: "element lost focus (does not bubble; prefer focusout)",
    },
    EventEntry {
        name: "focusin",
        summary: "element or descendant received focus (bubbles)",
    },
    EventEntry {
        name: "focusout",
        summary: "element or descendant lost focus (bubbles)",
    },
    // ── form ─────────────────────────────────────────────────────────
    EventEntry {
        name: "input",
        summary: "value changed (fires on every edit)",
    },
    EventEntry {
        name: "beforeinput",
        summary: "value is about to change; cancelable",
    },
    EventEntry {
        name: "change",
        summary: "value committed (blur/select/commit)",
    },
    EventEntry {
        name: "submit",
        summary: "form submit attempted",
    },
    EventEntry {
        name: "reset",
        summary: "form reset",
    },
    EventEntry {
        name: "invalid",
        summary: "form control failed constraint validation",
    },
    EventEntry {
        name: "select",
        summary: "text selection changed inside an input/textarea",
    },
    EventEntry {
        name: "formdata",
        summary: "FormData is being constructed on submit",
    },
    // ── drag ─────────────────────────────────────────────────────────
    EventEntry {
        name: "dragstart",
        summary: "drag operation began",
    },
    EventEntry {
        name: "drag",
        summary: "drag source is being dragged",
    },
    EventEntry {
        name: "dragend",
        summary: "drag operation ended",
    },
    EventEntry {
        name: "dragenter",
        summary: "drag entered a drop target",
    },
    EventEntry {
        name: "dragleave",
        summary: "drag left a drop target",
    },
    EventEntry {
        name: "dragover",
        summary: "drag is over a drop target (call preventDefault to allow drop)",
    },
    EventEntry {
        name: "drop",
        summary: "drop completed on a target",
    },
    // ── clipboard ────────────────────────────────────────────────────
    EventEntry {
        name: "copy",
        summary: "copy to clipboard",
    },
    EventEntry {
        name: "cut",
        summary: "cut to clipboard",
    },
    EventEntry {
        name: "paste",
        summary: "paste from clipboard",
    },
    // ── composition (IME) ────────────────────────────────────────────
    EventEntry {
        name: "compositionstart",
        summary: "IME composition began",
    },
    EventEntry {
        name: "compositionupdate",
        summary: "IME composition updated",
    },
    EventEntry {
        name: "compositionend",
        summary: "IME composition finished",
    },
    // ── media ────────────────────────────────────────────────────────
    EventEntry {
        name: "play",
        summary: "media playback started",
    },
    EventEntry {
        name: "pause",
        summary: "media playback paused",
    },
    EventEntry {
        name: "ended",
        summary: "media reached the end",
    },
    EventEntry {
        name: "timeupdate",
        summary: "media currentTime changed",
    },
    EventEntry {
        name: "volumechange",
        summary: "media volume or mute changed",
    },
    EventEntry {
        name: "ratechange",
        summary: "media playbackRate changed",
    },
    EventEntry {
        name: "loadeddata",
        summary: "media frame data loaded at current position",
    },
    EventEntry {
        name: "loadedmetadata",
        summary: "media metadata loaded",
    },
    EventEntry {
        name: "canplay",
        summary: "media can start playing",
    },
    EventEntry {
        name: "canplaythrough",
        summary: "media can play through without buffering",
    },
    EventEntry {
        name: "stalled",
        summary: "media data unexpectedly not available",
    },
    EventEntry {
        name: "suspend",
        summary: "media data loading suspended",
    },
    EventEntry {
        name: "waiting",
        summary: "playback stopped pending data",
    },
    EventEntry {
        name: "seeking",
        summary: "seek operation began",
    },
    EventEntry {
        name: "seeked",
        summary: "seek operation completed",
    },
    EventEntry {
        name: "durationchange",
        summary: "media duration changed",
    },
    EventEntry {
        name: "emptied",
        summary: "media became empty (e.g. network error / reload)",
    },
    EventEntry {
        name: "abort",
        summary: "loading aborted",
    },
    EventEntry {
        name: "error",
        summary: "resource failed to load (media, img, …)",
    },
    // ── window / document lifecycle ──────────────────────────────────
    EventEntry {
        name: "load",
        summary: "resource finished loading",
    },
    EventEntry {
        name: "unload",
        summary: "document is unloading",
    },
    EventEntry {
        name: "beforeunload",
        summary: "document is about to unload (may prompt)",
    },
    EventEntry {
        name: "resize",
        summary: "viewport resized",
    },
    EventEntry {
        name: "scroll",
        summary: "element scrolled (does not bubble; listen on the scroller)",
    },
    EventEntry {
        name: "scrollend",
        summary: "scroll sequence finished",
    },
    EventEntry {
        name: "hashchange",
        summary: "location.hash changed",
    },
    EventEntry {
        name: "popstate",
        summary: "session history changed (back/forward)",
    },
    EventEntry {
        name: "pagehide",
        summary: "page is being hidden (bfcache-aware)",
    },
    EventEntry {
        name: "pageshow",
        summary: "page is being shown (bfcache-aware)",
    },
    EventEntry {
        name: "visibilitychange",
        summary: "document.visibilityState changed",
    },
    EventEntry {
        name: "online",
        summary: "browser went online",
    },
    EventEntry {
        name: "offline",
        summary: "browser went offline",
    },
    // ── animation / transition ───────────────────────────────────────
    EventEntry {
        name: "animationstart",
        summary: "CSS animation started",
    },
    EventEntry {
        name: "animationend",
        summary: "CSS animation finished",
    },
    EventEntry {
        name: "animationiteration",
        summary: "CSS animation completed one iteration",
    },
    EventEntry {
        name: "animationcancel",
        summary: "CSS animation cancelled",
    },
    EventEntry {
        name: "transitionstart",
        summary: "CSS transition started",
    },
    EventEntry {
        name: "transitionend",
        summary: "CSS transition finished",
    },
    EventEntry {
        name: "transitionrun",
        summary: "CSS transition created / running",
    },
    EventEntry {
        name: "transitioncancel",
        summary: "CSS transition cancelled",
    },
    // ── dialog / details ─────────────────────────────────────────────
    EventEntry {
        name: "toggle",
        summary: "details/dialog open state toggled",
    },
    EventEntry {
        name: "close",
        summary: "dialog closed",
    },
    EventEntry {
        name: "cancel",
        summary: "dialog cancel (Escape) or similar",
    },
    // ── pointer lock / misc ──────────────────────────────────────────
    EventEntry {
        name: "pointerlockchange",
        summary: "pointer lock acquired or released",
    },
    EventEntry {
        name: "pointerlockerror",
        summary: "pointer lock request failed",
    },
    EventEntry {
        name: "message",
        summary: "message received (window/worker/BroadcastChannel)",
    },
    EventEntry {
        name: "messageerror",
        summary: "message could not be deserialized",
    },
    EventEntry {
        name: "storage",
        summary: "another document changed localStorage/sessionStorage",
    },
];

/// True when `name` is an accepted `on:` event — an `EVENT_NAMES` row
/// (standard DOM events only). Custom `sail-*` / `ship-*` / `tag:event`
/// names are refused; component moments are typed `on-*` callback props.
pub fn is_known_event(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    EVENT_NAMES.iter().any(|e| e.name == name)
}

/// Hint when an author reaches for a retired custom-event name via `on:`.
/// Maps common former `sail-*` event names to the callback prop they became.
pub fn custom_event_callback_hint(name: &str) -> Option<&'static str> {
    let bare = name
        .strip_prefix("sail-")
        .or_else(|| name.strip_prefix("ship-"))
        .unwrap_or(name);
    // Form mutation moments (were bare `success`/`failure` CustomEvents).
    if matches!(name, "success" | "failure") {
        return Some(if name == "success" {
            "on-success={handler}"
        } else {
            "on-failure={handler}"
        });
    }
    // Former sail-*/ship-* events → on-<rest> callback prop.
    if name.starts_with("sail-") || name.starts_with("ship-") {
        // A few renames where the prop name differs from the event tail.
        let prop = match bare {
            "toast-dismiss" => "on-dismiss",
            "menu-select" | "combobox-select" => "on-select",
            "composer-submit" => "on-submit",
            "citation-open" => "on-open",
            "error-reset" => "on-error-reset",
            "error-caught" => "on-error-caught",
            other => {
                // Generic: sail-page-change → on-page-change
                // Build is static-only; return a shared template via match
                // for the common tails used in-tree, else generic message.
                match other {
                    "change" => "on-change",
                    "close" => "on-close",
                    "dismiss" => "on-dismiss",
                    "remove" => "on-remove",
                    "select" => "on-select",
                    "save" => "on-save",
                    "cancel" => "on-cancel",
                    "tab-change" => "on-tab-change",
                    "page-change" => "on-page-change",
                    "toggle" => "on-toggle",
                    "toggle-change" => "on-toggle-change",
                    "variant-change" => "on-variant-change",
                    "theme-change" => "on-theme-change",
                    "date-change" => "on-change",
                    "range-change" => "on-change",
                    "view-change" => "on-change",
                    "mermaid-error" => "on-error",
                    "canvas-viewport" => "on-viewport-change",
                    "board-change" => "on-change",
                    "node-select" => "on-select",
                    "edge-create" => "on-edge-create",
                    "draw-change" => "on-change",
                    "draw-select" => "on-select",
                    "annotation-staged" => "on-staged",
                    "annotation-sent" => "on-sent",
                    "annotation-error" => "on-error",
                    "annotation-remove" => "on-remove",
                    "event-click" => "on-event-click",
                    "event-select" => "on-event-select",
                    "event-move" => "on-event-move",
                    "event-create-intent" => "on-create-intent",
                    _ => "on-…",
                }
            }
        };
        if prop == "on-…" {
            return Some("a typed on-* callback prop (props flow down; callbacks report up)");
        }
        // Leak a stable string: format into the error message at the call site
        // with the prop name. We return the prop attr form.
        // Use a static table so we can return &'static str.
        return Some(match prop {
            "on-dismiss" => "on-dismiss={handler}",
            "on-select" => "on-select={handler}",
            "on-submit" => "on-submit={handler}",
            "on-open" => "on-open={handler}",
            "on-error-reset" => "on-error-reset={handler}",
            "on-error-caught" => "on-error-caught={handler}",
            "on-change" => "on-change={handler}",
            "on-close" => "on-close={handler}",
            "on-remove" => "on-remove={handler}",
            "on-save" => "on-save={handler}",
            "on-cancel" => "on-cancel={handler}",
            "on-tab-change" => "on-tab-change={handler}",
            "on-page-change" => "on-page-change={handler}",
            "on-toggle" => "on-toggle={handler}",
            "on-toggle-change" => "on-toggle-change={handler}",
            "on-variant-change" => "on-variant-change={handler}",
            "on-theme-change" => "on-theme-change={handler}",
            "on-error" => "on-error={handler}",
            "on-viewport-change" => "on-viewport-change={handler}",
            "on-edge-create" => "on-edge-create={handler}",
            "on-staged" => "on-staged={handler}",
            "on-sent" => "on-sent={handler}",
            "on-event-click" => "on-event-click={handler}",
            "on-event-select" => "on-event-select={handler}",
            "on-event-move" => "on-event-move={handler}",
            "on-create-intent" => "on-create-intent={handler}",
            _ => "a typed on-* callback prop",
        });
    }
    None
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
    BindTargetEntry {
        target: "value",
        summary: "two-way binding to the element's value",
    },
    BindTargetEntry {
        target: "checked",
        summary: "two-way binding to a checkbox or radio checked state",
    },
];

/// One foreign-language habit and the sail way. Permanent teaching for
/// authors reaching for HTML/CSS/JS idioms — not a tombstone for our own
/// retired APIs (those are scrubbed out of existence).
#[derive(Debug, Clone, Serialize)]
pub struct ForeignIdiomEntry {
    pub syntax: &'static str,
    pub instead: &'static str,
}

/// Foreign idioms the parser and docs teach against. Own-API ghosts are not
/// listed here — after a scrub they simply do not exist (generic
/// unknown-vocabulary handles them).
pub const FOREIGN_IDIOMS: &[ForeignIdiomEntry] = &[
    ForeignIdiomEntry {
        syntax: "class= / class:<name>",
        instead: "styling is a tag-position style({…}) / style(name) / style(partial, …) call on the element",
    },
    ForeignIdiomEntry {
        syntax: "style= / style:<prop>",
        instead: "runtime geometry is owned by named, typed actions; paint is a tag-position style({…}) call, not an HTML style attribute",
    },
    ForeignIdiomEntry {
        syntax: "part=\"…\"",
        instead: "style with a tag-position style({…}) call on the element itself — light DOM has no shadow parts",
    },
    ForeignIdiomEntry {
        syntax: "<div> / <span> for layout structure",
        instead: "layout keywords <column>, <row>, <grid>, <stack>, <center> with axis attrs (gap, align, justify); participation is box channel style({ box: grow(1).minW(0) })",
    },
    ForeignIdiomEntry {
        syntax: "document.querySelector / getElementById in a controller",
        instead: "a local use: action that captures the node on mount (and clears on destroy), or closed-over state + on: handlers — controllers do not reach into the DOM by selector",
    },
];

/// One value form the motion config grammar accepts (compiler-private Style
/// motion attach from `.enter` / `.exit` / …).
#[derive(Debug, Clone, Serialize)]
pub struct MotionValueForm {
    pub form: &'static str,
    pub summary: &'static str,
}

/// The value grammar inside a motion config object — what each shape of
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
        assert_eq!(
            names.len(),
            before,
            "modifier names and aliases must be unique"
        );

        let mut targets: Vec<&str> = BIND_TARGETS.iter().map(|b| b.target).collect();
        targets.sort_unstable();
        targets.dedup();
        assert_eq!(
            targets.len(),
            BIND_TARGETS.len(),
            "bind targets must be unique"
        );

        let mut events: Vec<&str> = EVENT_NAMES.iter().map(|e| e.name).collect();
        let before = events.len();
        events.sort_unstable();
        events.dedup();
        assert_eq!(events.len(), before, "event names must be unique");

        let mut syntaxes: Vec<&str> = DIRECTIVES.iter().map(|d| d.syntax).collect();
        syntaxes.sort_unstable();
        syntaxes.dedup();
        assert_eq!(
            syntaxes.len(),
            DIRECTIVES.len(),
            "directive rows must be unique"
        );
    }

    #[test]
    fn known_events_are_accepted_and_typos_suggest() {
        assert!(is_known_event("click"));
        assert!(is_known_event("input"));
        assert!(is_known_event("close"));
        // Custom-event bus is gone — DOM only.
        assert!(!is_known_event("sail-change"));
        assert!(!is_known_event("sail-rail:toggle"));
        assert!(!is_known_event("ship-thing:done"));
        assert!(!is_known_event("success"));
        assert!(!is_known_event("failure"));
        assert!(!is_known_event("clik"));
        assert!(!is_known_event("Click"));
        assert_eq!(suggest_event("clik"), Some("click"));
        assert_eq!(suggest_event("Click"), Some("click"));
        assert_eq!(
            custom_event_callback_hint("sail-remove"),
            Some("on-remove={handler}")
        );
        assert_eq!(
            custom_event_callback_hint("success"),
            Some("on-success={handler}")
        );
    }

    #[test]
    fn event_names_are_dom_only() {
        for e in EVENT_NAMES {
            assert!(
                !e.name.starts_with("sail-") && !e.name.starts_with("ship-"),
                "EVENT_NAMES must not list custom events: {}",
                e.name
            );
            assert!(
                !e.name.contains(':'),
                "EVENT_NAMES must not list namespaced tag:event: {}",
                e.name
            );
            assert!(
                !matches!(e.name, "success" | "failure"),
                "form moments are on-success/on-failure callback props, not events"
            );
        }
    }

    /// Active lexer prefixes must match live DIRECTIVES rows; dead prefixes
    /// must not reappear as active vocabulary.
    #[test]
    fn directive_prefixes_match_active_directives() {
        for prefix in DIRECTIVE_PREFIXES {
            let needle = format!("{prefix}:");
            assert!(
                DIRECTIVES.iter().any(|d| d.syntax.contains(&needle)),
                "DIRECTIVE_PREFIXES has `{prefix}` but no DIRECTIVES row contains `{needle}`"
            );
        }
        for d in DIRECTIVES {
            let Some((raw_head, _)) = d.syntax.split_once(':') else {
                continue;
            };
            let head = raw_head
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>();
            if matches!(head.as_str(), "on" | "bind" | "use") {
                assert!(
                    DIRECTIVE_PREFIXES.contains(&head.as_str()),
                    "DIRECTIVES advertises `{head}:` (`{}`) but it is not in DIRECTIVE_PREFIXES",
                    d.syntax
                );
            }
            assert!(
                !matches!(
                    head.as_str(),
                    "fx" | "prop"
                        | "attr"
                        | "class"
                        | "style"
                        | "transition"
                        | "in"
                        | "out"
                        | "motion"
                ),
                "DIRECTIVES still advertises dead prefix `{head}:` (`{}`)",
                d.syntax
            );
        }
        const DEAD_PREFIXES: &[&str] = &[
            "fx",
            "prop",
            "attr",
            "class",
            "style",
            "transition",
            "in",
            "out",
            "motion",
        ];
        for p in DEAD_PREFIXES {
            assert!(
                !DIRECTIVE_PREFIXES.contains(p),
                "dead prefix `{p}` must not be in DIRECTIVE_PREFIXES"
            );
        }
        assert!(!BIND_TARGETS.iter().any(|b| b.target == "this"));
        assert!(
            FOREIGN_IDIOMS.iter().any(|r| r.syntax.contains("class=")),
            "class= must be listed in FOREIGN_IDIOMS"
        );
        assert!(
            FOREIGN_IDIOMS.iter().any(|r| r.syntax.contains("style=")),
            "style= must be listed in FOREIGN_IDIOMS"
        );
        assert!(
            !FOREIGN_IDIOMS.iter().any(|r| r.syntax.contains("fx:")),
            "own-API ghosts must not live in FOREIGN_IDIOMS"
        );
        assert!(
            !FOREIGN_IDIOMS
                .iter()
                .any(|r| r.syntax.contains("bind:this")),
            "own-API ghosts must not live in FOREIGN_IDIOMS"
        );
        let bind_row = DIRECTIVES
            .iter()
            .find(|d| d.syntax.contains("bind:"))
            .expect("bind: row");
        assert!(
            !bind_row.syntax.contains("this"),
            "active bind: DIRECTIVES row must not list bind:this: {}",
            bind_row.syntax
        );
        let style_row = DIRECTIVES
            .iter()
            .find(|d| d.syntax.starts_with("style("))
            .expect("style tag DIRECTIVES row");
        assert!(!style_row.syntax.contains("sx="), "{}", style_row.syntax);
    }
}
