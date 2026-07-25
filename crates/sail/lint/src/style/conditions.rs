//! Style media / environment conditions — product surface for `when({ key })`.
//!
//! Breakpoints and preference queries are **Style** (`when({ tablet: … })`),
//! not a leftover string-sx feature. Values are literal media queries because
//! `@media` cannot read custom properties; widths must match design-token
//! `breakpoint-*` lengths (enforced by tests).

/// Closed condition keys → `@media` query for Style `when` (and residual
/// lowerers until string product is deleted).
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
