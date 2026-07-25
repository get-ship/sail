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

/** Parse the computed `rotate` shorthand (`"none"` or `"Ndeg"`) into a
 *  plain degree number, normalized to `[0, 360)`. */
async function readRotateDeg(locator: Locator): Promise<number> {
  const raw = await locator.evaluate((el) => getComputedStyle(el).rotate)
  if (raw === 'none') return 0
  const deg = parseFloat(raw)
  return ((deg % 360) + 360) % 360
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

test('/motion page: spin preset rotates continuously, one direction, never ping-pongs', async ({ page }) => {
  const motionWarnings: string[] = []
  page.on('console', (msg) => {
    if (msg.text().includes('[sail/motion]')) motionWarnings.push(msg.text())
  })

  await page.goto('/motion', { waitUntil: 'domcontentloaded' })

  const spinner = page.getByTestId('spin-demo')
  await expect(spinner).toBeVisible()

  // Sample every 500ms across 2.5s. `motion:loop={{ rotate: 'spin' }}`
  // is a 14s-ish linear spin, so within this window the raw degree
  // readings never wrap past 360 — no need to unwrap the samples.
  const samples: number[] = []
  for (let i = 0; i < 6; i++) {
    samples.push(await readRotateDeg(spinner))
    if (i < 5) await page.waitForTimeout(500)
  }

  // Advanced: the rotation at t+2s differs from t (RED/GREEN — a dead
  // loop would report the same value at every sample, exactly like the
  // translate-x/y-loop-dead bug this file's header describes).
  expect(samples[5]).not.toBe(samples[0])

  // One direction: each consecutive sample is >= the last (monotonic,
  // modulo the 14s period this window doesn't reach) — a ping-pong
  // (direction: 'alternate') would show the sequence reverse partway
  // through instead of climbing the whole way.
  for (let i = 1; i < samples.length; i++) {
    expect(samples[i]).toBeGreaterThanOrEqual(samples[i - 1]!)
  }

  expect(motionWarnings).toEqual([])
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
    if (msg.text().includes('[sail/motion]')) motionWarnings.push(msg.text())
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
    if (msg.text().includes('[sail/motion]')) motionWarnings.push(msg.text())
  })

  await page.goto('/motion', { waitUntil: 'domcontentloaded' })

  const dot = page
    .locator('xpath=//h2[contains(text(),"Ambient")]/following-sibling::div[contains(@class,"sail-row")][1]/div')
    .first()
  await expect(dot).toBeVisible()

  await page.waitForTimeout(3_000)
  const { before, after } = await movedAfter600ms(page, dot)
  expect(after).not.toBe(before)

  expect(motionWarnings).toEqual([])
})

// ---------------------------------------------------------------------------
// Inline values — the end-to-end proof that a hand-written duration/easing/
// magnitude actually produces a MOVING element, not just a well-formed config.
//
// This is the failure mode the loop-dead bug taught: an animation can be
// `playState: 'running'` with `iterations: Infinity` and still move nothing.
// Object-existence assertions do not catch that; sampling the computed
// transform at two points in time does.

test('/motion page: an all-inline loop (bespoke duration, easing, magnitudes) actually moves', async ({
  page,
}) => {
  const motionWarnings: string[] = []
  page.on('console', (msg) => {
    if (msg.text().includes('[sail/motion]')) motionWarnings.push(msg.text())
  })

  await page.goto('/motion', { waitUntil: 'domcontentloaded' })

  const chip = page.locator('[data-testid="inline-demo"]')
  await expect(chip).toBeVisible()

  // All three channels are driven by inline values only — no preset supplies
  // any of these magnitudes, and no design token supplies the 3600ms.
  const translateBefore = await readTranslate(chip)
  const rotateBefore = await readRotateDeg(chip)
  await page.waitForTimeout(700)
  const translateAfter = await readTranslate(chip)
  const rotateAfter = await readRotateDeg(chip)

  expect(translateAfter).not.toBe(translateBefore)
  expect(rotateAfter).not.toBe(rotateBefore)

  // And the inline easing/duration produced a real WAAPI animation with the
  // timing the author wrote — 3600ms, not a token's value.
  const timing = await chip.evaluate((el) =>
    el.getAnimations().map((a) => {
      const t = a.effect?.getTiming()
      return { duration: t?.duration, iterations: t?.iterations }
    }),
  )
  expect(timing.length).toBeGreaterThan(0)
  expect(timing.some((t) => t.duration === 3600)).toBe(true)
  expect(timing.every((t) => t.iterations === Infinity)).toBe(true)

  expect(motionWarnings).toEqual([])
})

