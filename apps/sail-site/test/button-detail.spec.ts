// Button docs configurator — knobs must change computed paint / a11y name.

import { test, expect, type Locator, type Page } from '@playwright/test'

test.setTimeout(60_000)

const KNOWN_BENIGN = [
  'view-tokens.css',
  // Fresh bake of the docs shell currently throws in the components rail
  // (slugFromPath / location.pathname). Not a button product failure.
  "reading 'pathname'",
  "reading 'match'",
]

async function paint(el: Locator) {
  return el.evaluate((node) => {
    const cs = getComputedStyle(node)
    const r = node.getBoundingClientRect()
    return {
      bg: cs.backgroundColor,
      fg: cs.color,
      cursor: cs.cursor,
      opacity: cs.opacity,
      w: r.width,
      h: r.height,
      variant: node.getAttribute('variant'),
      tone: node.getAttribute('tone'),
      size: node.getAttribute('size'),
      disabled: (node as HTMLButtonElement).disabled,
      label: node.getAttribute('aria-label'),
    }
  })
}

async function ready(page: Page) {
  const specimen = page.locator('[slot="specimen"] button').first()
  await expect(specimen).toBeVisible({ timeout: 30_000 })
  return specimen
}

test('button detail: knobs change paint, size, disabled, and icon name', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (err) => errors.push(err.message))
  page.on('console', (msg) => {
    if (msg.type() !== 'error') return
    if (KNOWN_BENIGN.some((n) => msg.text().includes(n))) return
    errors.push(msg.text())
  })

  await page.goto('/components/button', { waitUntil: 'domcontentloaded' })
  const specimen = await ready(page)
  const knobs = page.locator('[slot="knobs"]')

  const rest = await paint(specimen)
  expect(rest.variant).toBe('solid')
  expect(rest.tone).toBe('primary')
  expect(rest.size).toBe('md')
  expect(rest.disabled).toBe(false)
  expect(rest.w).toBeGreaterThan(0)
  expect(rest.h).toBeGreaterThan(0)

  await knobs.getByLabel('Variant').selectOption('outline')
  await expect(specimen).toHaveAttribute('variant', 'outline')
  const outline = await paint(specimen)
  expect(outline.bg).not.toBe(rest.bg)
  const outlineBorder = await specimen.evaluate((node) => getComputedStyle(node).borderTopColor)
  expect(outlineBorder, 'outline must keep a visible stroke').not.toMatch(/rgba\(0,\s*0,\s*0,\s*0\)|transparent/)

  await knobs.getByLabel('Variant').selectOption('solid')
  await knobs.locator('[title="danger"]').click()
  await expect(specimen).toHaveAttribute('tone', 'danger')
  const danger = await paint(specimen)
  expect(danger.bg).not.toBe(rest.bg)

  const range = knobs.getByRole('slider', { name: 'Size' })
  await range.fill('0')
  await expect(specimen).toHaveAttribute('size', 'xs')
  const xs = await paint(specimen)
  expect(xs.h).toBeLessThan(rest.h)

  await range.fill('3')
  await expect(specimen).toHaveAttribute('size', 'lg')
  const lg = await paint(specimen)
  expect(lg.h).toBeGreaterThan(rest.h)

  await range.fill('4')
  await expect(specimen).toHaveAttribute('size', 'icon')
  const icon = await paint(specimen)
  expect(Math.abs(icon.w - icon.h)).toBeLessThan(1)
  expect(icon.label).toBe('Settings')

  await range.fill('2')
  await expect(specimen).toHaveAttribute('size', 'md')
  await knobs.getByRole('switch', { name: 'Disabled' }).click()
  await expect(specimen).toBeDisabled()
  const off = await paint(specimen)
  expect(off.cursor).toBe('not-allowed')
  expect(Number(off.opacity)).toBeLessThan(1)

  expect(errors, errors.join('\n')).toEqual([])
})
