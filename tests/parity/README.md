# PARITY-1 — static model vs browser truth

Two complementary suites:

| Suite | Path | Compares | Default gate |
|-------|------|----------|--------------|
| **Geometry** | `geometry/run.ts` | taffy box tree ↔ Chromium / Firefox / WebKit boxes | **structure** (counts/tags/root width) |
| **Paint** | `paint/run.ts` | takumi `regions[].style` ↔ browser `getComputedStyle` | **structure** (L1 host pairing) |

Style/size deltas are **reported** by default and only fail the process with `--strict`.

> **SSR-RENDER (slice landed):** `SailRender` / `sail-render` project S-tier
> controller `signal(literal)` initials (and simple `<if>` / `<for>`) before
> takumi — see [`crates/sail/render/docs/SSR-RENDER.md`](../../crates/sail/render/docs/SSR-RENDER.md).
> Parity fixtures here stay **static template text only** so structure gates do
> not depend on projection edge cases; controller regression lives in
> `crates/sail/render/tests/ssr_render.rs`.
>
> **SPA path:** temp fixture stages typically fall back to **lowered-html** +
> token/skin CSS (meaningful for L1 paint/geometry structure). Full SPA requires
> framework resolution in the stage; use `--strict-spa` to fail the fallback.
> Closing that gap is product follow-up, not the structure CI gate.

## Prerequisites

- Built `sail` binary (`target/release/sail` or `SAIL_BIN=…`)
- Playwright Chromium (repo `node_modules` + `~/.cache/ms-playwright/…`), or `CHROME_PATH`
- For paint: RENDER-1 region styles from `SailRender/Screenshot` (shipped in sail-render)

## Run

```bash
# Geometry — structure gate (CI default: chromium + firefox + webkit)
bun tests/parity/geometry/run.ts
bun tests/parity/geometry/run.ts --browser chromium

# Geometry — soft report (harness only; structure non-fatal)
bun tests/parity/geometry/run.ts --report-only

# Geometry — also fail on size Δ
bun tests/parity/geometry/run.ts --strict

# Paint — L1 skins / tokens / radius / borders (structure gate)
bun tests/parity/paint/run.ts

# Paint — soft report
bun tests/parity/paint/run.ts --report-only

# Paint — fail on style Δ
bun tests/parity/paint/run.ts --strict

# Single fixture
bun tests/parity/geometry/run.ts --fixture column-stack
bun tests/parity/paint/run.ts --fixture l1-chrome-row

# Both halves (same exit rules as CI)
bun tests/parity/run.ts
bun tests/parity/run.ts --report-only
```

## Exit codes

### Geometry (`geometry/run.ts`)

| Code | Meaning |
|------|---------|
| `0` | Harness ok (taffy + browser measured). With default mode: every fixture `structure_ok`. With `--report-only`: harness ok only. With `--strict`: also `size_fail === 0`. |
| `1` | Missing binary / LayoutCheck fail / browser skip-or-fail / structure fail (default) / size fail (`--strict`) |

Flags:

- `--report-only` — structure & size only in `out/summary.json` (soft CI probe)
- `--strict` — size divergences fatal
- `--strict-spa` — refuse lowered-html fallback
- `--taffy-only` — skip browser
- `--fixture <name>` — filter

### Paint (`paint/run.ts`)

| Code | Meaning |
|------|---------|
| `0` | Harness ok (screenshot + browser styles). Default: every fixture L1 structure_ok. `--report-only`: harness only. `--strict`: also `style_fail === 0`. |
| `1` | Screenshot/browser failure / structure fail (default) / style fail (`--strict`) |

Flags:

- `--report-only` / `--strict` / `--strict-spa` / `--takumi-only` / `--fixture <name>`

**Structure (default gate)** is stronger than tag pairing alone:

1. Every takumi L1 host pairs to a browser host (`data-sail-host` / tag)
2. At least one paired host has non-empty takumi `regions[].style`
3. At least `PARITY_PAINT_MIN_PROPS` (default **1**) style properties actually compared

Style Δ remains non-fatal unless `--strict`. Empty region styles or zero prop
compares fail **structure** (and therefore default CI).

Compared properties (within tolerance):

- `backgroundColor`, `color`, `borderColor` — per-channel max Δ ≤ `PARITY_PAINT_COLOR_TOL` (default 28)
- `borderRadius`, `borderWidth` — px Δ ≤ `PARITY_PAINT_PX_TOL` (default 2)
- `opacity` — when takumi exports it (Δ ≤ 0.05)

**Not compared (API vs suite surface):** none of the other RegionStyle slots are
held out intentionally beyond the above; `borderWidth` is often absent on solid
buttons (radius/color still compared). Full SPA runtime path is separate (`--strict-spa`
fails if only lowered-html is available).

L1 hosts: button, card, field, input, switch, badge, navbar, sidebar, dialog, table.

### Chrome resolution

Geometry engines: `tests/parity/browsers.ts` (Playwright `chromium` / `firefox` / `webkit`).
Paint still uses `tests/parity/chrome.ts` for the Chromium binary:

1. `CHROME_PATH` if set and exists  
2. `playwright` / `playwright-core` `chromium.executablePath()`  
3. Newest chrome under `~/.cache/ms-playwright`  
4. Pinned `chromium-1228` path (local fallback)

### Umbrella (`run.mjs`)

Runs geometry then paint. Exit `1` if either half fails under the same flags.

## Artifacts

- `geometry/out/summary.json` + per-fixture `*.report.json`
- `paint/out/summary.json` + per-fixture `*.report.json`

(`out/` is local; not required in git)

## CI

Workflow: `.github/workflows/parity.yml`

- **geometry-structure** job: `bun tests/parity/geometry/run.ts` (structure gate)
- **paint-structure** job: `bun tests/parity/paint/run.ts` (structure gate)
- Optional soft probe can use `--report-only` on a non-blocking check

Local equivalent:

```bash
cargo build -p deck-cli --release
SAIL_BIN=target/release/sail bun tests/parity/run.ts
```
