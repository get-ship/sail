//! Style IR — compile-time value language (setters chain, guards wrap).

use std::collections::BTreeMap;

/// Argument to a paint setter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaintArg {
    /// SSOT token / keyword (`primary`, `md`, `semibold`).
    Token(String),
    /// Reactive CSS var binding: `bind(rowY)` → `var(--row-y)`.
    Bind(String),
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
    /// Scope Style const reference.
    Ref(String),
    /// `sx={[ a, b, c ]}` composition.
    List(Vec<Style>),
}

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
    ParseError { message: String, source: String },
    UnknownSetter { setter: String, source: String },
    BadToken { setter: String, value: String, source: String },
    /// Setter called with more positional args than it takes. Border composites
    /// plus two-arg paint composites (`bgMix` / `bgWash`) accept two.
    BadArity { setter: String, got: usize, source: String },
    UnknownWhenKey { key: String, source: String },
    UnknownRef { name: String, source: String },
    ResidualProduct { message: String, source: String },
    Clash { slot: String, first: String, second: String },
    BadScrutinee { message: String, source: String },
}

impl StyleDiagnostic {
    pub fn message(&self) -> String {
        match self {
            StyleDiagnostic::ParseError { message, .. } => message.clone(),
            StyleDiagnostic::UnknownSetter { setter, .. } => {
                format!("style unknown setter `{setter}`")
            }
            StyleDiagnostic::BadToken { setter, value, .. } => {
                format!("style `{setter}({value})` — value is not a token on this setter's scale")
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
            StyleDiagnostic::Clash { slot, first, second } => {
                format!(
                    "style paint clash on `{slot}`: `{first}` and `{second}` (fail closed; no silent merge)"
                )
            }
            StyleDiagnostic::BadScrutinee { message, .. } => message.clone(),
        }
    }

    pub fn source(&self) -> &str {
        match self {
            StyleDiagnostic::ParseError { source, .. }
            | StyleDiagnostic::UnknownSetter { source, .. }
            | StyleDiagnostic::BadToken { source, .. }
            | StyleDiagnostic::BadArity { source, .. }
            | StyleDiagnostic::UnknownWhenKey { source, .. }
            | StyleDiagnostic::UnknownRef { source, .. }
            | StyleDiagnostic::ResidualProduct { source, .. }
            | StyleDiagnostic::BadScrutinee { source, .. } => source,
            StyleDiagnostic::Clash { first, .. } => first,
        }
    }
}
