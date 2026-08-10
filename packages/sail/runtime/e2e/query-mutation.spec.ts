// SPEC-TESTING-1 — query() + mutation() concept e2e (spike S6.1).
//
// Harness choice: packages/sail/runtime Playwright (same suite as
// timeline-tile.spec.ts). Runtime e2e already runs in CI via
// `bun run test:e2e` from this package; deck-conformance e2e is shell +
// playwright-cli host-loop proofs (and its `/_deck/view/*` surface was
// extracted), so language-concept gates belong here — not under deck PE
// mutation.sh (form PE, not sail mutation()).
//
// Surface: implementer owns `e2e/query-mutation-entry.ts`. That entry must
// mount a product UI equivalent to
// crates/deck/deck-conformance/app/routes/query-demo/page.sail into #root,
// driven by real queryKeyed/query + mutation (compiler-emitted or public
// uncompiled APIs). Data port may be in-memory / delayed fetch — never
// the assertion substrate (no mockQueryState as the thing we assert).
//
// ── CONTRACT (implementer must honor; do not weaken this file) ─────────────
// Root:        [data-testid="query-demo"]
// Query state: [data-testid="query-status"][data-kind="loading"|"ok"|"error"]
// Loading UX:  visible text "Loading…" while kind=loading (match query-demo)
// Deal list:   [data-testid="deal-list"] when ok
// Deal rows:   [data-testid="deal-{id}"] text includes `{id} — {stage} — {amount}`
//              Seed open: d1 (open, 4200), d2 (open, 9100)
//              Seed closed-only list: d3 (closed, 1500) and, after mutation, d1
// Stage keys:  role=button name "open" | "closed" (aria-pressed tracks filter)
// Mutation:    role=button name "close d1"
// Mut state:   [data-testid="mutation-status"][data-kind="idle"|"loading"|"ok"|"error"]
//
// Loading must paint long enough for the gate to observe it (≥100ms artificial
// delay on the first open-list fetch is fine). Stage change must re-key /
// refetch so the list identity changes. Successful close-d1 mutation must
// invalidate/refetch so open list no longer shows d1 (and closed list can).
//
//   bunx playwright test e2e/query-mutation.spec.ts   (from packages/sail/runtime)

import { test, expect } from '@playwright/test'
import { execFileSync } from 'node:child_process'
import { existsSync, mkdtempSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const ENTRY = resolve(HERE, 'query-mutation-entry.ts')

/** Build the real runtime surface for the browser. No mocks of query/mutation. */
function buildBundle(): string {
  if (!existsSync(ENTRY)) {
    throw new Error(
      [
        'Missing e2e/query-mutation-entry.ts.',
        'Implementer must add a browser entry that mounts the query/mutation',
        'contract surface into #root (see CONTRACT in query-mutation.spec.ts).',
        'Use real queryKeyed/query + mutation — not a static HTML mock of states.',
      ].join(' '),
    )
  }
  const out = join(mkdtempSync(join(tmpdir(), 'sail-query-mutation-e2e-')), 'bundle.js')
  execFileSync('bun', ['build', ENTRY, '--target=browser', '--outfile', out], {
    stdio: 'pipe',
  })
  return readFileSync(out, 'utf8')
}

const PAGE = `
<!doctype html>
<html lang="en">
  <head><meta charset="utf-8" /><title>query/mutation e2e</title></head>
  <body>
    <div id="root"></div>
  </body>
</html>
`

test.describe('query() + mutation() — public UI lifecycle', () => {
  test('loading → ok → key change → mutation invalidates list', async ({ page }) => {
    const bundle = buildBundle()
    await page.setContent(PAGE)
    await page.addScriptTag({ content: bundle })

    const root = page.getByTestId('query-demo')
    await expect(root, 'entry must mount [data-testid=query-demo] into #root').toBeVisible()

    const queryStatus = page.getByTestId('query-status')
    const mutationStatus = page.getByTestId('mutation-status')
    const dealList = page.getByTestId('deal-list')
    const openBtn = page.getByRole('button', { name: 'open', exact: true })
    const closedBtn = page.getByRole('button', { name: 'closed', exact: true })
    const closeD1 = page.getByRole('button', { name: 'close d1', exact: true })

    // ── 1. loading is observable ───────────────────────────────────────────
    // First paint of the open-stage query must expose loading before ok.
    await expect(queryStatus).toHaveAttribute('data-kind', 'loading', { timeout: 2_000 })
    await expect(page.getByText('Loading…', { exact: true })).toBeVisible()

    // ── 2. settles to ok with open-stage data ──────────────────────────────
    await expect(queryStatus).toHaveAttribute('data-kind', 'ok', { timeout: 10_000 })
    await expect(openBtn).toHaveAttribute('aria-pressed', 'true')
    await expect(dealList).toBeVisible()
    await expect(page.getByTestId('deal-d1')).toContainText('d1 — open — 4200')
    await expect(page.getByTestId('deal-d2')).toContainText('d2 — open — 9100')
    await expect(page.getByTestId('deal-d3')).toHaveCount(0)
    await expect(mutationStatus).toHaveAttribute('data-kind', 'idle')

    // ── 3. key / input change (stage filter) updates list identity ─────────
    await closedBtn.click()
    await expect(closedBtn).toHaveAttribute('aria-pressed', 'true')
    // Re-key must be visible as loading and/or a list that is no longer the open set.
    await expect
      .poll(async () => {
        const kind = await queryStatus.getAttribute('data-kind')
        if (kind === 'loading') return 'loading'
        if (kind !== 'ok') return kind ?? 'missing'
        if (await page.getByTestId('deal-d1').count()) return 'still-has-open-d1'
        if (await page.getByTestId('deal-d2').count()) return 'still-has-open-d2'
        if (await page.getByTestId('deal-d3').count()) return 'closed-ok'
        return 'ok-empty'
      }, { timeout: 10_000 })
      .toMatch(/loading|closed-ok/)

    await expect(queryStatus).toHaveAttribute('data-kind', 'ok', { timeout: 10_000 })
    await expect(page.getByTestId('deal-d3')).toContainText('d3 — closed — 1500')
    await expect(page.getByTestId('deal-d1')).toHaveCount(0)
    await expect(page.getByTestId('deal-d2')).toHaveCount(0)

    // Back to open for the mutation proof.
    await openBtn.click()
    await expect(openBtn).toHaveAttribute('aria-pressed', 'true')
    await expect(queryStatus).toHaveAttribute('data-kind', 'ok', { timeout: 10_000 })
    await expect(page.getByTestId('deal-d1')).toContainText('d1 — open — 4200')

    // ── 4. mutation runs and invalidates / updates the open list ───────────
    await closeD1.click()
    await expect
      .poll(async () => (await mutationStatus.getAttribute('data-kind')) ?? 'missing', {
        timeout: 10_000,
      })
      .toMatch(/loading|ok/)

    await expect(mutationStatus).toHaveAttribute('data-kind', 'ok', { timeout: 10_000 })
    // After invalidate/refetch, open list must drop d1 (it is now closed).
    await expect(page.getByTestId('deal-d1')).toHaveCount(0, { timeout: 10_000 })
    await expect(page.getByTestId('deal-d2')).toContainText('d2 — open — 9100')

    // Closed filter should now include the mutated deal.
    await closedBtn.click()
    await expect(queryStatus).toHaveAttribute('data-kind', 'ok', { timeout: 10_000 })
    await expect(page.getByTestId('deal-d1')).toContainText('d1 — closed — 4200')
    await expect(page.getByTestId('deal-d3')).toContainText('d3 — closed — 1500')
  })
})
