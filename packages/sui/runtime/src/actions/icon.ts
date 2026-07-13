// `use:icon` — glyph resolution for the L1 <icon name="…"> element. Fetches
// the named SVG from the host's icon endpoint (deck serves the embedded
// lucide set at `/_deck/icons/{name}.svg`), caches per name, and injects it
// as the element's glyph. A name-less <icon> keeps its slotted children.
// `primeIconCache` seeds glyphs without a network (tests, SSR-less hosts).

const cache = new Map<string, string>()

const ENDPOINT = '/_deck/icons/'

export function primeIconCache(glyphs: Record<string, string>): void {
  for (const [name, svg] of Object.entries(glyphs)) cache.set(name, svg)
}

async function resolve(name: string): Promise<string> {
  const hit = cache.get(name)
  if (hit != null) return hit
  let svg = ''
  try {
    const res = await fetch(`${ENDPOINT}${encodeURIComponent(name)}.svg`)
    if (res.ok) svg = await res.text()
  } catch {
    // Unreachable endpoint renders an empty glyph; the name stays cached so a
    // broken host doesn't refetch per element.
  }
  cache.set(name, svg)
  return svg
}

export const icon = (node: Element) => {
  let seq = 0
  const render = (): void => {
    const name = node.getAttribute('name') ?? ''
    if (!name) return
    const ticket = ++seq
    void resolve(name).then((svg) => {
      if (ticket !== seq || !svg) return
      node.innerHTML = svg
      const el = node.firstElementChild
      if (el) el.setAttribute('aria-hidden', 'true')
    })
  }
  const mo = new MutationObserver(render)
  mo.observe(node, { attributes: true, attributeFilter: ['name'] })
  render()
  return {
    destroy: () => mo.disconnect(),
  }
}
