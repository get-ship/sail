//! Style IR — compile-time value language (setters chain, guards wrap).

use std::collections::{BTreeMap, HashMap};

/// Argument to a paint setter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaintArg {
    /// SSOT token / keyword (`primary`, `md`, `semibold`).
    Token(String),
    /// Reactive CSS var binding: `bind(rowY)` → `var(--row-y)`.
    Bind(String),
    /// Token-group member path: `tone`, `tone.foreground`, `primary.foreground`.
    ///
    /// Static roots resolve to a public token id before CSS lower. Bound roots
    /// (`tone`) expand over the group family at emit with attr-gated classes.
    MemberPath { root: String, path: Vec<String> },
    /// Concern slot object: `border({ color: muted, width: hairline })`.
    Slots(BTreeMap<String, PaintArg>),
}

impl PaintArg {
    /// Author-facing display (`tone.foreground`, `primary`, `bind(x)`).
    pub fn display(&self) -> String {
        match self {
            PaintArg::Token(t) => t.clone(),
            PaintArg::Bind(n) => format!("bind({n})"),
            PaintArg::MemberPath { root, path } => {
                if path.is_empty() {
                    root.clone()
                } else {
                    format!("{root}.{}", path.join("."))
                }
            }
            PaintArg::Slots(slots) => {
                let inner = slots
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.display()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {inner} }}")
            }
        }
    }
}

/// One chained paint setter (never a guard).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintOp {
    /// Setter name (`bg`, `fg`, `padX`, `radius`, …).
    pub setter: String,
    /// Positional args. Almost always one (`style.bg(primary)`); composite
    /// setters take two (`style.border(1, border)` = width + color in one
    /// call). Empty-arg markers (`style.group()`) carry a single empty
    /// `Token("")`.
    pub args: Vec<PaintArg>,
    pub source: String,
}

impl PaintOp {
    /// First positional arg (`Token("")` when the call was `setter()`).
    pub fn first_arg(&self) -> &PaintArg {
        static EMPTY: PaintArg = PaintArg::Token(String::new());
        self.args.first().unwrap_or(&EMPTY)
    }
}

/// Style value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Style {
    Empty,
    Set(PaintOp),
    Chain(Vec<Style>),
    /// Prop branch — arms exclusive by scrutinee value.
    Match {
        scrutinee: String,
        arms: Vec<(String, Style)>,
    },
    /// Multi-prop intersection.
    Compound {
        when: BTreeMap<String, String>,
        body: Box<Style>,
    },
    /// Browser-applied context map (hover/focus/… → Style).
    When {
        entries: Vec<(String, Style)>,
    },
    /// Lifecycle / ambient motion attach — lowers to `AttrInstr::Motion`.
    ///
    /// Internal node derived from `style({ animate: { enter: play(…), … } })`.
    /// Not an author form: `.enter({…})` tails and `animate: enter({…})` chains
    /// are refused. The `config` is the play-lowered object body (including
    /// braces). Paint emit ignores lifecycle nodes; `transition: play(…)` is
    /// CSS. Preprocess injects `_sail_m_<trigger>` for AttrInstr::Motion.
    MotionAttach {
        /// enter | exit | loop | reveal | scroll | hover | press | focus | …
        trigger: String,
        /// Play-derived `{ opacity: { from, to }, … }` (or `{ preset, duration }`
        /// for CSS `transition`) including braces.
        config: String,
    },
    /// Scope Style const reference.
    Ref(String),
    /// `style={[ a, b, c ]}` composition.
    List(Vec<Style>),
}

/// Named-style environment for resolving `Style::Ref`. Domain type — not CSS.
pub type StyleEnv = HashMap<String, Style>;

impl Style {
    pub fn empty() -> Self {
        Style::Empty
    }

    pub fn chain(items: Vec<Style>) -> Self {
        let mut flat = Vec::new();
        for i in items {
            match i {
                Style::Empty => {}
                Style::Chain(mut inner) => flat.append(&mut inner),
                Style::List(inner) => {
                    for x in inner {
                        match x {
                            Style::Empty => {}
                            Style::Chain(mut c) => flat.append(&mut c),
                            other => flat.push(other),
                        }
                    }
                }
                other => flat.push(other),
            }
        }
        match flat.len() {
            0 => Style::Empty,
            1 => flat.swap_remove(0),
            _ => Style::Chain(flat),
        }
    }
}

/// Diagnostic from parse / resolve / clash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleDiagnostic {
    ParseError {
        message: String,
        source: String,
    },
    UnknownSetter {
        setter: String,
        source: String,
        /// Unique closest valid setter when confluent (typo correction).
        suggestion: Option<String>,
    },
    BadToken {
        setter: String,
        value: String,
        source: String,
    },
    /// L1 component token named in app Style — theme-owned (THEMES-1-ENFORCE).
    ThemeOwnedToken {
        name: String,
        source: String,
    },
    /// Setter called with more positional args than it takes. Border composites
    /// plus two-arg paint composites (`colorDeepen` / `colorMix`) accept two.
    BadArity {
        setter: String,
        got: usize,
        source: String,
    },
    UnknownWhenKey {
        key: String,
        source: String,
    },
    UnknownRef {
        name: String,
        source: String,
    },
    ResidualProduct {
        message: String,
        source: String,
    },
    Clash {
        slot: String,
        first: String,
        second: String,
    },
    BadScrutinee {
        message: String,
        source: String,
    },
    /// Slot name is not on this concern (`border({ radius: md })`).
    UnknownConcernSlot {
        setter: String,
        slot: String,
        source: String,
    },
    /// Same slot written twice inside one concern object.
    DuplicateSlot {
        setter: String,
        slot: String,
        source: String,
    },
    /// Covered property setter / color-only `border(x)` — concern form only.
    RetiredSetter {
        setter: String,
        replacement: String,
        source: String,
    },
    /// Name is a concern / axis / plain setter — not a style reference.
    NotAStyleRef {
        name: String,
        source: String,
    },
    /// File-local style under `styling: themed`. One-offs belong in the theme.
    ThemedLocalStyle {
        name: String,
        source: String,
    },
    /// Unknown after vocabulary + theme named styles + (open) locals.
    UnknownVocabulary {
        name: String,
        source: String,
    },
}

