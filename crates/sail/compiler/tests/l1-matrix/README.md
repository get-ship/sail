# L1 × theme matrix

Owned fixture + probe. **Not** `apps/sail-site`.

- Fixture: `../fixtures/l1-matrix` — generated per-host routes (`bun …/l1-matrix/generate.ts`). Add a tag to fan out.
- Cells: `cells.ts` — catalog × THEME_META fan-out (rest + axis sweeps).
- Probe: `probe.ts` — walks cells, visits each generated route, paints the floor (`EMPTY-RENDER`, `HIDDEN`, `OPACITY-0`, `LOW-CONTRAST`, `TINY`, `INVISIBLE-LINE`) plus layout-diag flags from `layout-diags.json`.

```
sail dev --project crates/sail/compiler/tests/fixtures/l1-matrix --addr 127.0.0.1:7892
SHIP_MISSION_SCRATCH=… bun crates/sail/compiler/tests/l1-matrix/probe.ts
```

Site `/components` remains a product gauge. Do not point crate tests at it.
