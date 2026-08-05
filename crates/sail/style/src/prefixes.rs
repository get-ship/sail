//! Closed Style / leaf-sx state and structural prefixes.
//!
//! Shared by Style `when` emit and residual leaf lowerers so lint, preprocess,
//! and emit cannot diverge on the prefix set.

pub use crate::conditions::{CONDITION_PREFIXES, condition_media};

/// The sx prefix/value forms that are grammar rather than named tables —
/// the `group` ancestor marker, the `group-<state>:` and `state-<value>:`
/// prefix families, and the `(--name)` custom-property value read. Table
/// form (form → one-line semantics) so surface documentation renders the
/// same shapes `is_prefix` / `state_data` / `Utility::is_var_ref` enforce.
pub const SX_EXTRA_FORMS: &[(&str, &str)] = &[
    (
        "group",
        "bare marker in an ancestor's sx; descendants react to that ancestor with group-<state>:",
    ),
    (
        "group-<state>:",
        "apply when the group-marked ancestor is in <state> — any state prefix, or a state-<value>",
    ),
    (
        "state-<value>:",
        "component-state prefix keyed on the reserved data-state attribute; lowers to [data-state~='<value>'], value is [a-z0-9-]+",
    ),
    (
        "(--name)",
        "read a CSS custom property as any atom's value — lowers to var(--name); typed actions write the property at runtime",
    ),
];

/// A state prefix (`hover:`/`focus:`/…) → its CSS pseudo-class. `focus` maps to
/// `:focus-visible` (the modern default — no ring on a mouse click). Shared by
/// the lint (validation) and preprocess (lowering) so they can't diverge.
/// State prefixes (name → selector) — table form for surface documentation.
/// `current` keys off aria-current (the author stamps it for assistive tech
/// anyway; the style reads the same source of truth).
pub const STATE_PREFIXES: &[(&str, &str)] = &[
    ("hover", ":hover"),
    ("focus", ":focus-visible"),
    ("focus-within", ":focus-within"),
    ("active", ":active"),
    ("disabled", ":is(:disabled,[aria-disabled='true'])"),
    ("current", "[aria-current]:not([aria-current='false'])"),
    // ARIA states — styling keys on the SAME attribute assistive tech reads,
    // so the visual is only reachable by stamping the semantics. A closed
    // set: new states are rulings, not a namespace.
    ("selected", "[aria-selected='true']"),
    ("expanded", "[aria-expanded='true']"),
    ("busy", "[aria-busy='true']"),
    ("checked", "[aria-checked='true']"),
    ("pressed", "[aria-pressed='true']"),
    ("open", "[open]"),
    ("sortable", "[aria-sort]"),
];

/// `state-<value>:` — the component-state prefix. Keys on the reserved
/// `data-state` attribute (`data-state="weekend"`), the single sanctioned
/// place a component declares a visual state that is neither an ARIA state
/// nor a CSS pseudo (out-of-month, weekend, in-range). One attribute, not a
/// namespace: `data-*` at large stays closed.
pub fn state_data(state: &str) -> Option<String> {
    let v = state.strip_prefix("state-")?;
    if v.is_empty()
        || !v
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return None;
    }
    Some(format!("[data-state~='{v}']"))
}

pub fn state_pseudo(state: &str) -> Option<&'static str> {
    STATE_PREFIXES
        .iter()
        .find(|(n, _)| *n == state)
        .map(|(_, p)| *p)
}

/// Boolean presence attrs that `group-<name>:` may key (host prop reflected as
/// a bare attribute). Closed — free attrs stay closed.
pub fn group_attr_presence(name: &str) -> Option<&'static str> {
    const PRESENCE: &[&str] = &[
        "dismissible",
        "removable",
        "inline",
        "sticky",
        "interactive",
        "bordered",
        "fill",
        "centered",
        "required",
        "invalid",
        "disabled",
        "open",
        "pill",
    ];
    PRESENCE.iter().copied().find(|&n| n == name)
}

/// Pseudo-element / structural prefixes (TW-shaped): attach as `::before`,
/// `::after`, or `:empty` on the styled element.
pub fn pseudo_element_prefix(p: &str) -> Option<&'static str> {
    match p {
        "before" => Some("::before"),
        "after" => Some("::after"),
        "empty" => Some(":empty"),
        _ => None,
    }
}

/// Is this a legal prefix? The set is closed and every member keys on semantics
/// an assistive technology also reads — which is what makes the accessible
/// component the cheapest one to build rather than a chore.
pub fn is_prefix(p: &str) -> bool {
    state_pseudo(p).is_some()
        || condition_media(p).is_some()
        || state_data(p).is_some()
        || pseudo_element_prefix(p).is_some()
        || p.strip_prefix("group-").is_some_and(|b| {
            state_pseudo(b).is_some() || state_data(b).is_some() || group_attr_presence(b).is_some()
        })
}
