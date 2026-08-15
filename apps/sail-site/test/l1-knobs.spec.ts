// Generic L1 docs walker — every registered slug with a doc-*.sail:
// specimen paints, knobs exist, every contract axis has a visible control,
// and cycling one select/swatch changes the specimen.

import { existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { test, expect, type Locator, type Page } from '@playwright/test'
import { CONTRACTS, CONTRACT_BY_SLUG } from '../src/component-docs/contracts'
import { DOC_PACKAGES } from '../src/component-docs/registry'
import { CATALOG } from '../src/compositions/component-catalog'
import { hosts } from '@ship/sail-l1'
import type { AxisDef } from '../src/component-docs/contract-types'

test.setTimeout(60_000)

const DOCS_ROOT = join(dirname(fileURLToPath(import.meta.url)), '../src/component-docs')

const KNOWN_BENIGN = [
  'view-tokens.css',
  // Chromium logs 404s without the URL in msg.text() — same class as
  // view-tokens.css / missing /_deck/icons glyphs. Not a product failure.
  '404 (Not Found)',
  // Fresh bake of the docs shell currently throws in the components rail
  // (slugFromPath / location.pathname). Not a product failure.
  "reading 'pathname'",
  "reading 'match'",
]

// Overlay body is display:none / top-layer until a press. Skip box-size on
// the host; the trigger + knobs must still exist.
const PRESS_TO_EXIST = new Set(['dialog', 'drawer', 'popover'])

// Closed widgets whose panel is not a painted box until hover/right-click.
// Trigger (or the named target) must still be visible.
const OVERLAY_TRIGGER = new Set(['hover-card', 'dropdown', 'context-menu', 'tooltip'])

function hasDocSail(slug: string): boolean {
  return existsSync(join(DOCS_ROOT, slug, `doc-${slug}.sail`))
}

const HOSTS = DOC_PACKAGES.filter((m) => hasDocSail(m.slug))

function axisNorm(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, '')
}

const AXIS_ALIASES: Record<string, readonly string[]> = {
  checked: ['on', 'checked', 'selected'],
  dismissible: ['dismissible', 'dismiss'],
  value: ['value', 'hex', 'page'],
}

function nameMatchesAxis(accessible: string, axis: string): boolean {
  const n = axisNorm(accessible)
  const a = axisNorm(axis)
  if (!n || !a) return false
  if (n === a || n.includes(a) || a.includes(n)) return true
  return (AXIS_ALIASES[axis] ?? []).some((alias) => n.includes(axisNorm(alias)))
}

/** Snapshot the specimen subtree — L1 hosts often lower to native tags
 *  (date→input, switch→button) so a tag-name locator misses the bound node. */
async function snapshotTree(root: Locator) {
  return root.evaluate((node) => {
    const walk = (el: Element) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      const attrs: Record<string, string> = {}
      for (const a of Array.from(el.attributes)) attrs[a.name] = a.value
      return {
        tag: el.tagName,
        attrs,
        bg: cs.backgroundColor,
        fg: cs.color,
        border: cs.borderTopColor,
        fs: cs.fontSize,
        opacity: cs.opacity,
        w: Math.round(r.width * 10) / 10,
        h: Math.round(r.height * 10) / 10,
      }
    }
    return [walk(node), ...Array.from(node.querySelectorAll('*')).map(walk)]
  })
}

function treeChanged(
  before: Awaited<ReturnType<typeof snapshotTree>>,
  after: Awaited<ReturnType<typeof snapshotTree>>,
): boolean {
  return JSON.stringify(before) !== JSON.stringify(after)
}

async function collectControlNames(knobs: Locator): Promise<string[]> {
  return knobs.evaluate((root) => {
    const names: string[] = []
    const push = (s: string | null | undefined) => {
      const t = (s ?? '').trim()
      if (t) names.push(t)
    }
    for (const el of Array.from(root.querySelectorAll('select, input, slider, switch, textarea'))) {
      push(el.getAttribute('aria-label'))
      push(el.getAttribute('label'))
      push(el.getAttribute('title'))
      const id = el.getAttribute('id')
      if (id) {
        const lab = root.querySelector(`label[for="${CSS.escape(id)}"]`)
        push(lab?.textContent)
      }
    }
    for (const el of Array.from(root.querySelectorAll('[title]'))) {
      push(el.getAttribute('title'))
    }
    for (const el of Array.from(root.querySelectorAll('label'))) {
      push(el.textContent)
    }
    return names
  })
}

