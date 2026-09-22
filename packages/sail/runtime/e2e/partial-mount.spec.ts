// Invoking an in-file partial must mount the same tree a handwritten
// expansion does. Compare observations (text, computed paint, boxes),
// never the compiler's JS.

import { test, expect, type Page } from '@playwright/test'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { compileAndBundle, pageHtml } from './compile-and-bundle'

const HERE = dirname(fileURLToPath(import.meta.url))
const INVOKE = resolve(HERE, 'fixtures/partial-invoke.sail')
const HAND = resolve(HERE, 'fixtures/partial-handwritten.sail')

interface Shot {
  titles: string[]
  slots: string[]
  tiles: Array<{
    padding: string
    background: string
    radius: string
    boxW: number
    boxH: number
  }>
}

async function mountAndRead(page: Page, bundle: string): Promise<Shot> {
  await page.setContent(pageHtml('partial-mount'))
  await page.addScriptTag({ content: bundle })
  await expect(page.getByTestId('host')).toBeVisible()
  return page.evaluate(() => {
    const titles = [...document.querySelectorAll('[data-testid="title"]')].map(
      (el) => el.textContent ?? '',
    )
    const slots = [
      document.querySelector('[data-testid="slot-a"]')?.textContent ?? '',
      document.querySelector('[data-testid="slot-b"]')?.textContent ?? '',
    ]
    const tiles = [...document.querySelectorAll('[data-testid="tile"]')].map((el) => {
      const cs = getComputedStyle(el)
      const box = el.getBoundingClientRect()
      return {
        padding: cs.padding,
        background: cs.backgroundColor,
        radius: cs.borderRadius,
        boxW: box.width,
        boxH: box.height,
      }
    })
    return { titles, slots, tiles }
  })
}

test.describe('partial invocation ≡ handwritten mount', () => {
  test('two invocations match two handwritten tiles in text, paint, and boxes', async ({
    browser,
  }) => {
    const invokeBundle = compileAndBundle(INVOKE)
    const handBundle = compileAndBundle(HAND)

    const invokePage = await browser.newPage()
    const handPage = await browser.newPage()
    try {
      const [invoke, hand] = await Promise.all([
        mountAndRead(invokePage, invokeBundle),
        mountAndRead(handPage, handBundle),
      ])

      expect(invoke.titles).toEqual(['Alpha', 'Beta'])
      expect(invoke.slots).toEqual(['A', 'B'])
      expect(invoke.tiles).toHaveLength(2)
      expect(invoke.tiles[0]!.padding).toBe('16px')
      expect(invoke.tiles[0]!.background).toBe('rgb(200, 40, 20)')
      expect(invoke.tiles[0]!.radius).toBe('6px')
      expect(invoke.tiles[0]!.boxW).toBeGreaterThan(0)
      expect(invoke.tiles[0]!.boxH).toBeGreaterThan(0)

      expect(invoke).toEqual(hand)
    } finally {
      await invokePage.close()
      await handPage.close()
    }
  })
})
