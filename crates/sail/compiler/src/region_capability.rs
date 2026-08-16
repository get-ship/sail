//! Region / expression capability matrix — CHANNELS-1 WI10 (S6).
//!
//! Documents **what is legal** in each `.sail` region and **what enforces it**.
//! This table is the SSOT for the `/sail/reference/regions` docs page (emitted
//! into `language-surface.gen.ts` via `sail-compiler prop-registry`).
//!
//! Status column:
//! - `enforced` — lint or compiler hard/soft gate is live
//! - `partial` — related gate exists; freeze prose is stricter than today
//! - `gap` — freeze law not yet implemented (listed honestly; do not claim it)
//! - `policy` — policy-layer warning (default on), not a hard error
//!
//! Freezing rule (SAIL-SYNTAX-FREEZE §1): a rule without an enforcement point
//! is a bug against the freeze. Rows marked `gap` are residual work, not silent
//! green.

use serde::Serialize;

/// One cell of the region/expression capability matrix.
#[derive(Debug, Clone, Serialize)]
pub struct RegionCapability {
    /// `scope` | `controller` | `template` | `chain`
    pub region: &'static str,
    /// Short capability name (matrix row label).
    pub capability: &'static str,
    /// `yes` | `no` | `subset` — is the construct legal in this region?
    pub legal: &'static str,
    /// `enforced` | `partial` | `gap` | `policy`
    pub status: &'static str,
    /// Enforcement point (rule id, module path, or explicit gap note).
    pub enforcement: &'static str,
    /// One-line note.
    pub notes: &'static str,
}

/// Thin-controller LOC threshold (must match `sail_lint::rules::controller_size::THRESHOLD`).
pub const CONTROLLER_SIZE_THRESHOLD: usize = 50;

