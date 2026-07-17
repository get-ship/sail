//! `sui-event-name` — `on:<event>` event names must be valid DOM
//! events (or a known custom-event prefix). Catches typos like
//! `on:Click` (capital C) which today bind a listener for a
//! never-fired event.
//!
//! The whitelist is the set of standard browser events most templates
//! use, plus four openings:
//!   - `sui-*` — framework-emitted custom events (e.g. `sui-rail:toggle`)
//!   - `ship-*` — Ship product custom events
//!   - `<tag>:` prefix — namespaced events from custom elements
//!   - a small set of bare SUI framework custom events that carry no
//!     prefix — `success` (and `failure`) emitted by `<form mutation=…>`
//!     after the mutation settles. These are real emitted events, not
//!     DOM events, so they live in their own allowlist rather than the
//!     DOM `KNOWN_EVENTS` table.
//!
//! Suggests the closest whitelisted name via Levenshtein when a typo
//! is detected. False negatives are acceptable (any custom event with
//! a `-`/`:` in the name passes); false positives are not — when in
//! doubt, allow.
//!
//! **Severity: Error.** Every `on:<event>` across the components/views
//! tree resolves to a known DOM event, a prefixed/namespaced custom
//! event, or a SUI framework custom event (verified by the repo-wide
//! scan). Error hard-fails a typo'd handler that would silently never
//! fire.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-event-name";

const KNOWN_EVENTS: &[&str] = &[
    // Mouse / pointer
    "click", "dblclick", "mousedown", "mouseup", "mousemove", "mouseenter",
    "mouseleave", "mouseover", "mouseout", "contextmenu", "wheel",
    "pointerdown", "pointerup", "pointermove", "pointerenter", "pointerleave",
    "pointerover", "pointerout", "pointercancel", "gotpointercapture",
    "lostpointercapture",
    // Touch
    "touchstart", "touchend", "touchmove", "touchcancel",
    // Keyboard
    "keydown", "keyup", "keypress",
    // Focus
    "focus", "blur", "focusin", "focusout",
    // Form
    "input", "change", "submit", "reset", "invalid", "select", "formdata",
    "beforeinput",
    // Drag
    "dragstart", "drag", "dragend", "dragenter", "dragleave", "dragover", "drop",
    // Clipboard
    "copy", "cut", "paste",
    // Composition (IME)
    "compositionstart", "compositionupdate", "compositionend",
    // Media
    "play", "pause", "ended", "timeupdate", "volumechange", "ratechange",
    "loadeddata", "loadedmetadata", "canplay", "canplaythrough", "stalled",
    "suspend", "waiting", "seeking", "seeked", "durationchange", "emptied",
    "abort", "error",
    // Window / load
    "load", "unload", "beforeunload", "resize", "scroll", "scrollend",
    "hashchange", "popstate", "pagehide", "pageshow", "visibilitychange",
    "online", "offline",
    // Animation / transition
    "animationstart", "animationend", "animationiteration", "animationcancel",
    "transitionstart", "transitionend", "transitionrun", "transitioncancel",
    // Toggle / dialog
    "toggle", "close", "cancel",
    // Pointer-lock
    "pointerlockchange", "pointerlockerror",
    // Misc
    "wheel", "message", "messageerror", "storage",
];

/// SUI framework custom events that carry NO prefix. `<form mutation=…>`
/// emits `success` / `failure` on its host once the mutation settles, so
/// `on:success=` / `on:failure=` are legitimate even though they aren't DOM
/// events and lack a `sui-`/`ship-` prefix. Keep this list tight — it's the
/// escape hatch for bare custom events the framework genuinely fires.
const SUI_CUSTOM_EVENTS: &[&str] = &["success", "failure"];

/// First index at or after `from` where `needle` occurs.
fn find(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from >= haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;

    while i + 3 < bytes.len() {
        // A comment is not code. This rule is a raw byte scan and knows nothing of
        // syntax, so it must skip `<!-- … -->` explicitly or it lints the prose
        // inside — including a comment that merely mentions `on:`.
        if bytes[i] == b'<' && bytes[i..].starts_with(b"<!--") {
            i = match find(bytes, i + 4, b"-->") {
                Some(end) => end + 3,
                None => break, // unterminated — the parser will say so, not us
            };
            continue;
        }
        // Find ` on:` (whitespace then `on:`) — the attribute boundary.
        if !(bytes[i].is_ascii_whitespace() && &bytes[i + 1..i + 4] == b"on:") {
            i += 1;
            continue;
        }
        let name_start = i + 4;
        // Read until `=`, whitespace, `>`, or `|` (modifier separator).
        let mut j = name_start;
        while j < bytes.len() {
            let c = bytes[j];
            if c == b'=' || c == b'>' || c == b'|' || c.is_ascii_whitespace() {
                break;
            }
            j += 1;
        }
        if j == name_start {
            i = j;
            continue;
        }
        let name = &source[name_start..j];
        if is_acceptable_event_name(name) {
            i = j;
            continue;
        }
        let suggestion = nearest_event(name);
        let mut message = format!("unknown event `{name}` in `on:{name}=`");
        if let Some(s) = &suggestion {
            message.push_str(&format!("; did you mean `on:{s}=`?"));
        }
        let (line, col) = byte_offset_to_line_col(source, name_start);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message,
            suggest: Some(
                "Event names are lowercase ASCII. Standard DOM events \
                 (click, input, change, …) and framework custom events \
                 (sui-*, ship-*, namespaced `tag:event`) are recognized; \
                 anything else is likely a typo."
                    .to_string(),
            ),
        });
        i = j;
    }

    findings
}

