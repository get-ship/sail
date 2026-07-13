//! The styling **policy** — a non-negotiable *floor* plus a small set of
//! *capability* keys the compiler dials on or off. This is NOT a spectrum from
//! "escape hatch" to "strict": there is no blessed escape hatch anywhere. Every
//! tier forbids raw CSS.
//!
//! ## The floor — not a key, cannot be turned off
//!
//! Authored `<style>` blocks (`sui-no-style-block`), inline `style=`
//! (`sui-no-inline-style`) and `class` (`sui-no-class` — light DOM has no
//! boundary, so a class is a global CSS target: the hook that lets banned CSS
//! live in a file the compiler never sees) are a hard `Error` under **every**
//! policy, and there
//! is no config key to relax them. This is what "floor" means: the thing not
//! even the compiler's configuration allows. (The compiler's own `--styles`
//! injection is internal, not authored source, so `sx`'s generated block is
//! never mistaken for author CSS.)
//!
//! ## The capability keys — the dial
//!
//! Three keys, each on/off, gate the styling surface *above* the floor:
//! - `sx` — the `sx="…"` token utility surface (`sui-no-sx`)
//! - `variants` — the `<variants>` prop→style table (`sui-no-variants`)
//! - `raw_containers` — raw `<div>`/`<span>` (`sui-no-raw-container`)
//!
//! ## The presets
//!
//! - [`Policy::FULL`] — primitive-authoring: every capability on. Where the SUI
//!   primitives are built. The floor still holds; a dissolved primitive expresses
//!   itself in `sx` + `<variants>` + layout keywords, never raw CSS.
//! - [`Policy::STRICT`] — app-authoring: every capability off. Compose primitives,
//!   drive them with props, use the layout keywords — no styling escape at all.
//!   Tests whether the primitive vocabulary is rich enough to need zero surface.
//!
//! A `sui.config.json` may name a preset (`{ "preset": "strict" }`) and/or set
//! individual keys (`{ "preset": "full", "raw_containers": false }`) — keys
//! override the preset; the floor is never a key. An un-configured package
//! resolves to `None` (opt-in): the policy applies only where a package has
//! declared one, so it rolls out per-package and every unconfigured tree — the
//! primitives still mid-dissolution — compiles exactly as before.
//!
//! Implementation: a `Policy` *compiles to* a bundle of [`LintOverride`]s that
//! rides the existing `lint_overrides` plumbing through `compile()`.

use std::path::Path;

use super::lint_overrides::{LintOverride, OverrideSeverity};

const CONFIG_FILE: &str = "sui.config.json";

/// A package's styling policy: the (implicit, non-overridable) floor plus the
/// three capability keys. See the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    /// The `sx="…"` token styling surface.
    pub sx: bool,
    /// The `<variants>` prop→style table.
    pub variants: bool,
    /// Raw `<div>`/`<span>` containers.
    pub raw_containers: bool,
}

impl Policy {
    /// `full` — primitive-authoring: every capability unlocked (the floor still
    /// holds). Where the primitives are built and dissolved.
    pub const FULL: Policy = Policy { sx: true, variants: true, raw_containers: true };

    /// `strict` — app-authoring: every capability off. Compose primitives and use
    /// their props; no `sx`, no `<variants>`, no raw boxes.
    pub const STRICT: Policy = Policy { sx: false, variants: false, raw_containers: false };

    /// Parse a preset name. Unknown names return `None`.
    pub fn preset(name: &str) -> Option<Policy> {
        match name.trim() {
            "full" => Some(Policy::FULL),
            "strict" => Some(Policy::STRICT),
            _ => None,
        }
    }

