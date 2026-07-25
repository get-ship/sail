// Editor-LSP smoke — drive the /sail/tutorial/01-a-page-is-markup lesson, type a
// syntax-broken .sail fragment into the template editor, and assert that the
// browser-side LSP wires through to live diagnostic gutters in the
// CodeMirror editor (the whole reason we ported the full
// ship-code-editor-with-LSP impl). Pre-LSP behavior: lint gutter stays
// empty. Post-LSP behavior: at least one `.cm-diagnostic` (lint-gutter
// marker) and one `.cm-lintRange` (underline) show up.

import { test, expect } from '@playwright/test'

test('LSP diagnostics populate the editor gutter for a broken .sail', async ({ page }) => {
  // Drop known benign noise; print everything else so the LSP
  // dynamic-import path is observable from the test output.
  const log: string[] = []
  page.on('console', (msg) => {
    const text = msg.text()
    if (msg.type() === 'error' && text.includes('view-tokens.css')) return
    log.push(`[${msg.type()}] ${text.slice(0, 220)}`)
  })
  page.on('pageerror', (err) => log.push(`[pageerror] ${err.message}`))

  await page.goto('/sail/tutorial/01-a-page-is-markup', { waitUntil: 'domcontentloaded' })
  await page.waitForSelector('iframe[data-testid="tut-preview"], iframe[title="Tutorial preview"]', {
    state: 'attached',
    timeout: 60_000,
  })
  await page.waitForFunction(
    () => document.querySelectorAll('sail-code-editor').length > 0,
    undefined,
    { timeout: 60_000 },
  )

  // Wait for the CodeMirror editor inside the source pane to actually mount.
  // Dual-pane (template/controller) was retired — one source editor holds the
  // whole `.sail` file.
  await page.waitForFunction(
    () => {
      const src = document.querySelector('sail-code-editor[data-pane="source"]') as HTMLElement | null
      return !!src?.shadowRoot?.querySelector('.cm-editor') ||
        !!src?.querySelector?.('.cm-editor')
    },
    undefined,
    { timeout: 60_000 },
  )

  // Settle: allow the LSP dynamic-import to complete (Strategy A fetches
  // the wasm). The bundle has a default arg = '/sail_compiler_bg.wasm' so
  // even the first ensureLsp() succeeds.
  await page.waitForTimeout(2000)

  // Probe: confirm the imperative API surface is on the host (setDiagnostics
  // defined by the controller) and the language is 'sail' so the LSP effect
  // gates-open.
  const before = await page.evaluate(() => {
    const src = document.querySelector('sail-code-editor[data-pane="source"]') as (HTMLElement & { setDiagnostics?: unknown; setCompletions?: unknown; value?: string }) | null
    return {
      hasSetDiag: typeof src?.setDiagnostics === 'function',
      hasSetComp: typeof src?.setCompletions === 'function',
      valueLen: (src?.value ?? '').length,
      lang: src?.getAttribute('language'),
    }
  })

  // Type a deliberately broken-sail fragment by setting value imperatively.
  // `host.value = v` calls the controller's imperative setter, which writes
  // to valSig; the controller's `effect(() => valSig())` then mirrors the
  // change into the CodeMirror view. The LSP effect also runs on valSig
  // change — 350ms debounce, then `lspDiagnose(text, wasmUrl())` is awaited
  // and the resulting SuiDiagnostic[] pushed into the lint gutter.
  await page.evaluate(async () => {
    const src = document.querySelector('sail-code-editor[data-pane="source"]') as HTMLElement & { value: string } | null
    if (!src) throw new Error('source editor not mounted')
    const broken = '<div>unclosed\n<another nope=\n  <sail-counter'
    src.value = broken
    src.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
  })

  // Wait for the 350ms LSP debounce + wasm init round-trip + diagnostic
  // application + DOM render.
  await page.waitForTimeout(8000)

  const probe = await page.evaluate(() => {
    const src = document.querySelector('sail-code-editor[data-pane="source"]') as Element | null
    // Light DOM (display:contents host) — cm-editor is a descendant, not shadow.
    const cm = (src?.shadowRoot?.querySelector('.cm-editor') ??
      src?.querySelector?.('.cm-editor')) as HTMLElement | null
    const diagnostics = cm?.querySelectorAll('.cm-diagnostic').length ?? 0
    const lintRanges = cm?.querySelectorAll('.cm-lintRange').length ?? 0
    const cmAnyLint = cm?.querySelectorAll('[class*="cm-lint"]').length ?? 0
    const cmAnyDiag = cm?.querySelectorAll('[class*="cm-diagnostic"]').length ?? 0
    const allClasses = Array.from(cm?.querySelectorAll('*') ?? []).slice(0, 30).map((n) => (n as HTMLElement).className).filter(Boolean)
    return { diagnostics, lintRanges, cmAnyLint, cmAnyDiag, allClasses }
  })

  // eslint-disable-next-line no-console
  console.log('[before]', JSON.stringify(before))
  // eslint-disable-next-line no-console
  console.log('[probe ]', JSON.stringify(probe))
  // eslint-disable-next-line no-console
  console.log('[console]\n' + log.slice(-15).join('\n'))

  const count = probe.diagnostics + probe.lintRanges + probe.cmAnyDiag + probe.cmAnyLint
  expect(count, `expected at least one diagnostic marker. before=${JSON.stringify(before)} probe=${JSON.stringify(probe)}`).toBeGreaterThan(0)
})