fn is_acceptable_event_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    // Known DOM events.
    if KNOWN_EVENTS.contains(&name) {
        return true;
    }
    // Bare SUI framework custom events (`success`/`failure` from <form mutation>).
    if SUI_CUSTOM_EVENTS.contains(&name) {
        return true;
    }
    // Framework / product custom-event prefixes.
    if name.starts_with("sui-") || name.starts_with("ship-") {
        return true;
    }
    // Namespaced custom-element event (`sui-rail:toggle`, `dialog:open`, …).
    if name.contains(':') {
        return true;
    }
    // Author-defined kebab-case event with a hyphen we don't recognize —
    // accept rather than scream. The `-` is the convention for non-DOM
    // custom events.
    if name.contains('-') && name.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
        return true;
    }
    false
}

fn nearest_event(name: &str) -> Option<String> {
    let mut best: Option<(usize, &str)> = None;
    let name_l = name.to_ascii_lowercase();
    for &known in KNOWN_EVENTS {
        let d = levenshtein(&name_l, known);
        let max_d = (known.len().max(name_l.len()) / 3).max(1);
        if d <= max_d && best.is_none_or(|(b, _)| d < b) {
            best = Some((d, known));
        }
    }
    best.map(|(_, s)| s.to_string())
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
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint(src: &str) -> Vec<Finding> {
        check(Path::new("test.sui"), src)
    }

    #[test]
    fn known_event_passes() {
        let src = r#"<button on:click={fire}>go</button>"#;
        assert_eq!(lint(src).len(), 0);
    }

    #[test]
    fn capital_typo_warns() {
        let src = r#"<button on:Click={fire}>go</button>"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("`Click`"));
        assert!(findings[0].message.contains("on:click"));
    }

    #[test]
    fn unknown_event_warns_with_suggestion() {
        let src = r#"<button on:clik={fire}>go</button>"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("click"));
    }

    #[test]
    fn sui_prefixed_event_passes() {
        let src = r#"<sui-rail on:sui-rail:toggle={fire}></sui-rail>"#;
        assert_eq!(lint(src).len(), 0);
    }

    #[test]
    fn ship_prefixed_event_passes() {
        let src = r#"<ship-thing on:ship-thing:done={fire}></ship-thing>"#;
        assert_eq!(lint(src).len(), 0);
    }

    #[test]
    fn modifier_doesnt_confuse_scan() {
        let src = r#"<button on:click|preventDefault|debounce:200={fire}>go</button>"#;
        assert_eq!(lint(src).len(), 0);
    }

    #[test]
    fn multiple_handlers_each_checked() {
        let src = r#"<div on:click={a} on:CHANGE={b} on:input={c}></div>"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("CHANGE"));
    }

    #[test]
    fn form_mutation_success_event_passes() {
        // `<form mutation=…>` emits `success`/`failure` — bare framework
        // custom events, not DOM events. Both must be accepted.
        let src = r#"<form mutation="ship.x.update" on:success={() => ok()}></form>"#;
        assert_eq!(lint(src).len(), 0, "{:?}", lint(src));
        let f = r#"<form mutation="ship.x.update" on:failure={() => bad()}></form>"#;
        assert_eq!(lint(f).len(), 0, "{:?}", lint(f));
    }

    #[test]
    fn severity_is_error() {
        let src = r#"<button on:Click={fire}>go</button>"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
    }

    /// A comment that mentions `on:` is prose, not an attribute.
    #[test]
    fn a_comment_is_not_code() {
        let src = "<!-- on:<event> takes modifiers: |prevent, |stop -->\n<button on:click={go}>go</button>";
        assert_eq!(lint(src).len(), 0, "prose inside a comment is not an attribute: {:?}", lint(src));
    }

    /// ...but the skip must not swallow the code after the comment.
    #[test]
    fn a_real_typo_after_a_comment_is_still_caught() {
        let src = "<!-- on:whatever -->\n<button on:Click={go}>go</button>";
        let findings = lint(src);
        assert_eq!(findings.len(), 1, "the real typo still fires: {findings:?}");
        assert!(findings[0].message.contains("on:click"));
    }
}
