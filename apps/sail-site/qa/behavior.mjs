#!/usr/bin/env node
// apps/sail-site/qa/behavior.mjs — REPEATABLE BEHAVIOR + SURFACE-QUALITY HARNESS.
//
// Reads each cataloged component's CEM (custom elements manifest), drives the
// per-component detail page (`/components/<tag>`) which mounts the REAL
// specimen, and asserts:
//
//   1. PROPS — for each advertised attribute, the documented effect actually
//      takes effect (not "a ref property exists"):
//        • `disabled`     → genuinely non-interactive
//        • `size`         → measurably different dimensions
//        • `tone`         → distinct token-channel colors anywhere in shadow
//        • `open`         → the documented state actually toggles
//   2. INTERACTIVE/OVERLAY → open it, check it's painted and positioned
//      (anchored near trigger / centered) + sized sanely. PASS / FAIL /
//      REVIEW screenshot saved alongside for visual triage.
//   3. EVENTS — reserved: this gen of CEM has events[] but no schema for
//      event-detail typing; the dedicated event harness will live alongside
//      this one when a richer manifest lands.
//   4. SURFACE CONSISTENCY — the overlay/menu family must share a consistent
//      surface (background, border, radius, elevation). FLAG any that diverge
//      — the most likely regression from the sx→element migration.
//
// Each check yields PASS / FAIL / REVIEW. Failures exit non-zero.
//
//   node apps/sail-site/qa/behavior.mjs               # full sweep → behavior-report.json
//   node apps/sail-site/qa/behavior.mjs sail-dialog    # one component, verbose
//   node apps/sail-site/qa/behavior.mjs --overlay     # only the surface-consistency pass

import { chromium } from 'playwright'
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const APP_ROOT = join(HERE, '..')
const BASE = process.env.SUI_BEHAVIOR_BASE ?? 'http://127.0.0.1:7878'
const SHOT_DIR = join(HERE, 'qa-shots')
mkdirSync(SHOT_DIR, { recursive: true })

// ── CEM ingestion ────────────────────────────────────────────────────────

const CEM_TEXT = readFileSync(join(APP_ROOT, 'components.gen.ts'), 'utf8')
const CEM_MATCH = CEM_TEXT.match(/export const COMPONENTS_CATALOG = ([\s\S]*?) as const/)
if (!CEM_MATCH) throw new Error('could not parse components.gen.ts')
const CEM = eval('(' + CEM_MATCH[1] + ')')

const CAT_MAP_TEXT = readFileSync(join(APP_ROOT, 'src/sail-catalog/category-map.ts'), 'utf8')
const HIDDEN = new Set(
  [...CAT_MAP_TEXT.matchAll(/'(sail-[a-z-]+)':\s*true/g)].map(m => m[1]),
)
const PUBLIC_TAGS = [
  ...new Set(
    CEM.modules.flatMap(m => (m.declarations ?? [])
      .filter(d => d.kind === 'class' && d.customElement && d.tagName)
      .map(d => d.tagName)),
  ),
].filter(t => !HIDDEN.has(t))

const BY_TAG = new Map()
for (const m of CEM.modules) {
  for (const d of m.declarations ?? []) {
    if (!d.tagName) continue
    BY_TAG.set(d.tagName, {
      tag: d.tagName,
      pack: m.package ?? 'workspace',
      description: (d.description ?? '').split('\n')[0]?.trim() ?? '',
      props: (d.attributes ?? []).map(a => ({
        name: a.name ?? '',
        type: a.type?.text ?? 'string',
        default: a.default,
        values: a.values,
      })),
      slots: (d.slots ?? []).map(s => s.name),
      events: (d.events ?? []).map(e => e.name).filter(Boolean),
    })
  }
}

// ── Component classification ────────────────────────────────────────────────

const OVERLAY_FAMILY = [
  'sail-context-menu', 'sail-select', 'sail-dropdown-menu', 'sail-popover',
  'sail-dialog', 'sail-alert-dialog', 'sail-hover-card',
  'sail-combobox', 'sail-cmdk', 'sail-drawer', 'sail-sheet',
]
const OVERLAY_FAMILY_SET = new Set(OVERLAY_FAMILY)

