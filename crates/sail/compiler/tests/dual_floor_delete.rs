//! SPEC-PLATFORM-NEUTRALITY-1 acceptance / SPIKE S2.3 — delete dual ambient-DOM floors.
//!
//! TypeEnv::Sail (S2.1) + corpus scrub (S2.2) make ambient `document`/`window`/
//! `localStorage` unnameable in app product code. The old regex lint floors that
//! dual-enforced the same law must be deleted (no dual path).
//!
//! Scout: SCOUT-S2-corpus.md §4a delete candidates.
//! Job: job-spec-spike-s2-3-delete-dual-floors.md
//!
//! Contract locked here (implementer must not skip/weaken):
//! 1. Dual-enforcement ambient rule IDs are not **live** (no check wired in
//!    APP_SOURCE_RULES / other live tables). HARDENING forever-pin may keep the
//!    id strings are not registered.
//! 2. `lint_app_source_source` must not emit those rule findings.
//! 3. Keep list remains live: authority, network, imperative-style floors.
//! 4. TypeEnv::Sail still has no DOM lib (type-env remains the law).
//!
//! Hygiene corpus (S2.2) stays green separately:
//! `bun test tests/hygiene/platform-neutrality-corpus.test.ts`

use std::collections::BTreeSet;
use std::path::Path;

use sail_compiler::ambients::{TypeEnv, lib_for_type_env};
use sail_lint::{all_rule_ids, lint_app_source_source};

/// Dual-enforcement ambient-DOM family deleted by S2.3 (SCOUT-S2 §4a).
///
/// Replaced by TypeEnv::Sail / Action observation set. Not security floors.
///
/// **Out of this delete set (intentionally):**
/// - `sail-no-ambient-network` / `sail-controller-no-ambient-authority` — keep
/// - `sail-no-ambient-timers` — partial (type-env alone may not ban timer *calls*)
/// - `sail-action-host-only` — job may keep as action product law
/// - `sail-no-imperative-style` — CSSOM on handed nodes; orthogonal
const DELETED_DUAL_FLOOR_IDS: &[&str] = &[
    "sail-no-ambient-dom",
    "sail-no-ambient-events",
    "sail-no-ambient-storage",
    "sail-no-ambient-nav",
];

/// Floors that must remain live after dual-path deletion (SCOUT-S2 §4b + job).
const KEPT_LIVE_IDS: &[&str] = &[
    "sail-controller-no-ambient-authority",
    "sail-no-ambient-network",
    "sail-no-imperative-style",
];

fn live_rule_ids() -> BTreeSet<&'static str> {
    sail_lint::rules::CONTROLLER_RULES
        .iter()
        .chain(sail_lint::rules::SUI_TEMPLATE_RULES.iter())
        .chain(sail_lint::rules::PACK_SOURCE_RULES.iter())
        .chain(sail_lint::rules::APP_SOURCE_RULES.iter())
        .chain(sail_lint::rules::ACTION_RULES.iter())
        .map(|(id, _)| *id)
        .collect()
}

fn app_source_live_ids() -> BTreeSet<&'static str> {
    sail_lint::rules::APP_SOURCE_RULES
        .iter()
        .map(|(id, _)| *id)
        .collect()
}

fn has_dom_lib(lib: &[&str]) -> bool {
    lib.iter()
        .any(|e| *e == "DOM" || *e == "DOM.Iterable" || e.starts_with("DOM."))
}

// ── 1. Dual floors gone from live registries ───────────────────────────────

#[test]
fn dual_ambient_floors_not_in_app_source_rules() {
    let live = app_source_live_ids();
    let still: Vec<&str> = DELETED_DUAL_FLOOR_IDS
        .iter()
        .copied()
        .filter(|id| live.contains(id))
        .collect();
    assert!(
        still.is_empty(),
        "S2.3: dual ambient-DOM floors must leave APP_SOURCE_RULES \
         (TypeEnv::Sail is the law). Still live: {still:?}\n\
         APP_SOURCE_RULES = {:?}",
        live.iter().copied().collect::<Vec<_>>()
    );
}

#[test]
fn dual_ambient_floors_not_live_in_any_rule_table() {
    let live = live_rule_ids();
    let still: Vec<&str> = DELETED_DUAL_FLOOR_IDS
        .iter()
        .copied()
        .filter(|id| live.contains(id))
        .collect();
    assert!(
        still.is_empty(),
        "S2.3: dual ambient floors must not be wired as live checks in any \
         rule table (CONTROLLER/SUI/PACK/APP/ACTION). Still live: {still:?}"
    );
}