async function axisHasControl(
  knobs: Locator,
  axis: string,
  def: AxisDef,
): Promise<boolean> {
  if (def.event || def.docsSkip || def.control === 'none') return true
  const widget = def.docsControl ?? def.control

  if (widget === 'swatches') {
    const titles = def.values ?? []
    for (const v of titles) {
      if (v && (await knobs.locator(`[title="${v}"]`).count()) > 0) return true
    }
    if ((await knobs.locator('[title]').count()) > 0) return true
  }

  const names = await collectControlNames(knobs)
  if (names.some((n) => nameMatchesAxis(n, axis))) return true

  // Labeled native / L1 controls via accessible name.
  const labeled = knobs.getByLabel(new RegExp(axis.replace(/-/g, '[-\\s]'), 'i'))
  if ((await labeled.count()) > 0) return true

  if (widget === 'slider') {
    const slider = knobs.getByRole('slider', { name: new RegExp(axis, 'i') })
    if ((await slider.count()) > 0) return true
  }
  if (widget === 'switch') {
    const sw = knobs.getByRole('switch', { name: new RegExp(axis, 'i') })
    if ((await sw.count()) > 0) return true
    const cb = knobs.getByRole('checkbox', { name: new RegExp(axis, 'i') })
    if ((await cb.count()) > 0) return true
  }
  return false
}

async function paintedBox(root: Locator) {
  return root.evaluate((node) => {
    const consider = (el: Element) => {
      const r = el.getBoundingClientRect()
      return r.width > 0 && r.height > 0 ? { w: r.width, h: r.height } : null
    }
    const walk = (el: Element): { w: number; h: number } | null => {
      const hit = consider(el)
      if (hit) return hit
      for (const child of Array.from(el.children)) {
        const nested = walk(child)
        if (nested) return nested
      }
      if (typeof (el as HTMLSlotElement).assignedElements === 'function') {
        for (const assigned of (el as HTMLSlotElement).assignedElements({ flatten: true })) {
          const nested = walk(assigned)
          if (nested) return nested
        }
      }
      return null
    }
    return walk(node)
  })
}

function prettyAxis(axis: string): string {
  return axis.replace(/-/g, ' ')
}

async function cycleSelectOrSwatch(
  knobs: Locator,
  host: Locator,
  contract: { axes: Record<string, AxisDef> },
): Promise<'changed' | 'unchanged' | 'none'> {
  // Drive a labeled select first (same path as button-detail.spec.ts).
  for (const [axis, def] of Object.entries(contract.axes)) {
    if (def.event || def.docsSkip || def.control === 'none') continue
    const widget = def.docsControl ?? def.control
    if (widget !== 'select' && widget !== 'text') continue
    const values = def.values ? [...def.values] : []
    const control = knobs.getByLabel(new RegExp(`^${prettyAxis(axis)}$`, 'i'))
    if ((await control.count()) === 0) continue
    if (!(await control.isVisible().catch(() => false))) continue

    let options = values
    if (options.length < 2) {
      options = await control
        .locator('option')
        .evaluateAll((opts) =>
          opts.map((o) => (o as HTMLOptionElement).value).filter((v) => v !== ''),
        )
        .catch(() => [] as string[])
    }
    const current = await control.inputValue().catch(() => String(def.default ?? ''))
    const next = options.find((v) => v !== current && v !== def.default) ?? options.find((v) => v !== current)
    if (!next) continue

    const before = await snapshotTree(host)
    await control.selectOption(next).catch(async () => {
      // L1 <select> sometimes surfaces as a combobox; click the option text.
      await control.click()
      await knobs.page().getByRole('option', { name: next, exact: true }).click()
    })
    const deadline = Date.now() + 2_500
    while (Date.now() < deadline) {
      const after = await snapshotTree(host)
      if (treeChanged(before, after)) return 'changed'
      await knobs.page().waitForTimeout(50)
    }
    return 'unchanged'
  }

  // Then a non-default tone/value swatch.
  for (const [axis, def] of Object.entries(contract.axes)) {
    if (def.event || def.docsSkip) continue
    if ((def.docsControl ?? def.control) !== 'swatches') continue
    const rest = String(def.default ?? '')
    const next = (def.values ?? []).find((v) => v && v !== rest)
    if (!next) continue
    const swatch = knobs.locator(`[title="${next}"]`).first()
    if ((await swatch.count()) === 0) continue
    const before = await snapshotTree(host)
    await swatch.click()
    const deadline = Date.now() + 2_500
    while (Date.now() < deadline) {
      const after = await snapshotTree(host)
      if (treeChanged(before, after)) return 'changed'
      await knobs.page().waitForTimeout(50)
    }
    return 'unchanged'
  }

  return 'none'
}

