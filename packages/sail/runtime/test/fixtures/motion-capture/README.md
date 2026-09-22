# motion-capture — ground truth

`waapi-animations.json` is a verbatim capture of the live animations on
motion.dev's hero, read out of the page with
`document.getAnimations().map(a => ({ ...a.effect.getTiming(), keyframes:
a.effect.getKeyframes() }))`. It is the reference `timeline()` is measured
against, not an invention of this repo — every offset, value, and
`cubic-bezier(...)` in it came off a shipping page.

The tile entry (`span.home-diag-tile`, 3600 ms, 7 stops) is the acceptance
case: translate settles at `.34` and HOLDS through `.44`, scale pops
`.54 → .64`, everything on one shared `cubic-bezier(0.6, 0, 0.2, 1)`. It is
the shape hand-rolled `Math.min/Math.max` progress math gets subtly wrong,
which is why it is the gauntlet.

Two tests read it:

- `test/timeline-acceptance.test.ts` — recreates the tile with `timeline()`
  and compares against an INDEPENDENT oracle written in the test (its own
  bezier solver, its own segment walk). Runs in `bun run test`.
- `e2e/timeline-tile.spec.ts` — replays the same capture through real WAAPI
  in headless Chromium and compares the browser's own computed transform to
  the one `timeline()` drives through custom properties. Runs under
  `bunx playwright test` from `packages/sail/runtime`.

Do not edit the JSON. If the reference page changes, re-capture it.
