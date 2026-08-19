//! The two lints that keep the tier system honest.
//!
//! A channel is only a seam if BOTH sides hold:
//!   - a theme may not reach below its tier (lint 1), and
//!   - a skin may not write the property a channel governs (lint 2).
//!
//! Without lint 1, a theme knob wired to `--space-5` moves every unrelated
//! thing that reads `--space-5` — the studio "card padding" failure that put
//! the tier boundary in the design in the first place. Without lint 2, a skin
//! that writes `border: 1px` locally makes `--border-width` a lie: the theme
//! sets it, the skin ignores it, and the bug looks like "the theme is broken".

use std::collections::BTreeMap;

const THEMES_CSS: &str = include_str!("../../../../packages/sail/tokens/src/themes.css");
const CONTRACTS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/sail-base.css");
const SKINS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/element-skins.css");

/// TIER 0 — primitives that are never themed. Spacing, icon sizes, z-index
/// are structural: they are read by dozens of unrelated surfaces, so
/// "retuning" one from a theme block is never a local change.
/// Overlay clamps are theme-owned (not Tier 0).
const TIER_0_PREFIXES: &[&str] = &["space-", "icon-size-", "z-"];

#[test]
fn no_tier_0_primitive_is_declared_inside_a_theme_block() {
    // themes.css is entirely theme blocks — every `--x:` in it is a theme
    // override — so any Tier-0 declaration anywhere in the file is a breach.
    let mut breaches: Vec<String> = Vec::new();
    for (n, line) in THEMES_CSS.lines().enumerate() {
        let t = line.trim();
        let Some(name) = t.strip_prefix("--") else {
            continue;
        };
        let Some(name) = name.split(':').next() else {
            continue;
        };
        if TIER_0_PREFIXES.iter().any(|p| name.starts_with(p)) {
            breaches.push(format!("themes.css:{}: --{}", n + 1, name));
        }
    }

    assert!(
        breaches.is_empty(),
        "a theme block declares a Tier-0 primitive:\n  {}\n\n\
         Tier-0 tokens (spacing, icon sizes, z-index) are read \
         by unrelated surfaces across the whole system, so overriding one from \
         a theme moves things the theme never meant to touch. If the theme \
         needs a different value for ONE surface, promote a named token for \
         that surface and override it instead.",
        breaches.join("\n  ")
    );
}

/// TIER 2 — properties a channel governs. A skin writing these literally
/// defeats the channel silently: the theme sets the token, the skin overrides
/// it, and nothing reports the conflict.
///
/// `(property, channel, why)`.
const GOVERNED: &[(&str, &str, &str)] = &[
    (
        "text-transform",
        "--heading-transform / --label-transform",
        "casing",
    ),
];

/// Declared exemptions, each with the reason it is NOT a channel breach.
/// Keep this list short and argued — an entry here is a claim that the value
/// is data, not design.
const GOVERNED_EXEMPTIONS: &[(&str, &str)] = &[(
    "sail-avatar",
    "avatar initials are upper-cased as a DATA transform (\"ab\" -> \"AB\"), \
         not as label styling — a theme that lower-cases its labels must not \
         turn initials into lowercase glyphs",
)];

#[test]
fn no_skin_hardcodes_a_channel_governed_property() {
    let mut breaches: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (label, css) in [
        ("sail-base.css", CONTRACTS_CSS),
        ("element-skins.css", SKINS_CSS),
    ] {
        // Track the nearest preceding selector so an exemption can be scoped
        // to the component it belongs to.
        let mut selector = String::new();
        for (n, line) in css.lines().enumerate() {
            let t = line.trim();
            if let Some(b) = t.find('{') {
                if b > 0 {
                    selector = t[..b].trim().to_string();
                }
            }
            for (prop, channel, _) in GOVERNED {
                let needle = format!("{prop}:");
                let Some(at) = t.find(&needle) else { continue };
                let value = t[at + needle.len()..].trim();
                let value = value.split(';').next().unwrap_or(value).trim();
                // Reading the channel (or explicitly inheriting) is the
                // correct form; a literal keyword is the breach.
                // `none` on text-transform is the type({ case: normal }) reset,
                // not a heading/label casing choice.
                if value.starts_with("var(--")
                    || value == "inherit"
                    || value == "initial"
                    || (*prop == "text-transform" && value == "none")
                {
                    continue;
                }
                if GOVERNED_EXEMPTIONS
                    .iter()
                    .any(|(scope, _)| selector.contains(scope))
                {
                    continue;
                }
                breaches
                    .entry((*channel).to_string())
                    .or_default()
                    .push(format!("{label}:{}: `{}` in `{}`", n + 1, t, selector));
            }
        }
    }

    let report: Vec<String> = breaches
        .iter()
        .map(|(channel, sites)| format!("{channel}:\n    {}", sites.join("\n    ")))
        .collect();

    assert!(
        breaches.is_empty(),
        "a skin hardcodes a property that a Tier-2 channel governs:\n  {}\n\n\
         Read the channel instead. A literal here silently wins over the theme: \
         the theme sets the token, this rule overrides it, and the failure \
         surfaces as \"the theme doesn't work\" with nothing pointing here.",
        report.join("\n  ")
    );
}

