//! The styling **policy** — a non-negotiable *floor* plus a small set of
//! *capability* keys the compiler dials on or off. This is NOT a spectrum from
//! "escape hatch" to "strict": there is no blessed escape hatch anywhere. Every
//! tier forbids raw CSS.
//!
//! ## The floor — not a key, cannot be turned off
//!
//! Authored `<style>` blocks (`sail-no-style-block`), inline `style=`
//! (`sail-no-inline-style`) and `class` (`sail-no-class` — light DOM has no
//! boundary, so a class is a global CSS target: the hook that lets banned CSS
//! live in a file the compiler never sees) are a hard `Error` under **every**
//! policy, and there
//! is no config key to relax them. This is what "floor" means: the thing not
//! even the compiler's configuration allows. (The compiler's own `--styles`
//! injection is internal, not authored source.) Product Style is the tag call
//! `style({…})` — ungated. Retired `sx=` is a compile error. Residual
//! utility-string `style=` is floor (`spinnaker` / `sail-no-inline-style`).
//!
//! ## The capability keys — the dial
//!
//! One key gates structure *above* the floor:
//! - `raw_containers` — raw `<div>`/`<span>` (`sail-no-raw-container`)
//!
//! A second key controls the compose auto-fix tier:
//! - `compose_rhythm` — when `true` (default), `sail-compose-rhythm` findings
//!   are emitted at `fix` severity and may be auto-repaired; when `false`, the
//!   rule downgrades to a warning and the compiler never rewrites spacing.
//!
//! Compose autofix **preset** (orthogonal to styling full/strict):
//! - `autofix: "repair"` (default) — confluent fixes on, ambiguities warn
//! - `autofix: "strict"` — compose findings as Error (no silent rewrite)
//! - `autofix: "off"` — compose pass silent
//!
//! ## The presets
//!
//! - [`Policy::FULL`] — primitive-authoring: `raw_containers` on. Where the SAIL
//!   primitives are built. The floor still holds; Style (`style({…})`, including
//!   `match` / `when` / `compound`) is product, not a grant.
//! - [`Policy::STRICT`] — app-authoring: `raw_containers` off. Compose primitives,
//!   drive them with props, use the layout keywords — no raw boxes.
//!
//! A `sail.config.json` may name a preset (`{ "preset": "strict" }`) and/or set
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

/// The floor rule IDs — the styling constructs banned in EVERY `.sail` compiled
/// as a component or route, with or without a `sail.config.json`. This is the
/// single source of truth: [`is_floor_rule`], [`Policy::floor`], and the
/// compiler's unconditional `enforce_floor` pass
/// (`check::lint_overrides::enforce_floor`) all read this one list, so a floor
/// rule added here is enforced everywhere with no second place to update.
///
/// `spinnaker` (the typo/unknown-atom catcher) joined the floor for
/// compiler-stability-audit #100 item 1: an unknown atom must NEVER become a
/// silent-nothing. Before this, `spinnaker` rode the ordinary override channel,
/// so a host that softened it to `Warn`/`Off` turned a typo'd atom into a dead
/// class. As a floor rule, `spinnaker` is `Error` under every policy and
/// `check::gate` refuses the build before the dead class ever reaches emitted
/// output.
pub const FLOOR_RULES: [&str; 5] = [
    "sail-no-style-block",
    "sail-no-inline-style",
    "sail-no-class",
    "sail-no-undeclared-stylesheet",
    "spinnaker",
];

/// The CAPABILITY rule ids — the dial ABOVE the floor, one per key on
/// [`Policy`]. Unlike the floor these are opt-in: they gate a surface a package
/// declares it wants closed. [`Policy::capabilities`] and
/// [`Policy::capability_defaults_for`] both derive from this one list.
pub const CAPABILITY_RULES: [&str; 1] = ["sail-no-raw-container"];

/// True iff `rule` is a floor rule — one the compiler promotes to `Error`
/// unconditionally and that no `lint_override` can soften. See [`FLOOR_RULES`].
pub fn is_floor_rule(rule: &str) -> bool {
    FLOOR_RULES.contains(&rule)
}

/// Compose autofix preset (AUTOFIX-1 contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutofixPreset {
    /// Confluent fixes on; ambiguities warn. Default.
    #[default]
    Repair,
    /// All compose findings as Error (no auto-rewrite path).
    Strict,
    /// Compose pass off.
    Off,
}

