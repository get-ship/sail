//! The styling **policy** — an *opinionated floor*, plus a tighter *stripped*
//! mode. This is NOT a spectrum from "escape hatch" to "strict"; there is no
//! blessed escape hatch. Both modes are opinionated, and both forbid raw CSS.
//!
//! Universal (every mode, every tier, primitives included): design **tokens**
//! (`sui-css-token` + the value-literal rules) and valid **`sx`** (`sui-sx`).
//! Off-token styling never compiles, anywhere.
//!
//! - [`Policy::Floor`] — the opinionated baseline, and the **default**. One way
//!   to do CSS: `sx` token utilities (raw `<style>` is an error). Raw `<div>`/
//!   `<span>` are fine — primitives compose structure from them. This is the
//!   mode **primitives** live in: not an escape hatch, not "strict", just the
//!   floor. Its opinions (what `sx` covers, which elements auto-wrap, which
//!   motion/variant/responsive language elements exist) are refined by evals.
//! - [`Policy::Stripped`] — the floor with raw HTML stripped away: no raw
//!   `<div>`/`<span>` either, so app authors compose primitives only. Strictly
//!   tighter than the floor; same no-`<style>`, sx still allowed.
//! - [`Policy::Strictest`] — the maximal tier: stripped **plus no `sx`**. "Here
//!   are the primitives — compose them and use their props; no styling escape
//!   at all." Tests whether the primitive vocabulary is rich enough to need
//!   zero styling surface.
//!
//! Migration is per package via `sui.config.json` `{ "policy": "stripped" }`.
//! An un-configured package gets the floor — opinionated by default. There is
//! no "allow raw styles" preset: a package that genuinely cannot yet express a
//! primitive within the floor is scaffolding to be removed, handled by an
//! explicit granular escape, never dignified as a mode.
//!
//! Implementation: a `Policy` *compiles to* a bundle of [`LintOverride`]s that
//! rides the existing `lint_overrides` plumbing through `compile()`. `Floor`
//! promotes `sui-no-style-block` to Error and silences `sui-no-raw-container`
//! (divs are fine); `Stripped` promotes both. The `<style>` gate is a
//! raw-source lint, so `sx`'s own injected block is never mistaken for author
//! CSS.

use std::path::Path;

use super::lint_overrides::{LintOverride, OverrideSeverity};

const CONFIG_FILE: &str = "sui.config.json";

/// A package's styling policy. See the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Policy {
    /// The opinionated baseline, and the default: no raw `<style>` (sx only),
    /// tokens enforced, raw containers allowed. Where primitives live.
    #[default]
    Floor,
    /// The floor with raw HTML stripped: no raw `<div>`/`<span>` either. Where
    /// app authors live — compose primitives, don't hand-roll boxes. sx allowed.
    Stripped,
    /// The maximal tier: stripped **plus no `sx`**. Primitives + their props
    /// only, no styling escape whatsoever.
    Strictest,
}

impl Policy {
    /// Parse the `"policy"` config value. Unknown strings return `None` (caller
    /// falls back to the default rather than failing the build).
    pub fn parse(s: &str) -> Option<Policy> {
        match s.trim() {
            "floor" => Some(Policy::Floor),
            "stripped" => Some(Policy::Stripped),
            "strictest" => Some(Policy::Strictest),
            _ => None,
        }
    }

    /// The `LintOverride` bundle this policy implies. Both modes promote
    /// `sui-no-style-block` to Error (the floor's core opinion — one way to do
    /// CSS). They differ only on raw containers: the floor silences them (divs
    /// are legitimate structure), `stripped` promotes them to Error.
    pub fn overrides(self) -> Vec<LintOverride> {
        use OverrideSeverity::{Error, Off};
        let lo = |rule: &str, severity| LintOverride { rule: rule.to_string(), severity };
        vec![
            // Raw `<style>` and raw inline `style=` are both banned at every
            // policy — the floor's core opinion is "one way to do CSS," and both
            // are a second, untokened way. (The strictest eval caught agents
            // falling through to `style=` when sx and `<style>` were closed.)
            lo("sui-no-style-block", Error),
            lo("sui-no-inline-style", Error),
            // Raw `<div>`/`<span>`: fine at the floor (silence the warn), banned
            // once stripped.
            lo(
                "sui-no-raw-container",
                match self {
                    Policy::Floor => Off,
                    Policy::Stripped | Policy::Strictest => Error,
                },
            ),
            // `sx`: blessed at floor + stripped, banned at strictest.
            lo(
                "sui-no-sx",
                match self {
                    Policy::Floor | Policy::Stripped => Off,
                    Policy::Strictest => Error,
                },
            ),
        ]
    }

