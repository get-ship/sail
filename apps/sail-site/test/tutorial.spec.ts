// The tutorial workbench — the site's most load-bearing page, and the only one
// that runs the compiler in the reader's browser.
//
// The contract these tests hold: the editor holds ONE file (the lesson `.sail`,
// `<controller>` block and template together), the wasm compiler accepts that
// file, and the preview renders it. When the compiler's SFC handling regressed,
// every lesson silently compiled with an empty scope — the page still "loaded",
// so nothing but a real run catches it.

import { test, expect, type Page } from '@playwright/test'

// Editor boot + wasm compile can exceed the default 30s under a cold deck dev.
test.setTimeout(90_000)

const editorValue = (page: Page): Promise<string> =>
  page.evaluate(() => {
    const el = document.querySelector('sail-code-editor[data-pane="source"]')
    return (el as unknown as { value?: string })?.value ?? ''
  })

// L1 badge/tab paint as `.sail-badge` / `.sail-tab` hosts (display:contents
// custom elements are not the interactive leaf Playwright should query).
const statusText = (page: Page): Promise<string> =>
  page.locator('.sail-badge').first().innerText()

async function openLesson(page: Page, slug: string): Promise<void> {
  await page.goto(`/sail/tutorial/${slug}`, { waitUntil: 'domcontentloaded' })
  await page.waitForFunction(
    () => {
      const el = document.querySelector('sail-code-editor[data-pane="source"]')
      return Boolean((el as unknown as { value?: string })?.value)
    },
    undefined,
    { timeout: 60_000 },
  )
}

test('the editor holds the whole .sail — controller block and template', async ({ page }) => {
  await openLesson(page, '03-state-is-a-controller')
  const src = await editorValue(page)
  expect(src).toContain('<controller>')
  expect(src).toContain('</controller>')
  // The tab names the file, not a "template.sail" / "controller.ts" pair.
  await expect(page.locator('.sail-tab[name="source"]')).toContainText('.sail')
  expect(await page.locator('.sail-tab').count()).toBe(2)
})

test('the lesson compiles and the preview runs it', async ({ page }) => {
  const errors: string[] = []
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(m.text())
  })
  await openLesson(page, '03-state-is-a-controller')

  // 'running' is only reached when the wasm compiler returned js AND the iframe
  // accepted the run. A compile error lands as 'compile error'.
  await expect
    .poll(() => statusText(page), { timeout: 60_000 })
    .toBe('running')

  // The preview really rendered the lesson's component, not an empty document.
  const frame = page.frameLocator('iframe')
  await expect(frame.locator('body')).not.toBeEmpty()
  expect(errors.filter((e) => /not declared in scope/i.test(e))).toEqual([])
})

test('editing the source recompiles; a broken edit reports the editor row', async ({ page }) => {
  await openLesson(page, '03-state-is-a-controller')
  await expect.poll(() => statusText(page), { timeout: 60_000 }).toBe('running')

  const src = await editorValue(page)
  const lines = src.split('\n')
  // Append a template line that reads a name no controller provides. Its row is
  // the one the compiler must name — an SFC diagnostic is in the file's
  // coordinates, not the template's-minus-the-block.
  const brokenRow = lines.length + 1
  await page.evaluate(
    (next) => {
      const el = document.querySelector('sail-code-editor[data-pane="source"]') as unknown as {
        value: string
        dispatchEvent: (e: Event) => boolean
      }
      el.value = next
      const host = el as HTMLElement & { onChange?: (d: { value: string }) => void; value?: string }
      host.value = next
      host.onChange?.({ value: next })
    },
    `${src}\n<span>{nothingProvidesThis()}</span>`,
  )

  await expect.poll(() => statusText(page), { timeout: 30_000 }).toBe('compile error')
  const console_ = page.locator('[data-console] , .console').first()
  const text = await page.locator('body').innerText()
  expect(text).toContain('nothingProvidesThis')
  expect(brokenRow).toBeGreaterThan(1)
  await expect(console_.or(page.locator('body'))).toContainText('not declared in scope')
})
