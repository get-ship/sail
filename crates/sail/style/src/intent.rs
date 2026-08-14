//! Intent — the output port of the style program.
//!
//! Resolved slots + conditions. No CSS property names, selectors, `@media`,
//! or class strings. Adapters (`css/`, later hosts) realize this tree.

use std::collections::BTreeMap;

use crate::ir::{PaintArg, PaintOp, Style};

/// Media / environment condition keys. Adapter maps these to host queries.
pub const CONDITION_KEYS: &[&str] = &[
    "mobile",
    "tablet",
    "desktop",
    "motion-reduce",
    "motion-safe",
    "forced-colors",
    "dark",
    "light",
];

/// Closed self-state `when:` keys (author names, both spellings).
const SELF_WHEN_KEYS: &[&str] = &[
    "hover",
    "focus",
    "focusWithin",
    "focus-within",
    "active",
    "disabled",
    "empty",
    "before",
    "after",
    "first",
    "first-child",
    "last",
    "last-child",
    "selected",
    "expanded",
    "checked",
    "pressed",
    "open",
    "current",
    "busy",
    "sortable",
    "state-live",
    "stateLive",
];

/// Closed group-relative `when:` keys. Group+attr grammar (`group-caught`,
/// `group-dismissible`, `group-inline`, …). Intent records the author name,
/// never a selector.
const GROUP_WHEN_KEYS: &[&str] = &[
    "group-current",
    "groupCurrent",
    "group-required",
    "groupRequired",
    "group-invalid",
    "groupInvalid",
    "group-disabled",
    "groupDisabled",
    "group-hover",
    "groupHover",
    "group-focus-within",
    "groupFocusWithin",
    "group-dismissible",
    "groupDismissible",
    "group-removable",
    "groupRemovable",
    "group-open",
    "groupOpen",
    "group-state-has-details",
    "groupStateHasDetails",
    "group-state-open",
    "groupStateOpen",
    "group-caught",
    "groupCaught",
    "group-inline",
    "groupInline",
];

/// Presence attrs `group-<name>` may name. Closed — free attrs stay closed.
const GROUP_ATTR_NAMES: &[&str] = &[
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
    "current",
    "hover",
    "checked",
    "pressed",
    "selected",
    "expanded",
    "busy",
    "sortable",
];

/// CamelCase spelling used by complete for multi-word condition keys.
pub fn condition_complete_name(key: &str) -> &str {
    match key {
        "motion-reduce" => "motionReduce",
        "motion-safe" => "motionSafe",
        "forced-colors" => "forcedColors",
        other => other,
    }
}

pub fn when_key_kebab(key: &str) -> String {
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

fn state_value_ok(v: &str) -> bool {
    !v.is_empty()
        && v.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// True when `when:` may name this key. Names only — no selectors.
pub fn is_known_when_key(key: &str) -> bool {
    if SELF_WHEN_KEYS.contains(&key) || GROUP_WHEN_KEYS.contains(&key) {
        return true;
    }
    if CONDITION_KEYS.contains(&key) {
        return true;
    }
    let kebab = when_key_kebab(key);
    if CONDITION_KEYS.contains(&kebab.as_str()) {
        return true;
    }
    if kebab.starts_with("state-") {
        return kebab.strip_prefix("state-").is_some_and(state_value_ok);
    }
    if let Some(base) = kebab.strip_prefix("group-") {
        if GROUP_ATTR_NAMES.contains(&base) {
            return true;
        }
        if let Some(v) = base.strip_prefix("state-") {
            return state_value_ok(v);
        }
        return state_value_ok(base);
    }
    false
}

/// Closed `when:` keys for complete (static list; dynamic families stay open).
pub fn closed_when_keys() -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = vec![
        "hover",
        "focus",
        "focusWithin",
        "active",
        "disabled",
        "empty",
        "before",
        "after",
        "first",
        "last",
        "selected",
        "expanded",
        "checked",
        "pressed",
        "open",
        "current",
        "busy",
        "sortable",
        "stateLive",
        "groupCurrent",
        "groupRequired",
        "groupInvalid",
        "groupDisabled",
        "groupHover",
        "groupFocusWithin",
        "groupDismissible",
        "groupRemovable",
        "groupOpen",
        "groupStateHasDetails",
        "groupStateOpen",
        "groupCaught",
        "groupInline",
    ];
    for name in CONDITION_KEYS {
        let camel = condition_complete_name(name);
        if !keys.contains(&camel) {
            keys.push(camel);
        }
    }
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// Resolved style program. Adapter input. Zero CSS types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Intent {
    pub nodes: Vec<IntentNode>,
}

/// One node in the Intent tree. Values are tokens / binds / slots — not longhands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentNode {
    Set {
        setter: String,
        args: Vec<IntentValue>,
    },
    On {
        key: String,
        body: Intent,
    },
    Match {
        scrutinee: String,
        arms: Vec<(String, Intent)>,
    },
    Compound {
        when: BTreeMap<String, String>,
        body: Intent,
    },
    /// Motion attach derived from `animate: { <trigger>: play(…) }`.
    Animate {
        trigger: String,
        config: String,
    },
    Ref(String),
}

