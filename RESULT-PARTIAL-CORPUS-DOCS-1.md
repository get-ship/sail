# RESULT-PARTIAL-CORPUS-DOCS-1

**Date:** 2026-08-08  
**Branch:** `job/partial-corpus-docs-1`  
**Base:** `cd508226` (release-integrate + PARTIAL-1)

## Summary

Finish partial adoption on the proof surface, land one more same-file corpus
win, and teach authors the difference between in-file `<partial>` and Style
composition that was loosely called “partials.”

## SHA

Branch tip on `job/partial-corpus-docs-1` (local only; no push). See `git log -1 --oneline`.

## Delivered

| Item | Change |
|------|--------|
| W1 theme-color-surface | Semantic success/warning/danger/info rows now invoke `color-token-row`. Body stamps `sem={token}` (same value as historical `sem=` on those rows; surfaces/text rows also get `sem` = token). **12/12** token list rows use the partial — no residual hand-duplicated semantic markup. |
| W2 corpus win | `theme-assets-surface.sail`: new `<partial type-role-select>` for Display/Body/Mono font role selects (pure params: token, label, value, disabled, options, onChange). |
| W3 author docs | Templates chapter: **In-file partials** section + compile-checked snippet; disambiguates from Style compose with link to styling. Styling chapter: “Style pieces” wording + pointer to templates. Tutorial `02-styling-is-sx.md` same disambiguation. |
| W4 surfaces | Skill already teaches in-file `<partial>` (control-flow / skill generator path from PARTIAL-1). No prop-registry regen required (no manifest/stdlib change). `site_snippets_compile` green. Both studio surfaces compile under `--mode component`. |

## Acceptance

| Criterion | Status |
|-----------|--------|
| theme-color-surface no residual hand-duplicated semantic rows | pass (all four → partial; `sem={token}` in body) |
| ≥1 additional file uses `<partial>` | pass (`theme-assets-surface`) |
| sail-site (or tutorial) teaches `<partial>` explicitly | pass (templates chapter + snippet; styling/tutorial disambiguate Style “partials”) |
| RESULT-PARTIAL-CORPUS-DOCS-1.md | this file |
| Local commits; no push | yes |

## Files touched

- `apps/studio/src/theme-color-surface/theme-color-surface.sail`
- `apps/studio/src/theme-assets-surface/theme-assets-surface.sail`
- `apps/sail-site/routes/sail/language/templates/page.sail`
- `apps/sail-site/routes/sail/language/templates/snippets.ts`
- `apps/sail-site/routes/sail/language/styling/page.sail`
- `packages/sail/tutorial/src/prose/02-styling-is-sx.md`
- `RESULT-PARTIAL-CORPUS-DOCS-1.md`

## Residuals

1. Brand hero cards in theme-color-surface remain hand-written (three similar cards; not in this job’s residual list — semantic rows only).
2. Specimen chrome traffic-light dots are still copy-pasted across several theme surfaces (cross-file; out of partial scope until shared component).
3. No runtime / compiler changes in this job (corpus + docs only).

## Notes

- Chose `sem={token}` over optional `sem?` because optional attrs left in the
  body do not strip when omitted (expand is textual param sub). Token and
  historical `sem` were always equal on semantic rows.
- No push. No Co-Authored-By.
