// /components catalog — live pack previews mount on pack selection,
// and hard perf guards catch the style-tag / heap runaway that previously
// OOM'd this page.

import { test, expect } from '@playwright/test'

const PACKS = [
  {
    label: /Charts/i,
    tags: ['sail-sparkline', 'sail-area-chart', 'sail-bar-chart', 'sail-donut', 'sail-gauge', 'sail-heatmap', 'sail-line-chart'],
  },
  {
    label: /Media/i,
    tags: ['sail-media-image', 'sail-media-av'],
  },
  {
    label: /Code editor/i,
    tags: ['sail-code-editor'],
  },
  {
    label: /Mermaid/i,
    tags: ['sail-mermaid'],
  },
  {
    label: /Chat elements/i,
    tags: ['sail-message', 'sail-citation', 'sail-composer', 'sail-thinking-dots', 'sail-tool-call'],
  },
] as const

const STYLE_TAG_CEILING = 300
const MEMORY_GROWTH_CEILING = 96 * 1024 * 1024 // 96 MB between pack switches

test('catalog renders, pack previews mount on demand, and perf stays bounded', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (err) => errors.push(err.message))
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text())
  })

  await page.goto('/components', { waitUntil: 'domcontentloaded' })

  // Catalog renders with the default L1 pack.
  // Featured live demos also set data-tile (+ data-live); catalog cards do not.
  await expect(page.locator('[data-tile="button"]:not([data-live])')).toBeVisible({ timeout: 30_000 })
  await expect(page.locator('[data-tile="input"]:not([data-live])')).toBeVisible()
  await expect(page.getByText(/in catalog/i)).toBeVisible()

  // Pack previews are not mounted by default (only the featured L1 demos + placeholders).
  await expect(page.locator('sail-sparkline')).toHaveCount(0)
  await expect(page.locator('sail-code-editor')).toHaveCount(0)

  const styleTagCount = () => page.evaluate(() => document.querySelectorAll('style').length)
  await expect.poll(styleTagCount, { timeout: 10_000 }).toBeLessThanOrEqual(STYLE_TAG_CEILING)

  const sampleMemory = async () => {
    interface MemoryInfo { usedJSHeapSize: number }
    const mem = await page.evaluate(() => {
      // performance.memory is Chromium-only and not in the standard DOM types.
      const p = performance as unknown as { memory?: MemoryInfo }
      return p.memory ? { used: p.memory.usedJSHeapSize } : null
    })
    return mem?.used ?? null
  }

  let previousMemory = await sampleMemory()

  for (const pack of PACKS) {
    await page.getByRole('button', { name: pack.label }).click()

    // Wait for the first live tag of this pack to mount.
    await expect(page.locator(pack.tags[0])).toBeVisible({ timeout: 30_000 })

    // Every expected live tag for this pack is present exactly once.
    for (const tag of pack.tags) {
      await expect(page.locator(tag)).toHaveCount(1)
    }

    // Hard guard: the runtime must not be leaking <style> tags.
    await expect.poll(styleTagCount, { timeout: 10_000 }).toBeLessThanOrEqual(STYLE_TAG_CEILING)

    // Hard guard: JS heap growth between pack switches stays bounded if exposed.
    const currentMemory = await sampleMemory()
    if (previousMemory !== null && currentMemory !== null) {
      const growth = currentMemory - previousMemory
      expect(growth).toBeLessThanOrEqual(MEMORY_GROWTH_CEILING)
    }
    previousMemory = currentMemory
  }

  // Stress case: "All packs" mounts every pack preview at once.
  await page.getByRole('button', { name: /All packs/i }).click()
  for (const pack of PACKS) {
    for (const tag of pack.tags) {
      await expect(page.locator(tag)).toHaveCount(1)
    }
  }
  await expect.poll(styleTagCount, { timeout: 10_000 }).toBeLessThanOrEqual(STYLE_TAG_CEILING)

  // Returning to the language pack unmounts the pack CEs.
  await page.getByRole('button', { name: /Component primitives/i }).click()
  await expect(page.locator('sail-sparkline')).toHaveCount(0)
  await expect(page.locator('sail-code-editor')).toHaveCount(0)

  expect(errors).toEqual([])
})