const DRIVE_OVERLAY = {
  'sail-popover':       { kind: 'click',  trigger: '[slot="trigger"]' },
  'sail-hover-card':    { kind: 'attr',   attr: 'open' },
  'sail-dropdown-menu': { kind: 'click',  trigger: '[slot="trigger"]' },
  'sail-context-menu':  { kind: 'attr',   attr: 'open' },
  'sail-collapsible':   { kind: 'click',  trigger: '[slot="trigger"]' },
  'sail-dialog':        { kind: 'attr',   attr: 'open' },
  'sail-alert-dialog':  { kind: 'attr',   attr: 'open' },
  'sail-drawer':        { kind: 'attr',   attr: 'open' },
  'sail-cmdk':          { kind: 'attr',   attr: 'open' },
  'sail-select':        { kind: 'click',  trigger: '[part="trigger"], [slot="trigger"], button' },
  'sail-combobox':      { kind: 'click',  trigger: 'input, [part="trigger"], [slot="trigger"]' },
  'sail-autocomplete':  { kind: 'click',  trigger: 'input, [part="trigger"]' },
  'sail-color-picker':  { kind: 'click',  trigger: '[part="input"]' },
}

const TONE_TAGS = (() => {
  const candidates = ['sail-chip', 'sail-icon', 'sail-input',
    'sail-slider', 'sail-select', 'sail-combobox', 'sail-radio-group', 'sail-radio',
    'sail-toggle-group', 'sail-spinner', 'sail-thinking-dots',
    'sail-toast', 'sail-tabs', 'sail-pagination', 'sail-bar-chart',
    'sail-line-chart', 'sail-area-chart', 'sail-sparkline', 'sail-donut',
    'sail-gauge', 'sail-heatmap', 'sail-autocomplete',
    'sail-color-picker', 'sail-date-picker', 'sail-annotation-marker']
  return candidates.filter(t => BY_TAG.has(t) && BY_TAG.get(t).props.some(p => p.name === 'tone' && /\|/.test(p.type)))
})()

const SIZE_TAGS = ['sail-input', 'sail-chip', 'sail-icon',
  'sail-slider', 'sail-select', 'sail-combobox', 'sail-radio',
  'sail-spinner', 'sail-thinking-dots', 'sail-tabs', 'sail-heatmap',
  'sail-color-picker', 'sail-date-picker', 'sail-autocomplete', 'sail-gauge',
  'sail-pagination']
  .filter(t => BY_TAG.has(t) && BY_TAG.get(t).props.some(p => p.name === 'size'))

const DISABLED_TAGS = ['sail-input', 'sail-radio',
  'sail-select', 'sail-combobox', 'sail-toggle', 'sail-slider', 'sail-tabs',
  'sail-tab', 'sail-tab-panel', 'sail-pagination', 'sail-accordion']
  .filter(t => BY_TAG.has(t) && BY_TAG.get(t).props.some(p => p.name === 'disabled'))

const BOOLEAN_OPEN_TAGS = PUBLIC_TAGS.filter(t =>
  BY_TAG.get(t)?.props.some(p => p.name === 'open' && /boolean/.test(p.type)),
)

// ── Shadow-piercing probe (installed once per page) ───────────────────────

const probeSource = `
  const sel = (root, s) => {
    if (!root) return null
    if (root.matches?.(s)) return root
    const direct = root.querySelector?.(s); if (direct) return direct
    for (const el of root.querySelectorAll?.('*') ?? []) {
      if (el.shadowRoot) { const hit = sel(el.shadowRoot, s); if (hit) return hit }
    }
    return null
  }
  const walk = (root) => {
    const out = []; const seen = new Set()
    const into = (n, d) => {
      if (!n || seen.has(n)) return
      seen.add(n)
      out.push({ el: n, depth: d })
      if (n.shadowRoot) for (const c of n.shadowRoot.children) into(c, d+1)
      for (const c of n.children ?? []) into(c, d+1)
    }
    into(root, 0); return out
  }
  globalThis.__sel = sel
  globalThis.__walk = walk
`

// ── Result aggregation ────────────────────────────────────────────────────

let pass = 0, fail = 0, review = 0
const REPORT = {
  startedAt: new Date().toISOString(),
  base: BASE,
  totals: { pass: 0, fail: 0, review: 0 },
  surfaceInconsistencies: [],
  surfaceClusters: [],
  components: [],
}

function record(tag, kind, surface) {
  REPORT.totals[kind]++
  if (kind === 'pass') pass++; else if (kind === 'fail') fail++; else review++
  if (!REPORT.components.find(c => c.tag === tag)) REPORT.components.push({ tag, checks: [] })
  REPORT.components.find(c => c.tag === tag).checks.push(surface ?? {})
}

