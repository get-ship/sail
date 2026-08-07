/**
 * THEMES-1 residual — override-order demo fixture + cascade proof.
 *
 * Spec acceptance: same element, overrides at each layer, resolved values
 * match instance > component > global > baked.
 *
 * Fixture: test/fixtures/override-order.html (openable demo page).
 * This suite mounts the fixture CSS model in happy-dom and asserts
 * getComputedStyle winners + resolve* helpers.
 */
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it, beforeEach } from 'vitest'
import {
  THEME_OVERRIDE_ORDER,
  resolveOverrideLayer,
  resolveTokenLayer,
} from '../src/theme-schema'

const FIXTURE = resolve(
  dirname(fileURLToPath(import.meta.url)),
  'fixtures/override-order.html',
)

const MARKERS = {
  baked: 'rgb(232, 228, 218)', // #e8e4da
  global: 'rgb(61, 126, 166)', // #3d7ea6
  component: 'rgb(196, 92, 38)', // #c45c26
  instance: 'rgb(91, 45, 142)', // #5b2d8e
} as const

function loadFixtureHtml(): string {
  return readFileSync(FIXTURE, 'utf8')
}

/** Extract <style>…</style> + body markup from the demo fixture. */
function parseFixture(html: string): { css: string; body: string } {
  const style = /<style>([\s\S]*?)<\/style>/i.exec(html)?.[1] ?? ''
  const body = /<body[^>]*>([\s\S]*?)<\/body>/i.exec(html)?.[1] ?? ''
  expect(style.length).toBeGreaterThan(100)
  expect(body.length).toBeGreaterThan(100)
  return { css: style, body }
}

function hexToRgb(hex: string): string {
  const h = hex.replace('#', '')
  const n = parseInt(h, 16)
  const r = (n >> 16) & 255
  const g = (n >> 8) & 255
  const b = n & 255
  return `rgb(${r}, ${g}, ${b})`
}

function normalizedBg(el: Element): string {
  const cs = getComputedStyle(el as HTMLElement)
  const raw =
    cs.backgroundColor?.trim() ||
    cs.getPropertyValue('background-color').trim() ||
    cs.getPropertyValue('background').trim()
  // happy-dom may return #hex or rgb()
  if (raw.startsWith('#')) return hexToRgb(raw)
  return raw.replace(/\s+/g, ' ')
}

describe('THEMES-1 override-order demo fixture', () => {
  beforeEach(() => {
    document.head.innerHTML = ''
    document.body.innerHTML = ''
  })

  it('fixture documents THEME_OVERRIDE_ORDER and four probes', () => {
    const html = loadFixtureHtml()
    expect(html).toContain('instance')
    expect(html).toContain('component')
    expect(html).toContain('global')
    expect(html).toContain('baked')
    expect(html).toMatch(/data-probe="baked"/)
    expect(html).toMatch(/data-probe="global"/)
    expect(html).toMatch(/data-probe="component"/)
    expect(html).toMatch(/data-probe="instance"/)
    expect([...THEME_OVERRIDE_ORDER]).toEqual([
      'instance',
      'component',
      'global',
      'baked',
    ])
  })

  it('same probe element: resolved paint matches instance > component > global > baked', () => {
    const { css, body } = parseFixture(loadFixtureHtml())
    const style = document.createElement('style')
    style.textContent = css
    document.head.appendChild(style)
    document.body.innerHTML = body

    const baked = document.querySelector('[data-probe="baked"]')
    const global = document.querySelector('[data-probe="global"]')
    const component = document.querySelector('[data-probe="component"]')
    const instance = document.querySelector('[data-probe="instance"]')
    expect(baked).toBeTruthy()
    expect(global).toBeTruthy()
    expect(component).toBeTruthy()
    expect(instance).toBeTruthy()

    // Same element kind (class="probe") at every layer.
    for (const el of [baked, global, component, instance]) {
      expect(el!.classList.contains('probe')).toBe(true)
    }

    expect(normalizedBg(baked!)).toBe(MARKERS.baked)
    expect(normalizedBg(global!)).toBe(MARKERS.global)
    expect(normalizedBg(component!)).toBe(MARKERS.component)
    expect(normalizedBg(instance!)).toBe(MARKERS.instance)

    // data-expect-* on the fixture stays honest with computed paint.
    for (const el of [baked, global, component, instance]) {
      const expectHex = el!.getAttribute('data-expect-color')!
      expect(normalizedBg(el!)).toBe(hexToRgb(expectHex))
    }
  })

  it('resolveTokenLayer flags match demo layer labels', () => {
    expect(resolveTokenLayer({}).layer).toBe('baked')
    expect(resolveTokenLayer({ globalOverride: true }).layer).toBe('global')
    expect(
      resolveTokenLayer({ componentOverride: true, globalOverride: true }).layer,
    ).toBe('component')
    expect(
      resolveTokenLayer({
        instance: true,
        componentOverride: true,
        globalOverride: true,
      }).layer,
    ).toBe('instance')

    // Progressive presence (demo cards): each higher layer beats lower ones.
    expect(resolveOverrideLayer({ baked: true })).toBe('baked')
    expect(resolveOverrideLayer({ global: true, baked: true })).toBe('global')
    expect(
      resolveOverrideLayer({ component: true, global: true, baked: true }),
    ).toBe('component')
    expect(
      resolveOverrideLayer({
        instance: true,
        component: true,
        global: true,
        baked: true,
      }),
    ).toBe('instance')
  })
})
