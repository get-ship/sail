//! CSS realization of `when({ key })` — selectors, ancestor prefixes, media.
//!
//! Author names are validated in `spinnaker::intent::is_known_when_key`.
//! This module maps those names onto CSS.

use std::borrow::Cow;

use spinnaker::intent::when_key_kebab;

use crate::prefixes as sx_rules;

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

/// Closed `when({ key: Style })` browser/framework-state and media keys.
///
/// Breakpoints and preference media come from [`crate::conditions`] — the
/// CSS adapter table, locked to design-token `breakpoint-*` lengths.
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
        other => {
            let kebab = when_key_kebab(other);
            if let Some(m) = crate::conditions::condition_media(other)
                .or_else(|| crate::conditions::condition_media(&kebab))
            {
                return Some(WhenTarget::Media(Cow::Owned(m)));
            }
            // ARIA / pseudo from the closed STATE_PREFIXES table.
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
            if let Some(base) = kebab.strip_prefix("group-") {
                if let Some(p) = sx_rules::state_pseudo(base) {
                    return Some(WhenTarget::Ancestor(Cow::Owned(format!(".sail-grp{p} "))));
                }
                if let Some(p) = sx_rules::state_data(base) {
                    return Some(WhenTarget::Ancestor(Cow::Owned(format!(".sail-grp{p} "))));
                }
                // Presence attrs win over a free `data-state~=` guess. `searchable`
                // / `mode-only` / `mode` are host booleans, not data-state tokens.
                if let Some(n) = sx_rules::group_attr_presence(base) {
                    return Some(WhenTarget::Ancestor(Cow::Owned(format!(".sail-grp[{n}] "))));
                }
                // Enumerated host prop: group-variant-ring → .sail-grp[variant='ring']
                // Same grammar for chrome / size / orientation / type / placement
                // (navbar rail). Presence-only attrs stay on group_attr_presence.
                for attr in [
                    "variant",
                    "chrome",
                    "size",
                    "orientation",
                    "type",
                    "placement",
                    "padding",
                ] {
                    if let Some(v) = base.strip_prefix(&format!("{attr}-")) {
                        if !v.is_empty()
                            && v.bytes()
                                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                        {
                            return Some(WhenTarget::Ancestor(Cow::Owned(format!(
                                ".sail-grp[{attr}='{v}'] "
                            ))));
                        }
                    }
                }
                // bare data-state value on group host: group-user → data-state~=user
                if let Some(p) = sx_rules::state_data(&format!("state-{base}")) {
                    return Some(WhenTarget::Ancestor(Cow::Owned(format!(".sail-grp{p} "))));
                }
            }
            None
        }
    }
}

/// Helper used by emit for simple pseudo append.
pub fn when_selector(key: &str) -> Option<String> {
    match when_target(key)? {
        WhenTarget::Pseudo(s) => Some(s.into_owned()),
        _ => None,
    }
}