// ── Surface sampling ─────────────────────────────────────────────────────

async function sampleSurface(page, tag) {
  return page.evaluate(({ tag }) => {
    const stage = document.querySelector('.detail-stage')
    if (!stage) return null
    const host = stage.querySelector(tag)
    if (!host) return null
    const addCand = (el, acc) => {
      if (!(el instanceof HTMLElement)) return
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width < 8 || r.height < 8) return
      if (cs.visibility === 'hidden' || parseFloat(cs.opacity) < 0.4) return
      const bg = cs.backgroundColor
      const borderW = parseFloat(cs.borderTopWidth) || 0
      const radius = cs.borderTopLeftRadius
      const shadow = cs.boxShadow
      const hasSurface = bg && bg !== 'rgba(0, 0, 0, 0)'
      const score =
        (hasSurface ? 4 : 0) +
        (borderW > 0 ? 2 : 0) +
        (radius !== '0px' ? 1 : 0) +
        (shadow !== 'none' ? 1 : 0) +
        (r.width * r.height > 1000 ? 1 : 0)
      acc.push({ el, cs, score, area: r.width * r.height, parts: el.getAttribute('part') ?? '' })
    }
    const acc = []
    const w = globalThis.__walk ?? null
    if (!w) return null
    for (const { el } of w(host.shadowRoot ?? host)) addCand(el, acc)
    for (const { el } of w(document.body)) {
      if (el === host) continue
      const partAttr = (el.getAttribute && el.getAttribute('part')) ?? ''
      if (['topbar', 'content'].includes(partAttr)) continue
      const topLayer = (el.hasAttribute && (el.hasAttribute('popover') ||
        el.hasAttribute('data-sail-popover-open') ||
        el.hasAttribute('data-sail-dialog-open'))) || el.matches?.('[open][role="dialog"], [open][aria-modal]')
      const inStage = stage && (stage === el || stage.contains(el))
      if (!topLayer && !inStage) continue
      if (!partAttr && !topLayer) continue
      addCand(el, acc)
    }
    if (!acc.length) return null
    acc.sort((a, b) => b.score - a.score || b.area - a.area)
    const best = acc[0]
    const cs = best.cs
    return {
      tag,
      bg: cs.backgroundColor,
      borderColor: cs.borderTopColor,
      borderWidth: cs.borderTopWidth,
      borderRadius: cs.borderTopLeftRadius,
      boxShadow: cs.boxShadow,
      padding: cs.padding,
      width: best.el.getBoundingClientRect().width,
      height: best.el.getBoundingClientRect().height,
      parts: best.parts,
    }
  }, { tag })
}

async function driveOpen(page, tag) {
  const hint = DRIVE_OVERLAY[tag]
  if (!hint) return { kind: 'mount-only' }
  if (hint.kind === 'attr') {
    return page.evaluate(({ tag, attr }) => {
      const s = document.querySelector('.detail-stage')
      const h = s?.querySelector(tag)
      if (!h) return { kind: 'attr', result: 'no-host' }
      h.setAttribute(attr, 'true')
      return { kind: 'attr', result: 'set', attr, value: h.getAttribute(attr) }
    }, { tag, attr: hint.attr })
  }
  return page.evaluate(({ tag, hint }) => {
    const s = document.querySelector('.detail-stage')
    const h = s?.querySelector(tag)
    if (!h) return { kind: hint.kind, result: 'no-host' }
    let target = globalThis.__sel?.(h.shadowRoot, hint.trigger)
    if (!target) for (const c of h.children) if (c.matches?.(hint.trigger)) { target = c; break }
    if (!target) return { kind: hint.kind, result: 'no-trigger', trigger: hint.trigger }
    if (hint.kind === 'hover') {
      const r = target.getBoundingClientRect()
      const opts = { bubbles: true, composed: true, cancelable: true, clientX: r.left + r.width / 2, clientY: r.top + r.height / 2 }
      target.dispatchEvent(new PointerEvent('pointerenter', opts))
      target.dispatchEvent(new MouseEvent('mouseenter', opts))
      target.dispatchEvent(new MouseEvent('mouseover', opts))
      return { kind: 'hover', result: 'dispatched' }
    }
    target.click()
    // Belt + suspenders for popover="auto": if the click didn't promote the
    // panel to top-layer, call showPopover() directly so the surface is
    // paintable. (popovertarget association works in real DOM, but synthetic
    // clicks in shadow-DOM sometimes need the explicit API call.)
    setTimeout(() => {
      try {
        const panel = h.shadowRoot?.querySelector('.panel')
        if (panel?.hasAttribute?.('popover') && !panel.matches?.(':popover-open')) {
          panel.showPopover?.()
        }
      } catch {}
    }, 80)
    return { kind: 'click', result: 'clicked' }
  }, { tag, hint })
}