impl AutofixPreset {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "repair" | "fix" | "on" => Some(Self::Repair),
            "strict" | "error" => Some(Self::Strict),
            "off" | "false" | "0" => Some(Self::Off),
            _ => None,
        }
    }
}

/// A package's styling policy: the (implicit, non-overridable) floor plus the
/// remaining capability key (`raw_containers`) and the compose auto-fix key.
/// See the module docs. Product Style is not a field — it is ungated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    /// Raw `<div>`/`<span>` containers.
    pub raw_containers: bool,
    /// Auto-fix sibling rhythm violations (`sail-compose-rhythm`).
    pub compose_rhythm: bool,
    /// Compose autofix preset: repair | strict | off.
    pub autofix: AutofixPreset,
}

impl Policy {
    /// `full` — primitive-authoring: `raw_containers` unlocked (the floor still
    /// holds). Where the primitives are built and dissolved. Style is product.
    pub const FULL: Policy = Policy {
        raw_containers: true,
        compose_rhythm: true,
        autofix: AutofixPreset::Repair,
    };

    /// `strict` — app-authoring: `raw_containers` off. Compose primitives and
    /// use their props; no raw boxes. Style (`style({…})`) stays legal.
    pub const STRICT: Policy = Policy {
        raw_containers: false,
        compose_rhythm: true,
        autofix: AutofixPreset::Repair,
    };

    /// Parse a preset name. Unknown names return `None`.
    pub fn preset(name: &str) -> Option<Policy> {
        match name.trim() {
            "full" => Some(Policy::FULL),
            "strict" => Some(Policy::STRICT),
            // Autofix-only presets keep capability defaults of FULL and set autofix.
            "repair" => Some(Policy {
                autofix: AutofixPreset::Repair,
                ..Policy::FULL
            }),
            "off" => Some(Policy {
                autofix: AutofixPreset::Off,
                compose_rhythm: false,
                ..Policy::FULL
            }),
            _ => None,
        }
    }

