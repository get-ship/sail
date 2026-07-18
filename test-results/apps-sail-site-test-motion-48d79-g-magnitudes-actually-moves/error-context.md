# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: apps/sail-site/test/motion.spec.ts >> /motion page: an all-inline loop (bespoke duration, easing, magnitudes) actually moves
- Location: apps/sail-site/test/motion.spec.ts:197:0

# Error details

```
Error: goto: Protocol error (Page.navigate): Cannot navigate to invalid URL
Call log:
  - navigating to "/motion", waiting until "domcontentloaded"

```

# Test source

```ts
  105 |   }
  106 | 
  107 |   expect(motionWarnings).toEqual([])
  108 | })
  109 | 
  110 | test('showcase page: animates with zero console errors', async ({ page }) => {
  111 |   const errors: string[] = []
  112 |   page.on('console', (msg) => {
  113 |     if (msg.type() === 'error') errors.push(msg.text())
  114 |   })
  115 |   page.on('pageerror', (err) => errors.push(err.message))
  116 | 
  117 |   await page.goto('/showcase', { waitUntil: 'domcontentloaded' })
  118 |   await page.waitForTimeout(1_500) // let mount/loop/view animations settle in
  119 | 
  120 |   expect(errors).toEqual([])
  121 | })
  122 | 
  123 | test('landing MOTION section: the spring ball, the stagger bar, and the drift chip actually move', async ({ page }) => {
  124 |   const motionWarnings: string[] = []
  125 |   page.on('console', (msg) => {
  126 |     if (msg.text().includes('[sail/motion]')) motionWarnings.push(msg.text())
  127 |   })
  128 | 
  129 |   await page.goto('/', { waitUntil: 'domcontentloaded' })
  130 | 
  131 |   // The `sx=` policy compiles utility classes to hashed `sx-*` names and
  132 |   // erases the `<card>`/`<column>` custom element tags entirely — so these
  133 |   // demos are found by the caption text immediately preceding them, not by
  134 |   // a stable class or tag name.
  135 |   // FIG. 01 SPRING PHYSICS — motion:loop={{ translate-y: 'spring:bouncy' }}
  136 |   const ball = page.locator('p:has-text("SPRING PHYSICS") + div')
  137 |   // FIG. 02 STAGGER WAVE — 5 bars, motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}
  138 |   const bar = page.locator('p:has-text("STAGGER WAVE") + div > div').first()
  139 |   // FIG. 03 DRIFT — motion:loop={{ translate-x: 'spring:gentle' }}
  140 |   const chip = page.locator('p:has-text("DRIFT") + div')
  141 | 
  142 |   await expect(ball).toBeVisible()
  143 |   await expect(bar).toBeVisible()
  144 |   await expect(chip).toBeVisible()
  145 | 
  146 |   // Let the cards' own enter/stagger animation finish before sampling the
  147 |   // ongoing loop — 3s clears the deal-in stagger with headroom.
  148 |   await page.waitForTimeout(3_000)
  149 | 
  150 |   const [ballSample, barSample, chipSample] = await Promise.all([
  151 |     movedAfter600ms(page, ball),
  152 |     movedAfter600ms(page, bar),
  153 |     movedAfter600ms(page, chip),
  154 |   ])
  155 | 
  156 |   // Each is a translate-y or translate-x spring:bouncy/gentle loop with
  157 |   // iterations: Infinity — over 600ms of an ongoing oscillation the
  158 |   // computed `translate` MUST differ. Before the fix these kept
  159 |   // reporting `none`/`0px 0px` forever: playState running, currentTime
  160 |   // advancing, zero pixels moved.
  161 |   expect(ballSample.after).not.toBe(ballSample.before)
  162 |   expect(barSample.after).not.toBe(barSample.before)
  163 |   expect(chipSample.after).not.toBe(chipSample.before)
  164 | 
  165 |   expect(motionWarnings).toEqual([])
  166 | })
  167 | 
  168 | test('/motion page: the ambient loop dots keep animating (translate-y, staggered, infinite)', async ({ page }) => {
  169 |   const motionWarnings: string[] = []
  170 |   page.on('console', (msg) => {
  171 |     if (msg.text().includes('[sail/motion]')) motionWarnings.push(msg.text())
  172 |   })
  173 | 
  174 |   await page.goto('/motion', { waitUntil: 'domcontentloaded' })
  175 | 
  176 |   const dot = page
  177 |     .locator('xpath=//h2[contains(text(),"Ambient")]/following-sibling::div[contains(@class,"sail-row")][1]/div')
  178 |     .first()
  179 |   await expect(dot).toBeVisible()
  180 | 
  181 |   await page.waitForTimeout(3_000)
  182 |   const { before, after } = await movedAfter600ms(page, dot)
  183 |   expect(after).not.toBe(before)
  184 | 
  185 |   expect(motionWarnings).toEqual([])
  186 | })
  187 | 
  188 | // ---------------------------------------------------------------------------
  189 | // Inline values — the end-to-end proof that a hand-written duration/easing/
  190 | // magnitude actually produces a MOVING element, not just a well-formed config.
  191 | //
  192 | // This is the failure mode the loop-dead bug taught: an animation can be
  193 | // `playState: 'running'` with `iterations: Infinity` and still move nothing.
  194 | // Object-existence assertions do not catch that; sampling the computed
  195 | // transform at two points in time does.
  196 | 
  197 | test('/motion page: an all-inline loop (bespoke duration, easing, magnitudes) actually moves', async ({
  198 |   page,
  199 | }) => {
  200 |   const motionWarnings: string[] = []
  201 |   page.on('console', (msg) => {
  202 |     if (msg.text().includes('[sail/motion]')) motionWarnings.push(msg.text())
  203 |   })
  204 | 
> 205 |   await page.goto('/motion', { waitUntil: 'domcontentloaded' })
      |             ^ Error: goto: Protocol error (Page.navigate): Cannot navigate to invalid URL
  206 | 
  207 |   const chip = page.locator('[data-testid="inline-demo"]')
  208 |   await expect(chip).toBeVisible()
  209 | 
  210 |   // All three channels are driven by inline values only — no preset supplies
  211 |   // any of these magnitudes, and no design token supplies the 3600ms.
  212 |   const translateBefore = await readTranslate(chip)
  213 |   const rotateBefore = await readRotateDeg(chip)
  214 |   await page.waitForTimeout(700)
  215 |   const translateAfter = await readTranslate(chip)
  216 |   const rotateAfter = await readRotateDeg(chip)
  217 | 
  218 |   expect(translateAfter).not.toBe(translateBefore)
  219 |   expect(rotateAfter).not.toBe(rotateBefore)
  220 | 
  221 |   // And the inline easing/duration produced a real WAAPI animation with the
  222 |   // timing the author wrote — 3600ms, not a token's value.
  223 |   const timing = await chip.evaluate((el) =>
  224 |     el.getAnimations().map((a) => {
  225 |       const t = a.effect?.getTiming()
  226 |       return { duration: t?.duration, iterations: t?.iterations }
  227 |     }),
  228 |   )
  229 |   expect(timing.length).toBeGreaterThan(0)
  230 |   expect(timing.some((t) => t.duration === 3600)).toBe(true)
  231 |   expect(timing.every((t) => t.iterations === Infinity)).toBe(true)
  232 | 
  233 |   expect(motionWarnings).toEqual([])
  234 | })
  235 | 
```