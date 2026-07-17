//! Per-rule severity overrides for the compiler's build gate.
//!
//! The lint engine (`sail-lint`) emits findings at their natural severity
//! (Error for the typo-catching rules like `sail-css-token`; Warn for
//! surfaced-but-non-blocking rules like `sail-for-key`). The compiler's
//! build gate (`check::run` + `check::gate`) only promotes Error-severity
//! findings to a hard build failure.
//!
//! An external SUI consumer (a third-party design system, a porting
//! target, a tutorial the operator wants to ship as-is without pruning
//! every `var(--…)` reference) can opt out of a specific rule's hard
//! wall without modifying the rule itself. The override is applied
//! after the lint emits its findings; the rule's natural severity is
//! preserved for everyone who consumes the lint directly (the LSP,
//! `sail_lint` CLI, `ship_view_lint`). The compiler just downgrades the
//! rule's gate impact.
//!
//! Overrides mostly *soften* (Error → Warn or Off) so an external consumer can
//! opt out of a rule without editing it. They can also *promote* (→ Error):
//! the styling policy (`check::policy::Policy`) uses that direction to turn a
//! migration-Warn capability rule (`sail-no-raw-container`) into a hard wall for
//! the `full`/`strict` presets. Promotion is reserved for those policy rules —
//! arbitrary Warn rules are tightened by editing the rule.
//!
//! THE FLOOR is NOT part of this override channel. The floor rules
//! (`check::policy::FLOOR_RULES`) are enforced unconditionally by
//! [`enforce_floor`], which runs on every compile independent of `overrides`.
//! [`apply_overrides`] additionally REFUSES to touch a floor rule — an
//! `Off`/`Warn` override naming a floor rule id is ignored — so the floor
//! cannot be softened through this channel either. Between the two, a floor
//! violation is an `Error` in every `.sui`, on every host, with any overrides.
//!
//! Plumbing: `CompileOpts::lint_overrides` carries a `Vec<LintOverride>`
//! from the build host (CLI / napi / wasm) into `compile()`. The
//! `compile()` entry point threads the slice into `check::run` (for
//! the dedicated `css_token::check` hard-wall) and `check::gate` (for
//! the full Error-severity lint stream). `check_all` (the LSP entry
//! point) also accepts the slice and applies it identically, so the
//! editor feedback matches the build.
//!
//! Surface: the only public API the host touches is
//! [`CompileOpts::with_lint_override`], which appends an entry and
//! returns the modified opts (builder style). The host typically sets
//! overrides once at startup and reuses the resulting `CompileOpts`
//! for every compile call.
//!
//! Out of scope:
//! - Per-rule *config* beyond severity (a rule's allowlist, its
//!   suggestion strategy, etc.) — none of the existing rules carry
//!   such config, and inventing a generic config surface would be a
//!   new config system. The redirect forbids that.
//! - Per-source-path overrides. The list is process-wide for a given
//!   `CompileOpts`; a host that wants path-specific behavior can
//!   construct a new `CompileOpts` per file.

use serde::{Deserialize, Serialize};

/// What to do with a rule's findings at the build gate. The lint still
/// produces them at the rule's natural severity; the override is a
/// build-time downgrader.
///
/// The internal monorepo compiles with the default (no overrides) and
/// stays strict — Error hard-fails. External consumers can opt out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverrideSeverity {
    /// Downgrade Error findings to Warning for the named rule. The
    /// build proceeds; the finding still surfaces in the LSP / agent
    /// feedback so authors can see the typo suggestion.
    Warn,
    /// Drop the rule's findings entirely at the build gate. The build
    /// proceeds; no diagnostic is emitted (the LSP still surfaces the
    /// finding if the consumer wants — `sail-lint`'s `lint_path` is
    /// unaffected). The opt-out.
    Off,
    /// PROMOTE the rule's findings to Error at the build gate. Unlike
    /// `Warn`/`Off` (which soften), this *tightens*. It is how the styling
    /// policy (`check::policy::Policy`) turns a migration-Warn rule
    /// (`sail-no-style-block`, `sail-no-raw-container`) into a hard wall for the
    /// `full`/`strict` presets — without editing the rule. The rule's natural
    /// Warn still stands for an unconfigured package.
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintOverride {
    /// Rule ID — e.g. `"sail-css-token"`. The set of valid IDs is
    /// `sail_lint::all_rule_ids()`. The compiler does NOT validate
    /// (forward-compat — a typo simply has no effect, which is the
    /// safe failure mode).
    pub rule: String,
    pub severity: OverrideSeverity,
}

/// Apply a list of overrides to a finding stream from `sail-lint`.
/// Returns the same findings, with severity adjusted (Warn downgrade)
/// or removed (Off). Order preserved; positions unchanged.
///
/// The function is pure: it never mutates the input slice. The
/// override slice is consulted linearly per finding; the override
/// list is small (typically 0–3 entries) so a HashMap would be more
/// code than it saves.
pub fn apply_overrides(
    findings: Vec<sail_lint::Finding>,
    overrides: &[LintOverride],
) -> Vec<sail_lint::Finding> {
    if overrides.is_empty() {
        return findings;
    }
    findings
        .into_iter()
        .filter_map(|mut f| {
            // THE FLOOR is non-negotiable: an override naming a floor rule id is
            // ignored, so no `lint_override` (Off/Warn) can drop or downgrade a
            // floor finding. Its Error severity is then guaranteed by
            // `enforce_floor`. This is what makes the floor a language property
            // rather than host-configurable lint wiring.
            if crate::check::policy::is_floor_rule(f.rule) {
                return Some(f);
            }
            for o in overrides {
                if o.rule == f.rule {
                    return match o.severity {
                        OverrideSeverity::Off => None,
                        OverrideSeverity::Warn => {
                            f.severity = sail_lint::Severity::Warn;
                            Some(f)
                        }
                        OverrideSeverity::Error => {
                            f.severity = sail_lint::Severity::Error;
                            Some(f)
                        }
                    };
                }
            }
            Some(f)
        })
        .collect()
}

/// THE FLOOR, as a compiler pass. Promote every finding whose rule is a
/// [`crate::check::policy::FLOOR_RULES`] id to `Error`, unconditionally.
///
/// This runs on every compile regardless of `opts.lint_overrides`, so the
/// styling constructs the floor bans (`sail-no-style-block`,
/// `sail-no-inline-style`, `sail-no-class`, `sail-no-undeclared-stylesheet`) are a
/// hard `Error` in every `.sui` — including on a host that passes no overrides
/// at all (the wasm/playground path that historically skipped the floor). The
/// two natural-`Warn` floor rules (`sail-no-style-block`, `sail-no-inline-style`)
/// gate here without any host promotion; the two natural-`Error` ones pass
/// through unchanged.
///
/// L1 element sources never reach this pass: they compile through
/// `compile_element`, which does not run the lint gate, because they ARE the
/// styling layer (the compiler's own generated `sx` block is likewise not
/// authored source).
pub fn enforce_floor(findings: Vec<sail_lint::Finding>) -> Vec<sail_lint::Finding> {
    findings
        .into_iter()
        .map(|mut f| {
            if crate::check::policy::is_floor_rule(f.rule) {
                f.severity = sail_lint::Severity::Error;
            }
            f
        })
        .collect()
}
