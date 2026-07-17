// Motion exit-hold + preset-validity smoke — drives /motion's live
// "Remove the panel" demo and asserts the runtime actually holds the node
// in the DOM through its exit animation instead of removing it
// immediately, and that zero "Invalid keyframe" WAAPI warnings are logged
// anywhere on the page (the rotate-preset bug). /showcase is a broader
// animate-heavy surface checked for zero console errors.
//
// The tests below this point pin the translate-x/y-loop-dead bug: a
// `motion:loop` animation can be `playState: 'running'` with
// `iterations: Infinity` and STILL move nothing, because its keyframes
// spoke `x`/`y` (motion/mini's internal value names) instead of the CSS
// `translate` property. Object-existence assertions (dispose is a
// function, `toBeVisible()`) do not catch this — only sampling the
// element's actual computed `translate` at two points in time does.

import { test, expect, type Locator, type Page } from '@playwright/test'

/** Read the live computed `translate` shorthand off an element — the
 *  one CSS property translate-x/y motions resolve through post-fix.
 *  Sampling this (not `getAnimations()`/`playState`) is what actually
 *  proves the element is moving. */
async function readTranslate(locator: Locator): Promise<string> {
  return locator.evaluate((el) => getComputedStyle(el).translate)
}

/** Sample an element's computed `translate` once, wait 600ms, sample
 *  again. The caller is expected to have already let ambient `loop`
 *  motions run past their initial mount/enter settle (~3s) before
 *  calling this, so the two samples land squarely inside the
 *  spring's ongoing oscillation rather than its one-shot arrival. */
async function movedAfter600ms(page: Page, locator: Locator): Promise<{ before: string; after: string }> {
  const before = await readTranslate(locator)
  await page.waitForTimeout(600)
  const after = await readTranslate(locator)
  return { before, after }
}

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

test('landing MOTION section: the spring ball, the stagger bar, and the drift chip actually move', async ({ page }) => {
  const motionWarnings: string[] = []
  page.on('console', (msg) => {
    if (msg.text().includes('[sui/motion]')) motionWarnings.push(msg.text())
  })

  await page.goto('/', { waitUntil: 'domcontentloaded' })

  // The `sx=` policy compiles utility classes to hashed `sx-*` names and
  // erases the `<card>`/`<column>` custom element tags entirely — so these
  // demos are found by the caption text immediately preceding them, not by
  // a stable class or tag name.
  // FIG. 01 SPRING PHYSICS — motion:loop={{ translate-y: 'spring:bouncy' }}
  const ball = page.locator('p:has-text("SPRING PHYSICS") + div')
  // FIG. 02 STAGGER WAVE — 5 bars, motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}
  const bar = page.locator('p:has-text("STAGGER WAVE") + div > div').first()
  // FIG. 03 DRIFT — motion:loop={{ translate-x: 'spring:gentle' }}
  const chip = page.locator('p:has-text("DRIFT") + div')

  await expect(ball).toBeVisible()
  await expect(bar).toBeVisible()
  await expect(chip).toBeVisible()

  // Let the cards' own enter/stagger animation finish before sampling the
  // ongoing loop — 3s clears the deal-in stagger with headroom.
  await page.waitForTimeout(3_000)

  const [ballSample, barSample, chipSample] = await Promise.all([
    movedAfter600ms(page, ball),
    movedAfter600ms(page, bar),
    movedAfter600ms(page, chip),
  ])

  // Each is a translate-y or translate-x spring:bouncy/gentle loop with
  // iterations: Infinity — over 600ms of an ongoing oscillation the
  // computed `translate` MUST differ. Before the fix these kept
  // reporting `none`/`0px 0px` forever: playState running, currentTime
  // advancing, zero pixels moved.
  expect(ballSample.after).not.toBe(ballSample.before)
  expect(barSample.after).not.toBe(barSample.before)
  expect(chipSample.after).not.toBe(chipSample.before)

  expect(motionWarnings).toEqual([])
})

test('/motion page: the ambient loop dots keep animating (translate-y, staggered, infinite)', async ({ page }) => {
  const motionWarnings: string[] = []
  page.on('console', (msg) => {
    if (msg.text().includes('[sui/motion]')) motionWarnings.push(msg.text())
  })

  await page.goto('/motion', { waitUntil: 'domcontentloaded' })

  const dot = page
    .locator('xpath=//h2[contains(text(),"Ambient")]/following-sibling::div[contains(@class,"sui-row")][1]/div')
    .first()
  await expect(dot).toBeVisible()

  await page.waitForTimeout(3_000)
  const { before, after } = await movedAfter600ms(page, dot)
  expect(after).not.toBe(before)

  expect(motionWarnings).toEqual([])
})
