---
group: Views
order: 9
title: E2E test suite
description: Playwright-based end-to-end coverage for SUI views. How to run, what it covers, and how to interpret failures.
audience: internal
---

# E2E test suite

The `tests/e2e/` directory contains a Playwright-based end-to-end regression net. Seven spec files cover the SUI view surface: `agents`, `counter`, `git`, `git-live`, `patch-elements`, `shell-rail`, and `studio`. These tests exercise real-browser behavior — SSR correctness, client-side `attach()`, signal reactivity, SSE-driven boundary swaps, and rail navigation — things jsdom-based loop tests cannot catch.

## Running the suite

Requires a running daemon. Start one with `just dev` then:

```sh
# Install Playwright browsers (once)
npx playwright install chromium

# Run the whole suite
just qa-e2e

# Or via bun directly
bun run test:e2e

# Run one spec
bun run test:e2e -- tests/e2e/counter.spec.ts

# Show the HTML report after a run
npx playwright show-report tests/e2e/report
```

The daemon URL is auto-detected from `~/.ship/machine/daemons/<hash>/port` (devx-04 per-worktree daemon convention). Override with `SHIPD_URL=http://localhost:9000 just qa-e2e`.

## What each spec covers

### `counter.spec.ts` — tier-1, no controller

| Test | What it gates |
|------|---------------|
| SSR returns 200 HTML | Daemon serves the view |
| Contains wrapper + data-test selectors | SSR walker emits correct structure |
| Buttons use `on:click="bN"` markers | Compiler outputs author-syntax markers, not Datastar `data-*` |
| `__set_init__` seeds count at 0 | Island serialization is correct |
| PE: count inline in comment markers | SSR value readable without JS |
| Attach: count shows 0 after load | `attach()` completes, island parsed |
| Attach: +3 → 3, - → 2, reset → 0 | Signal mutations update DOM |
| Attach: decrement below zero | No artificial floor — pure signal |

### `agents.spec.ts` — tier-2, controller + live data

| Test | What it gates |
|------|---------------|
| SSR returns 200 HTML | Daemon serves the view |
| Loading branch rendered server-side | `{#while-loading}` block compiles to `<sui-while-loading key>` boundary; non-loading state hidden. |
| `__set_init__` seeds `{kind: "loading"}` | Data envelope initialized correctly |
| Page loads without JS errors | Boot script + attach pipeline healthy |
| SSE resolves → grid or empty state | Full SSE → boundary-swap chain works |
| Resolved state is grid XOR empty | Exactly one branch visible post-swap |

### `git.spec.ts` — tier-2, complex controller

| Test | What it gates |
|------|---------------|
| SSR returns 200 HTML | Daemon serves the view |
| Loading branch rendered | `{#while-loading}` block compiles to `<sui-while-loading key>` boundary correctly |
| View wrapper and view name | Shell template correct |
| `__set_init__` seeds `tab: "changes"` | State declaration serialized |
| Page loads without JS errors | No attach errors |
| SSE resolves → git header visible | SSE → swap complete |
| Tab switching Changes → History → Branches | Signal mutation + DOM update |

## Interpreting failures

**SSR tests fail**: The daemon isn't running or the view fails to compile. Check `just logs`.

**Attach tests fail with timeout on `__suiDispose`**: The boot script threw before `attach()` completed. Open the browser with `npx playwright test --headed` to see console errors.

**SSE resolution tests fail (agents/git stuck on loading)**: The `/_deck/data` SSE channel opened but the boundary swap did not complete. This indicates a bug in the control-flow swap pipeline (`/_deck/render-fragment/<view>`). The failure is intentional — surfacing it is the point of the suite.

**Tab switching fails**: The `on:click` signal binding didn't wire up, or the signal update didn't re-render the panel. Check the `sui-dom` attach wire step.

## File layout

```
tests/e2e/
  _helpers.ts            shared utilities (getDaemonUrl, startDaemon, screenshot, waitForAttach)
  playwright.config.ts   chromium-only, 30s timeout, HTML reporter
  agents.spec.ts         SSR + attach + SSE resolution
  counter.spec.ts        SSR + PE + attach interactions
  git.spec.ts            SSR + attach + tab switching
  git-live.spec.ts       live SSE patch flow
  patch-elements.spec.ts patch-elements channel
  shell-rail.spec.ts    shell rail navigation
  studio.spec.ts         UI Studio catalog
  visual/                visual-regression specs
  screenshots/           debug captures (gitignored)
  report/                HTML report (gitignored)
```