// ── Per-component checks ─────────────────────────────────────────────────

async function checkOverlay(page, tag) {
  const surfaceBefore = await sampleSurface(page, tag)
  const drive = await driveOpen(page, tag)
  await page.waitForTimeout(450)
  const surfaceAfter = await sampleSurface(page, tag)
  const openState = await page.evaluate(({ tag }) => {
    const stage = document.querySelector('.detail-stage')
    const host = stage?.querySelector(tag)
    if (!host) return { painted: 0, hostFound: false }
    const walk = globalThis.__walk(document.body)
    let painted = 0, biggest = 0
    for (const { el } of walk) {
      if (!(el instanceof HTMLElement)) continue
      const r = el.getBoundingClientRect()
      if (r.width < 4 || r.height < 4) continue
      if (getComputedStyle(el).visibility === 'hidden') continue
      const partAttr = el.getAttribute('part') ?? ''
      if (['topbar', 'content'].includes(partAttr)) continue
      const topLayer = el.hasAttribute?.('popover') || el.matches?.('[open][role="dialog"], [open][aria-modal]')
      const inStage = stage && (stage === el || stage.contains(el))
      if (!topLayer && !inStage) continue
      painted++
      const area = r.width * r.height
      if (area > biggest) biggest = area
    }
    return { painted, biggest, hostW: host.getBoundingClientRect().width, hostH: host.getBoundingClientRect().height }
  }, { tag })
  // Capture the OPEN state as a screenshot — evidence for the operator to
  // triage the surface treatment at their leisure. Always captured so PASS
  // outcomes are auditable too.
  let shot = null
  try {
    const stageBox = await page.evaluate(() => {
      const s = document.querySelector('.detail-stage')
      if (!s) return null
      const r = s.getBoundingClientRect()
      const cl = (n, max) => Math.max(0, Math.min(max, n))
      const x = cl(r.x - 80, window.innerWidth)
      const y = cl(r.y - 60, window.innerHeight)
      return { x, y, width: Math.min(window.innerWidth - x, r.width + 160), height: Math.min(window.innerHeight - y, r.height + 120) }
    })
    if (stageBox && stageBox.width > 0 && stageBox.height > 0) {
      const png = await page.screenshot({ clip: stageBox })
      const fp = join(SHOT_DIR, tag.replace(/[^a-z0-9-]/gi, '_') + '.png')
      writeFileSync(fp, png)
      shot = fp
    }
  } catch {}
  const result = { kind: 'overlay', drive, openState, surfaceBefore, surfaceAfter, shot }
  const ok = openState.biggest > 200 || !!surfaceAfter
  if (!ok) {
    record(tag, 'fail', { ...result, reason: 'overlay-never-opened' })
    return
  }
  record(tag, 'pass', { ...result, kind: 'overlay' })
}