    /// Resolve the policy for a source file: walk up from its directory to the
    /// nearest `sui.config.json` carrying a `"policy"` key. Defaults to `Floor`
    /// (opinionated by default) when none is found, or when the nearest config
    /// is unreadable/unparseable (fail-open to the floor, and stop — a present
    /// config is the package's declared intent).
    pub fn resolve(source_file: &Path) -> Policy {
        let mut dir = source_file.parent();
        while let Some(d) = dir {
            let cfg = d.join(CONFIG_FILE);
            match std::fs::read_to_string(&cfg) {
                Ok(text) => return parse_config(&text).unwrap_or_default(),
                Err(_) => dir = d.parent(),
            }
        }
        Policy::default()
    }
}

fn parse_config(text: &str) -> Option<Policy> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    Policy::parse(v.get("policy")?.as_str()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_the_floor() {
        assert_eq!(Policy::default(), Policy::Floor);
    }

    #[test]
    fn floor_bans_style_but_allows_divs() {
        let ovr = Policy::Floor.overrides();
        let style = ovr.iter().find(|o| o.rule == "sui-no-style-block").unwrap();
        let div = ovr.iter().find(|o| o.rule == "sui-no-raw-container").unwrap();
        assert_eq!(style.severity, OverrideSeverity::Error, "floor forbids <style>");
        assert_eq!(div.severity, OverrideSeverity::Off, "floor allows raw <div>");
    }

    fn sev(p: Policy, rule: &str) -> OverrideSeverity {
        p.overrides().iter().find(|o| o.rule == rule).unwrap().severity
    }

    #[test]
    fn floor_allows_divs_and_sx() {
        assert_eq!(sev(Policy::Floor, "sui-no-raw-container"), OverrideSeverity::Off);
        assert_eq!(sev(Policy::Floor, "sui-no-sx"), OverrideSeverity::Off);
    }

    #[test]
    fn stripped_bans_divs_keeps_sx() {
        assert_eq!(sev(Policy::Stripped, "sui-no-raw-container"), OverrideSeverity::Error);
        assert_eq!(sev(Policy::Stripped, "sui-no-sx"), OverrideSeverity::Off);
    }

    #[test]
    fn strictest_bans_everything() {
        let ovr = Policy::Strictest.overrides();
        assert_eq!(ovr.len(), 4); // style-block, inline-style, raw-container, sx
        assert!(ovr.iter().all(|o| o.severity == OverrideSeverity::Error));
    }

    #[test]
    fn inline_style_banned_at_every_policy() {
        for p in [Policy::Floor, Policy::Stripped, Policy::Strictest] {
            assert_eq!(sev(p, "sui-no-inline-style"), OverrideSeverity::Error);
        }
    }

    #[test]
    fn parses_known_modes_only() {
        assert_eq!(Policy::parse("floor"), Some(Policy::Floor));
        assert_eq!(Policy::parse("  stripped "), Some(Policy::Stripped));
        assert_eq!(Policy::parse("strictest"), Some(Policy::Strictest));
        assert_eq!(Policy::parse("authoring"), None);
    }

    #[test]
    fn parse_config_reads_policy_key() {
        assert_eq!(parse_config(r#"{"policy":"stripped"}"#), Some(Policy::Stripped));
        assert_eq!(parse_config(r#"{"policy":"floor"}"#), Some(Policy::Floor));
        assert_eq!(parse_config(r#"{"other":1}"#), None);
        assert_eq!(parse_config("not json"), None);
    }
}
