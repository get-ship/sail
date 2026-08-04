// U2.1b — Site theme-picker (header chrome).
// Catalog filter tabs (no paint on tab click); theme row applies paint;
// never data-mode=system on the document.

import { test, expect, type Page } from '@playwright/test'

async function openHeaderPicker(page: Page) {
  await page.goto('/', { waitUntil: 'domcontentloaded' })
  // Header trigger: aria-label "Theme: …" or "Choose theme" before action sync.
  const trigger = page
    .locator('sail-site-header button[popovertarget], sail-site-header button[aria-label*="Theme"]')
    .first()
  await expect(trigger).toBeVisible({ timeout: 30_000 })
  await trigger.click()
  const panel = page.locator('sail-site-header [popover], sail-site-header [role="listbox"]').first()
  await expect(panel).toBeVisible({ timeout: 10_000 })
  return { trigger, panel }
}

test.describe('U2.1b site theme-picker', () => {
  test('catalog options match effective theme ids; System shown for dual-mode', async ({
    page,
  }) => {
    const { panel } = await openHeaderPicker(page)

    const optionValues = await panel.locator('[role="option"][value]').evaluateAll((els) =>
      els.map((e) => e.getAttribute('value') ?? ''),
    )
    // Catalog is non-empty and includes core + known aesthetics.
    expect(optionValues.length).toBeGreaterThanOrEqual(8)
    expect(optionValues).toContain('ship')
    expect(optionValues).toContain('synthwave')
    expect(optionValues).toContain('paper')
    // No twin ids
    expect(optionValues.some((v) => v.includes('-dark') || v.includes('-light'))).toBe(false)

    // Dual-mode default theme: Light | Dark | System visible.
    const modeLabels = await panel.locator('[data-mode-bar] button, button[data-mode]').evaluateAll((els) =>
      els
        .filter((e) => !(e as HTMLElement).hidden)
        .map((e) => ({
          mode: e.getAttribute('data-mode'),
          text: (e.textContent ?? '').trim(),
        })),
    )
    const modes = modeLabels.map((m) => m.mode)
    expect(modes).toContain('light')
    expect(modes).toContain('dark')
    expect(modes).toContain('system')
  })

  test('filter tabs do not paint document; only filter + preview data-mode', async ({
    page,
  }) => {
    const { panel } = await openHeaderPicker(page)

    const before = await page.evaluate(() => ({
      theme: document.documentElement.getAttribute('theme') ?? '',
      mode: document.documentElement.dataset.mode ?? '',
      pref: localStorage.getItem('ship-site-theme-mode'),
    }))

    await panel.locator('button[data-mode="dark"]').click()
    // Give the filter a tick to update previews — document must stay put.
    await page.waitForFunction(
      () => {
        const tab = document.querySelector(
          'sail-site-header button[data-mode="dark"]',
        )
        return tab?.getAttribute('aria-selected') === 'true'
      },
      undefined,
      { timeout: 10_000 },
    )

    const afterDark = await page.evaluate(() => ({
      theme: document.documentElement.getAttribute('theme') ?? '',
      mode: document.documentElement.dataset.mode ?? '',
      pref: localStorage.getItem('ship-site-theme-mode'),
    }))
    expect(afterDark.theme).toBe(before.theme)
    expect(afterDark.mode).toBe(before.mode)
    expect(afterDark.pref).toBe(before.pref)
    expect(afterDark.mode).not.toBe('system')

    // Dual-mode theme swatches track filter specimen mode.
    const shipSwatch = await panel
      .locator('[theme="ship"], [theme="sail"]')
      .first()
      .getAttribute('data-mode')
    expect(shipSwatch).toBe('dark')

    await panel.locator('button[data-mode="system"]').click()
    await page.waitForFunction(
      () => {
        const tab = document.querySelector(
          'sail-site-header button[data-mode="system"]',
        )
        return tab?.getAttribute('aria-selected') === 'true'
      },
      undefined,
      { timeout: 10_000 },
    )
    const afterSystem = await page.evaluate(() => ({
      theme: document.documentElement.getAttribute('theme') ?? '',
      mode: document.documentElement.dataset.mode ?? '',
      pref: localStorage.getItem('ship-site-theme-mode'),
    }))
    expect(afterSystem.theme).toBe(before.theme)
    expect(afterSystem.mode).toBe(before.mode)
    expect(afterSystem.pref).toBe(before.pref)
    expect(afterSystem.mode).not.toBe('system')

    await panel.locator('button[data-mode="light"]').click()
    await page.waitForFunction(
      () => {
        const tab = document.querySelector(
          'sail-site-header button[data-mode="light"]',
        )
        return tab?.getAttribute('aria-selected') === 'true'
      },
      undefined,
      { timeout: 10_000 },
    )
    const afterLight = await page.evaluate(() => ({
      mode: document.documentElement.dataset.mode ?? '',
      pref: localStorage.getItem('ship-site-theme-mode'),
    }))
    expect(afterLight.mode).toBe(before.mode)
    expect(afterLight.pref).toBe(before.pref)
  })

  test('selecting a theme applies theme, marks aria-selected, keeps panel open', async ({
    page,
  }) => {
    const { trigger, panel } = await openHeaderPicker(page)

    // Ensure light filter so paper applies light.
    await panel.locator('button[data-mode="light"]').click()

    // paper remains dual-mode; safe pick that keeps light valid.
    await panel.locator('[role="option"][value="paper"]').click()
    await page.waitForFunction(
      () => (document.documentElement.getAttribute('theme') ?? '') === 'paper',
      undefined,
      { timeout: 10_000 },
    )
    const after = await page.evaluate(() => ({
      theme: document.documentElement.getAttribute('theme') ?? '',
      mode: document.documentElement.dataset.mode ?? '',
      darkClass: document.documentElement.classList.contains('dark'),
    }))
    expect(after.theme).toBe('paper')
    // Paint mode is always light|dark, never system.
    expect(['light', 'dark']).toContain(after.mode)
    expect(after.mode).not.toBe('system')
    expect(after.darkClass).toBe(false)

    // Catalog stays open for comparison; selection chrome updates in place.
    await expect(panel).toBeVisible()
    await expect(panel.locator('[role="option"][value="paper"]')).toHaveAttribute(
      'aria-selected',
      'true',
    )
    const name = await trigger.getAttribute('aria-label')
    expect(name ?? '').toMatch(/paper|Paper/i)
  })

  test('Light|Dark|System tabs filter the catalog and live previews track mode', async ({
    page,
  }) => {
    const { panel } = await openHeaderPicker(page)

    // Live preview specimens present (not chip-only).
    await expect(panel.locator('[theme]').first()).toBeVisible()

    // Row layout: options are full-width list rows (not a 2-col card grid of huge cards).
    const layout = await panel.locator('[data-theme-grid]').evaluate((grid) => {
      const style = getComputedStyle(grid)
      return {
        display: style.display,
        cols: style.gridTemplateColumns,
      }
    })
    // column/flex list — not multi-column grid
    if (layout.display === 'grid') {
      // single column only
      const colCount = layout.cols.split(' ').filter(Boolean).length
      expect(colCount).toBeLessThanOrEqual(1)
    }

    await panel.locator('button[data-mode="dark"]').click()
    await page.waitForFunction(
      () => {
        const tab = document.querySelector(
          'sail-site-header button[data-mode="dark"]',
        )
        return tab?.getAttribute('aria-selected') === 'true'
      },
      undefined,
      { timeout: 10_000 },
    )
    // dark tab selected (filter chrome only)
    await expect(panel.locator('button[data-mode="dark"]')).toHaveAttribute(
      'aria-selected',
      'true',
    )
    // light-only themes hidden
    const glassHidden = await panel
      .locator('[role="option"][value="glass"]')
      .evaluate((el) => (el as HTMLElement).hidden)
      .catch(() => true)
    expect(glassHidden).toBe(true)

    // dual preview paints dark specimen
    const shipMode = await panel
      .locator('[theme="ship"], [theme="sail"]')
      .first()
      .getAttribute('data-mode')
    expect(shipMode).toBe('dark')

    // Theme click under dark filter is what paints the site (dual-mode theme).
    await panel.locator('[role="option"][value="neutral"]').click()
    await page.waitForFunction(
      () =>
        (document.documentElement.getAttribute('theme') ?? '') === 'neutral' &&
        (document.documentElement.dataset.mode ?? '') === 'dark',
      undefined,
      { timeout: 10_000 },
    )
  })

  test('popover anchors near header trigger (not top-left 0,0)', async ({ page }) => {
    const { trigger, panel } = await openHeaderPicker(page)
    const geometry = await page.evaluate(() => {
      const t = document.querySelector(
        'sail-site-header button[popovertarget], sail-site-header button[aria-label*="Theme"]',
      ) as HTMLElement | null
      const p = document.querySelector(
        'sail-site-header [popover], sail-site-header [role="listbox"]',
      ) as HTMLElement | null
      if (!t || !p) return null
      const tr = t.getBoundingClientRect()
      const pr = p.getBoundingClientRect()
      return {
        triggerTop: tr.top,
        triggerLeft: tr.left,
        panelTop: pr.top,
        panelLeft: pr.left,
        panelW: pr.width,
        panelH: pr.height,
      }
    })
    expect(geometry).not.toBeNull()
    expect(geometry!.panelW).toBeGreaterThan(100)
    expect(geometry!.panelH).toBeGreaterThan(80)
    // Anchored under/near the header control — not abandoned at 0,0.
    expect(geometry!.panelTop).toBeGreaterThan(geometry!.triggerTop - 8)
    expect(Math.abs(geometry!.panelLeft)).toBeGreaterThan(0)
  })
})
