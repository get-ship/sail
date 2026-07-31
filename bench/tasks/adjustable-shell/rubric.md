# adjustable-shell

**System stressed:** layout + `--*` geometry channel

## Mechanical

- Compiles clean (`sail check` zero errors and warnings)
- Vocab share ≥ 0.25 (L1-first)
- No layout gate failures (overflow/clip/collapse findings)

## Scope criteria (beauty / human)

| Criterion | 0 | 1 | 2 |
|---|---|---|---|
| Shell structure | Missing sidebar or main | Present but not fixed/partitioned | Fixed sidebar + main + dual panels |
| Geometry intent | Hardcoded only, no channel | Partial custom-prop / resize affordance | Clear geometry channel for panel sizes |
| Mobile collapse | No mobile story | Partial / cosmetic | Sidebar collapses with menu control |
| Hierarchy & spacing | Flat / cramped | Acceptable | Clear chrome vs content rhythm |

Beauty is human-graded from the one-pager screenshot. Until RENDER-1 lands,
L1 paint may be incomplete — flag beauty as suspect.
