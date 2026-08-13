// Compiled <button> × first-party themes.
//
// First-party for this gate: ship (light product) + console (dark product).
// We do not matrix sail + neutral here — they are first-party cousins of ship
// and blow the surface without adding a new paint model. Fun themes (halo,
// glass, brut, 8bit, synthwave, paper, contrast) stay picker catalog / taste,
// not this CI grid.
//
// This is the product suite. l1-conformance hand-HTML is not.

import { test, expect, type Page } from '@playwright/test'

test.setTimeout(60_000)

const FIRST_PARTY = [
  { id: 'ship', mode: 'light' },
  { id: 'console', mode: 'dark' },
] as const

const VARIANTS = ['solid', 'soft', 'outline', 'ghost'] as const

function clear(v: string): boolean {
  const s = v.trim().toLowerCase()
  return (
    s === '' ||
    s === 'transparent' ||
    s === 'rgba(0, 0, 0, 0)' ||
    s === 'rgba(0,0,0,0)'
  )
}

async function applyTheme(page: Page, id: string, mode: string) {
  await page.evaluate(
    ({ id: t, mode: m }) => {
      try {
        localStorage.setItem('ship-site-theme', t)
        localStorage.setItem('ship-site-theme-mode', m)
      } catch {
        /* ignore */
      }
      const root = document.documentElement
      root.setAttribute('theme', t)
      root.dataset.mode = m
    },
    { id, mode },
  )
}

async function paint(page: Page) {
  const el = page.locator('[slot="specimen"] button').first()
  await expect(el).toBeVisible({ timeout: 30_000 })
  return el.evaluate((node) => {
    const cs = getComputedStyle(node)
    const r = node.getBoundingClientRect()
    return {
      bg: cs.backgroundColor,
      fg: cs.color,
      border: cs.borderTopColor,
      w: r.width,
      h: r.height,
      variant: node.getAttribute('variant'),
      tone: node.getAttribute('tone'),
    }
  })
}

test.describe('button × first-party themes (compiled docs)', () => {
  for (const { id, mode } of FIRST_PARTY) {
    test(`${id}/${mode}: bare default + each variant is a finished control`, async ({
      page,
    }) => {
      await page.goto('/components/button', { waitUntil: 'domcontentloaded' })
      await applyTheme(page, id, mode)
      const knobs = page.locator('[slot="knobs"]')
      const specimen = page.locator('[slot="specimen"] button').first()
      await expect(specimen).toBeVisible({ timeout: 30_000 })

      const def = await paint(page)
      expect(def.w).toBeGreaterThan(0)
      expect(def.h).toBeGreaterThan(0)
      expect(def.tone, 'default tone is primary').toBe('primary')
      expect(def.variant).toBe('solid')
      expect(clear(def.bg), `${id} default solid must fill`).toBe(false)
      expect(def.bg).not.toBe(def.fg)

      for (const v of VARIANTS) {
        await knobs.getByLabel('Variant').selectOption(v)
        await expect(specimen).toHaveAttribute('variant', v)
        const p = await paint(page)
        expect(p.w).toBeGreaterThan(0)
        expect(p.h).toBeGreaterThan(0)
        expect(p.tone).toBe('primary')
        if (v === 'solid' || v === 'soft') {
          expect(clear(p.bg), `${id} ${v} must fill`).toBe(false)
          expect(p.bg, `${id} ${v} text ≠ fill`).not.toBe(p.fg)
        }
        if (v === 'outline') {
          expect(clear(p.border), `${id} outline must stroke`).toBe(false)
        }
        if (v === 'ghost') {
          expect(clear(p.fg), `${id} ghost must have ink`).toBe(false)
        }
      }
    })
  }
})
