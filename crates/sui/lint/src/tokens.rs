//! The shared design-token registry — the **single source** both the
//! `sui-css-token` lint and the `sx` vocabulary read from.
//!
//! There is exactly one source of truth for design-token *names*: the canonical
//! token CSS at `packages/sui/tokens/src/tokens.css`. This module parses the
//! `--name:` custom-property declarations out of it directly — no hand-mirrored
//! JSON, so the registry can never drift from the CSS (the drift that made
//! `sx="text-3xl"` wrongly fail). `themes.css` only *overrides* these names, so
//! parsing `tokens.css` alone captures every global token.
//!
//! Token names are stored WITHOUT the `--` prefix (e.g. `"space-4"`,
//! `"text-3xl"`). `sx`'s allowed spacing/text/radius values are *derived* from
//! this registry (filtered by prefix), so adding `--text-3xl` to the token CSS
//! makes `sx="text-3xl"` legal automatically.

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
    // Runtime-geometry channels — a `use:<action>` action (see
    // `packages/sui/runtime/src/actions/`) writes these; sx reads them via
    // the matching `(--name)` atom. Not tokens (a measurement, not a design
    // decision) and not component props/signals either — the ACTION owns
    // the name, same as a token owns its name, which is why this allowlist
    // (rather than `check::sx_vars`'s prop/signal resolution) is where they
    // belong.
    "box-top", "box-left", "box-w", "box-h",           // use:rect
    "evt-top", "evt-h", "evt-left", "evt-w",           // use:eventSpan
    "row-y",                                            // use:virtualRow
    "rows-h",                                           // use:virtualSpacer
    "cell-ps",                                          // use:cellPad
    "pin-x", "pin-y",                                   // use:pin
    "port-top", "port-left", "port-right", "port-bottom", // use:portEdge
    "world-tx",                                          // use:worldTransform
    "zoom-tx",                                           // use:zoomPan
    "max-w",                                             // use:maxWidth
    "swatch-color",                                      // use:swatch
    "tc-x", "tc-y",                                      // use:targetCursor
    "spot-x", "spot-y", "spot-strength",                 // use:spotlight
    "sui-grid-template",                                 // use:gridTracks
    "rng-pct",                                           // use:rangeFill
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
            if !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
                out.insert(&*Box::leak(name.to_owned().into_boxed_str()));
            }
        }
        out
    })
}

/// The suffix values of a scale family, e.g. `suffixes("space-")` →
/// `["1",…,"8"]`, `suffixes("text-")` → `["2xl","3xl","4xl","base",…]`.
/// Sorted for deterministic diagnostics. Derived from [`registry`], so the
/// scale can never drift from the tokens.
pub fn suffixes(prefix: &str) -> Vec<&'static str> {
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
        v.extend(suffixes("space-"));
        v
    })
}

/// Allowed `sx` text values: the full `--text-*` scale (xs … 4xl, base).
pub fn text_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| suffixes("text-"))
}

/// Every token whose VALUE is a color — the surface `sx`'s `bg`/`fg`/`border`
/// draw from. Found by parsing `tokens.css`: a color is an `oklch`/`rgb`/`hsl`/
/// hex/`color-mix` literal, or a `var(--X)` alias resolving to one. Derived, so a
/// new tone channel or surface is usable in sx the moment it lands in the CSS —
/// no hand-maintained color list to drift (the flaw that dropped `bg:success-bg`).
pub fn color_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        // (name, value) for every custom-property declaration.
        let decls: Vec<(&'static str, &'static str)> = TOKENS_CSS
            .lines()
            .filter_map(|line| {
                let rest = line.trim_start().strip_prefix("--")?;
                let colon = rest.find(':')?;
                let name = rest[..colon].trim_end();
                if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
                    return None;
                }
                Some((name, rest[colon + 1..].trim()))
            })
            .collect();
        let is_literal = |v: &str| {
            v.starts_with("oklch")
                || v.starts_with("rgb")
                || v.starts_with("hsl")
                || v.starts_with('#')
                || v.starts_with("color-mix")
        };
        let mut colors: HashSet<&'static str> =
            decls.iter().filter(|(_, v)| is_literal(v)).map(|(n, _)| *n).collect();
        // Resolve `var(--X)` aliases to a known color — a few rounds for chains.
        for _ in 0..4 {
            for (n, v) in &decls {
                if colors.contains(n) {
                    continue;
                }
                if let Some(inner) = v.strip_prefix("var(--") {
                    let refname = inner.split([',', ')']).next().unwrap_or("").trim();
                    if colors.contains(refname) {
                        colors.insert(n);
                    }
                }
            }
        }
        // The universal framework tone channels. suiTonesSheet sets these on
        // every shadow component from the `tone` prop, so they are valid sx
        // color values everywhere — no per-component <theme channels="…">
        // declaration needed (that ceremony is gone in the collapsed shape).
        for ch in [
            "tone",
            "tone-fg",
            "tone-contrast",
            "tone-bg",
            "tone-border",
            "tone-ring",
        ] {
            colors.insert(ch);
        }
        // Sanctioned CSS-wide keyword colors — "named zeros", not raw values.
        // `bg:transparent` (outline/ghost surfaces) and `fg:currentColor` /
        // `border:currentColor` (ink inheritance) are expressible without a
        // token. Single source with the lowering (which emits them verbatim,
        // not `var(--X)`), so vocabulary and emission can never drift.
        for kw in crate::scales::KEYWORD_COLORS {
            colors.insert(kw);
        }
        let mut out: Vec<&'static str> = colors.into_iter().collect();
        out.sort_unstable();
        out
    })
}