// ── The choreography lane, live on the page ──────────────────────────────────
//
// The /motion page's `timeline()` demo is the teaching surface for the second
// lane, and its whole claim is "scrub the slider and the tile moves the way
// the data says". That claim is only worth making if it survives the real
// compile -> reflect -> CSS path: `timeline()` returns signals, the compiler
// emits a custom-property reflection for each `(--tile-x)` read, and the
// element's `translate`/`rotate`/`scale` resolve from those properties.
//
// Sampling the tile's computed values at specific slider positions is the only
// assertion that proves all three links at once — `toBeVisible()` passes on a
// tile that never moves, which is precisely the failure this file already
// exists to catch for `motion:loop`.

test.describe('/motion — timeline() scrub demo', () => {
  /** Set the range input and let the reflection effect flush. */
  async function scrubTo(page: Page, value: number): Promise<void> {
    const slider = page.getByTestId('timeline-scrub')
    await slider.fill(String(value))
    await slider.dispatchEvent('input')
    await page.waitForTimeout(50)
  }

  /** The tile's computed transform channels, as plain numbers. */
  async function readTile(page: Page): Promise<{ x: number; rotate: number; scale: number }> {
    return page.getByTestId('timeline-tile').evaluate((el) => {
      const s = getComputedStyle(el)
      return {
        x: parseFloat(s.translate) || 0,
        rotate: parseFloat(s.rotate) || 0,
        scale: parseFloat(s.scale) || 1,
      }
    })
  }

  test('the tile follows the keyframe data, holds, and pops', async ({ page }) => {
    await page.goto('/motion')
    await expect(page.getByTestId('timeline-tile')).toBeVisible()

    // 0 — parked at the origin, unrotated, unscaled.
    await scrubTo(page, 0)
    expect(await readTile(page)).toMatchObject({ x: 0, rotate: 0, scale: 1 })

    // 34 — the translate and rotate have arrived in full.
    const settled = (await scrubTo(page, 34), await readTile(page))
    expect(settled.x).toBeCloseTo(120, 1)
    expect(settled.rotate).toBeCloseTo(15, 1)
    expect(settled.scale).toBeCloseTo(1, 2)

    // 44 — unchanged. This is the HOLD: the same VALUE repeated at a later
    // offset. Clamp-and-hope arithmetic drifts here.
    const held = (await scrubTo(page, 44), await readTile(page))
    expect(held.x).toBeCloseTo(120, 1)
    expect(held.rotate).toBeCloseTo(15, 1)

    // 54 — the pop happens IN PLACE: scale peaks while x and rotate stay put.
    const popped = (await scrubTo(page, 54), await readTile(page))
    expect(popped.scale).toBeCloseTo(1.75, 2)
    expect(popped.x).toBeCloseTo(120, 1)
    expect(popped.rotate).toBeCloseTo(15, 1)

    // 64 — scale back to 1, still parked.
    const relaxed = (await scrubTo(page, 64), await readTile(page))
    expect(relaxed.scale).toBeCloseTo(1, 2)
    expect(relaxed.x).toBeCloseTo(120, 1)

    // 100 — home.
    const home = (await scrubTo(page, 100), await readTile(page))
    expect(home.x).toBeCloseTo(0, 1)
    expect(home.rotate).toBeCloseTo(0, 1)
    expect(home.scale).toBeCloseTo(1, 2)
  })

  test('the demo is genuinely driven — the scrub sweeps it through a real range', async ({
    page,
  }) => {
    await page.goto('/motion')
    await expect(page.getByTestId('timeline-tile')).toBeVisible()

    const xs: number[] = []
    for (const v of [0, 10, 20, 25, 30, 34, 70, 80, 90, 100]) {
      await scrubTo(page, v)
      xs.push((await readTile(page)).x)
    }
    // A tile whose reflection never wired up reports ONE value for all of them.
    expect(new Set(xs.map((n) => n.toFixed(2))).size).toBeGreaterThan(6)
    expect(Math.max(...xs)).toBeCloseTo(120, 1)
    expect(Math.min(...xs)).toBeCloseTo(0, 1)
  })
})