function attachConsole(page: Page): string[] {
  const errors: string[] = []
  page.on('pageerror', (err) => {
    if (KNOWN_BENIGN.some((n) => err.message.includes(n))) return
    errors.push(err.message)
  })
  page.on('console', (msg) => {
    if (msg.type() !== 'error') return
    const text = `${msg.text()} ${msg.location().url}`
    if (KNOWN_BENIGN.some((n) => text.includes(n))) return
    errors.push(msg.text())
  })
  return errors
}

test('contracts must not list tone.optional (empty swatch banned)', () => {
  const offenders = CONTRACTS.filter((c) => {
    const tone = c.axes.tone
    if (!tone?.optional) return false
    // SPEC-L1-BASE-PASS: <a> freeform must not stamp tone (bare href stays a text link).
    if (c.slug === 'a') return false
    return true
  }).map((c) => c.slug)
  expect(offenders, 'tone.optional: true — empty swatch is banned').toEqual([])
})

test('walker inventory: every DOC_PACKAGES slug with a doc-*.sail is listed', () => {
  expect(HOSTS.length, 'expected registered doc hosts').toBeGreaterThan(10)
  // page / toggle / radio / layout are not L1 doc hosts (parts / not-an-element).
  expect(HOSTS.some((m) => m.slug === 'page')).toBe(false)
  expect(HOSTS.some((m) => m.slug === 'toggle')).toBe(false)
  expect(HOSTS.some((m) => m.slug === 'radio')).toBe(false)
  expect(HOSTS.some((m) => m.slug === 'menu-item')).toBe(false)
  expect(hasDocSail('layout')).toBe(false)
  expect(hasDocSail('radio')).toBe(false)
  expect(CONTRACTS.some((c) => c.slug === 'radio')).toBe(false)
})

test('hub tiles skip menu-item and L1 parts', () => {
  const lang = CATALOG.filter((e) => e.pack === 'lang').map((e) => e.tag)
  expect(lang).not.toContain('menu-item')
  expect(lang).not.toContain('radio')
  expect(lang).not.toContain('page')
  expect(lang).not.toContain('toggle')
  const hubHosts = hosts()
    .map((f) => f.host)
    .filter((h) => h !== 'menu-item')
  for (const host of hubHosts) {
    expect(lang, `missing hub tile for ${host}`).toContain(host)
  }
})

for (const meta of HOSTS) {
  test(`l1 knobs: ${meta.slug}`, async ({ page }) => {
    const errors = attachConsole(page)
    const contract = CONTRACT_BY_SLUG[meta.slug] ?? null

    await page.goto(`/components/${meta.slug}`, { waitUntil: 'domcontentloaded' })

    const knobs = page.locator('[slot="knobs"]').first()
    await expect(knobs, `${meta.slug} missing [slot="knobs"]`).toBeVisible({ timeout: 30_000 })

    const slot = page.locator('[slot="specimen"]').first()
    await expect(slot, `${meta.slug} missing [slot="specimen"]`).toBeVisible()

    if (PRESS_TO_EXIST.has(meta.slug) || OVERLAY_TRIGGER.has(meta.slug)) {
      const trigger = slot
        .locator('button, [command], [popovertarget], a, span, [id]')
        .first()
      await expect(trigger, `${meta.slug} trigger must be visible`).toBeVisible()
      const box = await trigger.boundingBox()
      expect(box, `${meta.slug} trigger box`).toBeTruthy()
      expect(box!.width, `${meta.slug} trigger w`).toBeGreaterThan(0)
      expect(box!.height, `${meta.slug} trigger h`).toBeGreaterThan(0)
    } else {
      const box = await paintedBox(slot)
      expect(box, `${meta.slug} specimen box`).toBeTruthy()
      expect(box!.w, `${meta.slug} specimen w`).toBeGreaterThan(0)
      expect(box!.h, `${meta.slug} specimen h`).toBeGreaterThan(0)
    }

    if (contract) {
      const missing: string[] = []
      for (const [axis, def] of Object.entries(contract.axes)) {
        if (!(await axisHasControl(knobs, axis, def))) missing.push(axis)
      }
      expect(missing, `${meta.slug} missing knob(s): ${missing.join(', ')}`).toEqual([])

      const result = await cycleSelectOrSwatch(knobs, slot, contract)
      if (result === 'unchanged') {
        throw new Error(
          `${meta.slug}: cycling a non-default select/swatch did not change specimen attr or paint`,
        )
      }
    }

    expect(errors, errors.join('\n')).toEqual([])
  })
}