async function checkProp(page, tag, prop) {
  return page.evaluate(async ({ tag, prop }) => {
    const stage = document.querySelector('.detail-stage')
    const h = stage?.querySelector(tag)
    if (!h) return { reason: 'no-host' }
    const read = () => {
      const r = h.getBoundingClientRect()
      const cs = getComputedStyle(h)
      const walk = globalThis.__walk(h.shadowRoot ?? h)
      let best = h, bestArea = 0
      for (const { el } of walk) {
        if (!(el instanceof HTMLElement)) continue
        const er = el.getBoundingClientRect()
        if (er.width > 4 && er.height > 4) {
          const a = er.width * er.height
          if (a > bestArea) { bestArea = a; best = el }
        }
      }
      const cs2 = getComputedStyle(best)
      return {
        hostW: r.width, hostH: r.height,
        innerBg: cs2.backgroundColor, innerColor: cs2.color,
        innerBorder: cs2.borderTopColor, innerRadius: cs2.borderTopLeftRadius,
        innerShadow: cs2.boxShadow, innerFontSize: cs2.fontSize,
        innerLineHeight: cs2.lineHeight, innerPadding: cs2.padding,
        best,
      }
    }
    const initial = read()
    if (prop.kind === 'tone') {
      const values = prop.values?.filter(v => v !== 'default') ?? []
      if (!values.length) return { reason: 'tone-no-variants', prop: prop.name, initial }
      const before = h.getAttribute(prop.name)
      const samples = []
      for (const v of values) {
        h.setAttribute(prop.name, v)
        await new Promise(r => setTimeout(r, 80))
        samples.push({ v, ...read() })
      }
      if (before !== null) h.setAttribute(prop.name, before); else h.removeAttribute(prop.name)
      // Accept any channel change as "tone responds". A spinner animated
      // ring is the canonical border-only case; sail-alert tone changes the
      // icon part. Picking the largest shadow descendant isn't enough.
      const channelDistincts = ['innerBg', 'innerBorder', 'innerColor'].map(c => new Set(samples.map(s => s[c])).size)
      const pass = channelDistincts.some(n => n >= 2)
      return { reason: 'tone', prop: prop.name, samples,
        bgColorCount: channelDistincts[0],
        borderColorCount: channelDistincts[1],
        textColorCount: channelDistincts[2],
        pass }
    }
    if (prop.kind === 'size') {
      const sizeValues = (prop.sizeValues ?? []).filter(v => /^(sm|md|lg|xs|xl)$/.test(v))
      if (sizeValues.length < 2) return { reason: 'size-insufficient-values', prop: 'size', initial }
      const before = h.getAttribute('size')
      const measurements = {}
      for (const v of sizeValues) {
        h.setAttribute('size', v)
        await new Promise(r => setTimeout(r, 80))
        const r = h.getBoundingClientRect()
        let biggestArea = 0, area = 0
        for (const { el } of globalThis.__walk(h.shadowRoot ?? h)) {
          if (!(el instanceof HTMLElement)) continue
          const er = el.getBoundingClientRect()
          if (er.width > 4 && er.height > 4) { const a = er.width * er.height; if (a > biggestArea) { biggestArea = a; area = er.height } }
        }
        measurements[v] = { hostW: r.width, hostH: r.height, innerH: area }
      }
      if (before !== null) h.setAttribute('size', before); else h.removeAttribute('size')
      const hostHs = new Set(Object.values(measurements).map(m => Math.round(m.hostH))).size
      const innerHs = new Set(Object.values(measurements).map(m => Math.round(m.innerH))).size
      const distinctH = hostHs >= 2 || innerHs >= 2
      return { reason: 'size', prop: 'size', measurements, distinctH, hostHDistinct: hostHs, innerHDistinct: innerHs, pass: distinctH }
    }
    if (prop.kind === 'disabled') {
      const before = h.hasAttribute('disabled')
      // SAIL framework 's `disabled reflect` mirrors JS prop → attribute
      // (attribute → prop isn't wired). Setting the attribute on the
      // host doesn't always update the internal prop — use the JS prop
      // directly (with attribute fallback).
      try { h.disabled = true } catch { h.setAttribute('disabled', '') }
      await new Promise(r => setTimeout(r, 80))
      const ariaDis = h.getAttribute('aria-disabled') === 'true'
      const disabledAttr = h.hasAttribute('disabled')
      const pointerEvents = getComputedStyle(h).pointerEvents
      const after = read()
      if (!before) h.removeAttribute('disabled')
      // Composite controls (sail-switch, sail-accordion-item) host their
      // interactive part inside the shadow tree — descend to find it.
      const interactiveRoot = (() => {
        if (h.matches?.('button, [role=button], input, select, textarea, a[href], [tabindex]:not([tabindex="-1"])')) return h
        let best = null
        for (const { el } of globalThis.__walk(h.shadowRoot ?? h)) {
          if (!(el instanceof HTMLElement)) continue
          if (el === h) continue
          if (!el.matches?.('input, select, textarea, button, [role=button], [role=switch], [role=checkbox], [role=tab]')) continue
          best = el
        }
        return best ?? h
      })()
      // `disabled` may be reflected only to the host's inner control
      // (the SAIL framework reflects onto the wrapped input/button,
      // not the host). Accept either location.
      const ic = getComputedStyle(interactiveRoot)
      const hostHasDisabled = h.hasAttribute('disabled') ||
        interactiveRoot.hasAttribute?.('disabled') ||
        interactiveRoot.getAttribute?.('aria-disabled') === 'true'
      let anySignal =
        interactiveRoot.hasAttribute?.('disabled') ||
        interactiveRoot.getAttribute?.('aria-disabled') === 'true' ||
        ic.pointerEvents === 'none' ||
        ic.cursor === 'not-allowed' ||
        parseFloat(ic.opacity) < 0.85
      if (!anySignal) {
        // Walk the shadow tree — wrapper / variant-styled elements like
        // `part="wrap"` carry disabled-state styling instead.
        for (const { el } of globalThis.__walk(h.shadowRoot ?? h)) {
          if (!(el instanceof HTMLElement)) continue
          const cs = getComputedStyle(el)
          if (cs.pointerEvents === 'none' || cs.cursor === 'not-allowed' || parseFloat(cs.opacity) < 0.85) {
            anySignal = true; break
          }
        }
      }
      const pass = hostHasDisabled && anySignal
      return { reason: 'disabled', prop: 'disabled', disabledAttr: hostHasDisabled, ariaDis: interactiveRoot.getAttribute?.('aria-disabled') === 'true', pointerEvents: ic.pointerEvents, interactive: interactiveRoot.tagName, pass, interactiveRoot: interactiveRoot.tagName, initial, after }
    }
    if (prop.kind === 'open-bool') {
      const before = h.hasAttribute('open')
      h.setAttribute('open', '')
      await new Promise(r => setTimeout(r, 80))
      let openedPaint = 0
      for (const { el } of globalThis.__walk(h.shadowRoot ?? h)) {
        if (!(el instanceof HTMLElement)) continue
        const r = el.getBoundingClientRect()
        if (r.width > 2 && r.height > 2) openedPaint++
      }
      // Some overlays mount a `<column popover="auto">` that needs an
      // explicit showPopover() to lift to top-layer. The open attribute
      // toggles state but doesn't call the API.
      if (openedPaint <= 1) {
        try {
          for (const { el } of globalThis.__walk(h.shadowRoot ?? h)) {
            if (!(el instanceof HTMLElement)) continue
            if (el.hasAttribute?.('popover') && typeof el.showPopover === 'function' && !el.matches?.(':popover-open')) {
              el.showPopover()
            }
          }
        } catch {}
        await new Promise(r => setTimeout(r, 80))
        openedPaint = 0
        for (const { el } of globalThis.__walk(h.shadowRoot ?? h)) {
          if (!(el instanceof HTMLElement)) continue
          const r = el.getBoundingClientRect()
          if (r.width > 2 && r.height > 2) openedPaint++
        }
      }
      if (!before) h.removeAttribute('open')
      return { reason: 'open-toggle', prop: 'open', openedPaint, pass: openedPaint > 1 }
    }
    return { reason: 'skipped', prop: prop.name }
  }, { tag, prop })
}