/// The region/expression capability matrix (documentation order).
pub const REGION_CAPABILITY_MATRIX: &[RegionCapability] = &[
    // ── scope ──────────────────────────────────────────────────────────
    RegionCapability {
        region: "scope",
        capability: "type / export type aliases",
        legal: "yes",
        status: "enforced",
        enforcement: "parse::scope_block + scope_manifest::SCOPE_GRAMMAR",
        notes: "Static surface only — aliases hoist into the compiled module",
    },
    RegionCapability {
        region: "scope",
        capability: "props: field declarations",
        legal: "yes",
        status: "enforced",
        enforcement: "parse::scope_block + scope_manifest::SCOPE_GRAMMAR",
        notes: "Literals + annotations; identity types pass through",
    },
    RegionCapability {
        region: "scope",
        capability: "Style declarations (style name = {…})",
        legal: "yes",
        status: "enforced",
        enforcement: "parse::scope_block (StyleEnv) + spinnaker lower",
        notes: "Channel values as compile-time Style only; prefer style name =",
    },
    RegionCapability {
        region: "scope",
        capability: "signals / effects / DOM / runtime computation",
        legal: "no",
        status: "enforced",
        enforcement: "parse::scope_block (closed member set; let/var/non-Style const refused)",
        notes: "If it computes, it belongs in <controller>",
    },
    RegionCapability {
        region: "scope",
        capability: "static imports (types + channel values ONLY)",
        legal: "subset",
        status: "enforced",
        enforcement: "parse::scope_block (type imports + Style-value from relative .sail; refuse package/side-effect/default/namespace)",
        notes: "Type-only and Style-value imports only; runtime behavior imports belong in <controller>",
    },
    // ── controller ─────────────────────────────────────────────────────
    RegionCapability {
        region: "controller",
        capability: "full TypeScript (signals, effects, data, actions wiring)",
        legal: "yes",
        status: "enforced",
        enforcement: "tsc + synthetic emit; runtime ambient inject",
        notes: "Controller is the behavior home",
    },
    RegionCapability {
        region: "controller",
        capability: "DOM / raw HTML writes",
        legal: "no",
        status: "enforced",
        enforcement: "sail-controller-purity + sail-no-imperative-style + ambient DOM bans",
        notes: "DOM-free: markup stays in the template",
    },
    RegionCapability {
        region: "controller",
        capability: "thin-controller size (~50 LOC)",
        legal: "subset",
        status: "policy",
        enforcement: "sail-controller-size (Warn ≥ 50 non-empty non-comment setup lines; default on)",
        notes: "Orchestration not implementation; silence via policy.lint.severity if needed",
    },
    RegionCapability {
        region: "controller",
        capability: "imports only from table homes + `@ship/*`",
        legal: "subset",
        status: "enforced",
        enforcement: "sail-controller-import-convention (Error) + ACTIONS-1 floor on src/actions/**",
        notes: "L3 value imports: @ship/*, table @/ aliases (components/compositions/stores/actions/themes), relative .sail, relative under src/actions|compositions|components or themes/. Pack envelope (package.json customElements): relative modules under pack root + declared dependencies/peerDependencies. Type-only free",
    },
    RegionCapability {
        region: "controller",
        capability: "ambient network / eval / storage authority",
        legal: "no",
        status: "enforced",
        enforcement: "sail-controller-no-ambient-authority (+ network/unconfined policy)",
        notes: "Data through ship SDK; SES-compatible confinement",
    },
    // ── template ───────────────────────────────────────────────────────
    RegionCapability {
        region: "template",
        capability: "expression subset (reads, ternaries, pure method allowlist)",
        legal: "subset",
        status: "enforced",
        enforcement: "check::expr_subset",
        notes: "No assignments, no new/await, no statements; method allowlist for pure helpers",
    },
    RegionCapability {
        region: "template",
        capability: "control-flow blocks (if/for/match/when)",
        legal: "yes",
        status: "enforced",
        enforcement: "block_manifest + parse::template::blocks",
        notes: "Exactly four block kinds; arms in-brace",
    },
    RegionCapability {
        region: "template",
        capability: "multi-statement event handlers without a controller",
        legal: "no",
        status: "enforced",
        enforcement: "check::handler_complexity",
        notes: "Tier-2 work moves into <controller>",
    },
    // ── chain (Style product) ──────────────────────────────────────────
    RegionCapability {
        region: "chain",
        capability: "closed Style grammar style({ pad(4).bg(surface), when:, animate: })",
        legal: "yes",
        status: "enforced",
        enforcement: "spinnaker parse/lower + sail-no-sx / sail-no-inline-style floors",
        notes: "One styling form; sx= and raw style= refused",
    },
    RegionCapability {
        region: "chain",
        capability: "brace splices for genuine expressions",
        legal: "yes",
        status: "enforced",
        enforcement: "spinnaker parse (brace splice) + channel scale checks",
        notes: "box.w({ pct() }) and token-scale values; off-scale is error",
    },
    RegionCapability {
        region: "chain",
        capability: "open CSS / arbitrary property bags",
        legal: "no",
        status: "enforced",
        enforcement: "spinnaker clash/scale + sail-css-token / css_literals",
        notes: "Closed vocabulary from Style IR; not a CSS escape hatch",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_is_non_empty_and_covers_four_regions() {
        assert!(REGION_CAPABILITY_MATRIX.len() >= 12);
        let mut regions: Vec<&str> = REGION_CAPABILITY_MATRIX.iter().map(|r| r.region).collect();
        regions.sort_unstable();
        regions.dedup();
        assert_eq!(
            regions,
            vec!["chain", "controller", "scope", "template"],
            "matrix must cover all four freeze regions"
        );
    }

    #[test]
    fn rows_have_known_status_and_legal() {
        for row in REGION_CAPABILITY_MATRIX {
            assert!(
                matches!(row.legal, "yes" | "no" | "subset"),
                "bad legal on {}:{}",
                row.region,
                row.capability
            );
            assert!(
                matches!(row.status, "enforced" | "partial" | "gap" | "policy"),
                "bad status on {}:{}",
                row.region,
                row.capability
            );
            assert!(!row.enforcement.is_empty());
            assert!(!row.notes.is_empty());
        }
    }

    #[test]
    fn capability_labels_unique_per_region() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for row in REGION_CAPABILITY_MATRIX {
            assert!(
                seen.insert((row.region, row.capability)),
                "duplicate matrix row: {} / {}",
                row.region,
                row.capability
            );
        }
    }

    #[test]
    fn controller_size_threshold_matches_lint() {
        assert_eq!(
            CONTROLLER_SIZE_THRESHOLD,
            sail_lint::rules::controller_size::THRESHOLD
        );
    }

    #[test]
    fn thin_controller_row_cites_size_rule() {
        let row = REGION_CAPABILITY_MATRIX
            .iter()
            .find(|r| r.capability.contains("thin-controller"))
            .expect("thin-controller row");
        assert_eq!(row.status, "policy");
        assert!(row.enforcement.contains("sail-controller-size"));
        assert!(row.enforcement.contains("50"));
    }
}
