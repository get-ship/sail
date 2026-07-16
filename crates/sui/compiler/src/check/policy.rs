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
//! Implementation: the **floor** is enforced by the compiler itself — an
//! unconditional `enforce_floor` pass (`check::lint_overrides::enforce_floor`)
//! that promotes every [`FLOOR_RULES`] finding to `Error` on every compile,
//! independent of `opts.lint_overrides`. It does NOT ride the override channel,
//! so no host (or missing host wiring) can skip it and no `lint_override` can
//! soften it. Only the **capability keys** compile to a bundle of
//! [`LintOverride`]s that rides the `lint_overrides` plumbing through
//! `compile()` — they are the configurable surface ABOVE the floor.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::lint_overrides::{LintOverride, OverrideSeverity};

/// The floor rule IDs — the styling constructs banned in EVERY `.sui` compiled
/// as a component or view, with or without a `sui.config.json`. This is the
/// single source of truth: [`is_floor_rule`], [`Policy::floor`], and the
/// compiler's unconditional `enforce_floor` pass
/// (`check::lint_overrides::enforce_floor`) all read this one list, so a floor
/// rule added here is enforced everywhere with no second place to update.
pub const FLOOR_RULES: [&str; 4] = [
    "sui-no-style-block",
    "sui-no-inline-style",
    "sui-no-class",
    "sui-no-undeclared-stylesheet",
];

/// True iff `rule` is a floor rule — one the compiler promotes to `Error`
/// unconditionally and that no `lint_override` can soften. See [`FLOOR_RULES`].
pub fn is_floor_rule(rule: &str) -> bool {
    FLOOR_RULES.contains(&rule)
}

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

    /// THE FLOOR, expressed as `Error` overrides for every [`FLOOR_RULES`] id.
    ///
    /// Authored `<style>`, inline `style=`, `class`, and undeclared companion
    /// stylesheets are an error in every SUI file, with or without a
    /// `sui.config.json`. This is not a policy tier and not opt-in: it is a
    /// property of the language. The ONLY exception is the L1 element sources
    /// inside this compiler, which do not take this path — they compile through
    /// `compile_element`, because they ARE the styling layer.
    ///
    /// The floor is NO LONGER delivered by threading this bundle through a host's
    /// `opts.lint_overrides` (an opt-in floor is not a floor — every unconfigured
    /// tree had a silent escape hatch, and a host that passed no overrides, like
    /// the wasm/playground path, skipped it entirely). The compiler now enforces
    /// it directly via `check::lint_overrides::enforce_floor`, keyed on the same
    /// [`FLOOR_RULES`] list. This function remains the single description of the
    /// floor-as-overrides for `Policy::overrides` and tests.
    ///
    /// When the floor bites something real — syntax highlighting, say — the
    /// answer is a missing L1 element, not a hole in the floor.
    pub fn floor() -> Vec<LintOverride> {
        FLOOR_RULES
            .iter()
            .map(|rule| LintOverride { rule: rule.to_string(), severity: OverrideSeverity::Error })
            .collect()
    }

    /// The CAPABILITY overrides the compiler should apply for `path` — the
    /// package's declared capability keys ABOVE the floor. The floor itself is
    /// enforced unconditionally by the compiler (`enforce_floor`) and is NOT part
    /// of this bundle: no host wiring can skip it. An unconfigured package
    /// declares no capabilities and gets an empty bundle (the floor still holds).
    ///
    /// THE ONE LOG SEAM: every build host (CLI, deck-bundler, napi) resolves
    /// capabilities by calling this function — so it is the single place that
    /// announces/refuses a package's resolved tier out loud. A config-loud-strict
    /// package (typo'd key, broken JSON, or a `stylesheets`-only config with no
    /// tier signal) NEVER falls to STRICT in silence again: see [`announce`].
    pub fn overrides_for(path: &Path) -> Vec<LintOverride> {
        match Policy::resolve_outcome(path) {
            Some((config_path, resolution)) => {
                announce(&config_path, &resolution);
                resolution.tier.capabilities()
            }
            None => Vec::new(),
        }
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

    /// The full `LintOverride` bundle this policy implies: the **floor**
    /// (`Error` for every [`FLOOR_RULES`] id, via [`Policy::floor`]) plus the
    /// three capability keys toggled between `Off` (allowed) and `Error`
    /// (forbidden). The floor is derived from the one [`FLOOR_RULES`] list, not
    /// re-hardcoded here. Used by tests + tooling that want the whole picture;
    /// the compiler enforces the floor half unconditionally regardless.
    pub fn overrides(self) -> Vec<LintOverride> {
        let mut out = Policy::floor();
        out.extend(self.capabilities());
        out
    }

    /// Resolve the CAPABILITY policy for a source file from the nearest
    /// `sui.config.json`, or `None` when no config is found on the walk up. This
    /// is the **opt-in** entry the deck bundler and CLI use: capability keys
    /// apply only to packages that declared a config, so they roll out
    /// per-package. The floor is orthogonal — enforced unconditionally by the
    /// compiler regardless of whether this returns `Some` or `None`.
    ///
    /// FAIL CLOSED: a present-but-unrecognized/broken config resolves to
    /// [`Policy::STRICT`] (every capability OFF), never [`Policy::FULL`]. A
    /// config's presence is the package's declared intent to constrain; if we
    /// cannot read that intent (typo, unknown keys, unparseable body), we deny
    /// capabilities rather than silently unlock the most-permissive tier. STRICT
    /// still compiles — a typo never hard-fails a build — but it grants nothing.
    pub fn resolve_opt(source_file: &Path) -> Option<Policy> {
        Policy::resolve_outcome(source_file).map(|(_, r)| r.tier)
    }

    /// Resolve the full [`PolicyResolution`] (tier + WHY) for a source file,
    /// alongside the `sui.config.json` path it came from. `None` when no
    /// config exists anywhere on the walk up — the opt-in case, unchanged from
    /// [`Policy::resolve_opt`]. This is the entry [`overrides_for`] uses to
    /// build its announce/refuse message; tests use it directly to pin the
    /// exact wording without depending on the log seam's stderr output.
    pub fn resolve_outcome(source_file: &Path) -> Option<(PathBuf, PolicyResolution)> {
        let (config_path, cfg) = sui_lint::config::resolve(source_file.parent()?)?;
        Some((config_path, classify(&cfg)))
    }
}

