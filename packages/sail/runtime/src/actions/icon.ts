// `use:icon` — glyph resolution for the L1 <icon> element.
//
// Resolution order: `src` (arbitrary URL) wins over `name`; names fetch from
// the host's icon endpoint (deck serves the embedded lucide set at
// `/_deck/icons/{name}.svg`). Fetches are cached and single-flight per key;
// `primeIconCache` seeds glyphs without a network (tests, SSR-less hosts).
// `label` makes the element role=img with an aria-label; label-less icons are
// aria-hidden. A failed resolve removes any injected glyph so the slotted
// fallback shows.
//
// `spin` is owned here (not contracts PE): the glyph is injected with no
// structure slot, so the action attaches a motion:loop rotate on the host
// when `[spin]` is present. Duration matches the ambient `--animate-spin`
// 0.7s; reduced motion settles via the motion runtime (no spin).

import { __sui_motion_attach } from '../motion'

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
    // Host rotate spins the injected glyph (no structure slot for the svg).
    // 0.7s matches the ambient --animate-spin catalog entry the PE used.
    spinDispose = __sui_motion_attach(node, {
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
    // Glyph sizing is `.sail-icon svg` in the L1 skin — no class stamp.
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

    const key = src ? `src:${src}` : name ? `name:${name}` : ''
    if (!key) {
      drop()
      return
    }
    // Primed lucide glyphs are stored by bare name; fetches key composite.
    let cached = cache.get(key)
    if (cached === undefined && !src) cached = cache.get(name)
    if (cached !== undefined) {
      if (cached) inject(cached, key)
      else drop()
      return
    }
    const ticket = ++seq
    const url = src || `${ENDPOINT}${encodeURIComponent(name)}.svg`
    void fetchKey(url, key).then((svg) => {
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