    /// THE FLOOR, expressed as `Error` overrides for every [`FLOOR_RULES`] id.
    ///
    /// Authored `<style>`, inline `style=`, `class`, and undeclared companion
    /// stylesheets are an error in every SAIL file, with or without a
    /// `sail.config.json`. This is not a policy tier and not opt-in: it is a
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
            .map(|rule| LintOverride {
                rule: rule.to_string(),
                severity: OverrideSeverity::Error,
            })
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
                // Capability dial + registry lint.severity.* tree — one bundle.
                let mut out = resolution.tier.capabilities();
                out.extend(resolution.lint_severity_overrides);
                out
            }
            None => Vec::new(),
        }
    }

    /// The capability keys only — the floor is added separately by `overrides_for`.
    pub fn capabilities(self) -> Vec<LintOverride> {
        use OverrideSeverity::{Error, Off};
        CAPABILITY_RULES
            .iter()
            .map(|&rule| LintOverride {
                rule: rule.to_string(),
                severity: if self.grants(rule) { Off } else { Error },
            })
            .collect()
    }

    /// Does this policy grant the capability the named rule gates? Unknown
    /// rule ids are not capabilities and are never granted here.
    fn grants(self, rule: &str) -> bool {
        match rule {
            "sail-no-raw-container" => self.raw_containers,
            _ => false,
        }
    }

    /// The `sail-compose-rhythm` severity implied by this policy: `Fix` when
    /// auto-fix is enabled (the default), `Warning` when the package opts out.
    pub fn compose_rhythm_severity(self) -> OverrideSeverity {
        match self.autofix {
            AutofixPreset::Off => OverrideSeverity::Off,
            AutofixPreset::Strict => OverrideSeverity::Error,
            AutofixPreset::Repair => {
                if self.compose_rhythm {
                    OverrideSeverity::Fix
                } else {
                    OverrideSeverity::Warn
                }
            }
        }
    }

    /// The capability-rule severities that apply to `path` when the host has
    /// NOT named the rule itself — the seam that keeps `analyze()` honest
    /// about what `compile()` will do.
    ///
    /// The capability rules carry a natural `Warn` so an unconfigured package
    /// keeps compiling. A capability is a thing a package DECLARES; with no
    /// `sail.config.json` in reach there is no policy to violate, so the rule
    /// is `Off` rather than `Warn`. Where a config does exist, its resolved
    /// tier decides (`Off` granted / `Error` withheld) — the same answer
    /// [`Policy::overrides_for`] hands the build host, so both surfaces
    /// read one resolver.
    ///
    /// Host-supplied overrides still win: anything already named in
    /// `host_overrides` is left alone.
    pub fn capability_defaults_for(
        path: &Path,
        host_overrides: &[LintOverride],
    ) -> Vec<LintOverride> {
        let declared = Policy::resolve_opt(path);
        CAPABILITY_RULES
            .iter()
            .filter(|&&rule| !host_overrides.iter().any(|o| o.rule == rule))
            .map(|&rule| LintOverride {
                rule: rule.to_string(),
                severity: match declared {
                    None => OverrideSeverity::Off,
                    Some(p) if p.grants(rule) => OverrideSeverity::Off,
                    Some(_) => OverrideSeverity::Error,
                },
            })
            .collect()
    }

    /// The full `LintOverride` bundle this policy implies: the **floor**
    /// (`Error` for every [`FLOOR_RULES`] id, via [`Policy::floor`]) plus the
    /// remaining capability key (`raw_containers`) toggled between `Off`
    /// (allowed) and `Error` (forbidden). The floor is derived from the one
    /// [`FLOOR_RULES`] list, not re-hardcoded here. Used by tests + tooling
    /// that want the whole picture; the compiler enforces the floor half
    /// unconditionally regardless.
    pub fn overrides(self) -> Vec<LintOverride> {
        let mut out = Policy::floor();
        out.extend(self.capabilities());
        out
    }

    /// Resolve the CAPABILITY policy for a source file from the nearest
    /// `sail.config.json`, or `None` when no config is found on the walk up. This
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
    /// alongside the `sail.config.json` path it came from. `None` when no
    /// config exists anywhere on the walk up — the opt-in case, unchanged from
    /// [`Policy::resolve_opt`]. This is the entry [`overrides_for`] uses to
    /// build its announce/refuse message; tests use it directly to pin the
    /// exact wording without depending on the log seam's stderr output.
    pub fn resolve_outcome(source_file: &Path) -> Option<(PathBuf, PolicyResolution)> {
        // std::fs panics on wasm32 (unsupported platform op): no config
        // resolution in the browser — defaults apply, same as "no config found".
        #[cfg(target_arch = "wasm32")]
        {
            let _ = source_file;
            return None;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (config_path, cfg) = sail_lint::config::resolve(source_file.parent()?)?;
            Some((config_path, classify(&cfg)))
        }
    }

    /// The resolved `"network"` per-origin allowlist for a source file's
    /// nearest `sail.config.json` — the fetch/WebSocket/EventSource capability
    /// grant that drives the network-aware lint pass
    /// (`sail_lint::rules::controller_no_ambient_authority::check_confined`).
    /// This is NOT a second `sail.config.json` reader: it re-derives from
    /// [`Policy::resolve_outcome`], the same walk-up/parse `overrides_for`
    /// uses. Empty when no config exists, the config fails closed
    /// (`Rejected`), or the config simply never declares `"network"` —
    /// today's full refusal, unchanged for anyone who doesn't opt in.
    pub fn network_for(source_file: &Path) -> Vec<String> {
        Policy::resolve_outcome(source_file)
            .map(|(_, r)| r.network)
            .unwrap_or_default()
    }

    /// Whether a source file's nearest `sail.config.json` grants the
    /// `"unconfined"` capability — the whole-hog opt-out of the
    /// confined-by-default `sail-controller-no-ambient-authority` gate
    /// (`sail_lint::rules::controller_no_ambient_authority::check_confined`).
    /// Re-derives from [`Policy::resolve_outcome`], the same resolver
    /// `overrides_for`/`network_for` use — not a second `sail.config.json`
    /// reader. `false` when no config exists, the config fails closed
    /// (`Rejected`), or the config simply never declares `"unconfined"` —
    /// confined by default, unchanged for anyone who doesn't opt in.
    pub fn unconfined_for(source_file: &Path) -> bool {
        Policy::resolve_outcome(source_file)
            .map(|(_, r)| r.unconfined)
            .unwrap_or(false)
    }
}