/// The literal-border census on the real paint surfaces (contracts + lowered
/// element skins). Ratchet: count may fall, never rise. Prefer `--border-width`.
#[test]
fn literal_border_widths_in_skins_do_not_increase() {
    const BASELINE: usize = 0; // ratchet after host Style dissolve into skins/channels

    fn count_literal_borders(src: &str) -> usize {
        let mut count = 0usize;
        for line in src.lines() {
            let t = line.trim();
            if !t.starts_with("border") {
                continue;
            }
            if t.starts_with("border-radius")
                || t.starts_with("border-collapse")
                || t.starts_with("border-spacing")
            {
                continue;
            }
            let Some(colon) = t.find(':') else { continue };
            let value = t[colon + 1..].trim();
            if value
                .split_whitespace()
                .next()
                .is_some_and(|first| first.trim_end_matches(';').ends_with("px"))
            {
                count += 1;
            }
        }
        count
    }

    let count = count_literal_borders(CONTRACTS_CSS) + count_literal_borders(SKINS_CSS);
    assert!(
        count <= BASELINE,
        "literal border widths rose to {count} (baseline {BASELINE}). \
         Spend --border-width for control hairlines."
    );
    assert!(
        count >= BASELINE || BASELINE - count <= 5,
        "literal border widths dropped to {count}; lower BASELINE to {count}."
    );
}

/// Contract-owned tone channel vars. host Style untoned defaults write these only
/// under `:not([tone])` / `[tone='default']`. Bare `.sail-* { … }` must not —
/// same specificity as branded `[tone=X]` would clobber them after load order.
const CONTRACT_TONE_CHANNELS: &[&str] = &[
    "--tone:",
    "--tone-fg:",
    "--tone-bg:",
    "--tone-border:",
    "--tone-contrast:",
    "--tone-ring:",
    "--tone-glow:",
];

/// Fail-closed channel ownership: a bare `.sail-*` host must not write the
/// framework tone contract vars. Soft-status / untoned defaults key on
/// `:not([tone])` or `[tone='default']` so branded `[tone=X]` is not clobbered
/// at equal specificity. Positive invariant — not a eulogy for deleted host CSS.
#[test]
fn bare_sail_host_must_not_write_contract_tone_channels() {
    let mut breaches = Vec::new();
    for css in [CONTRACTS_CSS, SKINS_CSS] {
        let mut rest = css;
        while let Some(rel) = rest.find(".sail-") {
            let slice = &rest[rel..];
            let Some(open) = slice.find('{') else { break };
            let sel = slice[..open].trim();
            let is_bare = sel
                .strip_prefix(".sail-")
                .is_some_and(|tail| tail.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
            if is_bare {
                let body_end = slice[open + 1..]
                    .find('}')
                    .map(|i| open + 1 + i)
                    .unwrap_or(slice.len());
                let body = &slice[open + 1..body_end];
                for ch in CONTRACT_TONE_CHANNELS {
                    if body.contains(ch) {
                        let name = ch.trim_end_matches(':');
                        breaches.push(format!("{sel} writes {name}"));
                    }
                }
            }
            rest = &slice[1..];
        }
    }
    assert!(
        breaches.is_empty(),
        "bare .sail-* host must not write contract tone channels \
         (use :not([tone]) / [tone=default] / branded attr):\n  {}",
        breaches.join("\n  ")
    );
}