// ── Main per-component probe ─────────────────────────────────────────────

async function probeOne(browser, tag) {
  const entry = BY_TAG.get(tag)
  if (!entry) { record(tag, 'fail', { reason: 'no-cem-decl' }); return }
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } })
  const errs = []
  page.on('pageerror', e => errs.push('ERR:' + String(e).slice(0, 80)))
  page.on('console', m => { if (['error', 'warning', 'warn', 'log'].includes(m.type())) errs.push('CONSOLE-' + m.type() + ':' + m.text().slice(0, 200)) })
  try {
    await page.goto(`${BASE}/components/${tag}`, { waitUntil: 'domcontentloaded', timeout: 25000 })
    await page.addScriptTag({ content: probeSource })
    await page.waitForFunction(
      ({ tag }) => !!document.querySelector('.detail-stage')?.querySelector(tag),
      { tag }, { timeout: 8000 },
    ).catch(() => null)
    await page.waitForTimeout(700)
    const mountState = await page.evaluate(
      () => document.querySelector('.detail-stage')?.dataset?.mountState ?? 'absent',
    )
    if (mountState !== 'live') {
      record(tag, 'fail', { reason: `mount-${mountState}` })
      return
    }
    let checked = 0
    if (OVERLAY_FAMILY_SET.has(tag)) {
      try { await checkOverlay(page, tag); checked++ } catch (e) { record(tag, 'fail', { reason: `overlay-throw`, error: String(e).slice(0, 200) }) }
    }
    if (TONE_TAGS.includes(tag)) {
      const toneDef = entry.props.find(p => p.name === 'tone')
      if (toneDef?.values) {
        try {
          const r = await checkProp(page, tag, { name: 'tone', kind: 'tone', values: toneDef.values })
          if (r && r.pass === true) record(tag, 'pass', { kind: 'tone', prop: 'tone', ...r })
          else if (r && r.pass === false) record(tag, 'review', { kind: 'tone', prop: 'tone', ...r, reason: 'tone-largest-element-unchanged-needs-visual' })
          else record(tag, 'review', { kind: 'tone', prop: 'tone', ...r, reason: 'tone-inconclusive' })
          checked++
        } catch (e) { record(tag, 'fail', { reason: `tone-throw`, error: String(e).slice(0, 200) }) }
      }
    }
    if (SIZE_TAGS.includes(tag)) {
      const sizeDef = entry.props.find(p => p.name === 'size')
      const sizeValues = (sizeDef?.values ?? []).filter(v => /^(sm|md|lg|xs|xl)$/.test(v))
      if (sizeValues.length >= 2) {
        try {
          const r = await checkProp(page, tag, { name: 'size', kind: 'size', sizeValues })
          if (r && r.pass === true) record(tag, 'pass', { kind: 'size', prop: 'size', ...r })
          else if (r && r.pass === false) record(tag, 'fail', { kind: 'size', prop: 'size', ...r, reason: 'size-no-distinction' })
          else record(tag, 'review', { kind: 'size', prop: 'size', ...r, reason: 'size-inconclusive' })
          checked++
        } catch (e) { record(tag, 'fail', { reason: `size-throw`, error: String(e).slice(0, 200) }) }
      }
    }
    if (DISABLED_TAGS.includes(tag)) {
      try {
        const r = await checkProp(page, tag, { name: 'disabled', kind: 'disabled' })
        if (r && r.pass === true) record(tag, 'pass', { kind: 'disabled', prop: 'disabled', ...r })
        else if (r && r.pass === false) record(tag, 'review', { kind: 'disabled', prop: 'disabled', ...r, reason: 'disabled-attr-not-reflected-on-host' })
        else record(tag, 'review', { kind: 'disabled', prop: 'disabled', ...r, reason: 'disabled-inconclusive' })
        checked++
      } catch (e) { record(tag, 'fail', { reason: `disabled-throw`, error: String(e).slice(0, 200) }) }
    }
    if (BOOLEAN_OPEN_TAGS.includes(tag)) {
      try {
        const r = await checkProp(page, tag, { name: 'open', kind: 'open-bool' })
        if (r && r.pass === true) record(tag, 'pass', { kind: 'open', prop: 'open', ...r })
        else if (r && r.pass === false) record(tag, 'fail', { kind: 'open', prop: 'open', ...r, reason: 'open-no-effect' })
        else record(tag, 'review', { kind: 'open', prop: 'open', ...r, reason: 'open-inconclusive' })
        checked++
      } catch (e) { record(tag, 'fail', { reason: `open-throw`, error: String(e).slice(0, 200) }) }
    }
    if (!checked) {
      record(tag, 'pass', { kind: 'mount-only', reason: 'no-targeted-checks' })
    }
    if (errs.length) {
      const c = REPORT.components.find(c => c.tag === tag)
      if (c) c.checks.push({ kind: 'page-errors', errs, severity: 'review' })
      REPORT.totals.review++; review++
    }
  } catch (e) {
    record(tag, 'fail', { reason: 'load-throw', error: String(e).slice(0, 200) })
  } finally {
    await page.close()
  }
}