/// WHY a package resolved to its tier — carries enough detail for the log
/// seam ([`announce`]) to name the offending key / the signal that produced
/// the tier, per the config-loud-strict contract: silence is the bug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyResolution {
    pub tier: Policy,
    pub kind: ResolutionKind,
    /// The `"network"` per-origin allowlist declared by this config, or empty
    /// when absent — the fetch/WebSocket/EventSource capability grant. Empty
    /// on every `Rejected` resolution too: a config that fails closed grants
    /// nothing, regardless of what its (unread) `"network"` value might have
    /// said. Orthogonal to `tier` — the styling capability keys don't gate it
    /// and it doesn't gate them.
    pub network: Vec<String>,
    /// The declared `"unconfined"` capability — whole-hog opt-out of the
    /// confined-by-default `sail-controller-no-ambient-authority` gate.
    /// `false` (confined) by default, and on every `Rejected` resolution too:
    /// a config that fails closed grants nothing. Orthogonal to `tier` and
    /// `network` — the styling capability keys and the per-origin network
    /// grant don't gate it and it doesn't gate them.
    pub unconfined: bool,
    /// Registry `lint.severity.<rule-id>` overrides from the `"policy"` object.
    /// Empty on every `Rejected` resolution and when the tree is unset.
    pub lint_severity_overrides: Vec<LintOverride>,
}

