// Language style-object: compile → mount → assert computed paint + geometry.
// Token vars come from the page floor (same contract as `@ship/tokens`);
// the gate is what Chromium computes, not the emitted class string.

import { test, expect } from '@playwright/test'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { compileAndBundle, pageHtml } from './compile-and-bundle'

const HERE = dirname(fileURLToPath(import.meta.url))
const FIXTURE = resolve(HERE, 'fixtures/style-object.sail')

test.describe('style object — compile → DOM', () => {
  test('named style paints and lays out; an unstyled sibling does not', async ({ page }) => {
    const bundle = compileAndBundle(FIXTURE)
    await page.setContent(pageHtml('style-object'))
    await page.addScriptTag({ content: bundle })

    const styled = page.getByTestId('styled-tile')
    const plain = page.getByTestId('plain')
    await expect(styled).toBeVisible()
    await expect(plain).toBeVisible()

    const shot = await page.evaluate(() => {
      const read = (el: Element) => {
        const cs = getComputedStyle(el)
        const box = el.getBoundingClientRect()
        return {
          padding: cs.padding,
          width: cs.width,
          height: cs.height,
          background: cs.backgroundColor,
          radius: cs.borderRadius,
          boxW: box.width,
          boxH: box.height,
        }
      }
      return {
        styled: read(document.querySelector('[data-testid="styled-tile"]')!),
        plain: read(document.querySelector('[data-testid="plain"]')!),
      }
    })

    expect(shot.styled.padding).toBe('16px')
    expect(shot.styled.width).toBe('192px')
    expect(shot.styled.height).toBe('80px')
    expect(shot.styled.background).toBe('rgb(200, 40, 20)')
    expect(shot.styled.radius).toBe('6px')
    expect(shot.styled.boxW).toBe(224)
    expect(shot.styled.boxH).toBe(112)

    expect(shot.plain.padding).not.toBe('16px')
    expect(shot.plain.background).not.toBe('rgb(200, 40, 20)')
    expect(shot.plain.boxW).toBeLessThan(shot.styled.boxW)
    expect(shot.plain.boxH).toBeLessThan(shot.styled.boxH)
  })
})