/// WHY a package resolved to its tier — carries enough detail for the log
/// seam ([`announce`]) to name the offending key / the signal that produced
/// the tier, per the config-loud-strict contract: silence is the bug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyResolution {
    pub tier: Policy,
    pub kind: ResolutionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionKind {
    /// A recognized signal (preset and/or capability keys) produced the tier.
    /// The joined `key: value` pairs that set it, e.g. `"preset: full"` or
    /// `"sx: true, raw_containers: false"`.
    Recognized(String),
    /// The config is well-formed and every key it names is recognized, but
    /// none of them carry tier signal (e.g. `{"stylesheets": []}` alone, or
    /// `{}`). Fails closed to STRICT — this is the original config-loud-strict
    /// bug: a config with no capability signal used to silently strip sx.
    /// `String` names which recognized-but-signal-free keys are present (or
    /// "no keys at all" for `{}`).
    NoSignal(String),
    /// The config is unparseable, OR names a key/value the parser does not
    /// recognize (a typo'd key, or a preset name that isn't `full`/`strict`).
    /// Fails closed to STRICT and REFUSES loudly. `String` names the offense.
    Rejected(String),
}

/// Derive the full tier + reason from a resolved `sui.config.json`. The read +
/// parse is owned by `sui_lint::config`; this only classifies the parsed value.
fn classify(cfg: &sui_lint::config::SuiConfig) -> PolicyResolution {
    if !cfg.parse_ok() {
        return PolicyResolution {
            tier: Policy::STRICT,
            kind: ResolutionKind::Rejected("the file is not valid JSON".to_string()),
        };
    }
    let unknown = cfg.unknown_keys();
    if !unknown.is_empty() {
        return PolicyResolution {
            tier: Policy::STRICT,
            kind: ResolutionKind::Rejected(format!(
                "unrecognized key{} {}",
                if unknown.len() == 1 { "" } else { "s" },
                unknown
                    .iter()
                    .map(|k| format!("\"{k}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        };
    }

    let mut p = Policy::STRICT;
    let mut signal = Vec::new();
    if let Some(name) = cfg.preset() {
        match Policy::preset(name) {
            Some(preset) => {
                p = preset;
                signal.push(format!("preset: {name}"));
            }
            None => {
                return PolicyResolution {
                    tier: Policy::STRICT,
                    kind: ResolutionKind::Rejected(format!(
                        "unrecognized preset \"{name}\" (expected \"full\" or \"strict\")"
                    )),
                };
            }
        }
    }
    for key in ["sx", "variants", "raw_containers"] {
        if let Some(b) = cfg.bool_key(key) {
            match key {
                "sx" => p.sx = b,
                "variants" => p.variants = b,
                "raw_containers" => p.raw_containers = b,
                _ => unreachable!(),
            }
            signal.push(format!("{key}: {b}"));
        }
    }

    if signal.is_empty() {
        let recognized_present: Vec<&str> = sui_lint::config::RECOGNIZED_KEYS
            .iter()
            .copied()
            .filter(|k| cfg.has_key(k))
            .collect();
        let hint = if recognized_present.is_empty() {
            "no keys at all".to_string()
        } else {
            recognized_present.join(", ")
        };
        PolicyResolution { tier: Policy::STRICT, kind: ResolutionKind::NoSignal(hint) }
    } else {
        PolicyResolution { tier: p, kind: ResolutionKind::Recognized(signal.join(", ")) }
    }
}

/// THE LOG SEAM. One line to stderr per `(config file, resulting tier)`, the
/// first time [`Policy::overrides_for`] resolves a given `sui.config.json` in
/// this process — dedup so a many-file package doesn't repeat itself once per
/// compiled `.sui`, while still guaranteeing at least one line per package per
/// build/dev-server run. Every wording names the file, the tier, and WHY:
///
/// - [`ResolutionKind::Rejected`] — REFUSES loudly: "typo'd/broken config,
///   here is exactly what's wrong, tier fell back to strict."
/// - [`ResolutionKind::NoSignal`] — ANNOUNCES: "this config carries no
///   capability signal, so the tier fell back to strict — here's what WAS in
///   the file." This is the config-loud-strict fix: `{"stylesheets": []}`
///   used to strip `sx` with zero output.
/// - [`ResolutionKind::Recognized`] — ANNOUNCES: "here is the tier and the
///   exact keys that produced it." No noise beyond this one line.
fn announce(config_path: &Path, resolution: &PolicyResolution) {
    static SEEN: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    {
        let mut guard = seen.lock().unwrap_or_else(|e| e.into_inner());
        if !guard.insert(config_path.to_path_buf()) {
            return;
        }
    }
    eprintln!("{}", render_announcement(config_path, resolution));
}

/// Pure message rendering — split from [`announce`] so tests can pin the
/// exact wording without capturing stderr or fighting the process-wide dedup.
fn render_announcement(config_path: &Path, resolution: &PolicyResolution) -> String {
    let tier_name = tier_name(resolution.tier);
    match &resolution.kind {
        ResolutionKind::Rejected(reason) => format!(
            "sui-compiler: REFUSED {} — {reason}; capability tier falls back to strict (no capabilities granted)",
            config_path.display()
        ),
        ResolutionKind::NoSignal(hint) => format!(
            "sui-compiler: {} — policy: strict (config declares {hint}, which carries no capability \
             signal; add \"preset\" or a capability key to grant more)",
            config_path.display()
        ),
        ResolutionKind::Recognized(signal) => format!(
            "sui-compiler: {} — policy: {tier_name} ({signal})",
            config_path.display()
        ),
    }
}

fn tier_name(p: Policy) -> &'static str {
    if p == Policy::FULL {
        "full"
    } else if p == Policy::STRICT {
        "strict"
    } else {
        "mixed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sev(p: Policy, rule: &str) -> OverrideSeverity {
        p.overrides().iter().find(|o| o.rule == rule).unwrap().severity
    }

    /// Derive a tier from config JSON text — the tier half of the resolution,
    /// with the read/parse owned by `sui_lint::config`. `classify` always
    /// resolves SOME tier (fail-closed to STRICT for the unknown/no-signal
    /// cases); [`ResolutionKind`] is what carries "why", tested separately.
    fn tier(json: &str) -> Policy {
        classify(&sui_lint::config::SuiConfig::from_json(json)).tier
    }

    fn kind(json: &str) -> ResolutionKind {
        classify(&sui_lint::config::SuiConfig::from_json(json)).kind
    }

    #[test]
    fn floor_is_error_under_every_preset_and_is_not_a_key() {
        for p in [Policy::FULL, Policy::STRICT] {
            assert_eq!(sev(p, "sui-no-style-block"), OverrideSeverity::Error);
            assert_eq!(sev(p, "sui-no-inline-style"), OverrideSeverity::Error);
        }
        // No config key can turn the floor off — only capability keys exist.
        let p = tier(r#"{"preset":"full","sx":false}"#);
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
        let p = tier(r#"{"preset":"full","raw_containers":false}"#);
        assert!(p.sx && p.variants && !p.raw_containers);
        // strict, but sx re-enabled — the in-between tier
        let p = tier(r#"{"preset":"strict","sx":true}"#);
        assert!(p.sx && !p.variants && !p.raw_containers);
        // keys only, no preset → base STRICT, dial up
        let p = tier(r#"{"sx":true}"#);
        assert!(p.sx && !p.variants && !p.raw_containers);
    }

    // ----------------------------------------------------------------------
    // config-loud-strict: `classify` always resolves a tier (fail-closed to
    // STRICT), but WHICH bucket it fails into differs — an unrecognized key
    // (typo) or unparseable body is `Rejected` (loud refusal); a well-formed
    // config whose recognized keys carry no tier signal is `NoSignal`
    // (announced, not refused — nothing is actually wrong). This is the
    // distinction the log seam (`announce`) renders differently.
    // ----------------------------------------------------------------------

    #[test]
    fn unknown_key_or_broken_json_is_rejected_and_fails_closed_to_strict() {
        assert_eq!(tier(r#"{"other":1}"#), Policy::STRICT);
        assert!(matches!(kind(r#"{"other":1}"#), ResolutionKind::Rejected(r) if r.contains("other")));

        assert_eq!(tier("not json"), Policy::STRICT);
        assert!(matches!(kind("not json"), ResolutionKind::Rejected(r) if r.contains("not valid JSON")));

        assert_eq!(tier(r#"{"preset":"bogus"}"#), Policy::STRICT);
        assert!(matches!(kind(r#"{"preset":"bogus"}"#), ResolutionKind::Rejected(r) if r.contains("bogus")));
    }

    #[test]
    fn stylesheets_only_config_is_no_signal_not_rejected() {
        // The original bug: a config declaring ONLY `stylesheets` is
        // well-formed and every key it names is recognized — it just doesn't
        // set a tier. That's `NoSignal`, not `Rejected`: nothing is typo'd.
        assert_eq!(tier(r#"{"stylesheets": []}"#), Policy::STRICT);
        assert!(matches!(
            kind(r#"{"stylesheets": []}"#),
            ResolutionKind::NoSignal(hint) if hint.contains("stylesheets")
        ));
    }

    #[test]
    fn empty_config_object_is_no_signal() {
        assert_eq!(tier("{}"), Policy::STRICT);
        assert!(matches!(kind("{}"), ResolutionKind::NoSignal(hint) if hint.contains("no keys")));
    }

    #[test]
    fn granted_keys_are_recognized_and_name_their_signal() {
        assert!(matches!(
            kind(r#"{"preset":"full"}"#),
            ResolutionKind::Recognized(s) if s == "preset: full"
        ));
        assert!(matches!(
            kind(r#"{"preset":"strict","sx":true}"#),
            ResolutionKind::Recognized(s) if s.contains("preset: strict") && s.contains("sx: true")
        ));
    }

    // ----------------------------------------------------------------------
    // Config resolution FAILS CLOSED — a broken/typo'd `sui.config.json`
    // resolves to STRICT (capabilities off), never FULL (audit finding #4).
    // ----------------------------------------------------------------------

    /// Write a `sui.config.json` in a tempdir and resolve the policy for a
    /// source under it — the real on-disk resolution path `overrides_for` uses.
    fn resolve_config(body: &str) -> Option<Policy> {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("sui.config.json"), body).unwrap();
        let src = tmp.path().join("src").join("w.sui");
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::write(&src, "<p>{x}</p>").unwrap();
        Policy::resolve_opt(&src)
    }

    #[test]
    fn broken_config_fails_closed_to_strict_not_full() {
        // An unparseable body is present-but-broken: its presence is intent to
        // constrain, but we can't read it — so deny capabilities (STRICT), never
        // unlock the most-permissive tier (FULL). This is the audit-#4 fix.
        assert_eq!(resolve_config("{ not valid json"), Some(Policy::STRICT));
        assert_ne!(resolve_config("{ not valid json"), Some(Policy::FULL));
    }

    #[test]
    fn unknown_key_config_fails_closed_to_strict() {
        // A present config with no recognized policy signal (typo'd keys) is
        // still a declaration — fail closed to STRICT, not FULL.
        assert_eq!(resolve_config(r#"{"polcy":"strict"}"#), Some(Policy::STRICT));
        assert_eq!(resolve_config(r#"{"preset":"strictest"}"#), Some(Policy::STRICT));
    }

    #[test]
    fn recognized_preset_resolves_to_its_tier() {
        assert_eq!(resolve_config(r#"{"preset":"strict"}"#), Some(Policy::STRICT));
        assert_eq!(resolve_config(r#"{"preset":"full"}"#), Some(Policy::FULL));
    }

    #[test]
    fn no_config_resolves_to_none() {
        // No `sui.config.json` anywhere on the walk → None (opt-in; the floor
        // still holds structurally, but no capability tier is imposed).
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("w.sui");
        std::fs::write(&src, "<p>{x}</p>").unwrap();
        assert_eq!(Policy::resolve_opt(&src), None);
    }

    // ----------------------------------------------------------------------
    // Doc/parser key match: the config key the parser reads is `preset`, with
    // values `full`/`strict` — NOT the drifted `{"policy":"floor|stripped|
    // strictest"}` the docs used to advertise. Standardized on `preset`.
    // ----------------------------------------------------------------------

    #[test]
    fn preset_is_the_parser_key_and_the_drifted_values_do_not_parse() {
        // `{"preset":"strict"}` — the standardized key — resolves.
        assert_eq!(tier(r#"{"preset":"strict"}"#), Policy::STRICT);
        assert_eq!(tier(r#"{"preset":"full"}"#), Policy::FULL);
        // The OLD advertised values (`floor`/`stripped`/`strictest`) are NOT
        // preset names the parser knows — proving the docs must name `full`/
        // `strict`, not the drifted vocabulary.
        assert_eq!(Policy::preset("floor"), None);
        assert_eq!(Policy::preset("stripped"), None);
        assert_eq!(Policy::preset("strictest"), None);
        // The OLD key name never carried a tier either — and being an
        // unrecognized key, it now fails closed LOUDLY (Rejected), not silently.
        assert_eq!(tier(r#"{"policy":"floor"}"#), Policy::STRICT);
        assert!(matches!(kind(r#"{"policy":"floor"}"#), ResolutionKind::Rejected(_)));
    }

    // ----------------------------------------------------------------------
    // config-loud-strict: the exact announcement/refusal text the log seam
    // renders. Pinned here (pure `render_announcement`, no stderr capture).
    // ----------------------------------------------------------------------

    #[test]
    fn rejected_message_refuses_loudly_and_names_file_and_key() {
        let cfg = sui_lint::config::SuiConfig::from_json(r#"{"polcy":"strict"}"#);
        let resolution = classify(&cfg);
        let msg = render_announcement(Path::new("/pkg/sui.config.json"), &resolution);
        assert!(msg.contains("REFUSED"), "{msg}");
        assert!(msg.contains("/pkg/sui.config.json"), "{msg}");
        assert!(msg.contains("polcy"), "{msg}");
        assert!(msg.contains("strict"), "{msg}");
    }

    #[test]
    fn no_signal_message_announces_and_names_file_and_signal() {
        let cfg = sui_lint::config::SuiConfig::from_json(r#"{"stylesheets": []}"#);
        let resolution = classify(&cfg);
        let msg = render_announcement(Path::new("/pkg/sui.config.json"), &resolution);
        assert!(!msg.contains("REFUSED"), "{msg}");
        assert!(msg.contains("/pkg/sui.config.json"), "{msg}");
        assert!(msg.contains("strict"), "{msg}");
        assert!(msg.contains("stylesheets"), "{msg}");
    }

    #[test]
    fn recognized_message_names_file_tier_and_signal_with_no_extra_noise() {
        let cfg = sui_lint::config::SuiConfig::from_json(r#"{"preset":"full"}"#);
        let resolution = classify(&cfg);
        let msg = render_announcement(Path::new("/pkg/sui.config.json"), &resolution);
        assert!(!msg.contains("REFUSED"), "{msg}");
        assert_eq!(msg.lines().count(), 1, "exactly one line: {msg}");
        assert!(msg.contains("/pkg/sui.config.json"), "{msg}");
        assert!(msg.contains("full"), "{msg}");
        assert!(msg.contains("preset: full"), "{msg}");
    }
}
