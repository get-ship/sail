# Sail eval RESULTS

> **UNPROVEN harness.** This report is **recorded-lane fixture replay**: graders score committed goldens, not a live agent. A 100% pass rate here does **not** mean agents author good sail. Real evidence requires `--dispatch omp` runs.
> Trajectories live under `eval/runs/` (gitignored). Aggregates only here.

- **Lane:** `cheap`
- **Dispatch:** `recorded`
- **Started:** 2026-07-30T21:49:47.601Z
- **Finished:** 2026-07-30T21:50:07.139Z
- **Pass / Fail / Skip:** 10 / 0 / 0
- **Pass rate:** 100%

## Tasks

| Task | OK | Turns | Grades | Notes |
|---|:---:|---:|---|---|
| asymmetric-editorial-layout | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| card-grid | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| dashboard-shell | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| dense-data-table | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| docs-article-page | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| empty-state-list | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| login-form | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| mobile-first-nav-flow | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| realtime-chat-feed | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |
| settings-page | ✓ | 1 | compile:✓ vocab:100% gates:✓ behavior:skip beauty:skip |  |

## Leg averages

- **Vocab share (mean):** 100.0% (n=10)
- **Compile green:** 10/10
- **Auto-repairs (sum):** 0
- **Turns (mean):** 1.0

## Notes

- Cheap lane grades compile + vocab_share + gates only (pass `--cheap-behavior` for source-text heuristics).
- Frontier adds playwright behavior + beauty (judge/operator). Missing beauty score is incomplete unless `--allow-skip-beauty`.
- Studio vocab low-water mark: **17%**. Default min share: **25%**. Strict mode: missing sail fails (not pass).
