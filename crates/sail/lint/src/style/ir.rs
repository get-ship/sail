//! Sail Style IR — compile-time style algebra.
//!
//! Setters CHAIN (`style.bg(primary).fg(onPrimary)`), guards WRAP
//! (`match`, `when`, `compound`), reactive paint via `bind(name)`.
//! Everything lowers to CSS; no JS runtime.

use crate::sx_ir::SxValue;

/// One typed setter call: `style.bg(primary)`.
///
/// `name` is the setter name from the re-homed vocabulary table
/// (`bg`, `fg`, `padX`, `radius`, …). `arg` is the token argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintOp {
    pub name: String,
    pub arg: SxValue,
    pub source: String,
}

/// Style expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Style {
    Empty,
    /// One unconditional setter.
    Set(PaintOp),
    /// Chain composition: `style.a(x).b(y).c(z)`.
    Chain(Vec<Style>),
    /// Prop branch: `match(variant, { filled: …, tonal: … })`.
    Match {
        scrutinee: String,
        arms: Vec<(String, Style)>,
    },
    /// Multi-prop intersection: `compound({ variant: filled, size: lg }, style)`.
    Compound {
        conditions: Vec<(String, String)>,
        body: Box<Style>,
    },
    /// Browser context map: `when({ hover: …, focus: … })`.
    When { map: Vec<(String, Style)> },
    /// Reactive custom-property paint: `bind(rowY)`.
    Bind { name: String },
    /// Named Style const reference from `<scope>`: `filled`.
    Ref { name: String },
}

impl Style {
    pub fn empty() -> Self {
        Style::Empty
    }

    pub fn set(op: PaintOp) -> Self {
        Style::Set(op)
    }

    pub fn chain(items: Vec<Style>) -> Self {
        let mut flat: Vec<Style> = Vec::with_capacity(items.len());
        for item in items {
            match item {
                Style::Empty => {}
                Style::Chain(mut inner) => flat.append(&mut inner),
                other => flat.push(other),
            }
        }
        if flat.is_empty() {
            Style::Empty
        } else if flat.len() == 1 {
            flat.into_iter().next().unwrap()
        } else {
            Style::Chain(flat)
        }
    }

    pub fn match_(scrutinee: impl Into<String>, arms: Vec<(String, Style)>) -> Self {
        Style::Match {
            scrutinee: scrutinee.into(),
            arms,
        }
    }

    pub fn compound(conditions: Vec<(String, String)>, body: Style) -> Self {
        Style::Compound {
            conditions,
            body: Box::new(body),
        }
    }

    pub fn when(map: Vec<(String, Style)>) -> Self {
        Style::When { map }
    }

    pub fn bind(name: impl Into<String>) -> Self {
        Style::Bind { name: name.into() }
    }

    pub fn ref_(name: impl Into<String>) -> Self {
        Style::Ref { name: name.into() }
    }
}

/// Diagnostic from parse, resolve, or clash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleDiagnostic {
    UnknownSetter { name: String, source: String },
    IllegalToken { setter: String, arg: String, source: String },
    IllegalScrutinee { scrutinee: String, source: String },
    IllegalWhenKey { key: String, source: String },
    IllegalBind { name: String, source: String },
    UnknownRef { name: String, source: String },
    GuardAsSetter { guard: String, source: String },
    SetterAsGuard { source: String },
    Clash {
        prop: String,
        first: String,
        second: String,
    },
    ParseError { message: String, source: String },
}

impl StyleDiagnostic {
    pub fn message(&self) -> String {
        match self {
            StyleDiagnostic::UnknownSetter { name, .. } => {
                format!("style unknown setter `{name}` — not in the re-homed vocabulary table")
            }
            StyleDiagnostic::IllegalToken { setter, arg, .. } => {
                format!("style `{setter}({arg})` — token is not on the SSOT scale for this setter")
            }
            StyleDiagnostic::IllegalScrutinee { scrutinee, .. } => {
                format!("style match scrutinee `{scrutinee}` must be a declared prop or static path")
            }
            StyleDiagnostic::IllegalWhenKey { key, .. } => {
                format!("style when key `{key}` is not in the closed browser-context table")
            }
            StyleDiagnostic::IllegalBind { name, .. } => {
                format!("style bind name `{name}` is not a known prop / signal / setup binding")
            }
            StyleDiagnostic::UnknownRef { name, .. } => {
                format!("style unknown const `{name}` — not defined in `<scope>`")
            }
            StyleDiagnostic::GuardAsSetter { guard, .. } => {
                format!("style guard `{guard}` cannot appear as a setter chain link — guards wrap Styles")
            }
            StyleDiagnostic::SetterAsGuard { source } => {
                format!("style setter `{source}` must be inside a chain, not a guard body by itself")
            }
            StyleDiagnostic::Clash { prop, first, second } => {
                format!("style clash on `{prop}`: `{first}` and `{second}` both write the same CSS property")
            }
            StyleDiagnostic::ParseError { message, .. } => message.clone(),
        }
    }

    pub fn source(&self) -> &str {
        match self {
            StyleDiagnostic::UnknownSetter { source, .. }
            | StyleDiagnostic::IllegalToken { source, .. }
            | StyleDiagnostic::IllegalScrutinee { source, .. }
            | StyleDiagnostic::IllegalWhenKey { source, .. }
            | StyleDiagnostic::IllegalBind { source, .. }
            | StyleDiagnostic::UnknownRef { source, .. }
            | StyleDiagnostic::GuardAsSetter { source, .. }
            | StyleDiagnostic::SetterAsGuard { source, .. }
            | StyleDiagnostic::ParseError { source, .. } => source,
            StyleDiagnostic::Clash { first, .. } => first,
        }
    }
}