impl StyleDiagnostic {
    pub fn message(&self) -> String {
        match self {
            StyleDiagnostic::ParseError { message, .. } => message.clone(),
            StyleDiagnostic::UnknownSetter {
                setter, suggestion, ..
            } => {
                if let Some(teach) = crate::foreign_idioms::teaching_line(setter) {
                    format!("style unknown setter `{setter}` — {teach}")
                } else {
                    match suggestion {
                        Some(s) => format!(
                            "style unknown setter `{setter}` — did you mean `{s}`? [sail:fix:setter={s}]"
                        ),
                        None => format!("style unknown setter `{setter}`"),
                    }
                }
            }
            StyleDiagnostic::BadToken { setter, value, .. } => {
                format!("style `{setter}({value})` — value is not a token on this setter's scale")
            }
            StyleDiagnostic::ThemeOwnedToken { name, .. } => {
                format!(
                    "component token `{name}` is theme-owned; override via theme= or the theme document"
                )
            }
            StyleDiagnostic::BadArity { setter, got, .. } => {
                format!("style `{setter}` does not accept {got} positional arguments")
            }
            StyleDiagnostic::UnknownWhenKey { key, .. } => {
                format!("style when key `{key}` is not in the closed browser-state table")
            }
            StyleDiagnostic::UnknownRef { name, .. } => {
                format!("style const `{name}` is not defined in scope")
            }
            StyleDiagnostic::ResidualProduct { message, .. } => message.clone(),
            StyleDiagnostic::Clash {
                slot,
                first,
                second,
            } => {
                format!(
                    "style paint clash on `{slot}`: `{first}` and `{second}` (fail closed; no silent merge)"
                )
            }
            StyleDiagnostic::BadScrutinee { message, .. } => message.clone(),
            StyleDiagnostic::UnknownConcernSlot { setter, slot, .. } => match setter.as_str() {
                "border" if slot == "radius" => {
                    "style `border` does not take `radius` — corner geometry uses `radius(…)`"
                        .into()
                }
                "size" if slot.starts_with("pad") || slot == "text" => {
                    format!(
                        "style `size` does not take `{slot}` — padding uses `pad(…)`; type size uses `type(…)`"
                    )
                }
                _ => format!(
                    "style `{setter}({{ {slot}: … }})` — `{slot}` is not a slot on this concern"
                ),
            },
            StyleDiagnostic::DuplicateSlot { setter, slot, .. } => format!(
                "style `{setter}` assigns `{slot}` twice in one slot object (fail closed; no last-wins)"
            ),
            StyleDiagnostic::RetiredSetter {
                setter,
                replacement,
                ..
            } => format!("style `{setter}` is retired — use `{replacement}`"),
            StyleDiagnostic::NotAStyleRef { name, .. } => {
                format!("`{name}` is vocabulary, not a style ref")
            }
            StyleDiagnostic::ThemedLocalStyle { name, .. } => {
                format!(
                    "local style `{name}` is not legal in themed mode — one-offs belong in the theme; create `{name}` in studio"
                )
            }
            StyleDiagnostic::UnknownVocabulary { name, .. } => {
                match crate::foreign_idioms::teaching_line(name) {
                    Some(teach) => format!("unknown vocabulary `{name}` — {teach}"),
                    None => format!("unknown vocabulary `{name}`"),
                }
            }
        }
    }

    pub fn source(&self) -> &str {
        match self {
            StyleDiagnostic::ParseError { source, .. }
            | StyleDiagnostic::UnknownSetter { source, .. }
            | StyleDiagnostic::BadToken { source, .. }
            | StyleDiagnostic::ThemeOwnedToken { source, .. }
            | StyleDiagnostic::BadArity { source, .. }
            | StyleDiagnostic::UnknownWhenKey { source, .. }
            | StyleDiagnostic::UnknownRef { source, .. }
            | StyleDiagnostic::ResidualProduct { source, .. }
            | StyleDiagnostic::BadScrutinee { source, .. }
            | StyleDiagnostic::UnknownConcernSlot { source, .. }
            | StyleDiagnostic::DuplicateSlot { source, .. }
            | StyleDiagnostic::RetiredSetter { source, .. }
            | StyleDiagnostic::NotAStyleRef { source, .. }
            | StyleDiagnostic::ThemedLocalStyle { source, .. }
            | StyleDiagnostic::UnknownVocabulary { source, .. } => source,
            StyleDiagnostic::Clash { first, .. } => first,
        }
    }
}