#[test]
fn dual_ambient_floors_absent_from_all_rule_ids() {
    let all: BTreeSet<&str> = all_rule_ids().into_iter().collect();
    let still: Vec<&str> = DELETED_DUAL_FLOOR_IDS
        .iter()
        .copied()
        .filter(|id| all.contains(id))
        .collect();
    assert!(
        still.is_empty(),
        "deleted dual-floor ids must not stay registered: {still:?}"
    );
}

// ── 2. Behavioral: app-source lint must not fire deleted dual floors ───────

#[test]
fn lint_app_source_does_not_emit_deleted_dual_floor_findings() {
    // Classic ambient god-reach that the deleted rules used to catch.
    // After S2.3 these are TypeEnv errors, not lint dual-path findings.
    let path = Path::new("apps/demo/src/compositions/ambient-probe.ts");
    // r## so `#root` in the TS snippet does not close a single-hash raw string.
    let src = r##"
export function probe() {
  const el = document.createElement("div");
  document.querySelector("#root");
  window.addEventListener("popstate", () => {});
  localStorage.getItem("k");
  sessionStorage.setItem("k", "v");
  const h = location.href;
  history.pushState({}, "", "/x");
  return { el, h };
}
"##;
    let findings = lint_app_source_source(path, src);
    let dual_hits: Vec<&str> = findings
        .iter()
        .filter(|f| DELETED_DUAL_FLOOR_IDS.contains(&f.rule))
        .map(|f| f.rule)
        .collect();
    assert!(
        dual_hits.is_empty(),
        "S2.3: lint_app_source_source must not emit deleted dual-floor rule ids \
         (TypeEnv owns ambient ban). Got dual findings: {dual_hits:?}\nall findings: {:?}",
        findings
            .iter()
            .map(|f| format!("{}:{}", f.rule, f.line))
            .collect::<Vec<_>>()
    );
}

// ── 3. Keep list still live ────────────────────────────────────────────────

#[test]
fn kept_security_and_style_floors_remain_live() {
    let live = live_rule_ids();
    let all: BTreeSet<&str> = all_rule_ids().into_iter().collect();
    let missing: Vec<&str> = KEPT_LIVE_IDS
        .iter()
        .copied()
        .filter(|id| !live.contains(id))
        .collect();
    assert!(
        missing.is_empty(),
        "S2.3 must not delete authority/network/imperative-style floors: missing {missing:?}"
    );
    // Network dual-pack member stays on APP_SOURCE; authority on controllers.
    assert!(
        app_source_live_ids().contains("sail-no-ambient-network")
            || live.contains("sail-no-ambient-network"),
        "sail-no-ambient-network must remain a live check"
    );
    assert!(
        live.contains("sail-controller-no-ambient-authority"),
        "sail-controller-no-ambient-authority must remain live"
    );
    assert!(
        live.contains("sail-no-imperative-style"),
        "sail-no-imperative-style must remain live"
    );
    // Job 0: promote the same floor onto the action surface. Do not invent
    // a sibling id (`sail-l1-cssom-smell` or similar).
    let action_ids: BTreeSet<&str> = sail_lint::rules::ACTION_RULES
        .iter()
        .map(|(id, _)| *id)
        .collect();
    assert!(
        action_ids.contains("sail-no-imperative-style"),
        "sail-no-imperative-style must stay live on ACTION_RULES (author + catalog stamps). \
         ACTION_RULES = {action_ids:?}"
    );
    for id in KEPT_LIVE_IDS {
        assert!(
            all.contains(id),
            "kept floor {id} must remain registered in all_rule_ids"
        );
    }
}

// ── 4. TypeEnv::Sail still the law (must stay green while floors delete) ────

#[test]
fn type_env_pure_still_excludes_dom_lib() {
    let lib = lib_for_type_env(TypeEnv::Sail);
    assert!(
        !has_dom_lib(lib),
        "Pure type env must keep document unnameable (no DOM lib). got {lib:?}"
    );
    let action = lib_for_type_env(TypeEnv::Action);
    assert!(
        !has_dom_lib(action),
        "Action type env must keep document unnameable (observation ambient only). got {action:?}"
    );
}
