# @ship/deck-e2e

Browser-e2e harness for the **deck** framework. Drives the real `deck` CLI
standalone — **no shipd, no host adapter** — against a non-trivial `routes/` app
and asserts deck's runtime behavioral conditions in headless Chromium.

This is the harness that proves "our prod is the user's dev": the behaviors here
are exactly what a deck user gets from `deck dev` / `deck build`.

## Run

```bash
cd packages/deck/e2e
bunx playwright test            # headless (CI shape)
bunx playwright test --headed   # watch it in a real browser
bunx playwright test --ui       # Playwright UI mode
```

`playwright.config.ts` boots the dev server via its `webServer` hook
(`cargo run -p deck-cli -- dev --project ./app`), so the first run compiles the
CLI. `@playwright/test` is vendored at the repo root; Chromium must be installed
once (`bunx playwright install chromium`).

## Layout

- `app/` — a non-trivial deck **`routes/`-SPA** app fixture (deck's own dynamic
  routes, not the named-`view` path). `package.json` + `routes/` only; deck
  resolves the `@ship/*` framework internally (no workspace install needed).
- `tests/` — the specs. `dev-loop.spec.ts` covers the standalone SPA loop:
  CSR shell → client mount → client-side `<a>` navigation → counter reactivity.

## What this is for

Runtime behaviors (SSR/CSR contract, mount/hydration, navigation, lazy-load,
state-preserving HMR) are otherwise unverifiable from Rust integration tests
alone — they need a real browser against the real CLI. The first thing this
harness caught: a zero-config standalone app served a data-only entry that
never called `mountRouter`, so nothing mounted (an `inv-deck-standalone` gap the
build-only `cli_smoke` test missed). Verify-then-assert: probe the actual
behavior before pinning it in a spec; never assert aspirational behavior.

### Gotchas

- The dev server streams **live-reload SSE** (`/_deck/dev/hmr`), so Playwright's
  `waitUntil: 'networkidle'` never fires. Use `'load'` + locator waits.
- The dev SPA path is **CSR** (`<meta name="deck-render" content="csr">`).
  Per-route SSR holds for the named-`view` path (see
  `crates/deck/deck-conformance/e2e/counter.sh`) but not the dev `routes/` SPA.
