# Beauty rubric (versioned)

Committed, versioned like code. Used by the frontier-lane beauty grader
(judge model) and as the operator sample-grade checklist for cheap lane.

Score each dimension 0–2. Total / 10.

| Dimension | 0 | 1 | 2 |
|---|---|---|---|
| Hierarchy | Flat / competing weights | Clear primary + secondary | Strong focal + calm support |
| Spacing rhythm | Irregular / cramped / sparse | Mostly consistent | Token-scale rhythm throughout |
| Alignment | Drift / uneven edges | Mostly grid-aligned | Clean columns/rows, no orphans |
| Density | Overfull or empty | Acceptable | Comfortable for the task density |
| Craft | Broken chrome, overflow, collisions | Minor polish issues | Production-ready finish |

**Lane rules**
- `cheap`: mechanical only + optional operator sample-grades (this rubric).
- `frontier`: judge model scores with this rubric + operator audit sample.

Do not change judge-model choice or rubric weights without an operator decision.
