# Operator pairing queue — scaffold_ready

**Updated:** 2026-08-11  
**Branch tip:** `job/release-integrate` (local scaffold batch)

Tags below are **scaffold_ready** / **operator_pending**: contracts, first-class docs knobs, visual `bun run visual <tag>` green. Operator taste / multi-theme approval still open — **not** `done`.

## Queue (B0 + B1 + B2 + B3)

| Tag | Docs host | Contract | Visual | Notes for pairing |
|-----|-----------|----------|--------|-------------------|
| button | `doc-button` | yes | green | Pilot; disabled knob docsSkip |
| badge | `doc-badge` | yes | green | Rest tone path |
| fieldset | `doc-fieldset` | yes | green | Legend nested only |
| chip | `doc-chip` | yes | green | selected / removable |
| alert | `doc-alert` | yes | green | dismissible remount |
| separator | `doc-separator` | yes | green | orientation × variant × tone |
| spinner | `doc-spinner` | yes | green | rest fg/track |
| skeleton | `doc-skeleton` | yes | green | shape × size |
| icon | `doc-icon` | yes | green | no component.icon leaves |
| progress | `doc-progress` | yes | green | omit value = indeterminate |
| meter | `doc-meter` | yes | green | low/high/optimum bands fixed in stage |
| **label** | `doc-label` | yes | green | size × required; for= specimen |
| **field** | `doc-field` | yes | green | invalid only; hint/error slots |
| **radio** | `doc-radio` | yes | green | value × disabled; grouping = radio-group |
| **radio-group** | `doc-radio-group` | yes | green | no component.radio-group.*; layout host |
| **switch** | `doc-switch` | yes | green | size × tone × checked × disabled |
| **slider** | `doc-slider` | yes | green | single thumb; no range invent |
| **input** | `doc-input` | yes | green | type matrix · size xs–lg · field faces |
| **textarea** | `doc-textarea` | yes | green | size · rows · shared field faces |
| **select** | `doc-select` | yes | green | size · multiple · shared field faces |

## Alias residual (not first-class host)

| Tag | Kind | Notes |
|-----|------|-------|
| checkbox | alias | No CE; use `<input type="checkbox">` → input detail |

## Explicitly not in queue yet

- B4–B6 peels (display / overlays / …)
- Product invent work: card ladder, slider range, loading, avatar/breadcrumb variants, motion doctrine
- Playwright knobs→paint

## Checker

```bash
bun apps/sail-site/scripts/check-l1-surface-contract.mjs
# or: cd apps/sail-site && bun run check:l1-surface
# expect 20 contracts after B3 (3 new hosts)
```