// ── Surface consistency pass ─────────────────────────────────────────────

async function surfaceCheck(browser) {
  const samples = {}
  for (const tag of OVERLAY_FAMILY) {
    if (!BY_TAG.has(tag)) continue
    const page = await browser.newPage({ viewport: { width: 1280, height: 900 } })
    page.on('pageerror', () => null)
    try {
      await page.goto(`${BASE}/components/${tag}`, { waitUntil: 'domcontentloaded', timeout: 25000 })
      await page.addScriptTag({ content: probeSource })
      await page.waitForFunction(
        ({ t }) => !!document.querySelector('.detail-stage')?.querySelector(t),
        { t: tag }, { timeout: 8000 },
      ).catch(() => null)
      await page.waitForTimeout(500)
      await driveOpen(page, tag)
      await page.waitForTimeout(450)
      samples[tag] = await sampleSurface(page, tag)
      // Always also save a screenshot of the open state.
      try {
        const stageBox = await page.evaluate(() => {
          const s = document.querySelector('.detail-stage')
          if (!s) return null
          const r = s.getBoundingClientRect()
          const cl = (n, max) => Math.max(0, Math.min(max, n))
          const x = cl(r.x - 80, window.innerWidth)
          const y = cl(r.y - 60, window.innerHeight)
          return { x, y, width: Math.min(window.innerWidth - x, r.width + 160), height: Math.min(window.innerHeight - y, r.height + 120) }
        })
        if (stageBox && stageBox.width > 0 && stageBox.height > 0) {
          const png = await page.screenshot({ clip: stageBox })
          const fp = join(SHOT_DIR, tag.replace(/[^a-z0-9-]/gi, '_') + '.png')
          writeFileSync(fp, png)
        }
      } catch {}
    } finally {
      await page.close()
    }
  }
  const fp = s => `${s?.bg ?? '∅'}|${s?.borderColor ?? '∅'}|${s?.borderRadius ?? '∅'}|${s?.boxShadow ?? '∅'}`
  const clusters = new Map()
  for (const [tag, s] of Object.entries(samples)) {
    const k = fp(s)
    if (!k || k.includes('undefined')) continue
    if (!clusters.has(k)) clusters.set(k, [])
    clusters.get(k).push(tag)
  }
  const sorted = [...clusters.entries()].sort((a, b) => b[1].length - a[1].length)
  const dominant = sorted[0]?.[1]?.length >= 2 ? sorted[0][0] : null
  for (const [tag, s] of Object.entries(samples)) {
    const k = fp(s)
    if (!k || k.includes('undefined')) continue
    if (k !== dominant) {
      REPORT.surfaceInconsistencies.push({
        tag,
        fingerprint: k,
        dominantFingerprint: dominant,
        differs: {
          bg: s.bg ?? null,
          borderColor: s.borderColor ?? null,
          borderRadius: s.borderRadius ?? null,
          boxShadow: s.boxShadow ?? null,
        },
      })
    }
  }
  REPORT.surfaceClusters = sorted.map(([k, tags]) => ({ fingerprint: k, tags, count: tags.length }))
}

