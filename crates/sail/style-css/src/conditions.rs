//! Style media / environment conditions — CSS realization of `when({ key })`.
//!
//! Author names live on `sail_style::intent::CONDITION_KEYS`. This table maps
//! those names to `@media` queries. Widths must match design-token
//! `breakpoint-*` lengths (enforced by adapter tests).

/// Closed condition keys → `@media` query for Style `when` (and residual
/// string-sx lowerers until that product is deleted).
pub const CONDITION_PREFIXES: &[(&str, &str)] = &[
    // Responsive, mobile-first (min-width). Widths ARE the breakpoint tokens.
    ("mobile", "(min-width: 390px)"),
    ("tablet", "(min-width: 768px)"),
    ("desktop", "(min-width: 1280px)"),
    ("motion-reduce", "(prefers-reduced-motion: reduce)"),
    ("motion-safe", "(prefers-reduced-motion: no-preference)"),
    ("forced-colors", "(forced-colors: active)"),
    ("dark", "(prefers-color-scheme: dark)"),
    ("light", "(prefers-color-scheme: light)"),
];

pub fn condition_media(name: &str) -> Option<&'static str> {
    CONDITION_PREFIXES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, m)| *m)
}