/// Resolved argument. Token identities, not `var(--…)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentValue {
    Token(String),
    Bind(String),
    MemberPath { root: String, path: Vec<String> },
    Slots(BTreeMap<String, IntentValue>),
}

impl IntentValue {
    fn from_arg(arg: &PaintArg) -> Self {
        match arg {
            PaintArg::Token(t) => IntentValue::Token(t.clone()),
            PaintArg::Bind(n) => IntentValue::Bind(n.clone()),
            PaintArg::MemberPath { root, path } => IntentValue::MemberPath {
                root: root.clone(),
                path: path.clone(),
            },
            PaintArg::Slots(slots) => IntentValue::Slots(
                slots
                    .iter()
                    .map(|(k, v)| (k.clone(), IntentValue::from_arg(v)))
                    .collect(),
            ),
        }
    }
}

fn set_from_op(op: &PaintOp) -> IntentNode {
    IntentNode::Set {
        setter: op.setter.clone(),
        args: op.args.iter().map(IntentValue::from_arg).collect(),
    }
}

/// Lower the author `Style` tree to Intent. No CSS realization.
pub fn lower_to_intent(style: &Style) -> Intent {
    let mut nodes = Vec::new();
    push_style(style, &mut nodes);
    Intent { nodes }
}

fn push_style(style: &Style, nodes: &mut Vec<IntentNode>) {
    match style {
        Style::Empty => {}
        Style::Set(op) => nodes.push(set_from_op(op)),
        Style::Chain(items) | Style::List(items) => {
            for item in items {
                push_style(item, nodes);
            }
        }
        Style::When { entries } => {
            for (key, body) in entries {
                nodes.push(IntentNode::On {
                    key: key.clone(),
                    body: lower_to_intent(body),
                });
            }
        }
        Style::Match { scrutinee, arms } => {
            nodes.push(IntentNode::Match {
                scrutinee: scrutinee.clone(),
                arms: arms
                    .iter()
                    .map(|(k, v)| (k.clone(), lower_to_intent(v)))
                    .collect(),
            });
        }
        Style::Compound { when, body } => {
            nodes.push(IntentNode::Compound {
                when: when.clone(),
                body: lower_to_intent(body),
            });
        }
        Style::MotionAttach { trigger, config } => {
            nodes.push(IntentNode::Animate {
                trigger: trigger.clone(),
                config: config.clone(),
            });
        }
        Style::Ref(name) => nodes.push(IntentNode::Ref(name.clone())),
    }
}

/// Debug dump used as the port proof. Must stay free of CSS strings.
pub fn intent_dump(style: &Style) -> String {
    format!("{:?}", lower_to_intent(style))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn known_when_keys_are_names() {
        assert!(is_known_when_key("hover"));
        assert!(is_known_when_key("tablet"));
        assert!(is_known_when_key("motionReduce"));
        assert!(is_known_when_key("group-hover"));
        assert!(is_known_when_key("state-weekend"));
        assert!(!is_known_when_key("not-a-when-key"));
    }

    #[test]
    fn intent_dump_has_no_css_strings() {
        let (style, diags) = parse("style({ pad(4).bg(surface), when: { hover: bg(muted) } })");
        assert!(diags.is_empty(), "{diags:?}");
        let dump = intent_dump(&style);
        // Concatenate so this file never contains CSS property spellings.
        let forbidden = [
            format!("{}-{}", "background", "color"),
            format!(":{}", "hover"),
            format!("@{}", "media"),
            format!(".{}-", "s"),
            format!("{}-{}", "padding", "left"),
            format!("{}(--", "var"),
        ];
        for needle in &forbidden {
            assert!(
                !dump.contains(needle),
                "Intent must not contain {needle:?}:\n{dump}"
            );
        }
        assert!(dump.contains("pad"), "{dump}");
        assert!(dump.contains("bg"), "{dump}");
        assert!(dump.contains("hover"), "{dump}");
        assert!(dump.contains("surface"), "{dump}");
        assert!(dump.contains("muted"), "{dump}");
    }
}
