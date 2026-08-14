//! SPEC-HARDENING-1 WI6 — diagnostic / rule ID drift pin.
//!
//! Rule IDs are forever (freeze §15). Messages may improve; IDs must never
//! be removed or renamed without a deliberate pin update. Adding a rule is
//! fine once its ID is appended to [`PINNED_RULE_IDS`].

/// Sorted unique rule IDs registered in `sail_lint` at the time of HARDENING-1.
///
/// - **Add** a new rule → append its id here (sorted) in the same PR.
/// - **Remove or rename** a rule → this test fails until the pin is updated
///   deliberately (IDs are forever; renames are freeze-breaking).
const PINNED_RULE_IDS: &[&str] = &[
    "sail-action-host-only",
    "sail-action-no-network",
    "sail-action-no-state",
    "sail-action-no-style",
    "sail-action-signature",
    "sail-animate-token-only",
    "sail-attr-bool-prop",
    "sail-attr-presence",
    "sail-clipped-text",
    "sail-controller-context",
    "sail-controller-empty",
    "sail-controller-import-convention",
    "sail-controller-matchmedia-layout",
    "sail-controller-navigation",
    "sail-controller-no-ambient-authority",
    "sail-controller-no-load-state",
    "sail-controller-purity",
    "sail-controller-redundant-ambient-import",
    "sail-controller-set-value",
    "sail-controller-size",
    "sail-controller-state-cast",
    "sail-css-content-text",
    "sail-css-literal",
    "sail-css-literals",
    "sail-css-token",
    "sail-dimension-literal",
    "sail-directory-jurisdiction",
    "sail-display-on-layout",
    "sail-event-name",
    "sail-font-literal",
    "sail-for-key",
    "sail-framework-import-teaching",
    "sail-height-chain",
    "sail-layout",
    "sail-layout-primitive",
    "sail-no-ambient-network",
    "sail-no-ambient-timers",
    "sail-no-class",
    "sail-no-component-token",
    "sail-no-data-attr",
    "sail-no-free-aria",
    "sail-no-imperative-style",
    "sail-no-inline-style",
    "sail-no-node-import",
    "sail-no-raw-container",
    "sail-no-script",
    "sail-no-style-block",
    "sail-no-sx",
    "sail-no-undeclared-stylesheet",
    "sail-no-variants",
    "sail-participation-style",
    "sail-prefer-primitive",
    "sail-prefer-query",
    "sail-primary-for-status",
    "sail-prop-value",
    "sail-route-inline-style",
    "sail-space-literal",
    "sail-state-set-call",
    "spinnaker",
    "spinnaker-value-enumeration",
    "sail-sub-min-tap-target",
    "sail-surface-not-card",
    "sail-unintended-overflow",
    "sail-virtualizer",
];

#[test]
fn rule_ids_are_forever_pin() {
    let mut live: Vec<&str> = sail_lint::all_rule_ids();
    live.sort_unstable();
    live.dedup();

    let pinned: Vec<&str> = PINNED_RULE_IDS.to_vec();
    assert!(
        pinned.windows(2).all(|w| w[0] < w[1]),
        "PINNED_RULE_IDS must be strictly sorted unique"
    );

    // Removing or renaming an ID from the registry fails here.
    let live_set: std::collections::BTreeSet<&str> = live.iter().copied().collect();
    let pinned_set: std::collections::BTreeSet<&str> = pinned.iter().copied().collect();

    let removed: Vec<&str> = pinned_set.difference(&live_set).copied().collect();
    assert!(
        removed.is_empty(),
        "rule ID removed or renamed (IDs are forever — restore or supersede deliberately): {removed:?}"
    );

    // Adding is fine only when the pin is updated in the same change.
    let added: Vec<&str> = live_set.difference(&pinned_set).copied().collect();
    assert!(
        added.is_empty(),
        "new rule ID(s) not yet pinned — append to PINNED_RULE_IDS (sorted): {added:?}"
    );

    assert_eq!(
        live, pinned,
        "live all_rule_ids() unique-sorted must equal PINNED_RULE_IDS"
    );
}