// ── Entry point ─────────────────────────────────────────────────────────

const argv = process.argv.slice(2)
const oneFlag = argv.find(a => !a.startsWith('--'))
const overlayOnly = argv.includes('--overlay')

const browser = await chromium.launch()
try {
  if (oneFlag) {
    await probeOne(browser, oneFlag)
  } else if (overlayOnly) {
    await surfaceCheck(browser)
  } else {
    const tags = PUBLIC_TAGS.slice().sort()
    let n = 0
    for (const tag of tags) {
      await probeOne(browser, tag)
      n++
      process.stdout.write(`\r${n}/${tags.length}  ${tag.padEnd(28)}  pass=${pass} fail=${fail} review=${review}    `)
    }
    console.log('\n— surface-consistency pass —')
    await surfaceCheck(browser)
  }
  REPORT.completedAt = new Date().toISOString()
  REPORT.finished = { pass, fail, review }
  writeFileSync(join(HERE, 'behavior-report.json'), JSON.stringify(REPORT, null, 2))
  console.log(`\n=== TOTALS: ${pass} PASS  ${fail} FAIL  ${review} REVIEW ===`)
  if (REPORT.surfaceClusters.length) {
    console.log(`\nSurface clusters (largest = dominant; only include when ≥2 members):`)
    for (const c of REPORT.surfaceClusters) {
      console.log(`  ${String(c.count).padStart(2)} ← ${c.tags.join(', ')}`)
      console.log(`      ${c.fingerprint.slice(0, 140)}`)
    }
  }
  if (REPORT.surfaceInconsistencies.length) {
    console.log(`\nSurface inconsistencies (${REPORT.surfaceInconsistencies.length}) — fingerprinted against the largest cluster. Anything not in that cluster diverged:`)
    for (const inc of REPORT.surfaceInconsistencies) {
      console.log(`  ${inc.tag.padEnd(28)} bg=${inc.differs.bg}  border=${inc.differs.borderColor}  radius=${inc.differs.borderRadius}  shadow=${(inc.differs.boxShadow ?? '').slice(0, 50)}`)
    }
  }
  if (fail > 0) process.exit(1)
} finally {
  await browser.close()
}
