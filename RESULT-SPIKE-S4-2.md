# RESULT — SPIKE S4.2 — layout kinds corpus canary

**Status:** GREEN  
**Branch:** `job/spike-s4-2-canary`  
**SHA:** `cae37ee974b59642f74c06c977712dc80c8bc8e5`  
**Red parent:** `7380322158c4a0124896b2ca919a2841aa33c23c`  
**Spec:** SPEC-LAYOUT-DIAG-1 §3 rollout  
**Date:** 2026-08-11

## Deliverable

Canary harness runs experimental layout kinds (`unintended-overflow`,
`clipped-text`) against a labeled fixture corpus under
`CheckOptions::experimental()`, records machine-readable counts + sample
findings + FP/TP verdicts, and **does not promote** kinds to default severity
(`promotion.experimental_only: true`).

| Surface | Contract |
|---------|----------|
| `sail_layout::canary::run` | experimental scan over fixtures (+ optional `*.html` corpus roots) |
| `CanaryCounts` | `files_scanned`, `unintended_overflow`, `clipped_text` |
| `CanarySample` | path + kind + message |
| `FpVerdict` | path + kind + judgment + note |
| `PromotionStatus` | `experimental_only` + criteria prose |
| `report_to_json` | machine-readable RESULT JSON |

## Canary table (gate fixtures)

Scanned under experimental @ viewport 400×300.

### Counts

| Metric | Value |
|--------|------:|
| `files_scanned` | 3 |
| `unintended_overflow` | 1 |
| `clipped_text` | 1 |
| experimental findings (sum) | 2 |

### Sample findings

| Path | Kind | Judgment | Note |
|------|------|----------|------|
| `fixtures/overflow-no-optin.html` | `unintended-overflow` | true-positive | known vehicle: fixed-height column, child taller + shrink-0, no overflow opt-in — intentional TP under experimental |
| `fixtures/clipped-text-no-truncate.html` | `clipped-text` | true-positive | known vehicle: narrow box + long unbreakable text with overflow-x hidden and no truncate — intentional TP under experimental |
| `fixtures/clean-box.html` | — | — | control: no experimental findings |

### Sample messages (geometry facts, not text-golden pins)

| Path | Message (abbrev) |
|------|------------------|
| overflow-no-optin | child 160px main overflows parent 80px main without overflow opt-in |
| clipped-text-no-truncate | text ~197px in 64px box; clipped without `truncate()` |

### Promotion

| Field | Value |
|-------|-------|
| `experimental_only` | **true** (no silent promote) |
| criteria | Promote to default Error only when corpus canary FP rate ≈ 0 on intentional TP vehicles **and** a reviewed sample of real corpus findings has no material false positives. Until then keep experimental-only (`strict` / `layoutExperimental`). S4.2 records; it does not promote. |

**Default check still skips experimental kinds** (regression guard green).

## Implementation

| File | Change |
|------|--------|
| `crates/sail/layout/src/canary.rs` | real `run`: experimental scan, counts, samples, FP table, promotion |

- Fixture labels scanned via `check_preprocessed_with(..., CheckOptions::experimental())`.
- Optional `corpus_roots`: best-effort recursive `*.html` walk (skips target/node_modules).
- Known vehicles → `TruePositive`; other experimental emissions → `Unclear` pending human FP review.
- `report_to_json` unchanged (serde pretty).

## TEST_CMD

```bash
RUSTC_WRAPPER= cargo test -p sail-layout --test layout_canary
```

## GREEN_PROOF

```
running 9 tests
test canary_counts_structure_is_consistent ... ok
test canary_harness_scans_fixture_corpus_files ... ok
test canary_public_types_are_constructible_for_implementer_guidance ... ok
test canary_report_json_is_machine_readable_result_shape ... ok
test canary_fp_verdicts_cover_samples ... ok
test canary_detects_clipped_text_on_known_fixture ... ok
test canary_detects_unintended_overflow_on_known_fixture ... ok
test no_silent_promote_default_check_still_skips_experimental_kinds ... ok
test canary_documents_promotion_criteria_and_keeps_experimental_only ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Gate tests unchanged

```
# layout_canary.rs — FROZEN (test-writer owns; implementer did not modify)
git diff 73803221 -- crates/sail/layout/tests/layout_canary.rs  → empty
md5sum crates/sail/layout/tests/layout_canary.rs
2235c250ff57e3f9e547473073feae73  crates/sail/layout/tests/layout_canary.rs
```

## FILES_CHANGED

| File | Change |
|------|--------|
| `crates/sail/layout/src/canary.rs` | implement canary harness `run` |
| `RESULT-SPIKE-S4-2.md` | this RESULT + canary table |

## Out of scope (honored)

- Promoting kinds to default Error
- Text goldens of diagnostic messages
- Full FP judgment quality review of real sail-site / studio corpora (structure + labeled fixtures only; optional corpus_roots walk is best-effort)

No dual-path shims. No `#[ignore]`. No test weakening. No silent promote.
