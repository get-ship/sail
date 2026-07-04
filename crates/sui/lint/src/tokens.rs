//! The shared design-token registry — the **single source** both the
//! `sui-css-token` lint and the `sx` vocabulary read from.
//!
//! There is exactly one source of truth for design-token *names*: the canonical
//! token CSS at `packages/sui/tokens/src/tokens.css`. This module parses the
//! `--name:` custom-property declarations out of it directly — no hand-mirrored
//! JSON, so the registry can never drift from the CSS (the drift that made
//! `sx="text:3xl"` wrongly fail). `themes.css` only *overrides* these names, so
//! parsing `tokens.css` alone captures every global token.
//!
//! Token names are stored WITHOUT the `--` prefix (e.g. `"space-4"`,
//! `"text-3xl"`). `sx`'s allowed spacing/text/radius values are *derived* from
//! this registry (filtered by prefix), so adding `--text-3xl` to the token CSS
//! makes `sx="text:3xl"` legal automatically.

use std::collections::HashSet;
use std::sync::OnceLock;

/// The canonical token definitions. This is the ONE source — no mirror file.
const TOKENS_CSS: &str = include_str!("../../../../packages/sui/tokens/src/tokens.css");

/// Contextual custom properties that are legitimate in component `var(--x)`
/// references but are **not** global design tokens, so they carry no value in
/// `tokens.css:root` and cannot be parsed from it. They are set per-context
/// (at runtime, by a parent primitive, or inside a component's own shadow) —
/// a curated lint allowlist, not the token system. Keep this list tight;
/// anything with a global value belongs in `tokens.css`, not here.
///
/// Public so the LSP shares the exact same list (it completes these too) —
/// one definition, no cross-crate drift.
pub const CONVENTION_TOKENS: &[&str] = &[
    // Tone channel — set by the `tone` prop on a parent, read by descendants.
    "tone", "tone-bg", "tone-border", "tone-contrast", "tone-fg", "tone-on-fg", "tone-ring",
    // Floating-primitive positioning channels (dropdown / hovercard / popover / tooltip).
    "sui-dd-anchor", "sui-dd-default",
    "sui-hc-anchor", "sui-hc-default",
    "sui-popover-anchor", "sui-popover-default",
    "sui-tooltip-anchor", "sui-tooltip-default",
    // App-shell layout channels (Ship/helm rail + top bar).
    "rail-item-justify", "rail-label-display",
    "ship-rail-w", "ship-rail-w-collapsed", "ship-top-bar-h",
    // Misc component-local channels.
    "card-shadow-style", "font-sans", "toast-duration", "z-overlay",
];

/// Every valid design-token name (no `--` prefix): the custom-property
/// declarations parsed from `tokens.css`, plus the [`CONVENTION_TOKENS`]
/// allowlist. Parsed once.
pub fn registry() -> &'static HashSet<&'static str> {
    static CACHE: OnceLock<HashSet<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut out: HashSet<&'static str> = CONVENTION_TOKENS.iter().copied().collect();
        // A custom-property *declaration* is `--name: …`. A `var(--name)` usage
        // is `--name)` (no following `:`), so keying on the colon skips usages.
        for line in TOKENS_CSS.lines() {
            let Some(rest) = line.trim_start().strip_prefix("--") else { continue };
            let Some(colon) = rest.find(':') else { continue };
            let name = rest[..colon].trim_end();
            if !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
                out.insert(&*Box::leak(name.to_owned().into_boxed_str()));
            }
        }
        out
    })
}

/// The suffix values of a scale family, e.g. `scale_suffixes("space-")` →
/// `["1",…,"8"]`, `scale_suffixes("text-")` → `["2xl","3xl","4xl","base",…]`.
/// Sorted for deterministic diagnostics. Derived from [`registry`], so the
/// scale can never drift from the tokens.
fn scale_suffixes(prefix: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = registry()
        .iter()
        .filter_map(|&t| t.strip_prefix(prefix).filter(|s| !s.is_empty()))
        .collect();
    out.sort_unstable();
    out
}

/// Allowed `sx` spacing values: the `--space-N` scale plus the literal `0`
/// (no `--space-0` token exists; `p:0` must mean zero padding, not a var).
pub fn space_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v = vec!["0"];
        v.extend(scale_suffixes("space-"));
        v
    })
}

/// Allowed `sx` text values: the full `--text-*` scale (xs … 4xl, base).
pub fn text_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| scale_suffixes("text-"))
}

/// Allowed `sx` radius values: the `--radius-*` scale plus the literal `none`
/// (→ `0`). No phantom `xs` — the scale is whatever the tokens declare.
pub fn radius_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v = vec!["none"];
        v.extend(scale_suffixes("radius-"));
        v
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_scale_includes_the_full_registry_range() {
        let t = text_values();
        // the exact values that sx used to be missing — proof of no-drift.
        for want in ["xs", "sm", "md", "base", "lg", "xl", "2xl", "3xl", "4xl"] {
            assert!(t.contains(&want), "text scale missing `{want}`: {t:?}");
        }
    }

    #[test]
    fn space_has_zero_plus_scale_no_phantom() {
        let s = space_values();
        assert!(s.contains(&"0"), "p:0 must be allowed (literal zero)");
        for n in ["1", "4", "8"] {
            assert!(s.contains(&n), "space missing `{n}`");
        }
    }

    #[test]
    fn radius_has_none_plus_scale_without_xs() {
        let r = radius_values();
        assert!(r.contains(&"none"));
        assert!(r.contains(&"full") && r.contains(&"lg"));
        assert!(!r.contains(&"xs"), "radius `xs` was a phantom — not in tokens.css");
    }

    #[test]
    fn parses_global_tokens_and_keeps_conventions() {
        let reg = registry();
        // parsed straight from tokens.css
        assert!(reg.contains("primary"), "brand token from tokens.css");
        assert!(reg.contains("surface-1"), "semantic surface from tokens.css");
        assert!(reg.contains("space-4") && reg.contains("text-2xl"));
        // curated convention allowlist (no global value, must still be accepted)
        assert!(reg.contains("tone-bg") && reg.contains("sui-tooltip-anchor"));
        // a var() *usage* inside a value must never be mistaken for a declaration
        assert!(!reg.contains("foreground)"), "var() usages must not leak in");
    }
}