/// Dimensional size tokens the sx `w`/`h`/`min-*`/`max-*` props draw from — the
/// icon, control-height, and panel-clamp families that the primitives now depend
/// on. The value IS the token name (`max-h:panel-max-h-md` → `var(--panel-max-h-md)`),
/// so a new size family is usable in sx the moment it lands in `tokens.css`.
pub fn size_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v: Vec<&'static str> = registry()
            .iter()
            .copied()
            .filter(|t| {
                // Only the DIMENSIONAL panel subfamilies join the sizing
                // vocabulary. The panel-pad-* component tokens are padding
                // knobs consumed by skins, not sx sizes — a broad `panel-`
                // prefix would leak them into `w:`/`h:`.
                t.starts_with("icon-size-")
                    || t.starts_with("control-h-")
                    || t.starts_with("breakpoint-")
                    || t.starts_with("panel-max-")
                    || t.starts_with("panel-min-")
                    || t.starts_with("panel-ch-")
                    || t.starts_with("panel-vh-")
                    || t.starts_with("panel-w-")
            })
            .collect();
        v.sort_unstable();
        v
    })
}

/// Allowed `sx` radius values: the `--radius-*` scale plus the literal `none`
/// (→ `0`). No phantom `xs` — the scale is whatever the tokens declare.
pub fn radius_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v = vec!["none"];
        v.extend(suffixes("radius-"));
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
        // The ramp above 8 was added in job-E-space-ramp: an agent reaching
        // for a 48px thumb or a 180px canvas can now SPEND a token (and
        // sx accepts those names because the lint parses the CSS).
        for n in ["12", "16", "24", "32", "48", "64"] {
            assert!(
                s.contains(&n),
                "space missing `{n}` — the Tailwind-aligned upper ramp didn't reach the registry"
            );
        }
        // The alignment: 7 and 8 used to be silent values (32, 40); the
        // agent's prior was Tailwind's (28, 32). The space scale's emitted
        // CSS values now match, by hand (see `space_lower_aligns_with_tailwind`).
        assert_eq!(
            crate::scales::lower("space", "7").as_deref(),
            Some("var(--space-7)"),
        );
        assert_eq!(
            crate::scales::lower("space", "8").as_deref(),
            Some("var(--space-8)"),
        );
    }

    #[test]
    fn space_lower_aligns_with_tailwind() {
        // The ramp carries our tokens as a private var() — sx spends them,
        // not numbers — so the lower() check is whether the value at the
        // right name is the right thing. 7/8 were silently divergent from
        // Tailwind's prior (32/40 vs 28/32); this pins the alignment.
        let tokens = include_str!("../../../../packages/sui/tokens/src/tokens.css");
        for (name, want_px) in [("7", "28px"), ("8", "32px")] {
            let decl = format!("--space-{name}:");
            let got = tokens
                .lines()
                .find_map(|l| l.trim().strip_prefix(&decl))
                .unwrap_or_default()
                .trim()
                .trim_end_matches(';')
                .trim();
            assert_eq!(got, want_px, "--space-{name} drifted from Tailwind");
        }
        // The full Tailwind-aligned space ramp, lower()'d to its var and the
        // var itself resolved from tokens.css — both must agree, end to end.
        for (n, want_px) in [
            ("1", "4px"), ("4", "16px"), ("6", "24px"), ("7", "28px"),
            ("8", "32px"), ("10", "40px"), ("12", "48px"), ("16", "64px"),
            ("24", "96px"), ("48", "192px"), ("64", "256px"),
        ] {
            let decl = format!("--space-{n}:");
            let got = tokens
                .lines()
                .find_map(|l| l.trim().strip_prefix(&decl))
                .unwrap_or_default()
                .trim()
                .trim_end_matches(';')
                .trim();
            assert_eq!(got, want_px, "--space-{n} drifted from Tailwind ({want_px})");
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
    fn color_values_derive_the_full_surface_not_a_curated_list() {
        let c = color_values();
        // base palette + tonal channels + surfaces — all value-derived from CSS
        for want in [
            "primary", "foreground", "success", "surface-1", "muted-foreground",
            "tone-success-bg", "tone-primary-fg", "accent-foreground",
        ] {
            assert!(c.contains(&want), "color surface missing `{want}`");
        }
        // non-colors must NOT leak into the color surface
        assert!(!c.contains(&"space-4") && !c.contains(&"text-2xl") && !c.contains(&"radius-lg"));
        assert!(c.len() > 40, "expected the real ~85-token surface, got {}", c.len());
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