fn rejected(reason: String) -> PolicyResolution {
    PolicyResolution {
        tier: Policy::STRICT,
        kind: ResolutionKind::Rejected(reason),
        network: Vec::new(),
        unconfined: false,
        lint_severity_overrides: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionKind {
    /// A recognized signal (preset and/or capability keys) produced the tier.
    /// The joined `key: value` pairs that set it, e.g. `"preset: full"` or
    /// `"raw_containers: false"`.
    Recognized(String),
    /// The config is well-formed and every key it names is recognized, but
    /// none of them carry tier signal (e.g. `{"stylesheets": []}` alone, or
    /// `{}`). Fails closed to STRICT — this is the original config-loud-strict
    /// bug: a config with no capability signal used to silently lock the
    /// surface. `String` names which recognized-but-signal-free keys are
    /// present (or "no keys at all" for `{}`).
    NoSignal(String),
    /// The config is unparseable, OR names a key/value the parser does not
    /// recognize (a typo'd key, or a preset name that isn't `full`/`strict`).
    /// Fails closed to STRICT and REFUSES loudly. `String` names the offense.
    Rejected(String),
}

/// Derive the full tier + reason from a resolved `sail.config.json`. The read +
/// parse is owned by `sail_lint::config`; this only classifies the parsed value.
fn classify(cfg: &sail_lint::config::SailConfig) -> PolicyResolution {
    if !cfg.parse_ok() {
        return rejected("the file is not valid JSON".to_string());
    }
    let unknown = cfg.unknown_keys();
    if !unknown.is_empty() {
        // SPEC-HARDENING-1 WI5: validation errors name the allowed keys.
        let allowed = sail_lint::config::recognized_keys().join(", ");
        return rejected(format!(
            "unrecognized key{} {} (allowed keys: {allowed})",
            if unknown.len() == 1 { "" } else { "s" },
            unknown
                .iter()
                .map(|k| format!("\"{k}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(err) = cfg.type_error() {
        return rejected(err.to_string());
    }

    let registry = match crate::policy::apply_config_to_resolved(cfg) {
        Ok(r) => r,
        Err(e) => return rejected(e.message),
    };
    let lint_severity_overrides = registry.lint_overrides();

    let mut p = Policy::STRICT;
    let mut signal = Vec::new();
    if let Some(name) = cfg.preset() {
        match Policy::preset(name) {
            Some(preset) => {
                p = preset;
                signal.push(format!("preset: {name}"));
            }
            None => {
                return rejected(format!(
                    "unrecognized preset \"{name}\" (expected \"full\", \"strict\", \"repair\", or \"off\")"
                ));
            }
        }
    }
    if let Some(b) = cfg.raw_containers {
        p.raw_containers = b;
        signal.push(format!("raw_containers: {b}"));
    }
    if let Some(b) = cfg.compose_rhythm {
        p.compose_rhythm = b;
        signal.push(format!("compose_rhythm: {b}"));
    }
    if !registry.lint_severities.is_empty() {
        signal.push(format!(
            "lint.rules: {} override(s)",
            registry.lint_severities.len()
        ));
    }

    // SPEC-LINTS-1 §3 — top-level strict + lint severity floor are operator
    // signal (not unknown keys, not NoSignal-only). Floor alone is enough to
    // avoid fail-closed STRICT with zero announce when an app opts in.
    if cfg.has_key("strict") {
        signal.push(format!("strict: {}", registry.strict));
    }
    if let Some(floor) = registry.lint_severity_floor {
        signal.push(format!("lint.severity: {}", floor.as_str()));
    } else if cfg.has_key("lint") && registry.lint_severities.is_empty() {
        // Present lint object with neither floor nor rules — still signal so
        // a typo'd nested shape is not silent NoSignal after recognition.
        signal.push("lint: present".to_string());
    }
    if let Some(af) = cfg.autofix {
        let preset = match af {
            sail_lint::config::AutofixSetting::Repair => AutofixPreset::Repair,
            sail_lint::config::AutofixSetting::Strict => AutofixPreset::Strict,
            sail_lint::config::AutofixSetting::Off => AutofixPreset::Off,
        };
        p.autofix = preset;
        if preset == AutofixPreset::Off {
            p.compose_rhythm = false;
        }
        signal.push(format!("autofix: {}", af.as_str()));
    }

    // `"network"` — the fetch/WebSocket/EventSource per-origin allowlist.
    // Orthogonal to the styling tier: it never touches `p`, only `network`.
    // Wrong-typed values are refused above via `type_error`. An explicit
    // empty array is well-typed and contributes signal like any other
    // explicit capability key.
    let mut network = Vec::new();
    if let Some(origins) = cfg.network() {
        signal.push(format!("network: [{}]", origins.join(", ")));
        network = origins.to_vec();
    }

    // `"unconfined"` — the whole-hog opt-out of the confined-by-default
    // `sail-controller-no-ambient-authority` gate. Wrong-typed values are
    // refused above via `type_error`.
    let mut unconfined = false;
    if let Some(b) = cfg.unconfined() {
        signal.push(format!("unconfined: {b}"));
        unconfined = b;
    }

    // `"layout"` — severity profile for taffy / height-chain findings. Does not
    // change capability keys; counts as signal so a layout-only config is not
    // classified as NoSignal. Actual promotion is `layout_is_error_for`.
    if let Some(layout) = cfg.layout {
        signal.push(format!("layout: {}", layout.as_str()));
    }

    if signal.is_empty() {
        let recognized_present = cfg.present_recognized_keys();
        let hint = if recognized_present.is_empty() {
            "no keys at all".to_string()
        } else {
            recognized_present.join(", ")
        };
        PolicyResolution {
            tier: Policy::STRICT,
            kind: ResolutionKind::NoSignal(hint),
            network: Vec::new(),
            unconfined: false,
            lint_severity_overrides: Vec::new(),
        }
    } else {
        PolicyResolution {
            tier: p,
            kind: ResolutionKind::Recognized(signal.join(", ")),
            network,
            unconfined,
            lint_severity_overrides,
        }
    }
}

/// THE LOG SEAM. One line to stderr per `(config file, resulting tier)`, the
/// first time [`Policy::overrides_for`] resolves a given `sail.config.json` in
/// this process — dedup so a many-file package doesn't repeat itself once per
/// compiled `.sail`, while still guaranteeing at least one line per package per
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
            "sail-compiler: REFUSED {} — {reason}; capability tier falls back to strict (no capabilities granted)",
            config_path.display()
        ),
        ResolutionKind::NoSignal(hint) => format!(
            "sail-compiler: {} — policy: strict (config declares {hint}, which carries no capability \
             signal; add \"preset\" or a capability key to grant more)",
            config_path.display()
        ),
        ResolutionKind::Recognized(signal) => format!(
            "sail-compiler: {} — policy: {tier_name} ({signal})",
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
        p.overrides()
            .iter()
            .find(|o| o.rule == rule)
            .unwrap()
            .severity
    }

    /// Derive a tier from config JSON text — the tier half of the resolution,
    /// with the read/parse owned by `sail_lint::config`. `classify` always
    /// resolves SOME tier (fail-closed to STRICT for the unknown/no-signal
    /// cases); [`ResolutionKind`] is what carries "why", tested separately.
    fn tier(json: &str) -> Policy {
        classify(&sail_lint::config::SailConfig::from_json(json)).tier
    }

    fn kind(json: &str) -> ResolutionKind {
        classify(&sail_lint::config::SailConfig::from_json(json)).kind
    }

    #[test]
    fn floor_is_error_under_every_preset_and_is_not_a_key() {
        for p in [Policy::FULL, Policy::STRICT] {
            assert_eq!(sev(p, "sail-no-style-block"), OverrideSeverity::Error);
            assert_eq!(sev(p, "sail-no-inline-style"), OverrideSeverity::Error);
        }
        // No config key can turn the floor off — only remaining capability keys exist.
        let p = tier(r#"{"preset":"full","raw_containers":false}"#);
        assert_eq!(sev(p, "sail-no-style-block"), OverrideSeverity::Error);
    }

    #[test]
    fn full_unlocks_raw_containers() {
        assert_eq!(
            sev(Policy::FULL, "sail-no-raw-container"),
            OverrideSeverity::Off
        );
    }

    #[test]
    fn strict_forbids_raw_containers() {
        assert_eq!(
            sev(Policy::STRICT, "sail-no-raw-container"),
            OverrideSeverity::Error
        );
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
        assert!(!p.raw_containers);
        // keys only, no preset → base STRICT, dial up
        let p = tier(r#"{"raw_containers":true}"#);
        assert!(p.raw_containers);
    }

    #[test]
    fn leftover_sx_and_variants_keys_are_unknown_and_refuse() {
        // Product Style / variance are ungated. Leftover grant keys must not
        // silently pass — they are unknown top-level keys.
        for key in ["sx", "variants"] {
            let body = format!(r#"{{"{key}":true}}"#);
            assert_eq!(tier(&body), Policy::STRICT, "key={key}");
            let k = kind(&body);
            let ResolutionKind::Rejected(r) = k else {
                panic!("leftover `{key}` must Rejected; got {k:?}");
            };
            assert!(
                r.contains(key) && r.contains("allowed keys:"),
                "must name leftover key `{key}`: {r}"
            );
        }
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
        assert!(
            matches!(kind(r#"{"other":1}"#), ResolutionKind::Rejected(r) if r.contains("other"))
        );

        assert_eq!(tier("not json"), Policy::STRICT);
        assert!(
            matches!(kind("not json"), ResolutionKind::Rejected(r) if r.contains("not valid JSON"))
        );

        assert_eq!(tier(r#"{"preset":"bogus"}"#), Policy::STRICT);
        assert!(
            matches!(kind(r#"{"preset":"bogus"}"#), ResolutionKind::Rejected(r) if r.contains("bogus"))
        );
    }

    /// SPEC-SAIL-CONFIG-1 C1: top-level serve keys refuse; nested `deck` is ok.
    #[test]
    fn top_level_serve_keys_rejected_nested_deck_recognized() {
        for key in ["render", "entry", "injectGlobals"] {
            let body = format!(r#"{{"{key}":"csr"}}"#);
            assert_eq!(tier(&body), Policy::STRICT, "key={key}");
            let k = kind(&body);
            let ResolutionKind::Rejected(r) = k else {
                panic!("top-level `{key}` must Rejected; got {k:?}");
            };
            assert!(
                r.contains(key) && r.contains("allowed keys:"),
                "must name offending key and allowed list: {r}"
            );
            // Allowed set includes nested `deck` + SPEC-LINTS-1 strict/lint,
            // not the retired top-level serve keys or deleted policy/editor/fonts.
            assert!(
                r.contains("deck") && r.contains("strict") && r.contains("lint"),
                "allowed list should name deck/strict/lint: {r}"
            );
            assert!(
                !r.contains("editor") && !r.contains("policy") && !r.contains("fonts"),
                "deleted keys must not be allowed: {r}"
            );
        }
        // Nested deck alone is well-formed (no capability signal) — NoSignal, not Rejected.
        assert!(matches!(
            kind(r#"{"deck":{"render":"ssr"}}"#),
            ResolutionKind::NoSignal(hint) if hint.contains("deck")
        ));
        // deck + capability is Recognized, not Rejected.
        assert!(matches!(
            kind(r#"{"deck":{"render":"csr"},"preset":"full"}"#),
            ResolutionKind::Recognized(s) if s.contains("preset: full")
        ));
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
            kind(r#"{"preset":"strict","raw_containers":true}"#),
            ResolutionKind::Recognized(s) if s.contains("preset: strict") && s.contains("raw_containers: true")
        ));
    }

    // ----------------------------------------------------------------------
    // Config resolution FAILS CLOSED — a broken/typo'd `sail.config.json`
    // resolves to STRICT (capabilities off), never FULL (audit finding #4).
    // ----------------------------------------------------------------------

    /// Write a `sail.config.json` in a tempdir and resolve the policy for a
    /// source under it — the real on-disk resolution path `overrides_for` uses.
    fn resolve_config(body: &str) -> Option<Policy> {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("sail.config.json"), body).unwrap();
        let src = tmp.path().join("src").join("w.sail");
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
        assert_eq!(
            resolve_config(r#"{"polcy":"strict"}"#),
            Some(Policy::STRICT)
        );
        assert_eq!(
            resolve_config(r#"{"preset":"strictest"}"#),
            Some(Policy::STRICT)
        );
    }

    #[test]
    fn recognized_preset_resolves_to_its_tier() {
        assert_eq!(
            resolve_config(r#"{"preset":"strict"}"#),
            Some(Policy::STRICT)
        );
        assert_eq!(resolve_config(r#"{"preset":"full"}"#), Some(Policy::FULL));
    }

    #[test]
    fn no_config_resolves_to_none() {
        // No `sail.config.json` anywhere on the walk → None (opt-in; the floor
        // still holds structurally, but no capability tier is imposed).
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("w.sail");
        std::fs::write(&src, "<p>{x}</p>").unwrap();
        assert_eq!(Policy::resolve_opt(&src), None);
    }

    // ----------------------------------------------------------------------
    // config-loud-strict: the exact announcement/refusal text the log seam
    // renders. Pinned here (pure `render_announcement`, no stderr capture).
    // ----------------------------------------------------------------------

    #[test]
    fn rejected_message_refuses_loudly_and_names_file_and_key() {
        let cfg = sail_lint::config::SailConfig::from_json(r#"{"polcy":"strict"}"#);
        let resolution = classify(&cfg);
        let msg = render_announcement(Path::new("/pkg/sail.config.json"), &resolution);
        assert!(msg.contains("REFUSED"), "{msg}");
        assert!(msg.contains("/pkg/sail.config.json"), "{msg}");
        assert!(msg.contains("polcy"), "{msg}");
        assert!(msg.contains("strict"), "{msg}");
    }

    #[test]
    fn no_signal_message_announces_and_names_file_and_signal() {
        let cfg = sail_lint::config::SailConfig::from_json(r#"{"stylesheets": []}"#);
        let resolution = classify(&cfg);
        let msg = render_announcement(Path::new("/pkg/sail.config.json"), &resolution);
        assert!(!msg.contains("REFUSED"), "{msg}");
        assert!(msg.contains("/pkg/sail.config.json"), "{msg}");
        assert!(msg.contains("strict"), "{msg}");
        assert!(msg.contains("stylesheets"), "{msg}");
    }

    // ----------------------------------------------------------------------
    // `"network"` — the per-origin allowlist. Orthogonal to the styling tier;
    // parsed by `classify`, exposed via `PolicyResolution::network` and
    // `Policy::network_for`.
    // ----------------------------------------------------------------------

    #[test]
    fn absent_network_key_resolves_to_an_empty_allowlist() {
        let r = classify(&sail_lint::config::SailConfig::from_json(
            r#"{"preset":"full"}"#,
        ));
        assert!(r.network.is_empty());
    }

    #[test]
    fn declared_network_key_resolves_its_origins_and_is_recognized() {
        let r = classify(&sail_lint::config::SailConfig::from_json(
            r#"{"network":["https://api.example.com","self"]}"#,
        ));
        assert_eq!(
            r.network,
            vec!["https://api.example.com".to_string(), "self".to_string()]
        );
        assert!(matches!(&r.kind, ResolutionKind::Recognized(s) if s.contains("network:")));
    }

    #[test]
    fn empty_network_array_is_a_well_typed_no_grant_declaration() {
        // Explicit `[]` is not a type error — it's a declared "grant nothing",
        // same bucket as `"raw_containers": false`: Recognized, not Rejected.
        let r = classify(&sail_lint::config::SailConfig::from_json(
            r#"{"network":[]}"#,
        ));
        assert!(r.network.is_empty());
        assert!(
            matches!(&r.kind, ResolutionKind::Recognized(_)),
            "{:?}",
            r.kind
        );
    }

    #[test]
    fn non_array_network_value_is_a_type_error_and_fails_closed() {
        // `{"network": "https://x.com"}` — a string, not an array — must never
        // be silently treated as granting. Rejected, empty allowlist.
        let r = classify(&sail_lint::config::SailConfig::from_json(
            r#"{"network":"https://x.com"}"#,
        ));
        assert!(r.network.is_empty());
        assert!(
            matches!(&r.kind, ResolutionKind::Rejected(reason) if reason.contains("network")),
            "{:?}",
            r.kind
        );
    }

    #[test]
    fn non_string_element_in_network_array_is_a_type_error() {
        let r = classify(&sail_lint::config::SailConfig::from_json(
            r#"{"network":[1, 2]}"#,
        ));
        assert!(r.network.is_empty());
        assert!(
            matches!(&r.kind, ResolutionKind::Rejected(_)),
            "{:?}",
            r.kind
        );
    }

    #[test]
    fn network_for_resolves_on_disk_and_defaults_to_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("sail.config.json"),
            r#"{"network":["https://api.example.com"]}"#,
        )
        .unwrap();
        let src = tmp.path().join("src").join("w.controller.ts");
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::write(&src, "export default {}").unwrap();
        assert_eq!(
            Policy::network_for(&src),
            vec!["https://api.example.com".to_string()]
        );

        // No config anywhere on the walk → empty (today's default refusal).
        let tmp2 = tempfile::tempdir().unwrap();
        let src2 = tmp2.path().join("w.controller.ts");
        std::fs::write(&src2, "export default {}").unwrap();
        assert!(Policy::network_for(&src2).is_empty());
    }

    #[test]
    fn network_announcement_names_the_granted_origins() {
        // THE loud seam: granting network must name the origins, not just
        // say "granted: true" — same shape as the existing preset/capability
        // announcements.
        let cfg = sail_lint::config::SailConfig::from_json(
            r#"{"network":["https://api.example.com","self"]}"#,
        );
        let resolution = classify(&cfg);
        let msg = render_announcement(Path::new("/pkg/sail.config.json"), &resolution);
        assert!(!msg.contains("REFUSED"), "{msg}");
        assert!(msg.contains("/pkg/sail.config.json"), "{msg}");
        assert!(msg.contains("https://api.example.com"), "{msg}");
        assert!(msg.contains("self"), "{msg}");
    }

    #[test]
    fn non_array_network_announcement_refuses_loudly_and_names_the_key() {
        let cfg = sail_lint::config::SailConfig::from_json(r#"{"network":"https://x.com"}"#);
        let resolution = classify(&cfg);
        let msg = render_announcement(Path::new("/pkg/sail.config.json"), &resolution);
        assert!(msg.contains("REFUSED"), "{msg}");
        assert!(msg.contains("network"), "{msg}");
    }

    #[test]
    fn recognized_message_names_file_tier_and_signal_with_no_extra_noise() {
        let cfg = sail_lint::config::SailConfig::from_json(r#"{"preset":"full"}"#);
        let resolution = classify(&cfg);
        let msg = render_announcement(Path::new("/pkg/sail.config.json"), &resolution);
        assert!(!msg.contains("REFUSED"), "{msg}");
        assert_eq!(msg.lines().count(), 1, "exactly one line: {msg}");
        assert!(msg.contains("/pkg/sail.config.json"), "{msg}");
        assert!(msg.contains("full"), "{msg}");
        assert!(msg.contains("preset: full"), "{msg}");
    }

    #[test]
    fn styling_preset_strict_is_not_autofix_strict() {
        // `preset: "strict"` turns styling capabilities off; autofix stays Repair.
        let p = tier(r#"{"preset":"strict"}"#);
        assert_eq!(p, Policy::STRICT);
        assert_eq!(p.autofix, AutofixPreset::Repair);
        assert_eq!(p.compose_rhythm_severity(), OverrideSeverity::Fix);
    }

    #[test]
    fn autofix_key_sets_compose_severity() {
        let repair = tier(r#"{"preset":"full","autofix":"repair"}"#);
        assert_eq!(repair.autofix, AutofixPreset::Repair);
        assert_eq!(repair.compose_rhythm_severity(), OverrideSeverity::Fix);

        let strict = tier(r#"{"preset":"full","autofix":"strict"}"#);
        assert_eq!(strict.autofix, AutofixPreset::Strict);
        assert_eq!(strict.compose_rhythm_severity(), OverrideSeverity::Error);

        let off = tier(r#"{"preset":"full","autofix":"off"}"#);
        assert_eq!(off.autofix, AutofixPreset::Off);
        assert_eq!(off.compose_rhythm_severity(), OverrideSeverity::Off);
    }

    #[test]
    fn autofix_preset_names_parse() {
        assert_eq!(AutofixPreset::parse("repair"), Some(AutofixPreset::Repair));
        assert_eq!(AutofixPreset::parse("strict"), Some(AutofixPreset::Strict));
        assert_eq!(AutofixPreset::parse("off"), Some(AutofixPreset::Off));
        assert_eq!(AutofixPreset::parse("nope"), None);
    }
}
