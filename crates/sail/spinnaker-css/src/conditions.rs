//! Style media / environment conditions — CSS realization of `when({ key })`.
//!
//! Author names live on `spinnaker::intent`. Breakpoint widths come from
//! the active theme table (`spinnaker::tokens::breakpoint_table`). Preference
//! media stays language, not theme.

use spinnaker::tokens;

/// Preference / environment media. Not theme-declared.
pub const PREFERENCE_PREFIXES: &[(&str, &str)] = &[
    ("motion-reduce", "(prefers-reduced-motion: reduce)"),
    ("motion-safe", "(prefers-reduced-motion: no-preference)"),
    ("forced-colors", "(forced-colors: active)"),
    ("dark", "(prefers-color-scheme: dark)"),
    ("light", "(prefers-color-scheme: light)"),
];

/// Theme breakpoints + preference media. Widths follow the active theme.
pub fn condition_prefixes() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = tokens::breakpoint_table()
        .into_iter()
        .map(|(name, width)| (name, format!("(min-width: {width})")))
        .collect();
    for (name, query) in PREFERENCE_PREFIXES {
        out.push(((*name).to_string(), (*query).to_string()));
    }
    out
}

/// Closed table kept for residual string-sx callers. Preference media only —
/// viewport widths are theme-declared via [`condition_prefixes`].
pub const CONDITION_PREFIXES: &[(&str, &str)] = PREFERENCE_PREFIXES;

pub fn condition_media(name: &str) -> Option<String> {
    let kebab = spinnaker::intent::when_key_kebab(name);
    if let Some(width) = tokens::breakpoint_width(name).or_else(|| tokens::breakpoint_width(&kebab))
    {
        return Some(format!("(min-width: {width})"));
    }
    PREFERENCE_PREFIXES
        .iter()
        .find(|(n, _)| *n == name || *n == kebab.as_str())
        .map(|(_, q)| (*q).to_string())
}