    /// THE FLOOR. Authored `<style>` and inline `style=` are an error in every
    /// SUI file, with or without a `sui.config.json`. This is not a policy tier
    /// and not opt-in: it is a property of the language. The ONLY exception is
    /// the L1 element sources inside this compiler, which do not take this path —
    /// they compile through `compile_element`, because they ARE the styling layer.
    ///
    /// It used to be emitted only as part of a `Policy`, so a package with no
    /// config got no policy and therefore no floor. An opt-in floor is not a
    /// floor: every unconfigured tree — which was all of them — had a silent
    /// escape hatch, and authors (including this one) walked straight through it.
    ///
    /// When the floor bites something real — syntax highlighting, say — the
    /// answer is a missing L1 element, not a hole in the floor.
    pub fn floor() -> Vec<LintOverride> {
        use OverrideSeverity::Error;
        vec![
            LintOverride { rule: "sui-no-style-block".to_string(), severity: Error },
            LintOverride { rule: "sui-no-inline-style".to_string(), severity: Error },
            LintOverride { rule: "sui-no-class".to_string(), severity: Error },
        ]
    }

    /// What the compiler should enforce for `path`: the floor (always) plus
    /// whatever capability keys the package declared (if any).
    pub fn overrides_for(path: &Path) -> Vec<LintOverride> {
        let mut out = Policy::floor();
        if let Some(p) = Policy::resolve_opt(path) {
            out.extend(p.capabilities());
        }
        out
    }

    /// The capability keys only — the floor is added separately by `overrides_for`.
    pub fn capabilities(self) -> Vec<LintOverride> {
        use OverrideSeverity::{Error, Off};
        let cap = |on: bool| if on { Off } else { Error };
        vec![
            LintOverride { rule: "sui-no-sx".to_string(), severity: cap(self.sx) },
            LintOverride { rule: "sui-no-variants".to_string(), severity: cap(self.variants) },
            LintOverride { rule: "sui-no-raw-container".to_string(), severity: cap(self.raw_containers) },
        ]
    }

    /// The `LintOverride` bundle this policy implies. The **floor** is always
    /// `Error` and is emitted regardless of the keys — no configuration can bring
    /// authored `<style>` / `style=` back. The three capability keys toggle their
    /// ban rules between `Off` (allowed) and `Error` (forbidden).
    pub fn overrides(self) -> Vec<LintOverride> {
        use OverrideSeverity::{Error, Off};
        let lo = |rule: &str, severity| LintOverride { rule: rule.to_string(), severity };
        let cap = |on: bool| if on { Off } else { Error };
        vec![
            // FLOOR — non-overridable, present under every policy.
            lo("sui-no-style-block", Error),
            lo("sui-no-inline-style", Error),
            lo("sui-no-class", Error),
            // CAPABILITY keys — the dial.
            lo("sui-no-sx", cap(self.sx)),
            lo("sui-no-variants", cap(self.variants)),
            lo("sui-no-raw-container", cap(self.raw_containers)),
        ]
    }

    /// Resolve the policy for a source file, falling back to [`Policy::FULL`] when
    /// no config is found. (Direct callers that want the opt-in `None` should use
    /// [`resolve_opt`](Policy::resolve_opt).)
    pub fn resolve(source_file: &Path) -> Policy {
        Self::resolve_opt(source_file).unwrap_or(Policy::FULL)
    }

    /// Like [`resolve`] but returns `None` when no `sui.config.json` is found on
    /// the walk up, instead of a default. This is the **opt-in** entry the deck
    /// bundler and CLI use: the policy applies only to packages that have declared
    /// one, so it rolls out per-package and every unconfigured tree keeps
    /// compiling exactly as before. A present-but-unparseable config fails open to
    /// [`Policy::FULL`] (permissive — a config typo never hard-fails a build) and
    /// stops the walk (its presence is the package's declared intent).
    pub fn resolve_opt(source_file: &Path) -> Option<Policy> {
        let mut dir = source_file.parent();
        while let Some(d) = dir {
            let cfg = d.join(CONFIG_FILE);
            match std::fs::read_to_string(&cfg) {
                Ok(text) => return Some(parse_config(&text).unwrap_or(Policy::FULL)),
                Err(_) => dir = d.parent(),
            }
        }
        None
    }
}

