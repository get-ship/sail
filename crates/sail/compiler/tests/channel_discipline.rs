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
const CONTRACTS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/contracts.css");
const SKINS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/element-skins.css");

/// TIER 0 — primitives that are never themed. Spacing, icon sizes, z-index and
/// panel geometry are structural: they are read by dozens of unrelated
/// surfaces, so "retuning" one from a theme block is never a local change.
/// A theme that wants a roomier card changes the card's own token, not the
/// spacing step the card happens to spend.
const TIER_0_PREFIXES: &[&str] = &["space-", "icon-size-", "z-", "panel-"];

#[test]
fn no_tier_0_primitive_is_declared_inside_a_theme_block() {
    // themes.css is entirely theme blocks — every `--x:` in it is a theme
    // override — so any Tier-0 declaration anywhere in the file is a breach.
    let mut breaches: Vec<String> = Vec::new();
    for (n, line) in THEMES_CSS.lines().enumerate() {
        let t = line.trim();
        let Some(name) = t.strip_prefix("--") else { continue };
        let Some(name) = name.split(':').next() else { continue };
        if TIER_0_PREFIXES.iter().any(|p| name.starts_with(p)) {
            breaches.push(format!("themes.css:{}: --{}", n + 1, name));
        }
    }

    assert!(
        breaches.is_empty(),
        "a theme block declares a Tier-0 primitive:\n  {}\n\n\
         Tier-0 tokens (spacing, icon sizes, z-index, panel geometry) are read \
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
    ("text-transform", "--heading-transform / --label-transform", "casing"),
    ("border-image", "--corner-image", "corner treatment"),
];

/// Declared exemptions, each with the reason it is NOT a channel breach.
/// Keep this list short and argued — an entry here is a claim that the value
/// is data, not design.
const GOVERNED_EXEMPTIONS: &[(&str, &str)] = &[
    (
        "sail-avatar",
        "avatar initials are upper-cased as a DATA transform (\"ab\" -> \"AB\"), \
         not as label styling — a theme that lower-cases its labels must not \
         turn initials into lowercase glyphs",
    ),
];

#[test]
fn no_skin_hardcodes_a_channel_governed_property() {
    let mut breaches: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (label, css) in [("contracts.css", CONTRACTS_CSS), ("element-skins.css", SKINS_CSS)] {
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
                if value.starts_with("var(--") || value == "inherit" || value == "initial" {
                    continue;
                }
                if GOVERNED_EXEMPTIONS.iter().any(|(scope, _)| selector.contains(scope)) {
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

/// The literal-border census.
///
/// `--border-width` governs stroke weight, but 42 literal `Npx` borders
/// predate it across the stdlib (measured by this test, which is why the
/// number is not the 39 a naive grep reports — the grep misses multi-value
/// shorthands). They are NOT all breaches: a decorative 2px quote rule is a
/// design detail, not a control hairline. So this is a RATCHET, not a ban —
/// the count may fall, never rise. Driving it to zero means auditing each
/// site for "hairline or graphic?", a separate pass from landing the channel.
#[test]
fn literal_border_widths_in_skins_do_not_increase() {
    // Census lives in framework residual skins now — author `.sail` sources no
    // longer carry `<style>` (relocated to `stdlib/_residues.css`).
    const BASELINE: usize = 38;

    let mut count = 0usize;
    let residues = concat!(env!("CARGO_MANIFEST_DIR"), "/stdlib/_residues.css");
    let src = std::fs::read_to_string(residues).expect("stdlib/_residues.css must exist");
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("border") {
            continue;
        }
        // border-radius/-collapse/-spacing are not stroke weight — a
        // radius literal is the radius scale's business, not this channel's.
        if t.starts_with("border-radius")
            || t.starts_with("border-collapse")
            || t.starts_with("border-spacing")
        {
            continue;
        }
        let Some(colon) = t.find(':') else { continue };
        let value = t[colon + 1..].trim();
        // A literal width is a bare `Npx` in the first position of the
        // shorthand, or as the whole value of a *-width property.
        if value
            .split_whitespace()
            .next()
            .is_some_and(|first| first.trim_end_matches(';').ends_with("px"))
        {
            count += 1;
        }
    }

    assert!(
        count <= BASELINE,
        "literal border widths in residual skins rose to {count} (baseline {BASELINE}).\n\
         New skins must spend --border-width so themes can thicken hairlines. \
         If this is a decorative graphic rather than a control hairline, say so \
         in a comment and raise the baseline deliberately."
    );
    // Tightening is welcome — but the baseline must be lowered with it, or the
    // ratchet stops ratcheting.
    assert!(
        count >= BASELINE || BASELINE - count <= 5,
        "literal border widths dropped to {count}; lower BASELINE to {count} so \
         the ratchet keeps its grip."
    );
}

/// Attr-absent L1 defaults that contracts does NOT fill (contracts only keys
/// `[tone='X']`). Residual skins must still supply muted/default channels so
/// PE paint (alert stripe, badge soft base, spinner ring) stays valid.
#[test]
fn residual_default_tone_fills_for_untoned_hosts() {
    let css = include_str!("../stdlib/_residues.css");

    // alert muted default
    assert!(
        css.contains(".sail-alert:not([tone])")
            && css.contains("--tone: var(--muted-foreground)")
            && css.contains("--tone-bg: var(--tone-muted-bg)"),
        "alert untoned default must fill --tone/--tone-bg for stripe + icon chip"
    );

    // badge muted default (host sx reads tone-bg/fg/border)
    assert!(
        css.contains(".sail-badge:not([tone])")
            && css.contains("--tone-border: var(--tone-muted-border)"),
        "badge untoned default must fill muted tone channels for host sx"
    );

    // spinner default stroke
    assert!(
        css.contains(".sail-spinner:not([tone])")
            && css.contains("--tone-fg: var(--muted-foreground)"),
        "spinner untoned default must fill --tone-fg for .ring accent stroke"
    );

    // switch host base: attr-absent :checked uses var(--tone) with no fallback
    {
        let Some(idx) = css.find(".sail-switch {") else {
            panic!("missing .sail-switch host rule");
        };
        let rest = &css[idx..];
        let end = rest.find('}').unwrap_or(rest.len());
        let block = &rest[..end];
        assert!(
            block.contains("--tone: var(--primary)"),
            "untoned switch host must set --tone: var(--primary) for PE checked paint: {block}"
        );
        assert!(
            block.contains("--tone-on-fg: var(--primary-foreground)"),
            "untoned switch host must set default thumb contrast: {block}"
        );
    }

    // switch must NOT re-write global --tone on branded tones
    for tone in ["primary", "info", "success", "warning", "destructive", "accent", "muted"] {
        let needle = format!(".sail-switch[tone='{tone}']");
        let Some(idx) = css.find(&needle) else {
            panic!("missing switch tone rule for {tone}");
        };
        let rest = &css[idx..];
        let end = rest.find('}').unwrap_or(rest.len());
        let block = &rest[..end];
        assert!(
            !block.contains("--tone:"),
            "switch[{tone}] must not rewrite global --tone channel: {block}"
        );
        assert!(
            block.contains("--tone-on-fg:"),
            "switch[{tone}] must keep local thumb contrast --tone-on-fg: {block}"
        );
    }
}
