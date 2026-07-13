// `use:icon` — glyph resolution for the L1 <icon> element.
//
// Resolution order: `src` (arbitrary URL) wins over `name`. `set="app"` maps
// app identity names to their lucide glyphs (APP_GLYPH/APP_ALIASES); plain
// names fetch from the host's icon endpoint (deck serves the embedded lucide
// set at `/_deck/icons/{name}.svg`). Fetches are cached and single-flight per
// key; `primeIconCache` seeds glyphs without a network (tests, SSR-less
// hosts). `label` makes the element role=img with an aria-label; label-less
// icons are aria-hidden. A failed resolve removes any injected glyph so the
// slotted fallback shows.

type AppName =
  | 'dashboard' | 'pipelines' | 'agents' | 'skills' | 'git'
  | 'collections' | 'canvas' | 'settings' | 'chat' | 'events'
  | 'specs' | 'theme' | 'setup' | 'overview' | 'mesh'

const APP_GLYPH: Record<AppName, string> = {
  dashboard: 'layout-dashboard',
  pipelines: 'workflow',
  agents: 'bot',
  skills: 'sparkles',
  git: 'git-branch',
  collections: 'database',
  canvas: 'frame',
  settings: 'settings',
  chat: 'messages-square',
  events: 'radio',
  specs: 'file-text',
  theme: 'palette',
  setup: 'rocket',
  overview: 'compass',
  mesh: 'network',
}

const APP_ALIASES: Record<string, AppName> = {
  'theme-studio': 'theme',
}

const APP_NAMES = new Set([...Object.keys(APP_GLYPH), ...Object.keys(APP_ALIASES)])

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
    // The daemon emits stroke="currentColor", so host CSS color flows through.
    parsed.setAttribute('class', 'sui-icon-svg')
    const target = node.querySelector('.frame') ?? node
    if (svgEl) svgEl.replaceWith(parsed)
    else target.insertBefore(parsed, target.firstChild)
    svgEl = parsed
    lastKey = key
    syncAria()
  }

  const render = (): void => {
    syncAria()
    const src = node.getAttribute('src') ?? ''
    const rawName = node.getAttribute('name') ?? ''
    const set = node.getAttribute('set') ?? ''

    let name = rawName
    if (set === 'app' && rawName) {
      const resolved = APP_ALIASES[rawName] ?? (rawName as AppName)
      name = APP_NAMES.has(rawName) ? APP_GLYPH[resolved] : 'dot'
    }

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
  mo.observe(node, { attributes: true, attributeFilter: ['name', 'src', 'set', 'label'] })
  render()
  return {
    destroy: () => mo.disconnect(),
  }
}
