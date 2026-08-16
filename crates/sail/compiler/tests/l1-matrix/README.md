# L1 × theme matrix

Owned fixture + probe. **Not** `apps/sail-site`.

- Fixture: `../fixtures/l1-matrix` — rest-state L1 strip. Add a tag here to fan out.
- Themes: generated `THEME_META` (every catalog id × supported modes).
- Probe: `probe.ts` — `[data-sail-host]` painted-box / contrast / hidden.

```
sail dev --project crates/sail/compiler/tests/fixtures/l1-matrix --addr 127.0.0.1:7892
SHIP_MISSION_SCRATCH=… bun crates/sail/compiler/tests/l1-matrix/probe.ts
```

Site `/components` remains a product gauge. Do not point crate tests at it.
