// Motion exit-hold + preset-validity smoke — drives /motion's live
// "Remove the panel" demo and asserts the runtime actually holds the node
// in the DOM through its exit animation instead of removing it
// immediately, and that zero "Invalid keyframe" WAAPI warnings are logged
// anywhere on the page (the rotate-preset bug). /showcase is a broader
// animate-heavy surface checked for zero console errors.

import { test, expect } from '@playwright/test'

test('motion page: remove-panel demo holds the card through its exit animation, zero invalid-keyframe warnings', async ({ page }) => {
  const consoleMessages: string[] = []
  page.on('console', (msg) => consoleMessages.push(`[${msg.type()}] ${msg.text()}`))
  page.on('pageerror', (err) => consoleMessages.push(`[pageerror] ${err.message}`))

  await page.goto('/motion', { waitUntil: 'domcontentloaded' })

  const card = page.locator('text=I animate in, and I animate out.')
  await expect(card).toBeVisible()

  const button = page.getByRole('button', { name: /remove the panel/i })
  await button.click()

  // Immediately after the click the node must still be in the DOM —
  // this is the exact contract the page's own copy promises ("the runtime
  // holds me in the DOM until my exit finishes"). If this fails, the
  // hold regressed back to synchronous removal.
  await expect(card).toBeAttached()

  // It must actually leave once the exit animation completes — bounded by
  // the runtime's own timeout guard, so this must not hang.
  await expect(card).not.toBeAttached({ timeout: 4_500 })

  const invalidKeyframeWarnings = consoleMessages.filter((m) => m.includes('Invalid keyframe'))
  expect(invalidKeyframeWarnings).toEqual([])
})

test('showcase page: animates with zero console errors', async ({ page }) => {
  const errors: string[] = []
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text())
  })
  page.on('pageerror', (err) => errors.push(err.message))

  await page.goto('/showcase', { waitUntil: 'domcontentloaded' })
  await page.waitForTimeout(1_500) // let mount/loop/view animations settle in

  expect(errors).toEqual([])
})