/// Parse a `sui.config.json`. A recognised `"preset"` sets the base; individual
/// `"sx"` / `"variants"` / `"raw_containers"` booleans override it. With neither
/// a preset nor any key, returns `None` (not a policy config). A config that
/// names only keys starts from `STRICT` (the app default) and dials up.
fn parse_config(text: &str) -> Option<Policy> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let has_preset = v.get("preset").is_some();
    let mut p = match v.get("preset").and_then(|x| x.as_str()) {
        Some(name) => Policy::preset(name)?,
        None => Policy::STRICT,
    };
    let bool_key = |k: &str| v.get(k).and_then(|x| x.as_bool());
    let mut touched = has_preset;
    if let Some(b) = bool_key("sx") {
        p.sx = b;
        touched = true;
    }
    if let Some(b) = bool_key("variants") {
        p.variants = b;
        touched = true;
    }
    if let Some(b) = bool_key("raw_containers") {
        p.raw_containers = b;
        touched = true;
    }
    touched.then_some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sev(p: Policy, rule: &str) -> OverrideSeverity {
        p.overrides().iter().find(|o| o.rule == rule).unwrap().severity
    }

    #[test]
    fn floor_is_error_under_every_preset_and_is_not_a_key() {
        for p in [Policy::FULL, Policy::STRICT] {
            assert_eq!(sev(p, "sui-no-style-block"), OverrideSeverity::Error);
            assert_eq!(sev(p, "sui-no-inline-style"), OverrideSeverity::Error);
        }
        // No config key can turn the floor off — only capability keys exist.
        let p = parse_config(r#"{"preset":"full","sx":false}"#).unwrap();
        assert_eq!(sev(p, "sui-no-style-block"), OverrideSeverity::Error);
    }

    #[test]
    fn full_unlocks_every_capability() {
        assert_eq!(sev(Policy::FULL, "sui-no-sx"), OverrideSeverity::Off);
        assert_eq!(sev(Policy::FULL, "sui-no-variants"), OverrideSeverity::Off);
        assert_eq!(sev(Policy::FULL, "sui-no-raw-container"), OverrideSeverity::Off);
    }

    #[test]
    fn strict_forbids_every_capability() {
        assert_eq!(sev(Policy::STRICT, "sui-no-sx"), OverrideSeverity::Error);
        assert_eq!(sev(Policy::STRICT, "sui-no-variants"), OverrideSeverity::Error);
        assert_eq!(sev(Policy::STRICT, "sui-no-raw-container"), OverrideSeverity::Error);
    }

    #[test]
    fn parses_presets() {
        assert_eq!(Policy::preset("full"), Some(Policy::FULL));
        assert_eq!(Policy::preset("  strict "), Some(Policy::STRICT));
        assert_eq!(Policy::preset("stripped"), None);
    }

    #[test]
    fn granular_keys_override_the_preset() {
        // full, but raw containers banned
        let p = parse_config(r#"{"preset":"full","raw_containers":false}"#).unwrap();
        assert!(p.sx && p.variants && !p.raw_containers);
        // strict, but sx re-enabled — the in-between tier
        let p = parse_config(r#"{"preset":"strict","sx":true}"#).unwrap();
        assert!(p.sx && !p.variants && !p.raw_containers);
        // keys only, no preset → base STRICT, dial up
        let p = parse_config(r#"{"sx":true}"#).unwrap();
        assert!(p.sx && !p.variants && !p.raw_containers);
    }

    #[test]
    fn non_policy_config_is_none() {
        assert_eq!(parse_config(r#"{"other":1}"#), None);
        assert_eq!(parse_config("not json"), None);
        assert_eq!(parse_config(r#"{"preset":"bogus"}"#), None);
    }
}
