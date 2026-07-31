// `use:icon` — glyph resolution for the L1 <icon> element.
//
// Resolution order (TOOL-10):
//   1. Primed cache / prior fetch (tests, SSR hosts)
//   2. Local asset `/_deck/icons/{name}.svg` (compile-emitted used glyphs)
//   3. Same path served by the daemon embed in dev
//   4. Slotted fallback (failed resolve removes any injected glyph)
//
// Named icons NEVER use a CDN. The `src` attribute is only for **relative**
// or **same-origin** custom SVG URLs (not a pack escape hatch).

import { __sail_motion_attach } from '../motion'

const ENDPOINT = '/_deck/icons/'

// name → svg markup; null caches a miss so a broken host never refetches.
const cache = new Map<string, string | null>()
const inflight = new Map<string, Promise<string | null>>()

export function primeIconCache(glyphs: Record<string, string>): void {
  for (const [name, svg] of Object.entries(glyphs)) cache.set(name, svg)
}

function fetchKey(url: string, key: string): Promise<string | null> {
  const pending = inflight.get(key)
  if (pending) return pending
  const p = fetch(url)
    .then((r) => (r.ok ? r.text() : null))
    .catch(() => null)
    .then((svg) => {
      cache.set(key, svg)
      inflight.delete(key)
      return svg
    })
  inflight.set(key, p)
  return p
}

/**
 * `src=` allowlist: relative paths, same-origin absolute URLs, and
 * data/blob SVG only. Cross-origin http(s) is always rejected (no CDN).
 */
export function isAllowedIconSrc(url: string): boolean {
  const u = url.trim()
  if (!u) return false
  // Relative / root-relative / protocol-relative blocked (//cdn.example).
  if (u.startsWith('//')) return false
  if (u.startsWith('/') || u.startsWith('./') || u.startsWith('../')) return true
  if (u.startsWith('blob:')) return true
  if (u.startsWith('data:image/svg')) return true
  if (!/^https?:\/\//i.test(u)) {
    // Bare filename relative to document — treat as local.
    return !u.includes(':')
  }
  try {
    const base =
      typeof globalThis !== 'undefined' &&
      'location' in globalThis &&
      (globalThis as { location?: { href?: string } }).location?.href
        ? (globalThis as { location: { href: string } }).location.href
        : 'http://localhost/'
    const parsed = new URL(u, base)
    if (typeof globalThis !== 'undefined' && 'location' in globalThis) {
      const loc = (globalThis as { location?: { origin?: string } }).location
      if (loc?.origin && parsed.origin === loc.origin) return true
    }
    return false
  } catch {
    return false
  }
}

function parseSvg(doc: Document, markup: string): SVGSVGElement | null {
  const parsed = new DOMParser().parseFromString(markup, 'image/svg+xml')
  const root = parsed.documentElement
  if (root.nodeName.toLowerCase() !== 'svg') return null
  return doc.adoptNode(root) as unknown as SVGSVGElement
}

export const icon = (node: Element) => {
  let svgEl: SVGSVGElement | null = null
  let lastKey: string | null = null
  let seq = 0
  let spinDispose: (() => void) | null = null

  const syncAria = (): void => {
    const label = node.getAttribute('label') ?? ''
    if (label) {
      node.setAttribute('role', 'img')
      node.setAttribute('aria-label', label)
      node.removeAttribute('aria-hidden')
    } else {
      node.setAttribute('aria-hidden', 'true')
      node.removeAttribute('role')
      node.removeAttribute('aria-label')
    }
    if (svgEl) {
      if (label) svgEl.removeAttribute('aria-hidden')
      else svgEl.setAttribute('aria-hidden', 'true')
    }
  }

  const syncSpin = (): void => {
    spinDispose?.()
    spinDispose = null
    if (!node.hasAttribute('spin')) return
    spinDispose = __sail_motion_attach(node, {
      trigger: 'loop',
      motions: [
        {
          property: 'rotate',
          preset: 'spin',
          kind: 'tween',
          from: 0,
          to: 360,
          duration: 0.7,
          easing: 'linear',
          iterations: Infinity,
          direction: 'normal',
        },
      ],
    })
  }

  const drop = (): void => {
    if (svgEl) {
      svgEl.remove()
      svgEl = null
    }
    lastKey = null
  }

  const inject = (markup: string, key: string): void => {
    if (key === lastKey) return
    const parsed = parseSvg(node.ownerDocument, markup)
    if (!parsed) return
    if (svgEl) svgEl.replaceWith(parsed)
    else node.insertBefore(parsed, node.firstChild)
    svgEl = parsed
    lastKey = key
    syncAria()
  }

  const render = (): void => {
    syncAria()
    syncSpin()
    const src = node.getAttribute('src') ?? ''
    const name = node.getAttribute('name') ?? ''

    // Named icons always resolve via local/daemon asset path — never CDN.
    if (name && !src) {
      const key = `name:${name}`
      let cached = cache.get(key)
      if (cached === undefined) cached = cache.get(name)
      if (cached !== undefined) {
        if (cached) inject(cached, key)
        else drop()
        return
      }
      const ticket = ++seq
      const url = `${ENDPOINT}${encodeURIComponent(name)}.svg`
      void fetchKey(url, key).then((svg) => {
        if (ticket !== seq) return
        if (svg) inject(svg, key)
        else drop()
      })
      return
    }

    if (!src) {
      drop()
      return
    }

    if (!isAllowedIconSrc(src)) {
      drop()
      return
    }

    const key = `src:${src}`
    const cached = cache.get(key)
    if (cached !== undefined) {
      if (cached) inject(cached, key)
      else drop()
      return
    }
    const ticket = ++seq
    void fetchKey(src, key).then((svg) => {
      if (ticket !== seq) return
      if (svg) inject(svg, key)
      else drop()
    })
  }

  const mo = new MutationObserver(render)
  mo.observe(node, { attributes: true, attributeFilter: ['name', 'src', 'label', 'spin'] })
  render()
  return {
    destroy: () => {
      mo.disconnect()
      spinDispose?.()
      spinDispose = null
    },
  }
}
