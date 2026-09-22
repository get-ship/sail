// C-TH-NEST-1 — nested theme + mode on /showcase (shanty).
// Site dark + nested paper → filter input field tokens match the nested frame.

import { test, expect } from '@playwright/test'

test.describe('C-TH-NEST-1 nested theme field inheritance', () => {
  test('site dark + nested paper: filter input matches nested field tokens', async ({
    page,
  }) => {
    await page.goto('/showcase', { waitUntil: 'domcontentloaded' })

    // Nested frame: pick paper (light-only) via the shanty theme chip first.
    const paperBtn = page.getByRole('button', { name: 'paper', exact: true })
    await expect(paperBtn).toBeVisible({ timeout: 30_000 })
    await paperBtn.click()

    // Force site product theme dark after chip click so header/storage cannot
    // clobber the document mid-test. Nested paper mode is light-only.
    await page.evaluate(() => {
      const root = document.documentElement
      root.setAttribute('theme', 'sail')
      root.setAttribute('mode', 'dark')
      root.classList.remove('dark')
      // Re-stamp nested mode after document mode is known dark.
      document.dispatchEvent(
        new CustomEvent('ship:theme:change', {
          detail: { id: 'sail', dark: true, mode: 'dark' },
        }),
      )
    })

    await page.waitForFunction(
      () => {
        const root = document.documentElement
        const frame = document.getElementById('shanty-frame')
        return (
          root.getAttribute('theme') === 'sail' &&
          root.getAttribute('mode') === 'dark' &&
          frame?.getAttribute('theme') === 'paper' &&
          (frame?.getAttribute('mode') === 'light' ||
            frame?.getAttribute('mode') === 'light')
        )
      },
      undefined,
      { timeout: 15_000 },
    )

    const probe = await page.evaluate(() => {
      const root = document.documentElement
      const frame = document.getElementById('shanty-frame')
      const input = document.getElementById('shanty-filter') as HTMLInputElement | null
      if (!frame || !input) {
        return { ok: false as const, reason: 'missing frame or filter input' }
      }
      const frameCs = getComputedStyle(frame)
      const inputCs = getComputedStyle(input)
      const rootCs = getComputedStyle(root)
      const fieldBg = frameCs.getPropertyValue('--field').trim()
      const fieldFg = frameCs.getPropertyValue('--field-foreground').trim()
      const rootFieldBg = rootCs.getPropertyValue('--field').trim()
      // Force the same paint the L1 skin uses — proves the nested field token
      // is what the control reads (used-value oklab vs oklch custom props).
      const prevBg = input.style.backgroundColor
      const prevFg = input.style.color
      input.style.backgroundColor = 'var(--field)'
      input.style.color = 'var(--field-foreground)'
      const paintedBg = getComputedStyle(input).backgroundColor
      const paintedFg = getComputedStyle(input).color
      input.style.backgroundColor = prevBg
      input.style.color = prevFg
      const naturalBg = inputCs.backgroundColor
      const naturalFg = inputCs.color

      /** Parse first number in oklab/oklch/rgb as a rough L / channel. */
      const firstNum = (s: string): number => {
        const m = s.match(/[\d.]+/)
        return m ? Number(m[0]) : NaN
      }

      return {
        ok: true as const,
        frameTheme: frame.getAttribute('theme'),
        frameMode: frame.getAttribute('mode') ?? frame.getAttribute('mode') ?? '',
        rootTheme: root.getAttribute('theme'),
        rootMode: root.getAttribute('mode') ?? '',
        inputType: input.getAttribute('type'),
        fieldBg,
        fieldFg,
        rootFieldBg,
        paintedBg,
        paintedFg,
        naturalBg,
        naturalFg,
        paintedL: firstNum(paintedBg),
        naturalL: firstNum(naturalBg),
        rootFieldL: firstNum(rootFieldBg),
        fieldL: firstNum(fieldBg),
        colorScheme: frameCs.colorScheme,
      }
    })

    expect(probe.ok).toBe(true)
    if (!probe.ok) return

    expect(probe.frameTheme).toBe('paper')
    expect(probe.frameMode).toBe('light')
    expect(probe.rootTheme).toBe('sail')
    expect(probe.rootMode).toBe('dark')
    expect(probe.colorScheme).toBe('light')
    expect(probe.inputType).toBe('text')
    // Nested field tokens re-resolve against paper (not site dark).
    expect(probe.fieldBg.length).toBeGreaterThan(0)
    expect(probe.fieldBg).not.toBe(probe.rootFieldBg)
    expect(probe.fieldL).toBeGreaterThan(0.9)
    expect(probe.rootFieldL).toBeLessThan(0.5)
    // Input painted with var(--field) is light (paper), not site dark.
    expect(probe.paintedL).toBeGreaterThan(0.85)
    // Natural skin (when applied) should also be light; allow UA interim.
    if (Number.isFinite(probe.naturalL) && probe.naturalL > 0.01) {
      expect(probe.naturalL).toBeGreaterThan(0.7)
    }
  })

  test('theme chip flip restamps nested mode for dual-mode themes', async ({
    page,
  }) => {
    await page.goto('/showcase', { waitUntil: 'domcontentloaded' })
    await page.evaluate(() => {
      document.documentElement.setAttribute('theme', 'sail')
      document.documentElement.setAttribute('mode', 'dark')
      document.dispatchEvent(
        new CustomEvent('ship:theme:change', {
          detail: { id: 'sail', dark: true, mode: 'dark' },
        }),
      )
    })

    await page.getByRole('button', { name: 'sail', exact: true }).click()
    await page.waitForFunction(
      () => {
        const frame = document.getElementById('shanty-frame')
        return (
          frame?.getAttribute('theme') === 'sail' &&
          frame?.getAttribute('mode') === 'dark'
        )
      },
      undefined,
      { timeout: 10_000 },
    )

    await page.getByRole('button', { name: 'console', exact: true }).click()
    await page.waitForFunction(
      () => {
        const frame = document.getElementById('shanty-frame')
        return (
          frame?.getAttribute('theme') === 'console' &&
          frame?.getAttribute('mode') === 'dark'
        )
      },
      undefined,
      { timeout: 10_000 },
    )
  })
})
